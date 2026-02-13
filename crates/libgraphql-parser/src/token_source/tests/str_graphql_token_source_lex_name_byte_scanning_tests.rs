//! Tests for the byte-scanning optimization in
//! `StrGraphQLTokenSource::lex_name()`.
//!
//! ## Optimization summary
//!
//! `lex_name()` was rewritten to scan name bytes directly via
//! `source.as_bytes()` and `is_name_continue_byte()` instead of
//! calling `peek_char()` / `consume()` per character. Since GraphQL
//! names are ASCII-only (`[_A-Za-z][_0-9A-Za-z]*`) and never
//! contain newlines, the position state (byte offset, columns, line)
//! is batch-updated after the byte scan completes:
//!
//! - `curr_byte_offset += name_len`
//! - `curr_col_utf8 += name_len`
//! - `curr_col_utf16 += name_len`
//! - Line number stays the same
//! - `last_char_was_cr` is cleared
//!
//! ## What these tests verify
//!
//! - Batch position update produces correct start/end spans for
//!   names of various lengths
//! - Consecutive names accumulate positions correctly through
//!   repeated byte-scan → skip_whitespace → byte-scan cycles
//! - Names after newlines start at the correct (line, col=0)
//! - Names at EOF (no trailing content) are scanned correctly
//! - Single-character names (minimum length)
//! - Underscore-prefixed names (`__typename`) with byte scanning
//! - Keyword recognition (`true`, `false`, `null`) still works
//!   through the byte-scanning path
//!
//! Written by Claude Code, reviewed by a human.

use crate::token::GraphQLTokenKind;
use crate::token_source::StrGraphQLTokenSource;

// =============================================================================
// Batch position update correctness
// =============================================================================

/// Verifies that the byte-scanning loop produces correct start/end
/// spans for two names separated by whitespace.
///
/// After byte-scanning "type" (4 bytes), the batch update should
/// advance `col_utf8` and `col_utf16` by 4. After `skip_whitespace`
/// advances by 1 (the space), "Query" starts at col 5 and ends at
/// col 10.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn position_tracking_two_names() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("type Query").collect();
    // Tokens: type, Query, Eof
    assert_eq!(tokens.len(), 3);

    // "type" at (0,0)-(0,4)
    assert_eq!(tokens[0].span.start_inclusive.line(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(
        tokens[0].span.start_inclusive.byte_offset(),
        0,
    );
    assert_eq!(tokens[0].span.end_exclusive.line(), 0);
    assert_eq!(tokens[0].span.end_exclusive.col_utf8(), 4);
    assert_eq!(tokens[0].span.end_exclusive.byte_offset(), 4);

    // "Query" at (0,5)-(0,10)
    assert_eq!(tokens[1].span.start_inclusive.line(), 0);
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 5);
    assert_eq!(
        tokens[1].span.start_inclusive.byte_offset(),
        5,
    );
    assert_eq!(tokens[1].span.end_exclusive.line(), 0);
    assert_eq!(tokens[1].span.end_exclusive.col_utf8(), 10);
    assert_eq!(
        tokens[1].span.end_exclusive.byte_offset(),
        10,
    );
}

/// Verifies that consecutive names of increasing length accumulate
/// positions correctly through repeated byte-scan cycles.
///
/// Each name's byte scan batch-updates the position by `name_len`.
/// The intervening `skip_whitespace` adds 1 column per space. This
/// tests that the accumulation stays correct over many names.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn consecutive_names_accumulate_positions() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("a bb ccc dddd").collect();
    // Tokens: a, bb, ccc, dddd, Eof
    assert_eq!(tokens.len(), 5);

    // "a" at (0,0)-(0,1)
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[0].span.end_exclusive.col_utf8(), 1);

    // "bb" at (0,2)-(0,4)
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 2);
    assert_eq!(tokens[1].span.end_exclusive.col_utf8(), 4);

    // "ccc" at (0,5)-(0,8)
    assert_eq!(tokens[2].span.start_inclusive.col_utf8(), 5);
    assert_eq!(tokens[2].span.end_exclusive.col_utf8(), 8);

    // "dddd" at (0,9)-(0,13)
    assert_eq!(tokens[3].span.start_inclusive.col_utf8(), 9);
    assert_eq!(tokens[3].span.end_exclusive.col_utf8(), 13);
}

// =============================================================================
// Names after newlines
// =============================================================================

