use crate::error_note::ErrorNoteKind;
use crate::schema::SchemaBuildError;
use crate::schema::SchemaBuildErrorKind;
use crate::span::Span;
use crate::type_builders::DirectiveBuilder;
use crate::type_builders::EnumTypeBuilder;
use crate::type_builders::EnumValueDefBuilder;
use crate::type_builders::FieldDefBuilder;
use crate::type_builders::InputFieldDefBuilder;
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
        TypeAnnotation::named("ID", /* nullable = */ false),
        Span::builtin(),
    )).unwrap();
    let err = builder.add_field(FieldDefBuilder::new(
        "id",
        TypeAnnotation::named("String", /* nullable = */ true),
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
        TypeAnnotation::named("String", /* nullable = */ true),
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

// Verifies from_ast() returns Err with dunder-prefix errors
// instead of panicking.
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
    let errors = ObjectTypeBuilder::from_ast(
        td,
        crate::span::SourceMapId(1),
    ).unwrap_err();
    assert!(!errors.is_empty());
    match errors[0].kind() {
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
        TypeAnnotation::named("String", /* nullable = */ false),
        Span::builtin(),
    )).unwrap();
    let err = builder.add_parameter(ParameterDefBuilder::new(
        "role",
        TypeAnnotation::named("String", /* nullable = */ true),
        Span::builtin(),
    )).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::DuplicateParameterDefinition { .. },
    ));
}

// Regression test for a bug where
// DirectiveBuilder::add_parameter() emitted the wrong error
// kind when rejecting a `__`-prefixed parameter name. It
// previously returned
// SchemaBuildErrorKind::InvalidDunderPrefixedDirectiveName
// (which describes an invalid directive NAME) when the actual
// problem was the PARAMETER name; the correct variant is
// SchemaBuildErrorKind::InvalidDunderPrefixedParamName. This
// test asserts the corrected behavior so the wrong-variant bug
// cannot reappear.
//
// https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_add_parameter_rejects_dunder_prefix() {
    let mut builder = DirectiveBuilder::new(
        "myDirective", Span::builtin(),
    ).unwrap();
    builder.add_location(DirectiveLocationKind::FieldDefinition);
    let err = builder.add_parameter(ParameterDefBuilder::new(
        "__bad",
        TypeAnnotation::named("String", /* nullable = */ true),
        Span::builtin(),
    )).unwrap_err();
    assert!(
        matches!(
            err.kind(),
            SchemaBuildErrorKind::InvalidDunderPrefixedParamName {
                param_name,
                ..
            } if param_name == "__bad"
        ),
        "expected InvalidDunderPrefixedParamName, got: {:?}",
        err.kind(),
    );
}

// Verifies FieldDefBuilder::add_parameter() rejects duplicates.
// https://spec.graphql.org/September2025/#sec-Field-Arguments.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn field_add_parameter_rejects_duplicate() {
    let mut builder = FieldDefBuilder::new(
        "users",
        TypeAnnotation::named("User", /* nullable = */ true),
        Span::builtin(),
    );
    builder.add_parameter(ParameterDefBuilder::new(
        "first",
        TypeAnnotation::named("Int", /* nullable = */ true),
        Span::builtin(),
    )).unwrap();
    let err = builder.add_parameter(ParameterDefBuilder::new(
        "first",
        TypeAnnotation::named("Int", /* nullable = */ true),
        Span::builtin(),
    )).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::DuplicateParameterDefinition { .. },
    ));
}

// -----------------------------------------------------------
// Spec-note coverage (Task 16.6d)
// -----------------------------------------------------------

// Asserts that `err` carries at least one spec-reference note
// whose URL contains the given anchor fragment.
fn assert_spec_note(err: &SchemaBuildError, anchor: &str) {
    assert!(
        err.notes().iter().any(|n| {
            n.kind == ErrorNoteKind::Spec && n.message.contains(anchor)
        }),
        "expected a spec note containing `{anchor}`, got: {:?}",
        err.notes(),
    );
}

