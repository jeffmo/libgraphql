use crate::schema_source_map::LineCol;
use crate::schema_source_map::SchemaSourceMap;

// Verifies from_source() correctly computes line starts for a
// multi-line source with LF line endings.
// Written by Claude Code, reviewed by a human.
#[test]
fn line_starts_lf() {
    let source = "abc\ndef\nghi";
    let sm = SchemaSourceMap::from_source(source, None);
    // Lines start at byte offsets: 0 ("abc\n"), 4 ("def\n"), 8 ("ghi")
    assert_eq!(sm.line_starts, vec![0, 4, 8]);
}

// Verifies from_source() handles CRLF line endings.
// Written by Claude Code, reviewed by a human.
#[test]
fn line_starts_crlf() {
    let source = "abc\r\ndef\r\nghi";
    let sm = SchemaSourceMap::from_source(source, None);
    // Lines start at byte offsets: 0 ("abc\r\n"), 5 ("def\r\n"), 10 ("ghi")
    assert_eq!(sm.line_starts, vec![0, 5, 10]);
}

// Verifies from_source() handles bare CR line endings.
// Written by Claude Code, reviewed by a human.
#[test]
fn line_starts_cr() {
    let source = "abc\rdef\rghi";
    let sm = SchemaSourceMap::from_source(source, None);
    assert_eq!(sm.line_starts, vec![0, 4, 8]);
}

// Verifies resolve_offset() maps a byte offset to the correct
// 0-based line and column for ASCII content.
// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_offset_ascii() {
    let source = "type Query {\n  hello: String\n}";
    let sm = SchemaSourceMap::from_source(source, None);

    // Byte 0 -> line 0, col 0
    assert_eq!(
        sm.resolve_offset(0, Some(source)),
        LineCol { line: 0, col_linestart_byte_offset: 0, col_utf8: 0 },
    );

    // Byte 5 -> line 0, col 5 (the 'Q' in 'Query')
    assert_eq!(
        sm.resolve_offset(5, Some(source)),
        LineCol { line: 0, col_linestart_byte_offset: 5, col_utf8: 5 },
    );

    // Byte 15 -> line 1, col 2 (the 'h' in 'hello')
    assert_eq!(
        sm.resolve_offset(15, Some(source)),
        LineCol { line: 1, col_linestart_byte_offset: 2, col_utf8: 2 },
    );
}

// Verifies resolve_offset() correctly handles UTF-8 multi-byte
// characters where byte offset and character offset diverge.
// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_offset_utf8() {
    // "café" is 5 bytes (c=1, a=1, f=1, é=2), followed by newline
    let source = "café\nworld";
    let sm = SchemaSourceMap::from_source(source, None);

    // Byte 6 -> line 1, col byte 0, col utf8 0 (start of "world")
    assert_eq!(
        sm.resolve_offset(6, Some(source)),
        LineCol { line: 1, col_linestart_byte_offset: 0, col_utf8: 0 },
    );

    // Byte 3 -> line 0, col byte 3, col utf8 3 (the 'é' start)
    let result = sm.resolve_offset(3, Some(source));
    assert_eq!(result.line, 0);
    assert_eq!(result.col_linestart_byte_offset, 3);
    assert_eq!(result.col_utf8, 3);

    // Byte 5 -> line 0, col byte 5, but col utf8 4 (after 'é')
    let result = sm.resolve_offset(5, Some(source));
    assert_eq!(result.line, 0);
    assert_eq!(result.col_linestart_byte_offset, 5);
    assert_eq!(result.col_utf8, 4);
}

// Verifies resolve_offset() falls back to byte-offset columns
// when source text is unavailable (None).
// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_offset_no_source() {
    let source = "abc\ndef";
    let sm = SchemaSourceMap::from_source(source, None);

    let result = sm.resolve_offset(5, None);
    assert_eq!(result.line, 1);
    assert_eq!(result.col_linestart_byte_offset, 1);
    // Without source, col_utf8 falls back to byte offset
    assert_eq!(result.col_utf8, 1);
}

// Verifies builtin() creates a minimal source map.
// Written by Claude Code, reviewed by a human.
#[test]
fn builtin_source_map() {
    let sm = SchemaSourceMap::builtin();
    assert!(sm.file_path().is_none());
    assert_eq!(sm.line_starts, vec![0]);
}

// Verifies file_path() returns the stored path.
// Written by Claude Code, reviewed by a human.
#[test]
fn file_path_stored() {
    let sm = SchemaSourceMap::from_source(
        "type Query { x: Int }",
        Some("schema.graphql".into()),
    );
    assert_eq!(
        sm.file_path().unwrap().to_str().unwrap(),
        "schema.graphql",
    );
}

// Verifies serde round-trip via bincode for SchemaSourceMap.
// Written by Claude Code, reviewed by a human.
#[test]
fn schema_source_map_serde_roundtrip() {
    let sm = SchemaSourceMap::from_source(
        "abc\ndef",
        Some("test.graphql".into()),
    );
    let bytes = bincode::serde::encode_to_vec(
        &sm,
        bincode::config::standard(),
    ).unwrap();
    let (deserialized, _): (SchemaSourceMap, _) =
        bincode::serde::decode_from_slice(
            &bytes,
            bincode::config::standard(),
        ).unwrap();
    assert_eq!(sm, deserialized);
}
