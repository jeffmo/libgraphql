//! Tests for the ASCII fast path optimization in
//! `StrGraphQLTokenSource::peek_char()` and
//! `StrGraphQLTokenSource::consume()`.
//!
//! ## Optimization summary
//!
//! `peek_char()` uses direct byte indexing with an `is_ascii()`
//! check: when the current byte is < 0x80, it is cast directly to
//! `char` via `b as char` instead of constructing a full `Chars`
//! iterator. `consume()` advances `curr_byte_offset` by the
//! character's UTF-8 byte length. Line/column resolution is
//! deferred to SourceMap.
//!
//! ## What these tests verify
//!
//! - Pure-ASCII input produces correct byte spans
//! - Mixed `\n` / `\r` / `\r\n` newlines produce correct line
//!   positions via SourceMap
//! - Non-ASCII characters (BMP and supplementary plane) produce
//!   correct byte offsets and SourceMap-resolved columns
//! - Boundary conditions: empty input, single-char input, trailing CR
//!
//! Written by Claude Code, reviewed by a human.

use crate::token::GraphQLTokenKind;
use crate::token::GraphQLTokenSource;
use crate::token::StrGraphQLTokenSource;

// =============================================================================
// ASCII fast path validation
// =============================================================================

/// Verifies that pure-ASCII input produces correct token spans when
/// using the `b as char` fast path in `peek_char()`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn ascii_fast_path_token_spans() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("type Query { field: String }")
            .collect_with_source_map();

    assert_eq!(tokens.len(), 8);

    // "type" at (0,0)-(0,4)
    let s = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(s.line(), 0);
    assert_eq!(s.col_utf8(), 0);
    let e = source_map.resolve_offset(tokens[0].span.end).unwrap();
    assert_eq!(e.line(), 0);
    assert_eq!(e.col_utf8(), 4);

    // "Query" at (0,5)-(0,10)
    let s = source_map.resolve_offset(tokens[1].span.start).unwrap();
    assert_eq!(s.col_utf8(), 5);
    let e = source_map.resolve_offset(tokens[1].span.end).unwrap();
    assert_eq!(e.col_utf8(), 10);

    // "{" at (0,11)-(0,12)
    let s = source_map.resolve_offset(tokens[2].span.start).unwrap();
    assert_eq!(s.col_utf8(), 11);
    let e = source_map.resolve_offset(tokens[2].span.end).unwrap();
    assert_eq!(e.col_utf8(), 12);

    // "field" at (0,13)-(0,18)
    let s = source_map.resolve_offset(tokens[3].span.start).unwrap();
    assert_eq!(s.col_utf8(), 13);
    let e = source_map.resolve_offset(tokens[3].span.end).unwrap();
    assert_eq!(e.col_utf8(), 18);

    // ":" at (0,18)-(0,19)
    let s = source_map.resolve_offset(tokens[4].span.start).unwrap();
    assert_eq!(s.col_utf8(), 18);
    let e = source_map.resolve_offset(tokens[4].span.end).unwrap();
    assert_eq!(e.col_utf8(), 19);

    // "String" at (0,20)-(0,26)
    let s = source_map.resolve_offset(tokens[5].span.start).unwrap();
    assert_eq!(s.col_utf8(), 20);
    let e = source_map.resolve_offset(tokens[5].span.end).unwrap();
    assert_eq!(e.col_utf8(), 26);

    // "}" at (0,27)-(0,28)
    let s = source_map.resolve_offset(tokens[6].span.start).unwrap();
    assert_eq!(s.col_utf8(), 27);
    let e = source_map.resolve_offset(tokens[6].span.end).unwrap();
    assert_eq!(e.col_utf8(), 28);

    assert!(matches!(tokens[7].kind, GraphQLTokenKind::Eof));
}

