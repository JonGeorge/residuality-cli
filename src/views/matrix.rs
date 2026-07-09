use crate::{
    analysis::matrix::{sum_cols, sum_rows},
    model::{Matrix, Stressor},
};

pub fn print_matrix(matrix: &Matrix) {
    // The left column must be wide enough for the longest stressor id (or the header).
    let label_w = label_width(&matrix.stressors);

    // Rule width = label column + each component column ("  " gap + id width)
    //            + the trailing sum column ("  " gap + 3).
    let body_w: usize = matrix.components.iter().map(|c| 2 + c.id.len()).sum();
    let rule = "─".repeat(label_w + body_w + 2 + 3);

    // --- header: component ids across the top, Σ for the per-row sum ---
    print!("{:<1$}", "stressor", label_w);
    for c in &matrix.components {
        print!("  {}", c.id);
    }
    println!("  {:>3}", "Σ");
    println!("{rule}");

    let row_sums = sum_rows(matrix);

    // --- one row per stressor: ● = affected, · = not; row sum on the right ---
    for (row, s) in matrix.table.iter().zip(&matrix.stressors) {
        print!(
            "{:<1$}",
            s.name.as_deref().unwrap_or(s.id.as_deref().unwrap()),
            label_w
        );
        for (cell, c) in row.iter().zip(&matrix.components) {
            let glyph = if *cell == 1 { "●" } else { "·" };
            print!("  {:^1$}", glyph, c.id.len());
        }
        let row_sum: u32 = row_sums[matrix
            .stressors
            .iter()
            .position(|stressor| {
                s.name.as_deref().unwrap_or(s.id.as_deref().unwrap())
                    == stressor
                        .name
                        .as_deref()
                        .unwrap_or(stressor.id.as_deref().unwrap())
            })
            .unwrap()];
        println!("  {:>3}", row_sum);
    }
    println!("{rule}");

    // --- footer: column sums (contagion pressure per component) ---
    print!("{:<1$}", "Σ", label_w);
    for (i, sum) in sum_cols(matrix).iter().enumerate() {
        print!("  {:^1$}", sum, matrix.components[i].id.len());
    }
    println!();
}

// The left label column has to fit whatever's longest: a stressor id, or the
// word "stressor" in the header. Returns that width in characters.
fn label_width(stressors: &[Stressor]) -> usize {
    stressors
        .iter()
        .map(|s| s.id.as_deref().unwrap().len())
        .max()
        .unwrap_or(0)
        .max("stressor".len())
}
