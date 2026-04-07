use crate::schema::SchemaBuildErrorKind;
use crate::span::SourceMapId;
use crate::type_builders::DirectiveBuilder;
use crate::type_builders::EnumTypeBuilder;
use crate::type_builders::InputObjectTypeBuilder;
use crate::type_builders::InterfaceTypeBuilder;
use crate::type_builders::ObjectTypeBuilder;
use crate::type_builders::ScalarTypeBuilder;
use crate::type_builders::UnionTypeBuilder;
use libgraphql_parser::ast::Document;

fn parse_schema_static(
    source: &'static str,
) -> Document<'static> {
    libgraphql_parser::parse_schema(source).into_ast()
}

fn extract_type_def<'a, 'src>(
    doc: &'a Document<'src>,
    index: usize,
) -> &'a libgraphql_parser::ast::TypeDefinition<'src> {
    match &doc.definitions[index] {
        libgraphql_parser::ast::Definition::TypeDefinition(td) => td,
        _ => panic!("expected type definition at index {index}"),
    }
}

// Verifies from_ast() correctly converts a parsed object type
// with implements, fields, and directives.
// https://spec.graphql.org/September2025/#sec-Objects
// Written by Claude Code, reviewed by a human.
#[test]
fn object_type_from_ast() {
    let doc = parse_schema_static(
        "type User implements Node { id: ID!, name: String }",
    );
    let td = extract_type_def(&doc, 0);
    let ast_obj = match td {
        libgraphql_parser::ast::TypeDefinition::Object(obj) => obj,
        _ => panic!("expected object type definition"),
    };
    let builder = ObjectTypeBuilder::from_ast(
        ast_obj, SourceMapId(1),
    );
    assert!(builder.errors.is_empty());
    assert_eq!(builder.name.as_str(), "User");
    assert_eq!(builder.implements.len(), 1);
    assert_eq!(builder.implements[0].value.as_str(), "Node");
    assert_eq!(builder.fields.len(), 2);
    assert_eq!(builder.fields[0].name.as_str(), "id");
    assert_eq!(builder.fields[1].name.as_str(), "name");
}

// Verifies from_ast() for an interface type with implements.
// https://spec.graphql.org/September2025/#sec-Interfaces
// Written by Claude Code, reviewed by a human.
#[test]
fn interface_type_from_ast() {
    let doc = parse_schema_static(
        "interface Resource implements Node { id: ID! }",
    );
    let td = extract_type_def(&doc, 0);
    let ast_iface = match td {
        libgraphql_parser::ast::TypeDefinition::Interface(i) => i,
        _ => panic!("expected interface type definition"),
    };
    let builder = InterfaceTypeBuilder::from_ast(
        ast_iface, SourceMapId(1),
    );
    assert!(builder.errors.is_empty());
    assert_eq!(builder.name.as_str(), "Resource");
    assert_eq!(builder.implements.len(), 1);
    assert_eq!(builder.fields.len(), 1);
}

// Verifies from_ast() for a union type.
// https://spec.graphql.org/September2025/#sec-Unions
// Written by Claude Code, reviewed by a human.
#[test]
fn union_type_from_ast() {
    let doc = parse_schema_static(
        "union SearchResult = User | Post",
    );
    let td = extract_type_def(&doc, 0);
    let ast_union = match td {
        libgraphql_parser::ast::TypeDefinition::Union(u) => u,
        _ => panic!("expected union type definition"),
    };
    let builder = UnionTypeBuilder::from_ast(
        ast_union, SourceMapId(1),
    );
    assert!(builder.errors.is_empty());
    assert_eq!(builder.name.as_str(), "SearchResult");
    assert_eq!(builder.members.len(), 2);
    assert_eq!(builder.members[0].value.as_str(), "User");
    assert_eq!(builder.members[1].value.as_str(), "Post");
}

// Verifies from_ast() for an enum type with values.
// https://spec.graphql.org/September2025/#sec-Enums
// Written by Claude Code, reviewed by a human.
#[test]
fn enum_type_from_ast() {
    let doc = parse_schema_static(
        "enum Status { ACTIVE, INACTIVE }",
    );
    let td = extract_type_def(&doc, 0);
    let ast_enum = match td {
        libgraphql_parser::ast::TypeDefinition::Enum(e) => e,
        _ => panic!("expected enum type definition"),
    };
    let builder = EnumTypeBuilder::from_ast(
        ast_enum, SourceMapId(1),
    );
    assert!(builder.errors.is_empty());
    assert_eq!(builder.name.as_str(), "Status");
    assert_eq!(builder.values.len(), 2);
    assert_eq!(builder.values[0].name.as_str(), "ACTIVE");
    assert_eq!(builder.values[1].name.as_str(), "INACTIVE");
}

