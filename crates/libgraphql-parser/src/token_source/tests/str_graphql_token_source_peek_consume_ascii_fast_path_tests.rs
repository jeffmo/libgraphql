//! Tests for the ASCII fast path optimization in
//! `StrGraphQLTokenSource::peek_char()` and
//! `StrGraphQLTokenSource::consume()`.
//!
//! ## Optimization summary
//!
//! `peek_char()` was changed to use direct byte indexing with an
//! `is_ascii()` check: when the current byte is < 0x80, it is cast
//! directly to `char` via `b as char` instead of constructing a full
//! `Chars` iterator. `consume()` was similarly split into ASCII and
//! non-ASCII branches — the ASCII branch knows the character is
//! exactly 1 byte, 1 UTF-8 column, and 1 UTF-16 code unit, so it
//! skips `len_utf8()` / `len_utf16()` calls.
//!
//! ## What these tests verify
//!
//! - Pure-ASCII input produces the same token spans as before
//! - Mixed `\n` / `\r` / `\r\n` newlines are counted correctly in
//!   the ASCII branch (the `last_char_was_cr` flag)
//! - Non-ASCII characters (BMP and supplementary plane) fall through
//!   to the slow path and still produce correct byte offsets, UTF-8
//!   columns, and UTF-16 columns
//! - Boundary conditions: empty input, single-char input, trailing CR
//!
//! Written by Claude Code, reviewed by a human.

use crate::token::GraphQLTokenKind;
use crate::token_source::StrGraphQLTokenSource;

// =============================================================================
// ASCII fast path validation
// =============================================================================

/// Verifies that pure-ASCII input produces correct token spans when
/// using the `b as char` fast path in `peek_char()`.
///
/// This validates that the ASCII fast path returns the correct
/// character for all single-byte ASCII values, and that token spans
/// are computed correctly when the entire input is ASCII.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn ascii_fast_path_token_spans() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("type Query { field: String }")
            .collect();

    // "type" at (0,0)-(0,4)
    assert_eq!(tokens[0].span.start_inclusive.line(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[0].span.end_exclusive.line(), 0);
    assert_eq!(tokens[0].span.end_exclusive.col_utf8(), 4);

    // "Query" at (0,5)-(0,10)
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 5);
    assert_eq!(tokens[1].span.end_exclusive.col_utf8(), 10);

    // "{" at (0,11)-(0,12)
    assert_eq!(tokens[2].span.start_inclusive.col_utf8(), 11);
    assert_eq!(tokens[2].span.end_exclusive.col_utf8(), 12);

    // "field" at (0,13)-(0,18)
    assert_eq!(tokens[3].span.start_inclusive.col_utf8(), 13);
    assert_eq!(tokens[3].span.end_exclusive.col_utf8(), 18);

    // ":" at (0,18)-(0,19)
    assert_eq!(tokens[4].span.start_inclusive.col_utf8(), 18);
    assert_eq!(tokens[4].span.end_exclusive.col_utf8(), 19);

    // "String" at (0,20)-(0,26)
    assert_eq!(tokens[5].span.start_inclusive.col_utf8(), 20);
    assert_eq!(tokens[5].span.end_exclusive.col_utf8(), 26);

    // "}" at (0,27)-(0,28)
    assert_eq!(tokens[6].span.start_inclusive.col_utf8(), 27);
    assert_eq!(tokens[6].span.end_exclusive.col_utf8(), 28);
}

/// Verifies that the ASCII branch of `consume()` correctly tracks
/// line and column across mixed `\n`, `\r`, and `\r\n` newlines.
///
/// The ASCII branch handles newlines: `\n` increments the line
/// unless `last_char_was_cr` is set (to avoid double-counting
/// `\r\n`); `\r` always increments the line and sets the flag.
///
/// Per GraphQL spec, all three newline forms are valid line
/// terminators:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Line-Terminators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn ascii_newline_tracking_mixed_styles() {
    // \n then \r then \r\n — 3 distinct newline styles
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("a\nb\rc\r\nd").collect();
    // Tokens: a, b, c, d, Eof
    assert_eq!(tokens.len(), 5);

    // "a" at line 0
    assert_eq!(tokens[0].span.start_inclusive.line(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);

    // "b" at line 1 (after \n)
    assert_eq!(tokens[1].span.start_inclusive.line(), 1);
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 0);

    // "c" at line 2 (after \r)
    assert_eq!(tokens[2].span.start_inclusive.line(), 2);
    assert_eq!(tokens[2].span.start_inclusive.col_utf8(), 0);

    // "d" at line 3 (after \r\n — counted as ONE newline)
    assert_eq!(tokens[3].span.start_inclusive.line(), 3);
    assert_eq!(tokens[3].span.start_inclusive.col_utf8(), 0);
}

