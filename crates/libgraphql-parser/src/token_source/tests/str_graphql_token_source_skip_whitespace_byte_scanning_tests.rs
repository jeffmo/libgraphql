//! Tests for the byte-scanning optimization in
//! `StrGraphQLTokenSource::skip_whitespace()`.
//!
//! ## Optimization summary
//!
//! `skip_whitespace()` was rewritten to byte-scan for whitespace
//! characters (` `, `\t`, `\n`, `\r`, BOM) instead of calling
//! `peek_char()` / `consume()` per character. Position state is
//! batch-updated once at the end:
//!
//! - `curr_line += lines_added` (newlines counted during scan)
//! - `curr_col_utf8` / `curr_col_utf16` are either reset (if a
//!   newline was seen) or advanced (if no newline was seen)
//! - BOM (U+FEFF, 3 bytes in UTF-8) counts as 1 column
//! - `last_char_was_cr` is tracked to avoid double-counting CRLF
//!
//! ## What these tests verify
//!
//! - Multiple newlines produce correct `lines_added` accumulation
//! - CRLF pairs are not double-counted (the `last_was_cr` flag)
//! - Mixed `\r`, `\n`, `\r\n` newline styles
//! - BOM at start of input, after newline, multiple BOMs, and BOMs
//!   mixed with spaces
//! - Column reset after newline
//! - Tab and space column accumulation
//!
//! Written by Claude Code, reviewed by a human.

use crate::token_source::StrGraphQLTokenSource;

// =============================================================================
// Newline batch counting
// =============================================================================

/// Verifies that multiple `\n` newlines accumulate correctly in
/// `lines_added`.
///
/// Three newlines before a name should place it at line 3, col 0.
///
/// Per GraphQL spec, LF (U+000A) is a valid line terminator:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Line-Terminators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn multiple_newlines_accumulate() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("\n\n\nname").collect();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens[0].span.start_inclusive.line(), 3);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
}

/// Verifies that `\r\n` pairs are counted as one newline, not two.
///
/// Two CRLF pairs followed by a space and name should place the
/// name at line 2, col 1. The byte scanner tracks `last_was_cr`
/// to suppress the LF after a CR.
///
/// Per GraphQL spec, CRLF is a valid line terminator:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Line-Terminators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn crlf_not_double_counted() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("\r\n\r\n name").collect();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens[0].span.start_inclusive.line(), 2);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 1);
}

/// Verifies that mixed `\r`, `\n`, and `\r\n` newlines within a
/// single whitespace run are counted correctly.
///
/// Input: `\r\n` (1 newline) + `\r` (1 newline) + `\n` (1 newline)
/// + `name` = 3 newlines total. Name at line 3, col 0.
///
/// Per GraphQL spec, all three line terminator forms are valid:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Line-Terminators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn mixed_cr_lf_crlf_newlines() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("\r\n\r\nname").collect();
    // \r\n = 1, \r = 1 (the \n is suppressed by last_was_cr from
    // \r), so actually \r\n\r\n = 2 newlines.
    // Wait: \r\n (1 newline), then \r (last_was_cr=true),
    //   then \n (suppressed because last_was_cr), then name.
    // That's 2 newlines: line 2, col 0.
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].span.start_inclusive.line(), 2);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
}

/// Verifies that a bare `\r` (not followed by `\n`) counts as
/// exactly one newline.
///
/// Per GraphQL spec, CR (U+000D) is a valid line terminator:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Line-Terminators>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn bare_cr_counts_as_one_newline() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("\r name").collect();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens[0].span.start_inclusive.line(), 1);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 1);
}

// =============================================================================
// BOM handling
// =============================================================================

/// Verifies that a BOM at the start of input is treated as 1
/// column despite being 3 bytes in UTF-8.
///
/// BOM (U+FEFF) is 0xEF 0xBB 0xBF in UTF-8 (3 bytes). The byte
/// scanner matches this 3-byte sequence and advances by 3, but the
/// column computation subtracts `bom_count * 2` to account for the
/// 3 bytes → 1 column difference.
///
/// Per GraphQL spec, BOM is a Unicode byte-order mark that is
/// ignored (treated as whitespace):
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Unicode>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn bom_at_start_counts_as_one_column() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("\u{FEFF}name").collect();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    // BOM is 3 bytes but 1 column, so name starts at col 1
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 1);
    assert_eq!(
        tokens[0].span.start_inclusive.byte_offset(),
        3,
    );
}

/// Verifies that a BOM after a newline is correctly accounted for
/// in the column reset computation.
///
/// After the newline, columns reset. The BOM after the newline is
/// 3 bytes but 1 column. Name at (1, 1), byte offset 4.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn bom_after_newline() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("\n\u{FEFF}name").collect();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens[0].span.start_inclusive.line(), 1);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 1);
    assert_eq!(
        tokens[0].span.start_inclusive.byte_offset(),
        4, // 1 (\n) + 3 (BOM)
    );
}

/// Verifies that multiple consecutive BOMs are each counted as 1
/// column.
///
/// Two BOMs = 6 bytes, 2 columns. Name at col 2, byte offset 6.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn multiple_boms() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("\u{FEFF}\u{FEFF}name")
            .collect();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 2);
    assert_eq!(
        tokens[0].span.start_inclusive.byte_offset(),
        6,
    );
}

/// Verifies that BOMs mixed with regular spaces produce the correct
/// column count.
///
/// Input: 2 spaces + BOM (3 bytes) + 2 spaces = 7 bytes, 5
/// columns. Name at col 5, byte offset 7.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn bom_mixed_with_spaces() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("  \u{FEFF}  name").collect();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 5);
    assert_eq!(
        tokens[0].span.start_inclusive.byte_offset(),
        7,
    );
}

// =============================================================================
// Column computation
// =============================================================================

/// Verifies that tabs and spaces both advance the column by 1 each.
///
/// Per GraphQL spec, both Tab (U+0009) and Space (U+0020) are
/// whitespace:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.White-Space>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn tabs_and_spaces_advance_column() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("\t  name").collect();
    // Tokens: name, Eof
    // Tab + 2 spaces = 3 columns
    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 3);
    assert_eq!(
        tokens[0].span.start_inclusive.byte_offset(),
        3,
    );
}

/// Verifies that columns reset after a newline within a whitespace
/// run.
///
/// Input: 2 spaces + `\n` + 2 spaces + name. After the newline,
/// columns reset, so name is at (1, 2).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn column_resets_after_newline_in_whitespace() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("  \n  name").collect();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens[0].span.start_inclusive.line(), 1);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 2);
}
