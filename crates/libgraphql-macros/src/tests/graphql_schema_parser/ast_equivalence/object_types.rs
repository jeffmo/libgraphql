use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphql_parser;
use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphqlschemaparser;
use libgraphql_core::ast;

#[test]
fn test_object_type_basic() {
    let schema = r#"
        type User {
            id: ID
            name: String
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), 1);
    assert_eq!(ref_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Object(new_obj),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Object(ref_obj),
            ),
        ) => {
            assert_eq!(new_obj.name, ref_obj.name);
            assert_eq!(new_obj.fields.len(), ref_obj.fields.len());
            assert_eq!(new_obj.fields.len(), 2);
            assert_eq!(new_obj.fields[0].name, "id");
            assert_eq!(new_obj.fields[1].name, "name");
        }
        _ => panic!("Expected object type definitions"),
    }
}

#[test]
fn test_object_type_with_implements() {
    let schema = r#"
        type User implements Node {
            id: ID
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Object(new_obj),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Object(ref_obj),
            ),
        ) => {
            assert_eq!(
                new_obj.implements_interfaces.len(),
                ref_obj.implements_interfaces.len()
            );
            assert_eq!(new_obj.implements_interfaces.len(), 1);
            assert_eq!(new_obj.implements_interfaces[0], "Node");
        }
        _ => panic!("Expected object type definitions"),
    }
}

#[test]
fn test_object_type_with_multiple_implements() {
    let schema = r#"
        type User implements Node & Timestamped {
            id: ID
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Object(new_obj),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Object(ref_obj),
            ),
        ) => {
            assert_eq!(
                new_obj.implements_interfaces.len(),
                ref_obj.implements_interfaces.len()
            );
            assert_eq!(new_obj.implements_interfaces.len(), 2);
            assert_eq!(new_obj.implements_interfaces[0], "Node");
            assert_eq!(new_obj.implements_interfaces[1], "Timestamped");
        }
        _ => panic!("Expected object type definitions"),
    }
}

#[test]
fn test_object_type_with_field_arguments() {
    let schema = r#"
        type Query {
            user(id: ID!): User
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Object(new_obj),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Object(ref_obj),
            ),
        ) => {
            assert_eq!(new_obj.fields[0].arguments.len(), 1);
            assert_eq!(
                new_obj.fields[0].arguments.len(),
                ref_obj.fields[0].arguments.len()
            );
            assert_eq!(new_obj.fields[0].arguments[0].name, "id");
        }
        _ => panic!("Expected object type definitions"),
    }
}

#[test]
fn test_object_type_with_multiple_field_arguments() {
    let schema = r#"
        type Query {
            users(first: Int, after: String): [User]
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Object(new_obj),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Object(ref_obj),
            ),
        ) => {
            assert_eq!(new_obj.fields[0].arguments.len(), 2);
            assert_eq!(
                new_obj.fields[0].arguments.len(),
                ref_obj.fields[0].arguments.len()
            );
        }
        _ => panic!("Expected object type definitions"),
    }
}
