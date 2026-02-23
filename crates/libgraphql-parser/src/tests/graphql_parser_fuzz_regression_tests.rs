//! Fuzz regression tests for `GraphQLParser` and `StrGraphQLTokenSource`.
//!
//! Each test in this module reproduces a crash or hang discovered by
//! `cargo-fuzz`. Tests are grouped by bug category:
//!
//! 1. OOM / infinite loop — error recovery failed to consume tokens
//! 2. Invalid description string — `parse_description()` left tokens
//!    unconsumed on invalid escape sequences
//! 3. Catch-all branch — top-level catch-all branches did not consume
//!    unexpected tokens
//! 4. Stack overflow — unbounded recursion on deeply nested input
//! 5. Block string panic — byte-index panic from Unicode whitespace
//!    mismatch in `parse_block_string`
//!
//! Written by Claude Code, reviewed by a human.

use crate::tests::utils::parse_executable;
use crate::tests::utils::parse_mixed;
use crate::tests::utils::parse_schema;
use crate::GraphQLParser;

// =============================================================================
// OOM / infinite loop regressions
// =============================================================================

/// Regression test: a `$` variable reference inside a list value in a
/// const context (such as a directive argument default) must not cause an
/// infinite loop / OOM.
///
/// Previously, `parse_value` returned `Err` without consuming the `$`
/// token when variables were disallowed. The list-value recovery loop
/// treated `$` as a value-starter and restarted parsing, producing an
/// unbounded number of error objects until memory was exhausted.
///
/// The fix ensures `$` is consumed before returning the error so the
/// recovery loop always makes forward progress.
///
/// Discovered via cargo-fuzz (fuzz_parse_schema target). Minimized
/// reproducer: `e\0\0directive @dce(n:Sg =[g s$t`
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fuzz_regression_variable_in_const_list_no_oom() {
    // The original minimized fuzz input (30 bytes)
    let input = "e\0\0directive @dce(n:Sg =[g s$t";
    let result = parse_schema(input);
    assert!(result.has_errors());

    // A cleaner variant exercising the same code path: a `$var`
    // inside a list default value in a directive argument definition.
    let clean_input =
        "directive @d(arg: [String] = [$x]) on FIELD";
    let clean_result = parse_schema(clean_input);
    assert!(clean_result.has_errors());
}

/// Regression test: a schema-definition keyword (like `type`) in an
/// executable document must not cause an infinite loop / OOM.
///
/// Previously, `parse_executable_definition_item` returned `Err`
/// without consuming the keyword when it detected a wrong-document-kind
/// error. `recover_to_next_definition` then saw the keyword as a
/// valid definition start and broke without consuming, producing an
/// unbounded number of error objects until memory was exhausted.
///
/// The fix ensures the keyword token is consumed before returning the
/// error so recovery always makes forward progress.
///
/// Discovered via cargo-fuzz (fuzz_parse_executable target). Minimized
/// reproducer: `p}type i`
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fuzz_regression_type_keyword_in_executable_doc_no_oom() {
    // The original minimized fuzz input (8 bytes)
    let result = parse_executable("p}type i");
    assert!(result.has_errors());
}

/// Regression test: a string description followed by a type keyword
/// (e.g. `""type`) in an executable document must not cause an
/// infinite loop / OOM.
///
/// Previously, the description-followed-by-type-definition error
/// branch in `parse_executable_definition_item` returned `Err`
/// without consuming the string token. `recover_to_next_definition`
/// then saw the string followed by a type keyword and treated it as
/// a description-for-definition, breaking without consuming —
/// infinite loop.
///
/// The fix ensures the string token is consumed before returning the
/// error.
///
/// Discovered via cargo-fuzz (fuzz_parse_executable target).
/// Minimized reproducer: `t""type`
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fuzz_regression_description_before_type_in_executable_no_oom() {
    // The original minimized fuzz input (7 bytes)
    let result = parse_executable("t\"\"type");
    assert!(result.has_errors());

    // Cleaner variants
    let result = parse_executable("\"desc\" type Foo { f: Int }");
    assert!(result.has_errors());

    let result =
        parse_executable("\"\"\"block desc\"\"\" interface Bar");
    assert!(result.has_errors());
}

