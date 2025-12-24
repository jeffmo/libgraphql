use crate::operation::ExecutableDocumentBuildError;
use crate::operation::ExecutableDocumentBuilder;
use crate::operation::FragmentRegistry;
use crate::operation::FragmentRegistryBuilder;
use crate::schema::SchemaBuilder;

fn setup_schema() -> crate::schema::Schema {
    SchemaBuilder::from_str(
        None,
        r#"
        type Query {
            user(id: ID!): User
            users: [User!]!
            post(id: ID!): Post
            posts: [Post!]!
        }

        type Mutation {
            createUser(name: String!): User
            updateUser(id: ID!, name: String!): User
            deleteUser(id: ID!): Boolean
        }

        type Subscription {
            userCreated: User
            userUpdated: User
        }

        type User {
            id: ID!
            name: String!
            email: String
            posts: [Post!]!
            friends: [User!]!
        }

        type Post {
            id: ID!
            title: String!
            body: String!
            author: User!
            comments: [Comment!]!
        }

        type Comment {
            id: ID!
            text: String!
            author: User!
        }
        "#,
    )
    .unwrap()
    .build()
    .unwrap()
}

// =============================================================================
// Basic Functionality Tests
// =============================================================================

#[test]
fn empty_document_with_empty_registry() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    // Empty documents are invalid - GraphQL spec requires at least one Definition
    let result = ExecutableDocumentBuilder::from_str(&schema, &registry, "", None);

    match result {
        Err(errors) => {
            assert!(
                errors.iter().any(|e| matches!(e, ExecutableDocumentBuildError::ParseError(_))),
                "Expected ParseError for empty document"
            );
        }
        Ok(_) => panic!("Expected ParseError for empty document, got Ok"),
    }
}

