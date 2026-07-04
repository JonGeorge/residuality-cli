use crate::analysis::matrix::{generate_incidence_matrix};
use crate::cli::MatrixAction;
use crate::storage::{
    COMPONENTS_PATH, STRESSORS_PATH, get_matrix_path_with_date, get_rows, write_matrix_to_csv,
};
use crate::views::matrix::print_matrix;

pub fn run(action: MatrixAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        MatrixAction::Export => {
            // Get Header Row (Components) and Column (Stressors)
            let components = get_rows(COMPONENTS_PATH)?;
            let stressors = get_rows(STRESSORS_PATH)?;

            // Create matrix and get it as a vector
            let matrix = generate_incidence_matrix(stressors, components);

            // Write vectors to csv
            let matrix_path = get_matrix_path_with_date();
            write_matrix_to_csv(&matrix_path, &matrix)?;

            println!("Export saved to ./{}", matrix_path);
        }

        MatrixAction::Print => {
            let components = get_rows(COMPONENTS_PATH)?;
            let stressors = get_rows(STRESSORS_PATH)?;
            let matrix = generate_incidence_matrix(stressors, components);
            print_matrix(&matrix);
        }
    }

    Ok(())
}
