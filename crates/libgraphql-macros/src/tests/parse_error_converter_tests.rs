//! Unit tests for `crate::parse_error_converter`.
//!
//! These tests verify the formatting of GraphQL parse errors into
//! human-readable messages and `compile_error!` token streams.
//!
//! The converter is responsible for translating structured
//! `GraphQLParseError` values (with notes of varying kinds) into
//! messages that appear as Rust compiler diagnostics. Correct
//! formatting here is critical because it directly affects the
//! quality of error messages users see in their editor/terminal.

use crate::parse_error_converter::convert_parse_errors_to_tokenstream;
use crate::parse_error_converter::format_parse_error_message;
use crate::parse_error_converter::format_parse_error_note;
use crate::span_map::SpanMap;
use libgraphql_parser::GraphQLErrorNote;
use libgraphql_parser::GraphQLParseError;
use libgraphql_parser::GraphQLParseErrorKind;
use libgraphql_parser::GraphQLSourceSpan;
use libgraphql_parser::SourcePosition;
use proc_macro2::Span;
use std::collections::HashMap;

// ── Helpers ──────────────────────────────────────────────────────

/// Creates a dummy `GraphQLSourceSpan` at a given (line, col)
/// position. Both start and end point to the same position, which
/// is sufficient for these tests since we only care about the
/// `start_inclusive` field for span-map lookups.
fn dummy_span_at(line: usize, col: usize) -> GraphQLSourceSpan {
    let pos =
        SourcePosition::new(line, col, /* col_utf16 = */ None, 0);
    GraphQLSourceSpan::new(pos.clone(), pos)
}

/// Creates a `GraphQLParseError` with no notes at position (0,0).
fn simple_error(message: &str) -> GraphQLParseError {
    GraphQLParseError::new(
        message.to_string(),
        dummy_span_at(0, 0),
        GraphQLParseErrorKind::InvalidSyntax,
    )
}

/// Creates a `GraphQLParseError` with no notes at the given
/// (line, col) position.
fn error_at(
    message: &str,
    line: usize,
    col: usize,
) -> GraphQLParseError {
    GraphQLParseError::new(
        message.to_string(),
        dummy_span_at(line, col),
        GraphQLParseErrorKind::InvalidSyntax,
    )
}

/// Returns the number of `compile_error` invocations in a token
/// stream string representation.
fn count_compile_errors(stream_str: &str) -> usize {
    stream_str.matches("compile_error").count()
}

// ── format_parse_error_message ───────────────────────────────────

/// Verifies that an error with no notes produces just the error
/// message itself, with no note annotations appended.
///
/// This is the simplest case: no iteration over notes occurs.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn format_message_no_notes() {
    let error = simple_error("unexpected token `{`");
    let result = format_parse_error_message(&error);
    assert_eq!(result, "unexpected token `{`");
}

/// Verifies that a General-kind note is formatted with the
/// "note:" prefix and the standard indentation pattern
/// `\n  = note: ...`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn format_message_with_general_note() {
    let mut error = simple_error("unexpected token `{`");
    error.add_note("expected a field name");
    let result = format_parse_error_message(&error);
    assert_eq!(
        result,
        "unexpected token `{`\n  = note: expected a field name",
    );
}

/// Verifies that a Help-kind note is formatted with the "help:"
/// prefix.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn format_message_with_help_note() {
    let mut error = simple_error("unexpected token `{`");
    error.add_help("try adding a field name before `{`");
    let result = format_parse_error_message(&error);
    assert_eq!(
        result,
        "unexpected token `{`\
         \n  = help: try adding a field name before `{`",
    );
}

/// Verifies that a Spec-kind note is formatted with the "spec:"
/// prefix. Spec notes typically contain a URL to the relevant
/// section of the GraphQL specification.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn format_message_with_spec_note() {
    let mut error = simple_error("invalid directive location");
    error.add_spec(
        "https://spec.graphql.org/September2025/#sec-Type-System\
         .Directives",
    );
    let result = format_parse_error_message(&error);
    assert_eq!(
        result,
        "invalid directive location\
         \n  = spec: https://spec.graphql.org/September2025/\
         #sec-Type-System.Directives",
    );
}

/// Verifies that multiple notes of different kinds are all
/// appended in order with their respective prefixes.
///
/// This exercises the iteration over `error.notes()` and confirms
/// that each note kind maps to the correct prefix string.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn format_message_with_mixed_notes() {
    let mut error = simple_error("duplicate field `name`");
    error.add_note("previous definition was here");
    error.add_help("remove or rename one of the fields");
    error.add_spec("https://spec.graphql.org/September2025/#Fields");
    let result = format_parse_error_message(&error);
    assert_eq!(
        result,
        "duplicate field `name`\
         \n  = note: previous definition was here\
         \n  = help: remove or rename one of the fields\
         \n  = spec: https://spec.graphql.org/September2025/\
         #Fields",
    );
}

