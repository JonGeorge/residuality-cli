// storage.rs — file reading/writing, separated from main's command dispatch.

use std::fs::File;
use crate::model::{Component, Stressor};
use std::io::{Read, Seek, SeekFrom, Write};

pub const COMPONENTS_PATH: &str = "architecture/components.csv";
pub const STRESSORS_PATH: &str = "architecture/stressors.csv";

pub fn load_components_csv() -> Result<Vec<Component>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(COMPONENTS_PATH)?;
    let components = reader
        .deserialize()
        .collect::<Result<Vec<Component>, _>>()?;
    Ok(components)
}

pub fn load_stressors_csv() -> Result<Vec<Stressor>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(STRESSORS_PATH)?;
    let stressors = reader.deserialize().collect::<Result<Vec<Stressor>, _>>()?;
    Ok(stressors)
}

pub fn add_component(id: String, name: String) -> Result<(), Box<dyn std::error::Error>> {
    let new_component = Component { id, name };

    let file = std::fs::OpenOptions::new()
        .read(true)
        .append(true)
        .open(COMPONENTS_PATH);

    match file {
        Ok(mut file) => {
            // check last char on existing csv for line break
            if file.metadata()?.len() > 0 { // file has data
                ensure_last_char_is_new_line(&mut file)?;
                append_row(&file, &new_component, false)?;
            } else if file.metadata()?.len() == 0 { // file has no data
                append_row(&file, &new_component, true)?;
            }
        },
        Err(e) => match e.kind() {
            // File does not exist, create a new one
            std::io::ErrorKind::NotFound => {
                create_component_csv(&new_component)?;
            }
            _ => {
                return Err(e.into());
            }
        },
    }

    Ok(())
}

fn append_row(file: &File, component: &Component, has_headers: bool) -> std::io::Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(has_headers)
        .from_writer(file);

    writer.serialize(component)?;
    writer.flush()
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

fn create_component_csv(component: &Component) -> std::io::Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(COMPONENTS_PATH)?;

    writer.serialize(component)?;
    writer.flush()
}
