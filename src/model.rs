// model.rs — the core data types the whole tool revolves around.
//
// `pub` on the struct makes the TYPE visible outside this module; `pub` on each
// FIELD makes that field visible too. Both are needed because main.rs builds a
// Stressor (writing every field) and reads fields like `affected_components`.
// Without `pub` on the fields, main.rs could name the type but not touch its insides.

use std::{collections::BTreeSet, fmt};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

// A Component is one part of the architecture we're stressing.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Component {
    pub id: String,
    pub name: Option<String>,
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(n) => write!(f, "{}", n),
            None => write!(f, "{}", self.id),
        }
    }
}

// A Stressor is an environmental pressure on the architecture.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Stressor {
    pub id: Option<String>,

    pub name: Option<String>,

    pub detection: Option<String>,

    pub attractor: Option<String>,

    pub business_reaction: Option<String>,

    pub technical_change: Option<String>,

    // This ONE field is parsed by our function instead of serde's default Vec logic.
    // The CSV cell holds ids joined by ';'  (e.g. "auth_service;database").
    #[serde(
        serialize_with = "serialize_affects",
        deserialize_with = "deserialize_affects"
    )]
    pub affected_components: BTreeSet<String>,
}

fn deserialize_affects<'de, D>(deserializer: D) -> Result<BTreeSet<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let cell = String::deserialize(deserializer)?;

    let affected_components = cell
        .split(";")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(affected_components)
}

/// A custom field serializer: collapse the collection into ONE cell joined by ';'.
fn serialize_affects<S>(affects: &BTreeSet<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let vect: Vec<&str> = affects.iter().map(|c| c.as_str()).collect();
    serializer.serialize_str(vect.join(";").as_str())
}

#[derive(Serialize, Deserialize)]
pub struct TestStressor {
    pub id: Option<String>,

    pub name: Option<String>,

    pub detection: Option<String>,

    pub attractor: Option<String>,

    pub business_reaction: Option<String>,

    pub technical_change: Option<String>,

    // This ONE field is parsed by our function instead of serde's default Vec logic.
    // The CSV cell holds ids joined by ';'  (e.g. "auth_service;database").
    #[serde(
        serialize_with = "serialize_affects",
        deserialize_with = "deserialize_affects"
    )]
    pub affected_components: BTreeSet<String>,

    pub naive_technical_change: Option<String>,

    #[serde(
        serialize_with = "serialize_affects",
        deserialize_with = "deserialize_affects"
    )]
    pub covered_by: BTreeSet<String>,
}

pub struct Matrix {
    pub table: Vec<Vec<u32>>,
    pub stressors: Vec<Stressor>,
    pub components: Vec<Component>,
}

#[cfg(test)]
mod tests {
    use csv::ReaderBuilder;
    use std::collections::BTreeSet;

    use crate::model::Stressor;

    fn parse_stressor(csv_row: &str) -> Stressor {
        let result = parse_result(csv_row);

        result.unwrap().into_iter().next().unwrap()
    }

    fn parse_result(csv_row: &str) -> Result<Vec<Stressor>, csv::Error> {
        let csv_row_with_header = format!(
            "id,name,detection,attractor,business_reaction,technical_change,affected_components\n{csv_row}"
        );

        parse_result_custom_header(&csv_row_with_header)
    }

    fn parse_result_custom_header(csv_row_with_header: &str) -> Result<Vec<Stressor>, csv::Error> {
        let mut reader = ReaderBuilder::new().from_reader(csv_row_with_header.as_bytes()); //test uses from_reader, prod code uses from_path

        reader.deserialize().collect::<Result<Vec<Stressor>, _>>()
    }

    mod deserializer {
        use super::*;

        #[test]
        fn basic_stressor_is_deserialized() {
            let csv = "S1,TEST,eee,eee,eee,eee,test_comp";

            let stressor = parse_stressor(csv);

            let expected = Stressor {
                id: Some("S1".to_string()),
                name: Some("TEST".to_string()),
                detection: Some("eee".to_string()),
                attractor: Some("eee".to_string()),
                business_reaction: Some("eee".to_string()),
                technical_change: Some("eee".to_string()),
                affected_components: BTreeSet::from(["test_comp".to_string()]),
            };

            assert_eq!(stressor, expected);
        }

