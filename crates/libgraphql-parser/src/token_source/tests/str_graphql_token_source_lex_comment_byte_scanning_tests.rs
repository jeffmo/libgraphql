//! Tests for the byte-scanning optimization in
//! `StrGraphQLTokenSource::lex_comment()`.
//!
//! ## Optimization summary
//!
//! `lex_comment()` was rewritten to byte-scan for end-of-line
//! (`\n`, `\r`) instead of per-character `peek_char()` /
//! `consume()`. Since comments are single-line, only the column
//! advances — the line number stays the same. Column computation
//! is done once via `compute_columns_for_span()`, which has an
//! ASCII fast path (just use `len()`) and a non-ASCII fallback
//! (iterate chars, summing `len_utf16()` for UTF-16 columns).
//!
//! ## What these tests verify
//!
//! - Position of the token after a comment is correct
//! - Comments with multi-byte UTF-8 content produce correct
//!   positions via `compute_columns_for_span()`
//! - Comment at EOF (no trailing newline)
//! - Empty comment (just `#`)
//! - Comments with emoji (4-byte UTF-8, surrogate pair in UTF-16)
//!
//! Written by Claude Code, reviewed by a human.

use crate::token::GraphQLTokenKind;
use crate::token_source::StrGraphQLTokenSource;

// =============================================================================
// Position tracking after comments
// =============================================================================

/// Verifies that a name after a comment starts at the correct
/// position.
///
/// The comment byte scan advances the byte offset and column
/// through the comment content, then `skip_whitespace` handles
/// the newline. The name should start at (1, 0).
///
/// Per GraphQL spec, comments start with `#` and extend to the
/// end of the line:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Comments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_after_comment() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("# comment\nname").collect();
    // Tokens: name, Eof (comment is captured as trivia)
    assert_eq!(tokens.len(), 2);

    assert!(
        matches!(
            tokens[0].kind,
            GraphQLTokenKind::Name(ref n) if n == "name"
        ),
    );
    assert_eq!(tokens[0].span.start_inclusive.line(), 1);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
}

// =============================================================================
// Unicode content in comments
// =============================================================================

/// Verifies that positions are correct after a comment containing
/// multi-byte UTF-8 characters.
///
/// `compute_columns_for_span()` detects non-ASCII content and
/// iterates chars to compute `col_utf8` (char count) and
/// `col_utf16` (sum of `len_utf16()` per char). This ensures the
/// position state is correct for the next token.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn comment_with_unicode_content() {
    // "café ☕" contains:
    //   c (1b) a (1b) f (1b) é (2b) ' ' (1b) ☕ (3b) = 9 bytes
    //   6 chars (UTF-8 cols), 6 UTF-16 units (all BMP)
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("# caf\u{00E9} \u{2615}\nname")
            .collect();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens[0].span.start_inclusive.line(), 1);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    // Byte offset: # (1) + ' ' (1) + café (5) + ' ' (1) + ☕ (3)
    //   + \n (1) = 12
    assert_eq!(
        tokens[0].span.start_inclusive.byte_offset(),
        12,
    );
}

/// Verifies that positions are correct after a comment containing
/// emoji characters (4-byte UTF-8, surrogate pairs in UTF-16).
///
/// `compute_columns_for_span()` counts each emoji as 1 UTF-8
/// column but 2 UTF-16 units. This doesn't directly affect the
/// next token's position (which starts on a new line), but
/// validates that `compute_columns_for_span()` doesn't corrupt
/// state.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn comment_with_emoji() {
    // 🎉 (4 bytes, 1 UTF-8 col, 2 UTF-16 units)
    // 🎊 (4 bytes, 1 UTF-8 col, 2 UTF-16 units)
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("# \u{1F389}\u{1F38A}\nname")
            .collect();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    assert_eq!(tokens[0].span.start_inclusive.line(), 1);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    // Byte offset: # (1) + ' ' (1) + 🎉 (4) + 🎊 (4) + \n (1) = 11
    assert_eq!(
        tokens[0].span.start_inclusive.byte_offset(),
        11,
    );
}

// =============================================================================
// Boundary conditions
// =============================================================================

/// Verifies that a comment at EOF (no trailing newline) is handled
/// correctly.
///
/// The byte scan reaches `i >= bytes.len()` and exits. The comment
/// content is captured, and Eof follows.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn comment_at_eof_no_newline() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("name # trailing").collect();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    assert!(
        matches!(
            tokens[0].kind,
            GraphQLTokenKind::Name(ref n) if n == "name"
        ),
    );

    // Eof should have the comment as trivia and position at end
    assert!(matches!(tokens[1].kind, GraphQLTokenKind::Eof));
    assert_eq!(tokens[1].span.start_inclusive.line(), 0);
    // "name # trailing" = 15 bytes
    assert_eq!(
        tokens[1].span.start_inclusive.byte_offset(),
        15,
    );
}

/// Verifies that an empty comment (just `#` followed by newline)
/// is handled correctly.
///
/// The byte scan starts at content_start and immediately hits `\n`,
/// so the content is empty. `compute_columns_for_span("")` returns
/// (0, 0).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn empty_comment() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("#\nname").collect();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    assert!(
        matches!(
            tokens[0].kind,
            GraphQLTokenKind::Name(ref n) if n == "name"
        ),
    );
    assert_eq!(tokens[0].span.start_inclusive.line(), 1);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    // # (1) + \n (1) = 2 bytes
    assert_eq!(
        tokens[0].span.start_inclusive.byte_offset(),
        2,
    );
}