/// Verifies that consecutive `\r\n` pairs each count as exactly
/// one newline, not two.
///
/// This stresses the `last_char_was_cr` flag in `consume()`: when
/// we see `\r` we increment the line and set the flag; when we then
/// see `\n` and the flag is set, we skip the increment and clear
/// the flag.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn consecutive_crlf_pairs_counted_correctly() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("a\r\nb\r\nc\r\n").collect();
    // Tokens: a, b, c, Eof
    assert_eq!(tokens.len(), 4);

    assert_eq!(tokens[0].span.start_inclusive.line(), 0);
    assert_eq!(tokens[1].span.start_inclusive.line(), 1);
    assert_eq!(tokens[2].span.start_inclusive.line(), 2);
    // Eof at line 3 (three \r\n = 3 newlines total)
    assert_eq!(tokens[3].span.start_inclusive.line(), 3);
}

// =============================================================================
// Non-ASCII fallback validation
// =============================================================================

/// Verifies that the non-ASCII `consume()` branch correctly tracks
/// byte offsets and UTF-8/UTF-16 columns for multi-byte characters.
///
/// `é` (U+00E9) is 2 UTF-8 bytes but 1 UTF-16 unit (BMP). The
/// non-ASCII branch calls `ch.len_utf8()` and `ch.len_utf16()` for
/// correct position advancement.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn non_ascii_string_position_tracking() {
    // String "café" followed by a name
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("\"caf\u{00E9}\" name")
            .collect();
    // Tokens: StringValue, Name, Eof
    assert_eq!(tokens.len(), 3);

    // String "café":
    //   " (1 byte) + c (1) + a (1) + f (1) + é (2) + " (1) = 7
    //   UTF-8 cols: 6 (one per character)
    //   UTF-16 units: 6 (é is BMP, so same as UTF-8 char count)
    assert_eq!(tokens[0].span.start_inclusive.byte_offset(), 0);
    assert_eq!(tokens[0].span.end_exclusive.byte_offset(), 7);
    assert_eq!(tokens[0].span.end_exclusive.col_utf8(), 6);
    assert_eq!(
        tokens[0].span.end_exclusive.col_utf16(),
        Some(6),
    );

    // "name": starts at byte 8 (7 + 1 space), col 7
    assert_eq!(tokens[1].span.start_inclusive.byte_offset(), 8);
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 7);
    assert_eq!(
        tokens[1].span.start_inclusive.col_utf16(),
        Some(7),
    );
}

/// Verifies that emoji (supplementary plane characters) are tracked
/// correctly in UTF-16 column counting.
///
/// `🎉` (U+1F389) is 4 UTF-8 bytes, 1 UTF-8 character, and 2
/// UTF-16 code units (surrogate pair). The non-ASCII branch of
/// `consume()` calls `ch.len_utf16()` which returns 2 for
/// supplementary characters.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn emoji_utf16_column_tracking() {
    // "🎉" as a string value, then a name
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("\"🎉\" x").collect();
    // Tokens: StringValue, Name, Eof
    assert_eq!(tokens.len(), 3);

    // String "🎉":
    //   " (1 byte, 1 utf8 col, 1 utf16 unit)
    //   🎉 (4 bytes, 1 utf8 col, 2 utf16 units)
    //   " (1 byte, 1 utf8 col, 1 utf16 unit)
    //   Total: 6 bytes, 3 utf8 cols, 4 utf16 units
    assert_eq!(tokens[0].span.end_exclusive.byte_offset(), 6);
    assert_eq!(tokens[0].span.end_exclusive.col_utf8(), 3);
    assert_eq!(
        tokens[0].span.end_exclusive.col_utf16(),
        Some(4),
    );

    // "x": after space
    //   byte offset = 7, utf8 col = 4, utf16 col = 5
    assert_eq!(tokens[1].span.start_inclusive.byte_offset(), 7);
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 4);
    assert_eq!(
        tokens[1].span.start_inclusive.col_utf16(),
        Some(5),
    );
}

