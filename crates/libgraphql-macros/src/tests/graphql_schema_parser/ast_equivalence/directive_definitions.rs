use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphql_parser;
use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphqlschemaparser;
use libgraphql_core::ast;

#[test]
fn test_directive_definition_basic() {
    let schema = "directive @skip on FIELD";

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), 1);
    assert_eq!(ref_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::DirectiveDefinition(new_dir),
            ast::schema::Definition::DirectiveDefinition(ref_dir),
        ) => {
            assert_eq!(new_dir.name, ref_dir.name);
            assert_eq!(new_dir.name, "skip");
            assert_eq!(new_dir.locations.len(), 1);
        }
        _ => panic!("Expected directive definitions"),
    }
}

#[test]
fn test_directive_definition_with_arguments() {
    let schema = "directive @deprecated(reason: String) on FIELD_DEFINITION";

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::DirectiveDefinition(new_dir),
            ast::schema::Definition::DirectiveDefinition(ref_dir),
        ) => {
            assert_eq!(new_dir.arguments.len(), 1);
            assert_eq!(
                new_dir.arguments.len(),
                ref_dir.arguments.len()
            );
            assert_eq!(new_dir.arguments[0].name, "reason");
        }
        _ => panic!("Expected directive definitions"),
    }
}

#[test]
fn test_directive_definition_multiple_locations() {
    let schema = "directive @auth on FIELD_DEFINITION | OBJECT";

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::DirectiveDefinition(new_dir),
            ast::schema::Definition::DirectiveDefinition(ref_dir),
        ) => {
            assert_eq!(new_dir.locations.len(), 2);
            assert_eq!(
                new_dir.locations.len(),
                ref_dir.locations.len()
            );
        }
        _ => panic!("Expected directive definitions"),
    }
}

#[test]
fn test_directive_definition_repeatable() {
    let schema = "directive @tag repeatable on FIELD_DEFINITION";

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::DirectiveDefinition(new_dir),
            ast::schema::Definition::DirectiveDefinition(ref_dir),
        ) => {
            assert_eq!(new_dir.repeatable, ref_dir.repeatable);
            assert!(new_dir.repeatable);
        }
        _ => panic!("Expected directive definitions"),
    }
}
