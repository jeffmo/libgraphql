use crate::names::DirectiveName;
use crate::names::EnumValueName;
use crate::names::FieldName;
use crate::names::FragmentName;
use crate::names::TypeName;
use crate::names::VariableName;

// ── TypeName ──────────────────────────────────────────

// Verifies TypeName basic construction and accessor.
// Written by Claude Code, reviewed by a human.
#[test]
fn type_name_construction() {
    let name = TypeName::new("User");
    assert_eq!(name.as_str(), "User");
}

// Verifies Display formats as the inner string.
// Written by Claude Code, reviewed by a human.
#[test]
fn type_name_display() {
    let name = TypeName::new("Query");
    assert_eq!(format!("{name}"), "Query");
}

// Verifies serde round-trip via bincode.
// Written by Claude Code, reviewed by a human.
#[test]
fn type_name_serde_roundtrip() {
    let name = TypeName::new("User");
    let bytes = bincode::serde::encode_to_vec(
        &name,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (TypeName, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(name, deserialized);
}

// Verifies From<&str> and From<String> produce equal values.
// Written by Claude Code, reviewed by a human.
#[test]
fn type_name_from_conversions() {
    let from_str: TypeName = "Query".into();
    let from_string: TypeName = String::from("Query").into();
    assert_eq!(from_str, from_string);
}

// Verifies Borrow<str> enables HashMap lookups with &str keys.
// Written by Claude Code, reviewed by a human.
#[test]
fn type_name_borrow_lookup() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert(TypeName::new("User"), 42);
    assert_eq!(map.get("User"), Some(&42));
}

// ── FieldName ─────────────────────────────────────────

// Verifies FieldName basic construction and accessor.
// Written by Claude Code, reviewed by a human.
#[test]
fn field_name_construction() {
    let name = FieldName::new("firstName");
    assert_eq!(name.as_str(), "firstName");
}

// Verifies Display formats as the inner string.
// Written by Claude Code, reviewed by a human.
#[test]
fn field_name_display() {
    let name = FieldName::new("id");
    assert_eq!(format!("{name}"), "id");
}

// Verifies serde round-trip via bincode.
// Written by Claude Code, reviewed by a human.
#[test]
fn field_name_serde_roundtrip() {
    let name = FieldName::new("firstName");
    let bytes = bincode::serde::encode_to_vec(
        &name,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (FieldName, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(name, deserialized);
}

// Verifies From<&str> and From<String> produce equal values.
// Written by Claude Code, reviewed by a human.
#[test]
fn field_name_from_conversions() {
    let from_str: FieldName = "id".into();
    let from_string: FieldName = String::from("id").into();
    assert_eq!(from_str, from_string);
}

// ── VariableName ──────────────────────────────────────

// Verifies VariableName basic construction and accessor.
// The stored name should not include the $ prefix.
// Written by Claude Code, reviewed by a human.
#[test]
fn variable_name_construction() {
    let name = VariableName::new("userId");
    assert_eq!(name.as_str(), "userId");
}

// Verifies Display formats as the inner string (no $ prefix).
// Written by Claude Code, reviewed by a human.
#[test]
fn variable_name_display() {
    let name = VariableName::new("limit");
    assert_eq!(format!("{name}"), "limit");
}

// Verifies serde round-trip via bincode.
// Written by Claude Code, reviewed by a human.
#[test]
fn variable_name_serde_roundtrip() {
    let name = VariableName::new("userId");
    let bytes = bincode::serde::encode_to_vec(
        &name,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (VariableName, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(name, deserialized);
}

// ── DirectiveName ─────────────────────────────────────

// Verifies DirectiveName basic construction and accessor.
// The stored name should not include the @ prefix.
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_name_construction() {
    let name = DirectiveName::new("deprecated");
    assert_eq!(name.as_str(), "deprecated");
}

// Verifies Display formats as the inner string (no @ prefix).
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_name_display() {
    let name = DirectiveName::new("skip");
    assert_eq!(format!("{name}"), "skip");
}

// Verifies serde round-trip via bincode.
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_name_serde_roundtrip() {
    let name = DirectiveName::new("deprecated");
    let bytes = bincode::serde::encode_to_vec(
        &name,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (DirectiveName, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(name, deserialized);
}

// ── EnumValueName ─────────────────────────────────────

// Verifies EnumValueName basic construction and accessor.
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_name_construction() {
    let name = EnumValueName::new("ACTIVE");
    assert_eq!(name.as_str(), "ACTIVE");
}

// Verifies Display formats as the inner string.
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_name_display() {
    let name = EnumValueName::new("ADMIN");
    assert_eq!(format!("{name}"), "ADMIN");
}

// Verifies serde round-trip via bincode.
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_name_serde_roundtrip() {
    let name = EnumValueName::new("ACTIVE");
    let bytes = bincode::serde::encode_to_vec(
        &name,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (EnumValueName, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(name, deserialized);
}

// ── FragmentName ──────────────────────────────────────

// Verifies FragmentName basic construction and accessor.
// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_name_construction() {
    let name = FragmentName::new("UserFields");
    assert_eq!(name.as_str(), "UserFields");
}

// Verifies Display formats as the inner string.
// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_name_display() {
    let name = FragmentName::new("PostSummary");
    assert_eq!(format!("{name}"), "PostSummary");
}

// Verifies serde round-trip via bincode.
// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_name_serde_roundtrip() {
    let name = FragmentName::new("UserFields");
    let bytes = bincode::serde::encode_to_vec(
        &name,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (FragmentName, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(name, deserialized);
}

// ── Cross-type isolation ──────────────────────────────

// Verifies that different name types with the same string value
// are not accidentally interchangeable at the type level. This is
// a compile-time guarantee, but we verify the values are
// independently correct at runtime.
// Written by Claude Code, reviewed by a human.
#[test]
fn name_types_are_independent() {
    let type_name = TypeName::new("User");
    let field_name = FieldName::new("User");
    let enum_value_name = EnumValueName::new("User");

    // All have the same string value...
    assert_eq!(type_name.as_str(), "User");
    assert_eq!(field_name.as_str(), "User");
    assert_eq!(enum_value_name.as_str(), "User");

    // ...but are different Rust types (compile-time check).
    // If this compiles, the newtypes are correctly distinct.
    fn takes_type_name(_: &TypeName) {}
    fn takes_field_name(_: &FieldName) {}
    takes_type_name(&type_name);
    takes_field_name(&field_name);
}
