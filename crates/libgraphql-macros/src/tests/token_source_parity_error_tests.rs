//! Section B — Error Parity tests.
//!
//! These tests feed identical invalid input strings to both
//! `RustMacroGraphQLTokenSource` and `StrGraphQLTokenSource`,
//! asserting that they produce errors at the same positions with
//! matching error messages and notes.
//!
//! See: https://spec.graphql.org/September2025/#sec-Lexical-Tokens

use crate::tests::token_source_parity_utils::assert_parity;

/// Tests that a standalone minus sign (`-` not followed by a
/// number) produces identical error tokens from both sources.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn error_parity_standalone_minus() {
    assert_parity("a - b");
}

/// Tests that a percent sign produces identical error tokens from
/// both sources. `%` is not valid GraphQL syntax.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn error_parity_percent() {
    assert_parity("a % b");
}

/// Tests that a tilde produces identical error tokens from both
/// sources. `~` is not valid GraphQL syntax.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn error_parity_tilde() {
    assert_parity("~");
}

/// Tests that a single dot surrounded by names produces identical
/// error tokens from both sources.
///
/// Both sources produce `"Unexpected \`.\`"` with no error notes
/// for a single isolated dot.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn error_parity_single_dot() {
    assert_parity("a . b");
}

/// Tests that two adjacent dots (`..`) produce identical error
/// tokens from both sources.
///
/// Both sources produce
/// `"Unexpected \`..\` (use \`...\` for spread operator)"` with
/// a help note suggesting to add one more dot.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn error_parity_double_dot() {
    assert_parity("a..b");
}

/// Tests that two spaced dots (`. .`) on the same line produce
/// identical error tokens from both sources.
///
/// Both sources produce
/// `"Unexpected \`. .\` (use \`...\` for spread operator)"` with
/// a help note about removing spacing.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn error_parity_spaced_dots() {
    assert_parity(". .");
}

/// Tests that `.. .` (two adjacent dots then a spaced dot)
/// produces identical error tokens from both sources.
///
/// Both sources produce `"Unexpected \`.. .\`"` with a help note
/// about the third dot possibly intended to complete `...`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn error_parity_dot_dot_space_dot() {
    assert_parity(".. .");
}

/// Tests that `. . .` (all spaced dots) produces identical error
/// tokens from both sources.
///
/// Both sources produce `"Unexpected \`. . .\`"` with a help note
/// suggesting to remove spacing to form `...`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn error_parity_all_spaced_dots() {
    assert_parity(". . .");
}
