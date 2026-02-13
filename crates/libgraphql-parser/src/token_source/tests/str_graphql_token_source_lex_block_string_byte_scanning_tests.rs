//! Tests for the byte-scanning optimization in
//! `StrGraphQLTokenSource::lex_block_string()`.
//!
//! ## Optimization summary
//!
//! `lex_block_string()` was rewritten to byte-scan for sentinel
//! bytes (`"`, `\`, `\n`, `\r`) instead of per-character
//! `peek_char()` / `consume()`. Non-sentinel bytes are skipped
//! with a single `i += 1`. Position state is batch-updated once
//! after the scan completes.
//!
//! This is safe for multi-byte UTF-8 content because all sentinel
//! bytes are ASCII (<0x80) and cannot appear as continuation bytes
//! in multi-byte UTF-8 sequences (which are always >=0x80).
//!
//! ## What these tests verify
//!
//! - Position of the next token after single-line and multi-line
//!   block strings
//! - CRLF and CR-only newlines within block strings
//! - Escaped triple quotes (`\"""`) don't close the string
//! - Unicode content near sentinel bytes
//! - Large block string content
//!
//! Written by Claude Code, reviewed by a human.

use crate::token::GraphQLTokenKind;
use crate::token_source::StrGraphQLTokenSource;

// =============================================================================
// Position tracking after block strings
// =============================================================================

/// Verifies that the token after a single-line block string has
/// the correct position.
///
/// A single-line block string has no newlines, so the column
/// advances through the entire `"""..."""` span via
/// `compute_columns_for_span()`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn single_line_block_string_positions() {
    // """hello""" name
    // 0123456789012345
    // """hello""" = 12 bytes (3+5+3+1 space)
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("\"\"\"hello\"\"\" name")
            .collect();
    // Tokens: StringValue, Name, Eof
    assert_eq!(tokens.len(), 3);

    // Block string: """hello""" = 3+5+3 = 11 bytes
    assert_eq!(tokens[0].span.start_inclusive.line(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(
        tokens[0].span.end_exclusive.byte_offset(),
        11, // 3(""") + 5(hello) + 3(""")
    );

    // "name" starts at col 12 (11 + 1 space)
    assert!(
        matches!(
            tokens[1].kind,
            GraphQLTokenKind::Name(ref n) if n == "name"
        ),
    );
    assert_eq!(tokens[1].span.start_inclusive.line(), 0);
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 12);
}

/// Verifies that the token after a multi-line block string has
/// the correct line and column.
///
/// Newlines within the block string are counted via the byte
/// scanner. After the last newline, columns are reset and computed
/// from `nl_pos + 1..i` via `compute_columns_for_span()`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn multiline_block_string_positions() {
    // """line1\nline2\nline3""" name
    // After 2 newlines in block string, closing """ is on line 2.
    // "name" should be on line 2 with correct column.
    let input = "\"\"\"line1\nline2\nline3\"\"\" name";
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new(input).collect();
    // Tokens: StringValue, Name, Eof
    assert_eq!(tokens.len(), 3);

    // "name" on line 2 (2 newlines in block string)
    assert_eq!(tokens[1].span.start_inclusive.line(), 2);
    // After last \n: "line3" (5) + """ (3) + " " (1) = col 9
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 9);
}

// =============================================================================
// Newline handling within block strings
// =============================================================================

/// Verifies that CRLF within a block string is counted as one
/// newline.
///
/// The byte scanner tracks `last_was_cr` to suppress the LF after
/// CR, matching the behavior of `skip_whitespace()` and
/// `consume()`.
///
/// Per GraphQL spec, CRLF is a valid line terminator:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Line-Terminators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn crlf_in_block_string_positions() {
    // """line1\r\nline2""" name
    // CRLF = 1 newline, so name is on line 1
    let input = "\"\"\"line1\r\nline2\"\"\" name";
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new(input).collect();
    // Tokens: StringValue, Name, Eof
    assert_eq!(tokens.len(), 3);

    assert_eq!(tokens[1].span.start_inclusive.line(), 1);
    // After \r\n: "line2" (5) + """ (3) + " " (1) = col 9
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 9);
}

/// Verifies that a bare CR within a block string is counted as
/// one newline.
///
/// Per GraphQL spec, CR (U+000D) is a valid line terminator:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Line-Terminators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn cr_only_in_block_string_positions() {
    // """line1\rline2""" name
    // CR = 1 newline, so name is on line 1
    let input = "\"\"\"line1\rline2\"\"\" name";
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new(input).collect();
    // Tokens: StringValue, Name, Eof
    assert_eq!(tokens.len(), 3);

    assert_eq!(tokens[1].span.start_inclusive.line(), 1);
    // After \r: "line2" (5) + """ (3) + " " (1) = col 9
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 9);
}

