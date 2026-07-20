use crate::schema::Schema;
use crate::schema::SchemaBuilder;
use crate::span::SourceMapId;
use crate::span::Span;
use libgraphql_parser::ByteSpan;

/// A multi-kind schema exercising every typed lookup category:
/// objects (including all three root operation types),
/// interfaces (including an interface implementing another
/// interface), an enum, a union, an input object, and a custom
/// scalar.
const KITCHEN_SINK_SCHEMA: &str = "\
schema {
  query: TheQuery
  mutation: TheMutation
  subscription: TheSubscription
}

type TheQuery { user: User }
type TheMutation { addUser(name: String!): User }
type TheSubscription { userAdded: User }

interface Node { id: ID! }
interface Timestamped implements Node { id: ID! createdAt: String }

type User implements Node { id: ID! name: String }
type Post implements Node & Timestamped {
  id: ID!
  createdAt: String
  title: String
}

enum Color { RED GREEN BLUE }
union SearchResult = User | Post
input UserFilter { nameContains: String }
scalar DateTime
";

fn build_kitchen_sink_schema() -> Schema {
    SchemaBuilder::build_from_str(KITCHEN_SINK_SCHEMA)
        .expect("kitchen-sink schema should build")
}

// Verifies that each typed lookup (`object_type`,
// `interface_type`, `enum_type`, `union_type`,
// `input_object_type`, `scalar_type`) returns the named type
// when the name refers to a type of the matching kind --
// including built-in scalars for `scalar_type`.
//
// See https://spec.graphql.org/September2025/#sec-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn typed_lookups_return_kind_matched_types() {
    let schema = build_kitchen_sink_schema();

    let user = schema.object_type("User")
        .expect("object_type(User) should be Some");
    assert_eq!(user.name().as_str(), "User");

    let node = schema.interface_type("Node")
        .expect("interface_type(Node) should be Some");
    assert_eq!(node.name().as_str(), "Node");

    let color = schema.enum_type("Color")
        .expect("enum_type(Color) should be Some");
    assert_eq!(color.name().as_str(), "Color");

    let search_result = schema.union_type("SearchResult")
        .expect("union_type(SearchResult) should be Some");
    assert_eq!(search_result.name().as_str(), "SearchResult");

    let user_filter = schema.input_object_type("UserFilter")
        .expect("input_object_type(UserFilter) should be Some");
    assert_eq!(user_filter.name().as_str(), "UserFilter");

    let date_time = schema.scalar_type("DateTime")
        .expect("scalar_type(DateTime) should be Some");
    assert_eq!(date_time.name().as_str(), "DateTime");

    // Built-in scalars are reachable through the typed lookup
    // too.
    let string_scalar = schema.scalar_type("String")
        .expect("scalar_type(String) should be Some");
    assert!(string_scalar.is_builtin());
}

// Verifies that the typed lookups return `None` both for names
// that are not defined at all and for names that are defined
// but refer to a type of a different kind (e.g. asking for
// `object_type("Node")` when `Node` is an interface).
//
// See https://spec.graphql.org/September2025/#sec-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn typed_lookups_return_none_for_missing_or_mismatched() {
    let schema = build_kitchen_sink_schema();

    // Undefined name -> None from every lookup.
    assert!(schema.get_type("Missing").is_none());
    assert!(schema.object_type("Missing").is_none());
    assert!(schema.interface_type("Missing").is_none());
    assert!(schema.enum_type("Missing").is_none());
    assert!(schema.union_type("Missing").is_none());
    assert!(schema.input_object_type("Missing").is_none());
    assert!(schema.scalar_type("Missing").is_none());

    // Defined name, wrong kind -> None.
    assert!(schema.object_type("Node").is_none());
    assert!(schema.interface_type("User").is_none());
    assert!(schema.enum_type("SearchResult").is_none());
    assert!(schema.union_type("Color").is_none());
    assert!(schema.input_object_type("DateTime").is_none());
    assert!(schema.scalar_type("UserFilter").is_none());
}

