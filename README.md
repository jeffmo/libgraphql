# libgraphql

A comprehensive Rust library for building GraphQL tools, clients, and servers
with full schema validation and type-safe operations.

> [!WARNING]
> Note that this library is not yet complete. It lacks various key validations
> and support for some key features of GraphQL.

## Overview

libgraphql provides a toolkit for working with GraphQL schemas and operations
in Rust. It offers:

- **Schema Building & Validation**: Parse and validate GraphQL schema
  definitions from files or strings
- **Type-Safe Operations**: Build queries, mutations, and subscriptions with
  compile-time type checking
- **Schema Introspection**: Programmatically inspect schema types, fields, and
  relationships
- **File Location Tracking**: Maintain precise source location information for
  error reporting

Built on top of the 
[`graphql-parser`](https://github.com/graphql-rust/graphql-parser) crate, 
libgraphql extends the basic parsing capabilities with comprehensive validation,
type-safe operation building, and rich schema introspection APIs.

## Quick Start

Add libgraphql to your `Cargo.toml`:

```bash
$ cargo add libgraphql
```

Basic usage:

```rust
use libgraphql::schema::Schema;
use libgraphql::operation::Query;

// Build a schema from a file
let schema = Schema::builder()
    .load_file(Path::new("/path/to/schema.graphql"))?
    .build()
    .expect("the schema should be fully valid");

// Load a query from a file
let query = Query::builder(&schema)?
    .selection("user(id: \"123\")")? 
    .selection("  name")?
    .selection("  email")?
    .build()?;
```

## Library Structure

libgraphql is organized into several key modules that work together to provide
a complete GraphQL implementation:

### Schema Module (`schema`)

The foundation of libgraphql is the schema system, which validates and
represents a collection of GraphQL type definitions.

#### Building a Schema

The
[`SchemaBuilder`](https://docs.rs/libgraphql/latest/libgraphql/schema/struct.SchemaBuilder.html) 
types provides flexible methods for loading schema definitions:

```rust
use libgraphql::schema::Schema;

// From a single file
let schema = Schema::builder()
    .load_file("schema.graphql")?
    .build()?;

// From multiple files
let schema = Schema::builder()
    .load_files(vec!["types.graphql", "mutations.graphql"])?
    .build()?;

// From string content
let schema_content = r#"
    type Query {
        user(id: ID!): User
    }
    
    type User {
        id: ID!
        name: String!
        email: String!
    }
"#;

let schema = Schema::builder()
    .load_str(None, schema_content)?
    .build()?;
```

#### Schema Validation

[`SchemaBuilder`](https://docs.rs/libgraphql/latest/libgraphql/schema/struct.SchemaBuilder.html)
automatically validates schemas during construction, checking for:

- **Type consistency**: All referenced types must be defined or built-in
- **Type validation**: All type-compatibility scenarios must validate 
  (e.g. Objects & interfaces that implement interfaces, input/output type
  positions, unique enum-values, union member types, etc)
- **Directive validation**: (TODO) Directives defined with a specific 
  [directive location](https://spec.graphql.org/October2021/#DirectiveLocations)
  may only be used according to those location definitions.

#### Schema Introspection

Once built, schemas provide rich introspection capabilities:

```rust
// Access root operation types
let query_type = schema.query_type();
let mutation_type = schema.mutation_type(); // Option<&ObjectType>
let subscription_type = schema.subscription_type(); // Option<&ObjectType>

// Look up types by name
if let Some(GraphQLType::Object(user_type)) = schema.lookup_type("User") {
    println!("User type has {} fields", obj.fields().len());
    for field in obj.fields().values() {
        println!("  {}: {}", field.name(), field.type_annotation());
    }
}

// Look up directive definitions
if let Some(directive_def) = schema.lookup_directive("deprecated") {
    println!("Found directive definition: {:?}", directive_def);
}
```

### Types Module (`types`)

The types module defines all GraphQL type system components with full validation and introspection support.

#### Core Type System

GraphQL types are represented by the `GraphQLType` enum:

```rust
use libgraphql::types::GraphQLType;

match my_graphql_type {
    GraphQLType::Object(my_obj_type) => {
        // Access object type fields, interfaces, etc.
        for field in my_obj_type.fields().values() {
            println!("Field: {}", field.name());
        }
    }
    GraphQLType::Scalar(my_scalar_type) => {
        println!("Scalar: {}", my_scalar_type.name());
    }
    GraphQLType::Enum(my_enum_type) => {
        for value in my_enum_type.values().values() {
            println!("Enum value: {}", value.name());
        }
    }
    // ... other type kinds
}
```

#### Built-in Types

libgraphql includes all GraphQL built-in types:

- **Scalar Types**: `String`, `Int`, `Float`, `Boolean`, `ID`
- **Built-in Directives**: `@skip`, `@include`, `@deprecated`, `@specifiedBy`

#### Type Annotations

Type annotations represent field types, parameter types, and variable types:

```rust
use libgraphql::types::TypeAnnotation;

// Type annotations support:
// - Named types: User, String, ID
// - List types: [String], [User!]
// - Non-null types: String!, [User!]!
// - Nested combinations: [String!]!, [[Int]]

let type_annotation = my_field.type_annotation();
println!("Field type: {}", type_annotation);

// Check if type is nullable
if type_annotation.nullable() {
    println!("This field is nullable");
}
```

### Operations Module (`operation`)

The operations module provides a type-safe construction for GraphQL queries,
mutations, and subscriptions.

#### Building Operations

Operations are built using builder patterns that enforce schema validation:

```rust
use libgraphql::operation::{Query, Mutation, Subscription};

// Build a query
// TODO(!!!)
let query = Query::builder(&schema)?
    .name("GetUserProfile")?
    .variable("userId", "ID!")?
    .selection("user(id: $userId)")?
    .selection("  id")?
    .selection("  name")?
    .selection("  posts {")?
    .selection("    title")?
    .selection("    content")?
    .selection("  }")?
    .build()?;

// Build a mutation
// TODO(!!!)
let mutation = Mutation::builder(&schema)?
    .selection("updateUser(id: \"123\", input: {name: \"John\"})")?
    .selection("  id")?
    .selection("  name")?
    .build()?;

// Build a subscription
// TODO(!!!)
let subscription = Subscription::builder(&schema)?
    .selection("messageAdded {")?
    .selection("  id")?
    .selection("  content")?
    .selection("  user { name }")?
    .selection("}")?
    .build()?;
```

#### Variables and Directives

Operations support variables and directive annotations:

```rust
use libgraphql::DirectiveAnnotation;

// TODO(!!!)
let query = Query::builder(&schema)?
    .name("ConditionalQuery")?
    .variable("includeEmail", "Boolean!")?
    .selection("user(id: \"123\")")?
    .selection("  name")?
    .selection("  email @include(if: $includeEmail)")?
    .add_directive(DirectiveAnnotation::new("cached", vec![]))?
    .build()?;
```

#### Operation Validation

The operation builders validate:

- **Field existence**: All selected fields must exist on their parent types
- **Argument matching**: Field arguments must match schema definitions
- **Variable usage**: Variables must be defined and used correctly
- **Fragment validity**: Inline and named fragments must be valid
- **Directive application**: Directives must be applicable to their locations

### AST Module (`ast`)

The AST module provides direct access to GraphQL abstract syntax trees for advanced use cases:

```rust
use libgraphql::ast;

// Parse operation AST
let operation_ast = ast::operation::parse_query(query_string)?;

// Parse schema AST  
let schema_ast = ast::schema::parse_schema(schema_string)?;

// Work with AST nodes directly
for definition in schema_ast.definitions {
    match definition {
        ast::schema::Definition::TypeDefinition(type_def) => {
            // Process type definition
        }
        ast::schema::Definition::DirectiveDefinition(directive_def) => {
            // Process directive definition  
        }
        _ => {}
    }
}
```

### Location Tracking (`loc`)

libgraphql maintains precise source location information for comprehensive error reporting:

```rust
use libgraphql::loc::{FilePosition, SchemaDefLocation};

// Schema definitions include location information
if let Some(location) = object_type.def_location() {
    println!("Type defined at {}:{}", location.file().display(), location.line());
}

// Validation errors include precise locations
match schema_build_result {
    Err(SchemaBuildError::DuplicateTypeDefinition { 
        type_name, 
        def1, 
        def2 
    }) => {
        println!("Duplicate type '{}' defined at:", type_name);
        println!("  {}", def1);
        println!("  {}", def2);
    }
    _ => {}
}
```

## Advanced Usage

### Custom Schema Validation

Extend schema validation with custom logic:

```rust
let mut schema_builder = Schema::builder();

// Load base schema
schema_builder = schema_builder.load_file("base.graphql")?;

// Add custom validation logic here before building
let schema = schema_builder.build()?;
```

### Working with Fragment Sets

Operations support fragment definitions and references:

```rust
use libgraphql::operation::{NamedFragment, FragmentSet};

// Define fragments
let user_fragment = NamedFragment::builder(&schema)?
    .name("UserFields")?
    .type_condition("User")?
    .selection("id")?
    .selection("name")?
    .selection("email")?
    .build()?;

let fragment_set = FragmentSet::new()
    .add_fragment(user_fragment)?;

// Use fragments in operations
let query = Query::builder(&schema)?
    .selection("user(id: \"123\") { ...UserFields }")?
    .build_with_fragments(&fragment_set)?;
```

### Error Handling

libgraphql provides comprehensive error types with source location information:

```rust
use libgraphql::schema::SchemaBuildError;
use libgraphql::operation::QueryBuildError;

match Schema::builder().load_file("invalid.graphql") {
    Err(SchemaBuildError::ParseError { file, err }) => {
        eprintln!("Parse error in {}: {}", file.display(), err);
    }
    Err(SchemaBuildError::DuplicateTypeDefinition { type_name, def1, def2 }) => {
        eprintln!("Duplicate definition of type '{}':", type_name);
        eprintln!("  First defined at: {}", def1);
        eprintln!("  Also defined at: {}", def2);
    }
    // ... handle other error types
    Ok(builder) => {
        // Continue with valid schema
    }
}
```

## Examples

### Complete Schema Processing Pipeline

```rust
use libgraphql::{schema::Schema, operation::Query, types::GraphQLType};

// 1. Build validated schema
let schema = Schema::builder()
    .load_files(vec!["schema/types.graphql", "schema/operations.graphql"])?
    .build()?;

// 2. Introspect schema structure
println!("Schema has {} types", schema.lookup_type("").is_some());
for (name, graphql_type) in &schema.types {
    match graphql_type {
        GraphQLType::Object(obj) => {
            println!("Object type: {} ({} fields)", name, obj.fields().len());
        }
        GraphQLType::Interface(iface) => {
            println!("Interface: {} ({} fields)", name, iface.fields().len());
        }
        _ => {}
    }
}

// 3. Build type-safe operations
let query = Query::builder(&schema)?
    .name("GetUserWithPosts")?
    .variable("userId", "ID!")?
    .variable("first", "Int")?
    .selection("user(id: $userId)")?
    .selection("  id")?
    .selection("  name")?
    .selection("  posts(first: $first) {")?
    .selection("    edges {")?
    .selection("      node {")?
    .selection("        title")?
    .selection("        publishedAt")?
    .selection("      }")?
    .selection("    }")?
    .selection("  }")?
    .build()?;

println!("Built query: {}", query.name().unwrap_or("unnamed"));
```

## Testing

Run the test suite:

```bash
cargo test
```

Generate test coverage:

```bash
./scripts/generate-test-coverage-report.sh
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
