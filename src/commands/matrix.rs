use crate::model::{Component, Matrix, Stressor};
use crate::storage::{get_rows, COMPONENTS_PATH, STRESSORS_PATH};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let components = get_rows(COMPONENTS_PATH)?;
    let stressors = get_rows(STRESSORS_PATH)?;
    let matrix = generate_matrix(stressors, components);
    print_matrix(&matrix);
    Ok(())
}

fn generate_matrix(stressors: Vec<Stressor>, components: Vec<Component>) -> Matrix {
    Matrix {
        table: stressors
            .iter()
            .map(|s| {
                components
                    .iter()
                    .map(|c| {
                        if s.affected_components.contains(&c.id) {
                            1
                        } else {
                            0
                        }
                    })
                    .collect()
            })
            .collect(),

        stressors: stressors,

        components: components

    }
}

fn print_matrix(matrix: &Matrix) {
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

    // --- one row per stressor: ● = affected, · = not; row sum on the right ---
    for (row, s) in matrix.table.iter().zip(&matrix.stressors) {
        print!("{:<1$}", s.name.as_deref().unwrap_or(&s.id), label_w);
        for (cell, c) in row.iter().zip(&matrix.components) {
            let glyph = if *cell == 1 { "●" } else { "·" };
            print!("  {:^1$}", glyph, c.id.len());
        }
        let row_sum: u32 = row.iter().sum();
        println!("  {:>3}", row_sum);
    }
    println!("{rule}");

    // --- footer: column sums (contagion pressure per component) ---
    print!("{:<1$}", "Σ", label_w);
    for (i, c) in matrix.components.iter().enumerate() {
        let col_sum: u32 = matrix.table.iter().map(|r| r[i]).sum();
        print!("  {:^1$}", col_sum, c.id.len());
    }
    println!();
}

// The left label column has to fit whatever's longest: a stressor id, or the
// word "stressor" in the header. Returns that width in characters.
fn label_width(stressors: &[Stressor]) -> usize {
    stressors
        .iter()
        .map(|s| s.id.len())
        .max()
        .unwrap_or(0)
        .max("stressor".len())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tiny builders so each test isn't buried in empty-string fields.
    fn component(id: &str) -> Component {
        Component {
            id: id.to_string(),
            name: Some(String::new()),
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

    #[test]
    fn marks_each_affected_component_with_1() {
        // Arrange: three components, one stressor that hits the 1st and 3rd.
        let components = vec![component("a"), component("b"), component("c")];
        let stressors = vec![stressor("s1", &["a", "c"])];

        // Act
        let matrix = generate_matrix(stressors, components);

        // Assert: one row (one stressor); 1 for a and c, 0 for b.
        assert_eq!(matrix.table, vec![vec![1, 0, 1]]);
    }

    #[test]
    fn stressor_affecting_nothing_is_all_zeros() {
        let components = vec![component("a"), component("b")];
        let stressors = vec![stressor("s1", &[])]; // affects no components

        let matrix = generate_matrix(stressors, components);

        assert_eq!(matrix.table, vec![vec![0, 0]]);
    }
}
