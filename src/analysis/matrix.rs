use std::cmp::Reverse;

use crate::model::{Component, Matrix, Stressor};

pub fn generate_incidence_matrix(stressors: Vec<Stressor>, components: Vec<Component>) -> Matrix {
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

        stressors,

        components,
    }
}

/// Returns all rows whose sum is above the average
pub fn analyze_highest_row_totals(matrix: &Matrix) -> Vec<(&Stressor, u32)> {
    let sums: Vec<u32> = sum_rows(matrix);
    let average = sums.iter().sum::<u32>() as f32 / sums.len() as f32;

    let mut top_stressors: Vec<(&Stressor, u32)> = matrix
        .stressors
        .iter()
        .zip(sums)
        .filter(|(_, sum)| *sum as f32 > average)
        .collect();

    top_stressors.sort_by_key(|s| Reverse(s.1));
    top_stressors
}

pub fn analyze_highest_col_totals(matrix: &Matrix) -> Vec<(&Component, u32)> {
    let sums = sum_cols(matrix);
    let average = sums.iter().sum::<u32>() as f32 / sums.len() as f32;

    let mut top_components: Vec<(&Component, u32)> = matrix
        .components
        .iter()
        .zip(sums)
        .filter(|(_, sum)| *sum as f32 > average)
        .collect();

    top_components.sort_by_key(|s| Reverse(s.1));
    top_components
}

pub fn analyze_coupling(matrix: &Matrix) -> Vec<(&Component, &Component, u32)> {
    let mut couplings: Vec<(&Component, &Component, u32)> = Vec::new();

    for i in 0..matrix.components.len() {
        let mut count = 0;
        for j in (i + 1)..matrix.components.len() {
            for row in &matrix.table {
                if row[i] == 1 && row[j] == 1 {
                    count += 1;
                }
            }

            if count > 0 {
                couplings.push((&matrix.components[i], &matrix.components[j], count));
            }
            count = 0;
        }
    }

    let mut sum = 0;
    for (_, _, count) in couplings.iter() {
        sum += count;
    }
    let average = sum as f32 / couplings.len() as f32;

    couplings = couplings
        .into_iter()
        .filter(|(_, _, count)| *count as f32 >= average.floor())
        .collect();

    couplings.sort_by_key(|s| Reverse(s.2));
    couplings
}

pub fn analyze_similar_responses_to_stress(matrix: &Matrix) -> Vec<Vec<&Component>> {
    let mut similar_stressed_components: Vec<Vec<&Component>> = Vec::new();
    let mut components_stressed_by_no_stressors = Vec::new();

    // Check for components stressed by every stressor or no stressors
    for (i, c) in matrix.components.iter().enumerate() {
        // Check if all rows have 0 in i column
        if matrix.table.iter().all(|row| row[i] == 0) {
            components_stressed_by_no_stressors.push(c);
        }
    }

    for (i, c) in matrix.components.iter().enumerate() {
        // Skip if the component is all 0's
        if components_stressed_by_no_stressors.contains(&c) {
            continue;
        }

        // Skip if the component is already in a cluster
        if similar_stressed_components
            .iter()
            .any(|cluster| cluster.contains(&c))
        {
            continue;
        }

        let mut cluster = Vec::new();

        for j in i + 1..matrix.components.len() {
            // If all rows for columns i and j are equal, then add both to the cluster
            if matrix.table.iter().all(|r| r[i] == r[j]) {
                // If this is the first time we add to a cluster, add both components that we compared
                if cluster.is_empty() {
                    cluster.push(c);
                }
                cluster.push(&matrix.components[j]);
            }
        }

        if !cluster.is_empty() {
            similar_stressed_components.push(cluster);
        }
    }
    similar_stressed_components
}

pub fn analyze_unstressed_components(matrix: &Matrix) -> Vec<&Component> {
    let mut unstressed_components: Vec<&Component> = Vec::new();

    for c in matrix.components.iter() {
        if !matrix
            .stressors
            .iter()
            .any(|s| s.affected_components.contains(&c.id))
        {
            if !unstressed_components.contains(&c) {
                unstressed_components.push(c);
            }
        }
    }

    unstressed_components
}

pub fn sum_cols(matrix: &Matrix) -> Vec<u32> {
    let mut col_sums = Vec::new();
    for (col, _) in matrix.components.iter().enumerate() {
        col_sums.push(matrix.table.iter().fold(
            0,
            |acc, row| {
                if row[col] == 1 { acc + 1 } else { acc }
            },
        ));
    }

    col_sums
}

