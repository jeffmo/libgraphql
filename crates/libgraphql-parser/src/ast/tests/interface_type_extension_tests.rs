//! Tests for [`crate::ast::InterfaceTypeExtension`]
//! and
//! [`crate::ast::InterfaceTypeExtensionSyntax`].

use crate::ast::DirectiveAnnotation;
use crate::ast::InterfaceTypeExtension;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `InterfaceTypeExtension` stores name and
/// directives, and `append_source` slices the correct
/// source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Interface-Extensions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn interface_type_extension_source_slice() {
    let source = "extend interface Node @foo";
    let ite = InterfaceTypeExtension {
        span: make_byte_span(0, 26),
        name: make_name("Node", 17, 21),
        implements: vec![],
        directives: vec![DirectiveAnnotation {
            span: make_byte_span(22, 26),
            name: make_name("foo", 23, 26),
            arguments: vec![],
            syntax: None,
        }],
        fields: vec![],
        syntax: None,
    };
    assert_eq!(ite.name.value, "Node");
    assert_eq!(ite.directives.len(), 1);

    let mut sink = String::new();
    ite.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
