use crate::error_note::ErrorNoteKind;
use crate::operation_kind::OperationKind;
use crate::schema::SchemaBuildErrorKind;
use crate::schema::SchemaBuilder;
use crate::schema::TypeValidationErrorKind;
use crate::types::GraphQLTypeKind;

// ---------------------------------------------------------
// Object type extensions
// ---------------------------------------------------------

// Verifies that an object type extension merges its fields and
// directive annotations into the target object type.
//
// See https://spec.graphql.org/September2025/#sec-Object-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn object_extension_merges_fields_and_directives() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend type Query @tag { y: String }",
    ).unwrap();
    let query = schema.object_type("Query").unwrap();
    assert!(query.fields().contains_key("x"));
    assert!(query.fields().contains_key("y"));
    assert!(
        query.directives().iter().any(|d| d.name().as_str() == "tag"),
        "extension directive should be merged onto the type",
    );
}

// Verifies that an object type extension appearing textually
// BEFORE the target type's definition is deferred and then
// merged once the definition is loaded (v0-parity deferred
// extension behavior).
//
// See https://spec.graphql.org/September2025/#sec-Object-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn object_extension_before_definition_merges() {
    let schema = SchemaBuilder::build_from_str(
        "extend type Query { y: String }\n\
         type Query { x: Int }",
    ).unwrap();
    let query = schema.object_type("Query").unwrap();
    assert!(query.fields().contains_key("x"));
    assert!(query.fields().contains_key("y"));
}

// Verifies that an object type extension whose target type is
// never defined produces an ExtensionOfUndefinedType error at
// build() time, and that the error carries a spec-reference
// note.
//
// See https://spec.graphql.org/September2025/#sec-Object-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn object_extension_of_undefined_type_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend type Missing { y: String }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let ext_error = errors.errors().iter().find(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::ExtensionOfUndefinedType {
                type_name,
            } if type_name == "Missing",
        )
    });
    let ext_error = ext_error
        .expect("expected ExtensionOfUndefinedType for `Missing`");
    assert!(
        ext_error.notes().iter().any(|n| {
            n.kind == ErrorNoteKind::Spec
        }),
        "expected a spec-reference note on the error",
    );
}

// Verifies that `extend type` applied to a non-object type
// (here: an enum) produces an InvalidExtensionTypeKind error
// carrying both the extension kind and the actual type kind,
// and that the extension is not applied.
//
// See https://spec.graphql.org/September2025/#sec-Object-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn object_extension_kind_mismatch_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         enum Color { RED }\n\
         extend type Color { y: String }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::InvalidExtensionTypeKind {
                actual_kind: GraphQLTypeKind::Enum,
                extension_kind: GraphQLTypeKind::Object,
                type_name,
            } if type_name == "Color",
        )
    });
    assert!(has_error, "expected InvalidExtensionTypeKind for `Color`");
}

// Verifies that an object type extension contributing a field
// that already exists on the target type produces a
// DuplicateFieldNameDefinition error.
//
// See https://spec.graphql.org/September2025/#sec-Object-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn object_extension_duplicate_field_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend type Query { x: Int }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::DuplicateFieldNameDefinition {
                field_name,
                type_name,
            } if field_name == "x" && type_name == "Query",
        )
    });
    assert!(has_error, "expected DuplicateFieldNameDefinition");
}

// Verifies that an object type extension can add an
// `implements` declaration, and that the resulting schema
// reports the object as implementing the interface.
//
// See https://spec.graphql.org/September2025/#sec-Object-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn object_extension_merges_implements_declaration() {
    let schema = SchemaBuilder::build_from_str(
        "interface Node { id: ID }\n\
         type Query { id: ID }\n\
         extend type Query implements Node",
    ).unwrap();
    let query = schema.object_type("Query").unwrap();
    assert!(
        query.interfaces().iter().any(|l| l.value.as_str() == "Node"),
        "extension `implements Node` should be merged",
    );
    assert!(
        schema.types_implementing("Node").iter().any(|t| {
            t.name().as_str() == "Query"
        }),
    );
}

