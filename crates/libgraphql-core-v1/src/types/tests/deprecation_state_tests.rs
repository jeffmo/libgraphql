use crate::directive_annotation::DirectiveAnnotation;
use crate::names::DirectiveName;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::DeprecationState;
use crate::types::FieldDefinition;
use crate::types::InputField;
use crate::types::ParameterDefinition;
use crate::types::TypeAnnotation;
use crate::value::Value;
use indexmap::IndexMap;

fn deprecated_annotation(reason: Option<Value>) -> DirectiveAnnotation {
    let mut arguments = IndexMap::new();
    if let Some(reason) = reason {
        arguments.insert(FieldName::new("reason"), reason);
    }
    DirectiveAnnotation {
        arguments,
        name: DirectiveName::new("deprecated"),
        span: Span::dummy(),
    }
}

fn make_field(directives: Vec<DirectiveAnnotation>) -> FieldDefinition {
    FieldDefinition {
        description: None,
        directives,
        name: FieldName::new("oldField"),
        parameters: IndexMap::new(),
        parent_type_name: TypeName::new("SomeType"),
        span: Span::dummy(),
        type_annotation: TypeAnnotation::named(
            "String",
            /* nullable = */ true,
        ),
    }
}

fn make_param(directives: Vec<DirectiveAnnotation>) -> ParameterDefinition {
    ParameterDefinition {
        default_value: None,
        description: None,
        directives,
        name: FieldName::new("oldArg"),
        span: Span::dummy(),
        type_annotation: TypeAnnotation::named(
            "String",
            /* nullable = */ true,
        ),
    }
}

fn make_input_field(directives: Vec<DirectiveAnnotation>) -> InputField {
    InputField {
        default_value: None,
        description: None,
        directives,
        name: FieldName::new("oldInputField"),
        parent_type_name: TypeName::new("SomeInput"),
        span: Span::dummy(),
        type_annotation: TypeAnnotation::named(
            "String",
            /* nullable = */ true,
        ),
    }
}

// Verifies Active state is not deprecated.
// https://spec.graphql.org/September2025/#sec--deprecated
// Written by Claude Code, reviewed by a human.
#[test]
fn active_not_deprecated() {
    let state = DeprecationState::Active;
    assert!(!state.is_deprecated());
}

// Verifies Deprecated state without reason.
// Written by Claude Code, reviewed by a human.
#[test]
fn deprecated_without_reason() {
    let state = DeprecationState::Deprecated { reason: None };
    assert!(state.is_deprecated());
}

// Verifies that FieldDefinition::deprecation_state() returns
// Active when no `@deprecated` annotation is present, including
// when other (non-deprecation) directives are applied.
// https://spec.graphql.org/September2025/#sec--deprecated
// Written by Claude Code, reviewed by a human.
#[test]
fn field_definition_without_deprecated_is_active() {
    let field = make_field(vec![]);
    assert_eq!(field.deprecation_state(), DeprecationState::Active);

    let other_directive = DirectiveAnnotation {
        arguments: IndexMap::new(),
        name: DirectiveName::new("someOtherDirective"),
        span: Span::dummy(),
    };
    let field = make_field(vec![other_directive]);
    assert_eq!(field.deprecation_state(), DeprecationState::Active);
}

// Verifies that FieldDefinition::deprecation_state() extracts an
// explicitly-provided `reason` argument from the `@deprecated`
// annotation.
// https://spec.graphql.org/September2025/#sec--deprecated
// Written by Claude Code, reviewed by a human.
#[test]
fn field_definition_deprecated_with_explicit_reason() {
    let field = make_field(vec![deprecated_annotation(Some(
        Value::String("Use `newField`.".to_string()),
    ))]);
    assert_eq!(
        field.deprecation_state(),
        DeprecationState::Deprecated {
            reason: Some("Use `newField`."),
        },
    );
}

// Verifies that FieldDefinition::deprecation_state() applies the
// spec-defined default reason ("No longer supported") when the
// `@deprecated` annotation omits its `reason` argument, per the
// built-in definition
// `directive @deprecated(reason: String! = "No longer supported")`.
// https://spec.graphql.org/September2025/#sec--deprecated
// Written by Claude Code, reviewed by a human.
#[test]
fn field_definition_deprecated_with_default_reason() {
    let field = make_field(vec![deprecated_annotation(None)]);
    assert_eq!(
        field.deprecation_state(),
        DeprecationState::Deprecated {
            reason: Some(DeprecationState::DEFAULT_REASON),
        },
    );
    assert_eq!(
        DeprecationState::DEFAULT_REASON,
        "No longer supported",
    );
}

// Verifies that FieldDefinition::deprecation_state() yields a
// reason of None when the `reason` argument is explicitly `null`.
// (An explicit `null` is invalid per the `String!` parameter type,
// but the accessor tolerates it rather than panicking; argument
// value coercion is validated elsewhere.)
// https://spec.graphql.org/September2025/#sec--deprecated
// Written by Claude Code, reviewed by a human.
#[test]
fn field_definition_deprecated_with_null_reason() {
    let field = make_field(vec![deprecated_annotation(Some(Value::Null))]);
    assert_eq!(
        field.deprecation_state(),
        DeprecationState::Deprecated { reason: None },
    );
}

// Verifies that ParameterDefinition::deprecation_state() reports
// Active with no annotation and Deprecated (with extracted
// reason) when `@deprecated` is applied.
// https://spec.graphql.org/September2025/#sec--deprecated
// Written by Claude Code, reviewed by a human.
#[test]
fn parameter_definition_deprecation_state() {
    let param = make_param(vec![]);
    assert_eq!(param.deprecation_state(), DeprecationState::Active);

    let param = make_param(vec![deprecated_annotation(Some(
        Value::String("Use `newArg`.".to_string()),
    ))]);
    assert_eq!(
        param.deprecation_state(),
        DeprecationState::Deprecated {
            reason: Some("Use `newArg`."),
        },
    );
}

// Verifies that InputField::deprecation_state() reports Active
// with no annotation, and Deprecated with the spec-defined
// default reason when `@deprecated` is applied without a
// `reason` argument.
// https://spec.graphql.org/September2025/#sec--deprecated
// Written by Claude Code, reviewed by a human.
#[test]
fn input_field_deprecation_state() {
    let input_field = make_input_field(vec![]);
    assert_eq!(
        input_field.deprecation_state(),
        DeprecationState::Active,
    );

    let input_field = make_input_field(vec![deprecated_annotation(None)]);
    assert_eq!(
        input_field.deprecation_state(),
        DeprecationState::Deprecated {
            reason: Some(DeprecationState::DEFAULT_REASON),
        },
    );
}
