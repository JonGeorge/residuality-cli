// model.rs — the core data types the whole tool revolves around.
//
// `pub` on the struct makes the TYPE visible outside this module; `pub` on each
// FIELD makes that field visible too. Both are needed because main.rs builds a
// Stressor (writing every field) and reads fields like `affected_components`.
// Without `pub` on the fields, main.rs could name the type but not touch its insides.

use std::{collections::BTreeSet, fmt};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

// A Component is one part of the architecture we're stressing.
#[derive(Serialize, Deserialize)]
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

pub struct Matrix {
    pub table: Vec<Vec<u32>>,
    pub stressors: Vec<Stressor>,
    pub components: Vec<Component>,
}