/// Verifies that a note whose message contains special characters
/// (newlines, quotes) is included verbatim without escaping in the
/// formatted output.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn format_message_preserves_special_chars_in_note() {
    let mut error = simple_error("parse error");
    error.add_note("found `\"hello\"`");
    let result = format_parse_error_message(&error);
    assert!(
        result.contains("note: found `\"hello\"`"),
        "Note message should be included verbatim, got: {result}",
    );
}

// ── format_parse_error_note ──────────────────────────────────────

/// Verifies that a General-kind note formats as "note: {message}".
///
/// This function is used for secondary `compile_error!` messages
/// emitted at note span locations.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn format_note_general() {
    let note = GraphQLErrorNote::general(
        "previous definition was here".to_string(),
    );
    let result = format_parse_error_note(&note);
    assert_eq!(result, "note: previous definition was here");
}

/// Verifies that a Help-kind note formats as "help: {message}".
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn format_note_help() {
    let note = GraphQLErrorNote::help(
        "try removing the duplicate".to_string(),
    );
    let result = format_parse_error_note(&note);
    assert_eq!(result, "help: try removing the duplicate");
}

/// Verifies that a Spec-kind note formats as "spec: {url}".
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn format_note_spec() {
    let note = GraphQLErrorNote::spec(
        "https://spec.graphql.org/September2025/#sec-Names"
            .to_string(),
    );
    let result = format_parse_error_note(&note);
    assert_eq!(
        result,
        "spec: https://spec.graphql.org/September2025/#sec-Names",
    );
}

// ── convert_parse_errors_to_tokenstream ──────────────────────────

/// Verifies that an empty slice of errors produces an empty token
/// stream. This is important because the caller may have filtered
/// out all errors and should get a no-op result.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn convert_empty_errors_produces_empty_stream() {
    let errors: &[GraphQLParseError] = &[];
    let span_map = SpanMap::new(HashMap::new());
    let result = convert_parse_errors_to_tokenstream(
        errors, &span_map,
    );
    assert!(
        result.is_empty(),
        "Empty errors should produce empty token stream",
    );
}

/// Verifies that a single error with no notes produces exactly one
/// `compile_error!` invocation containing the error message.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn convert_single_error_no_notes() {
    let error = error_at("unexpected `}`", 0, 0);
    let mut map = HashMap::new();
    map.insert((0, 0), Span::call_site());
    let span_map = SpanMap::new(map);

    let result = convert_parse_errors_to_tokenstream(
        &[error],
        &span_map,
    );
    let output = result.to_string();

    assert_eq!(
        count_compile_errors(&output),
        1,
        "Expected exactly 1 compile_error!, got: {output}",
    );
    assert!(
        output.contains("unexpected `}`"),
        "compile_error! should contain the error message, \
         got: {output}",
    );
}

/// Verifies that when a note has a span that exists in the span
/// map, the converter emits an additional `compile_error!` at
/// the note's span location with the note-specific message.
///
/// This is the mechanism that gives users a secondary diagnostic
/// pointer (e.g. "note: previous definition was here" pointing at
/// the original definition site).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn convert_error_with_spanned_note_emits_secondary_compile_error() {
    let mut error = error_at("duplicate field `name`", 0, 0);
    error.add_note_with_span(
        "previous definition was here".to_string(),
        dummy_span_at(5, 4),
    );

    let mut map = HashMap::new();
    map.insert((0, 0), Span::call_site());
    map.insert((5, 4), Span::call_site());
    let span_map = SpanMap::new(map);

    let result = convert_parse_errors_to_tokenstream(
        &[error],
        &span_map,
    );
    let output = result.to_string();

    // Primary compile_error! + secondary compile_error! for the
    // spanned note.
    assert_eq!(
        count_compile_errors(&output),
        2,
        "Expected 2 compile_error! invocations (primary + note), \
         got: {output}",
    );
    assert!(
        output.contains("previous definition was here"),
        "Secondary compile_error! should contain the note message, \
         got: {output}",
    );
}

/// Verifies that notes WITHOUT a span do not produce an additional
/// `compile_error!`. They are still included inline in the primary
/// error message (tested by `format_message_*` tests), but should
/// not generate a secondary diagnostic pointer.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn convert_error_with_unspanned_note_no_extra_compile_error() {
    let mut error = error_at("type mismatch", 0, 0);
    error.add_note("expected `String`, found `Int`");

    let mut map = HashMap::new();
    map.insert((0, 0), Span::call_site());
    let span_map = SpanMap::new(map);

    let result = convert_parse_errors_to_tokenstream(
        &[error],
        &span_map,
    );
    let output = result.to_string();

    assert_eq!(
        count_compile_errors(&output),
        1,
        "Unspanned notes should NOT produce a secondary \
         compile_error!, got: {output}",
    );
    // But the note should still appear inline in the primary
    // message.
    assert!(
        output.contains("expected `String`, found `Int`"),
        "Note should appear inline in the primary message, \
         got: {output}",
    );
}

