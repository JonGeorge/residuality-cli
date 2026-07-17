use crate::{
    analysis::matrix::{
        analyze_highest_col_totals, analyze_highest_row_totals, generate_incidence_matrix,
    },
    model::{Component, Stressor},
    storage::{COMPONENTS_PATH, STRESSORS_PATH, get_analysis_path_with_date, get_rows},
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let analysis_path = get_analysis_path_with_date();

    // Get matrix struct

    let stressors = get_rows(STRESSORS_PATH)?;
    let components = get_rows(COMPONENTS_PATH)?;
    let matrix = generate_incidence_matrix(stressors, components);

    // Run through all analyses
    let highest_row_totals: Vec<(&Stressor, u32)> = analyze_highest_row_totals(&matrix);
    let highest_col_totals: Vec<(&Component, u32)> = analyze_highest_col_totals(&matrix);

    // for (s, count) in highest_row_totals {
    //     println!("{} - {}", count, s.id.as_deref().unwrap());
    // }

    // for (c, count) in highest_col_totals {
    //     println!("{} - {}", count, c.id);
    // }

    // Output results to file
    // println!("Report has been saved to {}", analysis_path);
    Ok(())
}
