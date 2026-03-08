//! Tests for the byte-scanning optimization in
//! `StrGraphQLTokenSource::lex_name()`.
//!
//! ## Optimization summary
//!
//! `lex_name()` scans name bytes directly via `source.as_bytes()`
//! and `is_name_continue_byte()` instead of calling `peek_char()` /
//! `consume()` per character. Since GraphQL names are ASCII-only
//! (`[_A-Za-z][_0-9A-Za-z]*`) and never contain newlines, only
//! `curr_byte_offset` needs updating. Line/column resolution is
//! deferred to SourceMap.
//!
//! ## What these tests verify
//!
//! - Byte spans for names of various lengths
//! - Consecutive names accumulate byte offsets correctly
//! - Names after newlines start at the correct line/col
//! - Names at EOF (no trailing content) are scanned correctly
//! - Single-character names (minimum length)
//! - Underscore-prefixed names (`__typename`) with byte scanning
//! - Keyword recognition (`true`, `false`, `null`) still works
//!   through the byte-scanning path
//!
//! Written by Claude Code, reviewed by a human.

use crate::token::GraphQLTokenKind;
use crate::token_source::GraphQLTokenSource;
use crate::token_source::StrGraphQLTokenSource;

// =============================================================================
// Batch position update correctness
// =============================================================================

/// Verifies that the byte-scanning loop produces correct start/end
/// spans for two names separated by whitespace.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_tracking_two_names() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("type Query")
            .collect_with_source_map();
    // Tokens: type, Query, Eof
    assert_eq!(tokens.len(), 3);

    // "type" at (0,0)-(0,4)
    let start = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(start.line(), 0);
    assert_eq!(start.col_utf8(), 0);
    assert_eq!(tokens[0].span.start, 0);
    let end = source_map.resolve_offset(tokens[0].span.end).unwrap();
    assert_eq!(end.line(), 0);
    assert_eq!(end.col_utf8(), 4);
    assert_eq!(tokens[0].span.end, 4);

    // "Query" at (0,5)-(0,10)
    let start = source_map.resolve_offset(tokens[1].span.start).unwrap();
    assert_eq!(start.line(), 0);
    assert_eq!(start.col_utf8(), 5);
    assert_eq!(tokens[1].span.start, 5);
    let end = source_map.resolve_offset(tokens[1].span.end).unwrap();
    assert_eq!(end.line(), 0);
    assert_eq!(end.col_utf8(), 10);
    assert_eq!(tokens[1].span.end, 10);
}

/// Verifies that consecutive names of increasing length accumulate
/// positions correctly through repeated byte-scan cycles.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn consecutive_names_accumulate_positions() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("a bb ccc dddd")
            .collect_with_source_map();
    // Tokens: a, bb, ccc, dddd, Eof
    assert_eq!(tokens.len(), 5);

    // "a" at (0,0)-(0,1)
    let pos = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(pos.col_utf8(), 0);
    let pos = source_map.resolve_offset(tokens[0].span.end).unwrap();
    assert_eq!(pos.col_utf8(), 1);

    // "bb" at (0,2)-(0,4)
    let pos = source_map.resolve_offset(tokens[1].span.start).unwrap();
    assert_eq!(pos.col_utf8(), 2);
    let pos = source_map.resolve_offset(tokens[1].span.end).unwrap();
    assert_eq!(pos.col_utf8(), 4);

    // "ccc" at (0,5)-(0,8)
    let pos = source_map.resolve_offset(tokens[2].span.start).unwrap();
    assert_eq!(pos.col_utf8(), 5);
    let pos = source_map.resolve_offset(tokens[2].span.end).unwrap();
    assert_eq!(pos.col_utf8(), 8);

    // "dddd" at (0,9)-(0,13)
    let pos = source_map.resolve_offset(tokens[3].span.start).unwrap();
    assert_eq!(pos.col_utf8(), 9);
    let pos = source_map.resolve_offset(tokens[3].span.end).unwrap();
    assert_eq!(pos.col_utf8(), 13);
}

// =============================================================================
// Names after newlines
// =============================================================================