// Verifies that the typed iterators (`object_types`,
// `interface_types`, `enum_types`) yield exactly the types of
// each respective kind that are defined in the schema.
//
// See https://spec.graphql.org/September2025/#sec-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn typed_iterators_yield_all_types_of_each_kind() {
    let schema = build_kitchen_sink_schema();

    let mut object_names: Vec<&str> = schema.object_types()
        .map(|t| t.name().as_str())
        .collect();
    object_names.sort_unstable();
    assert_eq!(
        object_names,
        vec!["Post", "TheMutation", "TheQuery", "TheSubscription", "User"],
    );

    let mut interface_names: Vec<&str> = schema.interface_types()
        .map(|t| t.name().as_str())
        .collect();
    interface_names.sort_unstable();
    assert_eq!(interface_names, vec!["Node", "Timestamped"]);

    let enum_names: Vec<&str> = schema.enum_types()
        .map(|t| t.name().as_str())
        .collect();
    assert_eq!(enum_names, vec!["Color"]);
}

// Verifies that `types_implementing()` returns every type --
// both object types AND interface types -- that declares it
// implements the given interface, and nothing else.
//
// See https://spec.graphql.org/September2025/#IsValidImplementation()
//
// Written by Claude Code, reviewed by a human.
#[test]
fn types_implementing_returns_objects_and_interfaces() {
    let schema = build_kitchen_sink_schema();

    let mut node_impls: Vec<&str> = schema.types_implementing("Node")
        .iter()
        .map(|t| t.name().as_str())
        .collect();
    node_impls.sort_unstable();
    assert_eq!(node_impls, vec!["Post", "Timestamped", "User"]);

    let timestamped_impls: Vec<&str> =
        schema.types_implementing("Timestamped")
            .iter()
            .map(|t| t.name().as_str())
            .collect();
    assert_eq!(timestamped_impls, vec!["Post"]);
}

// Verifies that `types_implementing()` returns an empty list
// for an interface name that no type implements (including a
// name that is not defined in the schema at all).
//
// See https://spec.graphql.org/September2025/#IsValidImplementation()
//
// Written by Claude Code, reviewed by a human.
#[test]
fn types_implementing_unknown_interface_returns_empty() {
    let schema = build_kitchen_sink_schema();
    assert!(schema.types_implementing("Missing").is_empty());
    // `User` is a defined type but not an interface anything
    // implements.
    assert!(schema.types_implementing("User").is_empty());
}

// Verifies the root operation accessors (`query_type`,
// `mutation_type`, `subscription_type` and their `*_type_name`
// variants) when all three root operation types are explicitly
// bound via a `schema { ... }` definition.
//
// See https://spec.graphql.org/September2025/#sec-Root-Operation-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn root_operation_type_accessors() {
    let schema = build_kitchen_sink_schema();

    assert_eq!(schema.query_type_name().as_str(), "TheQuery");
    let query_type = schema.query_type()
        .expect("query_type() should be Some");
    assert_eq!(query_type.name().as_str(), "TheQuery");

    assert_eq!(
        schema.mutation_type_name().map(|n| n.as_str()),
        Some("TheMutation"),
    );
    let mutation_type = schema.mutation_type()
        .expect("mutation_type() should be Some");
    assert_eq!(mutation_type.name().as_str(), "TheMutation");

    assert_eq!(
        schema.subscription_type_name().map(|n| n.as_str()),
        Some("TheSubscription"),
    );
    let subscription_type = schema.subscription_type()
        .expect("subscription_type() should be Some");
    assert_eq!(subscription_type.name().as_str(), "TheSubscription");
}

// Verifies root operation accessor behavior for a schema with
// no explicit `schema { ... }` definition: the query root
// defaults to the type named `Query`, while the mutation and
// subscription accessors (and their `*_type_name` variants)
// return `None`.
//
// See https://spec.graphql.org/September2025/#sec-Root-Operation-Types
//
// Written by Claude Code, reviewed by a human.
#[test]
fn root_operation_defaults_when_not_declared() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { hello: String }",
    ).expect("schema should build");

    assert_eq!(schema.query_type_name().as_str(), "Query");
    assert!(schema.query_type().is_some());
    assert!(schema.mutation_type_name().is_none());
    assert!(schema.mutation_type().is_none());
    assert!(schema.subscription_type_name().is_none());
    assert!(schema.subscription_type().is_none());
}

