//! Tests for `ParseResult` state management and conversion.
//!
//! ParseResult is a two-variant enum:
//! 1. `Ok(TAst)` — Complete success: AST present, no errors
//! 2. `Recovered { ast, errors }` — Recovered parse: AST present, errors present
//!
//! An AST is always present — there is no "complete failure" variant.
//!
//! Written by Claude Code, reviewed by a human.

use crate::GraphQLParseError;
use crate::GraphQLParseErrorKind;
use crate::GraphQLSourceSpan;
use crate::ParseResult;
use crate::SourcePosition;

/// Helper to create a test span for error construction.
fn test_span() -> GraphQLSourceSpan {
    GraphQLSourceSpan::new(
        SourcePosition::new(0, 0, Some(0), 0),
        SourcePosition::new(0, 1, Some(1), 1),
    )
}

/// Helper to create a test error.
fn test_error(message: &str) -> GraphQLParseError {
    GraphQLParseError::new(
        message,
        test_span(),
        GraphQLParseErrorKind::UnexpectedToken {
            expected: vec!["test".to_string()],
            found: "other".to_string(),
        },
    )
}

// =============================================================================
// ParseResult State Tests
// =============================================================================

/// Verifies that `ParseResult::ok()` creates a successful result with AST and
/// no errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_ok_creates_success_state() {
    let result: ParseResult<String> = ParseResult::ok("test value".to_string());

    assert!(!result.has_errors(), "ok() should have no errors");
    assert!(result.errors().is_empty(), "ok() errors slice should be empty");
    assert_eq!(result.valid_ast(), Some(&"test value".to_string()));
}

/// Verifies that `ParseResult::recovered()` creates a state with both AST
/// and errors.
///
/// This represents error recovery: parsing encountered errors but was able
/// to produce a partial/best-effort AST.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_recovered_creates_mixed_state() {
    let errors = vec![test_error("syntax error")];
    let result: ParseResult<String> =
        ParseResult::recovered("partial result".to_string(), errors);

    assert!(result.has_errors(), "recovered() should have errors");
    assert_eq!(result.errors().len(), 1);
    assert_eq!(result.into_ast(), "partial result".to_string());
}

/// Verifies that `valid_ast()` returns None when errors are present.
///
/// Even if an AST was produced via error recovery, `valid_ast()` should
/// return None to indicate the AST is not guaranteed to be correct.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_valid_ast_returns_none_when_errors() {
    let errors = vec![test_error("some error")];
    let result: ParseResult<String> =
        ParseResult::recovered("value".to_string(), errors);

    assert!(
        result.valid_ast().is_none(),
        "valid_ast() should return None when errors present",
    );
}

/// Verifies that `into_valid_ast()` consumes the result and returns the AST
/// only when there are no errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_into_valid_ast_consumes() {
    let result: ParseResult<String> = ParseResult::ok("value".to_string());
    let ast = result.into_valid_ast();
    assert_eq!(ast, Some("value".to_string()));

    // With errors, should return None
    let errors = vec![test_error("error")];
    let result: ParseResult<String> =
        ParseResult::recovered("value".to_string(), errors);
    let ast = result.into_valid_ast();
    assert!(ast.is_none());
}

/// Verifies that `into_ast()` consumes the result and returns the AST
/// unconditionally.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_into_ast_consumes() {
    // Success case
    let result: ParseResult<String> = ParseResult::ok("value".to_string());
    let ast = result.into_ast();
    assert_eq!(ast, "value".to_string());

    // Recovered case — still returns AST
    let errors = vec![test_error("error")];
    let result: ParseResult<String> =
        ParseResult::recovered("recovered".to_string(), errors);
    let ast = result.into_ast();
    assert_eq!(ast, "recovered".to_string());
}

/// Verifies that `has_errors()` correctly detects the Recovered variant.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_has_errors_checks_variant() {
    // No errors (Ok)
    let result: ParseResult<String> = ParseResult::ok("value".to_string());
    assert!(!result.has_errors());

    // Has errors (Recovered)
    let errors = vec![test_error("error")];
    let result: ParseResult<String> =
        ParseResult::recovered("value".to_string(), errors);
    assert!(result.has_errors());
}

/// Verifies that `errors()` returns the correct slice for each variant.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_errors_returns_correct_slice() {
    // Ok variant returns empty slice
    let result: ParseResult<String> = ParseResult::ok("value".to_string());
    assert!(result.errors().is_empty());

    // Recovered variant returns non-empty slice
    let errors = vec![test_error("first"), test_error("second")];
    let result: ParseResult<String> =
        ParseResult::recovered("value".to_string(), errors);
    assert_eq!(result.errors().len(), 2);
}

/// Verifies that `format_errors()` aggregates and formats multiple errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_format_errors() {
    let errors = vec![test_error("first error"), test_error("second error")];
    let result: ParseResult<String> =
        ParseResult::recovered("value".to_string(), errors);

    let formatted = result.format_errors(None);

    assert!(formatted.contains("first error"));
    assert!(formatted.contains("second error"));
}

/// Verifies that `format_errors()` with source provides source snippets.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_format_errors_with_source() {
    let errors = vec![test_error("error here")];
    let result: ParseResult<String> =
        ParseResult::recovered("value".to_string(), errors);

    let formatted = result.format_errors(Some("type Query { field: String }"));

    // Should include the error message and source context
    assert!(formatted.contains("error here"));
}

/// Verifies that `format_errors()` returns an empty string for Ok results.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_format_errors_empty_for_ok() {
    let result: ParseResult<String> = ParseResult::ok("value".to_string());

    let formatted = result.format_errors(None);
    assert!(formatted.is_empty());
}

/// Verifies the debug_assert invariant: `recovered()` with empty errors
/// should panic in debug builds.
///
/// Written by Claude Code, reviewed by a human.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "ParseResult::recovered() called with empty errors vec")]
fn parse_result_recovered_empty_errors_panics_in_debug() {
    let _result: ParseResult<String> =
        ParseResult::recovered("value".to_string(), Vec::new());
}

// =============================================================================
// From<ParseResult> Conversion Tests
// =============================================================================

/// Verifies that `From<ParseResult>` converts Ok state to `Result::Ok`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_from_conversion_ok() {
    let parse_result: ParseResult<String> = ParseResult::ok("value".to_string());
    let std_result: Result<String, Vec<GraphQLParseError>> = parse_result.into();

    assert!(std_result.is_ok());
    assert_eq!(std_result.unwrap(), "value");
}

/// Verifies that `From<ParseResult>` converts Recovered state to `Result::Err`.
///
/// Even though a recovered AST exists, the conversion treats it as an error
/// because the AST may be incomplete or incorrect.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_from_conversion_recovered() {
    let errors = vec![test_error("error")];
    let parse_result: ParseResult<String> =
        ParseResult::recovered("value".to_string(), errors);
    let std_result: Result<String, Vec<GraphQLParseError>> = parse_result.into();

    assert!(std_result.is_err());
    let err_vec = std_result.unwrap_err();
    assert_eq!(err_vec.len(), 1);
}

/// Verifies that `From<ParseResult>` preserves multiple errors in Recovered
/// state.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_from_conversion_recovered_multiple_errors() {
    let errors = vec![test_error("error 1"), test_error("error 2")];
    let parse_result: ParseResult<String> =
        ParseResult::recovered("value".to_string(), errors);
    let std_result: Result<String, Vec<GraphQLParseError>> = parse_result.into();

    assert!(std_result.is_err());
    let err_vec = std_result.unwrap_err();
    assert_eq!(err_vec.len(), 2);
}
