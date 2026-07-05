use std::collections::BTreeSet;

use inquire::{InquireError, MultiSelect, Text};

use crate::{
    cli::StressorAction,
    model::{Component, Stressor},
    storage::{COMPONENTS_PATH, STRESSORS_PATH, append_csv, get_rows},
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
        } => match &id {
            Some(_) => {
                let new_stressor = Stressor {
                    id,
                    name,
                    detection,
                    attractor,
                    business_reaction,
                    technical_change,
                    affected_components: BTreeSet::from_iter(affected_components),
                };
                Ok(append_csv(STRESSORS_PATH, &new_stressor)?)
            }
            None => {
                prompt_for_stressors()?;
                Ok(())
            }
        },

        StressorAction::List => {
            let stressors: Vec<Stressor> = get_rows(STRESSORS_PATH)?;
            for stressor in stressors {
                if let Some(n) = stressor.name {
                    println!("{}", n);
                }
            }
            Ok(())
        }
    }
}

fn prompt_for_stressors() -> Result<(), Box<dyn std::error::Error>> {
    let raw_components: Vec<Component> = get_rows(COMPONENTS_PATH)?;
    let components: Vec<&Component> = raw_components.iter().collect();

    if components.is_empty() {
        return Err("no components yet — add some first, the affects picker needs them".into());
    }

    // let labels: Vec<&str> = components.iter().map(|c| c.name.as_deref().unwrap_or(&c.id.as_str())).collect();

    loop {
        let name = match Text::new("Name: ")
            .with_help_message("Esc to quit")
            .prompt()
        {
            Err(InquireError::OperationCanceled) => break,
            Err(e) => return Err(e.into()),
            Ok(n) => n,
        };

        let detection = match Text::new("Detection:")
            .with_help_message("Enter to skip, Esc to quit")
            .prompt_skippable()
        {
            Err(InquireError::OperationCanceled) => break,
            Err(e) => return Err(e.into()),
            Ok(n) => n,
        };

        let attractor = match Text::new("Attractor:")
            .with_help_message("Enter to skip, Esc to quit")
            .prompt_skippable()
        {
            Err(InquireError::OperationCanceled) => break,
            Err(e) => return Err(e.into()),
            Ok(n) => n,
        };

        let business_reaction = match Text::new("Business reaction:")
            .with_help_message("Enter to skip, Esc to quit")
            .prompt_skippable()
        {
            Err(InquireError::OperationCanceled) => break,
            Err(e) => return Err(e.into()),
            Ok(n) => n,
        };

        let technical_change = match Text::new("Technical change:")
            .with_help_message("Enter to skip, Esc to quit")
            .prompt_skippable()
        {
            Err(InquireError::OperationCanceled) => break,
            Err(e) => return Err(e.into()),
            Ok(n) => n,
        };

        let selected_affected_components = MultiSelect::new("Affects: ", components.clone())
            .with_page_size(10)
            .with_help_message("type to filter, space to toggle, enter to confirm, esc to cancel")
            .prompt();

        let affected_components = match selected_affected_components {
            Ok(c) => c,
            Err(InquireError::OperationCanceled) => continue,
            Err(e) => return Err(e.into()),
        };

        let new_stressor = Stressor {
            id: Some(get_next_stressor_id()?),
            name: Some(name),
            detection,
            technical_change,
            business_reaction,
            attractor,
            affected_components: affected_components.iter().map(|c| c.id.clone()).collect(),
        };

        append_csv(STRESSORS_PATH, &new_stressor)?;
    }

    Ok(())
}

fn get_next_stressor_id() -> Result<String, Box<dyn std::error::Error>> {
    let stressors: Vec<Stressor> = get_rows(STRESSORS_PATH)?;

    let max_id = stressors
        .iter()
        .filter_map(|s| s.id.as_deref()?.strip_prefix("S")?.parse::<u32>().ok())
        .max()
        .unwrap_or(0);

    let next_id = format!("S{}", max_id + 1);

    Ok(next_id)
}
