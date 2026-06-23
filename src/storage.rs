// storage.rs — file reading/writing, separated from main's command dispatch.

use crate::{Component, model::Stressor};

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

// fn load_components_yml(path: &str) -> Result<Vec<Component>, Box<dyn std::error::Error>> {
//     let yaml = std::fs::read_to_string(path)?;
//     let components: Vec<Component> = serde_yaml::from_str(&yaml)?;
//     Ok(components)
// }

fn save_components(
    path: &str,
    components: &Vec<Component>,
) -> Result<(), Box<dyn std::error::Error>> {
    let yaml = serde_yaml::to_string(components)?;
    std::fs::write(path, yaml)?;
    Ok(())
}

pub fn add_component(id: String, name: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut components = load_components_csv()?;

    let new_component = Component { id, name };

    components.push(new_component);

    save_components(COMPONENTS_PATH, &components)
}
