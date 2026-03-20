//! Tests for Parts 2.7-2.15: Schema Definitions from the GraphQL parser.
//!
//! These tests verify that the parser correctly parses GraphQL schema
//! definitions including schema definitions, scalar types, object types,
//! interface types, union types, enum types, input object types, directive
//! definitions, and type extensions.
//!
//! Written by Claude Code, reviewed by a human.

use crate::ast;
use crate::tests::ast_utils::extract_first_directive_def;
use crate::tests::ast_utils::extract_first_enum_type;
use crate::tests::ast_utils::extract_first_input_object_type;
use crate::tests::ast_utils::extract_first_interface_type;
use crate::tests::ast_utils::extract_first_object_type;
use crate::tests::ast_utils::extract_first_scalar_type;
use crate::tests::ast_utils::extract_first_type_extension;
use crate::tests::ast_utils::extract_first_union_type;
use crate::tests::ast_utils::extract_schema_def;
use crate::tests::ast_utils::find_root_op;
use crate::tests::utils::parse_schema;

// =============================================================================
// Schema Definitions
// =============================================================================

/// Verifies that a simple schema definition with query root type is parsed.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Schema>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn schema_simple() {
    let sd = extract_schema_def("schema { query: Query }");

    assert!(find_root_op(&sd, ast::OperationKind::Query).is_some());
    assert_eq!(find_root_op(&sd, ast::OperationKind::Query).unwrap(), "Query");
    assert!(find_root_op(&sd, ast::OperationKind::Mutation).is_none());
    assert!(find_root_op(&sd, ast::OperationKind::Subscription).is_none());
}

/// Verifies that a schema definition with query, mutation, and subscription
/// root types is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Schema>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn schema_all_operations() {
    let sd =
        extract_schema_def("schema { query: Q mutation: M subscription: S }");

    assert!(find_root_op(&sd, ast::OperationKind::Query).is_some());
    assert_eq!(find_root_op(&sd, ast::OperationKind::Query).unwrap(), "Q");
    assert!(find_root_op(&sd, ast::OperationKind::Mutation).is_some());
    assert_eq!(find_root_op(&sd, ast::OperationKind::Mutation).unwrap(), "M");
    assert!(find_root_op(&sd, ast::OperationKind::Subscription).is_some());
    assert_eq!(
        find_root_op(&sd, ast::OperationKind::Subscription).unwrap(),
        "S"
    );
}

/// Verifies that a schema definition with directives is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Schema>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn schema_with_directives() {
    let sd = extract_schema_def("schema @deprecated { query: Query }");

    assert_eq!(sd.directives.len(), 1);
    assert_eq!(sd.directives[0].name.value, "deprecated");
}

/// Verifies that an unclosed schema definition produces an error.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Schema>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn schema_unclosed_error() {
    let result = parse_schema("schema { query: Query");
    assert!(result.has_errors());
}

// =============================================================================
// Scalar Types
// =============================================================================

/// Verifies that a simple scalar definition is parsed with the correct name.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Scalars>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_simple() {
    let scalar = extract_first_scalar_type("scalar DateTime");

    assert_eq!(scalar.name.value, "DateTime");
    assert!(scalar.description.is_none());
    assert!(scalar.directives.is_empty());
}

/// Verifies that a scalar with a description is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Scalars>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_with_description() {
    let scalar =
        extract_first_scalar_type(r#""A date and time" scalar DateTime"#);

    assert_eq!(scalar.name.value, "DateTime");
    assert!(scalar.description.is_some());
    assert_eq!(
        scalar.description.as_ref().unwrap().value,
        "A date and time"
    );
}

/// Verifies that a scalar with directives is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Scalars>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_with_directives() {
    let scalar = extract_first_scalar_type(
        r#"scalar JSON @specifiedBy(url: "https://json.org")"#,
    );

    assert_eq!(scalar.name.value, "JSON");
    assert_eq!(scalar.directives.len(), 1);
    assert_eq!(scalar.directives[0].name.value, "specifiedBy");
    assert_eq!(scalar.directives[0].arguments.len(), 1);
}