// Verifies that an object type extension re-declaring an
// interface the target already implements produces a
// DuplicateInterfaceImplementsDeclaration error.
//
// See https://spec.graphql.org/September2025/#sec-Object-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn object_extension_duplicate_implements_fails() {
    let result = SchemaBuilder::build_from_str(
        "interface Node { id: ID }\n\
         type Query implements Node { id: ID }\n\
         extend type Query implements Node",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::DuplicateInterfaceImplementsDeclaration {
                interface_name,
                type_name,
            } if interface_name == "Node" && type_name == "Query",
        )
    });
    assert!(
        has_error,
        "expected DuplicateInterfaceImplementsDeclaration",
    );
}

// Verifies that an object type extension contributing a
// `__`-prefixed field name is rejected with an
// InvalidDunderPrefixedFieldName error (matching the check
// performed for fields contributed by type definitions).
//
// See https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
//
// Written by Claude Code, reviewed by a human.
#[test]
fn object_extension_dunder_field_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend type Query { __secret: Int }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::InvalidDunderPrefixedFieldName {
                field_name,
                type_name,
            } if field_name == "__secret" && type_name == "Query",
        )
    });
    assert!(has_error, "expected InvalidDunderPrefixedFieldName");
}

// ---------------------------------------------------------
// Interface type extensions
// ---------------------------------------------------------

// Verifies that an interface type extension merges its fields
// and directive annotations into the target interface type.
//
// See https://spec.graphql.org/September2025/#sec-Interface-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn interface_extension_merges_fields_and_directives() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         interface Node { id: ID }\n\
         extend interface Node @tag { name: String }",
    ).unwrap();
    let node = schema.interface_type("Node").unwrap();
    assert!(node.fields().contains_key("id"));
    assert!(node.fields().contains_key("name"));
    assert!(node.directives().iter().any(|d| d.name().as_str() == "tag"));
}

// Verifies that an interface type extension appearing textually
// BEFORE the interface's definition is deferred and merged once
// the definition is loaded.
//
// See https://spec.graphql.org/September2025/#sec-Interface-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn interface_extension_before_definition_merges() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend interface Node { name: String }\n\
         interface Node { id: ID }",
    ).unwrap();
    let node = schema.interface_type("Node").unwrap();
    assert!(node.fields().contains_key("id"));
    assert!(node.fields().contains_key("name"));
}

// Verifies that an interface type extension whose target type
// is never defined produces an ExtensionOfUndefinedType error
// at build() time.
//
// See https://spec.graphql.org/September2025/#sec-Interface-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn interface_extension_of_undefined_type_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend interface Missing { name: String }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::ExtensionOfUndefinedType {
                type_name,
            } if type_name == "Missing",
        )
    });
    assert!(has_error, "expected ExtensionOfUndefinedType");
}

// Verifies that `extend interface` applied to an object type
// produces an InvalidExtensionTypeKind error.
//
// See https://spec.graphql.org/September2025/#sec-Interface-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn interface_extension_kind_mismatch_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend interface Query { name: String }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::InvalidExtensionTypeKind {
                actual_kind: GraphQLTypeKind::Object,
                extension_kind: GraphQLTypeKind::Interface,
                type_name,
            } if type_name == "Query",
        )
    });
    assert!(has_error, "expected InvalidExtensionTypeKind");
}

// Verifies that an interface type extension contributing a
// field that already exists on the target interface produces a
// DuplicateFieldNameDefinition error.
//
// See https://spec.graphql.org/September2025/#sec-Interface-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn interface_extension_duplicate_field_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         interface Node { id: ID }\n\
         extend interface Node { id: ID }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::DuplicateFieldNameDefinition {
                field_name,
                type_name,
            } if field_name == "id" && type_name == "Node",
        )
    });
    assert!(has_error, "expected DuplicateFieldNameDefinition");
}

