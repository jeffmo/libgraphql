//! Tests for [`crate::ast::DirectiveAnnotation`].

use std::borrow::Cow;

use crate::ast::Argument;
use crate::ast::DirectiveAnnotation;
use crate::ast::StringValue;
use crate::ast::Value;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `DirectiveAnnotation` stores a name and
/// arguments, and slices the correct source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Language.Directives
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_annotation_construct_and_source_slice() {
    let source =
        r#"@deprecated(reason: "old")"#;
    let da = DirectiveAnnotation {
        span: make_span(0, 26),
        name: make_name("deprecated", 1, 11),
        arguments: vec![Argument {
            span: make_span(12, 25),
            name: make_name("reason", 12, 18),
            value: Value::String(StringValue {
                value: Cow::Borrowed("old"),
                span: make_span(20, 25),
                syntax: None,
            }),
            syntax: None,
        }],
        syntax: None,
    };
    assert_eq!(da.name.value, "deprecated");
    assert_eq!(da.arguments.len(), 1);

    let mut sink = String::new();
    da.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
