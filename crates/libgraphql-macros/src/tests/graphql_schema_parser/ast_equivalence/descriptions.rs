use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphql_parser;
use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphqlschemaparser;
use libgraphql_core::ast;

#[test]
fn test_scalar_with_description() {
    let schema = r#""""A custom scalar for dates"""
scalar DateTime"#;
    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), ref_ast.definitions.len());
    assert_eq!(new_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::Scalar(new_scalar)),
            ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::Scalar(ref_scalar)),
        ) => {
            assert_eq!(new_scalar.name, ref_scalar.name);
            assert_eq!(new_scalar.description, ref_scalar.description);
            assert_eq!(new_scalar.description, Some("A custom scalar for dates".to_string()));
        }
        _ => panic!("Expected scalar type definition"),
    }
}

#[test]
fn test_object_with_description() {
    let schema = r#""""Represents a user in the system"""
type User {
    id: ID
}"#;
    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), ref_ast.definitions.len());

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::Object(new_obj)),
            ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::Object(ref_obj)),
        ) => {
            assert_eq!(new_obj.name, ref_obj.name);
            assert_eq!(new_obj.description, ref_obj.description);
            assert_eq!(new_obj.description, Some("Represents a user in the system".to_string()));
        }
        _ => panic!("Expected object type definition"),
    }
}

#[test]
fn test_field_with_description() {
    let schema = r###"type User {
    """The user's unique identifier"""
    id: ID
}"###;
    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), ref_ast.definitions.len());

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::Object(new_obj)),
            ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::Object(ref_obj)),
        ) => {
            assert_eq!(new_obj.fields.len(), ref_obj.fields.len());
            assert_eq!(new_obj.fields[0].description, ref_obj.fields[0].description);
            assert_eq!(new_obj.fields[0].description, Some("The user's unique identifier".to_string()));
        }
        _ => panic!("Expected object type definition"),
    }
}