pub fn sum_rows(matrix: &Matrix) -> Vec<u32> {
    let mut row_sums = Vec::new();
    for (i, _) in matrix.table.iter().enumerate() {
        row_sums.push(matrix.table[i].iter().fold(
            0,
            |acc, col| {
                if *col == 1 { acc + 1 } else { acc }
            },
        ));
    }

    row_sums
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    // Tiny builders so each test isn't buried in empty-string fields.
    fn component(id: &str) -> Component {
        Component {
            id: id.to_string(),
            name: Some(String::new()),
        }
    }

    fn stressor(id: &str, affects: &[&str]) -> Stressor {
        Stressor {
            id: Some(id.to_string()),
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
        let matrix = generate_incidence_matrix(stressors, components);

        // Assert: one row (one stressor); 1 for a and c, 0 for b.
        assert_eq!(matrix.table, vec![vec![1, 0, 1]]);
    }

    #[test]
    fn stressor_affecting_nothing_is_all_zeros() {
        let components = vec![component("a"), component("b")];
        let stressors = vec![stressor("s1", &[])]; // affects no components

        let matrix = generate_incidence_matrix(stressors, components);

        assert_eq!(matrix.table, vec![vec![0, 0]]);
    }

    #[test]
    fn sum_matrix_cols_correctly() {
        let s1 = stressor("s1", &["a", "c"]);
        let s2 = stressor("s2", &["a", "c"]);

        let c1 = component("c1");
        let c2 = component("c2");
        let c3 = component("c3");

        let matrix = Matrix {
            table: vec![vec![1, 0, 1], vec![0, 0, 1]],
            stressors: vec![s1, s2],
            components: vec![c1, c2, c3],
        };

        assert_eq!(sum_cols(&matrix), vec![1, 0, 2]);
    }

    #[test]
    fn sum_matrix_rows_correctly() {
        let s1 = stressor("s1", &["a", "c"]);
        let s2 = stressor("s2", &["a", "c"]);

        let c1 = component("c1");
        let c2 = component("c2");
        let c3 = component("c3");

        let matrix = Matrix {
            table: vec![vec![1, 0, 1], vec![1, 1, 1]],
            stressors: vec![s1, s2],
            components: vec![c1, c2, c3],
        };

        assert_eq!(sum_rows(&matrix), vec![2, 3]);
    }

    #[test]
    fn unstressed_components_identified() {
        let s1 = stressor("s1", &["a", "c3"]);
        let s2 = stressor("s2", &["c1", "c"]);

        let c1 = component("c1");
        let c2 = component("c2");
        let c3 = component("c3");

        let matrix = Matrix {
            table: vec![vec![1, 0, 1], vec![1, 1, 1]],
            stressors: vec![s1, s2],
            components: vec![c1, c2, c3],
        };

        assert_eq!(
            analyze_unstressed_components(&matrix),
            vec![&matrix.components[1]]
        );
    }

    #[test]
    fn no_unstressed_components_identified() {
        let s1 = stressor("s1", &["c2", "c3"]);
        let s2 = stressor("s2", &["c1", "c"]);

        let c1 = component("c1");
        let c2 = component("c2");
        let c3 = component("c3");

        let matrix = Matrix {
            table: vec![vec![1, 0, 1], vec![1, 1, 1]],
            stressors: vec![s1, s2],
            components: vec![c1, c2, c3],
        };

        let result: Vec<&Component> = Vec::new();

        assert_eq!(analyze_unstressed_components(&matrix), result);
    }

    #[test]
    fn highest_rows_analyzed() {
        let s1 = stressor("s1", &["c2", "c3", "c1"]);
        let s2 = stressor("s2", &[""]);

        let c1 = component("c1");
        let c2 = component("c2");
        let c3 = component("c3");

        let matrix = Matrix {
            table: vec![vec![1, 1, 1], vec![0, 0, 0]],
            stressors: vec![s1, s2],
            components: vec![c1, c2, c3],
        };

        assert_eq!(
            analyze_highest_row_totals(&matrix),
            vec![(&matrix.stressors[0], 3)]
        );
    }

    #[test]
    fn highest_cols_analyzed() {
        let s1 = stressor("s1", &["c1"]);
        let s2 = stressor("s2", &["c1"]);

        let c1 = component("c1");
        let c2 = component("c2");
        let c3 = component("c3");

        let matrix = Matrix {
            table: vec![vec![1, 0, 0], vec![1, 0, 0]],
            stressors: vec![s1, s2],
            components: vec![c1, c2, c3],
        };

        assert_eq!(
            analyze_highest_col_totals(&matrix),
            vec![(&matrix.components[0], 2)]
        );
    }

    #[test]
    fn coupling_is_analyzed() {
        let s1 = stressor("s1", &["c1", "c2"]);
        let s2 = stressor("s2", &["c1", "c2"]);
        let s3 = stressor("s3", &["c1", "c2"]);

        let c1 = component("c1");
        let c2 = component("c2");
        let c3 = component("c3");

        let matrix = Matrix {
            table: vec![vec![1, 1, 0], vec![1, 1, 0], vec![1, 1, 0]],
            stressors: vec![s1, s2, s3],
            components: vec![c1, c2, c3],
        };

        assert_eq!(
            analyze_coupling(&matrix),
            vec![(&matrix.components[0], &matrix.components[1], 3)]
        );
    }

    #[test]
    fn coupling_is_analyzed_and_sorted() {
        // Pair counts: (c1,c2)=3, (c2,c3)=4, (c1,c3)=2 → average 3.
        // The 2 is filtered out; the survivors' sorted order (4 before 3)
        // is the reverse of loop visit order, so the sort is load-bearing.
        let s1 = stressor("s1", &["c1", "c2"]);
        let s2 = stressor("s2", &["c1", "c2"]);
        let s3 = stressor("s3", &["c1", "c2"]);
        let s4 = stressor("s4", &["c2", "c3"]);
        let s5 = stressor("s5", &["c2", "c3"]);
        let s6 = stressor("s6", &["c2", "c3"]);
        let s7 = stressor("s7", &["c2", "c3"]);
        let s8 = stressor("s8", &["c1", "c3"]);
        let s9 = stressor("s9", &["c1", "c3"]);

        let c1 = component("c1");
        let c2 = component("c2");
        let c3 = component("c3");

        let matrix = Matrix {
            table: vec![
                vec![1, 1, 0],
                vec![1, 1, 0],
                vec![1, 1, 0],
                vec![0, 1, 1],
                vec![0, 1, 1],
                vec![0, 1, 1],
                vec![0, 1, 1],
                vec![1, 0, 1],
                vec![1, 0, 1],
            ],
            stressors: vec![s1, s2, s3, s4, s5, s6, s7, s8, s9],
            components: vec![c1, c2, c3],
        };

        assert_eq!(
            analyze_coupling(&matrix),
            vec![
                (&matrix.components[1], &matrix.components[2], 4),
                (&matrix.components[0], &matrix.components[1], 3)
            ]
        );
    }

    #[test]
    fn similar_components_are_identified() {
        let s1 = stressor("s1", &["c1", "c2"]);
        let s2 = stressor("s2", &["c1", "c2"]);

        let c1 = component("c1");
        let c2 = component("c2");
        let c3 = component("c3");

        let matrix = Matrix {
            table: vec![vec![1, 1, 0], vec![1, 1, 0]],
            stressors: vec![s1, s2],
            components: vec![c1, c2, c3],
        };

        assert_eq!(
            analyze_similar_responses_to_stress(&matrix),
            vec![vec![&matrix.components[0], &matrix.components[1]]]
        );
    }

    #[test]
    fn similar_components_empty_are_identified() {
        let s1 = stressor("s1", &["c1"]);
        let s2 = stressor("s2", &["c1"]);

        let c1 = component("c1");
        let c2 = component("c2");
        let c3 = component("c3");

        let matrix = Matrix {
            table: vec![vec![1, 0, 0], vec![1, 0, 0]],
            stressors: vec![s1, s2],
            components: vec![c1, c2, c3],
        };

        assert_eq!(
            analyze_similar_responses_to_stress(&matrix),
            Vec::<Vec<&Component>>::new()
        );
    }

    #[test]
    fn similar_components_no_stressors_are_identified() {
        let c1 = component("c1");
        let c2 = component("c2");
        let c3 = component("c3");

        let matrix = Matrix {
            table: vec![],
            stressors: vec![],
            components: vec![c1, c2, c3],
        };

        assert_eq!(
            analyze_similar_responses_to_stress(&matrix),
            Vec::<Vec<&Component>>::new()
        );
    }
}