// Verifies from_ast() for a scalar type.
// https://spec.graphql.org/September2025/#sec-Scalars
// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_type_from_ast() {
    let doc = parse_schema_static(
        "scalar DateTime",
    );
    let td = extract_type_def(&doc, 0);
    let ast_scalar = match td {
        libgraphql_parser::ast::TypeDefinition::Scalar(s) => s,
        _ => panic!("expected scalar type definition"),
    };
    let builder = ScalarTypeBuilder::from_ast(
        ast_scalar, SourceMapId(1),
    );
    assert!(builder.errors.is_empty());
    assert_eq!(builder.name.as_str(), "DateTime");
}

// Verifies from_ast() for an input object type.
// https://spec.graphql.org/September2025/#sec-Input-Objects
// Written by Claude Code, reviewed by a human.
#[test]
fn input_object_type_from_ast() {
    let doc = parse_schema_static(
        "input CreateUserInput { name: String!, age: Int = 18 }",
    );
    let td = extract_type_def(&doc, 0);
    let ast_input = match td {
        libgraphql_parser::ast::TypeDefinition::InputObject(i) => i,
        _ => panic!("expected input object type definition"),
    };
    let builder = InputObjectTypeBuilder::from_ast(
        ast_input, SourceMapId(1),
    );
    assert!(builder.errors.is_empty());
    assert_eq!(builder.name.as_str(), "CreateUserInput");
    assert_eq!(builder.fields.len(), 2);
    assert_eq!(builder.fields[0].name.as_str(), "name");
    assert!(builder.fields[1].default_value.is_some());
}

// Verifies from_ast() for a directive definition.
// https://spec.graphql.org/September2025/#sec-Type-System.Directives
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_from_ast() {
    let doc = parse_schema_static(
        "directive @auth(role: String!) repeatable on FIELD_DEFINITION | OBJECT",
    );
    let ast_dir = match &doc.definitions[0] {
        libgraphql_parser::ast::Definition::DirectiveDefinition(d) => d,
        _ => panic!("expected directive definition"),
    };
    let builder = DirectiveBuilder::from_ast(
        ast_dir, SourceMapId(1),
    );
    assert!(builder.errors.is_empty());
    assert_eq!(builder.name.as_str(), "auth");
    assert!(builder.is_repeatable);
    assert_eq!(builder.locations.len(), 2);
    assert_eq!(builder.parameters.len(), 1);
    assert_eq!(builder.parameters[0].name.as_str(), "role");
}

// Verifies from_ast() with field parameters including
// default values.
// Written by Claude Code, reviewed by a human.
#[test]
fn field_with_parameters_from_ast() {
    let doc = parse_schema_static(
        "type Query { users(first: Int = 10, after: String): [User!]! }",
    );
    let td = extract_type_def(&doc, 0);
    let ast_obj = match td {
        libgraphql_parser::ast::TypeDefinition::Object(obj) => obj,
        _ => panic!("expected object type definition"),
    };
    let builder = ObjectTypeBuilder::from_ast(
        ast_obj, SourceMapId(1),
    );
    assert!(builder.errors.is_empty());
    assert_eq!(builder.fields.len(), 1);
    let field = &builder.fields[0];
    assert_eq!(field.name.as_str(), "users");
    assert_eq!(field.parameters.len(), 2);
    assert_eq!(field.parameters[0].name.as_str(), "first");
    assert!(field.parameters[0].default_value.is_some());
    assert_eq!(field.parameters[1].name.as_str(), "after");
    assert!(field.parameters[1].default_value.is_none());
}

// Verifies from_ast() maps the description string from a type
// definition.
// https://spec.graphql.org/September2025/#sec-Descriptions
// Written by Claude Code, reviewed by a human.
#[test]
fn description_mapped_from_ast() {
    let doc = parse_schema_static(
        "\"A user\" type User { id: ID! }",
    );
    let td = extract_type_def(&doc, 0);
    let ast_obj = match td {
        libgraphql_parser::ast::TypeDefinition::Object(obj) => obj,
        _ => panic!("expected object type definition"),
    };
    let builder = ObjectTypeBuilder::from_ast(
        ast_obj, SourceMapId(1),
    );
    assert!(builder.errors.is_empty());
    assert_eq!(
        builder.description,
        Some("A user".to_string()),
    );
}

// Verifies from_ast() collects duplicate field name errors
// instead of panicking.
// https://spec.graphql.org/September2025/#sec-Objects.Type-Validation
// Written by Claude Code, reviewed by a human.
#[test]
fn from_ast_collects_duplicate_field_errors() {
    let doc = parse_schema_static(
        "type Bad { x: Int, x: String }",
    );
    let td = extract_type_def(&doc, 0);
    let ast_obj = match td {
        libgraphql_parser::ast::TypeDefinition::Object(obj) => obj,
        _ => panic!("expected object type definition"),
    };
    let builder = ObjectTypeBuilder::from_ast(
        ast_obj, SourceMapId(1),
    );
    assert!(!builder.errors.is_empty());
    assert!(matches!(
        builder.errors[0].kind(),
        SchemaBuildErrorKind::DuplicateFieldNameDefinition { .. },
    ));
}
