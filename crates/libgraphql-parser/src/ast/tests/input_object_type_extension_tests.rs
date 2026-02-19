//! Tests for
//! [`crate::ast::InputObjectTypeExtension`] and
//! [`crate::ast::InputObjectTypeExtensionSyntax`].

use crate::ast::DirectiveAnnotation;
use crate::ast::InputObjectTypeExtension;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `InputObjectTypeExtension` with a directive
/// produces the correct source slice.
///
/// The directives-only form requires at least one
/// directive when no fields are present.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_type_extension_directive_only() {
    let source = "extend input Point @foo";
    let iote = InputObjectTypeExtension {
        directives: vec![DirectiveAnnotation {
            arguments: vec![],
            name: make_name("foo", 20, 23),
            span: make_byte_span(19, 23),
            syntax: None,
        }],
        fields: vec![],
        name: make_name("Point", 13, 18),
        span: make_byte_span(0, 23),
        syntax: None,
    };
    assert_eq!(iote.name.value, "Point");
    assert_eq!(iote.directives.len(), 1);
    assert!(iote.fields.is_empty());

    let mut sink = String::new();
    iote.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
