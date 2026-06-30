// storage.rs — file reading/writing, separated from main's command dispatch.

use serde::Serialize;
use serde::de::DeserializeOwned;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

pub const COMPONENTS_PATH: &str = "architecture/components.csv";
pub const STRESSORS_PATH: &str = "architecture/stressors.csv";
pub const MATRIX_PATH: &str = "architecture/matrix.csv";

pub fn append_csv<T: Serialize>(path: &str, thing: &T) -> std::io::Result<()> {
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
                write_row(&file, thing, false)?;
            } else if file.metadata()?.len() == 0 {
                // file has no data
                write_row(&file, thing, true)?;
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

fn write_row<T: Serialize>(file: &File, row: &T, has_headers: bool) -> std::io::Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(has_headers)
        .from_writer(file);

    writer.serialize(row)?;
    writer.flush()
}

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

fn create_new_csv<T: Serialize>(path: &str, thing: &T) -> std::io::Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(path)?;

    writer.serialize(thing)?;
    writer.flush()
}
