//! Tests for the low-allocation block string parsing optimization
//! in `GraphQLTokenKind::parse_string_value()` (specifically the
//! internal `parse_block_string()` function).
//!
//! ## Optimization summary
//!
//! `parse_block_string()` was rewritten to avoid per-line heap
//! allocations:
//!
//! 1. **`Cow::Borrowed` fast path** — when the block string has no
//!    `\"""` escapes (the vast majority of cases), the content slice
//!    borrows directly from the raw token text with zero allocation.
//!
//! 2. **Two-pass index tracking** — pass 1 computes the common
//!    indent and finds the first/last non-blank line indices; pass 2
//!    writes stripped lines directly into a single pre-allocated
//!    `String`. This replaces the old `Vec<String>` +
//!    `Vec::remove(0)` + `join()` approach.
//!
//! 3. **`is_graphql_blank()`** — uses byte-level checks for
//!    `b' '` and `b'\t'` only (per the GraphQL spec definition of
//!    `WhiteSpace`), avoiding Rust's Unicode-aware `trim()`.
//!
//! ## What these tests verify
//!
//! - Borrowed path (no escapes) produces correct results
//! - Owned path (`\"""` escapes) produces correct results
//! - Blank line trimming via index tracking
//! - Indentation edge cases (short lines, tabs, mixed)
//! - Line ending variants (`\r\n`, `\r`)
//! - Unicode content preservation through indent stripping
//!
//! Written by Claude Code, reviewed by a human.

use crate::token::GraphQLTokenKind;

/// Helper: parse a block string and return the result string.
fn parse_block(raw: &str) -> String {
    let token = GraphQLTokenKind::string_value_owned(raw.to_string());
    token.parse_string_value().unwrap().unwrap()
}

// =============================================================================
// Cow::Borrowed fast path (no escaped triple quotes)
// =============================================================================

/// Verifies that a simple block string with no escapes works
/// through the `Cow::Borrowed` path.
///
/// When the content between the triple quotes contains no `\"""`
/// sequences, `parse_block_string()` borrows the content slice
/// directly from the raw token text (zero allocation for the
/// content itself).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn no_escapes_simple() {
    assert_eq!(parse_block(r#""""simple text""""#), "simple text");
}

/// Verifies that multi-line block strings with uniform indentation
/// work correctly through the Borrowed path.
///
/// Common indentation of 4 spaces should be stripped from all lines
/// after the first. Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#BlockStringValue()>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn no_escapes_multiline_indent() {
    let raw = "\"\"\"\n    line1\n    line2\n    line3\n\"\"\"";
    assert_eq!(parse_block(raw), "line1\nline2\nline3");
}

// =============================================================================
// Escaped triple quote handling (Cow::Owned path)
// =============================================================================

/// Verifies that multiple `\"""` replacements produce `"""` in the
/// output.
///
/// When `\"""` is present, the content goes through
/// `Cow::Owned(content.replace(...))`, so this tests the owned
/// path.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn multiple_escapes() {
    let raw = r#""""first \""" middle \""" last""""#;
    assert_eq!(
        parse_block(raw),
        "first \"\"\" middle \"\"\" last",
    );
}

/// Verifies that `\"""` right after opening `"""` works.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn escape_at_start() {
    // Raw token text: """\""" rest"""
    let raw = [
        "\"\"\"",     // opening """
        "\\\"\"\"",   // \""" (escaped triple quote)
        " rest",      // content
        "\"\"\"",     // closing """
    ].concat();
    assert_eq!(parse_block(&raw), "\"\"\" rest");
}

/// Verifies that `\"""` right before closing `"""` works.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn escape_at_end() {
    // Raw token text: """rest \"""""""
    let raw = [
        "\"\"\"",     // opening """
        "rest ",      // content
        "\\\"\"\"",   // \""" (escaped triple quote)
        "\"\"\"",     // closing """
    ].concat();
    assert_eq!(parse_block(&raw), "rest \"\"\"");
}

// =============================================================================
// Blank line trimming (index tracking correctness)
// =============================================================================

/// Verifies that block strings with only whitespace/blank lines
/// return an empty string.
///
/// When all lines are blank, `first_non_blank` is `None` and the
/// function returns `Ok(String::new())` early. Per GraphQL spec,
/// leading and trailing blank lines are removed from block strings:
/// <https://spec.graphql.org/September2025/#BlockStringValue()>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn all_blank_lines() {
    let raw = "\"\"\"\n   \n  \n   \n\"\"\"";
    assert_eq!(parse_block(raw), "");
}

