//! Tests for the `SourcePosition` struct.
//!
//! These tests verify:
//! - Constructor creates positions with correct values
//! - Accessor methods return expected values
//! - Equality comparisons work correctly
//! - Conversion to `AstPos` (1-based) is correct

use crate::SourcePosition;

// =============================================================================
// Constructor tests
// =============================================================================

/// Verify that `SourcePosition::new(0, 0, Some(0), 0)` represents the very
/// start of a document (first character of first line).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_constructor_document_start() {
    let pos = SourcePosition::new(0, 0, Some(0), 0);
    assert_eq!(pos.line(), 0, "First line should be 0");
    assert_eq!(pos.col_utf8(), 0, "First column (UTF-8) should be 0");
    assert_eq!(
        pos.col_utf16(),
        Some(0),
        "First column (UTF-16) should be Some(0)"
    );
    assert_eq!(pos.byte_offset(), 0, "First byte offset should be 0");
}

/// Verify that `SourcePosition::new(1, 0, Some(0), 10)` represents the
/// first character of the second line.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_constructor_second_line_start() {
    let pos = SourcePosition::new(1, 0, Some(0), 10);
    assert_eq!(pos.line(), 1, "Second line should be 1");
    assert_eq!(
        pos.col_utf8(),
        0,
        "First char of line should have col_utf8 = 0"
    );
    assert_eq!(
        pos.col_utf16(),
        Some(0),
        "First char of line should have col_utf16 = Some(0)"
    );
    assert_eq!(
        pos.byte_offset(),
        10,
        "Byte offset should be 10 (assuming 10 bytes on first line)"
    );
}

/// Verify that `SourcePosition::new(0, 5, None, 5)` creates a position
/// with `col_utf16() == None`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_constructor_no_utf16_column() {
    let pos = SourcePosition::new(0, 5, None, 5);
    assert_eq!(pos.line(), 0);
    assert_eq!(pos.col_utf8(), 5);
    assert_eq!(
        pos.col_utf16(),
        None,
        "col_utf16 should be None when not provided"
    );
    assert_eq!(pos.byte_offset(), 5);
}

// =============================================================================
// Accessor tests
// =============================================================================

/// Verify all accessor methods return correct values for various positions.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_accessors_various_positions() {
    // Middle of a document
    let pos = SourcePosition::new(5, 10, Some(12), 150);
    assert_eq!(pos.line(), 5);
    assert_eq!(pos.col_utf8(), 10);
    assert_eq!(pos.col_utf16(), Some(12));
    assert_eq!(pos.byte_offset(), 150);

    // Large values
    let pos_large = SourcePosition::new(10000, 500, Some(600), 1_000_000);
    assert_eq!(pos_large.line(), 10000);
    assert_eq!(pos_large.col_utf8(), 500);
    assert_eq!(pos_large.col_utf16(), Some(600));
    assert_eq!(pos_large.byte_offset(), 1_000_000);
}

// =============================================================================
// Equality tests
// =============================================================================

/// Verify two positions with the same values are equal.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_equality_same_values() {
    let pos1 = SourcePosition::new(3, 7, Some(8), 42);
    let pos2 = SourcePosition::new(3, 7, Some(8), 42);
    assert_eq!(pos1, pos2, "Positions with same values should be equal");
}

/// Verify positions with different values are not equal.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_equality_different_values() {
    let base = SourcePosition::new(3, 7, Some(8), 42);

    // Different line
    let diff_line = SourcePosition::new(4, 7, Some(8), 42);
    assert_ne!(base, diff_line, "Different line should not be equal");

    // Different col_utf8
    let diff_col_utf8 = SourcePosition::new(3, 8, Some(8), 42);
    assert_ne!(base, diff_col_utf8, "Different col_utf8 should not be equal");

    // Different col_utf16
    let diff_col_utf16 = SourcePosition::new(3, 7, Some(9), 42);
    assert_ne!(
        base, diff_col_utf16,
        "Different col_utf16 should not be equal"
    );

    // None vs Some for col_utf16
    let none_col_utf16 = SourcePosition::new(3, 7, None, 42);
    assert_ne!(
        base, none_col_utf16,
        "None col_utf16 should not equal Some col_utf16"
    );

    // Different byte_offset
    let diff_byte_offset = SourcePosition::new(3, 7, Some(8), 43);
    assert_ne!(
        base, diff_byte_offset,
        "Different byte_offset should not be equal"
    );
}

// =============================================================================
// to_ast_pos() tests
// =============================================================================

/// Verify conversion to 1-based AstPos is correct.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_to_ast_pos_basic() {
    // 0-based (0, 0) should become 1-based (1, 1)
    let pos = SourcePosition::new(0, 0, Some(0), 0);
    let ast_pos = pos.to_ast_pos();
    assert_eq!(ast_pos.line, 1, "AstPos line should be 1-based (0 -> 1)");
    assert_eq!(
        ast_pos.column, 1,
        "AstPos column should be 1-based (0 -> 1)"
    );

    // 0-based (5, 10) should become 1-based (6, 11)
    let pos2 = SourcePosition::new(5, 10, Some(12), 100);
    let ast_pos2 = pos2.to_ast_pos();
    assert_eq!(ast_pos2.line, 6, "AstPos line should be 5 + 1 = 6");
    assert_eq!(ast_pos2.column, 11, "AstPos column should be 10 + 1 = 11");
}

/// Verify `to_ast_pos()` always uses `col_utf8` for the column value,
/// not `col_utf16`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_to_ast_pos_uses_col_utf8() {
    // Create position where col_utf8 != col_utf16
    // This simulates text with characters outside BMP
    let pos = SourcePosition::new(0, 5, Some(7), 10);
    let ast_pos = pos.to_ast_pos();

    // Should use col_utf8 (5), not col_utf16 (7)
    assert_eq!(
        ast_pos.column, 6,
        "AstPos column should be col_utf8 + 1 = 6, not col_utf16 + 1 = 8"
    );
}

/// Verify `to_ast_pos()` works correctly when `col_utf16` is None.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_to_ast_pos_with_none_utf16() {
    let pos = SourcePosition::new(2, 15, None, 50);
    let ast_pos = pos.to_ast_pos();

    assert_eq!(ast_pos.line, 3, "AstPos line should be 2 + 1 = 3");
    assert_eq!(ast_pos.column, 16, "AstPos column should be 15 + 1 = 16");
}

// =============================================================================
// Clone tests
// =============================================================================

/// Verify SourcePosition can be cloned.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_clone() {
    let pos = SourcePosition::new(1, 2, Some(3), 4);
    let cloned = pos.clone();
    assert_eq!(pos, cloned, "Cloned position should equal original");
}

// =============================================================================
// Debug tests
// =============================================================================

/// Verify Debug implementation works (useful for error messages).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_debug() {
    let pos = SourcePosition::new(1, 2, Some(3), 4);
    let debug_str = format!("{pos:?}");
    assert!(
        debug_str.contains("SourcePosition"),
        "Debug output should contain struct name"
    );
    assert!(
        debug_str.contains("line: 1"),
        "Debug output should contain line value"
    );
}