// ---------------------------------------------------------
// Enum type extensions
// ---------------------------------------------------------

// Verifies that an enum type extension merges its values and
// directive annotations into the target enum type.
//
// See https://spec.graphql.org/September2025/#sec-Enum-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_extension_merges_values_and_directives() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         enum Color { RED }\n\
         extend enum Color @tag { GREEN BLUE }",
    ).unwrap();
    let color = schema.enum_type("Color").unwrap();
    assert!(color.values().contains_key("RED"));
    assert!(color.values().contains_key("GREEN"));
    assert!(color.values().contains_key("BLUE"));
    assert!(color.directives().iter().any(|d| d.name().as_str() == "tag"));
}

// Verifies that an enum type extension appearing textually
// BEFORE the enum's definition is deferred and merged once the
// definition is loaded.
//
// See https://spec.graphql.org/September2025/#sec-Enum-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_extension_before_definition_merges() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend enum Color { GREEN }\n\
         enum Color { RED }",
    ).unwrap();
    let color = schema.enum_type("Color").unwrap();
    assert!(color.values().contains_key("RED"));
    assert!(color.values().contains_key("GREEN"));
}

// Verifies that an enum type extension whose target type is
// never defined produces an ExtensionOfUndefinedType error at
// build() time.
//
// See https://spec.graphql.org/September2025/#sec-Enum-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_extension_of_undefined_type_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend enum Missing { GREEN }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::ExtensionOfUndefinedType {
                type_name,
            } if type_name == "Missing",
        )
    });
    assert!(has_error, "expected ExtensionOfUndefinedType");
}

// Verifies that `extend enum` applied to an object type
// produces an InvalidExtensionTypeKind error.
//
// See https://spec.graphql.org/September2025/#sec-Enum-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_extension_kind_mismatch_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend enum Query { GREEN }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::InvalidExtensionTypeKind {
                actual_kind: GraphQLTypeKind::Object,
                extension_kind: GraphQLTypeKind::Enum,
                type_name,
            } if type_name == "Query",
        )
    });
    assert!(has_error, "expected InvalidExtensionTypeKind");
}

// Verifies that an enum type extension contributing a value
// that already exists on the target enum produces a
// DuplicateEnumValueDefinition error.
//
// See https://spec.graphql.org/September2025/#sec-Enum-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_extension_duplicate_value_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         enum Color { RED }\n\
         extend enum Color { RED }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::DuplicateEnumValueDefinition {
                type_name,
                value_name,
            } if type_name == "Color" && value_name == "RED",
        )
    });
    assert!(has_error, "expected DuplicateEnumValueDefinition");
}

// ---------------------------------------------------------
// Union type extensions
// ---------------------------------------------------------

// Verifies that a union type extension merges its members and
// directive annotations into the target union type.
//
// See https://spec.graphql.org/September2025/#sec-Union-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn union_extension_merges_members_and_directives() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         type A { a: Int }\n\
         type B { b: Int }\n\
         union U = A\n\
         extend union U @tag = B",
    ).unwrap();
    let u = schema.union_type("U").unwrap();
    assert!(u.members().iter().any(|m| m.value.as_str() == "A"));
    assert!(u.members().iter().any(|m| m.value.as_str() == "B"));
    assert!(u.directives().iter().any(|d| d.name().as_str() == "tag"));
}

// Verifies that a union type extension appearing textually
// BEFORE the union's definition is deferred and merged once the
// definition is loaded.
//
// See https://spec.graphql.org/September2025/#sec-Union-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn union_extension_before_definition_merges() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         type A { a: Int }\n\
         type B { b: Int }\n\
         extend union U = B\n\
         union U = A",
    ).unwrap();
    let u = schema.union_type("U").unwrap();
    assert!(u.members().iter().any(|m| m.value.as_str() == "A"));
    assert!(u.members().iter().any(|m| m.value.as_str() == "B"));
}

