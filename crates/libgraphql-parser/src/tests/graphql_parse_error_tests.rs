//! Tests for `GraphQLParseError` construction and formatting.
//!
//! These tests verify error construction, note management, and display
//! formatting work correctly.
//!
//! Written by Claude Code, reviewed by a human.

use crate::ByteSpan;
use crate::GraphQLErrorNote;
use crate::GraphQLErrorNoteKind;
use crate::GraphQLErrorNotes;
use crate::GraphQLParseError;
use crate::GraphQLParseErrorKind;
use crate::ReservedNameContext;
use crate::SourceMap;
use crate::SourceSpan;

/// Helper to create a test `ByteSpan` at the specified byte offset with a
/// given length.
fn span_at(byte_offset: u32, len: u32) -> ByteSpan {
    ByteSpan::new(byte_offset, byte_offset + len)
}

/// Helper to create an UnexpectedToken error kind.
fn unexpected_token_kind() -> GraphQLParseErrorKind {
    GraphQLParseErrorKind::UnexpectedToken {
        expected: vec![":".to_string()],
        found: "String".to_string(),
    }
}

/// Helper to create an UnclosedDelimiter error kind.
fn unclosed_delimiter_kind() -> GraphQLParseErrorKind {
    GraphQLParseErrorKind::UnclosedDelimiter {
        delimiter: "{".to_string(),
    }
}

// =============================================================================
// Part 3.2: Constructor Tests
// =============================================================================

/// Verifies that `GraphQLParseError::new()` creates an error with empty notes.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_new_creates_empty_notes() {
    let error = GraphQLParseError::new(
        "Expected `:`",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        unexpected_token_kind(),
        SourceSpan::zero(),
    );

    assert_eq!(error.message(), "Expected `:`");
    assert!(matches!(error.kind(), GraphQLParseErrorKind::UnexpectedToken { .. }));
    assert!(error.notes().is_empty());
}

/// Verifies that `GraphQLParseError::with_notes()` creates an error with
/// pre-populated notes.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_with_notes_constructor() {
    let mut notes = GraphQLErrorNotes::new();
    notes.push(GraphQLErrorNote::general("Additional context"));
    notes.push(GraphQLErrorNote::help("Try adding a colon here"));

    let error = GraphQLParseError::with_notes(
        "Expected `:`",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        unexpected_token_kind(),
        notes,
        SourceSpan::zero(),
    );

    assert_eq!(error.message(), "Expected `:`");
    assert_eq!(error.notes().len(), 2);
}

/// Verifies that `GraphQLParseError::from_lexer_error()` correctly converts
/// a lexer error with its notes.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_from_lexer_error() {
    let mut lexer_notes = GraphQLErrorNotes::new();
    lexer_notes.push(GraphQLErrorNote::general("Lexer detected unterminated string"));

    let error = GraphQLParseError::from_lexer_error(
        "Unterminated string",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        lexer_notes,
        SourceSpan::zero(),
    );

    assert_eq!(error.message(), "Unterminated string");
    assert!(matches!(error.kind(), GraphQLParseErrorKind::LexerError));
    assert_eq!(error.notes().len(), 1);
}

// =============================================================================
// Part 3.2: Note Management Tests
// =============================================================================

/// Verifies that `add_note()` appends a general note without a span.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_add_note() {
    let mut error = GraphQLParseError::new(
        "Primary error",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        unexpected_token_kind(),
        SourceSpan::zero(),
    );

    error.add_note("This is additional context");

    assert_eq!(error.notes().len(), 1);
    let note = &error.notes()[0];
    assert!(matches!(note.kind, GraphQLErrorNoteKind::General));
    assert_eq!(note.message, "This is additional context");
    assert!(note.span.is_none());
}

/// Verifies that `add_note_with_span()` appends a general note with a location.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_add_note_with_span() {
    let mut error = GraphQLParseError::new(
        "Primary error",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        unclosed_delimiter_kind(),
        SourceSpan::zero(),
    );

    let opening_span = span_at(/* byte_offset = */ 50, /* len = */ 1);
    error.add_note_with_span("Opening `{` here", opening_span);

    assert_eq!(error.notes().len(), 1);
    let note = &error.notes()[0];
    assert!(matches!(note.kind, GraphQLErrorNoteKind::General));
    assert!(note.span.is_some());
}

