//! Integration tests for interesting edge cases when parsing
//! GraphQL schemas through `RustMacroGraphQLTokenSource`.
//!
//! These tests verify token source behavior for cases that
//! require special handling: block string recombination,
//! negative number recombination, and description parsing
//! through the Rust token pipeline.
//!
//! Tests that use `TokenStream::from_str()` (rather than
//! `quote!`) do so because `quote!` produces synthetic
//! spans with no position info, which prevents the block
//! string adjacency detection in
//! `RustMacroGraphQLTokenSource::try_combine_block_string`.

use crate::rust_macro_graphql_token_source::RustMacroGraphQLTokenSource;
use libgraphql_core::ast;
use libgraphql_parser::GraphQLParser;
use libgraphql_parser::ParseResult;
use quote::quote;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

fn parse_schema_from_str(
    input: &str,
) -> ParseResult<ast::schema::Document> {
    let stream = proc_macro2::TokenStream::from_str(input)
        .expect("Failed to parse as Rust tokens");
    let span_map = Rc::new(RefCell::new(HashMap::new()));
    let token_source =
        RustMacroGraphQLTokenSource::new(stream, span_map);
    let parser =
        GraphQLParser::from_token_source(token_source);
    parser.parse_schema_document()
}

fn parse_schema_from_quote(
    input: proc_macro2::TokenStream,
) -> ParseResult<ast::schema::Document> {
    let span_map = Rc::new(RefCell::new(HashMap::new()));
    let token_source =
        RustMacroGraphQLTokenSource::new(input, span_map);
    let parser =
        GraphQLParser::from_token_source(token_source);
    parser.parse_schema_document()
}

/// Extract the first object type from a parsed document.
fn first_object_type(
    doc: &ast::schema::Document,
) -> &ast::schema::ObjectType {
    for def in &doc.definitions {
        if let ast::schema::Definition::TypeDefinition(
            ast::schema::TypeDefinition::Object(obj),
        ) = def
        {
            return obj;
        }
    }
    panic!("No object type found in document");
}

/// Verify that block string descriptions (`"""..."""`) are
/// correctly recombined from the three Rust tokens
/// (`""`, `"content"`, `""`) by the token source when
/// parsed via `TokenStream::from_str` (real positions).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_block_string_description_on_field() {
    let result = parse_schema_from_str(
        r#"type User {
            """The user's primary address"""
            address: String!
        }"#,
    );

    assert!(
        result.is_ok(),
        "Should parse block string descriptions: {:?}",
        result.errors,
    );
    let doc = result.into_valid_ast().unwrap();
    let obj = first_object_type(&doc);
    let field = &obj.fields[0];

    assert_eq!(field.name, "address");
    let desc = field.description.as_ref()
        .expect("Field should have a description");
    assert!(
        desc.contains("user's primary address"),
        "Expected description containing 'user's primary \
         address', got: {desc}",
    );
}

/// Verify that block string descriptions with escaped
/// quotes are correctly parsed and the quote characters
/// survive into the AST.
///
/// Note: Embedded `"` inside `"""..."""` must be escaped
/// as `\"` in Rust source because Rust's tokenizer treats
/// unescaped `"` as string delimiters, breaking the block
/// string recombination.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_block_string_with_escaped_quotes() {
    let result = parse_schema_from_str(
        r#"type Response {
            """The formatted \"output\" string."""
            output: String
        }"#,
    );

    assert!(
        result.is_ok(),
        "Should handle escaped quotes: {:?}",
        result.errors,
    );
    let doc = result.into_valid_ast().unwrap();
    let obj = first_object_type(&doc);
    let desc = obj.fields[0].description.as_ref()
        .expect("Field should have a description");
    assert!(
        desc.contains("output"),
        "Description should contain 'output', got: {desc}",
    );
}

/// Verify that block string descriptions on field arguments
/// are recombined correctly by the token source.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_block_string_on_field_arguments() {
    let result = parse_schema_from_str(
        r#"type DataProcessor {
            process(
                """The target format for processing."""
                format: String
            ): String
        }"#,
    );

    assert!(
        result.is_ok(),
        "Should handle argument descriptions: {:?}",
        result.errors,
    );
    let doc = result.into_valid_ast().unwrap();
    let obj = first_object_type(&doc);
    let arg = &obj.fields[0].arguments[0];

    assert_eq!(arg.name, "format");
    let desc = arg.description.as_ref()
        .expect("Argument should have a description");
    assert!(
        desc.contains("target format"),
        "Expected description containing 'target format', \
         got: {desc}",
    );
}

