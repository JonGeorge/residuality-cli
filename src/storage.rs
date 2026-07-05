// storage.rs — file reading/writing, separated from main's command dispatch.

use serde::Serialize;
use serde::de::DeserializeOwned;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::model::Matrix;

pub const COMPONENTS_PATH: &str = "architecture/components.csv";
pub const STRESSORS_PATH: &str = "architecture/stressors.csv";

pub fn get_matrix_path_with_date() -> String {
    let date = chrono::Local::now().format("%Y%m%d");

    format!("reports/matrix_{}.csv", date)
}

pub fn append_csv<T: Serialize>(path: &str, thing: &T) -> std::io::Result<()> {
    // Check if directory exists. Without this, file write will fail
    if let Some(path) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(path)?;
    }

    let file = std::fs::OpenOptions::new()
        .read(true)
        .append(true)
        .open(path);

    match file {
        Ok(mut file) => {
            // check last char on existing csv for line break
            if file.metadata()?.len() > 0 {
                // file has data
                ensure_last_char_is_new_line(&mut file)?;
                write_row_to_file_from_struct(&file, thing, false)?;
            } else if file.metadata()?.len() == 0 {
                // file has no data
                write_row_to_file_from_struct(&file, thing, true)?;
            }
        }
        Err(e) => match e.kind() {
            // File does not exist, create a new one
            std::io::ErrorKind::NotFound => {
                create_new_csv(path, thing)?;
            }
            _ => {
                return Err(e);
            }
        },
    }

    Ok(())
}

fn write_row_to_file_from_struct<T: Serialize>(
    file: &File,
    row: &T,
    has_headers: bool,
) -> std::io::Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(has_headers)
        .from_writer(file);

    writer.serialize(row)?;
    writer.flush()
}

pub fn write_rows_to_path_from_vec(path: &str, rows: Vec<Vec<String>>) -> std::io::Result<()> {
    if let Some(path) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(path)?;
    }

    let mut writer = csv::WriterBuilder::new().from_path(path)?;

    for row in rows {
        writer.write_record(row)?;
    }
    writer.flush()
}

pub fn get_header_row_from_struct<T: Serialize>(
    thing: &T,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(vec![]);

    wtr.serialize(thing)?;

    let bytes = wtr.into_inner()?;
    let text = String::from_utf8(bytes)?;
    let first_line = text.lines().next();

    match first_line {
        Some(line) => Ok(line.split(',').map(str::to_string).collect()),
        None => Err("No first line".into()),
    }
}

pub fn write_matrix_to_csv(path: &str, matrix: &Matrix) -> std::io::Result<()> {
    // Check if directory exists. Without this, file write will fail if path doesnt exist
    if let Some(path) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(path)?;
    }

    let mut writer = csv::WriterBuilder::new().from_path(path)?;

    // For each component add to a vector ["", component1.id, component2.id, ... ]
    writer.write_field("")?;
    writer.write_record(
        matrix
            .components
            .iter()
            .map(|c| c.name.as_deref().unwrap_or(&c.id)),
    )?;

    for (i, stressor) in matrix.stressors.iter().enumerate() {
        writer.write_field(
            stressor
                .name
                .as_deref()
                .unwrap_or(stressor.id.as_deref().unwrap()),
        )?;
        writer.write_record(
            matrix.table[i]
                .iter()
                .map(|cell| if *cell == 1 { "1" } else { "0" }),
        )?;
    }
    writer.flush()
}

/// Get deserialized rows from a path. For example, components and stressors.
pub fn get_rows<T: DeserializeOwned>(path: &str) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let things = reader.deserialize().collect::<Result<Vec<T>, _>>()?;

    Ok(things)
}

fn ensure_last_char_is_new_line(file: &mut File) -> std::io::Result<()> {
    file.seek(SeekFrom::End(-1))?;
    let mut last_char = [0u8; 1]; // 1 byte
    file.read_exact(&mut last_char)?;

    if last_char[0] != b'\n' {
        file.write_all(b"\n")?
    }

    Ok(())
}

pub fn create_new_csv<T: Serialize>(path: &str, thing: &T) -> std::io::Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(path)?;

    writer.serialize(thing)?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::model::{Component, Stressor};

    use super::*;

    #[test]
    fn builds_matrix() {
        let matrix = Matrix {
            table: vec![vec![1, 0, 1]],
            components: vec![
                Component {
                    id: "network".to_string(),
                    name: Some("Network".to_string()),
                },
                Component {
                    id: "server".to_string(),
                    name: Some("Server".to_string()),
                },
                Component {
                    id: "storage".to_string(),
                    name: Some("Storage".to_string()),
                },
            ],
            stressors: vec![Stressor {
                id: Some("apocolypse".to_string()),
                name: Some("Apocolypse".to_string()),
                detection: None,
                attractor: None,
                business_reaction: None,
                technical_change: None,
                affected_components: BTreeSet::new(),
            }],
        };

        let result = write_matrix_to_csv(
            &std::env::temp_dir()
                .join("2h995uhu24h5iu54h2iuh92ufpi4test.csv")
                .to_string_lossy(),
            &matrix,
        );

        match result {
            Ok(()) => {
                let generated_csv = std::fs::read_to_string(
                    std::env::temp_dir().join("2h995uhu24h5iu54h2iuh92ufpi4test.csv"),
                )
                .unwrap_or("".to_string());

                assert_eq!(generated_csv, ",Network,Server,Storage\nApocolypse,1,0,1\n");
            }
            Err(e) => {
                assert!(false, "Couldn't write file. {e}");
            }
        }
    }
}