/// Verifies that `add_help()` appends a help note without a span.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_add_help() {
    let mut error = GraphQLParseError::new(
        "Missing colon",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        unexpected_token_kind(),
        SourceSpan::zero(),
    );

    error.add_help("Did you mean: `fieldName: Type`?");

    assert_eq!(error.notes().len(), 1);
    let note = &error.notes()[0];
    assert!(matches!(note.kind, GraphQLErrorNoteKind::Help));
    assert_eq!(note.message, "Did you mean: `fieldName: Type`?");
    assert!(note.span.is_none());
}

/// Verifies that `add_help_with_span()` appends a help note with a location.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_add_help_with_span() {
    let mut error = GraphQLParseError::new(
        "Unknown directive location",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        GraphQLParseErrorKind::InvalidSyntax,
        SourceSpan::zero(),
    );

    let suggestion_span = span_at(/* byte_offset = */ 20, /* len = */ 5);
    error.add_help_with_span("Did you mean `FIELD`?", suggestion_span);

    assert_eq!(error.notes().len(), 1);
    let note = &error.notes()[0];
    assert!(matches!(note.kind, GraphQLErrorNoteKind::Help));
    assert!(note.span.is_some());
}

/// Verifies that `add_spec()` appends a specification reference note.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_add_spec() {
    let mut error = GraphQLParseError::new(
        "Invalid enum value",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        GraphQLParseErrorKind::ReservedName {
            name: "true".to_string(),
            context: ReservedNameContext::EnumValue,
        },
        SourceSpan::zero(),
    );

    error.add_spec("https://spec.graphql.org/September2025/#sec-Enums");

    assert_eq!(error.notes().len(), 1);
    let note = &error.notes()[0];
    assert!(matches!(note.kind, GraphQLErrorNoteKind::Spec));
}

/// Verifies that multiple notes can be added in sequence.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_multiple_notes() {
    let mut error = GraphQLParseError::new(
        "Unclosed brace",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        unclosed_delimiter_kind(),
        SourceSpan::zero(),
    );

    error.add_note("Expected `}` to close type definition");
    error.add_note_with_span(
        "Opening `{` here",
        span_at(/* byte_offset = */ 15, /* len = */ 1),
    );
    error.add_help("Add a closing `}` at the end of the type definition");

    assert_eq!(error.notes().len(), 3);
}

// =============================================================================
// Part 3.3: Error Display Formatting Tests
// =============================================================================

/// Verifies that `format_oneline()` produces single-line error format
/// using the pre-resolved `SourceSpan`.
///
/// Format: "file:line:col: error: message"
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_format_oneline() {
    use crate::SourcePosition;

    // Build a resolved span at line 4, col 11 (0-indexed) = line 5, col 12
    // (1-indexed).
    let resolved = SourceSpan::new(
        SourcePosition::new(4, 11, Some(11), 55),
        SourcePosition::new(4, 16, Some(16), 60),
    );
    let error = GraphQLParseError::new(
        "Expected `:` after field name",
        ByteSpan::new(55, 60),
        unexpected_token_kind(),
        resolved,
    );

    let formatted = error.format_oneline();

    assert_eq!(
        formatted,
        "<input>:5:12: error: Expected `:` after field name",
    );
}

/// Verifies that `format_detailed()` without source produces basic format.
///
/// When no source is provided, we can still show location info but not source
/// snippets. Without source, resolve_offset returns None, so line:col defaults
/// to 1:1.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_format_detailed_without_source() {
    let error = GraphQLParseError::new(
        "Unexpected token",
        span_at(/* byte_offset = */ 5, /* len = */ 3),
        unexpected_token_kind(),
        SourceSpan::zero(),
    );

    let sm = SourceMap::empty();
    let formatted = error.format_detailed(&sm);

    assert!(formatted.contains("error:"));
    assert!(formatted.contains("Unexpected token"));
    assert!(formatted.contains("-->"));
    // Without source, SourceMap::empty() cannot resolve offsets, defaults to
    // 1:1
    assert!(formatted.contains("1:1"));
}

/// Verifies that `format_detailed()` with source includes source snippet.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_format_detailed_with_source() {
    let source = "type Query {\n    userName String\n}";
    // "String" starts at byte offset 25 in the source
    // "type Query {\n" = 13 bytes, "    userName " = 13 bytes => byte 26
    // Actually: "type Query {\n    userName " = 13 + 13 = 26, "String" at 26
    let span = ByteSpan::new(26, 32);
    let error = GraphQLParseError::new(
        "Expected `:` after field name",
        span,
        unexpected_token_kind(),
        SourceSpan::zero(),
    );

    let sm = SourceMap::new_with_source(source, None);
    let formatted = error.format_detailed(&sm);

    assert!(formatted.contains("error:"));
    assert!(formatted.contains("Expected `:` after field name"));
    // Should include the source line
    assert!(formatted.contains("userName String"));
}

