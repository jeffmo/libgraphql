use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphql_parser;
use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphqlschemaparser;
use libgraphql_core::ast;

#[test]
fn test_schema_definition_basic() {
    let schema = r#"
        schema {
            query: Query
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), 1);
    assert_eq!(ref_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::SchemaDefinition(new_schema),
            ast::schema::Definition::SchemaDefinition(ref_schema),
        ) => {
            assert_eq!(new_schema.query, ref_schema.query);
            assert_eq!(new_schema.query, Some("Query".to_string()));
        }
        _ => panic!("Expected schema definitions"),
    }
}

#[test]
fn test_schema_definition_full() {
    let schema = r#"
        schema {
            query: Query
            mutation: Mutation
            subscription: Subscription
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::SchemaDefinition(new_schema),
            ast::schema::Definition::SchemaDefinition(ref_schema),
        ) => {
            assert_eq!(new_schema.query, ref_schema.query);
            assert_eq!(new_schema.mutation, ref_schema.mutation);
            assert_eq!(new_schema.subscription, ref_schema.subscription);
        }
        _ => panic!("Expected schema definitions"),
    }
}