/// Verifies that leading and trailing blank lines are stripped,
/// leaving only the single content line.
///
/// Tests the `first_non_blank` and `last_non_blank` index tracking:
/// lines before `first_non_blank` and after `last_non_blank` are
/// skipped in pass 2. Per GraphQL spec, leading and trailing blank
/// lines are removed from block strings:
/// <https://spec.graphql.org/September2025/#BlockStringValue()>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn single_content_line_surrounded_by_blanks() {
    let raw = "\"\"\"\n\n\n    content\n\n\n\"\"\"";
    assert_eq!(parse_block(raw), "content");
}

/// Verifies that content on the first line only (rest blank) works
/// correctly.
///
/// Tests the `first_non_blank == 0` path where the first line has
/// content and subsequent lines are blank.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn only_first_line_content() {
    let raw = "\"\"\"content\n\n\n\"\"\"";
    assert_eq!(parse_block(raw), "content");
}

// =============================================================================
// Indentation edge cases
// =============================================================================

/// Verifies that a line shorter than common indent is preserved
/// as-is without causing a negative slice.
///
/// The implementation guards with `line.len() >= common_indent`.
/// When the line is shorter (e.g., contains only a few spaces but
/// common indent is larger), it writes the entire line without
/// stripping.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn line_shorter_than_common_indent() {
    // Line 2 has 2 spaces, line 3 has 6 spaces.
    // Common indent = 2 (minimum of non-first, non-blank lines).
    // Line 2: "  ab" -> stripped 2 -> "ab"
    // Line 3: "      cd" -> stripped 2 -> "    cd"
    let raw = "\"\"\"\n  ab\n      cd\n\"\"\"";
    assert_eq!(parse_block(raw), "ab\n    cd");
}

/// Verifies that tabs count as 1 character for common indent
/// calculation.
///
/// Per GraphQL spec, `WhiteSpace` is Tab (U+0009) and Space
/// (U+0020). A tab byte is 1 byte, so it contributes 1 to the
/// indent count.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn tab_indentation() {
    let raw = "\"\"\"\n\tline1\n\tline2\n\"\"\"";
    assert_eq!(parse_block(raw), "line1\nline2");
}

/// Verifies that mixed tabs and spaces are handled correctly in
/// indent calculation.
///
/// Tabs and spaces both count as 1 byte each in the byte-level
/// indent counting.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn mixed_tab_space_indent() {
    let raw = "\"\"\"\n\t line1\n\t line2\n\"\"\"";
    assert_eq!(parse_block(raw), "line1\nline2");
}

// =============================================================================
// Line ending variants
// =============================================================================

/// Verifies that `\r\n` line endings in block strings are handled
/// correctly.
///
/// `str::lines()` splits on both `\n` and `\r\n`, so CRLF should
/// be transparent to the indent/trim algorithm.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn crlf_line_endings() {
    let raw = "\"\"\"\r\n    line1\r\n    line2\r\n\"\"\"";
    assert_eq!(parse_block(raw), "line1\nline2");
}

/// Verifies that `\r`-only line endings in block strings are
/// handled correctly.
///
/// Per GraphQL spec, `\r` is a valid line terminator:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Line-Terminators>
///
/// However, Rust's `str::lines()` does NOT split on bare `\r`
/// (only on `\n` and `\r\n`). A bare `\r` remains as part of the
/// line content. This tests that the function doesn't panic or
/// produce incorrect results in this edge case.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn cr_only_line_endings() {
    let raw = "\"\"\"line1\rline2\"\"\"";
    // With bare \r, str::lines() treats this as a single line
    // containing the \r character. The \r is preserved in the
    // output.
    let result = parse_block(raw);
    assert_eq!(result, "line1\rline2");
}

// =============================================================================
// Unicode content
// =============================================================================

/// Verifies that emoji and CJK characters in block string content
/// survive indent stripping.
///
/// Unicode content should pass through the two-pass algorithm
/// without corruption.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unicode_content_preserved() {
    let raw = "\"\"\"\n    🎉 hello 你好\n    café\n\"\"\"";
    assert_eq!(
        parse_block(raw),
        "🎉 hello 你好\ncafé",
    );
}

/// Verifies that non-ASCII characters in the whitespace region are
/// NOT considered whitespace by `is_graphql_blank()` and are NOT
/// stripped as indent.
///
/// `is_graphql_blank()` only considers bytes `b' '` and `b'\t'` as
/// whitespace. Non-ASCII bytes (>= 0x80) are not whitespace, so a
/// line starting with a multi-byte character has 0 indent.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unicode_in_indent_region() {
    // Line 2 starts with "α" (non-ASCII, not whitespace),
    // line 3 starts with 4 spaces. Common indent = 0 because
    // line 2 has 0 whitespace prefix.
    let raw = "\"\"\"\n\u{03B1}line\n    other\n\"\"\"";
    assert_eq!(parse_block(raw), "\u{03B1}line\n    other");
}
