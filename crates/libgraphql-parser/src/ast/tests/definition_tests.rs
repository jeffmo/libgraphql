//! Tests for the [`crate::ast::Definition`] enum's
//! `append_source` delegation to inner variants.

use crate::ast::Definition;
use crate::ast::DirectiveAnnotation;
use crate::ast::DirectiveDefinition;
use crate::ast::DirectiveLocation;
use crate::ast::DirectiveLocationKind;
use crate::ast::Field;
use crate::ast::FieldDefinition;
use crate::ast::FragmentDefinition;
use crate::ast::NamedTypeAnnotation;
use crate::ast::Nullability;
use crate::ast::ObjectTypeExtension;
use crate::ast::OperationDefinition;
use crate::ast::OperationKind;
use crate::ast::Selection;
use crate::ast::SelectionSet;
use crate::ast::TypeAnnotation;
use crate::ast::TypeCondition;
use crate::ast::TypeDefinition;
use crate::ast::TypeExtension;
use crate::ast::tests::ast_test_helpers::make_name;
use crate::ast::tests::ast_test_helpers::make_span;

/// Verify `Definition` enum delegates `append_source`
/// correctly for the `TypeDefinition` variant.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn definition_type_definition_source_slice() {
    let source = "scalar URL";
    let def = Definition::TypeDefinition(
        TypeDefinition::Scalar(
            crate::ast::ScalarTypeDefinition {
                span: make_span(0, 10),
                description: None,
                name: make_name("URL", 7, 10),
                directives: vec![],
                syntax: None,
            },
        ),
    );
    let mut sink = String::new();
    def.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `Definition` enum delegates `append_source`
/// correctly for the `OperationDefinition` variant.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn definition_operation_source_slice() {
    let source = "query { hello }";
    let def = Definition::OperationDefinition(
        OperationDefinition {
            span: make_span(0, 15),
            description: None,
            operation_kind: OperationKind::Query,
            name: None,
            variable_definitions: vec![],
            directives: vec![],
            selection_set: SelectionSet {
                span: make_span(6, 15),
                selections: vec![
                    Selection::Field(Field {
                        span: make_span(8, 13),
                        alias: None,
                        name: make_name(
                            "hello", 8, 13,
                        ),
                        arguments: vec![],
                        directives: vec![],
                        selection_set: None,
                        syntax: None,
                    }),
                ],
                syntax: None,
            },
            syntax: None,
        },
    );
    let mut sink = String::new();
    def.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `Definition::DirectiveDefinition` variant
/// delegates `append_source` correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn definition_directive_definition_source_slice() {
    let source =
        "directive @skip on FIELD";
    let def = Definition::DirectiveDefinition(
        DirectiveDefinition {
            span: make_span(0, 24),
            description: None,
            name: make_name("skip", 11, 15),
            arguments: vec![],
            repeatable: false,
            locations: vec![DirectiveLocation {
                kind:
                    DirectiveLocationKind::Field,
                span: make_span(19, 24),
                syntax: None,
            }],
            syntax: None,
        },
    );
    let mut sink = String::new();
    def.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `Definition::FragmentDefinition` variant
/// delegates `append_source` correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn definition_fragment_source_slice() {
    let source =
        "fragment F on User { name }";
    let def = Definition::FragmentDefinition(
        FragmentDefinition {
            span: make_span(0, 27),
            description: None,
            name: make_name("F", 9, 10),
            type_condition: TypeCondition {
                span: make_span(11, 18),
                named_type: make_name(
                    "User", 14, 18,
                ),
                syntax: None,
            },
            directives: vec![],
            selection_set: SelectionSet {
                span: make_span(19, 27),
                selections: vec![
                    Selection::Field(Field {
                        span: make_span(21, 25),
                        alias: None,
                        name: make_name(
                            "name", 21, 25,
                        ),
                        arguments: vec![],
                        directives: vec![],
                        selection_set: None,
                        syntax: None,
                    }),
                ],
                syntax: None,
            },
            syntax: None,
        },
    );
    let mut sink = String::new();
    def.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `Definition::SchemaExtension` variant
/// delegates `append_source` correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn definition_schema_extension_source_slice() {
    let source = "extend schema @auth";
    let def = Definition::SchemaExtension(
        crate::ast::SchemaExtension {
            span: make_span(0, 19),
            directives: vec![
                DirectiveAnnotation {
                    span: make_span(14, 19),
                    name: make_name(
                        "auth", 15, 19,
                    ),
                    arguments: vec![],
                    syntax: None,
                },
            ],
            root_operations: vec![],
            syntax: None,
        },
    );
    let mut sink = String::new();
    def.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}

/// Verify `Definition::TypeExtension` variant delegates
/// `append_source` correctly.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn definition_type_extension_source_slice() {
    let source =
        "extend type Query { age: Int }";
    let def = Definition::TypeExtension(
        TypeExtension::Object(
            ObjectTypeExtension {
                span: make_span(0, 30),
                name: make_name(
                    "Query", 12, 17,
                ),
                implements: vec![],
                directives: vec![],
                fields: vec![FieldDefinition {
                    span: make_span(20, 28),
                    description: None,
                    name: make_name(
                        "age", 20, 23,
                    ),
                    arguments: vec![],
                    field_type:
                        TypeAnnotation::Named(
                            NamedTypeAnnotation {
                                name: make_name(
                                    "Int",
                                    25, 28,
                                ),
                                nullability:
                                    Nullability::Nullable,
                                span: make_span(
                                    25, 28,
                                ),
                            },
                        ),
                    directives: vec![],
                    syntax: None,
                }],
                syntax: None,
            },
        ),
    );
    let mut sink = String::new();
    def.append_source(&mut sink, Some(source));
    assert_eq!(sink, source);
}