        #[test]
        fn misspelled_header_is_none() {
            /*
             * A misspelled header means the column doesn't match any struct field,
             * and because the Stressor fields are Option, they become None
             */
            let csv = "id,name,detction,atractor,businessreaction,technical_change,affected_components\nS1,TEST,eee,eee,eee,eee,test_comp";

            let result = parse_result_custom_header(csv);
            let stressors = result.unwrap();

            let expected = Stressor {
                id: Some("S1".to_string()),
                name: Some("TEST".to_string()),
                detection: None,
                attractor: None,
                business_reaction: None,
                technical_change: Some("eee".to_string()),
                affected_components: BTreeSet::from(["test_comp".to_string()]),
            };

            assert_eq!(stressors[0], expected);
        }

        #[test]
        fn malformed_stressor_too_few_fields_is_err() {
            let csv = "S1,TEST,eee,eee,eee,test_comp";

            let deserialized_rows = parse_result(csv);

            assert!(deserialized_rows.is_err());
        }

        #[test]
        fn extra_fields_is_err() {
            let csv = "S1,TEST,eee,eee,eee,eee,test_comp,err1,err2,err3";

            let deserialized_rows = parse_result(csv);

            assert!(deserialized_rows.is_err());
        }

        #[test]
        fn basic_affects_are_deserialized() {
            let csv = "S1,TEST,eee,eee,eee,eee,test_comp1;test_comp2;test_comp3";

            let stressor = parse_stressor(csv);

            let expected = BTreeSet::from([
                "test_comp1".to_string(),
                "test_comp2".to_string(),
                "test_comp3".to_string(),
            ]);

            assert_eq!(stressor.affected_components, expected);
        }

        #[test]
        fn white_space_is_trimmed_in_affects() {
            let csv = "S1,TEST,eee,eee,eee,eee, test_comp;  test_comp2    ";

            let stressor = parse_stressor(csv);

            let expected = BTreeSet::from(["test_comp".to_string(), "test_comp2".to_string()]);

            assert_eq!(stressor.affected_components, expected);
        }

        #[test]
        fn empty_affects_are_filtered() {
            let csv = "S1,TEST,eee,eee,eee,eee, ;;;test_comp;;;; test_comp2    ;;;";

            let stressor = parse_stressor(csv);

            let expected = BTreeSet::from(["test_comp".to_string(), "test_comp2".to_string()]);

            assert_eq!(stressor.affected_components, expected);
        }

        #[test]
        fn completely_empty_affects_are_ignored() {
            let csv = "S1,TEST,eee,eee,eee,eee,";

            let stressor = parse_stressor(csv);

            let expected = BTreeSet::new();

            assert_eq!(stressor.affected_components, expected);
        }

        #[test]
        fn missing_affects_column_is_err() {
            let csv = "id,name,detection,attractor,business_reaction,technical_change\nS1,TEST,eee,eee,eee,eee";

            let stressor = parse_result_custom_header(csv);

            assert!(stressor.is_err());
        }

        #[test]
        fn duplicate_affects_are_deduped() {
            let csv = "S1,TEST,eee,eee,eee,eee,c1;c1;c1;c2";

            let stressor = parse_stressor(csv);

            let expected = BTreeSet::from(["c1".to_string(), "c2".to_string()]);

            assert_eq!(stressor.affected_components, expected);
        }

        #[test]
        fn affects_are_sorted() {
            let csv = "S1,TEST,eee,eee,eee,eee,zz;bb;mm;aa;tt;pp;432;c1;c1;c1;c2";

            let stressor = parse_stressor(csv); // BTreeSet is the container for affected_components
            let affected_components_in_order: Vec<&String> =
                stressor.affected_components.iter().collect();

            assert_eq!(
                affected_components_in_order,
                ["432", "aa", "bb", "c1", "c2", "mm", "pp", "tt", "zz"]
            );
        }

        #[test]
        fn unquoted_comma_in_affects_is_err() {
            let csv = "S1,TEST,eee,eee,eee,eee,c,2;ea"; // Doesnt match header len

            let result = parse_result(csv);

            assert!(result.is_err());
        }

        #[test]
        fn quoted_comma_retains_affects() {
            let csv = r#"S1,TEST,eee,eee,eee,eee,"c,2;ea""#; //raw string

            let affected_components = parse_stressor(csv).affected_components;

            assert_eq!(
                affected_components,
                BTreeSet::from(["c,2".to_string(), "ea".to_string()])
            );
        }
    }
}
