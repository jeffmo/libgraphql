use crate::tests::graphql_schema_parser::parse_error_recovery_tests::utils::parse_with_recovery;

#[test]
fn test_recover_after_invalid_keyword() {
    let schema = r#"
        invalid User { id: ID }
        type Query { user: User }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.has_errors());
    assert!(!result.errors.is_empty());
}

#[test]
fn test_recover_to_next_type_definition() {
    let schema = r#"
        type User { id ID }
        type Query { user: User }
    "#;
    let result = parse_with_recovery(schema);

    // Parser should error on User but recover to parse Query
    assert!(result.has_errors());
    assert!(!result.errors.is_empty());
}

#[test]
fn test_recover_after_malformed_union() {
    let schema = r#"
        union SearchResult
        type Query { search: SearchResult }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.has_errors());
    assert!(!result.errors.is_empty());
}

#[test]
fn test_recover_after_malformed_enum() {
    let schema = r#"
        enum Status
        type Query { status: Status }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.has_errors());
    assert!(!result.errors.is_empty());
}

#[test]
fn test_recover_after_malformed_interface() {
    let schema = r#"
        interface Node
        type User implements Node { id: ID }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.has_errors());
    assert!(!result.errors.is_empty());
}

#[test]
fn test_recover_after_malformed_input() {
    let schema = r#"
        input CreateUserInput
        type Mutation { createUser(input: CreateUserInput): User }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.has_errors());
    assert!(!result.errors.is_empty());
}

#[test]
fn test_recover_after_malformed_directive() {
    let schema = r#"
        directive skip
        type Query { field: String }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.has_errors());
    assert!(!result.errors.is_empty());
}

#[test]
fn test_recover_after_malformed_schema_definition() {
    let schema = r#"
        schema { invalid Query }
        type Query { field: String }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.has_errors());
    assert!(!result.errors.is_empty());
}

#[test]
fn test_recover_multiple_errors_across_definitions() {
    let schema = r#"
        type User { id ID }
        type Post { title Title }
        type Query { user: User }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.has_errors());
    // Should have at least 2 errors (one for User, one for Post)
    assert!(result.errors.len() >= 2);
}

#[test]
fn test_recover_with_valid_definition_between_errors() {
    let schema = r#"
        type User { id ID }
        type Post { title: String }
        type Comment { text Text }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.has_errors());
    // Should have errors for User and Comment
    assert!(result.errors.len() >= 2);
}

#[test]
fn test_recover_after_extend_with_invalid_keyword() {
    let schema = r#"
        extend invalid User { email: String }
        type Query { user: User }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.has_errors());
    assert!(!result.errors.is_empty());
}

#[test]
fn test_recover_after_unexpected_token_at_definition_level() {
    let schema = r#"
        { unexpected }
        type Query { field: String }
    "#;
    let result = parse_with_recovery(schema);

    assert!(result.has_errors());
    assert!(!result.errors.is_empty());
}
