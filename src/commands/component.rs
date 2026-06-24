use crate::cli::ComponentAction;
use crate::storage::add_component;

pub fn run(action: ComponentAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ComponentAction::Add { id, name } => add_component(id, name),
    }
}
