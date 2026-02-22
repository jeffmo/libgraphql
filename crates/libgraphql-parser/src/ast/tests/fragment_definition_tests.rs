//! Tests for [`crate::ast::FragmentDefinition`] and
//! [`crate::ast::FragmentDefinitionSyntax`].

use crate::ast::Field;
use crate::ast::FragmentDefinition;
use crate::ast::Selection;
use crate::ast::SelectionSet;
use crate::ast::TypeCondition;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `FragmentDefinition` stores name, type
/// condition, and selection set.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Language.Fragments
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_definition_construct_and_source_slice() {
    let source =
        "fragment UserFields on User { name }";
    let fd = FragmentDefinition {
        span: make_byte_span(0, 36),
        description: None,
        name: make_name("UserFields", 9, 19),
        type_condition: TypeCondition {
            span: make_byte_span(20, 27),
            named_type: make_name(
                "User", 23, 27,
            ),
            syntax: None,
        },
        directives: vec![],
        selection_set: SelectionSet {
            span: make_byte_span(28, 36),
            selections: vec![
                Selection::Field(Field {
                    span: make_byte_span(30, 34),
                    alias: None,
                    name: make_name(
                        "name", 30, 34,
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
    };
    assert_eq!(fd.name.value, "UserFields");
    assert_eq!(
        fd.type_condition.named_type.value,
        "User",
    );

    let mut sink = String::new();
    fd.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
