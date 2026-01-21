//! Tests for `ParseResult` state management and conversion.
//!
//! ParseResult is a tri-state type that can represent:
//! 1. Complete success: AST present, no errors
//! 2. Recovered parse: AST present, errors present (partial/best-effort)
//! 3. Complete failure: No AST, errors present
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
// Part 3.1: ParseResult State Tests
// =============================================================================

/// Verifies that `ParseResult::ok()` creates a successful result with AST and
/// no errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_ok_creates_success_state() {
    let result: ParseResult<String> = ParseResult::ok("test value".to_string());

    assert!(result.is_ok(), "ok() should create successful state");
    assert!(!result.has_errors(), "ok() should have no errors");
    assert!(result.errors.is_empty(), "ok() errors vec should be empty");
    assert_eq!(result.ast(), Some(&"test value".to_string()));
    assert_eq!(result.valid_ast(), Some(&"test value".to_string()));
}

/// Verifies that `ParseResult::err()` creates a failed state with no AST.
///
/// This tests the case where parsing completely fails and no AST can be
/// produced (even partially).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_err_creates_failed_state() {
    let errors = vec![test_error("error 1"), test_error("error 2")];
    let result: ParseResult<String> = ParseResult::err(errors);

    assert!(!result.is_ok(), "err() should not be ok");
    assert!(result.has_errors(), "err() should have errors");
    assert_eq!(result.errors.len(), 2);
    assert!(result.ast().is_none(), "err() should have no AST");
    assert!(result.valid_ast().is_none(), "err() should have no valid AST");
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

    assert!(!result.is_ok(), "recovered() should not be fully ok");
    assert!(result.has_errors(), "recovered() should have errors");
    assert_eq!(result.errors.len(), 1);
    assert!(
        result.ast().is_some(),
        "recovered() should have AST available"
    );
    assert!(
        result.valid_ast().is_none(),
        "recovered() should NOT have valid_ast (due to errors)"
    );
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
        "valid_ast() should return None when errors present"
    );
}

/// Verifies that `ast()` returns Some even when errors are present.
///
/// For IDE tooling that wants best-effort results, `ast()` provides access
/// to the recovered AST regardless of errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_ast_returns_some_when_recovered() {
    let errors = vec![test_error("error")];
    let result: ParseResult<String> =
        ParseResult::recovered("recovered value".to_string(), errors);

    assert_eq!(result.ast(), Some(&"recovered value".to_string()));
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
/// regardless of errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_into_ast_consumes() {
    // Success case
    let result: ParseResult<String> = ParseResult::ok("value".to_string());
    let ast = result.into_ast();
    assert_eq!(ast, Some("value".to_string()));

    // Recovered case - still returns AST
    let errors = vec![test_error("error")];
    let result: ParseResult<String> =
        ParseResult::recovered("recovered".to_string(), errors);
    let ast = result.into_ast();
    assert_eq!(ast, Some("recovered".to_string()));

    // Error case - no AST
    let errors = vec![test_error("error")];
    let result: ParseResult<String> = ParseResult::err(errors);
    let ast = result.into_ast();
    assert!(ast.is_none());
}

/// Verifies that `is_ok()` correctly identifies complete success.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_is_ok_checks_both_ast_and_errors() {
    // Success: AST present, no errors
    let result: ParseResult<String> = ParseResult::ok("value".to_string());
    assert!(result.is_ok());

    // Recovered: AST present, errors present -> not ok
    let errors = vec![test_error("error")];
    let result: ParseResult<String> =
        ParseResult::recovered("value".to_string(), errors);
    assert!(!result.is_ok());

    // Failed: no AST, errors present -> not ok
    let errors = vec![test_error("error")];
    let result: ParseResult<String> = ParseResult::err(errors);
    assert!(!result.is_ok());
}

/// Verifies that `has_errors()` correctly detects error presence.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_has_errors_checks_errors_vec() {
    // No errors
    let result: ParseResult<String> = ParseResult::ok("value".to_string());
    assert!(!result.has_errors());

    // Has errors
    let errors = vec![test_error("error")];
    let result: ParseResult<String> = ParseResult::err(errors);
    assert!(result.has_errors());
}

/// Verifies that `format_errors()` aggregates and formats multiple errors.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_format_errors() {
    let errors = vec![test_error("first error"), test_error("second error")];
    let result: ParseResult<String> = ParseResult::err(errors);

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
    let result: ParseResult<String> = ParseResult::err(errors);

    let formatted = result.format_errors(Some("type Query { field: String }"));

    // Should include the error message and source context
    assert!(formatted.contains("error here"));
}

// =============================================================================
// Part 3.1: From<ParseResult> Conversion Tests
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

/// Verifies that `From<ParseResult>` converts Failed state to `Result::Err`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_from_conversion_err() {
    let errors = vec![test_error("error 1"), test_error("error 2")];
    let parse_result: ParseResult<String> = ParseResult::err(errors);
    let std_result: Result<String, Vec<GraphQLParseError>> = parse_result.into();

    assert!(std_result.is_err());
    let err_vec = std_result.unwrap_err();
    assert_eq!(err_vec.len(), 2);
}

/// Verifies edge case: empty errors vec with no AST (unusual but valid).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_result_from_conversion_empty_errors_no_ast() {
    // This is an edge case: err() with empty errors
    let parse_result: ParseResult<String> = ParseResult::err(Vec::new());
    let std_result: Result<String, Vec<GraphQLParseError>> = parse_result.into();

    // No errors and no AST - should be Err with empty vec
    assert!(std_result.is_err());
    assert!(std_result.unwrap_err().is_empty());
}
