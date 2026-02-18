//! Tests for [`crate::ast::SelectionSet`] and
//! [`crate::ast::SelectionSetSyntax`].

use crate::ast::Field;
use crate::ast::Selection;
use crate::ast::SelectionSet;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `SelectionSet` stores a vector of `Selection`
/// items and slices correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Selection-Sets
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_set_construct_and_source_slice() {
    let source = "{ name }";
    let ss = SelectionSet {
        span: make_span(0, 8),
        selections: vec![Selection::Field(Field {
            span: make_span(2, 6),
            alias: None,
            name: make_name("name", 2, 6),
            arguments: vec![],
            directives: vec![],
            selection_set: None,
            syntax: None,
        })],
        syntax: None,
    };
    assert_eq!(ss.selections.len(), 1);

    let mut sink = String::new();
    ss.append_source(&mut sink, Some(source));
    assert_eq!(sink, "{ name }");
}