// Verifies that a union type extension whose target type is
// never defined produces an ExtensionOfUndefinedType error at
// build() time.
//
// See https://spec.graphql.org/September2025/#sec-Union-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn union_extension_of_undefined_type_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         type B { b: Int }\n\
         extend union Missing = B",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::ExtensionOfUndefinedType {
                type_name,
            } if type_name == "Missing",
        )
    });
    assert!(has_error, "expected ExtensionOfUndefinedType");
}

// Verifies that `extend union` applied to an object type
// produces an InvalidExtensionTypeKind error.
//
// See https://spec.graphql.org/September2025/#sec-Union-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn union_extension_kind_mismatch_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         type B { b: Int }\n\
         extend union Query = B",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::InvalidExtensionTypeKind {
                actual_kind: GraphQLTypeKind::Object,
                extension_kind: GraphQLTypeKind::Union,
                type_name,
            } if type_name == "Query",
        )
    });
    assert!(has_error, "expected InvalidExtensionTypeKind");
}

// Verifies that a union type extension contributing a member
// that already exists on the target union produces a
// DuplicateUnionMember error.
//
// See https://spec.graphql.org/September2025/#sec-Union-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn union_extension_duplicate_member_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         type A { a: Int }\n\
         union U = A\n\
         extend union U = A",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::DuplicateUnionMember {
                member_name,
                type_name,
            } if member_name == "A" && type_name == "U",
        )
    });
    assert!(has_error, "expected DuplicateUnionMember");
}

// ---------------------------------------------------------
// Input object type extensions
// ---------------------------------------------------------

// Verifies that an input object type extension merges its input
// fields and directive annotations into the target input object
// type.
//
// See https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_extension_merges_fields_and_directives() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         input In { a: Int }\n\
         extend input In @tag { b: String }",
    ).unwrap();
    let input = schema.input_object_type("In").unwrap();
    assert!(input.fields().contains_key("a"));
    assert!(input.fields().contains_key("b"));
    assert!(input.directives().iter().any(|d| d.name().as_str() == "tag"));
}

// Verifies that an input object type extension appearing
// textually BEFORE the input object's definition is deferred
// and merged once the definition is loaded.
//
// See https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_extension_before_definition_merges() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend input In { b: String }\n\
         input In { a: Int }",
    ).unwrap();
    let input = schema.input_object_type("In").unwrap();
    assert!(input.fields().contains_key("a"));
    assert!(input.fields().contains_key("b"));
}

// Verifies that an input object type extension whose target
// type is never defined produces an ExtensionOfUndefinedType
// error at build() time.
//
// See https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_extension_of_undefined_type_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend input Missing { b: String }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::ExtensionOfUndefinedType {
                type_name,
            } if type_name == "Missing",
        )
    });
    assert!(has_error, "expected ExtensionOfUndefinedType");
}

// Verifies that `extend input` applied to an object type
// produces an InvalidExtensionTypeKind error.
//
// See https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_extension_kind_mismatch_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend input Query { b: String }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::InvalidExtensionTypeKind {
                actual_kind: GraphQLTypeKind::Object,
                extension_kind: GraphQLTypeKind::InputObject,
                type_name,
            } if type_name == "Query",
        )
    });
    assert!(has_error, "expected InvalidExtensionTypeKind");
}

// Verifies that an input object type extension contributing an
// input field that already exists on the target type produces a
// DuplicateFieldNameDefinition error.
//
// See https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_extension_duplicate_field_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         input In { a: Int }\n\
         extend input In { a: Int }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::DuplicateFieldNameDefinition {
                field_name,
                type_name,
            } if field_name == "a" && type_name == "In",
        )
    });
    assert!(has_error, "expected DuplicateFieldNameDefinition");
}

