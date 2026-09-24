//! The published JSON Schema has to agree with the parser, in both directions.

use jsonschema::Validator;
use serde_json::Value;

fn validator() -> Validator {
    let schema = cratecv::json_schema();
    jsonschema::validator_for(&schema).expect("the generated schema is valid JSON Schema")
}

/// A resume as a JSON value, the way an editor or an assistant would hold it.
fn document(yaml: &str) -> Value {
    let node = marked_yaml::parse_yaml(0, yaml).expect("the fixture parses");
    to_json(&node)
}

/// marked-yaml reads every scalar as a string, which is all this schema has.
fn to_json(node: &marked_yaml::Node) -> Value {
    if let Some(scalar) = node.as_scalar() {
        return Value::String(scalar.as_str().to_owned());
    }
    if let Some(sequence) = node.as_sequence() {
        return Value::Array(sequence.iter().map(to_json).collect());
    }
    match node.as_mapping() {
        Some(mapping) => Value::Object(
            mapping
                .iter()
                .map(|(key, value)| (key.as_str().to_owned(), to_json(value)))
                .collect(),
        ),
        None => Value::Null,
    }
}

#[test]
fn the_example_validates() {
    let example = document(include_str!("../examples/resume.yaml"));
    let errors: Vec<String> = validator()
        .iter_errors(&example)
        .map(|why| format!("{} at {}", why, why.instance_path()))
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn the_starter_validates() {
    let starter = document(cratecv::STARTER_RESUME);
    assert!(validator().is_valid(&starter));
}

#[test]
fn the_schema_rejects_what_the_parser_rejects() {
    let validator = validator();

    let unknown = document("name: Ada\ncontact: {}\nsections:\n  - title: T\n    employer: X\n");
    assert!(!validator.is_valid(&unknown), "an unknown key");

    let empty = document("name: \"\"\ncontact: {}\nsections:\n  - title: T\n");
    assert!(!validator.is_valid(&empty), "an empty string");

    let sectionless = document("name: Ada\ncontact: {}\nsections: []\n");
    assert!(!validator.is_valid(&sectionless), "no sections");
}

#[test]
fn the_settings_block_is_described_too() {
    let schema = cratecv::json_schema();
    let cratecv = &schema["properties"]["cratecv"];
    assert!(
        !cratecv.is_null(),
        "an assistant should be able to set maxPages without guessing"
    );
    let defs = &schema["$defs"];
    assert!(
        !defs["CheckSettings"]["properties"]["maxPages"].is_null(),
        "{defs:#}"
    );
}
