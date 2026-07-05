use crate::{
    model::{Component, Stressor},
    storage::{
        COMPONENTS_PATH, STRESSORS_PATH, get_header_row_from_struct, write_rows_to_path_from_vec,
    },
};
use std::{collections::BTreeSet, path::Path};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut something_changed = false;

    let component_path: &Path = Path::new(COMPONENTS_PATH);
    if let Some(dir) = component_path.parent() {
        if !Path::exists(dir) {
            std::fs::create_dir_all(dir)?;
            println!("Created path '{}'", dir.to_string_lossy());
            something_changed = true;
        }

        if !Path::exists(component_path) {
            let header = get_header_row_from_struct(&Component {
                id: String::new(),
                name: None,
            })?;

            write_rows_to_path_from_vec(COMPONENTS_PATH, vec![header])?;
            println!(
                "Created component file '{}'",
                component_path.to_string_lossy()
            );
            something_changed = true;
        }
    }

    let stressor_path: &Path = Path::new(STRESSORS_PATH);
    if let Some(dir) = stressor_path.parent() {
        if !Path::exists(dir) {
            std::fs::create_dir_all(dir)?;
            println!("Created path '{}'", dir.to_string_lossy());
            something_changed = true;
        }

        if !Path::exists(stressor_path) {
            let header = get_header_row_from_struct(&Stressor {
                id: None,
                name: None,
                detection: None,
                attractor: None,
                business_reaction: None,
                technical_change: None,
                affected_components: BTreeSet::new(),
            })?;

            write_rows_to_path_from_vec(STRESSORS_PATH, vec![header])?;
            println!(
                "Created stressor file '{}'",
                stressor_path.to_string_lossy()
            );
            something_changed = true;
        }
    }

    if something_changed {
        println!("You're good to go!");
    } else {
        println!("No changes made");
    }

    Ok(())
}