// Verifies that an input object type extension contributing a
// `__`-prefixed input field name is rejected with an
// InvalidDunderPrefixedFieldName error.
//
// See https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
//
// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_extension_dunder_field_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         input In { a: Int }\n\
         extend input In { __b: Int }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::InvalidDunderPrefixedFieldName {
                field_name,
                type_name,
            } if field_name == "__b" && type_name == "In",
        )
    });
    assert!(has_error, "expected InvalidDunderPrefixedFieldName");
}

// Verifies Input Object Extensions rule 5: "The `@oneOf`
// directive must not be provided by an Input Object type
// extension." Providing it via `extend input` is an error
// regardless of the fields' nullability, and the directive is
// NOT merged onto the type (an otherwise-valid all-nullable
// input does not silently become a oneOf input).
//
// https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
// Written by Claude Code, reviewed by a human.
#[test]
fn oneof_via_extension_rejected() {
    // All-nullable fields: without rule 5 this would build
    // successfully as a spec-invalid oneOf input object.
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         input X { a: Int }\n\
         extend input X @oneOf",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::OneOfDirectiveProvidedByInputObjectExtension {
                type_name,
            } if type_name == "X",
        )
    });
    assert!(
        has_error,
        "expected OneOfDirectiveProvidedByInputObjectExtension \
        for `X`",
    );
}

// Verifies that when `@oneOf` arrives via an extension (rule 5
// violation), the directive is not merged, so the oneOf
// field-nullability constraints do NOT additionally fire — the
// rule-5 error is the only oneOf-related error even when the
// original definition has a non-nullable field.
//
// https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
// Written by Claude Code, reviewed by a human.
#[test]
fn oneof_via_extension_not_merged_no_constraint_errors() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         input X { a: Int! }\n\
         extend input X @oneOf",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_rule5_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::OneOfDirectiveProvidedByInputObjectExtension { .. },
        )
    });
    let has_constraint_error = errors.errors().iter().any(|e| {
        if let SchemaBuildErrorKind::TypeValidation(tve) = e.kind() {
            matches!(
                tve.kind(),
                TypeValidationErrorKind::InvalidNonNullableOneOfInputField { .. },
            )
        } else {
            false
        }
    });
    assert!(has_rule5_error, "expected the rule-5 error");
    assert!(
        !has_constraint_error,
        "the unmerged @oneOf must not trigger field constraints",
    );
}

// Verifies Input Object Extensions rule 6 territory: extending
// an input object that is ALREADY `@oneOf` with a non-nullable
// field is rejected — validation runs over the fully-merged
// type, so extension-contributed fields are subject to the
// oneOf constraints (merge-before-validate ordering).
//
// https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
// Written by Claude Code, reviewed by a human.
#[test]
fn extension_field_on_oneof_input_subject_to_constraints() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         input X @oneOf { a: Int }\n\
         extend input X { b: Int! }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        if let SchemaBuildErrorKind::TypeValidation(tve) = e.kind() {
            matches!(
                tve.kind(),
                TypeValidationErrorKind::InvalidNonNullableOneOfInputField {
                    field_name,
                    parent_type_name,
                } if field_name == "b" && parent_type_name == "X",
            )
        } else {
            false
        }
    });
    assert!(
        has_error,
        "expected InvalidNonNullableOneOfInputField for the \
        extension-contributed field `X.b`",
    );
}

// Verifies that `extend schema` works against a schema with no
// explicit `schema { }` definition (the schema is implicitly
// defined by its root types, matching graphql-js's permissive
// handling of §3.3.2 rule 1).
//
// https://spec.graphql.org/September2025/#sec-Schema-Extension
// Written by Claude Code, reviewed by a human.
#[test]
fn extend_schema_with_implicit_schema_definition() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         type M { y: Int }\n\
         extend schema { mutation: M }",
    ).unwrap();
    assert_eq!(
        schema.mutation_type().map(|t| t.name().as_str()),
        Some("M"),
    );
}

// ---------------------------------------------------------
// Scalar type extensions
// ---------------------------------------------------------

