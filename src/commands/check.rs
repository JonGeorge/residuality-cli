use crate::{
    model::{Component, Stressor},
    storage::{COMPONENTS_PATH, STRESSORS_PATH, get_rows},
    views::check::print_findings,
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut findings: Vec<String> = Vec::new();

    let components: Vec<Component> = match get_rows(COMPONENTS_PATH) {
        Ok(c) => c,
        Err(e) => {
            findings.push(format!("{} {}", COMPONENTS_PATH, e));
            Vec::new()
        }
    };

    let stressors: Vec<Stressor> = match get_rows(STRESSORS_PATH) {
        Ok(s) => s,
        Err(e) => {
            findings.push(format!("{} {}", STRESSORS_PATH, e));
            Vec::new()
        }
    };

    findings.extend(check_components(&components)?);
    findings.extend(check_stressors(&stressors, &components)?);

    if findings.is_empty() {
        println!("Everything looks good!");
        Ok(())
    } else {
        print_findings(findings);
        Err("check failed".into())
    }
}

fn check_components(components: &[Component]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut findings: Vec<String> = Vec::new();

    for (i, c) in components.iter().enumerate() {
        if let Some(issue) = check_component(c, components, IdToCheckIsFrom::ExistingList) {
            findings.push(format!("{} row {}- {}", COMPONENTS_PATH, i + 2, issue));
        }
    }

    Ok(findings)
}

pub fn check_component(
    component: &Component,
    components: &[Component],
    origin: IdToCheckIsFrom,
) -> Option<String> {
    let uniqueness_threshold = match origin {
        IdToCheckIsFrom::CommandLine => 1,
        IdToCheckIsFrom::ExistingList => 2,
    };

    // Check if id is empty
    if component.id.trim().is_empty() {
        Some("needs id".to_string())
    }
    // Check if id contains only letters, numbers, and underscores
    else if !id_chars_are_valid(&component.id) {
        Some("only numbers and letters allowed in id".to_string())
    }
    // Check if id is unique
    else if components.iter().fold(0, |acc, comp| {
        if component.id == comp.id {
            acc + 1
        } else {
            acc
        }
    }) >= uniqueness_threshold
    {
        Some(format!("id '{}' must be unique", component.id))
    }
    // Default case
    else {
        None
    }
}

fn check_stressors(
    stressors: &[Stressor],
    components: &[Component],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut findings: Vec<String> = Vec::new();

    for (i, stressor) in stressors.iter().enumerate() {
        let mut match_found = false;

        for affected_component in &stressor.affected_components {
            // Check affected component id characters
            if !id_chars_are_valid(affected_component) {
                findings.push(format!(
                        "{} row {} - only numbers and letters allowed in id for affected component '{}'",
                        STRESSORS_PATH,
                        i + 2,
                        affected_component
                    ));

                // Skip the integrity check for this id, move on to the next affected component
                continue;
            }

            // If chars look good, check referential integrity
            for component in components {
                if component.id == *affected_component {
                    match_found = true;
                    break;
                }
            }

            // If no matching component.id found, add finding
            if !match_found {
                findings.push(format!(
                    "{} row {} - affected component '{}' references non-existent component",
                    STRESSORS_PATH,
                    i + 2,
                    affected_component
                ));
            }

            // Reset match_found for next component
            match_found = false;
        }
    }

    Ok(findings)
}

fn id_chars_are_valid(id: &str) -> bool {
    id.chars().all(|ch| ch.is_alphanumeric() || ch == '_')
}

pub enum IdToCheckIsFrom {
    /// Not in the component list yet
    CommandLine,

    /// One of the list's own rows, it'll match itself once
    ExistingList,
}
