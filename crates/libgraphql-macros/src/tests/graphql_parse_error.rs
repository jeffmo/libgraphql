use crate::graphql_parse_error::{GraphQLParseError, GraphQLParseErrorKind, GraphQLParseErrors};
use proc_macro2::Span;

#[test]
fn test_parse_error_creation() {
    let error = GraphQLParseError::new(
        "Test error".to_string(),
        Span::call_site(),
        GraphQLParseErrorKind::InvalidSyntax,
    );

    assert_eq!(error.message, "Test error");
    assert_eq!(error.spans.len(), 1);
    assert!(matches!(error.kind, GraphQLParseErrorKind::InvalidSyntax));
}

#[test]
fn test_parse_error_with_multiple_spans() {
    let spans = vec![Span::call_site(), Span::call_site()];
    let error = GraphQLParseError::with_spans(
        "Test error".to_string(),
        spans.clone(),
        GraphQLParseErrorKind::InvalidSyntax,
    );

    assert_eq!(error.spans.len(), 2);
}

#[test]
fn test_parse_errors_collection() {
    let mut errors = GraphQLParseErrors::new();
    assert!(!errors.has_errors());
    assert_eq!(errors.len(), 0);

    errors.add(GraphQLParseError::new(
        "Error 1".to_string(),
        Span::call_site(),
        GraphQLParseErrorKind::InvalidSyntax,
    ));

    assert!(errors.has_errors());
    assert_eq!(errors.len(), 1);

    errors.add(GraphQLParseError::new(
        "Error 2".to_string(),
        Span::call_site(),
        GraphQLParseErrorKind::UnexpectedEof {
            expected: vec!["type".to_string()],
        },
    ));

    assert_eq!(errors.len(), 2);
}

#[test]
fn test_compile_error_generation() {
    let error = GraphQLParseError::new(
        "Test compile error".to_string(),
        Span::call_site(),
        GraphQLParseErrorKind::InvalidSyntax,
    );

    let compile_error = error.into_compile_error();
    let output = compile_error.to_string();

    // Verify that it contains compile_error! macro call
    assert!(output.contains("compile_error"));
    assert!(output.contains("Test compile error"));
}

#[test]
fn test_multiple_compile_errors() {
    let mut errors = GraphQLParseErrors::new();

    errors.add(GraphQLParseError::new(
        "Error 1".to_string(),
        Span::call_site(),
        GraphQLParseErrorKind::InvalidSyntax,
    ));

    errors.add(GraphQLParseError::new(
        "Error 2".to_string(),
        Span::call_site(),
        GraphQLParseErrorKind::InvalidSyntax,
    ));

    let compile_errors = errors.into_compile_errors();
    let output = compile_errors.to_string();

    // Should contain both error messages
    assert!(output.contains("Error 1"));
    assert!(output.contains("Error 2"));
}

#[test]
fn test_error_kind_variants() {
    // Test that all error kind variants can be constructed
    let unexpected_token = GraphQLParseErrorKind::UnexpectedToken {
        expected: vec!["type".to_string()],
        found: "{".to_string(),
    };
    assert!(matches!(unexpected_token, GraphQLParseErrorKind::UnexpectedToken { .. }));

    let unexpected_eof = GraphQLParseErrorKind::UnexpectedEof {
        expected: vec!["identifier".to_string()],
    };
    assert!(matches!(unexpected_eof, GraphQLParseErrorKind::UnexpectedEof { .. }));

    let duplicate = GraphQLParseErrorKind::DuplicateDefinition {
        name: "User".to_string(),
    };
    assert!(matches!(duplicate, GraphQLParseErrorKind::DuplicateDefinition { .. }));
}