/// Verifies that keywords like "type" and "query" can be used as scalar names.
///
/// Per GraphQL spec, keywords are contextual and can be used as names:
/// <https://spec.graphql.org/September2025/#sec-Names>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_name_keyword() {
    let scalar_type = extract_first_scalar_type("scalar type");
    assert_eq!(scalar_type.name.value, "type");

    let scalar_query = extract_first_scalar_type("scalar query");
    assert_eq!(scalar_query.name.value, "query");
}

// =============================================================================
// Object Types
// =============================================================================

/// Verifies that a simple object type is parsed with correct name and field.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_simple() {
    let obj = extract_first_object_type("type User { name: String }");

    assert_eq!(obj.name.value, "User");
    assert_eq!(obj.fields.len(), 1);
    assert_eq!(obj.fields[0].name.value, "name");

    match &obj.fields[0].field_type {
        ast::TypeAnnotation::Named(n) => assert_eq!(n.name.value, "String"),
        _ => panic!("Expected Named type annotation"),
    }
}

/// Verifies that an object type with a description is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_with_description() {
    let obj = extract_first_object_type(
        r#""User type" type User { name: String }"#,
    );

    assert_eq!(obj.name.value, "User");
    assert!(obj.description.is_some());
    assert_eq!(obj.description.as_ref().unwrap().value, "User type");
}

/// Verifies that an object type implementing one interface is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_implements_one() {
    let obj =
        extract_first_object_type("type User implements Node { id: ID! }");

    assert_eq!(obj.name.value, "User");
    assert_eq!(obj.implements.len(), 1);
    assert_eq!(obj.implements[0].value, "Node");
}

/// Verifies that an object implementing multiple interfaces is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_implements_multiple() {
    let obj = extract_first_object_type(
        "type User implements Node & Entity { id: ID! }",
    );

    assert_eq!(obj.name.value, "User");
    assert_eq!(obj.implements.len(), 2);
    assert_eq!(obj.implements[0].value, "Node");
    assert_eq!(obj.implements[1].value, "Entity");
}

/// Verifies that a leading ampersand in implements is valid and parsed.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_implements_leading_ampersand() {
    let obj = extract_first_object_type(
        "type User implements & Node & Entity { id: ID! }",
    );

    assert_eq!(obj.name.value, "User");
    assert_eq!(obj.implements.len(), 2);
    assert_eq!(obj.implements[0].value, "Node");
    assert_eq!(obj.implements[1].value, "Entity");
}

/// Verifies that an object type with directives is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_with_directives() {
    let obj =
        extract_first_object_type("type User @deprecated { name: String }");

    assert_eq!(obj.name.value, "User");
    assert_eq!(obj.directives.len(), 1);
    assert_eq!(obj.directives[0].name.value, "deprecated");
}

/// Verifies that an object type with multiple fields is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_multiple_fields() {
    let obj = extract_first_object_type(
        "type User { id: ID! name: String email: String! }",
    );

    assert_eq!(obj.name.value, "User");
    assert_eq!(obj.fields.len(), 3);
    assert_eq!(obj.fields[0].name.value, "id");
    assert_eq!(obj.fields[1].name.value, "name");
    assert_eq!(obj.fields[2].name.value, "email");
}

/// Verifies that a field with arguments is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_field_with_args() {
    let obj =
        extract_first_object_type("type Query { user(id: ID!): User }");

    assert_eq!(obj.name.value, "Query");
    assert_eq!(obj.fields.len(), 1);
    assert_eq!(obj.fields[0].name.value, "user");
    assert_eq!(obj.fields[0].parameters.len(), 1);
    assert_eq!(obj.fields[0].parameters[0].name.value, "id");
}