/// Verifies that each schema-definition keyword in an executable
/// document is properly consumed during error recovery, preventing
/// infinite loops. This covers every keyword branch in the
/// wrong-document-kind check of `parse_executable_definition_item`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn wrong_document_kind_keywords_in_executable_no_oom() {
    let keywords = [
        "type Foo",
        "interface Foo",
        "union Foo",
        "enum Foo",
        "scalar Foo",
        "input Foo",
        "directive @foo on FIELD",
        "schema { query: Q }",
        "extend type Foo",
    ];
    for keyword_input in keywords {
        let result = parse_executable(keyword_input);
        assert!(
            result.has_errors(),
            "expected error for executable doc input: \
             `{keyword_input}`",
        );
    }
}

/// Verifies that a description string followed by each
/// schema-definition keyword in an executable document is properly
/// consumed during error recovery. This covers the
/// description-followed-by-type-keyword check in
/// `parse_executable_definition_item`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn wrong_document_kind_description_keywords_in_executable_no_oom() {
    let keywords = [
        "\"d\" type Foo",
        "\"d\" interface Foo",
        "\"d\" union Foo",
        "\"d\" enum Foo",
        "\"d\" scalar Foo",
        "\"d\" input Foo",
        "\"d\" directive @foo on FIELD",
        "\"d\" schema { query: Q }",
        "\"d\" extend type Foo",
    ];
    for keyword_input in keywords {
        let result = parse_executable(keyword_input);
        assert!(
            result.has_errors(),
            "expected error for executable doc input: \
             `{keyword_input}`",
        );
    }
}

/// Verifies that each executable keyword in a schema document is
/// properly consumed during error recovery, preventing infinite
/// loops. This covers every keyword branch in the
/// wrong-document-kind check of `parse_schema_definition_item`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn wrong_document_kind_keywords_in_schema_no_oom() {
    let keywords = [
        "query { f }",
        "mutation { f }",
        "subscription { f }",
        "fragment F on T { f }",
    ];
    for keyword_input in keywords {
        let result = parse_schema(keyword_input);
        assert!(
            result.has_errors(),
            "expected error for schema doc input: \
             `{keyword_input}`",
        );
    }
}

// =============================================================================
// Fuzz regression: parse_description with invalid string escape
// =============================================================================

/// Regression test for a fuzz-discovered OOM (input: `:"\\d"scalar`).
///
/// When `parse_description()` encounters a StringValue token whose
/// `parse_string_value()` returns `Some(Err(...))` (e.g., invalid
/// escape `\d`), the description parser must consume the token and
/// record an error. Without consuming, recovery sees the StringValue
/// followed by a schema keyword like `scalar` and treats it as a
/// valid description-for-definition restart point — causing an
/// infinite loop with unbounded error accumulation.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fuzz_regression_invalid_description_string_no_oom() {
    // The exact minimized fuzz input (decoded: :"\\d"scalar)
    let result = parse_schema(":\"\\d\"scalar");
    assert!(
        result.has_errors(),
        "expected error for invalid string escape in description",
    );
}

/// Verifies that invalid string escapes in descriptions are properly
/// consumed across all schema definition keywords. Each input pairs
/// an invalid description string (`"\\d"`) with a keyword that starts
/// a schema definition, ensuring `parse_description()` never leaves
/// the StringValue unconsumed.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn invalid_description_string_before_schema_keywords_no_oom() {
    let inputs = [
        "\"\\d\"type Foo { f: Int }",
        "\"\\d\"interface Foo { f: Int }",
        "\"\\d\"union Foo = Bar",
        "\"\\d\"enum Foo { BAR }",
        "\"\\d\"scalar Foo",
        "\"\\d\"input Foo { f: Int }",
        "\"\\d\"directive @foo on FIELD",
        "\"\\d\"schema { query: Q }",
        "\"\\d\"extend type Foo { f: Int }",
    ];
    for input in inputs {
        let result = parse_schema(input);
        assert!(
            result.has_errors(),
            "expected error for schema doc input: `{input}`",
        );
    }
}