#[test]
fn single_query_operation_no_fragments() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: "1") {
                id
                name
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn single_mutation_operation_no_fragments() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        mutation CreateUser {
            createUser(name: "Alice") {
                id
                name
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn single_subscription_operation_no_fragments() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        subscription OnUserCreated {
            userCreated {
                id
                name
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn multiple_operations_in_one_document() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: "1") {
                id
                name
            }
        }

        query GetUsers {
            users {
                id
                name
            }
        }

        mutation CreateUser {
            createUser(name: "Alice") {
                id
                name
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 3);
}

#[test]
fn anonymous_query_operation() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        {
            user(id: "1") {
                id
                name
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn operation_with_variables() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser($userId: ID!) {
            user(id: $userId) {
                id
                name
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn operation_with_multiple_variables() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        mutation UpdateUser($userId: ID!, $userName: String!) {
            updateUser(id: $userId, name: $userName) {
                id
                name
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

// =============================================================================
// Fragment Registry Validation Tests
// =============================================================================

#[test]
fn document_with_fragment_matching_registry() {
    let schema = setup_schema();

    // Build registry with fragment
    let mut registry_builder = FragmentRegistryBuilder::new();
    registry_builder
        .add_from_document_str(
            &schema,
            "fragment UserFields on User { id name email }",
            None,
        )
        .unwrap();
    let registry = registry_builder.build().unwrap();

    // Build document with same fragment
    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        fragment UserFields on User { id name email }

        query GetUser {
            user(id: "1") {
                ...UserFields
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn document_with_multiple_fragments_matching_registry() {
    let schema = setup_schema();

    // Build registry with fragments
    let mut registry_builder = FragmentRegistryBuilder::new();
    registry_builder
        .add_from_document_str(
            &schema,
            r#"
            fragment UserFields on User { id name }
            fragment PostFields on Post { id title }
            "#,
            None,
        )
        .unwrap();
    let registry = registry_builder.build().unwrap();

    // Build document with same fragments
    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        fragment UserFields on User { id name }
        fragment PostFields on Post { id title }

        query GetUser {
            user(id: "1") {
                ...UserFields
                posts {
                    ...PostFields
                }
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn document_with_fragment_not_in_registry() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        fragment UserFields on User { id name }

        query GetUser {
            user(id: "1") {
                ...UserFields
            }
        }
        "#,
        None,
    );

    match result {
        Err(errors) => {
            assert!(
                errors.iter().any(|e| matches!(e, ExecutableDocumentBuildError::FragmentNotInRegistry { .. })),
                "Expected FragmentNotInRegistry error in errors"
            );
        }
        Ok(_) => panic!("Expected FragmentNotInRegistry error, got Ok"),
    }
}

#[test]
fn document_with_fragment_mismatching_registry() {
    let schema = setup_schema();

    // Build registry with fragment
    let mut registry_builder = FragmentRegistryBuilder::new();
    registry_builder
        .add_from_document_str(
            &schema,
            "fragment UserFields on User { id name email }",
            None,
        )
        .unwrap();
    let registry = registry_builder.build().unwrap();

    // Build document with different fragment (missing email field)
    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        fragment UserFields on User { id name }

        query GetUser {
            user(id: "1") {
                ...UserFields
            }
        }
        "#,
        None,
    );

    match result {
        Err(errors) => {
            assert!(
                errors.iter().any(|e| matches!(e, ExecutableDocumentBuildError::FragmentDefinitionMismatch { .. })),
                "Expected FragmentDefinitionMismatch error in errors"
            );
        }
        Ok(_) => panic!("Expected FragmentDefinitionMismatch error, got Ok"),
    }
}

#[test]
fn document_with_one_matching_one_missing_fragment() {
    let schema = setup_schema();

    // Build registry with only UserFields
    let mut registry_builder = FragmentRegistryBuilder::new();
    registry_builder
        .add_from_document_str(&schema, "fragment UserFields on User { id name }", None)
        .unwrap();
    let registry = registry_builder.build().unwrap();

    // Document has UserFields (matching) and PostFields (missing)
    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        fragment UserFields on User { id name }
        fragment PostFields on Post { id title }

        query GetUser {
            user(id: "1") {
                ...UserFields
            }
        }
        "#,
        None,
    );

    match result {
        Err(errors) => {
            assert!(
                errors.iter().any(|e| matches!(e, ExecutableDocumentBuildError::FragmentNotInRegistry { .. })),
                "Expected FragmentNotInRegistry error in errors"
            );
        }
        Ok(_) => panic!("Expected FragmentNotInRegistry error, got Ok"),
    }
}

#[test]
fn operation_using_fragment_spread_from_registry() {
    let schema = setup_schema();

    // Build registry with fragment
    let mut registry_builder = FragmentRegistryBuilder::new();
    registry_builder
        .add_from_document_str(&schema, "fragment UserFields on User { id name }", None)
        .unwrap();
    let registry = registry_builder.build().unwrap();

    // Document uses fragment via spread, but doesn't redefine it
    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: "1") {
                ...UserFields
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn document_only_fragments_no_operations() {
    let schema = setup_schema();

    // Build registry with fragment
    let mut registry_builder = FragmentRegistryBuilder::new();
    registry_builder
        .add_from_document_str(&schema, "fragment UserFields on User { id name }", None)
        .unwrap();
    let registry = registry_builder.build().unwrap();

    // Document only contains matching fragment definition
    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        "fragment UserFields on User { id name }",
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 0);
}

// =============================================================================
// Operation with Inline Fragments
// =============================================================================

#[test]
fn operation_with_inline_fragment() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: "1") {
                id
                ... on User {
                    name
                    email
                }
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn operation_with_nested_inline_fragments() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: "1") {
                id
                ... on User {
                    name
                    posts {
                        ... on Post {
                            title
                        }
                    }
                }
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

// =============================================================================
// Nested Selection Sets
// =============================================================================

#[test]
fn operation_with_nested_selection_sets() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: "1") {
                id
                name
                posts {
                    id
                    title
                    author {
                        id
                        name
                    }
                }
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn operation_with_deeply_nested_selection_sets() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: "1") {
                id
                posts {
                    id
                    comments {
                        id
                        author {
                            id
                            posts {
                                id
                                title
                            }
                        }
                    }
                }
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

// =============================================================================
// Fragment Spreads with Registry
// =============================================================================

#[test]
fn operation_with_nested_fragment_spreads() {
    let schema = setup_schema();

    // Build registry with nested fragments
    let mut registry_builder = FragmentRegistryBuilder::new();
    registry_builder
        .add_from_document_str(
            &schema,
            r#"
            fragment UserFields on User { id name }
            fragment PostFields on Post { id title author { ...UserFields } }
            "#,
            None,
        )
        .unwrap();
    let registry = registry_builder.build().unwrap();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetPosts {
            posts {
                ...PostFields
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn operation_with_multiple_fragment_spreads() {
    let schema = setup_schema();

    // Build registry with fragments
    let mut registry_builder = FragmentRegistryBuilder::new();
    registry_builder
        .add_from_document_str(
            &schema,
            r#"
            fragment UserBasicFields on User { id name }
            fragment UserEmailField on User { email }
            "#,
            None,
        )
        .unwrap();
    let registry = registry_builder.build().unwrap();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: "1") {
                ...UserBasicFields
                ...UserEmailField
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

// =============================================================================
// Error Cases - Parse Errors
// =============================================================================

#[test]
fn invalid_syntax_document() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    // Document with invalid syntax (unclosed brace)
    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        "query { user { id ",
        None,
    );

    match result {
        Err(errors) => {
            assert!(
                errors.iter().any(|e| matches!(e, ExecutableDocumentBuildError::ParseError(_))),
                "Expected ParseError in errors"
            );
        }
        Ok(_) => panic!("Expected ParseError, got Ok"),
    }
}

#[test]
fn incomplete_query_syntax() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: "1") {
                id
                name
        "#, // Missing closing braces
        None,
    );

    assert!(result.is_err());
}

// =============================================================================
// Error Cases - Operation Build Errors
// =============================================================================

#[test]
#[ignore] // TODO: OperationBuilder needs to implement field validation
fn operation_with_nonexistent_field() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: "1") {
                id
                nonexistentField
            }
        }
        "#,
        None,
    );

    match result {
        Err(errors) => {
            assert!(
                errors.iter().any(|e| matches!(e, ExecutableDocumentBuildError::OperationBuildErrors(_))),
                "Expected OperationBuildErrors in errors"
            );
        }
        Ok(_) => panic!("Expected OperationBuildErrors, got Ok"),
    }
}

#[test]
#[ignore] // TODO: OperationBuilder needs to implement root field validation
fn operation_with_nonexistent_type() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetNonexistent {
            nonexistentRootField {
                id
            }
        }
        "#,
        None,
    );

    match result {
        Err(errors) => {
            assert!(
                errors.iter().any(|e| matches!(e, ExecutableDocumentBuildError::OperationBuildErrors(_))),
                "Expected OperationBuildErrors for nonexistent root field"
            );
        }
        Ok(_) => panic!("Expected OperationBuildErrors for nonexistent root field, got Ok"),
    }
}

#[test]
fn operation_with_wrong_argument_type() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: 123) {
                id
            }
        }
        "#,
        None,
    );

    // This should parse correctly - argument type validation happens at execution time
    assert!(result.is_ok());
}

// =============================================================================
// File-Based Tests
// =============================================================================

#[test]
fn from_ast_with_valid_document() {
    use crate::ast;

    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let ast_doc = ast::operation::parse(
        r#"
        query GetUser {
            user(id: "1") {
                id
                name
            }
        }
        "#,
    )
    .unwrap();

    let result = ExecutableDocumentBuilder::from_ast(&schema, &registry, &ast_doc, None);

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn from_ast_with_fragments_matching_registry() {
    use crate::ast;

    let schema = setup_schema();

    // Build registry
    let mut registry_builder = FragmentRegistryBuilder::new();
    registry_builder
        .add_from_document_str(&schema, "fragment UserFields on User { id name }", None)
        .unwrap();
    let registry = registry_builder.build().unwrap();

    let ast_doc = ast::operation::parse(
        r#"
        fragment UserFields on User { id name }

        query GetUser {
            user(id: "1") {
                ...UserFields
            }
        }
        "#,
    )
    .unwrap();

    let result = ExecutableDocumentBuilder::from_ast(&schema, &registry, &ast_doc, None);

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

// =============================================================================
// Builder API Tests
// =============================================================================

#[test]
fn builder_new_and_build() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let builder = ExecutableDocumentBuilder::new(&schema, &registry);
    let result = builder.build();

    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(doc.operations().len(), 0);
}

#[test]
fn builder_from_str_and_build() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: "1") {
                id
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn document_with_only_whitespace() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    // Whitespace-only documents are invalid - GraphQL spec requires at least one Definition
    let result = ExecutableDocumentBuilder::from_str(&schema, &registry, "   \n\t  ", None);

    match result {
        Err(errors) => {
            assert!(
                errors.iter().any(|e| matches!(e, ExecutableDocumentBuildError::ParseError(_))),
                "Expected ParseError for whitespace-only document"
            );
        }
        Ok(_) => panic!("Expected ParseError for whitespace-only document, got Ok"),
    }
}

#[test]
fn document_with_comments_only() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    // Comment-only documents are invalid - GraphQL spec requires at least one Definition
    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        # This is a comment
        # Another comment
        "#,
        None,
    );

    match result {
        Err(errors) => {
            assert!(
                errors.iter().any(|e| matches!(e, ExecutableDocumentBuildError::ParseError(_))),
                "Expected ParseError for comment-only document"
            );
        }
        Ok(_) => panic!("Expected ParseError for comment-only document, got Ok"),
    }
}