/// Verifies that a field with a description is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_field_description() {
    let obj = extract_first_object_type(
        r#"type User { "The user's name" name: String }"#,
    );

    assert_eq!(obj.fields.len(), 1);
    assert!(obj.fields[0].description.is_some());
    assert_eq!(
        obj.fields[0].description.as_ref().unwrap().value,
        "The user's name"
    );
}

/// Verifies that a field with directives is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_field_directives() {
    let obj =
        extract_first_object_type("type User { name: String @deprecated }");

    assert_eq!(obj.fields.len(), 1);
    assert_eq!(obj.fields[0].directives.len(), 1);
    assert_eq!(obj.fields[0].directives[0].name.value, "deprecated");
}

/// Verifies that an object type with empty field set `{}` is valid.
///
/// Per GraphQL spec (September 2025), empty field sets are allowed:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_empty_fields() {
    let obj = extract_first_object_type("type User { }");

    assert_eq!(obj.name.value, "User");
    assert!(obj.fields.is_empty());
}

/// Verifies that an object type without body is valid.
///
/// Per GraphQL spec (September 2025), object types can omit the body:
/// <https://spec.graphql.org/September2025/#sec-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn object_no_body() {
    let obj = extract_first_object_type("type User");

    assert_eq!(obj.name.value, "User");
    assert!(obj.fields.is_empty());
}

// =============================================================================
// Interface Types
// =============================================================================

/// Verifies that a simple interface definition is parsed with correct name
/// and fields.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Interfaces>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn interface_simple() {
    let iface = extract_first_interface_type("interface Node { id: ID! }");

    assert_eq!(iface.name.value, "Node");
    assert_eq!(iface.fields.len(), 1);
    assert_eq!(iface.fields[0].name.value, "id");
}

/// Verifies that an interface implementing another interface is parsed.
///
/// Per GraphQL spec (June 2018+), interfaces can implement other interfaces:
/// <https://spec.graphql.org/September2025/#sec-Interfaces>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn interface_implements() {
    let iface = extract_first_interface_type(
        "interface Named implements Node { id: ID! }",
    );

    assert_eq!(iface.name.value, "Named");
    assert_eq!(iface.implements.len(), 1);
    assert_eq!(iface.implements[0].value, "Node");
}

/// Verifies that an interface with multiple fields is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Interfaces>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn interface_with_fields() {
    let iface = extract_first_interface_type(
        "interface Node { id: ID! createdAt: String }",
    );

    assert_eq!(iface.name.value, "Node");
    assert_eq!(iface.fields.len(), 2);
    assert_eq!(iface.fields[0].name.value, "id");
    assert_eq!(iface.fields[1].name.value, "createdAt");
}

/// Verifies that an interface without body is valid.
///
/// Per GraphQL spec (September 2025):
/// <https://spec.graphql.org/September2025/#sec-Interfaces>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn interface_no_body() {
    let iface = extract_first_interface_type("interface Node");

    assert_eq!(iface.name.value, "Node");
    assert!(iface.fields.is_empty());
}

// =============================================================================
// Union Types
// =============================================================================

/// Verifies that a simple union definition with a single member is parsed.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Unions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn union_simple() {
    let union_type =
        extract_first_union_type("union SearchResult = User");

    assert_eq!(union_type.name.value, "SearchResult");
    assert_eq!(union_type.members.len(), 1);
    assert_eq!(union_type.members[0].value, "User");
}

/// Verifies that a union with multiple members is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Unions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn union_multiple_members() {
    let union_type =
        extract_first_union_type("union Result = User | Post | Comment");

    assert_eq!(union_type.name.value, "Result");
    assert_eq!(union_type.members.len(), 3);
    assert_eq!(union_type.members[0].value, "User");
    assert_eq!(union_type.members[1].value, "Post");
    assert_eq!(union_type.members[2].value, "Comment");
}