// =============================================================================
// Escaped triple quotes
// =============================================================================

/// Verifies that `\"""` inside a block string doesn't close the
/// string, and the token after the block string has the correct
/// position.
///
/// The byte scanner matches `\"""` as a 4-byte escape sequence
/// (`i += 4`) and continues scanning. The actual closing `"""`
/// follows later.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn escaped_triple_quote_positions() {
    // """has \""" end""" name
    let input = [
        "\"\"\"",       // opening """
        "has ",          // content
        "\\\"\"\"",      // \""" (escaped)
        " end",          // more content
        "\"\"\"",        // closing """
        " name",         // trailing token
    ].concat();
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new(&input).collect();
    // Tokens: StringValue, Name, Eof
    assert_eq!(tokens.len(), 3);

    assert!(
        matches!(
            tokens[1].kind,
            GraphQLTokenKind::Name(ref n) if n == "name"
        ),
    );
    // No newlines, so name is on line 0
    assert_eq!(tokens[1].span.start_inclusive.line(), 0);
}

/// Verifies that `\"""` followed closely by closing `"""` doesn't
/// cause the byte scanner to overshoot.
///
/// The 4-byte skip (`i += 4`) for `\"""` must not mistake the
/// subsequent closing `"""` for part of the escape.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn escaped_triple_quote_near_closing() {
    // """\""""""  (escape + close, back-to-back)
    let input = [
        "\"\"\"",       // opening """
        "\\\"\"\"",      // \""" (escaped)
        "\"\"\"",        // closing """
        " name",         // trailing token
    ].concat();
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new(&input).collect();
    // Tokens: StringValue, Name, Eof
    assert_eq!(tokens.len(), 3);

    assert!(
        matches!(
            tokens[1].kind,
            GraphQLTokenKind::Name(ref n) if n == "name"
        ),
    );
}

// =============================================================================
// Unicode safety in byte scanning
// =============================================================================

/// Verifies that multi-byte UTF-8 characters adjacent to sentinel
/// bytes (`"` and `\`) don't confuse the byte scanner.
///
/// Since all sentinels are ASCII (<0x80) and UTF-8 continuation
/// bytes are >=0x80, they can never be confused. This test serves
/// as a regression guard.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unicode_near_sentinel_bytes() {
    // Block string with Unicode adjacent to "
    // café contains é (0xC3 0xA9) — bytes >= 0x80
    let input = "\"\"\"caf\u{00E9}\"\"\" name";
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new(input).collect();
    // Tokens: StringValue, Name, Eof
    assert_eq!(tokens.len(), 3);

    assert!(
        matches!(
            tokens[1].kind,
            GraphQLTokenKind::Name(ref n) if n == "name"
        ),
    );
    assert_eq!(tokens[1].span.start_inclusive.line(), 0);
}

/// Verifies that columns are computed correctly after a block
/// string containing Unicode on the last line.
///
/// `compute_columns_for_span()` handles non-ASCII content after
/// the last newline, counting UTF-8 characters (not bytes) for
/// `col_utf8`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unicode_after_newline_columns() {
    // """first\nsecond café""" name
    // After \n: "second café" = 11 UTF-8 chars (12 bytes)
    //   + """ = 3 chars + " " = 1 char
    // col_utf8 = 15
    let input =
        "\"\"\"first\nsecond caf\u{00E9}\"\"\" name";
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new(input).collect();
    // Tokens: StringValue, Name, Eof
    assert_eq!(tokens.len(), 3);

    assert_eq!(tokens[1].span.start_inclusive.line(), 1);
    // "second café" = 11 chars, """ = 3 chars, " " = 1 char
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 15);
}

// =============================================================================
// Large block string
// =============================================================================

/// Verifies that the byte scanner handles large block string
/// content correctly.
///
/// This stress-tests the byte scan loop with >10,000 characters
/// to ensure no issues with large offsets.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn large_block_string_content() {
    let content = "x".repeat(10_000);
    let input = format!("\"\"\"{content}\"\"\" name");
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new(&input).collect();
    // Tokens: StringValue, Name, Eof
    assert_eq!(tokens.len(), 3);

    assert!(
        matches!(
            tokens[1].kind,
            GraphQLTokenKind::Name(ref n) if n == "name"
        ),
    );
    // No newlines, so all on line 0
    assert_eq!(tokens[1].span.start_inclusive.line(), 0);
    // """ (3) + 10000 + """ (3) + " " (1) = 10007
    assert_eq!(
        tokens[1].span.start_inclusive.col_utf8(),
        10_007,
    );
}