// Verifies that a scalar type extension merges its directive
// annotations into the target scalar type (directives are the
// only contribution a scalar extension can make).
//
// See https://spec.graphql.org/September2025/#sec-Scalar-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_extension_merges_directives() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         scalar Date\n\
         extend scalar Date @specifiedBy(url: \"https://example.com\")",
    ).unwrap();
    let date = schema.scalar_type("Date").unwrap();
    assert!(
        date.directives().iter().any(|d| d.name().as_str() == "specifiedBy"),
        "extension directive should be merged onto the scalar",
    );
}

// Verifies that a scalar type extension appearing textually
// BEFORE the scalar's definition is deferred and merged once
// the definition is loaded.
//
// See https://spec.graphql.org/September2025/#sec-Scalar-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_extension_before_definition_merges() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend scalar Date @specifiedBy(url: \"https://example.com\")\n\
         scalar Date",
    ).unwrap();
    let date = schema.scalar_type("Date").unwrap();
    assert!(
        date.directives().iter().any(|d| d.name().as_str() == "specifiedBy"),
    );
}

// Verifies that a scalar type extension whose target type is
// never defined produces an ExtensionOfUndefinedType error at
// build() time.
//
// See https://spec.graphql.org/September2025/#sec-Scalar-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_extension_of_undefined_type_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend scalar Missing @specifiedBy(url: \"https://example.com\")",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::ExtensionOfUndefinedType {
                type_name,
            } if type_name == "Missing",
        )
    });
    assert!(has_error, "expected ExtensionOfUndefinedType");
}

// Verifies that `extend scalar` applied to an object type
// produces an InvalidExtensionTypeKind error.
//
// See https://spec.graphql.org/September2025/#sec-Scalar-Extensions
//
// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_extension_kind_mismatch_fails() {
    let result = SchemaBuilder::build_from_str(
        "type Query { x: Int }\n\
         extend scalar Query @specifiedBy(url: \"https://example.com\")",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::InvalidExtensionTypeKind {
                actual_kind: GraphQLTypeKind::Object,
                extension_kind: GraphQLTypeKind::Scalar,
                type_name,
            } if type_name == "Query",
        )
    });
    assert!(has_error, "expected InvalidExtensionTypeKind");
}

// ---------------------------------------------------------
// Schema extensions
// ---------------------------------------------------------

// Verifies that `extend schema { mutation: ... }` merges a new
// root operation type binding into the schema alongside the
// bindings from the original `schema { ... }` definition.
//
// See https://spec.graphql.org/September2025/#sec-Schema-Extension
//
// Written by Claude Code, reviewed by a human.
#[test]
fn schema_extension_merges_root_operations() {
    let schema = SchemaBuilder::build_from_str(
        "schema { query: Q }\n\
         type Q { x: Int }\n\
         type M { doThing: Boolean }\n\
         extend schema { mutation: M }",
    ).unwrap();
    assert_eq!(schema.query_type_name().as_str(), "Q");
    assert_eq!(
        schema.mutation_type_name().unwrap().as_str(),
        "M",
    );
    assert!(schema.mutation_type().is_some());
}

// Verifies that `extend schema` re-binding an already-bound
// root operation kind produces a DuplicateOperationDefinition
// error (same handling as duplicates within `schema { ... }`
// definitions).
//
// See https://spec.graphql.org/September2025/#sec-Schema-Extension
//
// Written by Claude Code, reviewed by a human.
#[test]
fn schema_extension_duplicate_root_operation_fails() {
    let result = SchemaBuilder::build_from_str(
        "schema { query: Q }\n\
         type Q { x: Int }\n\
         type Q2 { y: Int }\n\
         extend schema { query: Q2 }",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    let has_error = errors.errors().iter().any(|e| {
        matches!(
            e.kind(),
            SchemaBuildErrorKind::DuplicateOperationDefinition {
                operation: OperationKind::Query,
                type_name,
            } if type_name == "Q",
        )
    });
    assert!(has_error, "expected DuplicateOperationDefinition");
}
