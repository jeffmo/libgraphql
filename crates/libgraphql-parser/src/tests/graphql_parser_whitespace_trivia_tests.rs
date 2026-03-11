//! Tests for whitespace and trivia capture in the parser.
//!
//! All tests use the default (full-fidelity) config, which sets
//! `retain_syntax = true`. Trivia is accessed through the `*.syntax` fields
//! on AST nodes — specifically through `GraphQLToken.preceding_trivia`.
//!
//! Written by Claude Code, reviewed by a human.

use crate::ast;
use crate::tests::utils::parse_executable;
use crate::token::GraphQLTriviaToken;

/// Verifies that whitespace between tokens is captured as `Whitespace` trivia
/// on the following token's `preceding_trivia`.
///
/// Parses `"query  {  field  }"` (extra spaces) and checks that the `{` token
/// has whitespace trivia from the double space before it.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-White-Space>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn trivia_whitespace_between_tokens() {
    let source = "query  {  field  }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        // The SelectionSet `{` token should have whitespace trivia ("  ")
        let ss_syntax = op.selection_set.syntax.as_ref().unwrap();
        let open_brace = &ss_syntax.braces.open;
        let has_ws = open_brace.preceding_trivia.iter().any(|t| {
            matches!(t, GraphQLTriviaToken::Whitespace { value, .. } if value == "  ")
        });
        assert!(has_ws, "Open brace should have double-space whitespace trivia");

        // The field's name token should also have whitespace trivia ("  ")
        if let ast::Selection::Field(field) = &op.selection_set.selections[0] {
            let name_syntax = field.name.syntax.as_ref().unwrap();
            let has_ws = name_syntax.token.preceding_trivia.iter().any(|t| {
                matches!(t, GraphQLTriviaToken::Whitespace { value, .. } if value == "  ")
            });
            assert!(
                has_ws,
                "Field name token should have double-space whitespace trivia",
            );
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected an OperationDefinition");
    }
}

/// Verifies that newline whitespace is captured as trivia in multiline queries.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-White-Space>
/// <https://spec.graphql.org/September2025/#sec-Line-Terminators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn trivia_newlines() {
    let source = "query {\n  field\n}";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        // The field name token should have newline+indent whitespace trivia
        if let ast::Selection::Field(field) = &op.selection_set.selections[0] {
            let name_syntax = field.name.syntax.as_ref().unwrap();
            let has_newline_ws = name_syntax.token.preceding_trivia.iter().any(|t| {
                matches!(t, GraphQLTriviaToken::Whitespace { value, .. } if value == "\n  ")
            });
            assert!(
                has_newline_ws,
                "Field name token should have \"\\n  \" whitespace trivia",
            );
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected an OperationDefinition");
    }
}

/// Verifies that a comment is captured as `Comment` trivia on the following
/// token.
///
/// Parses `"# comment\nquery { f }"` and checks that the `query` keyword
/// token has Comment trivia with the expected value.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Comments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn trivia_comment_captured() {
    let source = "# comment\nquery { f }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        let op_syntax = op.syntax.as_ref().unwrap();
        let kw = op_syntax.operation_keyword.as_ref().unwrap();

        let has_comment = kw.preceding_trivia.iter().any(|t| {
            matches!(t, GraphQLTriviaToken::Comment { value, .. } if value == " comment")
        });
        assert!(
            has_comment,
            "query keyword should have Comment trivia with value \" comment\"",
        );
    } else {
        panic!("Expected an OperationDefinition");
    }
}

