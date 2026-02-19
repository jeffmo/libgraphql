//! Tests for the [`crate::ast::Selection`] enum's
//! `append_source` delegation to inner variants.

use crate::ast::Field;
use crate::ast::FragmentSpread;
use crate::ast::InlineFragment;
use crate::ast::Selection;
use crate::ast::SelectionSet;
use crate::ast::TypeCondition;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `Selection::Field` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Language.Fields
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_field_variant_source_slice() {
    let source = "hello";
    let sel = Selection::Field(Field {
        span: make_byte_span(0, 5),
        alias: None,
        name: make_name("hello", 0, 5),
        arguments: vec![],
        directives: vec![],
        selection_set: None,
        syntax: None,
    });
    let mut sink = String::new();
    sel.append_source(&mut sink, Some(source));
    assert_eq!(sink, "hello");
}

/// Verify `Selection::FragmentSpread` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#FragmentSpread
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_fragment_spread_source_slice() {
    let source = "...UserFields";
    let sel = Selection::FragmentSpread(
        FragmentSpread {
            span: make_byte_span(0, 13),
            name: make_name(
                "UserFields", 3, 13,
            ),
            directives: vec![],
            syntax: None,
        },
    );
    let mut sink = String::new();
    sel.append_source(&mut sink, Some(source));
    assert_eq!(sink, "...UserFields");
}

/// Verify `Selection::InlineFragment` variant delegates
/// `append_source` correctly.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#InlineFragment
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn selection_inline_fragment_source_slice() {
    let source = "... on User { name }";
    let sel = Selection::InlineFragment(
        InlineFragment {
            span: make_byte_span(0, 20),
            type_condition: Some(TypeCondition {
                span: make_byte_span(4, 11),
                named_type: make_name(
                    "User", 7, 11,
                ),
                syntax: None,
            }),
            directives: vec![],
            selection_set: SelectionSet {
                span: make_byte_span(12, 20),
                selections: vec![
                    Selection::Field(Field {
                        span: make_byte_span(14, 18),
                        alias: None,
                        name: make_name(
                            "name", 14, 18,
                        ),
                        arguments: vec![],
                        directives: vec![],
                        selection_set: None,
                        syntax: None,
                    }),
                ],
                syntax: None,
            },
            syntax: None,
        },
    );
    let mut sink = String::new();
    sel.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
