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
    use csv::{ReaderBuilder, WriterBuilder};

    use crate::model::Stressor;
    use std::collections::BTreeSet;

    const HEADER: &str =
        "id,name,detection,attractor,business_reaction,technical_change,affected_components\n";

    fn stressor(id: &str, affects: &[&str]) -> Stressor {
        Stressor {
            id: Some(id.to_string()),
            name: None,
            detection: None,
            attractor: None,
            business_reaction: None,
            technical_change: None,
            affected_components: affects.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn write_stressor_to_csv(stressor: &Stressor) -> Result<String, Box<dyn std::error::Error>> {
        let mut writer = WriterBuilder::new().has_headers(true).from_writer(vec![]);
        writer.serialize(stressor)?;

        let data = String::from_utf8(writer.into_inner()?)?;
        Ok(data)
    }

    fn parse_stressor(csv_row: &str) -> Stressor {
        let result = parse_result(csv_row);

        result.unwrap().into_iter().next().unwrap()
    }

    fn parse_result(csv_row: &str) -> Result<Vec<Stressor>, csv::Error> {
        let csv_row_with_header = format!("{HEADER}{csv_row}");

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

    mod serializer {

        use super::*;

        #[test]
        fn basic_stressor_is_serialized() {
            let stressor = Stressor {
                id: Some("S1".to_string()),
                name: Some("TEST".to_string()),
                detection: Some("eee".to_string()),
                attractor: Some("eee".to_string()),
                business_reaction: Some("eee".to_string()),
                technical_change: Some("eee".to_string()),
                affected_components: BTreeSet::from(["test_comp".to_string()]),
            };

            let csv = write_stressor_to_csv(&stressor).unwrap();

            assert_eq!(csv, format!("{HEADER}S1,TEST,eee,eee,eee,eee,test_comp\n"));
        }

        #[test]
        fn stressor_with_no_values_is_serialized() {
            let stressor = stressor("", &[]);

            let csv = write_stressor_to_csv(&stressor).unwrap();

            assert_eq!(csv, format!("{HEADER},,,,,,\n"));
        }

        #[test]
        fn multiple_affected_components_are_serialized() {
            let stressor = stressor("S1", &["c1", "c2", "c3"]);

            let csv = write_stressor_to_csv(&stressor).unwrap();

            assert_eq!(csv, format!("{HEADER}S1,,,,,,c1;c2;c3\n"));
        }

        #[test]
        fn affects_are_sorted() {
            let stressor = stressor("S1", &["z", "a", "b"]);

            let csv = write_stressor_to_csv(&stressor).unwrap();

            assert_eq!(csv, format!("{HEADER}S1,,,,,,a;b;z\n"));
        }

        #[test]
        fn comma_in_affects_is_quoted() {
            let stressor = stressor("S1", &["c,1", "c12", "c13"]);

            let csv = write_stressor_to_csv(&stressor).unwrap();

            assert_eq!(csv, format!("{HEADER}S1,,,,,,\"c,1;c12;c13\"\n"));
        }
    }

    #[test]
    fn stressor_can_be_written_and_read() {
        let stressor_initial = Stressor {
            id: Some("S1".to_string()),
            name: Some("Stressor1".to_string()),
            detection: Some("Something is broken".to_string()),
            attractor: Some("No one can log in".to_string()),
            business_reaction: Some("Add more help desk staff".to_string()),
            technical_change: None,
            affected_components: ["c1", "c2", "c3"].iter().map(|s| s.to_string()).collect(),
        };

        // Serialize stressor
        let csv = write_stressor_to_csv(&stressor_initial).unwrap();

        // Deserialize stressor
        let stressor_returned = parse_result_custom_header(csv.as_str()).unwrap();

        assert_eq!(stressor_initial, stressor_returned[0]);
    }
}
