use crate::{
    analysis::matrix::{
        analyze_coupling, analyze_highest_col_totals, analyze_highest_row_totals,
        analyze_identical_responses_to_stress, analyze_unstressed_components,
        generate_incidence_matrix, sum_cols, sum_rows,
    },
    storage::{COMPONENTS_PATH, STRESSORS_PATH, get_analysis_path_with_datetime, get_rows},
};

use std::{fmt::Write, path::Path};

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
        for (s, count) in highest_row_totals.iter() {
            println!("{:<6}{}", count, s);
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
        for (c, count) in highest_col_totals.iter() {
            println!("{:<6}{}", count, c);
        }
    }
    println!();

    println!("Hidden coupling");
    let couplings = analyze_coupling(&matrix);
    if couplings.is_empty() {
        println!("None");
    } else {
        for (c1, c2, coupling_count) in couplings.iter() {
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
        for c in unstressed_components.iter() {
            println!("{c}");
        }
    }
    println!();

    // Output results to file
    let mut markdown_report = String::new();

    writeln!(markdown_report, "# Contagion Triggers")?;
    writeln!(
        markdown_report,
        "_Generated from {} components × {} stressors._",
        matrix.components.len(),
        matrix.stressors.len()
    )?;
    writeln!(markdown_report)?;

    writeln!(
        markdown_report,
        "### {} Coupled pairs (hidden hyperliminal coupling)",
        couplings.len()
    )?;
    writeln!(
        markdown_report,
        "Rows where one stressor hits ≥2 components. If those components aren't functionally linked, this coupling is invisible until the stressor actually occurs."
    )?;
    writeln!(markdown_report)?;
    for (c1, c2, coupling_count) in couplings {
        writeln!(
            markdown_report,
            "- {} and {} have {} common stressors",
            c1, c2, coupling_count
        )?;
    }
    writeln!(markdown_report)?;

    writeln!(
        markdown_report,
        "### {} Merge candidates (identical stress response)",
        identical_components.len()
    )?;
    writeln!(
        markdown_report,
        "Components whose columns are identical, they live and die together, so a change in one usually means a change in the other."
    )?;
    writeln!(markdown_report)?;
    for cluster in identical_components {
        writeln!(
            markdown_report,
            "- {}",
            cluster
                .into_iter()
                .map(|component| component.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )?;
    }
    writeln!(markdown_report)?;

    /* High Row Totals Examaple
       | Stressor                        | Components hit |
       | ------------------------------- | -------------- |
       | #1 Server failure               | 5              |
       | #3 New car model                | 3              |
       | #9 Server failure during charge | 2              |
    */
    writeln!(
        markdown_report,
        "### {} Highest row totals (most impactful stressors)",
        highest_row_totals.len()
    )?;
    writeln!(
        markdown_report,
        "High totals here concentrate hyperliminal coupling, often a non-functional concern."
    )?;
    writeln!(markdown_report)?;
    writeln!(markdown_report, "|Stressor|Components hit|")?;
    writeln!(markdown_report, "|---|---|")?;
    for (s, count) in highest_row_totals {
        writeln!(markdown_report, "|{}|{}|", s, count)?;
    }
    writeln!(markdown_report)?;

    writeln!(
        markdown_report,
        "### {} Highest column totals (most stress-sensitive components)",
        highest_col_totals.len()
    )?;
    writeln!(
        markdown_report,
        "Doing too much, or genuinely load-bearing. If the latter, apply redundancy."
    )?;
    writeln!(markdown_report)?;
    writeln!(markdown_report, "|Component|Stressors hitting it|")?;
    writeln!(markdown_report, "|---|---|")?;
    for (c, count) in highest_col_totals {
        writeln!(markdown_report, "|{}|{}|", c, count)?;
    }
    writeln!(markdown_report)?;

    writeln!(
        markdown_report,
        "### {} Untouched components",
        unstressed_components.len()
    )?;
    writeln!(
        markdown_report,
        "Likely under-stressed rather than invulnerable. Add more stressors touching it."
    )?;
    writeln!(markdown_report)?;
    for c in unstressed_components {
        writeln!(markdown_report, "- {} - hit by 0 stressors", c)?;
    }

    let analysis_path = get_analysis_path_with_datetime();
    if let Some(dir) = Path::new(analysis_path.as_str()).parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&analysis_path, markdown_report)?;
    println!("Report has been saved to {}", analysis_path);
    Ok(())
}
