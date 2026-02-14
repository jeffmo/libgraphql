libgraphql
==========

[Documentation](https://docs.rs/libgraphql) |
[Github](https://github.com/jeffmo/libgraphql) |
[Crate](https://crates.io/crates/libgraphql)

`libgraphql` is a comprehensive "GraphQL Toolkit & Engine" for building tools, clients,
and servers that need to validate, interpret, execute, or otherwise
manipulate GraphQ schemas and operations.

Broadly, `libgraphql` models GraphQL systems in terms of a validated[^1]
[`Schema`](https://docs.rs/libgraphql/latest/libgraphql/schema/struct.Schema.html)
(which defines types and directives) and a collection of zero or more validated[^2]
[`Operation`](https://docs.rs/libgraphql/latest/libgraphql/operation/enum.Operation.html)s
(queries, mutations, & subscriptions).

[^1]: `libgraphql` validates
[`Schema`](https://docs.rs/libgraphql/latest/libgraphql/schema/struct.Schema.html)
objects as they are 
[built](https://docs.rs/libgraphql/latest/libgraphql/schema/struct.SchemaBuilder.html)
by type-checking all type & directive definitions and validating most of the
constraints specified in the 
[latest GraphQL specification](https://spec.graphql.org/September2025/).

[^2]: `libgraphql` validates all
[`Operation`](https://docs.rs/libgraphql/latest/libgraphql/operation/enum.Operation.html)
objects (including
[`Query`](https://docs.rs/libgraphql/latest/libgraphql/operation/struct.Query.html), 
[`Mutation`](https://docs.rs/libgraphql/latest/libgraphql/operation/struct.Mutation.html),
and
[`Subscription`](https://docs.rs/libgraphql/latest/libgraphql/operation/struct.Subscription.html)
objects) as they are
[built](https://docs.rs/libgraphql/latest/libgraphql/operation/struct.OperationBuilder.html)
by type-checking and validating each operation against a pre-validated
[`Schema`](https://docs.rs/libgraphql/latest/libgraphql/schema/struct.Schema.html).

## Quick Start

Add libgraphql to your `Cargo.toml`:

```bash
$ cargo add libgraphql
```

Basic usage:

```rust,no_run
use libgraphql::macros::graphql_schema;
use libgraphql::operation::FragmentRegistry;
use libgraphql::operation::QueryBuilder;
use libgraphql::schema::Schema;
use libgraphql::schema::SchemaBuilder;
use std::path::Path;

// Write a GraphQL schema directly in rust code.
// 
// Note that macro-built GraphQL schemas are typechecked and validated at
// Rust compile-time rather than runtime.
let schema = graphql_schema! {
  type Query {
    me: User
  }

  type User {
    firstName: String,
    lastName: String,
  }
};

// Or load, typecheck, and validate the GraphQL schema from a file on disk at
// runtime:
let schema = 
    SchemaBuilder::build_from_file(
        Path::new("/path/to/schema.graphql")
    ).expect("schema content failed to load from disk or validate");

// Print all GraphQL types defined in the loaded schema:
for (type_name, _graphql_type) in schema.defined_types().iter() {
    println!("Defined type: `{type_name}`");
}

// Find the `User` object type in the `Schema`:
let user_type = 
    schema.defined_types()
        .get("User")
        .expect("no `User` type defined in this schema");

// Build a typechecked and validated GraphQL query from a string at runtime:
let query_str = r##"
query MyFullName {
  me {
    firstName,
    lastName,
  }
}
"##;

let frag_registry = FragmentRegistry::empty();
let query = QueryBuilder::build_from_str(
    &schema, 
    FragmentRegistry::empty(), 
    /* file_path = */ None,
    query_str,
).expect("query did not parse or validate");

// Or load, typecheck, and validate a query from a file on disk at runtime:
let query = 
    QueryBuilder::build_from_file(
        &schema, 
        FragmentRegistry::empty(),
        Path::new("/path/to/query.graphql"),
    ).expect("query operation content failed to load from disk or failed to validate");

// Identify the name and type of each root field selected in the query:
let query_name = query.name().unwrap_or("<<unnamed query>>");
println!("The `{query_name}` query selects the following root fields:");
for field_selection in query.selection_set().selected_fields(frag_registry) {
    let field = field_selection.field();

    let field_name = field.name();
    let field_type_annotation = field.type_annotation().to_graphql_string();
    println!(" * `{field_name}`: `{field_type_annotation:?}`");
}
```

## Development Details

### **Testing**

Run all tests:

```bash
cargo test
```

Run all tests and generate test coverage output:

```bash
./scripts/generate-test-coverage-report.sh
```

## Feature Flags

### `macros` (default)

Enables the `graphql_schema!` and `graphql_schema_from_str!` compile-time
schema validation and definition macros. Disable with `default-features = false`
if Rust macros aren't needed.

### `use-libgraphql-parser` (experimental)

Enables the new `libgraphql-parser` crate which provides a unified GraphQL
parser with better error messages and recovery. This feature is experimental
and currently has no effect on runtime behavior - it exists for testing
the new parser infrastructure during development.

```toml
[dependencies]
libgraphql = { version = "...", features = ["use-libgraphql-parser"] }
```

## Documentation

Generate and view the API documentation:

```bash
cargo doc --open
```

Online documentation is available at: https://docs.rs/libgraphql/latest/libgraphql/

## License

libgraphql is MIT licensed.

libgraphql re-exports some types provided by the [`graphql_parser`](https://github.com/graphql-rust/graphql-parser) crate, which is licensed under the MIT license.