/// Verifies that `format_detailed()` renders notes with different kinds.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_format_detailed_with_notes() {
    let mut error = GraphQLParseError::new(
        "Unclosed `{`",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        unclosed_delimiter_kind(),
        SourceSpan::zero(),
    );
    error.add_note("Expected `}` to close type definition");
    error.add_help("Check that all braces are properly matched");

    let sm = SourceMap::empty();
    let formatted = error.format_detailed(&sm);

    assert!(formatted.contains("= note:"));
    assert!(formatted.contains("Expected `}` to close type definition"));
    assert!(formatted.contains("= help:"));
    assert!(formatted.contains("Check that all braces are properly matched"));
}

/// Verifies that spec notes are rendered correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_format_detailed_with_spec_note() {
    let mut error = GraphQLParseError::new(
        "Invalid enum value name",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        GraphQLParseErrorKind::ReservedName {
            name: "null".to_string(),
            context: ReservedNameContext::EnumValue,
        },
        SourceSpan::zero(),
    );
    error.add_spec("https://spec.graphql.org/September2025/#sec-Enums");

    let sm = SourceMap::empty();
    let formatted = error.format_detailed(&sm);

    assert!(formatted.contains("= spec:"));
    assert!(formatted.contains("spec.graphql.org"));
}

// =============================================================================
// Part 3.2: Accessor Tests
// =============================================================================

/// Verifies that `message()` returns the error message.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_message_accessor() {
    let error = GraphQLParseError::new(
        "Test message",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        unexpected_token_kind(),
        SourceSpan::zero(),
    );

    assert_eq!(error.message(), "Test message");
}

/// Verifies that `byte_span()` returns the error byte span with correct offsets.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_span_accessor() {
    let span = span_at(/* byte_offset = */ 20, /* len = */ 5);
    let error = GraphQLParseError::new(
        "Error",
        span,
        unexpected_token_kind(),
        SourceSpan::zero(),
    );

    assert_eq!(error.byte_span().start, 20);
    assert_eq!(error.byte_span().end, 25);
}

/// Verifies that `kind()` returns the error kind.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_kind_accessor() {
    let error = GraphQLParseError::new(
        "Error",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        unclosed_delimiter_kind(),
        SourceSpan::zero(),
    );

    assert!(matches!(
        error.kind(),
        GraphQLParseErrorKind::UnclosedDelimiter { .. }
    ));
}

/// Verifies that `notes()` returns the notes vector.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_notes_accessor() {
    let mut notes = GraphQLErrorNotes::new();
    notes.push(GraphQLErrorNote::general("note 1"));
    notes.push(GraphQLErrorNote::help("note 2"));

    let error = GraphQLParseError::with_notes(
        "Error",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        unexpected_token_kind(),
        notes,
        SourceSpan::zero(),
    );

    assert_eq!(error.notes().len(), 2);
}

// =============================================================================
// Part 3.3: Display Trait Test
// =============================================================================

/// Verifies that `format_source_snippet` correctly handles source text with
/// bare carriage return (`\r`) line endings (legacy Mac style).
///
/// The GraphQL spec (Section 2.2 "Source Text") recognizes `\r` as a line
/// terminator. `SourceMap::compute_line_starts()` correctly handles this, but
/// the snippet formatter must also split lines using the same logic rather
/// than relying on Rust's `str::lines()`, which does NOT treat bare `\r` as a
/// line terminator.
///
/// With the bug: `source.lines()` sees 1 line (the whole string), but
/// `SourceMap` resolves "hello" to line index 1 (0-based). Since
/// `line_num (1) >= lines.len() (1)`, `format_source_snippet` returns `None`
/// and the formatted output contains NO source snippet at all.
///
/// With the fix: the snippet should show line 2 (1-indexed) with content
/// "  hello: String" and the `^^^^^` underline beneath "hello".
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_format_detailed_with_bare_cr_line_endings() {
    // Source with bare \r line endings (no \n):
    // Line 0: "type Query {"
    // Line 1: "  hello: String"
    // Line 2: "}"
    let source = "type Query {\r  hello: String\r}";

    // "hello" starts at offset 15 (after "type Query {\r  ")
    let span = ByteSpan::new(15, 20); // "hello"

    let error = GraphQLParseError::new(
        "test error on CR-only source",
        span,
        unexpected_token_kind(),
        SourceSpan::zero(),
    );

    let sm = SourceMap::new_with_source(source, None);
    let formatted = error.format_detailed(&sm);

    // The formatted output should contain a source snippet with line number
    // "2" (1-indexed for line index 1). With the str::lines() bug, no snippet
    // is produced at all because str::lines() returns only 1 element.
    assert!(
        formatted.contains(" 2 |"),
        "Snippet should show line number 2 for the \\r-separated line \
         containing 'hello', but got:\n{formatted}",
    );
    // Underline carets should appear under "hello"
    assert!(
        formatted.contains("^^^^^"),
        "Snippet should underline 'hello' with 5 carets, but got:\n{formatted}",
    );
}

