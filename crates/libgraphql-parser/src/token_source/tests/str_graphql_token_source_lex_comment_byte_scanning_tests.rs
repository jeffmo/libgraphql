//! Tests for the byte-scanning optimization in
//! `StrGraphQLTokenSource::lex_comment()`.
//!
//! ## Optimization summary
//!
//! `lex_comment()` byte-scans for end-of-line (`\n`, `\r`) instead
//! of per-character `peek_char()` / `consume()`. Position resolution
//! (line/col) is deferred to SourceMap.
//!
//! ## What these tests verify
//!
//! - Byte offsets of the token after a comment are correct
//! - SourceMap resolves correct line/col for tokens after comments
//!   with multi-byte UTF-8 content
//! - Comment at EOF (no trailing newline)
//! - Empty comment (just `#`)
//! - Comments with emoji (4-byte UTF-8, surrogate pair in UTF-16)
//!
//! Written by Claude Code, reviewed by a human.

use crate::token::GraphQLTokenKind;
use crate::token_source::GraphQLTokenSource;
use crate::token_source::StrGraphQLTokenSource;

// =============================================================================
// Position tracking after comments
// =============================================================================

/// Verifies that a name after a comment starts at the correct
/// position.
///
/// The comment byte scan advances the byte offset through the
/// comment content, then `skip_whitespace` handles the newline.
/// The name should start at (1, 0).
///
/// Per GraphQL spec, comments start with `#` and extend to the
/// end of the line:
/// <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Comments>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_after_comment() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("# comment\nname")
            .collect_with_source_map();
    // Tokens: name, Eof (comment is captured as trivia)
    assert_eq!(tokens.len(), 2);

    assert!(
        matches!(
            tokens[0].kind,
            GraphQLTokenKind::Name(ref n) if n == "name"
        ),
    );
    let pos = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 0);
}

// =============================================================================
// Unicode content in comments
// =============================================================================

/// Verifies that positions are correct after a comment containing
/// multi-byte UTF-8 characters.
///
/// The SourceMap resolves the byte offset to the correct line/col.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn comment_with_unicode_content() {
    // "café ☕" contains:
    //   c (1b) a (1b) f (1b) é (2b) ' ' (1b) ☕ (3b) = 9 bytes
    //   6 chars (UTF-8 cols), 6 UTF-16 units (all BMP)
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("# caf\u{00E9} \u{2615}\nname")
            .collect_with_source_map();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    let pos = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 0);
    // Byte offset: # (1) + ' ' (1) + café (5) + ' ' (1) + ☕ (3)
    //   + \n (1) = 12
    assert_eq!(tokens[0].span.start, 12);
}

/// Verifies that positions are correct after a comment containing
/// emoji characters (4-byte UTF-8, surrogate pairs in UTF-16).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn comment_with_emoji() {
    // 🎉 (4 bytes, 1 UTF-8 col, 2 UTF-16 units)
    // 🎊 (4 bytes, 1 UTF-8 col, 2 UTF-16 units)
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("# \u{1F389}\u{1F38A}\nname")
            .collect_with_source_map();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    let pos = source_map.resolve_offset(tokens[0].span.start).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 0);
    // Byte offset: # (1) + ' ' (1) + 🎉 (4) + 🎊 (4) + \n (1) = 11
    assert_eq!(tokens[0].span.start, 11);
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
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("name # trailing")
            .collect_with_source_map();
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
    let pos = source_map.resolve_offset(tokens[1].span.start).unwrap();
    assert_eq!(pos.line(), 0);
    // "name # trailing" = 15 bytes
    assert_eq!(tokens[1].span.start, 15);
}

/// Verifies that an empty comment (just `#` followed by newline)
/// is handled correctly.
///
/// The byte scan starts at content_start and immediately hits `\n`,
/// so the content is empty.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn empty_comment() {
    let (tokens, source_map) =
        StrGraphQLTokenSource::new("#\nname")
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
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 0);
    // # (1) + \n (1) = 2 bytes
    assert_eq!(tokens[0].span.start, 2);
}
