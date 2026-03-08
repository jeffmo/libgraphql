// SourceMap unit tests.
// Written by Claude Code, reviewed by a human.

use crate::ByteSpan;
use crate::SourceMap;
use crate::SourcePosition;
use std::path::PathBuf;

// ── Source-text mode: line_starts computation ───────────

/// Empty string should have one line starting at offset 0.
#[test]
fn source_text_empty_string() {
    let sm = SourceMap::new_with_source("", None);
    let pos = sm.resolve_offset(0);
    assert_eq!(pos, Some(SourcePosition::new(0, 0, Some(0), 0)));
}

/// Single line with no trailing newline.
#[test]
fn source_text_single_line_ascii() {
    let sm = SourceMap::new_with_source("hello", None);

    let pos = sm.resolve_offset(0).unwrap();
    assert_eq!(pos.line(), 0);
    assert_eq!(pos.col_utf8(), 0);
    assert_eq!(pos.col_utf16(), Some(0));
    assert_eq!(pos.byte_offset(), 0);

    let pos = sm.resolve_offset(3).unwrap();
    assert_eq!(pos.line(), 0);
    assert_eq!(pos.col_utf8(), 3);
    assert_eq!(pos.col_utf16(), Some(3));
    assert_eq!(pos.byte_offset(), 3);

    // At end of string (one past last char)
    let pos = sm.resolve_offset(5).unwrap();
    assert_eq!(pos.line(), 0);
    assert_eq!(pos.col_utf8(), 5);
    assert_eq!(pos.byte_offset(), 5);
}

/// Multi-line with \n terminators.
#[test]
fn source_text_multiline_lf() {
    let src = "abc\ndef\nghi";
    let sm = SourceMap::new_with_source(src, None);

    // 'a' at offset 0
    let pos = sm.resolve_offset(0).unwrap();
    assert_eq!(pos.line(), 0);
    assert_eq!(pos.col_utf8(), 0);

    // 'd' at offset 4 (after "abc\n")
    let pos = sm.resolve_offset(4).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 0);

    // 'e' at offset 5
    let pos = sm.resolve_offset(5).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 1);

    // 'g' at offset 8 (after "abc\ndef\n")
    let pos = sm.resolve_offset(8).unwrap();
    assert_eq!(pos.line(), 2);
    assert_eq!(pos.col_utf8(), 0);
}

/// Multi-line with \r terminators (classic Mac).
#[test]
fn source_text_multiline_cr() {
    let src = "ab\rcd\ref";
    let sm = SourceMap::new_with_source(src, None);

    let pos = sm.resolve_offset(0).unwrap();
    assert_eq!(pos.line(), 0);
    assert_eq!(pos.col_utf8(), 0);

    // 'c' at offset 3 (after "ab\r")
    let pos = sm.resolve_offset(3).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 0);

    // 'e' at offset 6 (after "ab\rcd\r")
    let pos = sm.resolve_offset(6).unwrap();
    assert_eq!(pos.line(), 2);
    assert_eq!(pos.col_utf8(), 0);
}

/// Multi-line with \r\n terminators (Windows).
#[test]
fn source_text_multiline_crlf() {
    let src = "ab\r\ncd\r\nef";
    let sm = SourceMap::new_with_source(src, None);

    let pos = sm.resolve_offset(0).unwrap();
    assert_eq!(pos.line(), 0);

    // 'c' at offset 4 (after "ab\r\n")
    let pos = sm.resolve_offset(4).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 0);

    // 'e' at offset 8 (after "ab\r\ncd\r\n")
    let pos = sm.resolve_offset(8).unwrap();
    assert_eq!(pos.line(), 2);
    assert_eq!(pos.col_utf8(), 0);
}

/// Mixed line terminators: \n, \r, \r\n in the same source.
#[test]
fn source_text_mixed_line_terminators() {
    let src = "a\nb\rc\r\nd";
    let sm = SourceMap::new_with_source(src, None);

    // Line 0: "a"
    let pos = sm.resolve_offset(0).unwrap();
    assert_eq!(pos.line(), 0);

    // Line 1: "b" (after \n at offset 1)
    let pos = sm.resolve_offset(2).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 0);

    // Line 2: "c" (after \r at offset 3)
    let pos = sm.resolve_offset(4).unwrap();
    assert_eq!(pos.line(), 2);
    assert_eq!(pos.col_utf8(), 0);

    // Line 3: "d" (after \r\n at offsets 5-6)
    let pos = sm.resolve_offset(7).unwrap();
    assert_eq!(pos.line(), 3);
    assert_eq!(pos.col_utf8(), 0);
}

