//! Tests for [`crate::ast::Field`] and
//! [`crate::ast::FieldSyntax`].

use crate::ast::Field;
use crate::ast::tests::ast_test_helpers::make_byte_span;
use crate::ast::tests::ast_test_helpers::make_name;

/// Verify `Field` stores alias, name, arguments, and
/// nested selection set.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Language.Fields
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn field_with_alias_and_source_slice() {
    let source = "myName: name";
    let field = Field {
        span: make_byte_span(0, 12),
        alias: Some(make_name("myName", 0, 6)),
        name: make_name("name", 8, 12),
        arguments: vec![],
        directives: vec![],
        selection_set: None,
        syntax: None,
    };
    assert_eq!(
        field.alias.as_ref().unwrap().value,
        "myName",
    );
    assert_eq!(field.name.value, "name");

    let mut sink = String::new();
    field.append_source(
        &mut sink,
        Some(source),
    );
    assert_eq!(sink, source);
}
