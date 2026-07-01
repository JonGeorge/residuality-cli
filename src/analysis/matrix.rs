use crate::model::{Component, Matrix, Stressor};

pub fn get_matrix_as_vectors(
    matrix: &Matrix,
) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let mut vectors = std::vec::Vec::new();

    // For each component add to a vector ["", component1.id, component2.id, ... ]
    let mut header_row: Vec<String> = matrix
        .components
        .iter()
        .map(|c| c.name.as_deref().unwrap_or(&c.id).to_string())
        .collect::<Vec<String>>();
    header_row.insert(0, "".to_string());
    vectors.push(header_row);

    // For each stressor, add the stressor name and the incident table to a vector [stressor.name, 1, 0, ... ]
    for (i, stressor) in matrix.stressors.iter().enumerate() {
        let mut data_row: Vec<String> = std::vec::Vec::new();
        data_row.push(stressor.name.as_deref().unwrap_or(&stressor.id).to_string());
        data_row.extend(matrix.table[i].iter().map(|d| d.to_string()));
        vectors.push(data_row);
    }

    Ok(vectors)
}

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

        stressors: stressors,

        components: components,
    }
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
}
