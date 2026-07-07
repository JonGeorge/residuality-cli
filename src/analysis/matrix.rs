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
        row_sums.push(matrix.table[i].iter().fold(0, |acc, col| {
            if *col == 1 {
                acc + 1
            }
            else {
                acc
            }
        }));
    }

    row_sums
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, vec};

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
        let s1 = Stressor { id: None, name: None, detection: None, attractor: None, business_reaction: None, technical_change: None, affected_components: BTreeSet::new() };
        let s2 = Stressor { id: None, name: None, detection: None, attractor: None, business_reaction: None, technical_change: None, affected_components: BTreeSet::new() };

        let c1 = Component { id: "c1".to_string(), name: None };
        let c2 = Component { id: "c2".to_string(), name: None };
        let c3 = Component { id: "c3".to_string(), name: None };

        let matrix = Matrix { table: vec![vec![1,0,1], vec![0, 0, 1]], stressors: vec![s1, s2], components: vec![c1, c2, c3] };

        assert_eq!(sum_cols(&matrix), vec![1, 0, 2]);
    }

    #[test]
    fn sum_matrix_rows_correctly() {
        let s1 = Stressor { id: None, name: None, detection: None, attractor: None, business_reaction: None, technical_change: None, affected_components: BTreeSet::new() };
        let s2 = Stressor { id: None, name: None, detection: None, attractor: None, business_reaction: None, technical_change: None, affected_components: BTreeSet::new() };

        let c1 = Component { id: "c1".to_string(), name: None };
        let c2 = Component { id: "c2".to_string(), name: None };
        let c3 = Component { id: "c3".to_string(), name: None };

        let matrix = Matrix { table: vec![vec![1,0,1], vec![1, 1, 1]], stressors: vec![s1, s2], components: vec![c1, c2, c3] };

        assert_eq!(sum_rows(&matrix), vec![2, 3]);
    }
}