/// Verifies that the ASCII branch correctly tracks line positions
/// across mixed `\n`, `\r`, and `\r\n` newlines.
///
/// Per GraphQL spec, all three newline forms are valid line
/// terminators:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Line-Terminators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn ascii_newline_tracking_mixed_styles() {
    // \n then \r then \r\n — 3 distinct newline styles
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("a\nb\rc\r\nd")
            .collect_with_source_map();
    // Tokens: a, b, c, d, Eof
    assert_eq!(tokens.len(), 5);

    // "a" at line 0
    let pos = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(pos.line(), 0);
    assert_eq!(pos.col_utf8(), 0);

    // "b" at line 1 (after \n)
    let pos = source_map.resolve_offset(tokens[1].span.start).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 0);

    // "c" at line 2 (after \r)
    let pos = source_map.resolve_offset(tokens[2].span.start).unwrap();
    assert_eq!(pos.line(), 2);
    assert_eq!(pos.col_utf8(), 0);

    // "d" at line 3 (after \r\n — counted as ONE newline)
    let pos = source_map.resolve_offset(tokens[3].span.start).unwrap();
    assert_eq!(pos.line(), 3);
    assert_eq!(pos.col_utf8(), 0);
}

/// Verifies that consecutive `\r\n` pairs each count as exactly
/// one newline, not two.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn consecutive_crlf_pairs_counted_correctly() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("a\r\nb\r\nc\r\n")
            .collect_with_source_map();
    // Tokens: a, b, c, Eof
    assert_eq!(tokens.len(), 4);

    let pos = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(pos.line(), 0);
    let pos = source_map.resolve_offset(tokens[1].span.start).unwrap();
    assert_eq!(pos.line(), 1);
    let pos = source_map.resolve_offset(tokens[2].span.start).unwrap();
    assert_eq!(pos.line(), 2);
    // Eof at line 3 (three \r\n = 3 newlines total)
    let pos = source_map.resolve_offset(tokens[3].span.start).unwrap();
    assert_eq!(pos.line(), 3);
}

// =============================================================================
// Non-ASCII fallback validation
// =============================================================================

/// Verifies that non-ASCII characters produce correct byte offsets
/// and SourceMap-resolved columns.
///
/// `é` (U+00E9) is 2 UTF-8 bytes but 1 UTF-16 unit (BMP).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn non_ascii_string_position_tracking() {
    // String "café" followed by a name
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("\"caf\u{00E9}\" name")
            .collect_with_source_map();
    // Tokens: StringValue, Name, Eof
    assert_eq!(tokens.len(), 3);

    // String "café":
    //   " (1 byte) + c (1) + a (1) + f (1) + é (2) + " (1) = 7
    //   UTF-8 cols: 6 (one per character)
    //   UTF-16 units: 6 (é is BMP, so same as UTF-8 char count)
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 7);
    let end = source_map.resolve_offset(tokens[0].span.end).unwrap();
    assert_eq!(end.col_utf8(), 6);
    assert_eq!(end.col_utf16(), Some(6));

    // "name": starts at byte 8 (7 + 1 space), col 7
    assert_eq!(tokens[1].span.start, 8);
    let start = source_map.resolve_offset(tokens[1].span.start).unwrap();
    assert_eq!(start.col_utf8(), 7);
    assert_eq!(start.col_utf16(), Some(7));
}

/// Verifies that emoji (supplementary plane characters) are tracked
/// correctly in UTF-16 column counting.
///
/// `🎉` (U+1F389) is 4 UTF-8 bytes, 1 UTF-8 character, and 2
/// UTF-16 code units (surrogate pair).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn emoji_utf16_column_tracking() {
    // "🎉" as a string value, then a name
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("\"🎉\" x")
            .collect_with_source_map();
    // Tokens: StringValue, Name, Eof
    assert_eq!(tokens.len(), 3);

    // String "🎉":
    //   " (1 byte, 1 utf8 col, 1 utf16 unit)
    //   🎉 (4 bytes, 1 utf8 col, 2 utf16 units)
    //   " (1 byte, 1 utf8 col, 1 utf16 unit)
    //   Total: 6 bytes, 3 utf8 cols, 4 utf16 units
    assert_eq!(tokens[0].span.end, 6);
    let end = source_map.resolve_offset(tokens[0].span.end).unwrap();
    assert_eq!(end.col_utf8(), 3);
    assert_eq!(end.col_utf16(), Some(4));

    // "x": after space
    //   byte offset = 7, utf8 col = 4, utf16 col = 5
    assert_eq!(tokens[1].span.start, 7);
    let start = source_map.resolve_offset(tokens[1].span.start).unwrap();
    assert_eq!(start.col_utf8(), 4);
    assert_eq!(start.col_utf16(), Some(5));
}

