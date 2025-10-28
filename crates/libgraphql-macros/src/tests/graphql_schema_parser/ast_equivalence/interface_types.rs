use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphql_parser;
use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphqlschemaparser;
use libgraphql_core::ast;

#[test]
fn test_interface_type_basic() {
    let schema = r#"
        interface Node {
            id: ID!
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), 1);
    assert_eq!(ref_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Interface(new_iface),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Interface(ref_iface),
            ),
        ) => {
            assert_eq!(new_iface.name, ref_iface.name);
            assert_eq!(new_iface.name, "Node");
            assert_eq!(new_iface.fields.len(), 1);
        }
        _ => panic!("Expected interface type definitions"),
    }
}

#[test]
fn test_interface_with_implements() {
    let schema = r#"
        interface Resource implements Node {
            id: ID!
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Interface(new_iface),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Interface(ref_iface),
            ),
        ) => {
            assert_eq!(
                new_iface.implements_interfaces.len(),
                ref_iface.implements_interfaces.len()
            );
            assert_eq!(new_iface.implements_interfaces.len(), 1);
        }
        _ => panic!("Expected interface type definitions"),
    }
}
