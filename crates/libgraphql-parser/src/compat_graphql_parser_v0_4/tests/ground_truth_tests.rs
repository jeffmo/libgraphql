use std::fmt::Debug;

use crate::compat_graphql_parser_v0_4::to_graphql_parser_query_ast;
use crate::compat_graphql_parser_v0_4::to_graphql_parser_schema_ast;
use crate::GraphQLParser;

/// Compares two `Debug`-formattable values and, on
/// mismatch, prints a readable line-by-line diff of
/// their pretty-printed representations.
fn assert_ast_eq<T: Debug + PartialEq>(
    actual: &T,
    expected: &T,
    source: &str,
    kind: &str,
) {
    if actual == expected {
        return;
    }

    let actual_lines: Vec<&str> =
        format!("{actual:#?}").leak().lines().collect();
    let expected_lines: Vec<&str> =
        format!("{expected:#?}").leak().lines().collect();

    let mut diff = String::new();
    let max_lines = actual_lines.len().max(expected_lines.len());
    for i in 0..max_lines {
        let a = actual_lines.get(i).copied().unwrap_or("");
        let e = expected_lines.get(i).copied().unwrap_or("");
        if a != e {
            diff.push_str(&format!(
                "  line {i}:\n\
                 \x20   ours:     {a}\n\
                 \x20   expected: {e}\n",
            ));
        }
    }

    panic!(
        "\n\nGround-truth {kind} mismatch.\n\n\
         Source:\n{source}\n\n\
         Differing lines:\n{diff}",
    );
}

/// Parses `source` with both `graphql_parser` and our
/// parser (via the compat layer), then asserts the two
/// schema-document ASTs are structurally identical.
///
/// This validates the entire pipeline: lexer, parser,
/// AST construction, and compat-layer conversion.
fn assert_schema_ground_truth(source: &str) {
    let expected =
        graphql_parser::schema::parse_schema::<String>(source)
            .unwrap_or_else(|e| {
                panic!(
                    "graphql_parser failed to parse \
                     schema:\n{e}\n\nSource:\n{source}",
                )
            })
            .into_static();

    let our_ast = GraphQLParser::new(source)
        .parse_schema_document();
    assert!(
        !our_ast.has_errors(),
        "Our parser reported errors:\n{}\n\nSource:\n{source}",
        our_ast.format_errors(Some(source)),
    );
    let our_doc = our_ast.into_valid_ast().expect(
        "valid_ast should be Some when no errors",
    );
    let actual = to_graphql_parser_schema_ast(&our_doc)
        .into_ast();

    assert_ast_eq(&actual, &expected, source, "schema");
}

/// Parses `source` with both `graphql_parser` and our
/// parser (via the compat layer), then asserts the two
/// executable-document ASTs are structurally identical.
fn assert_query_ground_truth(source: &str) {
    let expected =
        graphql_parser::query::parse_query::<String>(source)
            .unwrap_or_else(|e| {
                panic!(
                    "graphql_parser failed to parse \
                     query:\n{e}\n\nSource:\n{source}",
                )
            })
            .into_static();

    let our_ast = GraphQLParser::new(source)
        .parse_executable_document();
    assert!(
        !our_ast.has_errors(),
        "Our parser reported errors:\n{}\n\nSource:\n{source}",
        our_ast.format_errors(Some(source)),
    );
    let our_doc = our_ast.into_valid_ast().expect(
        "valid_ast should be Some when no errors",
    );
    let actual = to_graphql_parser_query_ast(&our_doc)
        .into_ast();

    assert_ast_eq(&actual, &expected, source, "query");
}

// ─────────────────────────────────────────────
// Schema ground-truth tests
// ─────────────────────────────────────────────

/// Simple object type with fields and no descriptions.
///
/// Validates basic type/field parsing and position
/// tracking against `graphql_parser`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_ground_truth_simple_object() {
    assert_schema_ground_truth(
        "\
type User {
  id: ID!
  name: String
  age: Int
}
",
    );
}

/// Object type with described fields — exercises the
/// Part 1 position fix where `graphql_parser` captures
/// position before the description on `Field` sub-nodes.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_ground_truth_described_fields() {
    assert_schema_ground_truth(
        "\
type User {
  \"The unique identifier\"
  id: ID!
  \"The user's display name\"
  name: String
}
",
    );
}

