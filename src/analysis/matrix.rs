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

    couplings.retain(|(_, _, count)| *count as f32 >= average.floor());

    couplings.sort_by_key(|s| Reverse(s.2));
    couplings
}

pub fn analyze_identical_responses_to_stress(matrix: &Matrix) -> Vec<Vec<&Component>> {
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
            && !unstressed_components.contains(&c)
        {
            unstressed_components.push(c);
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

    mod generate_matrix {
        use super::*;

        #[test]
        fn marks_affected_components_with_1() {
            // Arrange: three components, one stressor that hits the 1st and 3rd.
            let components = vec![component("a"), component("b"), component("c")];
            let stressors = vec![stressor("s1", &["a", "c"])];

            // Act
            let matrix = generate_incidence_matrix(stressors, components);

            // Assert: one row (one stressor); 1 for a and c, 0 for b.
            assert_eq!(matrix.table, vec![vec![1, 0, 1]]);
        }

        #[test]
        fn stressor_affecting_nothing_yields_all_zero_row() {
            let components = vec![component("a"), component("b")];
            let stressors = vec![stressor("s1", &[])]; // affects no components

            let matrix = generate_incidence_matrix(stressors, components);

            assert_eq!(matrix.table, vec![vec![0, 0]]);
        }

        #[test]
        fn no_stressors_yields_empty_table() {
            let components = vec![component("a"), component("b")];
            let stressors = vec![];

            let matrix = generate_incidence_matrix(stressors, components);

            assert_eq!(matrix.table, Vec::<Vec<u32>>::new());
        }
    }

    mod sums {
        use super::*;

        #[test]
        fn col_sum_counts_stressors_hitting_a_component() {
            let stressors = vec![stressor("s1", &["c3"]), stressor("s2", &["c1", "c3"])];

            let components = vec![component("c1"), component("c2"), component("c3")];

            let matrix = generate_incidence_matrix(stressors, components);

            assert_eq!(sum_cols(&matrix), vec![1, 0, 2]);
        }

        #[test]
        fn row_sum_counts_components_a_stressor_hits() {
            let stressors = vec![
                stressor("s1", &["c1", "c3"]),
                stressor("s2", &["c1", "c2", "c3"]),
            ];
            let components = vec![component("c1"), component("c2"), component("c3")];

            let matrix = generate_incidence_matrix(stressors, components);

            assert_eq!(sum_rows(&matrix), vec![2, 3]);
        }
    }

    mod unstressed {
        use super::*;

        #[test]
        fn reports_components_no_stressor_touches() {
            let stressors = vec![stressor("s1", &["c3"]), stressor("s2", &["c1"])];
            let components = vec![component("c1"), component("c2"), component("c3")];

            let matrix = generate_incidence_matrix(stressors, components);

            assert_eq!(
                analyze_unstressed_components(&matrix),
                vec![&matrix.components[1]]
            );
        }

        #[test]
        fn reports_nothing_when_every_component_is_stressed() {
            let stressors = vec![stressor("s2", &["c1", "c"]), stressor("s1", &["c2", "c3"])];

            let components = vec![component("c1"), component("c2"), component("c3")];

            let matrix = generate_incidence_matrix(stressors, components);

            let result: Vec<&Component> = Vec::new();

            assert_eq!(analyze_unstressed_components(&matrix), result);
        }
    }

    mod highest_totals {
        use super::*;

        #[test]
        fn only_above_average_stressors_reported() {
            let stressors = vec![stressor("s1", &["c2", "c3", "c1"]), stressor("s2", &[])];
            let components = vec![component("c1"), component("c2"), component("c3")];

            let matrix = generate_incidence_matrix(stressors, components);

            assert_eq!(
                analyze_highest_row_totals(&matrix),
                vec![(&matrix.stressors[0], 3)]
            );
        }

        #[test]
        fn only_above_average_components_reported() {
            let stressors = vec![stressor("s1", &["c1"]), stressor("s2", &["c1"])];

            let components = vec![component("c1"), component("c2"), component("c3")];

            let matrix = generate_incidence_matrix(stressors, components);

            assert_eq!(
                analyze_highest_col_totals(&matrix),
                vec![(&matrix.components[0], 2)]
            );
        }
    }

    mod coupling {
        use super::*;

        #[test]
        fn reports_shared_stressor_count_per_pair() {
            let stressors = vec![
                stressor("s1", &["c1", "c2"]),
                stressor("s2", &["c1", "c2"]),
                stressor("s3", &["c1", "c2"]),
            ];
            let components = vec![component("c1"), component("c2"), component("c3")];

            let matrix = generate_incidence_matrix(stressors, components);

            assert_eq!(
                analyze_coupling(&matrix),
                vec![(&matrix.components[0], &matrix.components[1], 3)]
            );
        }

        #[test]
        fn ranks_strongest_first_and_drops_below_average() {
            // Pair counts: (c1,c2)=3, (c2,c3)=4, (c1,c3)=2 → average 3.
            // The 2 is filtered out; the survivors' sorted order (4 before 3)
            // is the reverse of loop visit order, so the sort is load-bearing.

            let components = vec![component("c1"), component("c2"), component("c3")];
            let stressors = vec![
                stressor("s1", &["c1", "c2"]),
                stressor("s2", &["c1", "c2"]),
                stressor("s3", &["c1", "c2"]),
                stressor("s4", &["c2", "c3"]),
                stressor("s5", &["c2", "c3"]),
                stressor("s6", &["c2", "c3"]),
                stressor("s7", &["c2", "c3"]),
                stressor("s8", &["c1", "c3"]),
                stressor("s9", &["c1", "c3"]),
            ];

            let matrix = generate_incidence_matrix(stressors, components);

            assert_eq!(
                analyze_coupling(&matrix),
                vec![
                    (&matrix.components[1], &matrix.components[2], 4),
                    (&matrix.components[0], &matrix.components[1], 3)
                ]
            );
        }
    }

    mod identical_responses_to_stress {
        use super::*;

        #[test]
        fn clusters_components_with_identical_columns() {
            let stressors = vec![stressor("s1", &["c1", "c2"]), stressor("s2", &["c1", "c2"])];
            let components = vec![component("c1"), component("c2"), component("c3")];

            let matrix = generate_incidence_matrix(stressors, components);

            assert_eq!(
                analyze_identical_responses_to_stress(&matrix),
                vec![vec![&matrix.components[0], &matrix.components[1]]]
            );
        }

        #[test]
        fn all_zero_columns_do_not_form_a_cluster() {
            let components = vec![component("c1"), component("c2"), component("c3")];
            let stressors = vec![stressor("s1", &["c1"]), stressor("s2", &["c1"])];

            let matrix = generate_incidence_matrix(stressors, components);

            assert_eq!(
                analyze_identical_responses_to_stress(&matrix),
                Vec::<Vec<&Component>>::new()
            );
        }

        #[test]
        fn no_stressors_means_no_clusters() {
            let components = vec![component("c1"), component("c2"), component("c3")];
            let stressors = vec![];

            let matrix = generate_incidence_matrix(stressors, components);

            assert_eq!(
                analyze_identical_responses_to_stress(&matrix),
                Vec::<Vec<&Component>>::new()
            );
        }
    }
}
