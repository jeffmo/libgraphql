use crate::tests::graphql_schema_parser::parse_error_tests::utils::parse_expecting_error;
use libgraphql_parser::GraphQLParseErrorKind;

#[test]
fn test_unexpected_token_in_type_definition() {
    let schema = "type % Query { }";
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
    assert!(matches!(
        errors[0].kind(),
        GraphQLParseErrorKind::LexerError
    ));
}

#[test]
fn test_unexpected_token_instead_of_field_type() {
    let schema = r#"
        type Query {
            field: %
        }
    "#;
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
}

#[test]
fn test_unexpected_token_in_implements() {
    let schema = "type User implements % { id: ID }";
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
}

#[test]
fn test_unexpected_punctuator_instead_of_name() {
    let schema = "type { field: String }";
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
    assert!(matches!(
        errors[0].kind(),
        GraphQLParseErrorKind::UnexpectedToken { .. }
    ));
}

#[test]
fn test_missing_colon_in_field() {
    let schema = r#"
        type Query {
            field String
        }
    "#;
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
}

#[test]
fn test_missing_equals_in_union() {
    let schema = "union SearchResult User | Post";
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
}

#[test]
fn test_invalid_directive_location() {
    let schema = "directive @test on INVALID_LOCATION";
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
    assert!(
        errors[0].message().contains("INVALID_LOCATION"),
        "Expected error message to mention INVALID_LOCATION, \
         got: {}",
        errors[0].message(),
    );
    assert!(matches!(
        errors[0].kind(),
        GraphQLParseErrorKind::InvalidSyntax,
    ));
}