/// Verifies that commas are captured as `Comma` trivia.
///
/// In GraphQL, commas are optional insignificant separators treated as
/// whitespace. Parses `"{ a, b }"` and checks that `b`'s name token has
/// Comma trivia.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Insignificant-Commas>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn trivia_comma_captured() {
    let source = "{ a, b }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        assert_eq!(op.selection_set.selections.len(), 2);

        // The second field "b" should have Comma trivia
        if let ast::Selection::Field(field_b) = &op.selection_set.selections[1] {
            assert_eq!(field_b.name.value, "b");
            let name_syntax = field_b.name.syntax.as_ref().unwrap();
            let has_comma = name_syntax.token.preceding_trivia.iter().any(|t| {
                matches!(t, GraphQLTriviaToken::Comma { .. })
            });
            assert!(has_comma, "Field 'b' name token should have Comma trivia");
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected an OperationDefinition");
    }
}

/// Verifies that leading whitespace before the first token is captured
/// as trivia on that token.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-White-Space>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn trivia_leading_whitespace() {
    let source = "  query { f }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        let op_syntax = op.syntax.as_ref().unwrap();
        let kw = op_syntax.operation_keyword.as_ref().unwrap();

        let has_leading_ws = kw.preceding_trivia.iter().any(|t| {
            matches!(t, GraphQLTriviaToken::Whitespace { value, .. } if value == "  ")
        });
        assert!(
            has_leading_ws,
            "query keyword should have leading whitespace trivia",
        );
    } else {
        panic!("Expected an OperationDefinition");
    }
}

/// Verifies that trailing trivia after the last definition is captured in
/// `doc.syntax.trailing_trivia`.
///
/// Parses `"query { f }\n  "` and checks that the document's trailing trivia
/// contains the expected trailing whitespace.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-White-Space>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn trivia_trailing_in_document() {
    let source = "query { f }\n  ";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();
    let doc_syntax = doc.syntax.as_ref().unwrap();

    let has_trailing_ws = doc_syntax.trailing_trivia.iter().any(|t| {
        matches!(t, GraphQLTriviaToken::Whitespace { value, .. } if value == "\n  ")
    });
    assert!(
        has_trailing_ws,
        "Document trailing trivia should contain \"\\n  \" Whitespace",
    );
}

/// Verifies that tightly packed tokens (no whitespace) produce no Whitespace
/// trivia. Parses `"{f}"` and checks that the `f` name token has no
/// `Whitespace` trivia.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-White-Space>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn trivia_no_whitespace_adjacent_tokens() {
    let source = "{f}";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        if let ast::Selection::Field(field) = &op.selection_set.selections[0] {
            let name_syntax = field.name.syntax.as_ref().unwrap();
            let has_ws = name_syntax.token.preceding_trivia.iter().any(|t| {
                matches!(t, GraphQLTriviaToken::Whitespace { .. })
            });
            assert!(
                !has_ws,
                "Field name should have no Whitespace trivia when adjacent to brace",
            );
        } else {
            panic!("Expected a Field selection");
        }
    } else {
        panic!("Expected an OperationDefinition");
    }
}

/// Verifies that a whitespace trivia token's span byte range corresponds to
/// the actual whitespace in the source string. Extracts a whitespace trivia
/// span and checks that the source substring at that byte range matches the
/// trivia value.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-White-Space>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn trivia_span_matches_source_slice() {
    let source = "query   { f }";
    let result = parse_executable(source);
    assert!(!result.has_errors());

    let doc = result.into_valid_ast().unwrap();
    if let ast::Definition::OperationDefinition(op) = &doc.definitions[0] {
        // The "{" token should have whitespace trivia ("   " — three spaces)
        let ss_syntax = op.selection_set.syntax.as_ref().unwrap();
        let open_brace = &ss_syntax.braces.open;

        for trivia in open_brace.preceding_trivia.iter() {
            if let GraphQLTriviaToken::Whitespace { value, span } = trivia {
                let start = span.start as usize;
                let end = span.end as usize;
                let source_slice = &source[start..end];
                assert_eq!(
                    source_slice, value,
                    "Trivia span byte range should match the trivia value in the source",
                );
                return;
            }
        }
        panic!("Expected to find Whitespace trivia on the open brace token");
    } else {
        panic!("Expected an OperationDefinition");
    }
}
