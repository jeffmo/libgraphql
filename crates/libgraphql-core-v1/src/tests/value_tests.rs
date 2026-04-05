use crate::names::EnumValueName;
use crate::names::VariableName;
use crate::value::Value;
use indexmap::IndexMap;

// Verifies all Value variants can be constructed.
// https://spec.graphql.org/September2025/#sec-Input-Values
// Written by Claude Code, reviewed by a human.
#[test]
fn value_variant_construction() {
    assert_eq!(Value::Boolean(true), Value::Boolean(true));
    assert_eq!(Value::Int(42), Value::Int(42));
    assert_eq!(Value::Float(3.14), Value::Float(3.14));
    assert_eq!(
        Value::String("hello".to_string()),
        Value::String("hello".to_string()),
    );
    assert_eq!(Value::Null, Value::Null);
    assert_eq!(
        Value::Enum(EnumValueName::new("ACTIVE")),
        Value::Enum(EnumValueName::new("ACTIVE")),
    );
    assert_eq!(
        Value::VarRef(VariableName::new("userId")),
        Value::VarRef(VariableName::new("userId")),
    );
}

// Verifies nested Value::List construction.
// Written by Claude Code, reviewed by a human.
#[test]
fn value_list() {
    let list = Value::List(vec![
        Value::Int(1),
        Value::Int(2),
        Value::String("three".to_string()),
    ]);
    if let Value::List(items) = &list {
        assert_eq!(items.len(), 3);
    } else {
        panic!("expected Value::List");
    }
}

// Verifies Value::Object construction with IndexMap.
// Written by Claude Code, reviewed by a human.
#[test]
fn value_object() {
    let mut fields = IndexMap::new();
    fields.insert("name".to_string(), Value::String("Alice".to_string()));
    fields.insert("age".to_string(), Value::Int(30));
    let obj = Value::Object(fields);
    if let Value::Object(map) = &obj {
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("name"), Some(&Value::String("Alice".to_string())));
    } else {
        panic!("expected Value::Object");
    }
}

// Verifies serde round-trip via bincode for Value.
// Written by Claude Code, reviewed by a human.
#[test]
fn value_serde_roundtrip() {
    let value = Value::List(vec![
        Value::Int(1),
        Value::String("hello".to_string()),
        Value::Null,
    ]);
    let bytes = bincode::serde::encode_to_vec(
        &value,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (Value, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(value, deserialized);
}