// ── Source-text mode: Unicode column handling ────────────

/// BOM (U+FEFF) at start of file: 3 UTF-8 bytes, 1 scalar value, 1
/// UTF-16 code unit.
#[test]
fn source_text_bom_at_start() {
    let src = "\u{FEFF}hello";
    let sm = SourceMap::new_with_source(src, None);

    // After BOM (3 bytes), 'h' is at byte offset 3
    let pos = sm.resolve_offset(3).unwrap();
    assert_eq!(pos.line(), 0);
    assert_eq!(pos.col_utf8(), 1); // BOM counts as 1 scalar value
    assert_eq!(pos.col_utf16(), Some(1)); // BOM is 1 UTF-16 unit
    assert_eq!(pos.byte_offset(), 3);
}

/// Emoji: 4-byte UTF-8 character, 2 UTF-16 code units.
#[test]
fn source_text_emoji() {
    // 🎉 is U+1F389: 4 UTF-8 bytes, 2 UTF-16 code units
    let src = "a🎉b";
    let sm = SourceMap::new_with_source(src, None);

    // 'a' at offset 0
    let pos = sm.resolve_offset(0).unwrap();
    assert_eq!(pos.col_utf8(), 0);
    assert_eq!(pos.col_utf16(), Some(0));

    // '🎉' at offset 1
    let pos = sm.resolve_offset(1).unwrap();
    assert_eq!(pos.col_utf8(), 1);
    assert_eq!(pos.col_utf16(), Some(1));

    // 'b' at offset 5 (1 + 4 bytes for emoji)
    let pos = sm.resolve_offset(5).unwrap();
    assert_eq!(pos.col_utf8(), 2); // 2 scalar values before 'b'
    assert_eq!(pos.col_utf16(), Some(3)); // 1 + 2 UTF-16 units before 'b'
}

/// CJK character: 3-byte UTF-8, 1 UTF-16 code unit.
#[test]
fn source_text_cjk() {
    // 中 is U+4E2D: 3 UTF-8 bytes, 1 UTF-16 code unit
    let src = "a中b";
    let sm = SourceMap::new_with_source(src, None);

    // 'b' at offset 4 (1 + 3 bytes for CJK char)
    let pos = sm.resolve_offset(4).unwrap();
    assert_eq!(pos.col_utf8(), 2);
    assert_eq!(pos.col_utf16(), Some(2));
}

/// Accented character (precomposed): 2-byte UTF-8, 1 UTF-16 code unit.
#[test]
fn source_text_accented_char() {
    // é is U+00E9: 2 UTF-8 bytes, 1 UTF-16 code unit
    let src = "café";
    let sm = SourceMap::new_with_source(src, None);

    // 'f' at offset 4 (c=1, a=1, f=1, é=2 bytes... wait)
    // Actually: c(1) a(1) f(1) é(2) = "café" is 5 bytes
    // 'é' starts at offset 3
    let pos = sm.resolve_offset(3).unwrap();
    assert_eq!(pos.col_utf8(), 3); // c, a, f before it
    assert_eq!(pos.col_utf16(), Some(3));
}

// ── Source-text mode: edge cases ────────────────────────

/// Offset at exact line boundary (the \n character itself).
#[test]
fn source_text_offset_at_newline() {
    let src = "ab\ncd";
    let sm = SourceMap::new_with_source(src, None);

    // The \n at offset 2 is still on line 0
    let pos = sm.resolve_offset(2).unwrap();
    assert_eq!(pos.line(), 0);
    assert_eq!(pos.col_utf8(), 2);
}

/// Offset at EOF (one past last byte).
#[test]
fn source_text_offset_at_eof() {
    let src = "abc";
    let sm = SourceMap::new_with_source(src, None);

    let pos = sm.resolve_offset(3).unwrap();
    assert_eq!(pos.line(), 0);
    assert_eq!(pos.col_utf8(), 3);
    assert_eq!(pos.byte_offset(), 3);
}