/// Verifies the same invalid description behavior in the mixed
/// document parser, which also calls `parse_description()`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn invalid_description_string_before_mixed_keywords_no_oom() {
    let inputs = [
        "\"\\d\"type Foo { f: Int }",
        "\"\\d\"scalar Foo",
        "\"\\d\"directive @foo on FIELD",
    ];
    for input in inputs {
        let result = parse_mixed(input);
        assert!(
            result.has_errors(),
            "expected error for mixed doc input: `{input}`",
        );
    }
}

// =============================================================================
// Fuzz regression: catch-all branches in definition item parsers
// =============================================================================

/// Verifies that unexpected tokens at the top level of a schema
/// document are consumed in the catch-all branch of
/// `parse_schema_definition_item`, preventing infinite loops during
/// error recovery.
///
/// Tokens like `{`, `@`, and numeric literals do not start valid
/// schema definitions but may be treated as restart points by the
/// recovery logic if left unconsumed.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unexpected_token_in_schema_catchall_no_oom() {
    let inputs = [
        "{ foo }",
        "@foo",
        "123",
        "!",
        "(",
    ];
    for input in inputs {
        let result = parse_schema(input);
        assert!(
            result.has_errors(),
            "expected error for schema doc input: `{input}`",
        );
    }
}

/// Verifies that unexpected tokens at the top level of an executable
/// document are consumed in the catch-all branch of
/// `parse_executable_definition_item`, preventing infinite loops
/// during error recovery.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unexpected_token_in_executable_catchall_no_oom() {
    let inputs = [
        "@foo",
        "123",
        "!",
        "(",
    ];
    for input in inputs {
        let result = parse_executable(input);
        assert!(
            result.has_errors(),
            "expected error for executable doc input: `{input}`",
        );
    }
}

/// Verifies that unexpected tokens at the top level of a mixed
/// document are consumed in the catch-all branch of
/// `parse_mixed_definition_item`, preventing infinite loops during
/// error recovery.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn unexpected_token_in_mixed_catchall_no_oom() {
    let inputs = [
        "@foo",
        "123",
        "!",
        "(",
    ];
    for input in inputs {
        let result = parse_mixed(input);
        assert!(
            result.has_errors(),
            "expected error for mixed doc input: `{input}`",
        );
    }
}

// =============================================================================
// Fuzz regression: recursion depth limit (stack overflow prevention)
// =============================================================================

/// Verifies that deeply nested list values (e.g. `[[[...`) do not
/// cause a stack overflow. The parser should report a "maximum nesting
/// depth exceeded" error instead.
///
/// Regression test for a crash found by `fuzz_parse_executable` where
/// hundreds of unclosed `[` brackets caused unbounded recursion in
/// `parse_value → parse_list_value → parse_value → ...`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fuzz_regression_deep_nested_list_value_no_stack_overflow() {
    // 200 unclosed brackets exceeds MAX_RECURSION_DEPTH (32)
    let deep_list = format!(
        "{{ field(arg: {open}) }}",
        open = "[".repeat(200),
    );
    let result = GraphQLParser::new(&deep_list)
        .parse_executable_document();
    assert!(
        result.has_errors(),
        "expected error for deeply nested list value",
    );
    let errors = &result.errors;
    assert!(
        errors.iter().any(|e| {
            e.message().contains("maximum nesting depth exceeded")
        }),
        "expected 'maximum nesting depth exceeded' error, \
         got: {:?}",
        errors.iter().map(|e| e.message()).collect::<Vec<_>>(),
    );
}

/// Verifies that deeply nested selection sets (e.g.
/// `{ f { f { f { ...`) do not cause a stack overflow.
///
/// Regression test for a crash found by `fuzz_parse_executable` where
/// hundreds of nested `{ field { field { ...` patterns caused
/// unbounded recursion in
/// `parse_selection_set → parse_field → parse_selection_set → ...`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fuzz_regression_deep_nested_selection_set_no_stack_overflow() {
    // 200 nested selection sets exceeds MAX_RECURSION_DEPTH (32)
    let deep_fields = format!(
        "{open}{close}",
        open = "{ f ".repeat(200),
        close = "}".repeat(200),
    );
    let result = GraphQLParser::new(&deep_fields)
        .parse_executable_document();
    assert!(
        result.has_errors(),
        "expected error for deeply nested selection set",
    );
    let errors = &result.errors;
    assert!(
        errors.iter().any(|e| {
            e.message().contains("maximum nesting depth exceeded")
        }),
        "expected 'maximum nesting depth exceeded' error, \
         got: {:?}",
        errors.iter().map(|e| e.message()).collect::<Vec<_>>(),
    );
}

