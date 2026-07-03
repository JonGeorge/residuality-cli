use std::path::Path;

use crate::cli::ComponentAction;
use crate::commands::check::{IdToCheckIsFrom, check_component};
use crate::model::Component;
use crate::storage::{COMPONENTS_PATH, append_csv, get_rows};

pub fn run(action: ComponentAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ComponentAction::Add { id, name } => {
            let new_component = Component { id, name };
            let components: Vec<Component> = if Path::new(COMPONENTS_PATH).exists() {
                get_rows(COMPONENTS_PATH)?
            } else {
                Vec::new()
            };

            if let Some(issue) =
                check_component(&new_component, &components, IdToCheckIsFrom::CommandLine)
            {
                eprintln!("{}", issue);
                Err("could not add component".into())
            } else {
                Ok(append_csv(COMPONENTS_PATH, &new_component)?)
            }
        }
        ComponentAction::List => {
            let components: Vec<Component> = get_rows(COMPONENTS_PATH)?;
            for component in components {
                println!("{}", component.id);
            }
            Ok(())
        }
    }
}
