use crate::cli::ComponentAction;
use crate::model::Component;
use crate::storage::{COMPONENTS_PATH, append_csv, get_rows};

pub fn run(action: ComponentAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ComponentAction::Add { id, name } => {
            let new_component = Component { id, name };
            Ok(append_csv(COMPONENTS_PATH, &new_component)?)
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
