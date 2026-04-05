use crate::located::Located;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::FieldDefinition;
use crate::types::ObjectType;
use crate::types::TypeAnnotation;
use crate::types::FieldedTypeData;
use indexmap::IndexMap;

fn sample_object_type() -> ObjectType {
    let mut fields = IndexMap::new();
    fields.insert(FieldName::new("id"), FieldDefinition {
        description: None,
        directives: vec![],
        name: FieldName::new("id"),
        parameters: IndexMap::new(),
        parent_type_name: TypeName::new("User"),
        span: Span::builtin(),
        type_annotation: TypeAnnotation::named("ID", false),
    });
    fields.insert(FieldName::new("name"), FieldDefinition {
        description: None,
        directives: vec![],
        name: FieldName::new("name"),
        parameters: IndexMap::new(),
        parent_type_name: TypeName::new("User"),
        span: Span::builtin(),
        type_annotation: TypeAnnotation::named("String", true),
    });
    ObjectType(FieldedTypeData {
        description: Some("A user account".to_string()),
        directives: vec![],
        fields,
        interfaces: vec![
            Located {
                value: TypeName::new("Node"),
                span: Span::builtin(),
            },
        ],
        name: TypeName::new("User"),
        span: Span::builtin(),
    })
}

// Verifies HasFieldsAndInterfaces trait methods on ObjectType
// delegate correctly through FieldedTypeData accessors.
// https://spec.graphql.org/September2025/#sec-Objects
// Written by Claude Code, reviewed by a human.
#[test]
fn object_type_trait_delegation() {
    let obj = sample_object_type();
    assert_eq!(obj.name().as_str(), "User");
    assert_eq!(obj.description(), Some("A user account"));
    assert_eq!(obj.fields().len(), 2);
    assert!(obj.field("id").is_some());
    assert!(obj.field("name").is_some());
    assert!(obj.field("nonexistent").is_none());
    assert_eq!(obj.interfaces().len(), 1);
    assert_eq!(obj.interfaces()[0].value.as_str(), "Node");
    assert!(obj.directives().is_empty());
}

// Verifies serde round-trip for ObjectType via bincode.
// Written by Claude Code, reviewed by a human.
#[test]
fn object_type_serde_roundtrip() {
    let obj = sample_object_type();
    let bytes = bincode::serde::encode_to_vec(
        &obj,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (ObjectType, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(obj, deserialized);
}