/// Offset beyond source returns None.
#[test]
fn source_text_offset_out_of_bounds() {
    let src = "abc";
    let sm = SourceMap::new_with_source(src, None);
    assert_eq!(sm.resolve_offset(4), None);
}

/// Trailing newline: "abc\n" has 2 lines, second is empty.
#[test]
fn source_text_trailing_newline() {
    let src = "abc\n";
    let sm = SourceMap::new_with_source(src, None);

    // Offset 4 is on line 1, col 0 (the empty line after \n)
    let pos = sm.resolve_offset(4).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 0);
}

// ── Source-text mode: resolve_span ──────────────────────

/// resolve_span resolves both endpoints and attaches file_path.
#[test]
fn source_text_resolve_span() {
    let src = "abc\ndef";
    let path = PathBuf::from("test.graphql");
    let sm = SourceMap::new_with_source(src, Some(path.clone()));

    let span = ByteSpan::new(0, 7);
    let resolved = sm.resolve_span(span).unwrap();

    assert_eq!(resolved.start_inclusive.line(), 0);
    assert_eq!(resolved.start_inclusive.col_utf8(), 0);
    assert_eq!(resolved.end_exclusive.line(), 1);
    assert_eq!(resolved.end_exclusive.col_utf8(), 3);
    assert_eq!(resolved.file_path, Some(path));
}

/// resolve_span with no file_path.
#[test]
fn source_text_resolve_span_no_file() {
    let src = "hello";
    let sm = SourceMap::new_with_source(src, None);

    let span = ByteSpan::new(1, 4);
    let resolved = sm.resolve_span(span).unwrap();

    assert_eq!(resolved.start_inclusive.col_utf8(), 1);
    assert_eq!(resolved.end_exclusive.col_utf8(), 4);
    assert_eq!(resolved.file_path, None);
}

/// resolve_span returns None if either endpoint is out of bounds.
#[test]
fn source_text_resolve_span_out_of_bounds() {
    let src = "abc";
    let sm = SourceMap::new_with_source(src, None);

    // End is out of bounds
    assert!(sm.resolve_span(ByteSpan::new(0, 10)).is_none());

    // Start is out of bounds
    assert!(sm.resolve_span(ByteSpan::new(10, 20)).is_none());
}

// ── Pre-computed columns mode ───────────────────────────

/// Basic pre-computed mode: insert positions and look them up.
#[test]
fn precomputed_basic_lookup() {
    let mut sm = SourceMap::new_precomputed(None);
    sm.insert_computed_position(
        0,
        SourcePosition::new(0, 0, Some(0), 0),
    );
    sm.insert_computed_position(
        5,
        SourcePosition::new(0, 5, Some(5), 5),
    );
    sm.insert_computed_position(
        10,
        SourcePosition::new(1, 0, Some(0), 10),
    );

    // Exact match on offset 5
    let pos = sm.resolve_offset(5).unwrap();
    assert_eq!(pos.col_utf8(), 5);

    // Floor lookup: offset 7 falls between entries 5 and 10,
    // returns entry at offset 5
    let pos = sm.resolve_offset(7).unwrap();
    assert_eq!(pos.col_utf8(), 5);
    assert_eq!(pos.byte_offset(), 5);

    // Exact match on offset 10
    let pos = sm.resolve_offset(10).unwrap();
    assert_eq!(pos.line(), 1);
    assert_eq!(pos.col_utf8(), 0);
}

/// Pre-computed mode with no entries returns None.
#[test]
fn precomputed_empty_returns_none() {
    let sm = SourceMap::new_precomputed(None);
    assert_eq!(sm.resolve_offset(0), None);
}

/// Pre-computed mode: offset before first entry returns None.
#[test]
fn precomputed_before_first_entry_returns_none() {
    let mut sm = SourceMap::new_precomputed(None);
    sm.insert_computed_position(
        10,
        SourcePosition::new(1, 0, None, 10),
    );

    // Offset 5 is before the first entry at 10
    assert_eq!(sm.resolve_offset(5), None);
}

/// Pre-computed mode: UTF-16 columns are whatever the token source
/// provided.
#[test]
fn precomputed_preserves_utf16_columns() {
    let mut sm = SourceMap::new_precomputed(None);
    sm.insert_computed_position(
        0,
        SourcePosition::new(0, 0, Some(0), 0),
    );
    sm.insert_computed_position(
        3,
        SourcePosition::new(0, 2, Some(3), 3),
    );

    let pos = sm.resolve_offset(3).unwrap();
    assert_eq!(pos.col_utf16(), Some(3));
}

