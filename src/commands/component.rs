use crate::storage::add_component;
use crate::cli::ComponentAction;

pub fn run(action: ComponentAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ComponentAction::Add { id, name } => add_component(id, name),
    }
}

