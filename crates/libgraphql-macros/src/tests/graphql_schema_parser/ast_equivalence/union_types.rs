use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphql_parser;
use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphqlschemaparser;
use libgraphql_core::ast;

#[test]
fn test_union_type_basic() {
    let schema = "union SearchResult = User | Post";

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    assert_eq!(new_ast.definitions.len(), 1);
    assert_eq!(ref_ast.definitions.len(), 1);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Union(new_union),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Union(ref_union),
            ),
        ) => {
            assert_eq!(new_union.name, ref_union.name);
            assert_eq!(new_union.types.len(), ref_union.types.len());
            assert_eq!(new_union.types.len(), 2);
            assert_eq!(new_union.types[0], "User");
            assert_eq!(new_union.types[1], "Post");
        }
        _ => panic!("Expected union type definitions"),
    }
}

#[test]
fn test_union_type_with_leading_pipe() {
    let schema = "union SearchResult = | User | Post";

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Union(new_union),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Union(ref_union),
            ),
        ) => {
            assert_eq!(new_union.types.len(), ref_union.types.len());
            assert_eq!(new_union.types.len(), 2);
        }
        _ => panic!("Expected union type definitions"),
    }
}

#[test]
fn test_union_type_many_members() {
    let schema = "union Content = Article | Video | Image | Audio";

    let new_ast = parse_with_graphqlschemaparser(schema);
    let ref_ast = parse_with_graphql_parser(schema);

    match (&new_ast.definitions[0], &ref_ast.definitions[0]) {
        (
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Union(new_union),
            ),
            ast::schema::Definition::TypeDefinition(
                ast::schema::TypeDefinition::Union(ref_union),
            ),
        ) => {
            assert_eq!(new_union.types.len(), 4);
            assert_eq!(new_union.types.len(), ref_union.types.len());
        }
        _ => panic!("Expected union type definitions"),
    }
}