/// Verifies that positions accumulate correctly when alternating
/// between ASCII tokens and non-ASCII string content.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn mixed_ascii_and_multibyte_positions() {
    // "α" (2-byte BMP) then name, then "β" (2-byte BMP) then name
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("\"α\" name \"β\" other")
            .collect_with_source_map();
    // Tokens: "α", name, "β", other, Eof
    assert_eq!(tokens.len(), 5);

    // "α": " (1b) α (2b) " (1b) = 4 bytes, 3 utf8 cols
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 4);
    let end = source_map.resolve_offset(tokens[0].span.end).unwrap();
    assert_eq!(end.col_utf8(), 3);

    // "name": after space, byte 5, col 4
    assert_eq!(tokens[1].span.start, 5);
    let start = source_map.resolve_offset(tokens[1].span.start).unwrap();
    assert_eq!(start.col_utf8(), 4);
    assert_eq!(tokens[1].span.end, 9);
    let end = source_map.resolve_offset(tokens[1].span.end).unwrap();
    assert_eq!(end.col_utf8(), 8);

    // "β": after space, byte 10
    //   " (1b) β (2b) " (1b) = 4 bytes, 3 utf8 cols
    assert_eq!(tokens[2].span.start, 10);
    let start = source_map.resolve_offset(tokens[2].span.start).unwrap();
    assert_eq!(start.col_utf8(), 9);
    assert_eq!(tokens[2].span.end, 14);
    let end = source_map.resolve_offset(tokens[2].span.end).unwrap();
    assert_eq!(end.col_utf8(), 12);

    // "other": after space, byte 15, col 13
    assert_eq!(tokens[3].span.start, 15);
    let start = source_map.resolve_offset(tokens[3].span.start).unwrap();
    assert_eq!(start.col_utf8(), 13);
}

// =============================================================================
// Boundary conditions
// =============================================================================

/// Verifies that empty input produces EOF at position (0,0,0).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn empty_input_positions() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("")
            .collect_with_source_map();
    assert_eq!(tokens.len(), 1); // Just Eof
    assert!(matches!(tokens[0].kind, GraphQLTokenKind::Eof));

    let pos = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(pos.line(), 0);
    assert_eq!(pos.col_utf8(), 0);
    assert_eq!(pos.col_utf16(), Some(0));
    assert_eq!(tokens[0].span.start, 0);
}

/// Verifies that a single-character input produces the correct span.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn single_char_input() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("{")
            .collect_with_source_map();
    assert_eq!(tokens.len(), 2); // CurlyBraceOpen, Eof

    assert_eq!(tokens[0].span.start, 0);
    let start = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(start.col_utf8(), 0);
    assert_eq!(tokens[0].span.end, 1);
    let end = source_map.resolve_offset(tokens[0].span.end).unwrap();
    assert_eq!(end.col_utf8(), 1);
}

/// Verifies that a trailing CR at EOF correctly increments the line.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn cr_at_eof() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("name\r")
            .collect_with_source_map();
    assert_eq!(tokens.len(), 2); // Name, Eof

    // "name" at line 0
    let pos = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(pos.line(), 0);

    // Eof at line 1 (CR incremented the line)
    let pos = source_map.resolve_offset(tokens[1].span.start).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 0);
}
