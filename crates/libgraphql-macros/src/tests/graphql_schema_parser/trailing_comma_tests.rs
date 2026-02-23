/// Tests for trailing comma support in GraphQL schemas
///
/// According to the GraphQL September 2025 spec, commas are
/// optional and "insignificant" throughout GraphQL syntax.
/// Trailing commas are explicitly allowed in:
/// - List values
/// - Input object values
/// - Argument lists
/// - And by extension, field lists, enum value lists, etc.
use crate::rust_macro_graphql_token_source::RustMacroGraphQLTokenSource;
use libgraphql_parser::GraphQLParser;
use libgraphql_parser::ParseResult;
use quote::quote;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

fn parse_schema(
    input: proc_macro2::TokenStream,
) -> ParseResult<libgraphql_core::ast::schema::Document> {
    let span_map = Rc::new(RefCell::new(HashMap::new()));
    let token_source =
        RustMacroGraphQLTokenSource::new(input, span_map);
    let parser =
        GraphQLParser::from_token_source(token_source);
    let result = parser.parse_schema_document();
    let mut errors = result.errors().to_vec();
    let doc = result.into_ast();
    let compat =
        libgraphql_parser::compat_graphql_parser_v0_4
            ::to_graphql_parser_schema_ast(&doc);
    errors.extend(compat.errors().to_vec());
    let legacy_doc = compat.into_ast();
    if errors.is_empty() {
        ParseResult::Ok(legacy_doc)
    } else {
        ParseResult::Recovered {
            ast: legacy_doc,
            errors,
        }
    }
}

#[test]
fn test_trailing_comma_in_field_definitions() {
    let input = quote! {
        type User {
            firstName: String,
            lastName: String,
        }
    };

    let result = parse_schema(input);

    assert!(
        result.is_ok(),
        "Should accept trailing comma in field definitions",
    );
    let doc = result.into_valid_ast().unwrap();
    assert_eq!(doc.definitions.len(), 1);
}

#[test]
fn test_trailing_comma_in_field_arguments() {
    let input = quote! {
        type Query {
            user(id: ID, name: String,): User
        }
    };

    let result = parse_schema(input);

    assert!(
        result.is_ok(),
        "Should accept trailing comma in field arguments",
    );
}

#[test]
fn test_trailing_comma_in_directive_arguments() {
    let input = quote! {
        type User @auth(role: "admin", scope: "read",) {
            id: ID
        }
    };

    let result = parse_schema(input);

    assert!(
        result.is_ok(),
        "Should accept trailing comma in directive arguments",
    );
}

#[test]
fn test_trailing_comma_in_input_object_fields() {
    let input = quote! {
        input UserInput {
            firstName: String,
            lastName: String,
        }
    };

    let result = parse_schema(input);

    assert!(
        result.is_ok(),
        "Should accept trailing comma in input object fields",
    );
}

#[test]
fn test_trailing_comma_in_enum_values() {
    let input = quote! {
        enum Role {
            ADMIN,
            USER,
            GUEST,
        }
    };

    let result = parse_schema(input);

    assert!(
        result.is_ok(),
        "Should accept trailing comma in enum values",
    );
}

#[test]
fn test_trailing_comma_in_list_value() {
    let input = quote! {
        type Query {
            users: [User] @default(value: [1, 2, 3,])
        }
    };

    let result = parse_schema(input);

    assert!(
        result.is_ok(),
        "Should accept trailing comma in list values",
    );
}

#[test]
fn test_trailing_comma_in_object_value() {
    let input = quote! {
        type User {
            name: String @default(value: { first: "John", last: "Doe", })
        }
    };

    let result = parse_schema(input);

    assert!(
        result.is_ok(),
        "Should accept trailing comma in object values",
    );
}

#[test]
fn test_trailing_comma_in_input_value_with_defaults() {
    let input = quote! {
        input PaginationInput {
            page: Int = 1,
            limit: Int = 10,
        }
    };

    let result = parse_schema(input);

    assert!(
        result.is_ok(),
        "Should accept trailing comma in input values with defaults",
    );
}


#[test]
fn test_nested_trailing_commas() {
    let input = quote! {
        type Query {
            complexField(
                input: InputType,
                options: OptionsInput,
            ): Result @validate(
                rules: ["required", "email",],
                config: { strict: true, },
            )
        }
    };

    let result = parse_schema(input);

    assert!(
        result.is_ok(),
        "Should accept nested trailing commas",
    );
}

#[test]
fn test_trailing_comma_in_schema_definition() {
    let input = quote! {
        schema {
            query: Query,
            mutation: Mutation,
        }
    };

    let result = parse_schema(input);

    assert!(
        result.is_ok(),
        "Should accept trailing comma in schema definition",
    );
}

#[test]
fn test_no_comma_between_fields_still_works() {
    // GraphQL doesn't require commas at all
    let input = quote! {
        type User {
            firstName: String
            lastName: String
        }
    };

    let result = parse_schema(input);

    assert!(
        result.is_ok(),
        "Should accept fields without commas",
    );
}

#[test]
fn test_mixed_comma_styles() {
    let input = quote! {
        type User {
            id: ID,
            firstName: String
            lastName: String,
        }
    };

    let result = parse_schema(input);

    assert!(
        result.is_ok(),
        "Should accept mixed comma usage",
    );
}

#[test]
fn test_trailing_comma_in_directive_definition_locations() {
    let input = quote! {
        directive @auth(role: String,) on FIELD | QUERY | MUTATION
    };

    let result = parse_schema(input);

    assert!(
        result.is_ok(),
        "Should accept trailing comma in directive definition arguments",
    );
}