/// Verifies that `format_note_snippet` correctly handles bare `\r` line
/// endings for note spans.
///
/// Same underlying issue as the source snippet test above. With the bug,
/// `str::lines()` produces a single element for `\r`-separated text, so
/// looking up line index 1 fails and no note snippet is produced.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_format_note_snippet_with_bare_cr_line_endings() {
    // Source with bare \r line endings:
    // Line 0: "type Query {"
    // Line 1: "  hello: String"
    // Line 2: "}"
    let source = "type Query {\r  hello: String\r}";

    // Primary error at offset 0 (line 0)
    let mut error = GraphQLParseError::new(
        "primary error",
        ByteSpan::new(0, 1),
        unexpected_token_kind(),
        SourceSpan::zero(),
    );

    // Note pointing to "hello" on line 1 (0-indexed)
    let note_span = ByteSpan::new(15, 20);
    error.add_note_with_span("see this token", note_span);

    let sm = SourceMap::new_with_source(source, None);
    let formatted = error.format_detailed(&sm);

    // The note snippet should show line number 2 (1-indexed). With the bug,
    // no note snippet is produced because str::lines() can't index line 1.
    assert!(
        formatted.contains("     2 |"),
        "Note snippet should show line number 2 for the \\r-separated line \
         containing 'hello', but got:\n{formatted}",
    );
}

/// Verifies that Display shows `<input>:1:1` when constructed with
/// `SourceSpan::zero()`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_display_trait_without_source() {
    let error = GraphQLParseError::new(
        "Test error message",
        span_at(/* byte_offset = */ 0, /* len = */ 1),
        unexpected_token_kind(),
        SourceSpan::zero(),
    );

    let display_output = format!("{error}");
    assert_eq!(
        display_output,
        "<input>:1:1: error: Test error message",
    );
}

/// Verifies that Display includes file:line:col from a resolved
/// `SourceSpan` with a file path.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_display_trait_with_resolved_span() {
    use crate::SourcePosition;
    use std::path::PathBuf;

    let resolved = SourceSpan::with_file(
        SourcePosition::new(
            /* line = */ 4, /* col_utf8 = */ 11,
            Some(11), /* byte_offset = */ 55,
        ),
        SourcePosition::new(
            /* line = */ 4, /* col_utf8 = */ 16,
            Some(16), /* byte_offset = */ 60,
        ),
        PathBuf::from("schema.graphql"),
    );
    let error = GraphQLParseError::new(
        "Expected `:` after field name",
        ByteSpan::new(55, 60),
        unexpected_token_kind(),
        resolved,
    );

    let display_output = format!("{error}");
    assert_eq!(
        display_output,
        "schema.graphql:5:12: error: Expected `:` after field name",
    );
}

/// Verifies Display falls back to `<input>` when resolved span has no
/// file path.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_display_trait_resolved_span_no_file() {
    use crate::SourcePosition;

    let resolved = SourceSpan::new(
        SourcePosition::new(2, 5, Some(5), 30),
        SourcePosition::new(2, 10, Some(10), 35),
    );
    let error = GraphQLParseError::new(
        "Unexpected token",
        ByteSpan::new(30, 35),
        unexpected_token_kind(),
        resolved,
    );

    let display_output = format!("{error}");
    assert_eq!(
        display_output,
        "<input>:3:6: error: Unexpected token",
    );
}

/// Verifies that errors produced by the parser carry resolved spans
/// with real line/column info, so Display shows useful locations
/// without a SourceMap.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_from_parser_has_resolved_span() {
    use crate::GraphQLParser;

    let source = "type Query {\n  name String\n}";
    let parser = GraphQLParser::new(source);
    let result = parser.parse_schema_document();
    assert!(result.has_errors(), "should have parse errors");

    let error = &result.errors()[0];
    let display = format!("{error}");
    // Should show real location (line 2), not the 1:1 fallback
    assert!(
        display.contains(":2:"),
        "Display should show real line number, got: {display}",
    );
    assert!(
        !display.contains(":1:1: error:"),
        "Display should not show fallback 1:1 position, got: {display}",
    );
}
