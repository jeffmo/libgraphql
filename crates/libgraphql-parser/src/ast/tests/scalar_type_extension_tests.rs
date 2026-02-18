//! Tests for [`crate::ast::ScalarTypeExtension`] and
//! [`crate::ast::ScalarTypeExtensionSyntax`].

use crate::ast::DirectiveAnnotation;
use crate::ast::ScalarTypeExtension;
use crate::ast::tests::ast_test_helpers::make_byte_span;
use crate::ast::tests::ast_test_helpers::make_name;

/// Verify `ScalarTypeExtension` stores name and
/// directives, and `append_source` slices the correct
/// source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Scalar-Extensions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_type_extension_source_slice() {
    let source = "extend scalar DateTime @foo";
    let ste = ScalarTypeExtension {
        span: make_byte_span(0, 27),
        name: make_name("DateTime", 14, 22),
        directives: vec![DirectiveAnnotation {
            span: make_byte_span(23, 27),
            name: make_name("foo", 24, 27),
            arguments: vec![],
            syntax: None,
        }],
        syntax: None,
    };
    assert_eq!(ste.name.value, "DateTime");
    assert_eq!(ste.directives.len(), 1);

    let mut sink = String::new();
    ste.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