/// Verifies that when a note has a span but that span is NOT
/// present in the span map, no secondary `compile_error!` is
/// emitted for it. The converter skips notes whose spans cannot
/// be resolved.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn convert_error_with_spanned_note_missing_from_map_is_skipped() {
    let mut error = error_at("duplicate type", 0, 0);
    // Note span at (10, 2) — not in our span map.
    error.add_note_with_span(
        "first defined here".to_string(),
        dummy_span_at(10, 2),
    );

    let mut map = HashMap::new();
    map.insert((0, 0), Span::call_site());
    // Deliberately NOT inserting (10, 2).
    let span_map = SpanMap::new(map);

    let result = convert_parse_errors_to_tokenstream(
        &[error],
        &span_map,
    );
    let output = result.to_string();

    assert_eq!(
        count_compile_errors(&output),
        1,
        "Note with unmapped span should not produce a secondary \
         compile_error!, got: {output}",
    );
}

/// Verifies that when the primary error's span is not in the span
/// map, the converter still produces a `compile_error!` (falling
/// back to `Span::call_site()`). Errors must never be silently
/// dropped.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn convert_error_with_unmapped_primary_span_still_emits() {
    let error = error_at("syntax error", 99, 99);
    // Empty span map — the primary span (99, 99) won't be found.
    let span_map = SpanMap::new(HashMap::new());

    let result = convert_parse_errors_to_tokenstream(
        &[error],
        &span_map,
    );
    let output = result.to_string();

    assert_eq!(
        count_compile_errors(&output),
        1,
        "Error should still emit compile_error! even when span \
         lookup fails, got: {output}",
    );
    assert!(
        output.contains("syntax error"),
        "Message should be preserved even with fallback span, \
         got: {output}",
    );
}

/// Verifies that multiple errors each produce their own
/// `compile_error!` invocations.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn convert_multiple_errors() {
    let error1 = error_at("first error", 0, 0);
    let error2 = error_at("second error", 1, 0);
    let error3 = error_at("third error", 2, 0);

    let mut map = HashMap::new();
    map.insert((0, 0), Span::call_site());
    map.insert((1, 0), Span::call_site());
    map.insert((2, 0), Span::call_site());
    let span_map = SpanMap::new(map);

    let result = convert_parse_errors_to_tokenstream(
        &[error1, error2, error3],
        &span_map,
    );
    let output = result.to_string();

    assert_eq!(
        count_compile_errors(&output),
        3,
        "Each error should produce one compile_error!, \
         got: {output}",
    );
    assert!(output.contains("first error"));
    assert!(output.contains("second error"));
    assert!(output.contains("third error"));
}

/// Verifies that multiple errors, some with spanned notes and some
/// without, produce the correct total number of `compile_error!`
/// invocations: one per error plus one per note that has a
/// resolvable span.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn convert_multiple_errors_with_mixed_notes() {
    // Error 1: has a spanned note at (3, 0) → 1 primary + 1
    // secondary.
    let mut error1 = error_at("error one", 0, 0);
    error1.add_note_with_span(
        "note for error one".to_string(),
        dummy_span_at(3, 0),
    );

    // Error 2: has an unspanned note → 1 primary only.
    let mut error2 = error_at("error two", 1, 0);
    error2.add_help("some help");

    // Error 3: has two spanned notes at (6, 0) and (7, 0) → 1
    // primary + 2 secondary.
    let mut error3 = error_at("error three", 2, 0);
    error3.add_note_with_span(
        "first note".to_string(),
        dummy_span_at(6, 0),
    );
    error3.add_help_with_span(
        "second note".to_string(),
        dummy_span_at(7, 0),
    );

    let mut map = HashMap::new();
    map.insert((0, 0), Span::call_site());
    map.insert((1, 0), Span::call_site());
    map.insert((2, 0), Span::call_site());
    map.insert((3, 0), Span::call_site());
    map.insert((6, 0), Span::call_site());
    map.insert((7, 0), Span::call_site());
    let span_map = SpanMap::new(map);

    let result = convert_parse_errors_to_tokenstream(
        &[error1, error2, error3],
        &span_map,
    );
    let output = result.to_string();

    // 3 primary + 1 (error1 note) + 0 (error2 unspanned) + 2
    // (error3 notes) = 6
    assert_eq!(
        count_compile_errors(&output),
        6,
        "Expected 6 compile_error! invocations (3 primary + 3 \
         secondary from spanned notes), got: {output}",
    );
}
