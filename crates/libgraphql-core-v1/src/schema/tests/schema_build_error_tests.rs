use crate::error_note::ErrorNote;
use crate::schema::SchemaBuildError;
use crate::schema::SchemaBuildErrorKind;
use crate::span::Span;

// Verifies SchemaBuildError construction and accessors.
// Written by Claude Code, reviewed by a human.
#[test]
fn schema_build_error_accessors() {
    let error = SchemaBuildError::new(
        SchemaBuildErrorKind::DuplicateTypeDefinition {
            type_name: "User".to_string(),
        },
        Span::builtin(),
        vec![
            ErrorNote::general_with_span(
                "first defined here",
                Span::builtin(),
            ),
            ErrorNote::spec(
                "https://spec.graphql.org/September2025/#sec-Objects",
            ),
        ],
    );

    assert!(matches!(
        error.kind(),
        SchemaBuildErrorKind::DuplicateTypeDefinition { type_name }
            if type_name == "User",
    ));
    assert_eq!(error.span(), Span::builtin());
    assert_eq!(error.notes().len(), 2);
}

// Verifies Display delegates to the kind's thiserror message.
// Written by Claude Code, reviewed by a human.
#[test]
fn schema_build_error_display() {
    let error = SchemaBuildError::new(
        SchemaBuildErrorKind::DuplicateFieldNameDefinition {
            field_name: "id".to_string(),
            type_name: "User".to_string(),
        },
        Span::builtin(),
        vec![],
    );
    assert_eq!(
        error.to_string(),
        "duplicate field `id` on `User`",
    );
}

// Verifies TypeValidation wrapper variant displays the inner
// error's message.
// Written by Claude Code, reviewed by a human.
#[test]
fn type_validation_wrapper_display() {
    use crate::schema::TypeValidationError;
    use crate::schema::TypeValidationErrorKind;

    let inner = TypeValidationError::new(
        TypeValidationErrorKind::UndefinedTypeName {
            undefined_type_name: "Foo".to_string(),
        },
        Span::builtin(),
        vec![],
    );
    let error = SchemaBuildError::new(
        SchemaBuildErrorKind::TypeValidation(inner),
        Span::builtin(),
        vec![],
    );
    assert_eq!(
        error.to_string(),
        "type `Foo` is referenced but not defined",
    );
}
