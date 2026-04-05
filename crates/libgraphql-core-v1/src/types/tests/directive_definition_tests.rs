use crate::names::DirectiveName;
use crate::names::FieldName;
use crate::span::Span;
use crate::types::DirectiveDefinition;
use crate::types::DirectiveDefinitionKind;
use crate::types::DirectiveLocationKind;
use crate::types::ParameterDefinition;
use crate::types::TypeAnnotation;
use indexmap::IndexMap;

// Verifies DirectiveDefinitionKind discriminates all built-ins.
// https://spec.graphql.org/September2025/#sec-Type-System.Directives.Built-in-Directives
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_kind_builtin() {
    assert!(DirectiveDefinitionKind::Deprecated.is_builtin());
    assert!(DirectiveDefinitionKind::Include.is_builtin());
    assert!(DirectiveDefinitionKind::OneOf.is_builtin());
    assert!(DirectiveDefinitionKind::Skip.is_builtin());
    assert!(DirectiveDefinitionKind::SpecifiedBy.is_builtin());
    assert!(!DirectiveDefinitionKind::Custom.is_builtin());
}

// Verifies DirectiveDefinition accessors for a built-in directive.
// Written by Claude Code, reviewed by a human.
#[test]
fn builtin_directive_accessors() {
    let mut params = IndexMap::new();
    params.insert(FieldName::new("if"), ParameterDefinition {
        default_value: None,
        description: None,
        name: FieldName::new("if"),
        span: Span::builtin(),
        type_annotation: TypeAnnotation::named("Boolean", false),
    });
    let skip = DirectiveDefinition {
        description: None,
        is_repeatable: false,
        kind: DirectiveDefinitionKind::Skip,
        locations: vec![
            DirectiveLocationKind::Field,
            DirectiveLocationKind::FragmentSpread,
            DirectiveLocationKind::InlineFragment,
        ],
        name: DirectiveName::new("skip"),
        parameters: params,
        span: Span::builtin(),
    };
    assert_eq!(skip.name().as_str(), "skip");
    assert!(skip.is_builtin());
    assert!(!skip.is_repeatable());
    assert_eq!(skip.locations().len(), 3);
    assert_eq!(skip.parameters().len(), 1);
    assert!(skip.parameters().get("if").is_some());
}

// Verifies serde round-trip for DirectiveDefinition via bincode.
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_definition_serde_roundtrip() {
    let dir = DirectiveDefinition {
        description: Some("Marks a field as deprecated".to_string()),
        is_repeatable: false,
        kind: DirectiveDefinitionKind::Deprecated,
        locations: vec![
            DirectiveLocationKind::FieldDefinition,
            DirectiveLocationKind::EnumValue,
        ],
        name: DirectiveName::new("deprecated"),
        parameters: IndexMap::new(),
        span: Span::builtin(),
    };
    let bytes = bincode::serde::encode_to_vec(
        &dir,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (DirectiveDefinition, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(dir, deserialized);
}
