use crate::schema::SchemaBuildErrorKind;
use crate::span::Span;
use crate::type_builders::DirectiveBuilder;
use crate::type_builders::EnumTypeBuilder;
use crate::type_builders::EnumValueDefBuilder;
use crate::type_builders::FieldDefBuilder;
use crate::type_builders::InputObjectTypeBuilder;
use crate::type_builders::InterfaceTypeBuilder;
use crate::type_builders::ObjectTypeBuilder;
use crate::type_builders::ParameterDefBuilder;
use crate::type_builders::ScalarTypeBuilder;
use crate::type_builders::UnionTypeBuilder;
use crate::types::DirectiveLocationKind;
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
    match builder.errors[0].kind() {
        SchemaBuildErrorKind::InvalidDunderPrefixedTypeName {
            type_name,
        } => {
            assert_eq!(type_name, "__Bad");
        },
        other => panic!("unexpected error kind: {other:?}"),
    }
}

// Verifies ScalarTypeBuilder::new() rejects __ prefix.
// https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_new_rejects_dunder_prefix() {
    let err = ScalarTypeBuilder::new(
        "__Bad", Span::builtin(),
    ).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::InvalidDunderPrefixedTypeName { .. },
    ));
}

// Verifies InterfaceTypeBuilder::new() rejects __ prefix.
// https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// Written by Claude Code, reviewed by a human.
#[test]
fn interface_new_rejects_dunder_prefix() {
    let err = InterfaceTypeBuilder::new(
        "__Bad", Span::builtin(),
    ).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::InvalidDunderPrefixedTypeName { .. },
    ));
}

// Verifies UnionTypeBuilder::new() rejects __ prefix.
// https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// Written by Claude Code, reviewed by a human.
#[test]
fn union_new_rejects_dunder_prefix() {
    let err = UnionTypeBuilder::new(
        "__Bad", Span::builtin(),
    ).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::InvalidDunderPrefixedTypeName { .. },
    ));
}

// Verifies EnumTypeBuilder::new() rejects __ prefix.
// https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_new_rejects_dunder_prefix() {
    let err = EnumTypeBuilder::new(
        "__Bad", Span::builtin(),
    ).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::InvalidDunderPrefixedTypeName { .. },
    ));
}

// Verifies InputObjectTypeBuilder::new() rejects __ prefix.
// https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_new_rejects_dunder_prefix() {
    let err = InputObjectTypeBuilder::new(
        "__Bad", Span::builtin(),
    ).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::InvalidDunderPrefixedTypeName { .. },
    ));
}

// Verifies DirectiveBuilder::new() rejects __ prefix.
// https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_new_rejects_dunder_prefix() {
    let err = DirectiveBuilder::new(
        "__bad", Span::builtin(),
    ).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::InvalidDunderPrefixedDirectiveName {
            ..
        },
    ));
}

// Verifies InterfaceTypeBuilder rejects self-implementation.
// https://spec.graphql.org/September2025/#sec-Interfaces.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn interface_rejects_self_implementation() {
    let mut builder = InterfaceTypeBuilder::new(
        "Node", Span::builtin(),
    ).unwrap();
    let err = builder.add_implements(
        "Node", Span::builtin(),
    ).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::InvalidSelfImplementingInterface {
            ..
        },
    ));
}

// Verifies EnumTypeBuilder::add_value() rejects duplicates.
// https://spec.graphql.org/September2025/#sec-Enums.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_add_value_rejects_duplicate() {
    let mut builder = EnumTypeBuilder::new(
        "Status", Span::builtin(),
    ).unwrap();
    builder.add_value(
        EnumValueDefBuilder::new("ACTIVE", Span::builtin()),
    ).unwrap();
    let err = builder.add_value(
        EnumValueDefBuilder::new("ACTIVE", Span::builtin()),
    ).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::DuplicateEnumValueDefinition { .. },
    ));
}

// Verifies UnionTypeBuilder::add_member() rejects duplicates.
// https://spec.graphql.org/September2025/#sec-Unions.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn union_add_member_rejects_duplicate() {
    let mut builder = UnionTypeBuilder::new(
        "SearchResult", Span::builtin(),
    ).unwrap();
    builder.add_member("User", Span::builtin()).unwrap();
    let err = builder.add_member(
        "User", Span::builtin(),
    ).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::DuplicateUnionMember { .. },
    ));
}

// Verifies DirectiveBuilder::add_parameter() rejects duplicates.
// https://spec.graphql.org/September2025/#sec-Type-System.Directives.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_add_parameter_rejects_duplicate() {
    let mut builder = DirectiveBuilder::new(
        "auth", Span::builtin(),
    ).unwrap();
    builder.add_location(DirectiveLocationKind::FieldDefinition);
    builder.add_parameter(ParameterDefBuilder::new(
        "role",
        TypeAnnotation::named("String", false),
        Span::builtin(),
    )).unwrap();
    let err = builder.add_parameter(ParameterDefBuilder::new(
        "role",
        TypeAnnotation::named("String", true),
        Span::builtin(),
    )).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::DuplicateParameterDefinition { .. },
    ));
}

// Verifies FieldDefBuilder::add_parameter() rejects duplicates.
// https://spec.graphql.org/September2025/#sec-Field-Arguments.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn field_add_parameter_rejects_duplicate() {
    let mut builder = FieldDefBuilder::new(
        "users",
        TypeAnnotation::named("User", true),
        Span::builtin(),
    );
    builder.add_parameter(ParameterDefBuilder::new(
        "first",
        TypeAnnotation::named("Int", true),
        Span::builtin(),
    )).unwrap();
    let err = builder.add_parameter(ParameterDefBuilder::new(
        "first",
        TypeAnnotation::named("Int", true),
        Span::builtin(),
    )).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::DuplicateParameterDefinition { .. },
    ));
}