/// Verifies that deeply nested list type annotations (e.g.
/// `[[[...String...]]]`) do not cause a stack overflow.
///
/// Regression test for a potential crash where hundreds of nested
/// list type wrappers caused unbounded recursion in
/// `parse_executable_type_annotation → parse_executable_list_type →
/// parse_executable_type_annotation → ...`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fuzz_regression_deep_nested_type_annotation_no_stack_overflow() {
    // 200 nested list types exceeds MAX_RECURSION_DEPTH (32)
    let deep_type = format!(
        "type Query {{ field: {open}String{close} }}",
        open = "[".repeat(200),
        close = "]".repeat(200),
    );
    let result = GraphQLParser::new(&deep_type)
        .parse_schema_document();
    assert!(
        result.has_errors(),
        "expected error for deeply nested type annotation",
    );
    let errors = &result.errors;
    assert!(
        errors.iter().any(|e| {
            e.message().contains("maximum nesting depth exceeded")
        }),
        "expected 'maximum nesting depth exceeded' error, \
         got: {:?}",
        errors.iter().map(|e| e.message()).collect::<Vec<_>>(),
    );
}

// =============================================================================
// Fuzz regression: block string UTF-8 boundary panics
// =============================================================================

/// Verifies that block strings with multi-byte Unicode whitespace in
/// indentation do not panic.
///
/// Regression test for a fuzz-discovered crash where
/// `parse_block_string` used `trim_start()` (which strips all Unicode
/// whitespace) to compute indentation in bytes, then sliced the
/// string at that byte offset. When lines contained multi-byte
/// whitespace characters (e.g. U+2000 EN QUAD, 3 bytes in UTF-8),
/// the computed byte offset could land inside a multi-byte character
/// on a different line, causing a panic.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fuzz_regression_block_string_unicode_whitespace_no_panic() {
    // U+2000 (EN QUAD) is a Unicode whitespace character that is 3
    // bytes in UTF-8 (E2 80 80). Include it as leading "whitespace"
    // on one line, followed by a line with ASCII-only content. The
    // old code would compute common_indent from the multi-byte char
    // and then panic slicing another line at that byte offset.
    let input = "{ f(a: \"\"\"\n\u{2000} line1\n  line2\n\"\"\") }";
    let result = parse_executable(input);
    // Should parse without panicking. May have errors (malformed
    // content), but must not panic.
    let _ = result;
}

/// Verifies that block strings containing U+0085 (NEL, Next Line) do
/// not panic.
///
/// Regression test for a fuzz-discovered crash where
/// `parse_block_string` used `trim()` (which strips all Unicode
/// whitespace, including NEL) to detect blank lines, but later
/// counted indentation using only ASCII spaces and tabs. A line
/// consisting solely of U+0085 was classified as "blank" by
/// `trim()` but was *not* blank according to the GraphQL spec's
/// definition of WhiteSpace (Tab U+0009 and Space U+0020 only):
/// <https://spec.graphql.org/September2025/#WhiteSpace>
///
/// This mismatch caused the line to be filtered out of the
/// blank-line check but still included in common-indent slicing,
/// leading to a byte-index panic inside a multi-byte character.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fuzz_regression_block_string_nel_whitespace_no_panic() {
    // U+0085 (NEL) is 2 bytes in UTF-8 (C2 85). Rust's
    // `str::trim()` considers it whitespace, but the GraphQL spec
    // does not. A line containing only NEL must not be treated as
    // blank by `parse_block_string`.
    let input = "{ f(a: \"\"\"\n\u{85}\n  line\n\"\"\") }";
    let result = parse_executable(input);
    // Should parse without panicking.
    let _ = result;
}
