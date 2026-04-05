use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::InputField;
use crate::types::InputObjectType;
use crate::types::TypeAnnotation;
use crate::value::Value;
use indexmap::IndexMap;

fn sample_input_object() -> InputObjectType {
    let mut fields = IndexMap::new();
    fields.insert(FieldName::new("name"), InputField {
        default_value: None,
        description: Some("The user's name".to_string()),
        directives: vec![],
        name: FieldName::new("name"),
        parent_type_name: TypeName::new("CreateUserInput"),
        span: Span::builtin(),
        type_annotation: TypeAnnotation::named("String", false),
    });
    fields.insert(FieldName::new("age"), InputField {
        default_value: Some(Value::Int(18)),
        description: None,
        directives: vec![],
        name: FieldName::new("age"),
        parent_type_name: TypeName::new("CreateUserInput"),
        span: Span::builtin(),
        type_annotation: TypeAnnotation::named("Int", true),
    });
    InputObjectType {
        description: Some("Input for creating a user".to_string()),
        directives: vec![],
        fields,
        name: TypeName::new("CreateUserInput"),
        span: Span::builtin(),
    }
}

// Verifies InputObjectType accessors and field lookup by &str.
// https://spec.graphql.org/September2025/#sec-Input-Objects
// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_type_accessors() {
    let input = sample_input_object();
    assert_eq!(input.name().as_str(), "CreateUserInput");
    assert_eq!(input.description(), Some("Input for creating a user"));
    assert_eq!(input.fields().len(), 2);
    assert!(input.field("name").is_some());
    assert!(input.field("age").is_some());
    assert!(input.field("nonexistent").is_none());
}

// Verifies InputField accessors including default_value and
// parent_type_name.
// Written by Claude Code, reviewed by a human.
#[test]
fn input_field_accessors() {
    let input = sample_input_object();
    let name_field = input.field("name").unwrap();
    assert_eq!(name_field.name().as_str(), "name");
    assert_eq!(name_field.description(), Some("The user's name"));
    assert_eq!(name_field.parent_type_name().as_str(), "CreateUserInput");
    assert!(name_field.default_value().is_none());

    let age_field = input.field("age").unwrap();
    assert_eq!(age_field.default_value(), Some(&Value::Int(18)));
}

// Verifies serde round-trip for InputObjectType via bincode.
// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_type_serde_roundtrip() {
    let input = sample_input_object();
    let bytes = bincode::serde::encode_to_vec(
        &input,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (InputObjectType, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(input, deserialized);
}