/// Scalar type with a directive.
///
/// Validates scalar definition + directive argument
/// conversion.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_ground_truth_scalar_with_directive() {
    assert_schema_ground_truth(
        "\
scalar DateTime @specifiedBy(url: \"https://scalars.graphql.org/andimarek/date-time\")
",
    );
}

/// Enum type with described values — exercises the
/// Part 1 position fix for `EnumValue` sub-nodes.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_ground_truth_enum_with_descriptions() {
    assert_schema_ground_truth(
        "\
enum Status {
  \"Currently active\"
  ACTIVE
  \"No longer active\"
  INACTIVE
  PENDING
}
",
    );
}

/// Union type definition.
///
/// Validates union member parsing against
/// `graphql_parser`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_ground_truth_union() {
    assert_schema_ground_truth(
        "\
union SearchResult = User | Post | Comment
",
    );
}

/// Input object type with described arguments —
/// exercises the Part 1 position fix for `InputValue`
/// sub-nodes.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_ground_truth_input_with_descriptions() {
    assert_schema_ground_truth(
        "\
input CreateUserInput {
  \"The user's name\"
  name: String!
  \"Optional email\"
  email: String
  age: Int = 0
}
",
    );
}

/// Interface type that implements another interface.
///
/// Validates interface parsing including the
/// `implements` clause.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_ground_truth_interface() {
    assert_schema_ground_truth(
        "\
interface Node {
  id: ID!
}

interface NamedNode implements Node {
  id: ID!
  name: String
}
",
    );
}

/// Directive definition with arguments, locations, and
/// the `repeatable` keyword.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_ground_truth_directive_definition() {
    assert_schema_ground_truth(
        "\
directive @cacheControl(maxAge: Int, scope: CacheControlScope) repeatable on FIELD_DEFINITION | OBJECT | INTERFACE
",
    );
}

/// Schema definition with root operation types.
///
/// Validates `schema { query: ... mutation: ... }`
/// syntax.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_ground_truth_schema_definition() {
    assert_schema_ground_truth(
        "\
schema {
  query: Query
  mutation: Mutation
  subscription: Subscription
}
",
    );
}

/// Type extensions for object, enum, union, interface,
/// scalar, and input object.
///
/// Validates all six type-extension forms.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_ground_truth_type_extensions() {
    assert_schema_ground_truth(
        "\
extend type User {
  email: String
}

extend enum Status {
  ARCHIVED
}

extend union SearchResult = Comment

extend interface Node {
  createdAt: DateTime
}

extend scalar DateTime @specifiedBy(url: \"https://scalars.graphql.org/andimarek/date-time\")

extend input CreateUserInput {
  role: String
}
",
    );
}

/// Complex multi-definition document containing
/// several type kinds, descriptions, directives, and
/// arguments.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_ground_truth_complex_document() {
    assert_schema_ground_truth(
        "\
\"The root query type\"
type Query {
  \"Fetch a user by ID\"
  user(id: ID!): User
  users(first: Int = 10, after: String): [User!]!
}

type User implements Node {
  id: ID!
  name: String!
  email: String
  role: Role!
}

interface Node {
  id: ID!
}

enum Role {
  ADMIN
  USER
  GUEST
}

union SearchResult = User | Post

input CreateUserInput {
  name: String!
  email: String
  role: Role = USER
}

scalar DateTime

directive @deprecated(reason: String = \"No longer supported\") on FIELD_DEFINITION | ENUM_VALUE
",
    );
}

// ─────────────────────────────────────────────
// Query ground-truth tests
// ─────────────────────────────────────────────

/// Simple named query with nested fields.
///
/// Validates basic operation + selection set parsing.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_ground_truth_simple_query() {
    assert_query_ground_truth(
        "\
query GetUser {
  user {
    id
    name
  }
}
",
    );
}

/// Shorthand query (`{ field }`) — verifies the
/// `SelectionSet` operation variant.
///
/// Per the GraphQL spec, a shorthand query is an
/// anonymous query operation written without the
/// `query` keyword.
/// https://spec.graphql.org/September2025/#sec-Language.Operations
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_ground_truth_shorthand() {
    assert_query_ground_truth(
        "\
{
  viewer {
    name
  }
}
",
    );
}

/// Mutation operation.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_ground_truth_mutation() {
    assert_query_ground_truth(
        "\
mutation CreateUser {
  createUser(name: \"Alice\") {
    id
    name
  }
}
",
    );
}

/// Subscription operation.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_ground_truth_subscription() {
    assert_query_ground_truth(
        "\
subscription OnMessageAdded {
  messageAdded {
    id
    content
    author {
      name
    }
  }
}
",
    );
}

