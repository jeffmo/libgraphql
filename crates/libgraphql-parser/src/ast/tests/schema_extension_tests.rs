//! Tests for [`crate::ast::SchemaExtension`] and
//! [`crate::ast::SchemaExtensionSyntax`].

use crate::ast::DirectiveAnnotation;
use crate::ast::OperationKind;
use crate::ast::RootOperationTypeDefinition;
use crate::ast::SchemaExtension;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `SchemaExtension` with a root operation
/// produces the correct source slice.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Schema-Extension
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn schema_extension_with_operation() {
    let source =
        "extend schema { mutation: Mutation }";
    let se = SchemaExtension {
        directives: vec![],
        root_operations: vec![
            RootOperationTypeDefinition {
                named_type: make_name(
                    "Mutation", 26, 34,
                ),
                operation_kind: OperationKind::Mutation,
                span: make_byte_span(16, 34),
                syntax: None,
            },
        ],
        span: make_byte_span(0, 36),
        syntax: None,
    };
    assert_eq!(se.root_operations.len(), 1);

    let mut sink = String::new();
    se.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `SchemaExtension` with only a directive
/// (directives-only form).
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Schema-Extension
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn schema_extension_directive_only() {
    let source = "extend schema @auth";
    let se = SchemaExtension {
        directives: vec![DirectiveAnnotation {
            arguments: vec![],
            name: make_name("auth", 15, 19),
            span: make_byte_span(14, 19),
            syntax: None,
        }],
        root_operations: vec![],
        span: make_byte_span(0, 19),
        syntax: None,
    };
    assert_eq!(se.directives.len(), 1);
    assert!(se.root_operations.is_empty());

    let mut sink = String::new();
    se.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
