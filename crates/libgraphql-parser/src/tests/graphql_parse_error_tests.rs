//! Tests for `GraphQLParseError` construction and formatting.
//!
//! These tests verify error construction, note management, and display
//! formatting work correctly.
//!
//! Written by Claude Code, reviewed by a human.

use crate::GraphQLErrorNote;
use crate::GraphQLErrorNoteKind;
use crate::GraphQLErrorNotes;
use crate::GraphQLParseError;
use crate::GraphQLParseErrorKind;
use crate::GraphQLSourceSpan;
use crate::ReservedNameContext;
use crate::SourcePosition;

/// Helper to create a test span at the specified position.
fn span_at(line: usize, col: usize, len: usize) -> GraphQLSourceSpan {
    GraphQLSourceSpan::new(
        SourcePosition::new(line, col, Some(col), col),
        SourcePosition::new(line, col + len, Some(col + len), col + len),
    )
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
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        unexpected_token_kind(),
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
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        unexpected_token_kind(),
        notes,
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
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        lexer_notes,
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
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        unexpected_token_kind(),
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
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        unclosed_delimiter_kind(),
    );

    let opening_span = span_at(/* line = */ 5, /* col = */ 10, /* len = */ 1);
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
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        unexpected_token_kind(),
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
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        GraphQLParseErrorKind::InvalidSyntax,
    );

    let suggestion_span = span_at(/* line = */ 1, /* col = */ 20, /* len = */ 5);
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
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        GraphQLParseErrorKind::ReservedName {
            name: "true".to_string(),
            context: ReservedNameContext::EnumValue,
        },
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
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        unclosed_delimiter_kind(),
    );

    error.add_note("Expected `}` to close type definition");
    error.add_note_with_span(
        "Opening `{` here",
        span_at(/* line = */ 1, /* col = */ 15, /* len = */ 1),
    );
    error.add_help("Add a closing `}` at the end of the type definition");

    assert_eq!(error.notes().len(), 3);
}

// =============================================================================
// Part 3.3: Error Display Formatting Tests
// =============================================================================

/// Verifies that `format_oneline()` produces single-line error format.
///
/// Format: "file:line:col: error: message"
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_format_oneline() {
    // 0-indexed, will display as line 5, col 12
    let span = span_at(/* line = */ 4, /* col = */ 11, /* len = */ 5);
    let error = GraphQLParseError::new(
        "Expected `:` after field name",
        span,
        unexpected_token_kind(),
    );

    let formatted = error.format_oneline();

    // Should contain file (or <input>), line number (1-indexed), column, message
    assert!(formatted.contains("<input>:5:12:"));
    assert!(formatted.contains("error:"));
    assert!(formatted.contains("Expected `:` after field name"));
}

/// Verifies that `format_detailed()` without source produces basic format.
///
/// When no source is provided, we can still show location info but not source
/// snippets.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_format_detailed_without_source() {
    let error = GraphQLParseError::new(
        "Unexpected token",
        span_at(/* line = */ 2, /* col = */ 5, /* len = */ 3),
        unexpected_token_kind(),
    );

    let formatted = error.format_detailed(None);

    assert!(formatted.contains("error:"));
    assert!(formatted.contains("Unexpected token"));
    assert!(formatted.contains("-->"));
    assert!(formatted.contains("3:6")); // 1-indexed line:col
}

/// Verifies that `format_detailed()` with source includes source snippet.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_format_detailed_with_source() {
    let source = "type Query {\n    userName String\n}";
    // Error at line 1 (0-indexed), col 13 (pointing at "String")
    let span = span_at(/* line = */ 1, /* col = */ 13, /* len = */ 6);
    let error = GraphQLParseError::new(
        "Expected `:` after field name",
        span,
        unexpected_token_kind(),
    );

    let formatted = error.format_detailed(Some(source));

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
        span_at(/* line = */ 3, /* col = */ 0, /* len = */ 1),
        unclosed_delimiter_kind(),
    );
    error.add_note("Expected `}` to close type definition");
    error.add_help("Check that all braces are properly matched");

    let formatted = error.format_detailed(None);

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
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        GraphQLParseErrorKind::ReservedName {
            name: "null".to_string(),
            context: ReservedNameContext::EnumValue,
        },
    );
    error.add_spec("https://spec.graphql.org/September2025/#sec-Enums");

    let formatted = error.format_detailed(None);

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
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        unexpected_token_kind(),
    );

    assert_eq!(error.message(), "Test message");
}

/// Verifies that `span()` returns the error span.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_span_accessor() {
    let span = span_at(/* line = */ 10, /* col = */ 20, /* len = */ 5);
    let error = GraphQLParseError::new(
        "Error",
        span.clone(),
        unexpected_token_kind(),
    );

    assert_eq!(error.span().start_inclusive.line(), 10);
    assert_eq!(error.span().start_inclusive.col_utf8(), 20);
}

/// Verifies that `kind()` returns the error kind.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_kind_accessor() {
    let error = GraphQLParseError::new(
        "Error",
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        unclosed_delimiter_kind(),
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
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        unexpected_token_kind(),
        notes,
    );

    assert_eq!(error.notes().len(), 2);
}

// =============================================================================
// Part 3.3: Display Trait Test
// =============================================================================

/// Verifies that the Display trait (via thiserror) uses `format_oneline()`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_error_display_trait() {
    let error = GraphQLParseError::new(
        "Test error message",
        span_at(/* line = */ 0, /* col = */ 0, /* len = */ 1),
        unexpected_token_kind(),
    );

    let display_output = format!("{error}");

    // Display should use the oneline format
    assert!(display_output.contains("Test error message"));
    assert!(display_output.contains("<input>:1:1:"));
}
