//! Tests for [`crate::ast::InterfaceTypeDefinition`]
//! and [`crate::ast::InterfaceTypeDefinitionSyntax`].

use crate::ast::FieldDefinition;
use crate::ast::InterfaceTypeDefinition;
use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::TypeAnnotation;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `InterfaceTypeDefinition` stores name,
/// fields, and `append_source` slices the correct
/// source range.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Interfaces
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn interface_type_definition_source_slice() {
    let source = "interface Node { id: ID }";
    let itd = InterfaceTypeDefinition {
        span: make_byte_span(0, 25),
        description: None,
        name: make_name("Node", 10, 14),
        implements: vec![],
        directives: vec![],
        fields: vec![FieldDefinition {
            span: make_byte_span(17, 23),
            description: None,
            name: make_name("id", 17, 19),
            arguments: vec![],
            field_type: TypeAnnotation::Named(
                NamedTypeAnnotation {
                    name: make_name("ID", 21, 23),
                    nullability:
                        Nullability::Nullable,
                    span: make_byte_span(21, 23),
                },
            ),
            directives: vec![],
            syntax: None,
        }],
        syntax: None,
    };
    assert_eq!(itd.name.value, "Node");
    assert_eq!(itd.fields.len(), 1);

    let mut sink = String::new();
    itd.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
