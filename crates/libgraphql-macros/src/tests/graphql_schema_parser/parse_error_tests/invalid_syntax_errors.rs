use crate::tests::graphql_schema_parser::parse_error_tests::utils::parse_expecting_error;

#[test]
fn test_invalid_operation_type_in_schema() {
    let schema = r#"
        schema {
            invalid: Query
        }
    "#;
    let errors = parse_expecting_error(schema);

    assert!(errors.has_errors());
}

#[test]
fn test_empty_union_type() {
    let schema = "union SearchResult =";
    let errors = parse_expecting_error(schema);

    assert!(errors.has_errors());
}

#[test]
fn test_empty_implements_clause() {
    let schema = "type User implements { id: ID }";
    let errors = parse_expecting_error(schema);

    assert!(errors.has_errors());
}

#[test]
fn test_invalid_type_extension_keyword() {
    let schema = "extend invalid User { }";
    let errors = parse_expecting_error(schema);

    assert!(errors.has_errors());
}

#[test]
fn test_directive_missing_at_symbol() {
    let schema = "directive skip on FIELD";
    let errors = parse_expecting_error(schema);

    assert!(errors.has_errors());
}

#[test]
fn test_multiple_errors_in_document() {
    let schema = r#"
        type Query {
            field1: %
            field2 String
        }
    "#;
    let errors = parse_expecting_error(schema);

    assert!(errors.has_errors());
    // Parser detects error in field1, then recovery skips malformed field2
    assert_eq!(errors.len(), 1);
}