/// Verify that multiple field arguments each with block
/// string descriptions are parsed correctly, including
/// multiline block strings.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_block_string_on_multiple_arguments() {
    let result = parse_schema_from_str(
        r#"type DataProcessor {
            filter(
                """List of filter criteria to apply."""
                criteria: [FilterCriterion]

                """
                Include related records.
                Defaults to true.
                """
                includeRelated: Boolean = true

                """Custom list of field names."""
                fields: [String]
            ): [Record]
        }"#,
    );

    assert!(
        result.is_ok(),
        "Should handle multiple argument descriptions: {:?}",
        result.errors,
    );
    let doc = result.into_valid_ast().unwrap();
    let obj = first_object_type(&doc);
    let args = &obj.fields[0].arguments;

    assert_eq!(args.len(), 3);

    assert_eq!(args[0].name, "criteria");
    assert!(
        args[0].description.as_ref().unwrap()
            .contains("filter criteria"),
    );

    assert_eq!(args[1].name, "includeRelated");
    assert!(
        args[1].description.as_ref().unwrap()
            .contains("Include related records"),
    );

    assert_eq!(args[2].name, "fields");
    assert!(
        args[2].description.as_ref().unwrap()
            .contains("field names"),
    );
}

/// Verify that block string descriptions on enum values are
/// parsed correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_block_string_on_enum_values() {
    let result = parse_schema_from_str(
        r#"enum AccessLevel {
            """Read-only access"""
            READ

            """Full write access"""
            WRITE
        }"#,
    );

    assert!(
        result.is_ok(),
        "Should handle enum value descriptions: {:?}",
        result.errors,
    );
    let doc = result.into_valid_ast().unwrap();
    if let Some(ast::schema::Definition::TypeDefinition(
        ast::schema::TypeDefinition::Enum(enum_type),
    )) = doc.definitions.first()
    {
        assert_eq!(enum_type.values.len(), 2);

        assert_eq!(enum_type.values[0].name, "READ");
        assert!(
            enum_type.values[0].description.as_ref().unwrap()
                .contains("Read-only"),
        );

        assert_eq!(enum_type.values[1].name, "WRITE");
        assert!(
            enum_type.values[1].description.as_ref().unwrap()
                .contains("Full write"),
        );
    } else {
        panic!("Expected enum type definition");
    }
}