/// Verifies that a union with a leading pipe is valid and parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Unions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn union_leading_pipe() {
    let union_type =
        extract_first_union_type("union Result = | User | Post");

    assert_eq!(union_type.name.value, "Result");
    assert_eq!(union_type.members.len(), 2);
    assert_eq!(union_type.members[0].value, "User");
    assert_eq!(union_type.members[1].value, "Post");
}

/// Verifies that a union with directives is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Unions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn union_with_directives() {
    let union_type =
        extract_first_union_type("union Result @deprecated = User");

    assert_eq!(union_type.name.value, "Result");
    assert_eq!(union_type.directives.len(), 1);
    assert_eq!(union_type.directives[0].name.value, "deprecated");
}

/// Verifies that a union without members is valid.
///
/// Per GraphQL spec (September 2025):
/// <https://spec.graphql.org/September2025/#sec-Unions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn union_no_members() {
    let union_type = extract_first_union_type("union Empty");

    assert_eq!(union_type.name.value, "Empty");
    assert!(union_type.members.is_empty());
}

// =============================================================================
// Enum Types
// =============================================================================

/// Verifies that a simple enum definition is parsed with correct name and
/// values.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_simple() {
    let enum_type =
        extract_first_enum_type("enum Status { ACTIVE INACTIVE }");

    assert_eq!(enum_type.name.value, "Status");
    assert_eq!(enum_type.values.len(), 2);
    assert_eq!(enum_type.values[0].name.value, "ACTIVE");
    assert_eq!(enum_type.values[1].name.value, "INACTIVE");
}

/// Verifies that an enum with a description is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_with_description() {
    let enum_type =
        extract_first_enum_type(r#""Status enum" enum Status { ACTIVE }"#);

    assert_eq!(enum_type.name.value, "Status");
    assert!(enum_type.description.is_some());
    assert_eq!(enum_type.description.as_ref().unwrap().value, "Status enum");
}

/// Verifies that an enum value with a description is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_description() {
    let enum_type =
        extract_first_enum_type(r#"enum Status { "Active status" ACTIVE }"#);

    assert_eq!(enum_type.values.len(), 1);
    assert!(enum_type.values[0].description.is_some());
    assert_eq!(
        enum_type.values[0].description.as_ref().unwrap().value,
        "Active status"
    );
}

/// Verifies that an enum value with directives is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_directives() {
    let enum_type =
        extract_first_enum_type("enum Status { ACTIVE @deprecated }");

    assert_eq!(enum_type.values.len(), 1);
    assert_eq!(enum_type.values[0].directives.len(), 1);
    assert_eq!(enum_type.values[0].directives[0].name.value, "deprecated");
}

/// Verifies that `true` as an enum value produces an error.
///
/// Per GraphQL spec, `true`, `false`, `null` cannot be enum values:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_true_error() {
    let result = parse_schema("enum Bool { true false }");
    assert!(result.has_errors());
}

/// Verifies that `null` as an enum value produces an error.
///
/// Per GraphQL spec, `true`, `false`, `null` cannot be enum values:
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_value_null_error() {
    let result = parse_schema("enum Maybe { null some }");
    assert!(result.has_errors());
}

/// Verifies that an enum with an empty body `{}` is valid.
///
/// Per GraphQL spec (September 2025):
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_empty_body() {
    let enum_type = extract_first_enum_type("enum Status { }");

    assert_eq!(enum_type.name.value, "Status");
    assert!(enum_type.values.is_empty());
}

/// Verifies that an enum without body is valid.
///
/// Per GraphQL spec (September 2025):
/// <https://spec.graphql.org/September2025/#sec-Enums>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn enum_no_body() {
    let enum_type = extract_first_enum_type("enum Status");

    assert_eq!(enum_type.name.value, "Status");
    assert!(enum_type.values.is_empty());
}

// =============================================================================
// Input Object Types
// =============================================================================

/// Verifies that a simple input definition is parsed with correct name and
/// fields.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_simple() {
    let input =
        extract_first_input_object_type("input CreateUserInput { name: String! }");

    assert_eq!(input.name.value, "CreateUserInput");
    assert_eq!(input.fields.len(), 1);
    assert_eq!(input.fields[0].name.value, "name");
}

