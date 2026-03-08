//! Source-slice round-trip tests for the GraphQL parser.
//!
//! These tests verify that parsing a GraphQL source string and then
//! reconstructing it via `doc.to_source(Some(source))` produces output
//! that is character-for-character identical to the original input.
//!
//! This validates that the parser's span tracking is correct: every
//! AST node records accurate byte offsets into the original source, so
//! slicing `&source[start..end]` for each node reproduces the original
//! text verbatim.
//!
//! Written by Claude Code, reviewed by a human.

use crate::ast;
use crate::ast::AstNode;
use crate::tests::utils::parse_executable;
use crate::tests::utils::parse_mixed;
use crate::tests::utils::parse_schema;

// =============================================================================
// Helpers
// =============================================================================

/// Parse source as an executable document, then reconstruct via
/// source-slice and assert the result matches the original.
fn assert_roundtrip_executable(source: &str) {
    let result = parse_executable(source);
    assert!(
        !result.has_errors(),
        "Parse failed:\n{}",
        result.format_errors(),
    );
    let doc = result.into_valid_ast().unwrap();
    let reconstructed = doc.to_source(Some(source));
    assert_eq!(reconstructed, source, "Round-trip mismatch");
}

/// Parse source as a schema document, then reconstruct via
/// source-slice and assert the result matches the original.
fn assert_roundtrip_schema(source: &str) {
    let result = parse_schema(source);
    assert!(
        !result.has_errors(),
        "Parse failed:\n{}",
        result.format_errors(),
    );
    let doc = result.into_valid_ast().unwrap();
    let reconstructed = doc.to_source(Some(source));
    assert_eq!(reconstructed, source, "Round-trip mismatch");
}

/// Parse source as a mixed document, then reconstruct via
/// source-slice and assert the result matches the original.
fn assert_roundtrip_mixed(source: &str) {
    let result = parse_mixed(source);
    assert!(
        !result.has_errors(),
        "Parse failed:\n{}",
        result.format_errors(),
    );
    let doc = result.into_valid_ast().unwrap();
    let reconstructed = doc.to_source(Some(source));
    assert_eq!(reconstructed, source, "Round-trip mismatch");
}

// =============================================================================
// 1. Operations (executable)
// =============================================================================

/// Verifies source-slice round trip for a shorthand query (anonymous
/// selection set without the `query` keyword).
///
/// Also verifies sub-node slicing: the single field's
/// `to_source(Some(source))` should return just `"field"`.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_shorthand_query() {
    let source = "{ field }";
    assert_roundtrip_executable(source);

    // Sub-node assertion: the field should slice to just "field".
    let doc = parse_executable(source).into_valid_ast().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        if let ast::Selection::Field(field) =
            &op.selection_set.selections[0]
        {
            assert_eq!(
                field.to_source(Some(source)),
                "field",
                "Sub-node field slice mismatch",
            );
        } else {
            panic!("Expected Field selection");
        }
    } else {
        panic!("Expected OperationDefinition");
    }
}

/// Verifies source-slice round trip for a named query with multiple
/// fields.
///
/// Also verifies sub-node slicing: the operation's
/// `to_source(Some(source))` should reproduce the full operation text.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_named_query() {
    let source = "query GetUser { name age }";
    assert_roundtrip_executable(source);

    // Sub-node assertion: the operation should slice to the full
    // source since it's the only definition.
    let doc = parse_executable(source).into_valid_ast().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        assert_eq!(
            op.to_source(Some(source)),
            source,
            "Sub-node operation slice mismatch",
        );
    } else {
        panic!("Expected OperationDefinition");
    }
}

/// Verifies source-slice round trip for a query with variable
/// definitions, including a default value.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Variables>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_query_with_variables() {
    let source =
        "query Q($id: ID!, $limit: Int = 10) { user(id: $id) { name } }";
    assert_roundtrip_executable(source);
}

/// Verifies source-slice round trip for a mutation operation.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_mutation() {
    let source =
        "mutation CreateUser($name: String!) { createUser(name: $name) { id } }";
    assert_roundtrip_executable(source);
}

/// Verifies source-slice round trip for a subscription operation.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Operations>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_subscription() {
    let source = "subscription OnMessage { messageAdded { text } }";
    assert_roundtrip_executable(source);
}

// =============================================================================
// 2. Fragments (executable)
// =============================================================================

/// Verifies source-slice round trip for a document containing both a
/// fragment definition and a query that uses a fragment spread.
///
/// Also verifies sub-node slicing: the fragment definition's
/// `to_source(Some(source))` should return just the fragment portion,
/// and the query operation should return just the query portion.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_fragment_definition() {
    let source =
        "fragment UserFields on User { name email }\n\
         query { user { ...UserFields } }";
    assert_roundtrip_executable(source);

    // Sub-node assertions: each definition should slice to its own
    // text.
    let doc = parse_executable(source).into_valid_ast().unwrap();
    assert_eq!(doc.definitions.len(), 2);

    if let ast::Definition::FragmentDefinition(frag) = &doc.definitions[0] {
        assert_eq!(
            frag.to_source(Some(source)),
            "fragment UserFields on User { name email }",
            "Sub-node fragment slice mismatch",
        );
    } else {
        panic!("Expected FragmentDefinition");
    }

    if let ast::Definition::OperationDefinition(op) = &doc.definitions[1] {
        assert_eq!(
            op.to_source(Some(source)),
            "query { user { ...UserFields } }",
            "Sub-node query slice mismatch",
        );
    } else {
        panic!("Expected OperationDefinition");
    }
}

