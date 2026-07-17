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

pub fn analyze_highest_col_totals(matrix: &Matrix) -> Vec<String> {
    Vec::new()
}

pub fn analyze_coupling(matrix: &Matrix) -> Vec<String> {
    Vec::new()
}

pub fn analyze_similar_responses_to_stress(matrix: &Matrix) -> Vec<String> {
    Vec::new()
}

pub fn analyze_unstressed_components(matrix: &Matrix) -> Vec<String> {
    Vec::new()
}

pub fn get_unstressed_components(matrix: &Matrix) -> Vec<String> {
    let mut unstressed_component_ids: Vec<String> = Vec::new();

    for c in matrix.components.iter() {
        if !matrix
            .stressors
            .iter()
            .any(|s| s.affected_components.contains(&c.id))
        {
            if !unstressed_component_ids.contains(&c.id) {
                unstressed_component_ids.push(c.id.clone());
            }
        }
    }

    unstressed_component_ids
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
    use std::vec;

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

        assert_eq!(get_unstressed_components(&matrix), vec!["c2"]);
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

        let result: Vec<String> = Vec::new();

        assert_eq!(get_unstressed_components(&matrix), result);
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
}
