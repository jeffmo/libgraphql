use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::FieldDefinition;
use crate::types::ParameterDefinition;
use crate::types::TypeAnnotation;
use crate::value::Value;
use indexmap::IndexMap;

// Verifies return_type_name() delegates through TypeAnnotation to
// get the innermost type name.
// https://spec.graphql.org/September2025/#FieldsDefinition
// Written by Claude Code, reviewed by a human.
#[test]
fn return_type_name_unwraps_list() {
    let field = FieldDefinition {
        description: None,
        directives: vec![],
        name: FieldName::new("friends"),
        parameters: IndexMap::new(),
        parent_type_name: TypeName::new("User"),
        span: Span::builtin(),
        type_annotation: TypeAnnotation::list(
            TypeAnnotation::named("User", false),
            true,
        ),
    };
    assert_eq!(field.return_type_name().as_str(), "User");
}

// Verifies FieldDefinition accessors.
// Written by Claude Code, reviewed by a human.
#[test]
fn field_definition_accessors() {
    let mut params = IndexMap::new();
    params.insert(FieldName::new("first"), ParameterDefinition {
        default_value: Some(Value::Int(10)),
        description: None,
        directives: vec![],
        name: FieldName::new("first"),
        span: Span::builtin(),
        type_annotation: TypeAnnotation::named("Int", true),
    });
    let field = FieldDefinition {
        description: Some("A list of friends".to_string()),
        directives: vec![],
        name: FieldName::new("friends"),
        parameters: params,
        parent_type_name: TypeName::new("User"),
        span: Span::builtin(),
        type_annotation: TypeAnnotation::list(
            TypeAnnotation::named("User", false),
            true,
        ),
    };
    assert_eq!(field.name().as_str(), "friends");
    assert_eq!(field.description(), Some("A list of friends"));
    assert_eq!(field.parent_type_name().as_str(), "User");
    assert_eq!(field.parameters().len(), 1);
    assert!(field.parameters().get("first").is_some());
    assert_eq!(
        field.parameters().get("first").unwrap().default_value(),
        Some(&Value::Int(10)),
    );
}
