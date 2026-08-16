use crate::{
    analysis::matrix::{
        analyze_coupling, analyze_highest_col_totals, analyze_highest_row_totals,
        analyze_identical_responses_to_stress, analyze_unstressed_components,
        generate_incidence_matrix, sum_cols, sum_rows,
    },
    storage::{COMPONENTS_PATH, STRESSORS_PATH, get_analysis_path_with_date, get_rows},
};
/*
9 stressors × 5 components — 21 links, density 0.47

Most-stressed stressors            (avg 2.3)
  4  Server failure
  3  New car model

Most-sensitive components          (avg 4.2)
  6  StopChargeCommand
  5  ChargeCommand

Hidden coupling — functionally linked?
  4  StopChargeCommand ↔ UnlockCar
  3  CaptureALPR ↔ BillingDecision

Merge candidates — identical response to stress
  ChargeCommand, StopChargeCommand

Untouched components — probably under-stressed
  CustomerLogin

Trigger 6 (stressor combinations) is not automated — see reports/triggers.md
*/
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Get matrix struct
    let stressors = get_rows(STRESSORS_PATH)?;
    let components = get_rows(COMPONENTS_PATH)?;
    let matrix = generate_incidence_matrix(stressors, components);

    print!(
        "{} stressor{}",
        matrix.stressors.len(),
        if matrix.stressors.len() == 1 { "" } else { "s" }
    );
    print!(" × ");
    print!(
        "{} component{}",
        matrix.components.len(),
        if matrix.components.len() == 1 {
            ""
        } else {
            "s"
        }
    );

    let row_sums = sum_rows(&matrix);
    let sum = row_sums.iter().sum::<u32>();
    print!(" {} links", sum);

    let cells = matrix.components.len() * matrix.stressors.len();
    let density = if cells == 0 {
        0.0
    } else {
        sum as f32 / cells as f32
    };
    println!(", density = {:.3}", density);
    println!();

    let row_average = if row_sums.len() == 0 {0.0} else {row_sums.iter().sum::<u32>() as f32 / row_sums.len() as f32};
    println!("Most impactful stressors\t(avg {:.2})", row_average);
    println!();

    let highest_row_totals = analyze_highest_row_totals(&matrix);
    for (s, count) in highest_row_totals {
        println!("{}     {}", count, s.id.as_deref().unwrap_or(""));
    }
    println!();

    let col_sums = sum_cols(&matrix);
    let col_average = col_sums.iter().sum::<u32>() as f32 / col_sums.len() as f32;
    println!("Most stressed components\t(avg {:.2})", col_average);
    println!();

    let highest_col_totals = analyze_highest_col_totals(&matrix);
    for (c, count) in highest_col_totals {
        println!("{}     {}", count, c.id);
    }
    println!();

    println!("Hidden coupling");
    let couplings = analyze_coupling(&matrix);
    for (c1, c2, coupling_count) in couplings {
        println!("{}     {} ↔ {}", coupling_count, c1, c2);
    }
    println!();

    println!("Merge candidates");
    let identical_components = analyze_identical_responses_to_stress(&matrix);
    for cluster in identical_components.iter() {
        print!(
            "{}     {} ",
            cluster.len(),
            cluster
                .iter()
                .map(|component| component.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
        println!();
    }
    println!();

    println!("Untouched components");
    let unstressed_components = analyze_unstressed_components(&matrix);
    for c in unstressed_components {
        println!("{c}");
    }
    println!();

    // Output results to file
    // let analysis_path = get_analysis_path_with_date();
    // println!("Report has been saved to {}", analysis_path);
    Ok(())
}
