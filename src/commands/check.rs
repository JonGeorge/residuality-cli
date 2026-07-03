use std::ops::Add;

use crate::{model::{Component, Stressor}, storage::{COMPONENTS_PATH, STRESSORS_PATH, get_rows}, views::check::print_findings,};

pub fn run() -> Result<i32, Box<dyn std::error::Error>> {
    let mut findings: Vec<String> = Vec::new();

    let components: Vec<Component> = match get_rows(COMPONENTS_PATH) {
        Ok(c) => c,
        Err(e) => {
            findings.push(format!("{} ", COMPONENTS_PATH).add(&e.to_string()));
            Vec::new()
        }
    };

    let stressors: Vec<Stressor> = match get_rows(STRESSORS_PATH) {
        Ok(s) => s,
        Err(e) => {
            findings.push(format!("{} ", STRESSORS_PATH).add(&e.to_string()));
            Vec::new()
        }
    };
    findings.extend(check_components(&components)?);
    findings.extend(check_stressors(&stressors, &components)?);

    if findings.is_empty() {
         println!("Everything looks good!");
         Ok(0)
    } else {
        print_findings(findings);
        Err("check failed".into())
    }


}

fn check_components(
    components: &[Component],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut findings: Vec<String> = Vec::new();

    for (i, c) in components.iter().enumerate() {
        // Check if id is empty
        if c.id.trim().is_empty() {
            findings.push(format!("{} row {}- needs id", COMPONENTS_PATH, i + 2));
        }
        // Check if id contains only letters, numbers, and underscores
        else if !id_chars_are_valid(&c.id) {
            findings.push(format!(
                "{} row {}- only numbers and letters allowed in id",
                COMPONENTS_PATH,
                i + 2
            ));
        }
    }

    Ok(findings)
}

fn check_stressors(
    stressors: &[Stressor],
    components: &[Component],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut findings: Vec<String> = Vec::new();

    if components.is_empty() {
        findings.push(format!("{} not checked due to invalid components file", STRESSORS_PATH));
    }
    else {
        for (i, stressor) in stressors.iter().enumerate() {
            let mut match_found = false;

            for affected_component in &stressor.affected_components  {
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
    }
    Ok(findings)
}

fn id_chars_are_valid(id: &str) -> bool {
    id.chars().all(|ch| ch.is_alphanumeric() || ch == '_')
}
