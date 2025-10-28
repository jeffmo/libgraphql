use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphql_parser;
use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphqlschemaparser;
use libgraphql_core::ast;

#[test]
fn test_enum_type_basic() {
    let schema = r#"
        enum Role {
            ADMIN
            USER
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), 1);
    assert_eq!(ref_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Enum(new_enum),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Enum(ref_enum),
            ),
        ) => {
            assert_eq!(new_enum.name, ref_enum.name);
            assert_eq!(new_enum.values.len(), ref_enum.values.len());
            assert_eq!(new_enum.values.len(), 2);
            assert_eq!(new_enum.values[0].name, "ADMIN");
            assert_eq!(new_enum.values[1].name, "USER");
        }
        _ => panic!("Expected enum type definitions"),
    }
}

#[test]
fn test_enum_type_with_directive() {
    let schema = r#"
        enum Status {
            ACTIVE @deprecated
            INACTIVE
        }
    "#;

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Enum(new_enum),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Enum(ref_enum),
            ),
        ) => {
            assert_eq!(new_enum.values[0].directives.len(), 1);
            assert_eq!(
                new_enum.values[0].directives.len(),
                ref_enum.values[0].directives.len()
            );
        }
        _ => panic!("Expected enum type definitions"),
    }
}
