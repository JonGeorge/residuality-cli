use crate::{
    analysis::matrix::{
        analyze_coupling, analyze_highest_col_totals, analyze_highest_row_totals,
        analyze_identical_responses_to_stress, analyze_unstressed_components,
        generate_incidence_matrix, sum_cols, sum_rows,
    },
    storage::{COMPONENTS_PATH, STRESSORS_PATH, get_rows},
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
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
    print!(" - {} links", sum);

    let cells = matrix.components.len() * matrix.stressors.len();
    let density = if cells == 0 {
        0.0
    } else {
        sum as f32 / cells as f32
    };
    println!(", density = {:.3}", density);
    println!();

    let row_average = if row_sums.is_empty() {
        0.0
    } else {
        sum as f32 / row_sums.len() as f32
    };
    println!("Most impactful stressors\t(avg {:.2})", row_average);

    let highest_row_totals = analyze_highest_row_totals(&matrix);
    if highest_row_totals.is_empty() {
        println!("None");
    } else {
        for (s, count) in highest_row_totals {
            println!(
                "{:<6}{}",
                count,
                s.name.as_deref().unwrap_or("<Missing stressor name>")
            );
        }
    }
    println!();

    let col_sums = sum_cols(&matrix);
    let col_average = if col_sums.is_empty() {
        0.0
    } else {
        col_sums.iter().sum::<u32>() as f32 / col_sums.len() as f32
    };
    println!("Most stressed components\t(avg {:.2})", col_average);

    let highest_col_totals = analyze_highest_col_totals(&matrix);
    if highest_col_totals.is_empty() {
        println!("None");
    } else {
        for (c, count) in highest_col_totals {
            println!("{:<6}{}", count, c);
        }
    }
    println!();

    println!("Hidden coupling");
    let couplings = analyze_coupling(&matrix);
    if couplings.is_empty() {
        println!("None");
    } else {
        for (c1, c2, coupling_count) in couplings {
            println!("{:<6}{} ↔ {}", coupling_count, c1, c2);
        }
    }
    println!();

    println!("Merge candidates");
    let identical_components = analyze_identical_responses_to_stress(&matrix);
    if identical_components.is_empty() {
        println!("None");
    } else {
        for cluster in identical_components.iter() {
            println!(
                "{:<6}{}",
                cluster.len(),
                cluster
                    .iter()
                    .map(|component| component.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            );
        }
    }
    println!();

    println!("Untouched components");
    let unstressed_components = analyze_unstressed_components(&matrix);
    if unstressed_components.is_empty() {
        println!("None");
    } else {
        for c in unstressed_components {
            println!("{c}");
        }
    }
    println!();

    // Output results to file
    // let analysis_path = get_analysis_path_with_date();
    // println!("Report has been saved to {}", analysis_path);
    Ok(())
}
