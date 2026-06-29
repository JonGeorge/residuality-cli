// model.rs — the core data types the whole tool revolves around.
//
// `pub` on the struct makes the TYPE visible outside this module; `pub` on each
// FIELD makes that field visible too. Both are needed because main.rs builds a
// Stressor (writing every field) and reads fields like `affected_components`.
// Without `pub` on the fields, main.rs could name the type but not touch its insides.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

// A Component is one part of the architecture we're stressing.
#[derive(Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub name: Option<String>,
}

// A Stressor is an environmental pressure on the architecture.
#[derive(Serialize, Deserialize)]
pub struct Stressor {
    pub id: String,

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
    pub affected_components: Vec<String>,
}

// A custom field deserializer. serde hands us a `deserializer` sitting on the raw
// cell, and WE decide how to turn it into a Vec<String>. It's wired in by the
// #[serde(deserialize_with = "...")] attribute above — the string must match this name.
//
//   <'de, D>             -> generic over any deserializer type D (CSV here; could be JSON)
//   'de                  -> a lifetime: "how long the input data lives" (serde requires it)
//   D: Deserializer<'de> -> the bound: D must actually be a deserializer
//   -> Result<Vec<String>, D::Error> -> succeed with the Vec, or fail with serde's error
fn deserialize_affects<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // Step 1 (done for you): pull the cell out as a plain String first, using serde's
    // built-in String impl. Now `cell` is an ordinary owned String we can slice up.
    let cell = String::deserialize(deserializer)?;

    let affected_components = cell
        .split(";")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(affected_components)
}

// A custom field serializer: collapse the Vec into ONE cell joined by ';'.
// This is the inverse of deserialize_affects.
fn serialize_affects<S>(affects: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let str = affects.join(";");
    serializer.serialize_str(&str)
}

pub struct Matrix {
    pub table: Vec<Vec<u32>>,
    pub stressors: Vec<Stressor>,
    pub components: Vec<Component>
}