#[test]
fn operation_with_comments() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        # Get a user by ID
        query GetUser {
            # Query the user field
            user(id: "1") {
                id  # User ID
                name  # User name
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn operation_with_alias() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUsers {
            firstUser: user(id: "1") {
                id
                name
            }
            secondUser: user(id: "2") {
                id
                name
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

#[test]
fn multiple_operations_same_type() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: "1") {
                id
            }
        }

        query GetAllUsers {
            users {
                id
            }
        }

        query GetPost {
            post(id: "1") {
                id
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 3);
}

// =============================================================================
// Complex Integration Tests
// =============================================================================

#[test]
fn complex_document_with_all_features() {
    let schema = setup_schema();

    // Build registry with fragments
    let mut registry_builder = FragmentRegistryBuilder::new();
    registry_builder
        .add_from_document_str(
            &schema,
            r#"
            fragment UserBasicFields on User {
                id
                name
                email
            }

            fragment PostWithAuthor on Post {
                id
                title
                body
                author {
                    ...UserBasicFields
                }
            }
            "#,
            None,
        )
        .unwrap();
    let registry = registry_builder.build().unwrap();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        # Define the same fragments as in registry
        fragment UserBasicFields on User {
            id
            name
            email
        }

        fragment PostWithAuthor on Post {
            id
            title
            body
            author {
                ...UserBasicFields
            }
        }

        # Query with variables and fragments
        query GetUserWithPosts($userId: ID!) {
            user(id: $userId) {
                ...UserBasicFields
                posts {
                    ...PostWithAuthor
                }
            }
        }

        # Query with inline fragments
        query GetUserFriends($userId: ID!) {
            user(id: $userId) {
                id
                friends {
                    ... on User {
                        id
                        name
                    }
                }
            }
        }

        # Mutation
        mutation CreateUser($name: String!) {
            createUser(name: $name) {
                ...UserBasicFields
            }
        }

        # Subscription
        subscription OnUserCreated {
            userCreated {
                ...UserBasicFields
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 4);
}

#[test]
fn document_with_fragment_spread_in_inline_fragment() {
    let schema = setup_schema();

    // Build registry
    let mut registry_builder = FragmentRegistryBuilder::new();
    registry_builder
        .add_from_document_str(&schema, "fragment UserFields on User { id name }", None)
        .unwrap();
    let registry = registry_builder.build().unwrap();

    let result = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        r#"
        query GetUser {
            user(id: "1") {
                ... on User {
                    ...UserFields
                    email
                }
            }
        }
        "#,
        None,
    );

    assert!(result.is_ok());
    let builder = result.unwrap();
    let doc = builder.build().unwrap();
    assert_eq!(doc.operations().len(), 1);
}

// =============================================================================
// Tests for ExecutableDocument Access Methods
// =============================================================================

#[test]
fn executable_document_provides_schema_access() {
    let schema = setup_schema();
    let registry = FragmentRegistry::empty();

    let doc = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        "query { users { id } }",
        None,
    )
    .unwrap()
    .build()
    .unwrap();

    // Verify we can access the schema
    assert_eq!(doc.schema().query_type().name(), "Query");
}

#[test]
fn executable_document_provides_fragment_registry_access() {
    let schema = setup_schema();

    // Build registry with fragment
    let mut registry_builder = FragmentRegistryBuilder::new();
    registry_builder
        .add_from_document_str(&schema, "fragment UserFields on User { id name }", None)
        .unwrap();
    let registry = registry_builder.build().unwrap();

    let doc = ExecutableDocumentBuilder::from_str(
        &schema,
        &registry,
        "query { users { ...UserFields } }",
        None,
    )
    .unwrap()
    .build()
    .unwrap();

    // Verify we can access the fragment registry
    assert!(doc.fragment_registry().fragments().contains_key("UserFields"));
}
