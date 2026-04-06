use crate::schema::SchemaBuildErrorKind;
use crate::span::Span;
use crate::type_builders::EnumTypeBuilder;
use crate::type_builders::EnumValueDefBuilder;
use crate::type_builders::FieldDefBuilder;
use crate::type_builders::ObjectTypeBuilder;
use crate::types::TypeAnnotation;

// Verifies new() fails immediately on __ prefix.
// https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// Written by Claude Code, reviewed by a human.
#[test]
fn new_rejects_dunder_prefix() {
    let err = ObjectTypeBuilder::new(
        "__Bad", Span::builtin(),
    ).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::InvalidDunderPrefixedTypeName { .. },
    ));
}

// Verifies add_field() fails immediately on duplicate name.
// Written by Claude Code, reviewed by a human.
#[test]
fn add_field_rejects_duplicate() {
    let mut builder = ObjectTypeBuilder::new(
        "User", Span::builtin(),
    ).unwrap();
    builder.add_field(FieldDefBuilder::new(
        "id",
        TypeAnnotation::named("ID", false),
        Span::builtin(),
    )).unwrap();
    let err = builder.add_field(FieldDefBuilder::new(
        "id",
        TypeAnnotation::named("String", true),
        Span::builtin(),
    )).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::DuplicateFieldNameDefinition { .. },
    ));
}

// Verifies add_field() fails on __ prefix.
// https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// Written by Claude Code, reviewed by a human.
#[test]
fn add_field_rejects_dunder_prefix() {
    let mut builder = ObjectTypeBuilder::new(
        "User", Span::builtin(),
    ).unwrap();
    let err = builder.add_field(FieldDefBuilder::new(
        "__bad",
        TypeAnnotation::named("String", true),
        Span::builtin(),
    )).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::InvalidDunderPrefixedFieldName { .. },
    ));
}

// Verifies add_implements() fails on duplicate interface.
// Written by Claude Code, reviewed by a human.
#[test]
fn add_implements_rejects_duplicate() {
    let mut builder = ObjectTypeBuilder::new(
        "User", Span::builtin(),
    ).unwrap();
    builder.add_implements("Node", Span::builtin()).unwrap();
    let err = builder.add_implements(
        "Node", Span::builtin(),
    ).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::DuplicateInterfaceImplementsDeclaration { .. },
    ));
}

// Verifies enum builder rejects true/false/null value names.
// https://spec.graphql.org/September2025/#EnumValuesDefinition
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_rejects_invalid_value_names() {
    let mut builder = EnumTypeBuilder::new(
        "Bool", Span::builtin(),
    ).unwrap();
    for invalid in ["true", "false", "null"] {
        let err = builder.add_value(
            EnumValueDefBuilder::new(invalid, Span::builtin()),
        ).unwrap_err();
        assert!(
            matches!(
                err.kind(),
                SchemaBuildErrorKind::InvalidEnumValueName { .. },
            ),
            "expected InvalidEnumValueName for `{invalid}`",
        );
    }
}

// Verifies from_ast() collects dunder-prefix errors instead
// of panicking.
// Written by Claude Code, reviewed by a human.
#[test]
fn from_ast_collects_dunder_errors() {
    let doc: libgraphql_parser::ast::Document<'static> =
        libgraphql_parser::parse_schema(
            "type __Bad { x: Int }",
        ).into_ast();
    let td = match &doc.definitions[0] {
        libgraphql_parser::ast::Definition::TypeDefinition(
            libgraphql_parser::ast::TypeDefinition::Object(obj),
        ) => obj,
        _ => panic!("expected object type definition"),
    };
    let builder = ObjectTypeBuilder::from_ast(
        td,
        crate::span::SourceMapId(1),
    );
    assert!(!builder.errors.is_empty());
    assert!(matches!(
        builder.errors[0].kind(),
        SchemaBuildErrorKind::InvalidDunderPrefixedTypeName { .. },
    ));
}
