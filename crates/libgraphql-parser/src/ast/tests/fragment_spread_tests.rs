//! Tests for [`crate::ast::FragmentSpread`] and
//! [`crate::ast::FragmentSpreadSyntax`].

use crate::ast::DirectiveAnnotation;
use crate::ast::FragmentSpread;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `FragmentSpread` stores name and directives
/// and produces the correct source slice.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#FragmentSpread
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_spread_source_slice() {
    let source = "...UserFields @skip(if: true)";
    let fs = FragmentSpread {
        directives: vec![DirectiveAnnotation {
            arguments: vec![],
            name: make_name("skip", 15, 19),
            span: make_byte_span(14, 29),
            syntax: None,
        }],
        name: make_name("UserFields", 3, 13),
        span: make_byte_span(0, 29),
        syntax: None,
    };
    assert_eq!(fs.name.value, "UserFields");
    assert_eq!(fs.directives.len(), 1);

    let mut sink = String::new();
    fs.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `FragmentSpread` without directives.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#FragmentSpread
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn fragment_spread_no_directives() {
    let source = "...UserFields";
    let fs = FragmentSpread {
        directives: vec![],
        name: make_name("UserFields", 3, 13),
        span: make_byte_span(0, 13),
        syntax: None,
    };
    assert_eq!(fs.name.value, "UserFields");
    assert!(fs.directives.is_empty());

    let mut sink = String::new();
    fs.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
