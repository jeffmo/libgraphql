use crate::names::TypeName;
use crate::span::Span;
use crate::types::ScalarKind;
use crate::types::ScalarType;

// Verifies custom scalars are not built-in.
// Written by Claude Code, reviewed by a human.
#[test]
fn custom_scalar_not_builtin() {
    let scalar = ScalarType {
        description: None,
        directives: vec![],
        kind: ScalarKind::Custom,
        name: TypeName::new("DateTime"),
        span: Span::builtin(),
    };
    assert!(!scalar.is_builtin());
}

// Verifies all built-in scalar kinds report is_builtin() = true.
// Written by Claude Code, reviewed by a human.
#[test]
fn all_builtin_kinds() {
    for (kind, name) in [
        (ScalarKind::Boolean, "Boolean"),
        (ScalarKind::Float, "Float"),
        (ScalarKind::ID, "ID"),
        (ScalarKind::Int, "Int"),
        (ScalarKind::String, "String"),
    ] {
        let scalar = ScalarType {
            description: None,
            directives: vec![],
            kind,
            name: TypeName::new(name),
            span: Span::builtin(),
        };
        assert!(
            scalar.is_builtin(),
            "{name} should be built-in",
        );
    }
}

// Verifies ScalarType accessors.
// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_type_accessors() {
    let scalar = ScalarType {
        description: Some("A date-time string".to_string()),
        directives: vec![],
        kind: ScalarKind::Custom,
        name: TypeName::new("DateTime"),
        span: Span::builtin(),
    };
    assert_eq!(scalar.name().as_str(), "DateTime");
    assert_eq!(scalar.description(), Some("A date-time string"));
    assert_eq!(scalar.kind(), ScalarKind::Custom);
    assert!(scalar.directives().is_empty());
    assert_eq!(scalar.span(), Span::builtin());
}
