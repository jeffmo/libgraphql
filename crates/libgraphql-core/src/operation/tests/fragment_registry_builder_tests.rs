use crate::operation::FragmentRegistryBuilder;
use crate::operation::FragmentRegistryBuildError;
use crate::schema::SchemaBuilder;

fn setup_schema() -> crate::schema::Schema {
    SchemaBuilder::from_str(
        None,
        r#"
        type Query {
            user: User
            post: Post
        }

        type User {
            id: ID!
            name: String!
            posts: [Post!]!
        }

        type Post {
            id: ID!
            title: String!
            author: User!
        }
        "#,
    )
    .unwrap()
    .build()
    .unwrap()
}

#[test]
fn empty_registry_creation() {
    let builder = FragmentRegistryBuilder::new();
    let registry = builder.build().unwrap();
    assert_eq!(registry.fragments().len(), 0);
}

#[test]
fn single_fragment_addition() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    builder
        .add_from_document_str(
            &schema,
            "fragment UserFields on User { id name }",
            None,
        )
        .unwrap();

    let registry = builder.build().unwrap();
    assert_eq!(registry.fragments().len(), 1);
    assert!(registry.fragments().contains_key("UserFields"));
}

#[test]
fn multiple_fragments_addition() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    builder
        .add_from_document_str(
            &schema,
            "fragment UserFields on User { id name }",
            None,
        )
        .unwrap();

    builder
        .add_from_document_str(
            &schema,
            "fragment PostFields on Post { id title }",
            None,
        )
        .unwrap();

    let registry = builder.build().unwrap();
    assert_eq!(registry.fragments().len(), 2);
    assert!(registry.fragments().contains_key("UserFields"));
    assert!(registry.fragments().contains_key("PostFields"));
}

#[test]
fn duplicate_fragment_detection() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    builder
        .add_from_document_str(
            &schema,
            "fragment UserFields on User { id name }",
            None,
        )
        .unwrap();

    let result = builder.add_from_document_str(
        &schema,
        "fragment UserFields on User { id }",
        None,
    );

    assert!(result.is_err());
    // The error should be a FragmentBuildError containing a duplicate error
}

#[test]
fn simple_self_referencing_cycle() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    builder
        .add_from_document_str(
            &schema,
            "fragment UserFields on User { id ...UserFields }",
            None,
        )
        .unwrap();

    let result = builder.build();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0],
        FragmentRegistryBuildError::FragmentCycleDetected { .. }
    ));
}

#[test]
fn two_fragment_cycle() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    builder
        .add_from_document_str(
            &schema,
            r#"
            fragment FragA on User { id ...FragB }
            fragment FragB on User { name ...FragA }
            "#,
            None,
        )
        .unwrap();

    let result = builder.build();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0],
        FragmentRegistryBuildError::FragmentCycleDetected { .. }
    ));
}

#[test]
fn three_fragment_cycle() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    builder
        .add_from_document_str(
            &schema,
            r#"
            fragment FragA on User { id ...FragB }
            fragment FragB on User { name ...FragC }
            fragment FragC on User { id ...FragA }
            "#,
            None,
        )
        .unwrap();

    let result = builder.build();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0],
        FragmentRegistryBuildError::FragmentCycleDetected { .. }
    ));
}

#[test]
fn four_fragment_cycle() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    builder
        .add_from_document_str(
            &schema,
            r#"
            fragment FragA on User { id ...FragB }
            fragment FragB on User { name ...FragC }
            fragment FragC on User { id ...FragD }
            fragment FragD on User { name ...FragA }
            "#,
            None,
        )
        .unwrap();

    let result = builder.build();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0],
        FragmentRegistryBuildError::FragmentCycleDetected { .. }
    ));
}

#[test]
fn phase_shifted_cycles_are_deduplicated() {
    let schema = setup_schema();

    // Build the cycle A → B → C → A three times with different starting points
    let mut builder = FragmentRegistryBuilder::new();

    builder
        .add_from_document_str(
            &schema,
            r#"
            fragment FragA on User { id ...FragB }
            fragment FragB on User { name ...FragC }
            fragment FragC on User { id ...FragA }
            "#,
            None,
        )
        .unwrap();

    let result = builder.build();
    assert!(result.is_err());
    let errors = result.unwrap_err();

    // Should only report ONE cycle error (phase-shifted cycles are deduplicated)
    let cycle_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, FragmentRegistryBuildError::FragmentCycleDetected { .. }))
        .collect();

    assert_eq!(
        cycle_errors.len(),
        1,
        "Expected 1 cycle error but got {}",
        cycle_errors.len()
    );
}