// Verifies that `Schema::resolve_span()` resolves an in-range
// span to the correct 0-based line and column on a multi-line
// schema. The span is constructed manually at a hand-computed
// byte offset so the expected `LineCol` is exact:
//
// ```text
// type Query {\n      <- line 0, bytes 0..=12
//   hello: String\n   <- line 1 starts at byte 13
// }\n                 <- `hello` starts at byte 15 (col 2)
// ```
//
// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_span_in_range_returns_correct_line_col() {
    let source = "type Query {\n  hello: String\n}\n";
    let schema = SchemaBuilder::build_from_str(source)
        .expect("schema should build");

    // User sources start at source-map id 1 (id 0 is reserved
    // for built-in definitions).
    let span = Span::new(ByteSpan::new(15, 20), SourceMapId(1));
    let line_col = schema.resolve_span(span)
        .expect("in-range span should resolve");
    assert_eq!(line_col.line, 1);
    assert_eq!(line_col.col_linestart_byte_offset, 2);
    assert_eq!(line_col.col_utf8, 2);
}

// Verifies that resolving the spans stored on schema types
// yields the line each type definition appears on in the
// original multi-line source text (expected lines are located
// independently via `str::lines()`).
//
// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_span_on_type_spans_matches_definition_lines() {
    let schema = build_kitchen_sink_schema();

    for (type_name, line_marker) in [
        ("User", "type User implements Node"),
        ("Node", "interface Node"),
        ("Color", "enum Color"),
        ("DateTime", "scalar DateTime"),
    ] {
        let expected_line = KITCHEN_SINK_SCHEMA.lines()
            .position(|line| line.starts_with(line_marker))
            .unwrap_or_else(|| {
                panic!("marker `{line_marker}` not found in source")
            }) as u32;
        let graphql_type = schema.get_type(type_name)
            .unwrap_or_else(|| panic!("type `{type_name}` not found"));
        let line_col = schema.resolve_span(graphql_type.span())
            .expect("schema-originated span should resolve");
        assert_eq!(
            line_col.line, expected_line,
            "span of `{type_name}` should resolve to its definition line",
        );
    }
}

// Verifies that `Schema::resolve_span()` returns `None` for a
// span whose `SourceMapId` is out of range for this schema
// (e.g. a span that originated from a different artifact with
// more source maps).
//
// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_span_out_of_range_source_map_id_returns_none() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { hello: String }",
    ).expect("schema should build");

    // Only ids 0 (builtin) and 1 (the loaded source) exist.
    assert_eq!(schema.source_maps().len(), 2);
    let span = Span::new(ByteSpan::new(0, 1), SourceMapId(2));
    assert!(schema.resolve_span(span).is_none());
    let far_span = Span::new(ByteSpan::new(0, 1), SourceMapId(999));
    assert!(schema.resolve_span(far_span).is_none());
}

// Verifies built-in span behavior: spans on built-in
// definitions (which carry the reserved source-map id 0 and no
// user-authored source) resolve against the synthetic built-in
// source map to line 0, column 0 -- they never resolve to user
// source text.
//
// Written by Claude Code, reviewed by a human.
#[test]
fn resolve_span_builtin_resolves_to_line_zero() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { hello: String }",
    ).expect("schema should build");

    let builtin_line_col = schema.resolve_span(Span::builtin())
        .expect("builtin span should resolve");
    assert_eq!(builtin_line_col.line, 0);
    assert_eq!(builtin_line_col.col_linestart_byte_offset, 0);
    assert_eq!(builtin_line_col.col_utf8, 0);

    // A real built-in definition's span behaves the same way.
    let boolean_scalar = schema.scalar_type("Boolean")
        .expect("built-in Boolean scalar should exist");
    let line_col = schema.resolve_span(boolean_scalar.span())
        .expect("built-in definition span should resolve");
    assert_eq!(line_col.line, 0);
    assert_eq!(line_col.col_linestart_byte_offset, 0);
}
