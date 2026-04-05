use crate::names::EnumValueName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::EnumType;
use crate::types::EnumValue;
use indexmap::IndexMap;

fn sample_enum_type() -> EnumType {
    let mut values = IndexMap::new();
    values.insert(EnumValueName::new("ACTIVE"), EnumValue {
        description: Some("Currently active".to_string()),
        directives: vec![],
        name: EnumValueName::new("ACTIVE"),
        parent_type_name: TypeName::new("Status"),
        span: Span::builtin(),
    });
    values.insert(EnumValueName::new("INACTIVE"), EnumValue {
        description: None,
        directives: vec![],
        name: EnumValueName::new("INACTIVE"),
        parent_type_name: TypeName::new("Status"),
        span: Span::builtin(),
    });
    EnumType {
        description: Some("User account status".to_string()),
        directives: vec![],
        name: TypeName::new("Status"),
        span: Span::builtin(),
        values,
    }
}

// Verifies EnumType accessors return correct values.
// https://spec.graphql.org/September2025/#sec-Enums
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_type_accessors() {
    let enum_type = sample_enum_type();
    assert_eq!(enum_type.name().as_str(), "Status");
    assert_eq!(enum_type.description(), Some("User account status"));
    assert_eq!(enum_type.values().len(), 2);
}

// Verifies EnumType::value() lookup by &str key.
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_type_value_lookup() {
    let enum_type = sample_enum_type();
    let active = enum_type.value("ACTIVE");
    assert!(active.is_some());
    assert_eq!(active.unwrap().name().as_str(), "ACTIVE");
    assert_eq!(
        active.unwrap().description(),
        Some("Currently active"),
    );

    assert!(enum_type.value("NONEXISTENT").is_none());
}

// Verifies EnumValue::parent_type_name() tracks the owning enum.
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_parent_type_name() {
    let enum_type = sample_enum_type();
    let active = enum_type.value("ACTIVE").unwrap();
    assert_eq!(active.parent_type_name().as_str(), "Status");
}

// Verifies serde round-trip for EnumType via bincode.
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_type_serde_roundtrip() {
    let enum_type = sample_enum_type();
    let bytes = bincode::serde::encode_to_vec(
        &enum_type,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (EnumType, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(enum_type, deserialized);
}
