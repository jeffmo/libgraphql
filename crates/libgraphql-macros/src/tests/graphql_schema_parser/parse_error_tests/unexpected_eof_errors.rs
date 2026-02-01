use crate::tests::graphql_schema_parser::parse_error_tests::utils::parse_expecting_error;
use libgraphql_parser::GraphQLParseErrorKind;

#[test]
fn test_unexpected_eof_in_type_definition() {
    let schema = "type Query";
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
    assert!(matches!(
        errors[0].kind(),
        GraphQLParseErrorKind::UnexpectedEof { .. }
    ));
}

#[test]
fn test_unexpected_eof_after_field_name() {
    let schema = r#"
        type Query {
            field
    "#;
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
}

#[test]
fn test_unexpected_eof_in_field_arguments() {
    let schema = r#"
        type Query {
            field(arg: String
    "#;
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
}

#[test]
fn test_unexpected_eof_in_implements() {
    let schema = "type User implements";
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
}

#[test]
fn test_unexpected_eof_in_union() {
    let schema = "union SearchResult =";
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
}

#[test]
fn test_unexpected_eof_in_directive_definition() {
    let schema = "directive @test on";
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
}

#[test]
fn test_unexpected_eof_after_enum_keyword() {
    let schema = "enum";
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
}

#[test]
fn test_unexpected_eof_in_schema_definition() {
    let schema = r#"
        schema {
            query:
    "#;
    let errors = parse_expecting_error(schema);

    assert!(!errors.is_empty());
}
