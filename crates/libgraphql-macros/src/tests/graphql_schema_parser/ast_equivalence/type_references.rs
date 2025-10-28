use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphql_parser;
use crate::tests::graphql_schema_parser::ast_equivalence::utils::parse_with_graphqlschemaparser;
use libgraphql_core::ast;

#[test]
fn test_list_types() {
    let schema = r#"
        type Query {
            users: [User]
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
            assert!(matches!(
                new_obj.fields[0].field_type,
                ast::schema::Type::ListType(_)
            ));
            assert!(matches!(
                ref_obj.fields[0].field_type,
                ast::schema::Type::ListType(_)
            ));
        }
        _ => panic!("Expected object type definitions"),
    }
}

#[test]
fn test_non_null_types() {
    let schema = r#"
        type Query {
            user: User!
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
            assert!(matches!(
                new_obj.fields[0].field_type,
                ast::schema::Type::NonNullType(_)
            ));
            assert!(matches!(
                ref_obj.fields[0].field_type,
                ast::schema::Type::NonNullType(_)
            ));
        }
        _ => panic!("Expected object type definitions"),
    }
}

#[test]
fn test_nested_types() {
    let schema = r#"
        type Query {
            users: [User!]!
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
            // Outer non-null
            if let ast::schema::Type::NonNullType(new_inner) =
                &new_obj.fields[0].field_type
            {
                // Inner list
                assert!(matches!(
                    **new_inner,
                    ast::schema::Type::ListType(_)
                ));
            } else {
                panic!("Expected non-null type");
            }

            if let ast::schema::Type::NonNullType(ref_inner) =
                &ref_obj.fields[0].field_type
            {
                assert!(matches!(
                    **ref_inner,
                    ast::schema::Type::ListType(_)
                ));
            }
        }
        _ => panic!("Expected object type definitions"),
    }
}