/// Verifies that a name after a newline starts at (line+1, col=0).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn name_after_newline_position() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("name\nsecondName")
            .collect_with_source_map();
    // Tokens: name, secondName, Eof
    assert_eq!(tokens.len(), 3);

    // "name" at (0,0)-(0,4)
    let pos = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(pos.line(), 0);
    assert_eq!(pos.col_utf8(), 0);
    let pos = source_map.resolve_offset(tokens[0].span.end).unwrap();
    assert_eq!(pos.col_utf8(), 4);

    // "secondName" at (1,0)-(1,10)
    let pos = source_map.resolve_offset(tokens[1].span.start).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 0);
    let pos = source_map.resolve_offset(tokens[1].span.end).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 10);
}

// =============================================================================
// Boundary conditions
// =============================================================================

/// Verifies that a name at EOF (no trailing whitespace or content)
/// is scanned correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn name_at_eof() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("name")
            .collect_with_source_map();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    assert!(
        matches!(
            tokens[0].kind,
            GraphQLTokenKind::Name(ref n) if n == "name"
        ),
    );
    let pos = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(pos.col_utf8(), 0);
    let pos = source_map.resolve_offset(tokens[0].span.end).unwrap();
    assert_eq!(pos.col_utf8(), 4);
    assert_eq!(tokens[0].span.end, 4);
}

/// Verifies that a single-character name produces the correct
/// 1-byte span.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn single_char_name() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("x")
            .collect_with_source_map();
    // Tokens: x, Eof
    assert_eq!(tokens.len(), 2);

    assert!(
        matches!(
            tokens[0].kind,
            GraphQLTokenKind::Name(ref n) if n == "x"
        ),
    );
    let pos = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(pos.col_utf8(), 0);
    assert_eq!(tokens[0].span.start, 0);
    let pos = source_map.resolve_offset(tokens[0].span.end).unwrap();
    assert_eq!(pos.col_utf8(), 1);
    assert_eq!(tokens[0].span.end, 1);
}

/// Verifies that underscore-prefixed names (`__typename`) are
/// scanned correctly by the byte loop.
///
/// Per GraphQL spec, names match `[_A-Za-z][_0-9A-Za-z]*`:
/// <https://spec.graphql.org/September2025/#sec-Names>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn underscore_prefix_name() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("__typename")
            .collect_with_source_map();
    // Tokens: __typename, Eof
    assert_eq!(tokens.len(), 2);

    assert!(
        matches!(
            tokens[0].kind,
            GraphQLTokenKind::Name(ref n) if n == "__typename"
        ),
    );
    let pos = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(pos.col_utf8(), 0);
    let pos = source_map.resolve_offset(tokens[0].span.end).unwrap();
    assert_eq!(pos.col_utf8(), 10);
    assert_eq!(tokens[0].span.end, 10);
}

// =============================================================================
// Keyword recognition through byte scanning
// =============================================================================

/// Verifies that keywords (`true`, `false`, `null`) are still
/// recognized correctly after byte-scanning the name.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn keywords_recognized_after_byte_scan() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("true false null name")
            .collect_with_source_map();
    // Tokens: true, false, null, name, Eof
    assert_eq!(tokens.len(), 5);

    assert!(matches!(tokens[0].kind, GraphQLTokenKind::True));
    assert!(matches!(tokens[1].kind, GraphQLTokenKind::False));
    assert!(matches!(tokens[2].kind, GraphQLTokenKind::Null));
    assert!(
        matches!(
            tokens[3].kind,
            GraphQLTokenKind::Name(ref n) if n == "name"
        ),
    );

    // Verify positions accumulate correctly across keywords
    // "true" at col 0-4
    let pos = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(pos.col_utf8(), 0);
    let pos = source_map.resolve_offset(tokens[0].span.end).unwrap();
    assert_eq!(pos.col_utf8(), 4);

    // "false" at col 5-10
    let pos = source_map.resolve_offset(tokens[1].span.start).unwrap();
    assert_eq!(pos.col_utf8(), 5);
    let pos = source_map.resolve_offset(tokens[1].span.end).unwrap();
    assert_eq!(pos.col_utf8(), 10);

    // "null" at col 11-15
    let pos = source_map.resolve_offset(tokens[2].span.start).unwrap();
    assert_eq!(pos.col_utf8(), 11);
    let pos = source_map.resolve_offset(tokens[2].span.end).unwrap();
    assert_eq!(pos.col_utf8(), 15);

    // "name" at col 16-20
    let pos = source_map.resolve_offset(tokens[3].span.start).unwrap();
    assert_eq!(pos.col_utf8(), 16);
    let pos = source_map.resolve_offset(tokens[3].span.end).unwrap();
    assert_eq!(pos.col_utf8(), 20);
}
