use crate::{
    analysis::matrix::generate_incidence_matrix,
    storage::{COMPONENTS_PATH, STRESSORS_PATH, get_analysis_path_with_date, get_rows},
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let analysis_path = get_analysis_path_with_date();

    // Get matrix struct

    let stressors = get_rows(STRESSORS_PATH)?;
    let components = get_rows(COMPONENTS_PATH)?;
    let matrix = generate_incidence_matrix(stressors, components);

    // Run through all analyses
    // Output results to file
    println!("Report has been saved to {}", analysis_path);
    Ok(())
}