// Verifies that every ObjectTypeBuilder error path attaches a
// spec-reference note pointing at the rule it enforces:
// reserved `__` names -> Reserved Names; duplicate fields and
// duplicate `implements` declarations -> Objects Type
// Validation.
//
// See https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// and https://spec.graphql.org/September2025/#sec-Objects.Type-Validation
//
// Written by Claude Code, reviewed by a human.
#[test]
fn object_builder_errors_carry_spec_notes() {
    let err = ObjectTypeBuilder::new(
        "__Bad", Span::builtin(),
    ).unwrap_err();
    assert_spec_note(&err, "#sec-Names.Reserved-Names");

    let mut builder = ObjectTypeBuilder::new(
        "User", Span::builtin(),
    ).unwrap();
    let err = builder.add_field(FieldDefBuilder::new(
        "__bad",
        TypeAnnotation::named("String", /* nullable = */ true),
        Span::builtin(),
    )).unwrap_err();
    assert_spec_note(&err, "#sec-Names.Reserved-Names");

    builder.add_field(FieldDefBuilder::new(
        "id",
        TypeAnnotation::named("ID", /* nullable = */ false),
        Span::builtin(),
    )).unwrap();
    let err = builder.add_field(FieldDefBuilder::new(
        "id",
        TypeAnnotation::named("ID", /* nullable = */ false),
        Span::builtin(),
    )).unwrap_err();
    assert_spec_note(&err, "#sec-Objects.Type-Validation");

    builder.add_implements("Node", Span::builtin()).unwrap();
    let err = builder.add_implements(
        "Node", Span::builtin(),
    ).unwrap_err();
    assert_spec_note(&err, "#sec-Objects.Type-Validation");
}

// Verifies that every FieldDefBuilder error path attaches a
// spec-reference note: reserved `__` parameter names ->
// Reserved Names; duplicate parameter names -> Objects Type
// Validation (rule 2.4: unique argument names per field).
//
// See https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// and https://spec.graphql.org/September2025/#sec-Objects.Type-Validation
//
// Written by Claude Code, reviewed by a human.
#[test]
fn field_def_builder_errors_carry_spec_notes() {
    let mut builder = FieldDefBuilder::new(
        "users",
        TypeAnnotation::named("User", /* nullable = */ true),
        Span::builtin(),
    );
    let err = builder.add_parameter(ParameterDefBuilder::new(
        "__bad",
        TypeAnnotation::named("Int", /* nullable = */ true),
        Span::builtin(),
    )).unwrap_err();
    assert_spec_note(&err, "#sec-Names.Reserved-Names");

    builder.add_parameter(ParameterDefBuilder::new(
        "first",
        TypeAnnotation::named("Int", /* nullable = */ true),
        Span::builtin(),
    )).unwrap();
    let err = builder.add_parameter(ParameterDefBuilder::new(
        "first",
        TypeAnnotation::named("Int", /* nullable = */ true),
        Span::builtin(),
    )).unwrap_err();
    assert_spec_note(&err, "#sec-Objects.Type-Validation");
}

// Verifies that every InterfaceTypeBuilder error path attaches
// a spec-reference note: reserved `__` names -> Reserved Names;
// duplicate fields, self-implementation, and duplicate
// `implements` declarations -> Interfaces Type Validation.
//
// See https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// and https://spec.graphql.org/September2025/#sec-Interfaces.Type-Validation
//
// Written by Claude Code, reviewed by a human.
#[test]
fn interface_builder_errors_carry_spec_notes() {
    let err = InterfaceTypeBuilder::new(
        "__Bad", Span::builtin(),
    ).unwrap_err();
    assert_spec_note(&err, "#sec-Names.Reserved-Names");

    let mut builder = InterfaceTypeBuilder::new(
        "Node", Span::builtin(),
    ).unwrap();
    let err = builder.add_field(FieldDefBuilder::new(
        "__bad",
        TypeAnnotation::named("String", /* nullable = */ true),
        Span::builtin(),
    )).unwrap_err();
    assert_spec_note(&err, "#sec-Names.Reserved-Names");

    builder.add_field(FieldDefBuilder::new(
        "id",
        TypeAnnotation::named("ID", /* nullable = */ false),
        Span::builtin(),
    )).unwrap();
    let err = builder.add_field(FieldDefBuilder::new(
        "id",
        TypeAnnotation::named("ID", /* nullable = */ false),
        Span::builtin(),
    )).unwrap_err();
    assert_spec_note(&err, "#sec-Interfaces.Type-Validation");

    let err = builder.add_implements(
        "Node", Span::builtin(),
    ).unwrap_err();
    assert_spec_note(&err, "#sec-Interfaces.Type-Validation");

    builder.add_implements("Base", Span::builtin()).unwrap();
    let err = builder.add_implements(
        "Base", Span::builtin(),
    ).unwrap_err();
    assert_spec_note(&err, "#sec-Interfaces.Type-Validation");
}

// Verifies that every UnionTypeBuilder error path attaches a
// spec-reference note: reserved `__` names -> Reserved Names;
// duplicate members -> Unions Type Validation ("one or more
// unique member types").
//
// See https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// and https://spec.graphql.org/September2025/#sec-Unions.Type-Validation
//
// Written by Claude Code, reviewed by a human.
#[test]
fn union_builder_errors_carry_spec_notes() {
    let err = UnionTypeBuilder::new(
        "__Bad", Span::builtin(),
    ).unwrap_err();
    assert_spec_note(&err, "#sec-Names.Reserved-Names");

    let mut builder = UnionTypeBuilder::new(
        "SearchResult", Span::builtin(),
    ).unwrap();
    builder.add_member("User", Span::builtin()).unwrap();
    let err = builder.add_member(
        "User", Span::builtin(),
    ).unwrap_err();
    assert_spec_note(&err, "#sec-Unions.Type-Validation");
}

