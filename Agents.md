# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

**Build:**
```bash
cargo build --verbose
```

**Run Tests:**
```bash
cargo test --verbose
```

**Lint (Clippy):**
```bash
cargo clippy --tests -- -Dwarnings
```

**Test Coverage Report:**
```bash
./scripts/generate-test-coverage-report.sh
```

## Code Architecture

This is a Rust library for building GraphQL tools, clients, and servers. The codebase is organized into several main modules:

### Core Modules

- **`ast`** - GraphQL syntax tree definitions, mostly re-exports from `graphql_parser` crate
- **`schema`** - GraphQL schema representation and building (`Schema`, `SchemaBuilder`)
- **`operation`** - GraphQL operations (queries, mutations, subscriptions) with builders
- **`types`** - GraphQL type system (scalars, objects, interfaces, unions, enums, input types)
- **`loc`** - File position and location tracking for definitions
- **`value`** - GraphQL value representation

### Key Design Patterns

- **Builder Pattern**: Most types use dedicated builder classes (e.g., `SchemaBuilder`, `QueryBuilder`, `ObjectTypeBuilder`)
- **Type Safety**: Heavy use of type annotations and validation to ensure GraphQL spec compliance
- **Error Handling**: Custom error types for build failures and validation errors
- **Modular Structure**: Each type category is organized in its own module with clear exports

### Type System Architecture

The `types` module contains the complete GraphQL type system:
- Object, Interface, Union, Enum, Scalar, and Input Object types
- Type annotations for nullable/non-null and list types
- Field definitions with parameters and directives
- Validation logic for type compatibility

### Operation Architecture

The `operation` module handles GraphQL operations:
- Query, Mutation, and Subscription builders
- Selection sets and field selections
- Fragment handling (named and inline)
- Variable definitions and references

## Dependencies

- `graphql-parser` - GraphQL parsing (re-exported types)
- `thiserror` - Error handling macros
- `lazy_static` - Static initialization
- `inherent` - Implementation inheritance patterns