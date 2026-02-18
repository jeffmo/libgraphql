//! Tests for [`crate::ast::Name`].

use crate::ast::tests::ast_test_helpers::make_name;

/// Verify that a `Name` node can be constructed with a
/// borrowed string, and that `append_source` slices the
/// correct byte range from the source text.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Names
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn name_construct_and_source_slice() {
    let source = "type Query { hello: String }";
    // "Query" sits at bytes 5..10
    let name = make_name("Query", 5, 10);

    assert_eq!(name.value, "Query");
    assert_eq!(
        name.span.start_inclusive.byte_offset(), 5,
    );
    assert_eq!(
        name.span.end_exclusive.byte_offset(), 10,
    );

    let mut sink = String::new();
    name.append_source(&mut sink, Some(source));
    assert_eq!(sink, "Query");
}

/// Verify that `append_source` with `source = None` is a
/// no-op (nothing is appended to the sink).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn name_append_source_none_is_noop() {
    let name = make_name("Foo", 0, 3);
    let mut sink = String::new();
    name.append_source(&mut sink, None);
    assert_eq!(sink, "");
}
