# libgraphql - Project Conventions

## Project Overview
`libgraphql` is a comprehensive GraphQL Engine library written in Rust for building tools, clients, and servers that need to validate, interpret, execute, or manipulate GraphQL schemas and operations. The project follows the GraphQL specification (September 2025 edition).

## Repository Structure

### Workspace Organization
The project is organized as a Cargo workspace with three crates:

1. **libgraphql** (`/crates/libgraphql/`) - Main public API crate that re-exports core functionality
2. **libgraphql-core** (`/crates/libgraphql-core/`) - Core implementation with schema, operation, and type system logic
3. **libgraphql-macros** (`/crates/libgraphql-macros/`) - Procedural macros for compile-time schema validation

### Core Modules (libgraphql-core)

- **`schema/`** - Schema building and validation (SchemaBuilder, Schema)
- **`operation/`** - GraphQL operations (QueryBuilder, MutationBuilder, SubscriptionBuilder, fragments)
- **`types/`** - GraphQL type system (object types, interfaces, unions, enums, scalars, input objects)
- **`ast.rs`** - Abstract syntax tree wrappers around `graphql-parser` crate
- **`loc.rs`** - Source location tracking for error reporting
- **`directive_annotation*.rs`** - Directive system support
- **`file_reader.rs`** - File I/O utilities
- **`readonly_map.rs`** - Read-only map wrapper for immutable collections

## Coding Conventions

### Rust Style

#### Line Length
- All lines of code should fit within 80 columns unless doing so is impossible or unreasonably less legible
- When 80 columns is not achievable, stay as close to 80 columns as possible

#### Naming Patterns
- **Builders:** `TypeBuilder`, `SchemaBuilder`, `QueryBuilder` - Follow builder pattern
- **References:** `NamedXyzRef` - Named references to items
- **Validators:** `XyzValidator` - Validation logic (typically private)
- **File names:** Match primary struct/enum name (e.g., `schema_builder.rs` contains `SchemaBuilder`)

#### Import Statements
- **IMPORTANT:** Never use compound `use` statements with curly braces
- Always import one symbol per line for clarity and consistency
- **IMPORTANT:** Always sort all `use` statements alphabetically
- Never use `super` in an import statement. Always use a `crate`-rooted module path for importing local modules.
- Example (correct):
  ```rust
  use crate::ast;
  use crate::operation::ExecutableDocumentBuilder;
  use crate::operation::FragmentRegistryBuilder;
  use crate::schema::Schema;
  use crate::schema::SchemaBuilder;
  use std::fs;
  use std::path::Path;
  use std::path::PathBuf;
  ```
- Example (incorrect):
  ```rust
  use crate::operation::{ExecutableDocumentBuilder, FragmentRegistryBuilder};
  use std::path::Path;
  use crate::ast;
  use super::some_module;
  ```

#### Match Expressions
- All cases in a match-expression should end with a comma -- including the last
  case and cases wrapped in `{}`

#### Error Handling
- Use `thiserror::Error` derive macro for custom error types
- Define enum-based error types with detailed variants
- Create `Result<T>` type alias: `type Result<T> = std::result::Result<T, MyError>;`
- Track source location information in errors for better diagnostics
- Use `#[error(...)]` attributes for error messages

Common error types:
- `SchemaBuildError`
- `QueryBuildError`, `MutationBuildError`, `SubscriptionBuildError`
- `FragmentBuildError`
- `TypeValidationError`
- `SelectionSetBuildError`
- `ExecutableDocumentBuildError`

#### Code Style Rules
- When calling a function with a literal boolean argument (`true` or `false`), always prefix the boolean literal with an inline comment that clarifies the name of the parameter in the form of `foo(/* should_skip_lines = */ true)`

#### Module Organization
- Separate files for each major type/builder
- Private implementation modules with public API exports in `mod.rs`
- Use builder pattern for complex objects to ensure type safety
- Place tests in a `tests` submodule adjacent to the code being tested. Use `#[cfg(test)]` annotations in `mod.rs` to conditionally build/link `tests` submodules.

