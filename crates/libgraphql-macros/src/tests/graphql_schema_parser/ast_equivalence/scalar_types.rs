use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphql_parser;
use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphqlschemaparser;
use libgraphql_core::ast;

#[test]
fn test_scalar_type_basic() {
    let schema = "scalar DateTime";

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), 1);
    assert_eq!(ref_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Scalar(new_scalar),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Scalar(ref_scalar),
            ),
        ) => {
            assert_eq!(new_scalar.name, ref_scalar.name);
            assert_eq!(new_scalar.name, "DateTime");
        }
        _ => panic!("Expected scalar type definitions"),
    }
}

#[test]
fn test_scalar_type_with_directive() {
    let schema = "scalar UUID @specifiedBy";

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), 1);
    assert_eq!(ref_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Scalar(new_scalar),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Scalar(ref_scalar),
            ),
        ) => {
            assert_eq!(new_scalar.directives.len(), 1);
            assert_eq!(
                new_scalar.directives.len(),
                ref_scalar.directives.len()
            );
        }
        _ => panic!("Expected scalar type definitions"),
    }
}
