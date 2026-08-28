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

pub fn check_stressors(
    stressors: &[Stressor],
    components: &[Component],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut findings: Vec<String> = Vec::new();

    for (i, stressor) in stressors.iter().enumerate() {
        if let Some(issue) = check_stressor(
            stressor,
            stressors,
            components,
            IdToCheckIsFrom::ExistingList,
        ) {
            findings.push(format!("{} row {}- {}", STRESSORS_PATH, i + 2, issue));
        }
    }

    Ok(findings)
}

pub fn check_stressor(
    stressor: &Stressor,
    stressors: &[Stressor],
    components: &[Component],
    origin: IdToCheckIsFrom,
) -> Option<String> {
    let uniqueness_threshold = match origin {
        IdToCheckIsFrom::CommandLine => 1,
        IdToCheckIsFrom::ExistingList => 2,
    };

    // Check if id is empty
    if stressor.id.trim().is_empty() {
        Some("missing stressor id".to_string())
    }
    // Check if id contains only letters, numbers, and underscores
    else if !id_chars_are_valid(&stressor.id) {
        Some("only numbers and letters allowed in id".to_string())
    }
    // Check if id is unique
    else if stressors
        .iter()
        .fold(0, |acc, s| if stressor.id == s.id { acc + 1 } else { acc })
        >= uniqueness_threshold
    {
        Some(format!("id '{}' must be unique", stressor.id))
    }
    // Check if affected component ids
    else {
        let mut affected_component_issues = Vec::new();

        for affected_component in &stressor.affected_components {
            // Check affected component id characters
            if !id_chars_are_valid(affected_component) {
                affected_component_issues.push(format!(
                    "only numbers and letters allowed in id for affected component '{}'",
                    affected_component
                ));

                // Skip the integrity check for this id, move on to the next affected component
                continue;
            }

            // If chars look good, check referential integrity
            let mut match_found = false;
            for component in components {
                if component.id == *affected_component {
                    match_found = true;
                    break;
                }
            }

            // If no matching component.id found, add finding
            if !match_found {
                affected_component_issues.push(format!(
                    "affected component '{}' references non-existent component",
                    affected_component
                ));
            }
        }

        if affected_component_issues.is_empty() {
            None
        } else {
            Some(affected_component_issues.join("\n\t"))
        }
    }
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

#[cfg(test)]
mod tests {
    use crate::{
        commands::check::{IdToCheckIsFrom, check_component},
        model::{Component, Stressor},
    };

    fn component(id: &str) -> Component {
        Component {
            id: id.to_string(),
            name: None,
        }
    }

    fn stressor(id: &str, affects: &[&str]) -> Stressor {
        Stressor {
            id: id.to_string(),
            name: Some(String::new()),
            detection: Some(String::new()),
            attractor: Some(String::new()),
            business_reaction: Some(String::new()),
            technical_change: Some(String::new()),
            affected_components: affects.iter().map(|s| s.to_string()).collect(),
        }
    }

    mod component_validation {
        use super::*;

        #[test]
        fn id_is_empty() {
            let components = [component("")];
            assert_eq!(
                check_component(&components[0], &components, IdToCheckIsFrom::ExistingList),
                Some("needs id".to_string())
            );
        }

        #[test]
        fn id_has_invalid_chars() {
            let components = [component("i-d")];
            assert_eq!(
                check_component(&components[0], &components, IdToCheckIsFrom::ExistingList),
                Some("only numbers and letters allowed in id".to_string())
            );
        }

        #[test]
        fn created_from_cli_is_dup() {
            let components = [component("c1")];
            assert_eq!(
                check_component(&component("c1"), &components, IdToCheckIsFrom::CommandLine),
                Some("id 'c1' must be unique".to_string())
            );
        }

        #[test]
        fn ad_hoc_check_is_dup() {
            let components = [component("c1"), component("c1")];
            assert_eq!(
                check_component(&components[0], &components, IdToCheckIsFrom::ExistingList),
                Some("id 'c1' must be unique".to_string())
            );
        }
    }

    mod stressor_validation {
        use crate::{
            commands::check::{check_stressor, check_stressors},
            storage::STRESSORS_PATH,
        };

        use super::*;

        #[test]
        fn created_from_cli_is_dup() {
            let components = [component("c1")];
            let stressors = [stressor("s1", &["c1"])];
            assert_eq!(
                check_stressor(
                    &stressor("s1", &["c1"]),
                    &stressors,
                    &components,
                    IdToCheckIsFrom::CommandLine
                ),
                Some("id 's1' must be unique".to_string())
            );
        }

        #[test]
        fn ad_hoc_check_is_dup() {
            let components = [component("c1")];
            let stressors = [stressor("s1", &["c1"]), stressor("s1", &["c1"])];
            let findings = check_stressors(&stressors, &components).unwrap();
            assert_eq!(findings.len(), 2);
            assert!(findings[0].contains("id 's1' must be unique"));
        }

        #[test]
        fn unique_id_is_ok() {
            let components = [component("c1")];
            let stressors = [stressor("s1", &["c1"]), stressor("s2", &["c1"])];
            assert_eq!(
                check_stressors(&stressors, &components).unwrap(),
                Vec::<String>::new()
            );
        }

        #[test]
        fn affects_nonexisting_component() {
            let components = [component("c1")];
            let stressors = [stressor("s1", &["c43"])];
            let findings = check_stressors(&stressors, &components).unwrap();

            assert_eq!(
                findings,
                [format!(
                    "{} row 2- affected component 'c43' references non-existent component",
                    STRESSORS_PATH,
                )]
            );
        }

        #[test]
        fn affected_component_id_has_invalid_chars() {
            let components = [component("c1")];
            let stressors = [stressor("s1", &["c1!!!"])];
            let findings = check_stressors(&stressors, &components).unwrap();

            assert_eq!(
                findings,
                [format!(
                    "{} row 2- only numbers and letters allowed in id for affected component 'c1!!!'",
                    STRESSORS_PATH,
                )]
            );
        }
    }
}
