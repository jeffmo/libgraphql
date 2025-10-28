use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphql_parser;
use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphqlschemaparser;
use libgraphql_core::ast;

#[test]
fn test_input_type_basic() {
    let schema = r#"
        input CreateUserInput {
            name: String!
            email: String!
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), 1);
    assert_eq!(ref_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::InputObject(new_input),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::InputObject(ref_input),
            ),
        ) => {
            assert_eq!(new_input.name, ref_input.name);
            assert_eq!(new_input.fields.len(), ref_input.fields.len());
            assert_eq!(new_input.fields.len(), 2);
        }
        _ => panic!("Expected input type definitions"),
    }
}

#[test]
fn test_input_type_with_default_values() {
    let schema = r#"
        input FilterInput {
            limit: Int = 10
            offset: Int = 0
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::InputObject(new_input),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::InputObject(ref_input),
            ),
        ) => {
            assert!(new_input.fields[0].default_value.is_some());
            assert!(new_input.fields[1].default_value.is_some());
            assert_eq!(
                new_input.fields[0].default_value.is_some(),
                ref_input.fields[0].default_value.is_some()
            );
        }
        _ => panic!("Expected input type definitions"),
    }
}