/// Query with variable definitions and default values.
///
/// Validates variable syntax including type annotations,
/// nullability, and defaults.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_ground_truth_variables() {
    assert_query_ground_truth(
        "\
query GetUsers($first: Int = 10, $after: String, $includeEmail: Boolean!) {
  users(first: $first, after: $after) {
    id
    name
  }
}
",
    );
}

/// Fragment definition and fragment spreads.
///
/// Validates fragment definition syntax and the
/// `...FragmentName` spread syntax.
/// https://spec.graphql.org/September2025/#sec-Language.Fragments
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_ground_truth_fragments() {
    assert_query_ground_truth(
        "\
query GetUser {
  user(id: 1) {
    ...UserFields
  }
}

fragment UserFields on User {
  id
  name
  email
}
",
    );
}

/// Inline fragments with type conditions.
///
/// Validates `... on Type { }` syntax.
/// https://spec.graphql.org/September2025/#sec-Inline-Fragments
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_ground_truth_inline_fragments() {
    assert_query_ground_truth(
        "\
query Search {
  search(query: \"test\") {
    ... on User {
      name
      email
    }
    ... on Post {
      title
      body
    }
  }
}
",
    );
}

/// Aliases and arguments with various value types:
/// int, float, string, boolean, null, enum, list,
/// and object.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_ground_truth_aliases_and_value_types() {
    assert_query_ground_truth(
        "\
query ValueTypes {
  intVal: field(arg: 42)
  floatVal: field(arg: 3.14)
  stringVal: field(arg: \"hello\")
  boolTrue: field(arg: true)
  boolFalse: field(arg: false)
  nullVal: field(arg: null)
  enumVal: field(arg: ACTIVE)
  listVal: field(arg: [1, 2, 3])
  objectVal: field(arg: {key: \"value\", nested: {a: 1}})
}
",
    );
}

/// Nested selection sets (3+ levels deep).
///
/// Validates deeply nested field selections.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_ground_truth_deep_nesting() {
    assert_query_ground_truth(
        "\
query DeepQuery {
  viewer {
    organization {
      teams {
        members {
          name
        }
      }
    }
  }
}
",
    );
}

/// Fields with varied type annotations: nullable list,
/// non-null list of non-null, and nested list types.
///
/// Validates that `[String]`, `[String!]!`, `[[Int]]`,
/// and `[[Int!]!]!` all produce identical AST structures
/// in both parsers.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_ground_truth_type_annotation_variety() {
    assert_schema_ground_truth(
        "\
type TypeAnnotations {
  nullableList: [String]
  nonNullList: [String!]!
  nestedList: [[Int]]
  complexNested: [[Int!]!]!
}
",
    );
}

/// Block string descriptions on types and fields.
///
/// Validates that triple-quoted block strings are parsed
/// and normalised identically by both parsers (whitespace
/// stripping, indentation removal per GraphQL spec
/// §2.9.4).
/// https://spec.graphql.org/September2025/#sec-String-Value
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_schema_ground_truth_block_string_descriptions() {
    assert_schema_ground_truth(
        r#"
"""
A user in the system.

Represents an authenticated account with profile data.
"""
type User {
  """The unique identifier."""
  id: ID!
  """
  The user's display name.
  May contain unicode characters.
  """
  name: String
}
"#,
    );
}

/// Query with variable values passed as arguments,
/// negative integer, and negative float.
///
/// Validates that `$varName` references in argument
/// positions are correctly represented, and that
/// negative numeric literals round-trip through both
/// parsers identically.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_ground_truth_variable_refs_and_negative_numbers() {
    assert_query_ground_truth(
        "\
query Fetch($id: ID!, $limit: Int) {
  user(id: $id) {
    posts(limit: $limit, offset: -5) {
      score(threshold: -1.5)
    }
  }
}
",
    );
}

/// Directives on operations, fields, and fragments.
///
/// Validates directive parsing and positioning across
/// different directive locations.
/// https://spec.graphql.org/September2025/#sec-Language.Directives
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn test_query_ground_truth_directives() {
    assert_query_ground_truth(
        "\
query GetUser($withEmail: Boolean!) @cacheControl(maxAge: 60) {
  user(id: 1) {
    id
    name
    email @include(if: $withEmail)
    ...UserExtra @skip(if: true)
  }
}

fragment UserExtra on User {
  age
  role
}
",
    );
}
