//! Tests for [`crate::ast::VariableDefinition`] and
//! [`crate::ast::VariableDefinitionSyntax`].

use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::TypeAnnotation;
use crate::ast::VariableDefinition;
use crate::ast::tests::ast_test_utils::make_byte_span;
use crate::ast::tests::ast_test_utils::make_name;

/// Verify `VariableDefinition` stores variable name,
/// type annotation, and produces the correct source
/// slice.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Language.Variables
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn variable_definition_source_slice() {
    let source = "$id: ID!";
    let vd = VariableDefinition {
        default_value: None,
        description: None,
        directives: vec![],
        span: make_byte_span(0, 8),
        syntax: None,
        var_type: TypeAnnotation::Named(
            NamedTypeAnnotation {
                name: make_name("ID", 5, 7),
                nullability: Nullability::NonNull {
                    syntax: None,
                },
                span: make_byte_span(5, 8),
            },
        ),
        variable: make_name("id", 1, 3),
    };
    assert_eq!(vd.variable.value, "id");

    let mut sink = String::new();
    vd.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `VariableDefinition` with a nullable type.
///
/// Relevant spec section:
/// https://spec.graphql.org/September2025/#sec-Language.Variables
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn variable_definition_nullable() {
    let source = "$name: String";
    let vd = VariableDefinition {
        default_value: None,
        description: None,
        directives: vec![],
        span: make_byte_span(0, 13),
        syntax: None,
        var_type: TypeAnnotation::Named(
            NamedTypeAnnotation {
                name: make_name("String", 7, 13),
                nullability: Nullability::Nullable,
                span: make_byte_span(7, 13),
            },
        ),
        variable: make_name("name", 1, 5),
    };
    assert_eq!(vd.variable.value, "name");

    let mut sink = String::new();
    vd.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
