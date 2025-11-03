//! Tests for interesting edge cases in GraphQL schema parsing
//!
//! These test cases cover various edge cases that our parser must handle correctly,
//! including embedded quotes, negative numbers, and complex description patterns.

use crate::graphql_schema_parser::GraphQLSchemaParser;
use crate::rust_to_graphql_token_adapter::RustToGraphQLTokenAdapter;
use libgraphql_core::ast;
use quote::quote;

#[test]
fn test_embedded_quotes_in_description() {
    // Using raw string to handle embedded quotes
    let input = quote! {
        type User {
            r#"The user's "primary" address"#
            address: String!
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should handle embedded quotes in descriptions");
    let doc = result.unwrap();

    // Verify the field has the description
    if let Some(ast::schema::Definition::TypeDefinition(
        ast::schema::TypeDefinition::Object(obj)
    )) = doc.definitions.first() {
        let field = &obj.fields[0];
        assert_eq!(field.name, "address");
        assert!(field.description.is_some());
        let desc = field.description.as_ref().unwrap();
        assert!(desc.contains("user's"), "Description should contain: user's");
        assert!(desc.contains(r#""primary""#), "Description should contain: \"primary\"");
    } else {
        panic!("Expected object type definition");
    }
}

#[test]
fn test_multiline_description_with_embedded_quotes() {
    // Using raw string for multi-line descriptions with embedded quotes
    let input = quote! {
        type Response {
            r#"The formatted output string.
            Special values are marked with "UNKNOWN"."#
            output: String
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should handle multi-line descriptions with embedded quotes");
    let doc = result.unwrap();

    // Verify the description contains the expected content
    if let Some(ast::schema::Definition::TypeDefinition(
        ast::schema::TypeDefinition::Object(obj)
    )) = doc.definitions.first() {
        let field = &obj.fields[0];
        assert!(field.description.is_some());
        let desc = field.description.as_ref().unwrap();
        assert!(desc.contains(r#""UNKNOWN""#), "Description should contain: \"UNKNOWN\"");
    }
}

#[test]
fn test_field_with_argument_descriptions() {
    // Arguments with descriptions
    let input = quote! {
        type DataProcessor {
            process(
                """The target format for processing."""
                format: String
            ): String
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should handle field arguments with descriptions");
}

#[test]
fn test_field_with_multiple_argument_descriptions() {
    // Multiple arguments with descriptions
    let input = quote! {
        type DataProcessor {
            filter(
                """List of filter criteria to apply."""
                criteria: [FilterCriterion]

                """
                Include related records.
                Default to true.
                """
                includeRelated: Boolean = true

                """Custom list of field names."""
                fields: [String]
            ): [Record]
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should handle multiple field arguments with descriptions");
}

#[test]
fn test_negative_default_values_in_input_types() {
    // Negative default values in input types
    let input = quote! {
        input PaginationInput {
            limit: Int = -1
            threshold: Float = -0.5
            offset: Int = -100
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should handle negative default values in input types");

    let doc = result.unwrap();
    // Verify the negative default values were parsed correctly
    if let Some(ast::schema::Definition::TypeDefinition(
        ast::schema::TypeDefinition::InputObject(input_obj)
    )) = doc.definitions.first() {
        assert_eq!(input_obj.fields[0].name, "limit");
        assert!(input_obj.fields[0].default_value.is_some());
        // The default value should be parsed as a negative integer
    }
}

#[test]
fn test_negative_default_in_field_arguments() {
    // Negative defaults can also appear in field arguments
    let input = quote! {
        type Query {
            records(limit: Int = -1, offset: Int = -10): [Record]
        }

        type Record {
            id: ID
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should handle negative default values in field arguments");
}

#[test]
fn test_complex_description_patterns() {
    // Combination of multiple edge cases: escaped quotes + negative numbers
    let input = quote! {
        type Query {
            """
            Search for records. Returns results with \"fuzzy\" matching.
            Supports operators like \"+\" and \"-\" for filtering.
            """
            search(
                """The search query string."""
                query: String!

                """Maximum number of results. Default is -1 (unlimited)."""
                limit: Int = -1

                """Whether to include \"draft\" versions."""
                includeDrafts: Boolean = false
            ): [Record]
        }

        type Record {
            r#"The record's "canonical" identifier."#
            id: ID!
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should handle complex combination of description patterns");
}

#[test]
fn test_enum_values_with_descriptions() {
    // Enum values with descriptions
    let input = quote! {
        enum AccessLevel {
            """Read-only access"""
            READ

            """Full write access"""
            WRITE
        }
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should handle enum values with descriptions");
}

#[test]
fn test_directive_with_description_and_arguments() {
    // Directives with descriptions and multiple arguments
    let input = quote! {
        """
        Marks a field as deprecated with a \"reason\".
        """
        directive @deprecated(
            """The reason for deprecation."""
            reason: String = "No longer supported"
        ) on FIELD_DEFINITION | ENUM_VALUE
    };

    let adapter = RustToGraphQLTokenAdapter::new(input);
    let parser = GraphQLSchemaParser::new(adapter);
    let result = parser.parse_document();

    assert!(result.is_ok(), "Should handle directives with descriptions and arguments with defaults");
}

#[test]
fn test_raw_string_equals_triple_quote() {
    // Verify that raw strings and triple-quoted strings produce identical schemas
    let raw_string_input = quote! {
        type User {
            r#"The user's "preferred" name"#
            displayName: String!
        }
    };

    let triple_quote_input = quote! {
        type User {
            """The user's \"preferred\" name"""
            displayName: String!
        }
    };

    let adapter1 = RustToGraphQLTokenAdapter::new(raw_string_input);
    let parser1 = GraphQLSchemaParser::new(adapter1);
    let result1 = parser1.parse_document();

    let adapter2 = RustToGraphQLTokenAdapter::new(triple_quote_input);
    let parser2 = GraphQLSchemaParser::new(adapter2);
    let result2 = parser2.parse_document();

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let doc1 = result1.unwrap();
    let doc2 = result2.unwrap();

    // Both should parse identically
    if let (
        Some(ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::Object(obj1))),
        Some(ast::schema::Definition::TypeDefinition(ast::schema::TypeDefinition::Object(obj2))),
    ) = (doc1.definitions.first(), doc2.definitions.first())
    {
        assert_eq!(obj1.name, obj2.name);
        assert_eq!(obj1.fields[0].name, obj2.fields[0].name);
        assert_eq!(obj1.fields[0].description, obj2.fields[0].description);
        assert!(obj1.fields[0].description.as_ref().unwrap().contains(r#""preferred""#));
    } else {
        panic!("Expected object type definitions");
    }
}