/// Verifies source-slice round trip for inline fragments with type
/// conditions.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Inline-Fragments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_inline_fragment() {
    let source =
        "{ node { ... on User { name } ... on Bot { handle } } }";
    assert_roundtrip_executable(source);
}

// =============================================================================
// 3. Values (executable)
// =============================================================================

/// Verifies source-slice round trip for a query exercising all GraphQL
/// value types: Int, Float, String, Boolean, Null, Enum, List, and
/// InputObject.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Values>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_all_value_types() {
    let source = concat!(
        "{ field(",
        "int: 42, ",
        "float: 3.14, ",
        "str: \"hello\", ",
        "yes: true, ",
        "no: false, ",
        "nil: null, ",
        "enumVal: ACTIVE, ",
        "list: [1, 2], ",
        "obj: {key: \"val\"}",
        ") }",
    );
    assert_roundtrip_executable(source);
}

// =============================================================================
// 4. Directives (executable)
// =============================================================================

/// Verifies source-slice round trip for directives on both operations
/// and fields.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_directives() {
    let source =
        "query @skip(if: true) { field @include(if: false) }";
    assert_roundtrip_executable(source);
}

// =============================================================================
// 5. Schema types (schema)
// =============================================================================

/// Verifies source-slice round trip for an object type definition with
/// field arguments and descriptions.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_object_type() {
    let source = concat!(
        "type User {\n",
        "  name: String\n",
        "  age(minAge: Int = 0): Int\n",
        "}",
    );
    assert_roundtrip_schema(source);
}

/// Verifies source-slice round trip for an interface type definition
/// with an `implements` clause.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Interfaces>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_interface_type() {
    let source = "interface Node implements Entity { id: ID! }";
    assert_roundtrip_schema(source);
}

/// Verifies source-slice round trip for an enum type definition with
/// values.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_enum_type() {
    let source = "enum Status { ACTIVE INACTIVE PENDING }";
    assert_roundtrip_schema(source);
}

/// Verifies source-slice round trip for a union type definition.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Unions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_union_type() {
    let source = "union SearchResult = User | Post | Comment";
    assert_roundtrip_schema(source);
}

/// Verifies source-slice round trip for a scalar type definition.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Scalars>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_scalar_type() {
    let source = "scalar DateTime";
    assert_roundtrip_schema(source);
}

/// Verifies source-slice round trip for an input object type
/// definition.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_input_object_type() {
    let source = concat!(
        "input CreateUserInput {\n",
        "  name: String!\n",
        "  email: String\n",
        "}",
    );
    assert_roundtrip_schema(source);
}

/// Verifies source-slice round trip for a directive definition with
/// multiple locations.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_directive_definition() {
    let source =
        "directive @auth(role: String!) on FIELD_DEFINITION | OBJECT";
    assert_roundtrip_schema(source);
}

/// Verifies source-slice round trip for a schema definition with
/// root operation types.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Schema>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_schema_definition() {
    let source = "schema { query: Query mutation: Mutation }";
    assert_roundtrip_schema(source);
}

// =============================================================================
// 6. Extensions (schema)
// =============================================================================

/// Verifies source-slice round trip for all type extension kinds:
/// object, enum, union, interface, scalar, and input object.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_type_extensions() {
    let source = concat!(
        "extend type User { age: Int }\n",
        "\n",
        "extend enum Status { INACTIVE }\n",
        "\n",
        "extend union SearchResult = Photo\n",
        "\n",
        "extend interface Node { createdAt: DateTime }\n",
        "\n",
        "extend scalar Date @deprecated\n",
        "\n",
        "extend input CreateUserInput { nickname: String }",
    );
    assert_roundtrip_schema(source);
}

// =============================================================================
// 7. Whitespace & trivia (mixed)
// =============================================================================

/// Verifies source-slice round trip for a multiline document
/// containing `#` comments. Comments are trivia captured in spans, so
/// they must survive the round trip.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Comments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_multiline_with_comments() {
    let source = concat!(
        "# This is a comment\n",
        "type User {\n",
        "  # The name field\n",
        "  name: String\n",
        "  age: Int\n",
        "}",
    );
    assert_roundtrip_mixed(source);
}

/// Verifies source-slice round trip for a document with extra
/// whitespace — including leading/trailing whitespace and extra
/// spaces between tokens.
///
/// Source-slice mode should preserve all whitespace exactly since it
/// slices the original source by byte offsets.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_extra_whitespace() {
    let source = "  query   {   field   }  ";
    assert_roundtrip_mixed(source);
}

/// Verifies source-slice round trip for a document using commas as
/// insignificant list separators between fields.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Insignificant-Commas>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_commas() {
    let source = "{ a, b, c }";
    assert_roundtrip_mixed(source);
}

// =============================================================================
// 8. Descriptions (schema)
// =============================================================================

/// Verifies source-slice round trip for string and block-string
/// descriptions on types and fields.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Descriptions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn roundtrip_string_descriptions() {
    let source = concat!(
        "\"A user in the system\"\n",
        "type User {\n",
        "  \"The user's name\"\n",
        "  name: String\n",
        "  \"\"\"\n",
        "  The user's email address.\n",
        "  Must be unique.\n",
        "  \"\"\"\n",
        "  email: String!\n",
        "}",
    );
    assert_roundtrip_schema(source);
}
