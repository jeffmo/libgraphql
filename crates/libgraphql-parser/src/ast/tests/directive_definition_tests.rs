//! Tests for [`crate::ast::DirectiveDefinition`] and
//! [`crate::ast::DirectiveDefinitionSyntax`].

use crate::ast::DirectiveDefinition;
use crate::ast::DirectiveLocation;
use crate::ast::DirectiveLocationKind;
use crate::ast::tests::ast_test_helpers::make_byte_span;
use crate::ast::tests::ast_test_helpers::make_name;

/// Verify `DirectiveDefinition` stores name, locations,
/// repeatability, and arguments.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Type-System.Directives
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_definition_construct_and_source_slice() {
    let source =
        "directive @log on FIELD";
    let dd = DirectiveDefinition {
        span: make_byte_span(0, 23),
        description: None,
        name: make_name("log", 11, 14),
        arguments: vec![],
        repeatable: false,
        locations: vec![DirectiveLocation {
            kind: DirectiveLocationKind::Field,
            span: make_byte_span(18, 23),
            syntax: None,
        }],
        syntax: None,
    };
    assert_eq!(dd.name.value, "log");
    assert!(!dd.repeatable);
    assert_eq!(dd.locations.len(), 1);

    let mut sink = String::new();
    dd.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