/// Verifies that a name after a newline starts at (line+1, col=0).
///
/// `skip_whitespace` handles the newline and resets the column.
/// The subsequent `lex_name` byte scan starts from col 0 on the
/// new line.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn name_after_newline_position() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("name\nsecondName").collect();
    // Tokens: name, secondName, Eof
    assert_eq!(tokens.len(), 3);

    // "name" at (0,0)-(0,4)
    assert_eq!(tokens[0].span.start_inclusive.line(), 0);
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[0].span.end_exclusive.col_utf8(), 4);

    // "secondName" at (1,0)-(1,10)
    assert_eq!(tokens[1].span.start_inclusive.line(), 1);
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[1].span.end_exclusive.line(), 1);
    assert_eq!(tokens[1].span.end_exclusive.col_utf8(), 10);
}

// =============================================================================
// Boundary conditions
// =============================================================================

/// Verifies that a name at EOF (no trailing whitespace or content)
/// is scanned correctly.
///
/// The byte-scan loop terminates when `i >= bytes.len()`. This
/// tests that the loop exits cleanly at the end of input without
/// overrunning.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn name_at_eof() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("name").collect();
    // Tokens: name, Eof
    assert_eq!(tokens.len(), 2);

    assert!(
        matches!(
            tokens[0].kind,
            GraphQLTokenKind::Name(ref n) if n == "name"
        ),
    );
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[0].span.end_exclusive.col_utf8(), 4);
    assert_eq!(tokens[0].span.end_exclusive.byte_offset(), 4);
}

/// Verifies that a single-character name produces the correct
/// 1-byte span.
///
/// This is the minimum-length name: the byte scan starts at
/// `name_start + 1` and immediately exits the loop since the next
/// byte (if any) is not a name-continue character. `name_len` is 1.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn single_char_name() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("x").collect();
    // Tokens: x, Eof
    assert_eq!(tokens.len(), 2);

    assert!(
        matches!(
            tokens[0].kind,
            GraphQLTokenKind::Name(ref n) if n == "x"
        ),
    );
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[0].span.start_inclusive.byte_offset(), 0);
    assert_eq!(tokens[0].span.end_exclusive.col_utf8(), 1);
    assert_eq!(tokens[0].span.end_exclusive.byte_offset(), 1);
}

/// Verifies that underscore-prefixed names (`__typename`) are
/// scanned correctly by the byte loop.
///
/// Underscores are valid in both the name-start and name-continue
/// positions per the GraphQL spec. The byte scanner checks
/// `is_name_continue_byte()` which includes `b'_'`.
///
/// Per GraphQL spec, names match `[_A-Za-z][_0-9A-Za-z]*`:
/// <https://spec.graphql.org/September2025/#sec-Names>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn underscore_prefix_name() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("__typename").collect();
    // Tokens: __typename, Eof
    assert_eq!(tokens.len(), 2);

    assert!(
        matches!(
            tokens[0].kind,
            GraphQLTokenKind::Name(ref n) if n == "__typename"
        ),
    );
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[0].span.end_exclusive.col_utf8(), 10);
    assert_eq!(
        tokens[0].span.end_exclusive.byte_offset(),
        10,
    );
}

// =============================================================================
// Keyword recognition through byte scanning
// =============================================================================

/// Verifies that keywords (`true`, `false`, `null`) are still
/// recognized correctly after byte-scanning the name.
///
/// After the byte scan completes, the name slice is matched against
/// known keywords. This ensures the byte-scanning path doesn't skip
/// or corrupt the keyword check.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn keywords_recognized_after_byte_scan() {
    let tokens: Vec<_> =
        StrGraphQLTokenSource::new("true false null name")
            .collect();
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
    assert_eq!(tokens[0].span.start_inclusive.col_utf8(), 0);
    assert_eq!(tokens[0].span.end_exclusive.col_utf8(), 4);

    // "false" at col 5-10
    assert_eq!(tokens[1].span.start_inclusive.col_utf8(), 5);
    assert_eq!(tokens[1].span.end_exclusive.col_utf8(), 10);

    // "null" at col 11-15
    assert_eq!(tokens[2].span.start_inclusive.col_utf8(), 11);
    assert_eq!(tokens[2].span.end_exclusive.col_utf8(), 15);

    // "name" at col 16-20
    assert_eq!(tokens[3].span.start_inclusive.col_utf8(), 16);
    assert_eq!(tokens[3].span.end_exclusive.col_utf8(), 20);
}
