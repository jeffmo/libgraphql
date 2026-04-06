use crate::schema::SchemaBuildError;
use crate::schema::SchemaBuildErrorKind;
use crate::schema::SchemaErrors;
use crate::span::Span;

// Verifies SchemaErrors collects and exposes multiple errors.
// Written by Claude Code, reviewed by a human.
#[test]
fn schema_errors_collection() {
    let errors = SchemaErrors::new(vec![
        SchemaBuildError::new(
            SchemaBuildErrorKind::DuplicateTypeDefinition {
                type_name: "User".to_string(),
            },
            Span::builtin(),
            vec![],
        ),
        SchemaBuildError::new(
            SchemaBuildErrorKind::DuplicateTypeDefinition {
                type_name: "Post".to_string(),
            },
            Span::builtin(),
            vec![],
        ),
    ]);

    assert_eq!(errors.len(), 2);
    assert_eq!(errors.errors().len(), 2);
}

// Verifies Display joins error messages.
// Written by Claude Code, reviewed by a human.
#[test]
fn schema_errors_display() {
    let errors = SchemaErrors::new(vec![
        SchemaBuildError::new(
            SchemaBuildErrorKind::DuplicateTypeDefinition {
                type_name: "User".to_string(),
            },
            Span::builtin(),
            vec![],
        ),
        SchemaBuildError::new(
            SchemaBuildErrorKind::NoQueryOperationTypeDefined,
            Span::builtin(),
            vec![],
        ),
    ]);

    let display = errors.to_string();
    assert!(display.contains("duplicate type definition `User`"));
    assert!(display.contains("no Query root operation type"));
}

// Verifies IntoIterator yields individual errors.
// Written by Claude Code, reviewed by a human.
#[test]
fn schema_errors_into_iter() {
    let errors = SchemaErrors::new(vec![
        SchemaBuildError::new(
            SchemaBuildErrorKind::DuplicateTypeDefinition {
                type_name: "A".to_string(),
            },
            Span::builtin(),
            vec![],
        ),
        SchemaBuildError::new(
            SchemaBuildErrorKind::DuplicateTypeDefinition {
                type_name: "B".to_string(),
            },
            Span::builtin(),
            vec![],
        ),
    ]);

    let collected: Vec<SchemaBuildError> = errors.into_iter().collect();
    assert_eq!(collected.len(), 2);
}

// Verifies &SchemaErrors IntoIterator yields references without
// consuming the collection.
// Written by Claude Code, reviewed by a human.
#[test]
fn schema_errors_ref_iter() {
    let errors = SchemaErrors::new(vec![
        SchemaBuildError::new(
            SchemaBuildErrorKind::DuplicateTypeDefinition {
                type_name: "A".to_string(),
            },
            Span::builtin(),
            vec![],
        ),
    ]);

    let mut count = 0;
    for _err in &errors {
        count += 1;
    }
    assert_eq!(count, 1);
    // errors is still usable (not consumed)
    assert_eq!(errors.len(), 1);
}
