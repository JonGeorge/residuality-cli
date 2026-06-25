use crate::cli::ComponentAction;
use crate::storage::{append_csv, COMPONENTS_PATH};
use crate::model::Component;

pub fn run(action: ComponentAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ComponentAction::Add { id, name } => {
            let new_component = Component { id, name };
            Ok(append_csv(COMPONENTS_PATH, &new_component)?)
        },
    }
}
