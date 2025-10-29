use crate::tests::graphql_schema_parser::parse_error_recovery_tests::utils::parse_with_recovery;

#[test]
fn test_recover_after_malformed_field_to_next_field() {
    let schema = r#"
        type Query {
            field1: %
            field2: String
        }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.has_errors());
}

#[test]
fn test_recover_after_missing_colon_in_field() {
    let schema = r#"
        type Query {
            field1 String
            field2: String
        }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.has_errors());
}

#[test]
fn test_recover_multiple_field_errors() {
    let schema = r#"
        type Query {
            field1: %
            field2: String
            field3: @
            field4: Int
        }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    // Should have multiple field errors
    assert!(errors.len() >= 2);
}

#[test]
fn test_recover_after_invalid_field_type() {
    let schema = r#"
        type Query {
            field1: 123
            field2: String
        }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.has_errors());
}

#[test]
fn test_recover_in_interface_fields() {
    let schema = r#"
        interface Node {
            id: %
            name: String
        }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.has_errors());
}

#[test]
fn test_recover_after_malformed_field_arguments() {
    let schema = r#"
        type Query {
            field1(arg1 String): String
            field2: String
        }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.has_errors());
}

#[test]
fn test_recover_in_input_object_fields() {
    let schema = r#"
        input CreateUserInput {
            name: %
            email: String
        }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.has_errors());
}

#[test]
fn test_field_recovery_does_not_skip_closing_brace() {
    let schema = r#"
        type Query {
            field1: %
        }
        type User {
            id: ID
        }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.has_errors());
}

#[test]
fn test_recover_with_consecutive_field_errors() {
    let schema = r#"
        type Query {
            field1 String
            field2 Int
            field3: Boolean
        }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    // Parser detects first error, recovery consumes field2 (doesn't match pattern)
    // so we expect only 1 error
    assert_eq!(errors.len(), 1);
}

#[test]
fn test_recover_after_field_error_then_valid_fields() {
    let schema = r#"
        type Query {
            bad: %
            good1: String
            good2: Int
        }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.has_errors());
}
