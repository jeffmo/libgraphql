use crate::tests::graphql_schema_parser::parse_error_tests::utils::parse_expecting_error;

#[test]
fn test_unclosed_brace_in_type() {
    let schema = r#"
        type Query {
            field: String
    "#;
    let errors = parse_expecting_error(schema);

    // Note: Unclosed braces are caught by Rust's tokenizer
    // so we just verify an error was produced
    assert!(errors.has_errors());
}

#[test]
fn test_unclosed_brace_in_enum() {
    let schema = r#"
        enum Status {
            ACTIVE
            INACTIVE
    "#;
    let errors = parse_expecting_error(schema);

    assert!(errors.has_errors());
}

#[test]
fn test_unclosed_brace_in_input() {
    let schema = r#"
        input CreateUserInput {
            name: String!
    "#;
    let errors = parse_expecting_error(schema);

    assert!(errors.has_errors());
}

#[test]
fn test_unclosed_brace_in_schema() {
    let schema = r#"
        schema {
            query: Query
    "#;
    let errors = parse_expecting_error(schema);

    assert!(errors.has_errors());
}

#[test]
fn test_unclosed_paren_in_field_arguments() {
    let schema = r#"
        type Query {
            field(arg: String: String
        }
    "#;
    let errors = parse_expecting_error(schema);

    assert!(errors.has_errors());
}

#[test]
fn test_unclosed_bracket_in_list_type() {
    let schema = r#"
        type Query {
            field: [String
        }
    "#;
    let errors = parse_expecting_error(schema);

    assert!(errors.has_errors());
}