// Verifies that every EnumTypeBuilder error path attaches a
// spec-reference note: reserved `__` names -> Reserved Names;
// `true`/`false`/`null` value names -> the EnumValue grammar
// rule ("Name but not true, false or null"); duplicate values
// -> Enums Type Validation ("one or more unique enum values").
//
// See https://spec.graphql.org/September2025/#sec-Names.Reserved-Names,
// https://spec.graphql.org/September2025/#sec-Enum-Value, and
// https://spec.graphql.org/September2025/#sec-Enums.Type-Validation
//
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_builder_errors_carry_spec_notes() {
    let err = EnumTypeBuilder::new(
        "__Bad", Span::builtin(),
    ).unwrap_err();
    assert_spec_note(&err, "#sec-Names.Reserved-Names");

    let mut builder = EnumTypeBuilder::new(
        "Status", Span::builtin(),
    ).unwrap();
    let err = builder.add_value(
        EnumValueDefBuilder::new("null", Span::builtin()),
    ).unwrap_err();
    assert_spec_note(&err, "#sec-Enum-Value");

    builder.add_value(
        EnumValueDefBuilder::new("ACTIVE", Span::builtin()),
    ).unwrap();
    let err = builder.add_value(
        EnumValueDefBuilder::new("ACTIVE", Span::builtin()),
    ).unwrap_err();
    assert_spec_note(&err, "#sec-Enums.Type-Validation");
}

// Verifies that every InputObjectTypeBuilder error path
// attaches a spec-reference note: reserved `__` names ->
// Reserved Names; duplicate input fields -> Input Objects Type
// Validation.
//
// See https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// and https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
//
// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_builder_errors_carry_spec_notes() {
    let err = InputObjectTypeBuilder::new(
        "__Bad", Span::builtin(),
    ).unwrap_err();
    assert_spec_note(&err, "#sec-Names.Reserved-Names");

    let mut builder = InputObjectTypeBuilder::new(
        "CreateInput", Span::builtin(),
    ).unwrap();
    let err = builder.add_field(InputFieldDefBuilder::new(
        "__bad",
        TypeAnnotation::named("String", /* nullable = */ true),
        Span::builtin(),
    )).unwrap_err();
    assert_spec_note(&err, "#sec-Names.Reserved-Names");

    builder.add_field(InputFieldDefBuilder::new(
        "name",
        TypeAnnotation::named("String", /* nullable = */ false),
        Span::builtin(),
    )).unwrap();
    let err = builder.add_field(InputFieldDefBuilder::new(
        "name",
        TypeAnnotation::named("String", /* nullable = */ false),
        Span::builtin(),
    )).unwrap_err();
    assert_spec_note(&err, "#sec-Input-Objects.Type-Validation");
}

// Verifies that ScalarTypeBuilder::new() attaches a
// spec-reference note when rejecting a reserved `__` name.
//
// See https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
//
// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_builder_errors_carry_spec_notes() {
    let err = ScalarTypeBuilder::new(
        "__Bad", Span::builtin(),
    ).unwrap_err();
    assert_spec_note(&err, "#sec-Names.Reserved-Names");
}

// Verifies that every DirectiveBuilder error path attaches a
// spec-reference note: reserved `__` directive/parameter names
// -> Reserved Names; duplicate parameters -> the Directives
// Type Validation rules (rule 5.2: unique argument names).
//
// See https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// and
// https://spec.graphql.org/September2025/#sec-Type-System.Directives.Type-Validation
//
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_builder_errors_carry_spec_notes() {
    let err = DirectiveBuilder::new(
        "__bad", Span::builtin(),
    ).unwrap_err();
    assert_spec_note(&err, "#sec-Names.Reserved-Names");

    let mut builder = DirectiveBuilder::new(
        "auth", Span::builtin(),
    ).unwrap();
    builder.add_location(DirectiveLocationKind::FieldDefinition);
    let err = builder.add_parameter(ParameterDefBuilder::new(
        "__bad",
        TypeAnnotation::named("String", /* nullable = */ true),
        Span::builtin(),
    )).unwrap_err();
    assert_spec_note(&err, "#sec-Names.Reserved-Names");

    builder.add_parameter(ParameterDefBuilder::new(
        "role",
        TypeAnnotation::named("String", /* nullable = */ true),
        Span::builtin(),
    )).unwrap();
    let err = builder.add_parameter(ParameterDefBuilder::new(
        "role",
        TypeAnnotation::named("String", /* nullable = */ true),
        Span::builtin(),
    )).unwrap_err();
    assert_spec_note(
        &err, "#sec-Type-System.Directives.Type-Validation",
    );
}