/// Pre-computed mode: None for col_utf16 when token source can't
/// provide it.
#[test]
fn precomputed_no_utf16() {
    let mut sm = SourceMap::new_precomputed(None);
    sm.insert_computed_position(
        0,
        SourcePosition::new(0, 0, None, 0),
    );

    let pos = sm.resolve_offset(0).unwrap();
    assert_eq!(pos.col_utf16(), None);
}

/// Pre-computed resolve_span works end-to-end.
#[test]
fn precomputed_resolve_span() {
    let path = PathBuf::from("macro_input.rs");
    let mut sm = SourceMap::new_precomputed(Some(path.clone()));
    sm.insert_computed_position(
        0,
        SourcePosition::new(0, 0, None, 0),
    );
    sm.insert_computed_position(
        5,
        SourcePosition::new(0, 5, None, 5),
    );

    let resolved = sm.resolve_span(ByteSpan::new(0, 5)).unwrap();
    assert_eq!(resolved.start_inclusive.col_utf8(), 0);
    assert_eq!(resolved.end_exclusive.col_utf8(), 5);
    assert_eq!(resolved.file_path, Some(path));
}

// ── Accessor methods ────────────────────────────────────

/// source() returns Some for source-text mode, None for pre-computed.
#[test]
fn source_accessor() {
    let src = "hello";
    let sm_source = SourceMap::new_with_source(src, None);
    assert_eq!(sm_source.source(), Some("hello"));

    let sm_precomputed = SourceMap::new_precomputed(None);
    assert_eq!(sm_precomputed.source(), None);
}

/// file_path() returns the path when provided.
#[test]
fn file_path_accessor() {
    let path = PathBuf::from("schema.graphql");
    let sm = SourceMap::new_with_source(
        "",
        Some(path.clone()),
    );
    assert_eq!(sm.file_path(), Some(path.as_path()));

    let sm_no_path = SourceMap::new_with_source("", None);
    assert_eq!(sm_no_path.file_path(), None);
}

// ── Round-trip validation ───────────────────────────────

/// Verifies that SourceMap resolves every token byte offset
/// produced by the lexer to a valid SourcePosition, and that
/// start <= end for each token's span.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn source_map_resolves_all_lexer_byte_offsets() {
    use crate::token_source::GraphQLTokenSource;
    use crate::token_source::StrGraphQLTokenSource;

    let sources = &[
        "type Query { hello: String }",
        "type Query {\n  hello: String\n  world: Int\n}",
        "# comment\ntype Foo { bar: ID! }",
        "\"description\"\ntype Bar { baz: [String!]! }",
        "type T {\n  emoji: String # 🎉\n}",
        "type T {\r\n  field: Int\r\n}",
        "type T {\r  field: Int\r}",
    ];

    for &src in sources {
        let (tokens, source_map) =
            StrGraphQLTokenSource::new(src).collect_with_source_map();

        for token in &tokens {
            let start = source_map
                .resolve_offset(token.span.start)
                .unwrap_or_else(|| {
                    panic!(
                        "start offset {} should resolve in {:?}",
                        token.span.start, src,
                    )
                });
            let end = source_map
                .resolve_offset(token.span.end)
                .unwrap_or_else(|| {
                    panic!(
                        "end offset {} should resolve in {:?}",
                        token.span.end, src,
                    )
                });

            // Start position must be <= end position
            assert!(
                start.line() < end.line()
                    || (start.line() == end.line()
                        && start.col_utf8() <= end.col_utf8()),
                "start ({},{}) must be <= end ({},{}) for token \
                 at bytes {}..{} in {:?}",
                start.line(),
                start.col_utf8(),
                end.line(),
                end.col_utf8(),
                token.span.start,
                token.span.end,
                src,
            );

            // UTF-16 columns should always be present in
            // source-text mode
            assert!(
                start.col_utf16().is_some(),
                "UTF-16 col missing for start at byte {} in {:?}",
                token.span.start,
                src,
            );
            assert!(
                end.col_utf16().is_some(),
                "UTF-16 col missing for end at byte {} in {:?}",
                token.span.end,
                src,
            );
        }
    }
}
