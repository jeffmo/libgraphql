//! Tests for [`crate::ast::EnumTypeDefinition`] and
//! [`crate::ast::EnumTypeDefinitionSyntax`].

use crate::ast::EnumTypeDefinition;
use crate::ast::EnumValueDefinition;
use crate::ast::tests::ast_test_helpers::make_byte_span;
use crate::ast::tests::ast_test_helpers::make_name;

/// Verify `EnumTypeDefinition` stores name and enum
/// value definitions, and `append_source` slices the
/// correct source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Enums
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_type_definition_source_slice() {
    let source =
        "enum Status { ACTIVE INACTIVE }";
    let etd = EnumTypeDefinition {
        span: make_byte_span(0, 31),
        description: None,
        name: make_name("Status", 5, 11),
        directives: vec![],
        values: vec![
            EnumValueDefinition {
                span: make_byte_span(14, 20),
                description: None,
                name: make_name(
                    "ACTIVE", 14, 20,
                ),
                directives: vec![],
            },
            EnumValueDefinition {
                span: make_byte_span(21, 29),
                description: None,
                name: make_name(
                    "INACTIVE", 21, 29,
                ),
                directives: vec![],
            },
        ],
        syntax: None,
    };
    assert_eq!(etd.name.value, "Status");
    assert_eq!(etd.values.len(), 2);

    let mut sink = String::new();
    etd.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
