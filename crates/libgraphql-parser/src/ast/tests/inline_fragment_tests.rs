//! Tests for [`crate::ast::InlineFragment`] and
//! [`crate::ast::InlineFragmentSyntax`].

use crate::ast::Field;
use crate::ast::InlineFragment;
use crate::ast::Selection;
use crate::ast::SelectionSet;
use crate::ast::TypeCondition;
use crate::ast::tests::ast_test_helpers::make_byte_span;
use crate::ast::tests::ast_test_helpers::make_name;

/// Verify `InlineFragment` with a type condition stores
/// the condition, selection set, and produces the correct
/// source slice.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#InlineFragment
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn inline_fragment_with_type_condition() {
    let source = "... on User { name }";
    let inf = InlineFragment {
        directives: vec![],
        selection_set: SelectionSet {
            selections: vec![
                Selection::Field(Field {
                    alias: None,
                    arguments: vec![],
                    directives: vec![],
                    name: make_name("name", 14, 18),
                    selection_set: None,
                    span: make_byte_span(14, 18),
                    syntax: None,
                }),
            ],
            span: make_byte_span(12, 20),
            syntax: None,
        },
        span: make_byte_span(0, 20),
        syntax: None,
        type_condition: Some(TypeCondition {
            named_type: make_name("User", 7, 11),
            span: make_byte_span(4, 11),
            syntax: None,
        }),
    };
    assert_eq!(
        inf.type_condition.as_ref().unwrap()
            .named_type.value,
        "User",
    );

    let mut sink = String::new();
    inf.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `InlineFragment` without a type condition
/// (bare `... { ... }`).
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#InlineFragment
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn inline_fragment_no_type_condition() {
    let source = "... { id }";
    let inf = InlineFragment {
        directives: vec![],
        selection_set: SelectionSet {
            selections: vec![
                Selection::Field(Field {
                    alias: None,
                    arguments: vec![],
                    directives: vec![],
                    name: make_name("id", 6, 8),
                    selection_set: None,
                    span: make_byte_span(6, 8),
                    syntax: None,
                }),
            ],
            span: make_byte_span(4, 10),
            syntax: None,
        },
        span: make_byte_span(0, 10),
        syntax: None,
        type_condition: None,
    };
    assert!(inf.type_condition.is_none());

    let mut sink = String::new();
    inf.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
