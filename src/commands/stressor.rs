use std::collections::BTreeSet;

use inquire::{
    InquireError, MultiSelect, Text,
    ui::{RenderConfig, Styled},
};

use crate::{
    cli::StressorAction,
    model::{Component, Stressor},
    storage::{COMPONENTS_PATH, STRESSORS_PATH, append_csv, get_rows, is_missing_file_err},
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
            let no_args_provided = id.is_none()
                && name.is_none()
                && detection.is_none()
                && attractor.is_none()
                && business_reaction.is_none()
                && technical_change.is_none()
                && affected_components.is_empty();

            if no_args_provided {
                inquire::set_global_render_config(
                    RenderConfig::default().with_canceled_prompt_indicator(
                        Styled::new("< exited >").with_fg(inquire::ui::Color::DarkRed),
                    ),
                );
                prompt_for_stressors()?;
                Ok(())
            } else {
                let new_id = if id.is_none() {
                    Some(get_next_stressor_id()?)
                } else {
                    id
                };

                let new_stressor = Stressor {
                    id: new_id,
                    name,
                    detection,
                    attractor,
                    business_reaction,
                    technical_change,
                    affected_components: BTreeSet::from_iter(affected_components),
                };
                Ok(append_csv(STRESSORS_PATH, &new_stressor)?)
            }
        }

        StressorAction::List => {
            let stressors: Vec<Stressor> = match get_rows(STRESSORS_PATH) {
                Ok(s) => s,
                Err(e) if is_missing_file_err(e.as_ref()) => {
                    eprintln!("Stressor file not found");
                    Vec::new()
                }
                Err(e) => return Err(e),
            };

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
    let raw_components: Vec<Component> = match get_rows(COMPONENTS_PATH) {
        Ok(s) => s,
        Err(e) if is_missing_file_err(e.as_ref()) => Vec::new(),
        Err(e) => return Err(e),
    };
    let components: Vec<&Component> = raw_components.iter().collect();

    if components.is_empty() {
        return Err("no components yet — add some first, the affects picker needs them".into());
    }

    loop {
        let name = match Text::new("Name:").with_help_message("Esc to quit").prompt() {
            Err(InquireError::OperationCanceled) => break,
            Err(e) => return Err(e.into()),
            Ok(n) => n,
        };

        let detection = match Text::new("Detection:")
            .with_help_message("Enter to skip, Esc to quit")
            .prompt_skippable()
        {
            Err(e) => return Err(e.into()),
            Ok(n) => match &n {
                None => break,
                Some(_) => n,
            },
        };

        let attractor = match Text::new("Attractor:")
            .with_help_message("Enter to skip, Esc to quit")
            .prompt_skippable()
        {
            Err(e) => return Err(e.into()),
            Ok(n) => match &n {
                None => break,
                Some(_) => n,
            },
        };

        let business_reaction = match Text::new("Business reaction:")
            .with_help_message("Enter to skip, Esc to quit")
            .prompt_skippable()
        {
            Err(e) => return Err(e.into()),
            Ok(n) => match &n {
                None => break,
                Some(_) => n,
            },
        };

        let technical_change = match Text::new("Technical change:")
            .with_help_message("Enter to skip, Esc to quit")
            .prompt_skippable()
        {
            Err(e) => return Err(e.into()),
            Ok(n) => match &n {
                None => break,
                Some(_) => n,
            },
        };

        let selected_affected_components = MultiSelect::new("Affects: ", components.clone())
            .with_page_size(10)
            .with_help_message(
                "Type to filter, Space to toggle, Enter to confirm, Esc to cancel entire stressor",
            )
            .prompt();

        let affected_components = match selected_affected_components {
            Ok(c) => c,
            Err(InquireError::OperationCanceled) => break,
            Err(e) => return Err(e.into()),
        };

        let next_id = get_next_stressor_id()?;
        let new_stressor = Stressor {
            id: Some(next_id.clone()),
            name: Some(name),
            detection,
            technical_change,
            business_reaction,
            attractor,
            affected_components: affected_components.iter().map(|c| c.id.clone()).collect(),
        };

        append_csv(STRESSORS_PATH, &new_stressor)?;
        println!("Saved #{}\n", next_id);
    }

    Ok(())
}

fn get_next_stressor_id() -> Result<String, Box<dyn std::error::Error>> {
    let stressors: Vec<Stressor> = match get_rows(STRESSORS_PATH) {
        Ok(s) => s,
        Err(e) if is_missing_file_err(e.as_ref()) => Vec::new(),
        Err(e) => return Err(e),
    };

    let max_id = stressors
        .iter()
        .filter_map(|s| s.id.as_deref()?.strip_prefix("S")?.parse::<u32>().ok())
        .max()
        .unwrap_or(0);

    let next_id = format!("S{}", max_id + 1);

    Ok(next_id)
}