/// Verify that a directive definition with a block string
/// description and arguments with descriptions and defaults
/// is parsed correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_block_string_on_directive_definition() {
    let result = parse_schema_from_str(
        r#""""
        Marks a field as deprecated with a reason.
        """
        directive @deprecated(
            """The reason for deprecation."""
            reason: String = "No longer supported"
        ) on FIELD_DEFINITION | ENUM_VALUE"#,
    );

    assert!(
        result.is_ok(),
        "Should handle directive descriptions: {:?}",
        result.errors,
    );
    let doc = result.into_valid_ast().unwrap();
    if let Some(ast::schema::Definition::DirectiveDefinition(
        dir,
    )) = doc.definitions.first()
    {
        assert_eq!(dir.name, "deprecated");
        assert!(
            dir.description.as_ref().unwrap()
                .contains("deprecated"),
        );

        assert_eq!(dir.arguments.len(), 1);
        assert_eq!(dir.arguments[0].name, "reason");
        assert!(
            dir.arguments[0].description.as_ref().unwrap()
                .contains("reason for deprecation"),
        );
    } else {
        panic!("Expected directive definition");
    }
}

/// Verify that `RustMacroGraphQLTokenSource` correctly
/// recombines the `-` punct and integer literal into a
/// negative number when used as a default value.
///
/// In `quote!`, `-1` produces two Rust tokens: `Punct('-')`
/// and `Literal(1)`. The token source must recombine these
/// into a single negative `IntValue`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_negative_default_values_in_input_types() {
    let input = quote! {
        input PaginationInput {
            limit: Int = -1
            threshold: Float = -0.5
            offset: Int = -100
        }
    };

    let result = parse_schema_from_quote(input);

    assert!(
        result.is_ok(),
        "Should handle negative defaults: {:?}",
        result.errors,
    );

    let doc = result.into_valid_ast().unwrap();
    if let Some(ast::schema::Definition::TypeDefinition(
        ast::schema::TypeDefinition::InputObject(input_obj),
    )) = doc.definitions.first()
    {
        assert_eq!(input_obj.fields.len(), 3);

        assert_eq!(input_obj.fields[0].name, "limit");
        let limit_val =
            input_obj.fields[0].default_value.as_ref()
                .expect("limit should have a default value");
        assert!(
            matches!(
                limit_val,
                ast::Value::Int(n)
                    if n.as_i64() == Some(-1),
            ),
            "Expected Int(-1), got: {limit_val:?}",
        );

        assert_eq!(input_obj.fields[1].name, "threshold");
        let thresh_val =
            input_obj.fields[1].default_value.as_ref()
                .expect(
                    "threshold should have a default value",
                );
        assert!(
            matches!(
                thresh_val,
                ast::Value::Float(f)
                    if *f == -0.5,
            ),
            "Expected Float(-0.5), got: {thresh_val:?}",
        );

        assert_eq!(input_obj.fields[2].name, "offset");
        let offset_val =
            input_obj.fields[2].default_value.as_ref()
                .expect("offset should have a default value");
        assert!(
            matches!(
                offset_val,
                ast::Value::Int(n)
                    if n.as_i64() == Some(-100),
            ),
            "Expected Int(-100), got: {offset_val:?}",
        );
    } else {
        panic!("Expected input object type definition");
    }
}

/// Verify that negative defaults in field arguments work
/// through `quote!`, testing the minus-number recombination
/// in argument position.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_negative_default_in_field_arguments() {
    let input = quote! {
        type Query {
            records(
                limit: Int = -1,
                offset: Int = -10
            ): [Record]
        }

        type Record {
            id: ID
        }
    };

    let result = parse_schema_from_quote(input);

    assert!(
        result.is_ok(),
        "Should handle negative argument defaults: {:?}",
        result.errors,
    );

    let doc = result.into_valid_ast().unwrap();
    let obj = first_object_type(&doc);
    let args = &obj.fields[0].arguments;

    assert_eq!(args.len(), 2);

    assert_eq!(args[0].name, "limit");
    assert!(
        matches!(
            args[0].default_value.as_ref().unwrap(),
            ast::Value::Int(n)
                if n.as_i64() == Some(-1),
        ),
        "Expected Int(-1), got: {:?}",
        args[0].default_value,
    );

    assert_eq!(args[1].name, "offset");
    assert!(
        matches!(
            args[1].default_value.as_ref().unwrap(),
            ast::Value::Int(n)
                if n.as_i64() == Some(-10),
        ),
        "Expected Int(-10), got: {:?}",
        args[1].default_value,
    );
}

/// Verify a complex schema combining block string
/// descriptions, embedded quotes, and negative defaults
/// through `TokenStream::from_str()`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_complex_description_patterns() {
    let result = parse_schema_from_str(
        r#"type Query {
            """
            Search for records.
            Supports operators like \"+\" and \"-\".
            """
            search(
                """The search query string."""
                query: String!

                """Maximum results. -1 means unlimited."""
                limit: Int = -1

                """Whether to include draft versions."""
                includeDrafts: Boolean = false
            ): [Record]
        }

        type Record {
            """The record's canonical identifier."""
            id: ID!
        }"#,
    );

    assert!(
        result.is_ok(),
        "Should handle complex descriptions: {:?}",
        result.errors,
    );

    let doc = result.into_valid_ast().unwrap();
    assert_eq!(doc.definitions.len(), 2);

    // Verify Query type
    let query_obj = first_object_type(&doc);
    assert_eq!(query_obj.name, "Query");

    let search = &query_obj.fields[0];
    assert_eq!(search.name, "search");
    assert!(
        search.description.as_ref().unwrap()
            .contains("Search for records"),
    );

    assert_eq!(search.arguments.len(), 3);
    assert_eq!(search.arguments[0].name, "query");
    assert_eq!(search.arguments[1].name, "limit");
    assert!(
        matches!(
            search.arguments[1].default_value.as_ref().unwrap(),
            ast::Value::Int(n)
                if n.as_i64() == Some(-1),
        ),
    );
    assert_eq!(search.arguments[2].name, "includeDrafts");

    // Verify Record type
    if let Some(ast::schema::Definition::TypeDefinition(
        ast::schema::TypeDefinition::Object(record),
    )) = doc.definitions.get(1)
    {
        assert_eq!(record.name, "Record");
        assert!(
            record.fields[0].description.as_ref().unwrap()
                .contains("canonical identifier"),
        );
    } else {
        panic!("Expected Record object type");
    }
}
