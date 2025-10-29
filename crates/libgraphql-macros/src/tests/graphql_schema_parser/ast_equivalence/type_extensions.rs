use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphql_parser;
use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphqlschemaparser;
use libgraphql_core::ast;

#[test]
fn test_extend_scalar_type() {
    let schema = "extend scalar DateTime @deprecated";
    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), ref_ast.definitions.len());
    assert_eq!(new_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::Scalar(new_ext)),
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::Scalar(ref_ext)),
        ) => {
            assert_eq!(new_ext.name, ref_ext.name);
            assert_eq!(new_ext.name, "DateTime");
            assert_eq!(new_ext.directives.len(), ref_ext.directives.len());
        }
        _ => panic!("Expected scalar type extension"),
    }
}

#[test]
fn test_extend_object_type() {
    let schema = r#"
        extend type User {
            email: String
        }
    "#;
    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), ref_ast.definitions.len());
    assert_eq!(new_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::Object(new_ext)),
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::Object(ref_ext)),
        ) => {
            assert_eq!(new_ext.name, ref_ext.name);
            assert_eq!(new_ext.name, "User");
            assert_eq!(new_ext.fields.len(), ref_ext.fields.len());
            assert_eq!(new_ext.fields.len(), 1);
            assert_eq!(new_ext.fields[0].name, "email");
        }
        _ => panic!("Expected object type extension"),
    }
}

#[test]
fn test_extend_interface_type() {
    let schema = r#"
        extend interface Node {
            createdAt: String
        }
    "#;
    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), ref_ast.definitions.len());
    assert_eq!(new_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::Interface(new_ext)),
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::Interface(ref_ext)),
        ) => {
            assert_eq!(new_ext.name, ref_ext.name);
            assert_eq!(new_ext.name, "Node");
            assert_eq!(new_ext.fields.len(), ref_ext.fields.len());
            assert_eq!(new_ext.fields.len(), 1);
        }
        _ => panic!("Expected interface type extension"),
    }
}

#[test]
fn test_extend_union_type() {
    let schema = "extend union SearchResult = Photo | Person";
    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), ref_ast.definitions.len());
    assert_eq!(new_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::Union(new_ext)),
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::Union(ref_ext)),
        ) => {
            assert_eq!(new_ext.name, ref_ext.name);
            assert_eq!(new_ext.name, "SearchResult");
            assert_eq!(new_ext.types.len(), ref_ext.types.len());
            assert_eq!(new_ext.types.len(), 2);
        }
        _ => panic!("Expected union type extension"),
    }
}

#[test]
fn test_extend_enum_type() {
    let schema = r#"
        extend enum Status {
            ARCHIVED
        }
    "#;
    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), ref_ast.definitions.len());
    assert_eq!(new_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::Enum(new_ext)),
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::Enum(ref_ext)),
        ) => {
            assert_eq!(new_ext.name, ref_ext.name);
            assert_eq!(new_ext.name, "Status");
            assert_eq!(new_ext.values.len(), ref_ext.values.len());
            assert_eq!(new_ext.values.len(), 1);
            assert_eq!(new_ext.values[0].name, "ARCHIVED");
        }
        _ => panic!("Expected enum type extension"),
    }
}

#[test]
fn test_extend_input_type() {
    let schema = r#"
        extend input CreateUserInput {
            metadata: String
        }
    "#;
    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), ref_ast.definitions.len());
    assert_eq!(new_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::InputObject(new_ext)),
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::InputObject(ref_ext)),
        ) => {
            assert_eq!(new_ext.name, ref_ext.name);
            assert_eq!(new_ext.name, "CreateUserInput");
            assert_eq!(new_ext.fields.len(), ref_ext.fields.len());
            assert_eq!(new_ext.fields.len(), 1);
        }
        _ => panic!("Expected input object type extension"),
    }
}

#[test]
fn test_extend_object_with_implements() {
    let schema = r#"
        extend type User implements Node {
            id: ID
        }
    "#;
    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), ref_ast.definitions.len());

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::Object(new_ext)),
            ast::schema::Definition::TypeExtension(ast::schema::TypeExtension::Object(ref_ext)),
        ) => {
            assert_eq!(new_ext.name, ref_ext.name);
            assert_eq!(new_ext.implements_interfaces.len(), ref_ext.implements_interfaces.len());
            assert_eq!(new_ext.implements_interfaces, vec!["Node"]);
        }
        _ => panic!("Expected object type extension"),
    }
}

