use clap::{Parser, Subcommand};

mod model;
use model::{Component, Stressor};

mod storage;
use storage::{add_component, load_components_csv, load_stressors_csv};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("Initialized...");
            Ok(())
        }

        Commands::Component { action } => match action {
            ComponentAction::Add { id, name } => add_component(id, name),
        },

        Commands::Matrix => {
            let components = load_components_csv()?;
            let stressors = load_stressors_csv()?;
            let matrix = generate_matrix(&stressors, &components);
            print_matrix(&matrix, &stressors, &components);
            Ok(())
        }

        _ => {
            eprintln!("not implemented yet");
            Ok(())
        }
    }
}

#[derive(Parser)]
#[command(name = "residuality")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Component {
        #[command(subcommand)]
        action: ComponentAction,
    },
    Stressor {
        #[command(subcommand)]
        action: StressorAction,
    },
    Matrix,
    Triggers,
    Test {
        file: String,
    },
}

#[derive(Subcommand)]
enum ComponentAction {
    Add { id: String, name: String },
}

#[derive(Subcommand)]
enum StressorAction {
    // Placeholder for now; we'll grow this when the Stressor struct comes back.
    Add { id: String, name: String },
}

fn generate_matrix(stressors: &[Stressor], components: &[Component]) -> Vec<Vec<u32>> {
    stressors
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
        .collect()
}

fn print_matrix(matrix: &[Vec<u32>], stressors: &[Stressor], components: &[Component]) {
    // The left column must be wide enough for the longest stressor id (or the header).
    let label_w = label_width(stressors);

    // Rule width = label column + each component column ("  " gap + id width)
    //            + the trailing sum column ("  " gap + 3).
    let body_w: usize = components.iter().map(|c| 2 + c.id.len()).sum();
    let rule = "─".repeat(label_w + body_w + 2 + 3);

    // --- header: component ids across the top, Σ for the per-row sum ---
    print!("{:<1$}", "stressor", label_w);
    for c in components {
        print!("  {}", c.id);
    }
    println!("  {:>3}", "Σ");
    println!("{rule}");

    // --- one row per stressor: ● = affected, · = not; row sum on the right ---
    for (row, s) in matrix.iter().zip(stressors) {
        print!("{:<1$}", s.id, label_w);
        for (cell, c) in row.iter().zip(components) {
            let glyph = if *cell == 1 { "●" } else { "·" };
            print!("  {:^1$}", glyph, c.id.len());
        }
        let row_sum: u32 = row.iter().sum();
        println!("  {:>3}", row_sum);
    }
    println!("{rule}");

    // --- footer: column sums (contagion pressure per component) ---
    print!("{:<1$}", "Σ", label_w);
    for (i, c) in components.iter().enumerate() {
        let col_sum: u32 = matrix.iter().map(|r| r[i]).sum();
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
            name: String::new(),
        }
    }

    fn stressor(id: &str, affects: &[&str]) -> Stressor {
        Stressor {
            id: id.to_string(),
            name: String::new(),
            detection: String::new(),
            attractor: String::new(),
            business_reaction: String::new(),
            technical_change: String::new(),
            affected_components: affects.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn marks_each_affected_component_with_1() {
        // Arrange: three components, one stressor that hits the 1st and 3rd.
        let components = vec![component("a"), component("b"), component("c")];
        let stressors = vec![stressor("s1", &["a", "c"])];

        // Act
        let matrix = generate_matrix(&stressors, &components);

        // Assert: one row (one stressor); 1 for a and c, 0 for b.
        assert_eq!(matrix, vec![vec![1, 0, 1]]);
    }

    #[test]
    fn stressor_affecting_nothing_is_all_zeros() {
        let components = vec![component("a"), component("b")];
        let stressors = vec![stressor("s1", &[])]; // affects no components

        let matrix = generate_matrix(&stressors, &components);

        assert_eq!(matrix, vec![vec![0, 0]]);
    }
}
