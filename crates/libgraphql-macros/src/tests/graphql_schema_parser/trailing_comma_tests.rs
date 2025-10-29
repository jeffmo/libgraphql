/// Tests for trailing comma support in GraphQL schemas
///
/// According to the GraphQL September 2025 spec, commas are optional and "insignificant"
/// throughout GraphQL syntax. Trailing commas are explicitly allowed in:
/// - List values
/// - Input object values
/// - Argument lists
/// - And by extension, field lists, enum value lists, etc.
use crate::graphql_schema_parser::GraphQLSchemaParser;
use crate::rust_to_graphql_token_adapter::RustToGraphQLTokenAdapter;
use quote::quote;

#[test]
fn test_trailing_comma_in_field_definitions() {
    let input = quote! {
        type User {
            firstName: String,
            lastName: String,
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should accept trailing comma in field definitions");
    let doc = result.unwrap();
    assert_eq!(doc.definitions.len(), 1);
}

#[test]
fn test_trailing_comma_in_field_arguments() {
    let input = quote! {
        type Query {
            user(id: ID, name: String,): User
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should accept trailing comma in field arguments");
}

#[test]
fn test_trailing_comma_in_directive_arguments() {
    let input = quote! {
        type User @auth(role: "admin", scope: "read",) {
            id: ID
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should accept trailing comma in directive arguments");
}

#[test]
fn test_trailing_comma_in_input_object_fields() {
    let input = quote! {
        input UserInput {
            firstName: String,
            lastName: String,
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should accept trailing comma in input object fields");
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

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should accept trailing comma in enum values");
}

#[test]
fn test_trailing_comma_in_list_value() {
    let input = quote! {
        type Query {
            users: [User] @default(value: [1, 2, 3,])
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should accept trailing comma in list values");
}

#[test]
fn test_trailing_comma_in_object_value() {
    let input = quote! {
        type User {
            name: String @default(value: { first: "John", last: "Doe", })
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should accept trailing comma in object values");
}

#[test]
fn test_trailing_comma_in_input_value_with_defaults() {
    let input = quote! {
        input PaginationInput {
            page: Int = 1,
            limit: Int = 10,
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should accept trailing comma in input values with defaults");
}

#[test]
fn test_multiple_trailing_commas_not_allowed() {
    // GraphQL spec says "repeated commas do not represent missing values"
    // but this is in the context of list values, not field definitions
    // Let's test that we handle this gracefully
    let input = quote! {
        type User {
            firstName: String,,
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    // This should fail because ",," is invalid syntax
    assert!(result.is_err(), "Should reject multiple consecutive commas in field definitions");
}

#[test]
fn test_trailing_comma_with_no_fields_is_ok() {
    // Empty field list with trailing comma should still be a syntax error
    // because there are no fields
    let input = quote! {
        type User {
            ,
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_err(), "Should reject comma with no preceding field");
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

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should accept nested trailing commas");
}

#[test]
fn test_trailing_comma_in_schema_definition() {
    let input = quote! {
        schema {
            query: Query,
            mutation: Mutation,
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should accept trailing comma in schema definition");
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

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should accept fields without commas");
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

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should accept mixed comma usage");
}

#[test]
fn test_trailing_comma_in_directive_definition_locations() {
    let input = quote! {
        directive @auth(role: String,) on FIELD | QUERY | MUTATION
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should accept trailing comma in directive definition arguments");
}

#[test]
fn test_trailing_comma_in_empty_list() {
    let input = quote! {
        type Query {
            field: String @default(value: [,])
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    // Empty list with just a comma should be an error
    assert!(result.is_err(), "Should reject list with only a comma");
}
