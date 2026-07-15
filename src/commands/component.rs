use std::path::Path;

use inquire::ui::{RenderConfig, Styled};
use inquire::{InquireError, Text};

use crate::cli::ComponentAction;
use crate::commands::check::{IdToCheckIsFrom, check_component};
use crate::model::Component;
use crate::storage::{COMPONENTS_PATH, append_csv, get_rows, is_missing_file_err};

pub fn run(action: ComponentAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ComponentAction::Add { id, name } => {
            let no_args_provided = id.is_none() && name.is_none();

            if no_args_provided {
                inquire::set_global_render_config(
                    RenderConfig::default().with_canceled_prompt_indicator(
                        Styled::new("< exited >").with_fg(inquire::ui::Color::DarkRed),
                    ),
                );
                prompt_for_components()?;
                Ok(())
            } else {
                let id = id.unwrap_or_default();
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
        }
        ComponentAction::List => {
            let components: Vec<Component> = match get_rows(COMPONENTS_PATH) {
                Ok(c) => c,
                Err(e) if is_missing_file_err(e.as_ref()) => {
                    eprintln!("Component file not found");
                    Vec::new()
                }
                Err(e) => return Err(e),
            };
            for component in components {
                println!("{}", component);
            }
            Ok(())
        }
    }
}

fn prompt_for_components() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let components: Vec<Component> = match get_rows(COMPONENTS_PATH) {
            Ok(c) => c,
            Err(e) if is_missing_file_err(e.as_ref()) => Vec::new(),
            Err(e) => return Err(e),
        };

        let id = match Text::new("ID:").with_help_message("Esc to quit").prompt() {
            Ok(id) => id,
            Err(InquireError::OperationCanceled) => break,
            Err(e) => return Err(e.into()),
        };

        let name = match Text::new("Name:").with_help_message("Esc to quit").prompt() {
            Ok(n) => n,
            Err(InquireError::OperationCanceled) => break,
            Err(e) => return Err(e.into()),
        };

        let new_component = Component {
            id,
            name: Some(name),
        };

        if let Some(issue) =
            check_component(&new_component, &components, IdToCheckIsFrom::CommandLine)
        {
            eprintln!("{issue}\n");
        } else {
            append_csv(COMPONENTS_PATH, &new_component)?;
            println!("Saved '{}'\n", new_component.id);
        }
    }
    Ok(())
}