#### Documentation
- Write comprehensive rustdoc comments with usage examples
- Link to the latest GraphQL specification when referencing spec rules (e.g. a version of the spec no older than https://spec.graphql.org/September2025/)
- Include doctests for public API examples
- Document error cases and validation rules

### Shell Script Conventions

- **IMPORTANT:** All functions defined within `scripts/_include.sh` must be ordered alphabetically
- Use snake_case for function names
- Write error messages to stderr with Unicode symbols for clarity
- Return 0 for success, 1 for failure
- Support cross-platform package managers (brew, apt, dnf, pacman, zypper)
- Include platform detection (macOS, Linux, Windows)

## Testing Conventions
- All newly-added or updated unit tests must include a clear, well-structured, English description of what the test aims to verify/validate and, if/when applicable, a link to the relevant portion(s) of the latest version of the GraphQL specification related to what the test is verifying/validating.
- All tests written by Claude should indicate in the aforementioned comment that the test was "Written by Claude Code, reviewed by a human."

### Test Organization
- Place tests in a `tests` submodule adjacent to the code being tested. Use `#[cfg(test)]` annotations in `mod.rs` to conditionally build/link `tests` submodules.
- Organize test modules in `tests/` subdirectory per module
- Use naming convention: `*_tests.rs` for test files
- Use `#[test]` attribute for test functions

### Test Locations
- `/crates/libgraphql-core/src/types/tests/` - Type system tests
- `/crates/libgraphql-core/src/schema/tests/` - Schema validation tests
- `/crates/libgraphql-macros/src/tests/` - Procedural macro tests
- `/crates/libgraphql/src/tests/` - Integration tests

### Running Tests
```bash
cargo test                           # Run all tests
cargo test --package libgraphql-core # Test specific package
cargo clippy --tests                 # Run linter
cargo check --tests                  # Quick compilation check
```

## Development Workflows

### Building and Documentation
```bash
cargo build                          # Build all crates
cargo doc --open                     # Generate and view documentation
cargo build --release                # Release build
```

### Test Coverage
```bash
./scripts/generate-test-coverage-report.sh
```

### Crate Version Management
```bash
./scripts/bump-version.sh            # Bump crate version numbers
```

### Plan Mode & Planning Docs
- Make plans extremely concise. Sacrifice grammar for the sake of concision.
- At the end of each plan, write a list of unresolved questions to answer (if any)

## Key Dependencies

### Core Dependencies
- **graphql-parser** (0.4.0) - GraphQL syntax parsing
- **serde** (1.0.226) - Serialization framework
- **bincode** (2.0.1) - Binary serialization for macro-generated schemas
- **indexmap** (2.10.0) - Ordered hash maps with serde support
- **thiserror** (2.0.9) - Error type derivation

### Procedural Macro Support
- **proc-macro2** (1.0.101) - Procedural macro utilities
- **quote** (1.0.40) - Code generation helpers
- **syn** (2.0.106) - Rust syntax parsing

### Utility Libraries
- **inherent** (1.0.12) - Trait delegation macro

## Architecture Patterns

### Builder Pattern
The codebase extensively uses the builder pattern to ensure type-safe construction:
- Builders validate input at construction time
- Invalid states are prevented at compile time where possible
- Builders return `Result<T, Error>` for runtime validation

### Separation of Concerns
- **Parsing:** Handled by `graphql-parser` crate and custom AST wrappers
- **Validation:** Separate validators enforce GraphQL specification rules
- **Building:** Builders construct validated, immutable structures
- **Macros:** Compile-time parsing and validation in proc macros

### Error Location Tracking
All errors include source location information (`FilePosition`, `SourceLocation`) for precise error reporting and debugging.

## Common Patterns

### Schema Building
```rust
use libgraphql::SchemaBuilder;

let schema = SchemaBuilder::new()
    .read_schema_from_file("schema.graphql")?
    .build()?;
```

### Compile-Time Schema Macros
```rust
use libgraphql::graphql_schema;

let schema = graphql_schema! {
    type Query {
        hello: String
    }
};
```

### Run-Time Query Building
```rust
use libgraphql::QueryBuilder;

let query = QueryBuilder::new(&schema)
    .read_query_from_file("query.graphql")?
    .build()?;
```

## Project Metadata

- **License:** MIT
- **Repository:** https://github.com/jeffmo/libgraphql
- **Documentation:** https://docs.rs/libgraphql
- **Rust Edition:** 2024
- **Latest Version:** 0.0.32 (main crate)

## Features

### Optional Features
- **macros** (default) - Enables compile-time `graphql_schema!` macros
  - Can be disabled with `default-features = false` if only runtime functionality is needed
