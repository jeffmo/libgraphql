use crate::directive_annotation::DirectiveAnnotation;
use crate::names::DirectiveName;
use crate::names::FieldName;
use crate::span::Span;
use crate::value::Value;
use indexmap::IndexMap;

// Verifies DirectiveAnnotation accessor methods.
// https://spec.graphql.org/September2025/#sec-Language.Directives
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_annotation_accessors() {
    let mut args = IndexMap::new();
    args.insert(
        FieldName::new("reason"),
        Value::String("Use newField".to_string()),
    );
    let annotation = DirectiveAnnotation {
        arguments: args,
        name: DirectiveName::new("deprecated"),
        span: Span::builtin(),
    };

    assert_eq!(annotation.name().as_str(), "deprecated");
    assert_eq!(annotation.span(), Span::builtin());
    assert_eq!(annotation.arguments().len(), 1);
    assert_eq!(
        annotation.arguments().get("reason"),
        Some(&Value::String("Use newField".to_string())),
    );
}

// Verifies DirectiveAnnotation with no arguments.
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_annotation_no_args() {
    let annotation = DirectiveAnnotation {
        arguments: IndexMap::new(),
        name: DirectiveName::new("skip"),
        span: Span::builtin(),
    };
    assert_eq!(annotation.name().as_str(), "skip");
    assert!(annotation.arguments().is_empty());
}

// Verifies serde round-trip via bincode for DirectiveAnnotation.
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_annotation_serde_roundtrip() {
    let mut args = IndexMap::new();
    args.insert(FieldName::new("if"), Value::Boolean(true));
    let annotation = DirectiveAnnotation {
        arguments: args,
        name: DirectiveName::new("include"),
        span: Span::builtin(),
    };
    let bytes = bincode::serde::encode_to_vec(
        &annotation,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (DirectiveAnnotation, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(annotation, deserialized);
}