#[test]
fn different_cycles_are_both_reported() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    // Create two different cycles: A → B → A and A → B → C → A
    builder
        .add_from_document_str(
            &schema,
            r#"
            fragment FragA on User { id ...FragB }
            fragment FragB on User { name ...FragA ...FragC }
            fragment FragC on User { id ...FragA }
            "#,
            None,
        )
        .unwrap();

    let result = builder.build();
    assert!(result.is_err());
    let errors = result.unwrap_err();

    // Should report TWO distinct cycles
    let cycle_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, FragmentRegistryBuildError::FragmentCycleDetected { .. }))
        .collect();

    assert_eq!(
        cycle_errors.len(),
        2,
        "Expected 2 distinct cycle errors but got {}",
        cycle_errors.len()
    );
}

#[test]
fn multiple_distinct_cycles_in_same_registry() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    // Create two unrelated cycles: A → B → A and C → D → E → C
    builder
        .add_from_document_str(
            &schema,
            r#"
            fragment FragA on User { id ...FragB }
            fragment FragB on User { name ...FragA }
            fragment FragC on Post { id ...FragD }
            fragment FragD on Post { title ...FragE }
            fragment FragE on Post { id ...FragC }
            "#,
            None,
        )
        .unwrap();

    let result = builder.build();
    assert!(result.is_err());
    let errors = result.unwrap_err();

    // Should report TWO distinct unrelated cycles
    let cycle_errors: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, FragmentRegistryBuildError::FragmentCycleDetected { .. }))
        .collect();

    assert_eq!(
        cycle_errors.len(),
        2,
        "Expected 2 distinct unrelated cycle errors but got {}",
        cycle_errors.len()
    );
}

#[test]
fn diamond_pattern_no_cycle() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    // Diamond: A → C, B → C (shared fragment, not a cycle)
    builder
        .add_from_document_str(
            &schema,
            r#"
            fragment FragA on User { id ...FragC }
            fragment FragB on User { name ...FragC }
            fragment FragC on User { id }
            "#,
            None,
        )
        .unwrap();

    let registry = builder.build().unwrap();
    assert_eq!(registry.fragments().len(), 3);
}

#[test]
fn linear_chain_no_cycle() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    // Linear: A → B → C (terminates, no cycle)
    builder
        .add_from_document_str(
            &schema,
            r#"
            fragment FragA on User { id ...FragB }
            fragment FragB on User { name ...FragC }
            fragment FragC on User { id }
            "#,
            None,
        )
        .unwrap();

    let registry = builder.build().unwrap();
    assert_eq!(registry.fragments().len(), 3);
}

#[test]
fn undefined_fragment_reference_detection() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    builder
        .add_from_document_str(
            &schema,
            "fragment FragA on User { id ...UndefinedFragment }",
            None,
        )
        .unwrap();

    let result = builder.build();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0],
        FragmentRegistryBuildError::UndefinedFragmentReference { .. }
    ));
}

#[test]
fn multiple_undefined_references_collected() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    builder
        .add_from_document_str(
            &schema,
            r#"
            fragment FragA on User { id ...Undefined1 }
            fragment FragB on User { name ...Undefined2 }
            "#,
            None,
        )
        .unwrap();

    let result = builder.build();
    assert!(result.is_err());
    let errors = result.unwrap_err();

    let undefined_errors: Vec<_> = errors
        .iter()
        .filter(|e| {
            matches!(
                e,
                FragmentRegistryBuildError::UndefinedFragmentReference { .. }
            )
        })
        .collect();

    assert_eq!(
        undefined_errors.len(),
        2,
        "Expected 2 undefined reference errors but got {}",
        undefined_errors.len()
    );
}

#[test]
fn fragments_from_document_parsing() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    builder
        .add_from_document_str(
            &schema,
            r#"
            fragment UserFields on User { id name }
            fragment PostFields on Post { id title }
            "#,
            None,
        )
        .unwrap();

    let registry = builder.build().unwrap();
    assert_eq!(registry.fragments().len(), 2);
}

#[test]
fn mixed_fragments_and_operations_in_document() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    // add_from_document_str should ignore operations and only process fragments
    builder
        .add_from_document_str(
            &schema,
            r#"
            fragment UserFields on User { id name }

            query GetUser {
                user {
                    ...UserFields
                }
            }
            "#,
            None,
        )
        .unwrap();

    let registry = builder.build().unwrap();
    assert_eq!(registry.fragments().len(), 1);
    assert!(registry.fragments().contains_key("UserFields"));
}

#[test]
fn multiple_documents_contributing_fragments() {
    let schema = setup_schema();
    let mut builder = FragmentRegistryBuilder::new();

    builder
        .add_from_document_str(
            &schema,
            "fragment UserFields on User { id name }",
            None,
        )
        .unwrap();

    builder
        .add_from_document_str(
            &schema,
            "fragment PostFields on Post { id title }",
            None,
        )
        .unwrap();

    builder
        .add_from_document_str(
            &schema,
            "fragment AuthorFields on User { id name }",
            None,
        )
        .unwrap();

    let registry = builder.build().unwrap();
    assert_eq!(registry.fragments().len(), 3);
}

#[test]
fn empty_registry_from_empty_static() {
    use crate::operation::FragmentRegistry;
    let registry = FragmentRegistry::empty();
    assert_eq!(registry.fragments().len(), 0);
}