/// Verifies that positions accumulate correctly when alternating
/// between ASCII tokens and non-ASCII string content.
///
/// This tests that switching between the ASCII and non-ASCII
/// branches of `consume()` doesn't corrupt the position state
/// across multiple tokens.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn mixed_ascii_and_multibyte_positions() {
    // "α" (2-byte BMP) then name, then "β" (2-byte BMP) then name
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("\"α\" name \"β\" other")
            .collect();
    // Tokens: "α", name, "β", other, Eof
    assert_eq!(tokens.len(), 5);

    // "α": " (1b) α (2b) " (1b) = 4 bytes, 3 utf8 cols
    assert_eq!(tokens[0].span.start_inclusive.byte_offset(), 0);
    assert_eq!(tokens[0].span.end_exclusive.byte_offset(), 4);
    assert_eq!(tokens[0].span.end_exclusive.col_utf8(), 3);

    // "name": after space, byte 5, col 4
    assert_eq!(tokens[1].span.start_inclusive.byte_offset(), 5);
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 4);
    assert_eq!(tokens[1].span.end_exclusive.byte_offset(), 9);
    assert_eq!(tokens[1].span.end_exclusive.col_utf8(), 8);

    // "β": after space, byte 10
    //   " (1b) β (2b) " (1b) = 4 bytes, 3 utf8 cols
    assert_eq!(
        tokens[2].span.start_inclusive.byte_offset(),
        10,
    );
    assert_eq!(tokens[2].span.start_inclusive.col_utf8(), 9);
    assert_eq!(
        tokens[2].span.end_exclusive.byte_offset(),
        14,
    );
    assert_eq!(tokens[2].span.end_exclusive.col_utf8(), 12);

    // "other": after space, byte 15, col 13
    assert_eq!(
        tokens[3].span.start_inclusive.byte_offset(),
        15,
    );
    assert_eq!(tokens[3].span.start_inclusive.col_utf8(), 13);
}

// =============================================================================
// Boundary conditions
// =============================================================================

/// Verifies that empty input produces EOF at position (0,0,0).
///
/// With empty input, `peek_char()` immediately returns None (the
/// byte offset check `>= bytes.len()` catches it). This is a
/// boundary condition for both the fast path check and `consume()`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn empty_input_positions() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("").collect();
    assert_eq!(tokens.len(), 1); // Just Eof
    assert!(matches!(tokens[0].kind, GraphQLTokenKind::Eof));

    assert_eq!(tokens[0].span.start_inclusive.line(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(
        tokens[0].span.start_inclusive.col_utf16(),
        Some(0),
    );
    assert_eq!(
        tokens[0].span.start_inclusive.byte_offset(),
        0,
    );
}

/// Verifies that a single-character input produces the correct span.
///
/// Tests the minimum-length token case: one byte consumed via the
/// ASCII fast path, then EOF.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn single_char_input() {
    let tokens: Vec<_> = StrGraphQLTokenSource::new("{").collect();
    assert_eq!(tokens.len(), 2); // CurlyBraceOpen, Eof

    assert_eq!(
        tokens[0].span.start_inclusive.byte_offset(),
        0,
    );
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[0].span.end_exclusive.byte_offset(), 1);
    assert_eq!(tokens[0].span.end_exclusive.col_utf8(), 1);
}

/// Verifies that a trailing CR at EOF correctly increments the line.
///
/// When `\r` is the last character, `consume()` sets
/// `last_char_was_cr = true` and increments the line. There is no
/// subsequent `\n` to suppress. The EOF token should be on the new
/// line.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn cr_at_eof() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("name\r").collect();
    assert_eq!(tokens.len(), 2); // Name, Eof

    // "name" at line 0
    assert_eq!(tokens[0].span.start_inclusive.line(), 0);

    // Eof at line 1 (CR incremented the line)
    assert_eq!(tokens[1].span.start_inclusive.line(), 1);
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 0);
}