/// Verifies that an input field with a default value is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_with_defaults() {
    let input =
        extract_first_input_object_type("input I { limit: Int = 10 }");

    assert_eq!(input.fields.len(), 1);
    assert_eq!(input.fields[0].name.value, "limit");
    assert!(input.fields[0].default_value.is_some());

    match input.fields[0].default_value.as_ref().unwrap() {
        ast::Value::Int(n) => assert_eq!(n.as_i64(), 10),
        other => panic!("Expected Int default value, got: {other:?}"),
    }
}

/// Verifies that an input field with directives is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Input-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_field_directives() {
    let input =
        extract_first_input_object_type("input I { name: String @deprecated }");

    assert_eq!(input.fields.len(), 1);
    assert_eq!(input.fields[0].directives.len(), 1);
    assert_eq!(input.fields[0].directives[0].name.value, "deprecated");
}

/// Verifies that an input object with an empty body `{}` is valid.
///
/// Per GraphQL spec (September 2025):
/// <https://spec.graphql.org/September2025/#sec-Input-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_empty_body() {
    let input = extract_first_input_object_type("input I { }");

    assert_eq!(input.name.value, "I");
    assert!(input.fields.is_empty());
}

/// Verifies that an input object without body is valid.
///
/// Per GraphQL spec (September 2025):
/// <https://spec.graphql.org/September2025/#sec-Input-Objects>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn input_no_body() {
    let input = extract_first_input_object_type("input I");

    assert_eq!(input.name.value, "I");
    assert!(input.fields.is_empty());
}

// =============================================================================
// Directive Definitions
// =============================================================================

/// Verifies that a simple directive definition is parsed with correct name
/// and location.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_def_simple() {
    let directive =
        extract_first_directive_def("directive @deprecated on FIELD_DEFINITION");

    assert_eq!(directive.name.value, "deprecated");
    assert_eq!(directive.locations.len(), 1);
    assert_eq!(
        directive.locations[0].kind,
        ast::DirectiveLocationKind::FieldDefinition
    );
}

/// Verifies that a directive with multiple locations is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_def_multiple_locations() {
    let directive =
        extract_first_directive_def("directive @d on FIELD | OBJECT");

    assert_eq!(directive.name.value, "d");
    assert_eq!(directive.locations.len(), 2);
    assert_eq!(
        directive.locations[0].kind,
        ast::DirectiveLocationKind::Field
    );
    assert_eq!(
        directive.locations[1].kind,
        ast::DirectiveLocationKind::Object
    );
}

/// Verifies that a leading pipe in directive locations is valid.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_def_leading_pipe() {
    let directive =
        extract_first_directive_def("directive @d on | FIELD | OBJECT");

    assert_eq!(directive.name.value, "d");
    assert_eq!(directive.locations.len(), 2);
}

/// Verifies that a directive definition with arguments is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_def_with_args() {
    let directive = extract_first_directive_def(
        "directive @deprecated(reason: String) on FIELD_DEFINITION",
    );

    assert_eq!(directive.name.value, "deprecated");
    assert_eq!(directive.arguments.len(), 1);
    assert_eq!(directive.arguments[0].name.value, "reason");
}

/// Verifies that a `repeatable` directive definition is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_def_repeatable() {
    let directive =
        extract_first_directive_def("directive @tag repeatable on OBJECT");

    assert_eq!(directive.name.value, "tag");
    assert!(directive.repeatable);
}

/// Verifies that an unknown directive location produces an error.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System.Directives>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn directive_def_unknown_location_error() {
    let result = parse_schema("directive @d on FOOBAR");
    assert!(result.has_errors());
}

// =============================================================================
// Type Extensions
// =============================================================================

