use std::collections::BTreeMap;

use crate::{
    model::{Stressor, TestStressor},
    storage::{STRESSORS_PATH, get_rows, is_missing_file_err},
};

pub fn run(file: String) -> Result<(), Box<dyn std::error::Error>> {
    let test_stressors: Vec<TestStressor> = match get_rows(&file) {
        Ok(s) => s,
        Err(e) if is_missing_file_err(e.as_ref()) => {
            eprintln!("Test file not found");
            Vec::new()
        }
        Err(e) => return Err(e),
    };

    if test_stressors.is_empty() {
        println!("No test rows to evaluate");
        return Ok(());
    }

    let original_stressors: Vec<Stressor> = match get_rows(STRESSORS_PATH) {
        Ok(s) => s,
        Err(e) if is_missing_file_err(e.as_ref()) => {
            eprintln!(
                "Original stressor file not found, can't verify 'covered_by' values in test file"
            );
            Vec::new()
        }
        Err(e) => return Err(e),
    };

    let mut naive_survival_count = 0;
    let mut residue_survival_count = 0;
    let mut multi_residue_survival_count = 0;
    let mut residue_cite_count: BTreeMap<&str, i32> = BTreeMap::new();

    let mut failed_covered_by_references = false;

    for (i, test_stressor) in test_stressors.iter().enumerate() {
        let label = if test_stressor.id.trim().is_empty() {
            format!("Row {}", i + 2)
        } else {
            test_stressor.id.clone()
        };

        // If no technical change required for architecture to survive, then stressor must be covered by a residue
        if test_stressor.technical_change.is_none() && test_stressor.covered_by.is_empty() {
            println!(
                "{}: Technical Change is empty but stressor is not covered by any residues",
                label
            );
        }

        // All stressors ids referenced in covered_by must exist

        for residue in test_stressor.covered_by.iter() {
            if !original_stressors.iter().any(|s| &s.id == residue) {
                eprintln!("{} must exist in stressors.csv to cover a test", residue);
                failed_covered_by_references = true;
            }
        }

        /*
         * Test metrics
         * combination_rate = (residual survivals with more than one covered_by) / Y
         * leverage[r] = how many test stressors cite residue r
         * orphans = residues in stressors.csv cited by nothing.
         * X = rows where naive_technical_change is blank (naive survivals)
         * Y = rows where technical_change is blank (residual survivals)
         * S = total rows
         * Ri = (Y - X) / S
         * Survived/total (Y/S) is a criticality score.
         */
        if test_stressor.naive_technical_change.is_none() {
            naive_survival_count += 1;
        }
        if test_stressor.technical_change.is_none() {
            residue_survival_count += 1;
            if test_stressor.covered_by.len() > 1 {
                multi_residue_survival_count += 1;
            }
        }
        for s in test_stressor.covered_by.iter() {
            *residue_cite_count.entry(s.as_str()).or_insert(0) += 1;
        }
    }

    if failed_covered_by_references {
        return Err("Update stressors.csv or fix covered_by values in your test".into());
    }

    let combination_rate = if residue_survival_count == 0 {
        0.0
    } else {
        multi_residue_survival_count as f32 / residue_survival_count as f32
    };

    let residual_index =
        (residue_survival_count - naive_survival_count) as f32 / test_stressors.len() as f32;

    let criticality_score = residue_survival_count as f32 / test_stressors.len() as f32;

    let mut leverage: Vec<(&&str, &i32)> = residue_cite_count.iter().collect();
    leverage.sort_by(|a, b| b.1.cmp(a.1));

    let orphans: Vec<&str> = original_stressors
        .iter()
        .map(|s| s.id.as_str())
        .filter(|id| !id.trim().is_empty())
        .filter(|id| !residue_cite_count.contains_key(id))
        .collect();

    println!("Residual index = {:.3}", residual_index);
    println!("Criticality score = {:.3}", criticality_score);
    println!(
        "Rate of multiple residues covering a single test = {:.3}",
        combination_rate
    );
    println!();
    println!("Highest leverage residues");
    for residue in leverage.iter().take(3) {
        println!("{:<6}{}", residue.0, residue.1);
    }
    if !orphans.is_empty() {
        println!();
        println!("Residues with no impact");
        for residue in orphans {
            println!("{}", residue);
        }
    }
    Ok(())
}
