use crate::{
    cli::StressorAction,
    model::Stressor,
    storage::{STRESSORS_PATH, append_csv},
};

pub fn run(action: StressorAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        StressorAction::Add {
            id,
            name,
            detection,
            attractor,
            business_reaction,
            technical_change,
            affected_components,
        } => {
            let new_stressor = Stressor {
                id,
                name,
                detection,
                attractor,
                business_reaction,
                technical_change,
                affected_components,
            };

            Ok(append_csv(STRESSORS_PATH, &new_stressor)?)
        }
    }
}