/// Verifies that a scalar extension is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_scalar() {
    let ext = extract_first_type_extension(
        r#"extend scalar DateTime @specifiedBy(url: "https://example.com")"#,
    );

    match ext {
        ast::TypeExtension::Scalar(scalar_ext) => {
            assert_eq!(scalar_ext.name.value, "DateTime");
            assert_eq!(scalar_ext.directives.len(), 1);
            assert_eq!(scalar_ext.directives[0].name.value, "specifiedBy");
        },
        _ => panic!("Expected ScalarTypeExtension"),
    }
}

/// Verifies that a type extension adding fields is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_type_add_fields() {
    let ext =
        extract_first_type_extension("extend type User { age: Int }");

    match ext {
        ast::TypeExtension::Object(obj_ext) => {
            assert_eq!(obj_ext.name.value, "User");
            assert_eq!(obj_ext.fields.len(), 1);
            assert_eq!(obj_ext.fields[0].name.value, "age");
        },
        _ => panic!("Expected ObjectTypeExtension"),
    }
}

/// Verifies that a type extension adding implements is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_type_add_implements() {
    let ext = extract_first_type_extension(
        "extend type User implements NewInterface",
    );

    match ext {
        ast::TypeExtension::Object(obj_ext) => {
            assert_eq!(obj_ext.name.value, "User");
            assert_eq!(obj_ext.implements.len(), 1);
            assert_eq!(obj_ext.implements[0].value, "NewInterface");
        },
        _ => panic!("Expected ObjectTypeExtension"),
    }
}

/// Verifies that a type extension adding directives is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_type_add_directives() {
    let ext =
        extract_first_type_extension("extend type User @deprecated");

    match ext {
        ast::TypeExtension::Object(obj_ext) => {
            assert_eq!(obj_ext.name.value, "User");
            assert_eq!(obj_ext.directives.len(), 1);
            assert_eq!(obj_ext.directives[0].name.value, "deprecated");
        },
        _ => panic!("Expected ObjectTypeExtension"),
    }
}

/// Verifies that an interface extension is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_interface() {
    let ext = extract_first_type_extension(
        "extend interface Node { extra: String }",
    );

    match ext {
        ast::TypeExtension::Interface(iface_ext) => {
            assert_eq!(iface_ext.name.value, "Node");
            assert_eq!(iface_ext.fields.len(), 1);
            assert_eq!(iface_ext.fields[0].name.value, "extra");
        },
        _ => panic!("Expected InterfaceTypeExtension"),
    }
}

/// Verifies that a union extension is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_union() {
    let ext =
        extract_first_type_extension("extend union Result = NewType");

    match ext {
        ast::TypeExtension::Union(union_ext) => {
            assert_eq!(union_ext.name.value, "Result");
            assert_eq!(union_ext.members.len(), 1);
            assert_eq!(union_ext.members[0].value, "NewType");
        },
        _ => panic!("Expected UnionTypeExtension"),
    }
}

/// Verifies that an enum extension is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_enum() {
    let ext =
        extract_first_type_extension("extend enum Status { PENDING }");

    match ext {
        ast::TypeExtension::Enum(enum_ext) => {
            assert_eq!(enum_ext.name.value, "Status");
            assert_eq!(enum_ext.values.len(), 1);
            assert_eq!(enum_ext.values[0].name.value, "PENDING");
        },
        _ => panic!("Expected EnumTypeExtension"),
    }
}

/// Verifies that an input extension is parsed correctly.
///
/// Per GraphQL spec:
/// <https://spec.graphql.org/September2025/#sec-Type-System-Extensions>
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn extend_input() {
    let ext = extract_first_type_extension(
        "extend input CreateUserInput { extra: String }",
    );

    match ext {
        ast::TypeExtension::InputObject(input_ext) => {
            assert_eq!(input_ext.name.value, "CreateUserInput");
            assert_eq!(input_ext.fields.len(), 1);
            assert_eq!(input_ext.fields[0].name.value, "extra");
        },
        _ => panic!("Expected InputObjectTypeExtension"),
    }
}
