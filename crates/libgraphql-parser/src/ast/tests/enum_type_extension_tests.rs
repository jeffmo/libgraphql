//! Tests for [`crate::ast::EnumTypeExtension`] and
//! [`crate::ast::EnumTypeExtensionSyntax`].

use crate::ast::EnumTypeExtension;
use crate::ast::EnumValueDefinition;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `EnumTypeExtension` stores name and enum
/// value definitions, and `append_source` slices the
/// correct source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Enum-Extensions
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_type_extension_source_slice() {
    let source =
        "extend enum Status { DELETED }";
    let ete = EnumTypeExtension {
        span: make_byte_span(0, 30),
        name: make_name("Status", 12, 18),
        directives: vec![],
        values: vec![EnumValueDefinition {
            span: make_byte_span(21, 28),
            description: None,
            name: make_name("DELETED", 21, 28),
            directives: vec![],
        }],
        syntax: None,
    };
    assert_eq!(ete.name.value, "Status");
    assert_eq!(ete.values.len(), 1);

    let mut sink = String::new();
    ete.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
