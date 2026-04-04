# libgraphql-core-v2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a from-scratch rewrite of `libgraphql-core` that consumes `libgraphql-parser` AST directly, exposes public type builders, leverages Rust's type system for safety, and implements complete GraphQL September 2025 spec validation.

**Architecture:** Owned semantic types (no lifetime params) built from parser AST via public builders registered with `SchemaBuilder`. Name newtypes (`TypeName`, `FieldName`, etc.) prevent cross-domain confusion. A shared `HasFieldsAndInterfaces` trait enables generic validation over Object/Interface types. `SchemaBuilder::build()` runs comprehensive cross-type validation and returns `Result<Schema, SchemaErrors>`. Operations are a single `Operation` type with `kind: OperationKind`. All types are serde-serializable for macro crate integration.

**Tech Stack:** Rust 2024 edition, `libgraphql-parser`, `serde`+`bincode`, `indexmap`, `thiserror`

---

## Architectural Decisions (Consolidated from Review)

### AD1. Name Newtypes — Prevent cross-domain string confusion
```rust
// Zero-cost #[repr(transparent)] newtypes for each name domain
pub struct TypeName(String);
pub struct FieldName(String);
pub struct VariableName(String);
pub struct DirectiveName(String);
pub struct EnumValueName(String);
pub struct FragmentName(String);
```
Used throughout: `TypeRef(TypeName)`, `IndexMap<TypeName, GraphQLType>`, `Value::VarRef(VariableName)`, etc.

### AD2. Shared Trait for Object/Interface Types
```rust
pub trait HasFieldsAndInterfaces {
    fn fields(&self) -> &IndexMap<FieldName, Field>;
    fn field(&self, name: &FieldName) -> Option<&Field>;
    fn interface_names(&self) -> &[TypeName];
}
```
Both `ObjectType` and `InterfaceType` implement this. The validator is generic over `T: HasFieldsAndInterfaces`.

### AD3. Unified DirectiveDefinition (not enum)
Replace v1's `Directive` enum with a single struct. Built-ins are regular entries with `builtin: true`. Includes `locations` and `is_repeatable` fields (absent in v1, causing entire directive-usage validation subsystem to be missing).

### AD4. GraphQLType — 6 Variants, Built-in Scalars as ScalarType
`GraphQLType::Scalar(Box<ScalarType>)` where `ScalarType { builtin: bool }`. No more `Bool`/`Float`/`ID`/`Int`/`String` enum variants.

### AD5. Source Locations — Span = ByteSpan + SourceMapId
Compact (12 bytes, Copy). Schema owns `Vec<OwnedSourceMap>`. Resolution on demand.

### AD6. Pre-Resolved FieldSelection
`FieldSelection` stores `parent_type_name: TypeName`, `field_return_type_name: TypeName`, `requires_selection_set: bool` — validated at build time, queryable without `&Schema`.

### AD7. SchemaErrors Newtype
`SchemaBuilder::build()` returns `Result<Schema, SchemaErrors>` where `SchemaErrors` implements `Error + Display + IntoIterator<Item = SchemaBuildError>`, enabling `?` propagation.

### AD8. Typed Schema Query Methods
`schema.object_type(&name) -> Option<&ObjectType>`, `schema.object_types() -> impl Iterator`, `schema.types_implementing(&name) -> impl Iterator`, etc.

### AD9. Registration via SchemaBuilder
`schema_builder.register_type(builder)` (subject-verb-object) rather than `builder.register(&mut sb)`.

### AD10. Load via ParseResult
`schema_builder.load_parse_result(&parse_result)` bundles AST + source map. Convenience `SchemaBuilder::from_str()` wraps parsing.

---

## File Structure

```
crates/libgraphql-core-v2/
  Cargo.toml
  src/
    lib.rs

    // ---- Foundational ----
    names.rs                         -- TypeName, FieldName, VariableName, DirectiveName,
                                        EnumValueName, FragmentName newtypes + macro
    span.rs                          -- Span, SourceMapId, BUILTIN_SOURCE_MAP_ID
    owned_source_map.rs              -- OwnedSourceMap
    value.rs                         -- Value enum (uses VariableName, EnumValueName)
    directive_annotation.rs          -- DirectiveAnnotation (applied directive w/ args)
    readonly_map.rs                  -- ReadOnlyMap (filtered view)
    file_reader.rs                   -- read_content() utility

    // ---- Type system (immutable, validated) ----
    types/
      mod.rs                         -- re-exports + HasFieldsAndInterfaces trait
      type_ref.rs                    -- TypeRef(TypeName), DirectiveRef(DirectiveName)
      graphql_type.rs                -- GraphQLType (6 variants) + is_input/is_output
      graphql_type_kind.rs           -- GraphQLTypeKind
      type_annotation.rs             -- TypeAnnotation + subtype/equivalence logic
      deprecation_state.rs           -- DeprecationState
      object_type.rs                 -- ObjectType (impl HasFieldsAndInterfaces)
      interface_type.rs              -- InterfaceType (impl HasFieldsAndInterfaces)
      union_type.rs                  -- UnionType
      enum_type.rs                   -- EnumType + EnumValue
      scalar_type.rs                 -- ScalarType { builtin: bool }
      input_object_type.rs           -- InputObjectType + InputField
      field.rs                       -- Field (output field) + Parameter
      directive_definition.rs        -- DirectiveDefinition (unified struct)
      directive_location.rs          -- DirectiveLocation enum
      tests/

    // ---- Type builders (public) ----
    type_builders/
      mod.rs
      common.rs                      -- FieldDef, InputFieldDef, ParameterDef,
                                        EnumValueDef, DirectiveAnnotationData
      object_type_builder.rs
      interface_type_builder.rs
      union_type_builder.rs
      enum_type_builder.rs
      scalar_type_builder.rs
      input_object_type_builder.rs
      directive_builder.rs
      ast_conversion.rs              -- from_ast() helpers
      tests/

    // ---- Schema ----
    schema/
      mod.rs
      schema.rs                      -- Schema + typed query API
      schema_builder.rs              -- SchemaBuilder
      schema_errors.rs               -- SchemaErrors newtype + SchemaBuildError enum
      type_validation_error.rs       -- TypeValidationError enum
      _macro_runtime.rs
      tests/

    // ---- Validators (private) ----
    validators/
      mod.rs
      object_or_interface_validator.rs
      union_validator.rs
      input_object_validator.rs
      directive_validator.rs         -- NEW: directive usage + definition validation
      type_ref_validator.rs          -- type annotation input/output + resolution checks

    // ---- Operations ----
    operation/
      mod.rs
      operation.rs                   -- Operation (single type w/ OperationKind)
      operation_kind.rs
      operation_builder.rs
      operation_errors.rs
      variable.rs
      selection_set.rs               -- SelectionSet + all_fields() iterator
      selection_set_builder.rs
      selection.rs                   -- Selection enum
      field_selection.rs             -- FieldSelection (pre-resolved metadata)
      fragment_spread.rs
      inline_fragment.rs
      fragment.rs
      fragment_builder.rs
      fragment_registry.rs
      fragment_registry_builder.rs
      executable_document.rs
      executable_document_builder.rs
      tests/
```

---

## Task Breakdown

### Task 1: Crate Scaffolding

**Files:**
- Create: `crates/libgraphql-core-v2/Cargo.toml`
- Create: `crates/libgraphql-core-v2/src/lib.rs`
- Modify: `Cargo.toml` (workspace members)

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "libgraphql-core-v2"
version = "0.0.1"
edition = "2024"
description = "Core semantic types and validation for GraphQL (v2)"
license = "MIT"

[dependencies]
bincode = { workspace = true }
indexmap = { workspace = true, features = ["serde"] }
libgraphql-parser = { path = "../libgraphql-parser", version = "0.0.5" }
serde = { workspace = true, features = ["derive"] }
thiserror = { workspace = true }

[dev-dependencies]
```

- [ ] **Step 2: Create stub lib.rs**

```rust
//! `libgraphql-core-v2` provides validated, semantic GraphQL constructs
//! for tools, servers, and clients that need to validate, interpret,
//! execute, or manipulate GraphQL schemas and operations.
//!
//! All types produced by this crate are fully validated against the
//! [GraphQL September 2025 specification](https://spec.graphql.org/September2025/).

pub mod names;
pub mod span;
```

- [ ] **Step 3: Add to workspace**

Add `"crates/libgraphql-core-v2"` to the `members` list in the root `Cargo.toml`.

- [ ] **Step 4: Verify compilation**

Run: `cargo check --package libgraphql-core-v2`
Expected: Success (empty crate compiles)

- [ ] **Step 5: Commit**

```
[libgraphql-core-v2] Scaffold new crate
```

---

### Task 2: Name Newtypes

**Files:**
- Create: `crates/libgraphql-core-v2/src/names.rs`
- Test: inline `#[cfg(test)]` module

- [ ] **Step 1: Write tests for name newtypes**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Verifies that name newtypes are distinct types that prevent
    // cross-domain confusion (e.g. TypeName vs FieldName).
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn name_newtypes_are_distinct() {
        let type_name = TypeName::new("User");
        let field_name = FieldName::new("User");
        // These are different types -- cannot be compared or mixed
        assert_eq!(type_name.as_str(), "User");
        assert_eq!(field_name.as_str(), "User");
    }

    // Verifies Display impl formats as the inner string.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn name_display() {
        let name = TypeName::new("Query");
        assert_eq!(format!("{name}"), "Query");
    }

    // Verifies serde round-trip serialization.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn name_serde_roundtrip() {
        let name = TypeName::new("User");
        let serialized = bincode::serde::encode_to_vec(
            &name,
            bincode::config::standard(),
        ).unwrap();
        let (deserialized, _): (TypeName, _) = bincode::serde::decode_from_slice(
            &serialized,
            bincode::config::standard(),
        ).unwrap();
        assert_eq!(name, deserialized);
    }

    // Verifies From<&str> and From<String> conversions.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn name_from_conversions() {
        let from_str: TypeName = "Query".into();
        let from_string: TypeName = String::from("Query").into();
        assert_eq!(from_str, from_string);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package libgraphql-core-v2 -- names`
Expected: FAIL (module doesn't exist yet)

- [ ] **Step 3: Implement name newtypes**

```rust
//! Strongly-typed newtypes for the distinct name domains in GraphQL.
//!
//! GraphQL has several semantically distinct name spaces — type names,
//! field names, variable names, directive names, enum value names, and
//! fragment names. Using bare `String` for all of them allows accidental
//! cross-domain mixing (e.g. passing a field name where a type name is
//! expected). These newtypes prevent that at compile time with zero
//! runtime cost (`#[repr(transparent)]`).

macro_rules! graphql_name_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[derive(serde::Deserialize, serde::Serialize)]
        #[repr(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(s: impl Into<String>) -> Self {
                Self(s.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_owned())
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl std::borrow::Borrow<str> for $name {
            fn borrow(&self) -> &str {
                &self.0
            }
        }
    };
}

graphql_name_newtype!(
    /// A GraphQL type name (e.g. `User`, `String`, `Query`).
    TypeName
);

graphql_name_newtype!(
    /// A GraphQL field name (e.g. `firstName`, `id`).
    FieldName
);

graphql_name_newtype!(
    /// A GraphQL variable name (e.g. `userId` — without the `$` prefix).
    VariableName
);

graphql_name_newtype!(
    /// A GraphQL directive name (e.g. `deprecated`, `skip` — without the
    /// `@` prefix).
    DirectiveName
);

graphql_name_newtype!(
    /// A GraphQL enum value name (e.g. `ACTIVE`, `ADMIN`).
    EnumValueName
);

graphql_name_newtype!(
    /// A GraphQL fragment name (e.g. `UserFields`).
    FragmentName
);

// Export the macro for internal use across the crate
pub(crate) use graphql_name_newtype;

#[cfg(test)]
mod tests {
    // ... (tests from step 1)
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --package libgraphql-core-v2 -- names`
Expected: All pass

- [ ] **Step 5: Commit**

```
[libgraphql-core-v2] Add name newtypes (TypeName, FieldName, etc.)
```

---

### Task 3: Span and Source Map Types

**Files:**
- Create: `crates/libgraphql-core-v2/src/span.rs`
- Create: `crates/libgraphql-core-v2/src/owned_source_map.rs`

- [ ] **Step 1: Write tests for Span**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use libgraphql_parser::ByteSpan;

    // Verifies Span construction and field access.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn span_construction() {
        let span = Span::new(ByteSpan::new(10, 20), SourceMapId(1));
        assert_eq!(span.byte_span.start, 10);
        assert_eq!(span.byte_span.end, 20);
        assert_eq!(span.source_map_id, SourceMapId(1));
    }

    // Verifies BUILTIN span is at source_map_id 0 with empty byte span.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn builtin_span() {
        let span = Span::builtin();
        assert_eq!(span.source_map_id, BUILTIN_SOURCE_MAP_ID);
        assert!(span.byte_span.is_empty());
    }
}
```

- [ ] **Step 2: Implement Span types**

```rust
//! Compact source location types for tracking where schema and operation
//! elements were defined.

use libgraphql_parser::ByteSpan;

/// Identifies a source map within a [`Schema`](crate::schema::Schema)'s
/// collection of source maps. Index 0 is reserved for built-in
/// definitions.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct SourceMapId(pub(crate) u16);

/// The source map ID used for built-in types and directives (Boolean,
/// String, @skip, @include, etc.) that have no user-authored source.
pub const BUILTIN_SOURCE_MAP_ID: SourceMapId = SourceMapId(0);

/// A compact source location: a byte-offset range within a specific
/// source map. 12 bytes, `Copy`.
///
/// Resolving to line/column positions requires the corresponding
/// [`OwnedSourceMap`](crate::owned_source_map::OwnedSourceMap), which
/// is stored on the [`Schema`](crate::schema::Schema).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Span {
    pub byte_span: ByteSpan,
    pub source_map_id: SourceMapId,
}

impl Span {
    pub fn new(byte_span: ByteSpan, source_map_id: SourceMapId) -> Self {
        Self { byte_span, source_map_id }
    }

    /// A zero-width span for built-in definitions.
    pub fn builtin() -> Self {
        Self {
            byte_span: ByteSpan::empty_at(0),
            source_map_id: BUILTIN_SOURCE_MAP_ID,
        }
    }
}
```

- [ ] **Step 3: Implement OwnedSourceMap**

```rust
//! Owned source map data for resolving byte offsets to line/column
//! positions.

use std::path::PathBuf;

/// Serializable source map containing the data needed to resolve
/// [`ByteSpan`](libgraphql_parser::ByteSpan)s to line/column positions.
///
/// One `OwnedSourceMap` exists per source file or string loaded into a
/// [`SchemaBuilder`](crate::schema::SchemaBuilder).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct OwnedSourceMap {
    pub file_path: Option<PathBuf>,
    pub line_starts: Vec<u32>,
}

impl OwnedSourceMap {
    /// Creates an owned source map from a
    /// [`libgraphql_parser::SourceMap`].
    pub fn from_parser_source_map(
        source_map: &libgraphql_parser::SourceMap<'_>,
        file_path: Option<PathBuf>,
    ) -> Self {
        Self {
            file_path,
            line_starts: source_map.line_starts().to_vec(),
        }
    }

    /// Creates a synthetic source map for built-in definitions.
    pub fn builtin() -> Self {
        Self {
            file_path: None,
            line_starts: vec![0],
        }
    }

    /// Resolves a byte offset to a 0-based (line, column) pair.
    pub fn resolve_offset(&self, byte_offset: u32) -> (u32, u32) {
        let line = self.line_starts
            .partition_point(|&start| start <= byte_offset)
            .saturating_sub(1);
        let col = byte_offset - self.line_starts[line];
        (line as u32, col)
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --package libgraphql-core-v2`
Expected: All pass

- [ ] **Step 5: Commit**

```
[libgraphql-core-v2] Add Span, SourceMapId, and OwnedSourceMap types
```

---

### Task 4: Value Enum

**Files:**
- Create: `crates/libgraphql-core-v2/src/value.rs`

- [ ] **Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Verifies Value variants construct correctly with typed names.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn value_variants() {
        let int_val = Value::Int(42);
        let str_val = Value::String("hello".to_string());
        let var_val = Value::VarRef(VariableName::new("userId"));
        let enum_val = Value::Enum(EnumValueName::new("ACTIVE"));
        let null_val = Value::Null;
        let bool_val = Value::Boolean(true);
        let list_val = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let obj_val = Value::Object(IndexMap::from([
            ("key".to_string(), Value::String("val".to_string())),
        ]));

        assert!(matches!(int_val, Value::Int(42)));
        assert!(matches!(var_val, Value::VarRef(_)));
        assert!(matches!(enum_val, Value::Enum(_)));
        assert!(matches!(null_val, Value::Null));
        assert!(matches!(bool_val, Value::Boolean(true)));
        assert!(matches!(list_val, Value::List(_)));
        assert!(matches!(obj_val, Value::Object(_)));
        assert!(matches!(str_val, Value::String(_)));
    }
}
```

- [ ] **Step 2: Implement Value**

```rust
//! GraphQL input values used in arguments, defaults, and variables.

use crate::names::EnumValueName;
use crate::names::VariableName;
use indexmap::IndexMap;

/// A GraphQL input value.
///
/// See [Input Values](https://spec.graphql.org/September2025/#sec-Input-Values).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum Value {
    Boolean(bool),
    Enum(EnumValueName),
    Float(f64),
    Int(i64),
    List(Vec<Value>),
    Null,
    Object(IndexMap<String, Value>),
    String(String),
    VarRef(VariableName),
}
```

- [ ] **Step 3: Run tests, verify pass**

Run: `cargo test --package libgraphql-core-v2 -- value`

- [ ] **Step 4: Commit**

```
[libgraphql-core-v2] Add Value enum with typed name references
```

---

### Task 5: DirectiveAnnotation

**Files:**
- Create: `crates/libgraphql-core-v2/src/directive_annotation.rs`

- [ ] **Step 1: Implement DirectiveAnnotation**

```rust
//! Applied directive annotations (e.g. `@deprecated(reason: "Use X")`).

use crate::names::DirectiveName;
use crate::span::Span;
use crate::value::Value;
use indexmap::IndexMap;

/// An applied directive instance on a type, field, argument, etc.
///
/// This represents a *usage* of a directive, not its *definition*.
/// See [Directives](https://spec.graphql.org/September2025/#sec-Language.Directives).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct DirectiveAnnotation {
    pub(crate) arguments: IndexMap<String, Value>,
    pub(crate) name: DirectiveName,
    pub(crate) span: Span,
}

impl DirectiveAnnotation {
    pub fn arguments(&self) -> &IndexMap<String, Value> {
        &self.arguments
    }

    pub fn name(&self) -> &DirectiveName {
        &self.name
    }

    pub fn span(&self) -> Span {
        self.span
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check --package libgraphql-core-v2`

- [ ] **Step 3: Commit**

```
[libgraphql-core-v2] Add DirectiveAnnotation
```

---

### Task 6: ReadOnlyMap and File Reader

**Files:**
- Create: `crates/libgraphql-core-v2/src/readonly_map.rs`
- Create: `crates/libgraphql-core-v2/src/file_reader.rs`

- [ ] **Step 1: Port ReadOnlyMap from v1**

Port `/crates/libgraphql-core/src/readonly_map.rs` with minor adaptations. Key change: parameterize over `IndexMap` instead of `HashMap` since v2 uses `IndexMap` throughout.

- [ ] **Step 2: Port file_reader from v1**

Port `/crates/libgraphql-core/src/file_reader.rs` as-is.

- [ ] **Step 3: Run compilation check**

Run: `cargo check --package libgraphql-core-v2`

- [ ] **Step 4: Commit**

```
[libgraphql-core-v2] Add ReadOnlyMap and file_reader utilities
```

---

### Task 7: Type References

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/type_ref.rs`

- [ ] **Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Verifies TypeRef stores and exposes a TypeName.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn type_ref_name() {
        let r = TypeRef::new(TypeName::new("User"));
        assert_eq!(r.name().as_str(), "User");
    }
}
```

- [ ] **Step 2: Implement TypeRef and DirectiveRef**

```rust
//! String-based named references to types and directives within a
//! [`Schema`](crate::schema::Schema).

use crate::names::DirectiveName;
use crate::names::TypeName;

/// A named reference to a GraphQL type. Resolves via
/// [`Schema::get_type()`](crate::schema::Schema::get_type).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct TypeRef(TypeName);

impl TypeRef {
    pub fn new(name: TypeName) -> Self {
        Self(name)
    }

    pub fn name(&self) -> &TypeName {
        &self.0
    }
}

/// A named reference to a directive definition.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct DirectiveRef(DirectiveName);

impl DirectiveRef {
    pub fn new(name: DirectiveName) -> Self {
        Self(name)
    }

    pub fn name(&self) -> &DirectiveName {
        &self.0
    }
}
```

- [ ] **Step 3: Create types/mod.rs with re-exports**

```rust
mod type_ref;

pub use type_ref::DirectiveRef;
pub use type_ref::TypeRef;
```

- [ ] **Step 4: Run tests, commit**

```
[libgraphql-core-v2] Add TypeRef and DirectiveRef
```

---

### Task 8: TypeAnnotation + Subtype Logic

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/type_annotation.rs`

- [ ] **Step 1: Write tests for equivalence and subtype checking**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Verifies that identical type annotations are equivalent.
    // Per GraphQL spec Section 3.6.1, parameter types must be
    // structurally identical.
    // https://spec.graphql.org/September2025/#IsValidImplementation()
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn equivalent_named_types() {
        let a = TypeAnnotation::named("String", false);
        let b = TypeAnnotation::named("String", false);
        assert!(a.is_equivalent_to(&b));
    }

    // Verifies nullability difference breaks equivalence.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn non_equivalent_nullability() {
        let nullable = TypeAnnotation::named("String", true);
        let non_null = TypeAnnotation::named("String", false);
        assert!(!nullable.is_equivalent_to(&non_null));
    }

    // Verifies list vs named breaks equivalence.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn non_equivalent_list_vs_named() {
        let named = TypeAnnotation::named("String", false);
        let list = TypeAnnotation::list(
            TypeAnnotation::named("String", false),
            false,
        );
        assert!(!named.is_equivalent_to(&list));
    }

    // Verifies Display formatting matches GraphQL syntax.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn display_formatting() {
        assert_eq!(
            TypeAnnotation::named("String", false).to_string(),
            "String!",
        );
        assert_eq!(
            TypeAnnotation::named("String", true).to_string(),
            "String",
        );
        assert_eq!(
            TypeAnnotation::list(
                TypeAnnotation::named("Int", false),
                true,
            ).to_string(),
            "[Int!]",
        );
    }
}
```

- [ ] **Step 2: Implement TypeAnnotation**

```rust
//! Type annotations representing GraphQL type references with
//! nullability and list wrapping.

use crate::names::TypeName;
use crate::span::Span;

/// A GraphQL type annotation (type reference with nullability).
///
/// See [Type References](https://spec.graphql.org/September2025/#sec-Type-References).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum TypeAnnotation {
    List(ListTypeAnnotation),
    Named(NamedTypeAnnotation),
}

#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct NamedTypeAnnotation {
    pub(crate) nullable: bool,
    pub(crate) span: Span,
    pub(crate) type_name: TypeName,
}

#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ListTypeAnnotation {
    pub(crate) inner: Box<TypeAnnotation>,
    pub(crate) nullable: bool,
    pub(crate) span: Span,
}

impl TypeAnnotation {
    /// Convenience constructor for tests and programmatic building.
    pub fn named(type_name: impl Into<TypeName>, nullable: bool) -> Self {
        Self::Named(NamedTypeAnnotation {
            nullable,
            span: Span::builtin(),
            type_name: type_name.into(),
        })
    }

    /// Convenience constructor for list types.
    pub fn list(inner: TypeAnnotation, nullable: bool) -> Self {
        Self::List(ListTypeAnnotation {
            inner: Box::new(inner),
            nullable,
            span: Span::builtin(),
        })
    }

    pub fn nullable(&self) -> bool {
        match self {
            Self::List(l) => l.nullable,
            Self::Named(n) => n.nullable,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::List(l) => l.span,
            Self::Named(n) => n.span,
        }
    }

    /// Returns the innermost named type, unwrapping all list layers.
    pub fn innermost_named(&self) -> &NamedTypeAnnotation {
        match self {
            Self::List(l) => l.inner.innermost_named(),
            Self::Named(n) => n,
        }
    }

    /// The name of the innermost type.
    pub fn innermost_type_name(&self) -> &TypeName {
        &self.innermost_named().type_name
    }

    /// Structural equivalence (same shape, same nullability, same name).
    /// Used for parameter type validation per
    /// [IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation()).
    pub fn is_equivalent_to(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Named(a), Self::Named(b)) => {
                a.nullable == b.nullable && a.type_name == b.type_name
            },
            (Self::List(a), Self::List(b)) => {
                a.nullable == b.nullable
                    && a.inner.is_equivalent_to(&b.inner)
            },
            _ => false,
        }
    }

    /// Covariant subtype check for field return type validation.
    /// `self` is a subtype of `other` if:
    /// - Same structure and `self` is equal or more restrictive in
    ///   nullability
    /// - Or `self`'s inner named type is a member/implementor of
    ///   `other`'s inner named type (requires types_map for resolution)
    pub fn is_subtype_of(
        &self,
        types_map: &indexmap::IndexMap<TypeName, crate::types::GraphQLType>,
        other: &Self,
    ) -> bool {
        match (self, other) {
            (Self::Named(a), Self::Named(b)) => {
                a.is_subtype_of(types_map, b)
            },
            (Self::List(a), Self::List(b)) => {
                a.is_subtype_of(types_map, b)
            },
            _ => false,
        }
    }
}

impl NamedTypeAnnotation {
    pub fn nullable(&self) -> bool {
        self.nullable
    }

    pub fn type_name(&self) -> &TypeName {
        &self.type_name
    }

    fn is_subtype_of(
        &self,
        types_map: &indexmap::IndexMap<TypeName, crate::types::GraphQLType>,
        other: &Self,
    ) -> bool {
        // Non-null is subtype of nullable (same name)
        if self.type_name == other.type_name {
            return !self.nullable || other.nullable;
        }
        // Different names: check if self implements/is-member-of other
        if !self.nullable || other.nullable {
            // (subtype relationship check requires schema context)
            // TODO: implement abstract type subtyping
        }
        false
    }
}

impl ListTypeAnnotation {
    fn is_subtype_of(
        &self,
        types_map: &indexmap::IndexMap<TypeName, crate::types::GraphQLType>,
        other: &Self,
    ) -> bool {
        (!self.nullable || other.nullable)
            && self.inner.is_subtype_of(types_map, &other.inner)
    }
}

impl std::fmt::Display for TypeAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Named(n) => {
                write!(f, "{}{}", n.type_name, if n.nullable { "" } else { "!" })
            },
            Self::List(l) => {
                write!(f, "[{}]{}", l.inner, if l.nullable { "" } else { "!" })
            },
        }
    }
}
```

- [ ] **Step 3: Run tests, verify pass**
- [ ] **Step 4: Commit**

```
[libgraphql-core-v2] Add TypeAnnotation with subtype/equivalence logic
```

---

### Task 9: Core Type Structs (ScalarType, EnumType, EnumValue, DeprecationState)

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/scalar_type.rs`
- Create: `crates/libgraphql-core-v2/src/types/enum_type.rs`
- Create: `crates/libgraphql-core-v2/src/types/deprecation_state.rs`

- [ ] **Step 1: Implement DeprecationState**

```rust
/// Deprecation status derived from the `@deprecated` directive.
#[derive(Clone, Debug, PartialEq)]
pub enum DeprecationState<'a> {
    Active,
    Deprecated { reason: Option<&'a str> },
}
```

- [ ] **Step 2: Implement ScalarType**

```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::TypeName;
use crate::span::Span;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ScalarType {
    pub(crate) builtin: bool,
    pub(crate) def_location: Span,
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: TypeName,
}

impl ScalarType {
    pub fn builtin(&self) -> bool { self.builtin }
    pub fn def_location(&self) -> Span { self.def_location }
    pub fn description(&self) -> Option<&str> { self.description.as_deref() }
    pub fn directives(&self) -> &[DirectiveAnnotation] { &self.directives }
    pub fn name(&self) -> &TypeName { &self.name }
}
```

- [ ] **Step 3: Implement EnumValue and EnumType**

```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::EnumValueName;
use crate::names::TypeName;
use crate::span::Span;
use indexmap::IndexMap;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EnumValue {
    pub(crate) def_location: Span,
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: EnumValueName,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EnumType {
    pub(crate) def_location: Span,
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: TypeName,
    pub(crate) values: IndexMap<EnumValueName, EnumValue>,
}
// ... accessor methods
```

- [ ] **Step 4: Run check, commit**

```
[libgraphql-core-v2] Add ScalarType, EnumType, EnumValue, DeprecationState
```

---

### Task 10: Field, Parameter, and HasFieldsAndInterfaces Trait

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/field.rs`
- Modify: `crates/libgraphql-core-v2/src/types/mod.rs`

- [ ] **Step 1: Implement Parameter**

```rust
use crate::names::FieldName;
use crate::span::Span;
use crate::types::type_annotation::TypeAnnotation;
use crate::value::Value;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Parameter {
    pub(crate) def_location: Span,
    pub(crate) default_value: Option<Value>,
    pub(crate) description: Option<String>,
    pub(crate) name: FieldName,
    pub(crate) type_annotation: TypeAnnotation,
}
// ... accessor methods
```

- [ ] **Step 2: Implement Field**

```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::type_annotation::TypeAnnotation;
use indexmap::IndexMap;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Field {
    pub(crate) def_location: Span,
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: FieldName,
    pub(crate) parameters: IndexMap<FieldName, Parameter>,
    pub(crate) parent_type_name: TypeName,
    pub(crate) type_annotation: TypeAnnotation,
}

impl Field {
    pub fn name(&self) -> &FieldName { &self.name }
    pub fn type_annotation(&self) -> &TypeAnnotation { &self.type_annotation }
    pub fn parameters(&self) -> &IndexMap<FieldName, Parameter> { &self.parameters }
    pub fn parent_type_name(&self) -> &TypeName { &self.parent_type_name }
    pub fn return_type_name(&self) -> &TypeName {
        self.type_annotation.innermost_type_name()
    }
    // ... remaining accessors
}
```

- [ ] **Step 3: Define HasFieldsAndInterfaces trait in types/mod.rs**

```rust
use crate::names::FieldName;
use crate::names::TypeName;
use crate::types::field::Field;
use indexmap::IndexMap;

/// Trait shared by [`ObjectType`] and [`InterfaceType`], both of which
/// define fields and can implement interfaces.
///
/// This enables the
/// [`ObjectOrInterfaceValidator`](crate::validators::object_or_interface_validator)
/// to be generic over both types, and allows downstream consumers to
/// write code that operates on either.
pub trait HasFieldsAndInterfaces {
    fn def_location(&self) -> crate::span::Span;
    fn description(&self) -> Option<&str>;
    fn directives(&self) -> &[crate::directive_annotation::DirectiveAnnotation];
    fn field(&self, name: &str) -> Option<&Field>;
    fn fields(&self) -> &IndexMap<FieldName, Field>;
    fn interface_names(&self) -> &[TypeName];
    fn name(&self) -> &TypeName;
}
```

- [ ] **Step 4: Commit**

```
[libgraphql-core-v2] Add Field, Parameter, HasFieldsAndInterfaces trait
```

---

### Task 11: ObjectType and InterfaceType

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/object_type.rs`
- Create: `crates/libgraphql-core-v2/src/types/interface_type.rs`

- [ ] **Step 1: Implement shared data struct + ObjectType**

```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::field::Field;
use crate::types::HasFieldsAndInterfaces;
use indexmap::IndexMap;

/// Internal data shared by ObjectType and InterfaceType.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct FieldedTypeData {
    pub(crate) def_location: Span,
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) fields: IndexMap<FieldName, Field>,
    pub(crate) interfaces: Vec<TypeName>,
    pub(crate) name: TypeName,
}

/// A GraphQL [object type](https://spec.graphql.org/September2025/#sec-Objects).
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ObjectType(pub(crate) FieldedTypeData);

impl HasFieldsAndInterfaces for ObjectType {
    fn def_location(&self) -> Span { self.0.def_location }
    fn description(&self) -> Option<&str> { self.0.description.as_deref() }
    fn directives(&self) -> &[DirectiveAnnotation] { &self.0.directives }
    fn field(&self, name: &str) -> Option<&Field> { self.0.fields.get(name) }
    fn fields(&self) -> &IndexMap<FieldName, Field> { &self.0.fields }
    fn interface_names(&self) -> &[TypeName] { &self.0.interfaces }
    fn name(&self) -> &TypeName { &self.0.name }
}
```

- [ ] **Step 2: Implement InterfaceType (same pattern)**

```rust
/// A GraphQL [interface type](https://spec.graphql.org/September2025/#sec-Interfaces).
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct InterfaceType(pub(crate) FieldedTypeData);

impl HasFieldsAndInterfaces for InterfaceType {
    // ... identical delegation to self.0
}
```

- [ ] **Step 3: Commit**

```
[libgraphql-core-v2] Add ObjectType and InterfaceType with shared trait
```

---

### Task 12: UnionType, InputObjectType, InputField

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/union_type.rs`
- Create: `crates/libgraphql-core-v2/src/types/input_object_type.rs`

- [ ] **Step 1: Implement UnionType**

```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::TypeName;
use crate::span::Span;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct UnionType {
    pub(crate) def_location: Span,
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) members: Vec<TypeName>,
    pub(crate) name: TypeName,
}
// ... accessor methods
```

- [ ] **Step 2: Implement InputField and InputObjectType**

```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::type_annotation::TypeAnnotation;
use crate::value::Value;
use indexmap::IndexMap;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct InputField {
    pub(crate) def_location: Span,
    pub(crate) default_value: Option<Value>,
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: FieldName,
    pub(crate) parent_type_name: TypeName,
    pub(crate) type_annotation: TypeAnnotation,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct InputObjectType {
    pub(crate) def_location: Span,
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) fields: IndexMap<FieldName, InputField>,
    pub(crate) name: TypeName,
}
// ... accessor methods
```

- [ ] **Step 3: Commit**

```
[libgraphql-core-v2] Add UnionType, InputObjectType, InputField
```

---

### Task 13: DirectiveDefinition and DirectiveLocation

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/directive_definition.rs`
- Create: `crates/libgraphql-core-v2/src/types/directive_location.rs`

- [ ] **Step 1: Implement DirectiveLocation**

```rust
/// All valid directive locations per the GraphQL spec.
/// See [Directive Locations](https://spec.graphql.org/September2025/#DirectiveLocation).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum DirectiveLocation {
    // Executable locations
    ArgumentDefinition,
    EnumValue,
    Field,
    FieldDefinition,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    InputFieldDefinition,
    InputObject,
    Mutation,
    Object,
    Query,
    Scalar,
    Schema,
    Subscription,
    Union,
    VariableDefinition,
    // Type system locations
    Enum,
    Interface,
}
```

- [ ] **Step 2: Implement DirectiveDefinition (unified struct)**

```rust
use crate::names::DirectiveName;
use crate::names::FieldName;
use crate::span::Span;
use crate::types::directive_location::DirectiveLocation;
use crate::types::field::Parameter;
use indexmap::IndexMap;

/// A directive definition (schema-level).
///
/// Unlike v1's `Directive` enum with separate built-in variants, v2
/// uses a single struct for all directives. Built-in directives
/// (`@skip`, `@include`, `@deprecated`, `@specifiedBy`) are regular
/// entries with `builtin: true`.
///
/// See [Directive Definitions](https://spec.graphql.org/September2025/#sec-Type-System.Directives).
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DirectiveDefinition {
    pub(crate) builtin: bool,
    pub(crate) def_location: Span,
    pub(crate) description: Option<String>,
    pub(crate) is_repeatable: bool,
    pub(crate) locations: Vec<DirectiveLocation>,
    pub(crate) name: DirectiveName,
    pub(crate) parameters: IndexMap<FieldName, Parameter>,
}
// ... accessor methods
```

- [ ] **Step 3: Commit**

```
[libgraphql-core-v2] Add DirectiveDefinition (unified) and DirectiveLocation
```

---

### Task 14: GraphQLType Enum + Methods

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/graphql_type.rs`
- Create: `crates/libgraphql-core-v2/src/types/graphql_type_kind.rs`

- [ ] **Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Verifies is_input_type/is_output_type correctly classify types.
    // Per GraphQL spec:
    // - Input: Scalar, Enum, InputObject
    // - Output: Scalar, Enum, Object, Interface, Union
    // https://spec.graphql.org/September2025/#sec-Input-and-Output-Types
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn input_output_classification() {
        // Scalar: both input and output
        let scalar = GraphQLType::Scalar(Box::new(ScalarType {
            builtin: true,
            def_location: Span::builtin(),
            description: None,
            directives: vec![],
            name: TypeName::new("String"),
        }));
        assert!(scalar.is_input_type());
        assert!(scalar.is_output_type());

        // Object: output only
        // (construct minimal ObjectType)
        // ...assert !is_input, is_output

        // InputObject: input only
        // ...assert is_input, !is_output
    }
}
```

- [ ] **Step 2: Implement GraphQLType**

```rust
use crate::names::TypeName;
use crate::span::Span;
use crate::types::enum_type::EnumType;
use crate::types::input_object_type::InputObjectType;
use crate::types::interface_type::InterfaceType;
use crate::types::object_type::ObjectType;
use crate::types::scalar_type::ScalarType;
use crate::types::union_type::UnionType;

/// A defined GraphQL type.
///
/// Unlike v1, built-in scalars (Boolean, Float, ID, Int, String) are
/// represented as `Scalar(ScalarType { builtin: true })` rather than
/// separate enum variants. This reduces match arms from 11 to 6.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum GraphQLType {
    Enum(Box<EnumType>),
    InputObject(Box<InputObjectType>),
    Interface(Box<InterfaceType>),
    Object(Box<ObjectType>),
    Scalar(Box<ScalarType>),
    Union(Box<UnionType>),
}

impl GraphQLType {
    pub fn name(&self) -> &TypeName {
        match self {
            Self::Enum(t) => &t.name,
            Self::InputObject(t) => &t.name,
            Self::Interface(t) => crate::types::HasFieldsAndInterfaces::name(t.as_ref()),
            Self::Object(t) => crate::types::HasFieldsAndInterfaces::name(t.as_ref()),
            Self::Scalar(t) => &t.name,
            Self::Union(t) => &t.name,
        }
    }

    pub fn def_location(&self) -> Span {
        match self {
            Self::Enum(t) => t.def_location,
            Self::InputObject(t) => t.def_location,
            Self::Interface(t) => crate::types::HasFieldsAndInterfaces::def_location(t.as_ref()),
            Self::Object(t) => crate::types::HasFieldsAndInterfaces::def_location(t.as_ref()),
            Self::Scalar(t) => t.def_location,
            Self::Union(t) => t.def_location,
        }
    }

    /// Input types: Scalar, Enum, InputObject.
    /// https://spec.graphql.org/September2025/#sec-Input-and-Output-Types
    pub fn is_input_type(&self) -> bool {
        matches!(self, Self::Enum(_) | Self::InputObject(_) | Self::Scalar(_))
    }

    /// Output types: Scalar, Enum, Object, Interface, Union.
    /// https://spec.graphql.org/September2025/#sec-Input-and-Output-Types
    pub fn is_output_type(&self) -> bool {
        matches!(
            self,
            Self::Enum(_) | Self::Interface(_) | Self::Object(_)
                | Self::Scalar(_) | Self::Union(_),
        )
    }

    pub fn is_builtin(&self) -> bool {
        match self {
            Self::Scalar(s) => s.builtin,
            _ => false,
        }
    }

    pub fn requires_selection_set(&self) -> bool {
        matches!(self, Self::Interface(_) | Self::Object(_) | Self::Union(_))
    }

    pub fn is_composite_type(&self) -> bool {
        matches!(self, Self::Interface(_) | Self::Object(_) | Self::Union(_))
    }

    pub fn is_leaf_type(&self) -> bool {
        matches!(self, Self::Enum(_) | Self::Scalar(_))
    }

    // Typed downcasts
    pub fn as_enum(&self) -> Option<&EnumType> {
        if let Self::Enum(t) = self { Some(t) } else { None }
    }
    pub fn as_input_object(&self) -> Option<&InputObjectType> {
        if let Self::InputObject(t) = self { Some(t) } else { None }
    }
    pub fn as_interface(&self) -> Option<&InterfaceType> {
        if let Self::Interface(t) = self { Some(t) } else { None }
    }
    pub fn as_object(&self) -> Option<&ObjectType> {
        if let Self::Object(t) = self { Some(t) } else { None }
    }
    pub fn as_scalar(&self) -> Option<&ScalarType> {
        if let Self::Scalar(t) = self { Some(t) } else { None }
    }
    pub fn as_union(&self) -> Option<&UnionType> {
        if let Self::Union(t) = self { Some(t) } else { None }
    }

    pub fn type_kind(&self) -> GraphQLTypeKind {
        match self {
            Self::Enum(_) => GraphQLTypeKind::Enum,
            Self::InputObject(_) => GraphQLTypeKind::InputObject,
            Self::Interface(_) => GraphQLTypeKind::Interface,
            Self::Object(_) => GraphQLTypeKind::Object,
            Self::Scalar(_) => GraphQLTypeKind::Scalar,
            Self::Union(_) => GraphQLTypeKind::Union,
        }
    }
}
```

- [ ] **Step 3: Implement GraphQLTypeKind**

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum GraphQLTypeKind {
    Enum,
    InputObject,
    Interface,
    Object,
    Scalar,
    Union,
}
```

- [ ] **Step 4: Wire up all types/mod.rs re-exports**

Ensure `types/mod.rs` re-exports all types and the `HasFieldsAndInterfaces` trait.

- [ ] **Step 5: Run tests, commit**

```
[libgraphql-core-v2] Add GraphQLType enum (6 variants) + GraphQLTypeKind
```

---

### Task 15: Schema Errors

**Files:**
- Create: `crates/libgraphql-core-v2/src/schema/schema_errors.rs`
- Create: `crates/libgraphql-core-v2/src/schema/type_validation_error.rs`

- [ ] **Step 1: Implement SchemaBuildError**

Port all ~21 variants from v1, replacing `loc::SourceLocation` with `Span`. Add new variants for previously-missing validations:
- `RootOperationTypeNotObjectType`
- `RootOperationTypeNotDefined`
- `EmptyObjectType` / `EmptyInterfaceType` / `EmptyUnionType` / `EmptyInputObjectType`
- `InvalidEnumValueName` (for `true`/`false`/`null`)
- `DuplicateDirectiveArgument`
- `DirectiveArgumentNotInputType`

- [ ] **Step 2: Implement TypeValidationError**

Port all ~13 variants from v1, with `Span` instead of `SourceLocation`. Add:
- `InvalidInputFieldWithInterfaceType` / `InvalidInputFieldWithUnionType` (v1 only caught Object)

- [ ] **Step 3: Implement SchemaErrors newtype**

```rust
/// Collection of schema build errors that implements `Error` and enables
/// `?` propagation.
#[derive(Debug)]
pub struct SchemaErrors {
    errors: Vec<SchemaBuildError>,
}

impl SchemaErrors {
    pub fn new(errors: Vec<SchemaBuildError>) -> Self {
        debug_assert!(!errors.is_empty());
        Self { errors }
    }

    pub fn errors(&self) -> &[SchemaBuildError] {
        &self.errors
    }
}

impl std::fmt::Display for SchemaErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, err) in self.errors.iter().enumerate() {
            if i > 0 { writeln!(f)?; }
            write!(f, "{err}")?;
        }
        Ok(())
    }
}

impl std::error::Error for SchemaErrors {}

impl IntoIterator for SchemaErrors {
    type Item = SchemaBuildError;
    type IntoIter = std::vec::IntoIter<SchemaBuildError>;
    fn into_iter(self) -> Self::IntoIter { self.errors.into_iter() }
}
```

- [ ] **Step 4: Commit**

```
[libgraphql-core-v2] Add SchemaBuildError, TypeValidationError, SchemaErrors
```

---

### Task 16: Type Builders (Common Types + ObjectTypeBuilder)

**Files:**
- Create: `crates/libgraphql-core-v2/src/type_builders/mod.rs`
- Create: `crates/libgraphql-core-v2/src/type_builders/common.rs`
- Create: `crates/libgraphql-core-v2/src/type_builders/object_type_builder.rs`

- [ ] **Step 1: Implement builder-stage data types in common.rs**

```rust
//! Builder-stage data types used during schema construction.

use crate::directive_annotation::DirectiveAnnotation;
use crate::names::EnumValueName;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::type_annotation::TypeAnnotation;
use crate::value::Value;

/// A field definition during building (before validation).
pub struct FieldDef {
    pub description: Option<String>,
    pub directives: Vec<DirectiveAnnotation>,
    pub name: FieldName,
    pub parameters: Vec<ParameterDef>,
    pub span: Span,
    pub type_annotation: TypeAnnotation,
}

/// A parameter definition during building.
pub struct ParameterDef {
    pub default_value: Option<Value>,
    pub description: Option<String>,
    pub name: FieldName,
    pub span: Span,
    pub type_annotation: TypeAnnotation,
}

/// An input field definition during building.
pub struct InputFieldDef {
    pub default_value: Option<Value>,
    pub description: Option<String>,
    pub directives: Vec<DirectiveAnnotation>,
    pub name: FieldName,
    pub span: Span,
    pub type_annotation: TypeAnnotation,
}

/// An enum value definition during building.
pub struct EnumValueDef {
    pub description: Option<String>,
    pub directives: Vec<DirectiveAnnotation>,
    pub name: EnumValueName,
    pub span: Span,
}
```

- [ ] **Step 2: Implement ObjectTypeBuilder**

```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::TypeName;
use crate::span::Span;
use crate::type_builders::common::FieldDef;

pub struct ObjectTypeBuilder {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) fields: Vec<FieldDef>,
    pub(crate) implements: Vec<TypeName>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

impl ObjectTypeBuilder {
    pub fn new(name: impl Into<TypeName>, span: Span) -> Self {
        Self {
            description: None,
            directives: vec![],
            fields: vec![],
            implements: vec![],
            name: name.into(),
            span,
        }
    }

    pub fn set_description(&mut self, desc: impl Into<String>) -> &mut Self {
        self.description = Some(desc.into());
        self
    }

    pub fn add_field(&mut self, field: FieldDef) -> &mut Self {
        self.fields.push(field);
        self
    }

    pub fn add_implements(&mut self, iface: impl Into<TypeName>) -> &mut Self {
        self.implements.push(iface.into());
        self
    }

    pub fn add_directive(&mut self, dir: DirectiveAnnotation) -> &mut Self {
        self.directives.push(dir);
        self
    }
}
```

Note: The `register()` method is added in Task 18 (SchemaBuilder) since it depends on `SchemaBuilder` existing.

- [ ] **Step 3: Commit**

```
[libgraphql-core-v2] Add builder-stage types and ObjectTypeBuilder
```

---

### Task 17: Remaining Type Builders

**Files:**
- Create: `crates/libgraphql-core-v2/src/type_builders/interface_type_builder.rs`
- Create: `crates/libgraphql-core-v2/src/type_builders/union_type_builder.rs`
- Create: `crates/libgraphql-core-v2/src/type_builders/enum_type_builder.rs`
- Create: `crates/libgraphql-core-v2/src/type_builders/scalar_type_builder.rs`
- Create: `crates/libgraphql-core-v2/src/type_builders/input_object_type_builder.rs`
- Create: `crates/libgraphql-core-v2/src/type_builders/directive_builder.rs`

- [ ] **Step 1: Implement all remaining builders**

Each follows the same pattern as `ObjectTypeBuilder`: `new()`, `set_*()`, `add_*()` methods. Use `InterfaceTypeBuilder` (identical to ObjectTypeBuilder with `interface` instead of `object` semantics), `UnionTypeBuilder` (adds members), `EnumTypeBuilder` (adds values), `ScalarTypeBuilder` (minimal), `InputObjectTypeBuilder` (adds input fields), `DirectiveBuilder` (adds locations, parameters, repeatable flag).

- [ ] **Step 2: Verify compilation**

Run: `cargo check --package libgraphql-core-v2`

- [ ] **Step 3: Commit**

```
[libgraphql-core-v2] Add all remaining type builders
```

---

### Task 18: SchemaBuilder Core + Registration

**Files:**
- Create: `crates/libgraphql-core-v2/src/schema/mod.rs`
- Create: `crates/libgraphql-core-v2/src/schema/schema_builder.rs`
- Create: `crates/libgraphql-core-v2/src/schema/schema.rs`

- [ ] **Step 1: Write tests for basic schema building**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Verifies a minimal schema (just Query type) builds successfully.
    // https://spec.graphql.org/September2025/#sec-Root-Operation-Types
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn minimal_schema_builds() {
        let mut sb = SchemaBuilder::new();
        let mut query = ObjectTypeBuilder::new("Query", Span::builtin());
        query.add_field(FieldDef {
            name: FieldName::new("hello"),
            type_annotation: TypeAnnotation::named("String", true),
            // ... remaining fields with defaults
        });
        sb.register_type(query).unwrap();
        let schema = sb.build().unwrap();
        assert!(schema.query_type().is_some());
    }

    // Verifies built-in scalars are present.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn builtin_scalars_present() {
        let mut sb = SchemaBuilder::new();
        // ... register minimal Query type
        let schema = sb.build().unwrap();
        assert!(schema.get_type(&TypeName::new("String")).is_some());
        assert!(schema.get_type(&TypeName::new("Int")).is_some());
        assert!(schema.get_type(&TypeName::new("Float")).is_some());
        assert!(schema.get_type(&TypeName::new("Boolean")).is_some());
        assert!(schema.get_type(&TypeName::new("ID")).is_some());
    }

    // Verifies duplicate type registration is rejected.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn duplicate_type_rejected() {
        let mut sb = SchemaBuilder::new();
        sb.register_type(ObjectTypeBuilder::new("Foo", Span::builtin())).unwrap();
        let result = sb.register_type(ObjectTypeBuilder::new("Foo", Span::builtin()));
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Implement SchemaBuilder**

```rust
use crate::names::DirectiveName;
use crate::names::TypeName;
use crate::owned_source_map::OwnedSourceMap;
use crate::schema::schema_errors::SchemaBuildError;
use crate::schema::schema_errors::SchemaErrors;
use crate::span::SourceMapId;
use crate::span::Span;
use crate::types::GraphQLType;
use crate::types::directive_definition::DirectiveDefinition;
use indexmap::IndexMap;

pub struct SchemaBuilder {
    directive_defs: IndexMap<DirectiveName, DirectiveDefinition>,
    errors: Vec<SchemaBuildError>,
    mutation_type_name: Option<(TypeName, Span)>,
    // pending_extensions: Vec<TypeExtensionData>,
    query_type_name: Option<(TypeName, Span)>,
    source_maps: Vec<OwnedSourceMap>,
    subscription_type_name: Option<(TypeName, Span)>,
    types: IndexMap<TypeName, GraphQLType>,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        let mut builder = Self {
            directive_defs: IndexMap::new(),
            errors: vec![],
            mutation_type_name: None,
            query_type_name: None,
            source_maps: vec![OwnedSourceMap::builtin()],
            subscription_type_name: None,
            types: IndexMap::new(),
        };
        builder.seed_builtins();
        builder
    }

    /// Register a type builder. Performs early validation (name checks,
    /// duplicate detection) and inserts into the accumulator.
    pub fn register_type(
        &mut self,
        builder: impl Into<TypeBuilderInput>,
    ) -> Result<(), SchemaBuildError> {
        let input: TypeBuilderInput = builder.into();
        // ... validate name, check duplicates, insert
        todo!()
    }

    /// Load a ParseResult from the parser.
    pub fn load_parse_result(
        &mut self,
        parse_result: &libgraphql_parser::ParseResult<
            '_,
            libgraphql_parser::ast::Document<'_>,
        >,
    ) -> Result<(), Vec<SchemaBuildError>> {
        // ... iterate definitions, create builders, register
        todo!()
    }

    /// Convenience: parse a string and load it.
    pub fn load_str(
        &mut self,
        source: &str,
    ) -> Result<(), Vec<SchemaBuildError>> {
        let parse_result = libgraphql_parser::parse_schema(source);
        self.load_parse_result(&parse_result)
    }

    pub fn build(self) -> Result<crate::schema::Schema, SchemaErrors> {
        // 1. Apply pending extensions
        // 2. Run all validators
        // 3. Resolve root operation types
        // 4. Return Schema or errors
        todo!()
    }

    fn seed_builtins(&mut self) {
        // Insert Boolean, Float, ID, Int, String as ScalarType { builtin: true }
        // Insert @skip, @include, @deprecated, @specifiedBy
    }
}
```

- [ ] **Step 3: Implement Schema struct with typed query API**

```rust
use crate::names::DirectiveName;
use crate::names::TypeName;
use crate::owned_source_map::OwnedSourceMap;
use crate::types::GraphQLType;
use crate::types::directive_definition::DirectiveDefinition;
use crate::types::enum_type::EnumType;
use crate::types::input_object_type::InputObjectType;
use crate::types::interface_type::InterfaceType;
use crate::types::object_type::ObjectType;
use crate::types::scalar_type::ScalarType;
use crate::types::union_type::UnionType;
use indexmap::IndexMap;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Schema {
    pub(crate) directive_defs: IndexMap<DirectiveName, DirectiveDefinition>,
    pub(crate) mutation_type_name: Option<TypeName>,
    pub(crate) query_type_name: TypeName,
    pub(crate) source_maps: Vec<OwnedSourceMap>,
    pub(crate) subscription_type_name: Option<TypeName>,
    pub(crate) types: IndexMap<TypeName, GraphQLType>,
}

impl Schema {
    // --- Generic lookups ---
    pub fn get_type(&self, name: &TypeName) -> Option<&GraphQLType> {
        self.types.get(name)
    }

    pub fn get_directive(&self, name: &DirectiveName) -> Option<&DirectiveDefinition> {
        self.directive_defs.get(name)
    }

    // --- Typed lookups ---
    pub fn object_type(&self, name: &TypeName) -> Option<&ObjectType> {
        self.types.get(name).and_then(|t| t.as_object())
    }

    pub fn interface_type(&self, name: &TypeName) -> Option<&InterfaceType> {
        self.types.get(name).and_then(|t| t.as_interface())
    }

    pub fn enum_type(&self, name: &TypeName) -> Option<&EnumType> {
        self.types.get(name).and_then(|t| t.as_enum())
    }

    pub fn scalar_type(&self, name: &TypeName) -> Option<&ScalarType> {
        self.types.get(name).and_then(|t| t.as_scalar())
    }

    pub fn union_type(&self, name: &TypeName) -> Option<&UnionType> {
        self.types.get(name).and_then(|t| t.as_union())
    }

    pub fn input_object_type(&self, name: &TypeName) -> Option<&InputObjectType> {
        self.types.get(name).and_then(|t| t.as_input_object())
    }

    // --- Typed iterators ---
    pub fn object_types(&self) -> impl Iterator<Item = &ObjectType> {
        self.types.values().filter_map(|t| t.as_object())
    }

    pub fn interface_types(&self) -> impl Iterator<Item = &InterfaceType> {
        self.types.values().filter_map(|t| t.as_interface())
    }

    /// All types that implement a given interface.
    pub fn types_implementing(
        &self,
        interface_name: &TypeName,
    ) -> impl Iterator<Item = &GraphQLType> {
        let name = interface_name.clone();
        self.types.values().filter(move |t| {
            match t {
                GraphQLType::Object(obj) => {
                    crate::types::HasFieldsAndInterfaces::interface_names(obj.as_ref())
                        .iter()
                        .any(|n| n == &name)
                },
                GraphQLType::Interface(iface) => {
                    crate::types::HasFieldsAndInterfaces::interface_names(iface.as_ref())
                        .iter()
                        .any(|n| n == &name)
                },
                _ => false,
            }
        })
    }

    // --- Root operation types ---
    pub fn query_type(&self) -> Option<&ObjectType> {
        self.object_type(&self.query_type_name)
    }

    pub fn mutation_type(&self) -> Option<&ObjectType> {
        self.mutation_type_name.as_ref().and_then(|n| self.object_type(n))
    }

    pub fn subscription_type(&self) -> Option<&ObjectType> {
        self.subscription_type_name.as_ref().and_then(|n| self.object_type(n))
    }
}
```

- [ ] **Step 4: Run tests, commit**

```
[libgraphql-core-v2] Add SchemaBuilder, Schema with typed query API
```

---

### Task 19: AST Conversion (Parser AST -> Builders)

**Files:**
- Create: `crates/libgraphql-core-v2/src/type_builders/ast_conversion.rs`

- [ ] **Step 1: Implement from_ast helpers**

This module converts `libgraphql_parser::ast::*` nodes into builder types. Each function takes a parser AST node + `SourceMapId` and returns the corresponding builder with all `Cow<'src, str>` values converted to owned `String`s via `.to_string()` or `.into_owned()`.

Key functions:
- `object_type_builder_from_ast(&ObjectTypeDefinition, SourceMapId) -> ObjectTypeBuilder`
- `interface_type_builder_from_ast(&InterfaceTypeDefinition, SourceMapId) -> InterfaceTypeBuilder`
- `field_def_from_ast(&FieldDefinition, SourceMapId) -> FieldDef`
- `type_annotation_from_ast(&TypeAnnotation, SourceMapId) -> types::TypeAnnotation`
- `value_from_ast(&Value) -> value::Value`
- `directive_annotation_from_ast(&DirectiveAnnotation, SourceMapId) -> DirectiveAnnotation`
- ... (one per AST node type)

- [ ] **Step 2: Write tests with round-trip parsing**

```rust
#[cfg(test)]
mod tests {
    // Verifies that a parsed ObjectTypeDefinition converts to a correct
    // ObjectTypeBuilder.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn object_type_from_ast() {
        let result = libgraphql_parser::parse_schema(
            "type User { id: ID!, name: String }",
        );
        let (doc, _) = result.valid().unwrap();
        // ... extract ObjectTypeDefinition, convert, verify fields
    }
}
```

- [ ] **Step 3: Commit**

```
[libgraphql-core-v2] Add AST conversion (parser AST -> builders)
```

---

### Task 20: Validators

**Files:**
- Create: `crates/libgraphql-core-v2/src/validators/mod.rs`
- Create: `crates/libgraphql-core-v2/src/validators/object_or_interface_validator.rs`
- Create: `crates/libgraphql-core-v2/src/validators/union_validator.rs`
- Create: `crates/libgraphql-core-v2/src/validators/input_object_validator.rs`
- Create: `crates/libgraphql-core-v2/src/validators/directive_validator.rs`
- Create: `crates/libgraphql-core-v2/src/validators/type_ref_validator.rs`

- [ ] **Step 1: Port ObjectOrInterfaceValidator from v1**

Port `/crates/libgraphql-core/src/types/object_or_interface_type_validator.rs`. Key changes:
- Generic over `T: HasFieldsAndInterfaces` instead of taking `ObjectOrInterfaceTypeData`
- Use `TypeName` / `FieldName` instead of `&str`
- Use `Span` instead of `SourceLocation`
- Use `IndexMap<TypeName, GraphQLType>` instead of `HashMap<String, GraphQLType>`
- **Fix v1 bug:** Input object validator must reject Interface and Union types as input fields (not just Object)

- [ ] **Step 2: Port UnionValidator from v1**

Port `/crates/libgraphql-core/src/types/union_type_validator.rs`. Add check for empty union (U3 from spec audit).

- [ ] **Step 3: Port InputObjectValidator from v1**

Port `/crates/libgraphql-core/src/types/input_object_type_validator.rs`. **Fix v1 bug:** Use `!is_input_type()` instead of `as_object().is_some()`.

- [ ] **Step 4: Implement DirectiveValidator (NEW)**

New validator that v1 was missing entirely:
- Directive argument types must be input types (D4)
- Directive argument names must not start with `__` (D5)
- Directive argument names must be unique (D6)
- Directive locations are stored and valid (D8)

- [ ] **Step 5: Implement TypeRefValidator**

Validates that all type annotations across all types resolve to defined types, and that output fields use output types while input fields/parameters use input types.

- [ ] **Step 6: Write comprehensive validator tests**

Test each validation rule with both valid and invalid schemas. At minimum:
- Interface implementation with missing field
- Interface implementation with wrong param type
- Union with non-object member
- Input object circular reference
- Input object with output-type field (Object, Interface, Union)
- `__`-prefixed names
- Empty types (no fields/values/members)
- `true`/`false`/`null` enum values

- [ ] **Step 7: Commit**

```
[libgraphql-core-v2] Add all schema validators (object/interface, union, input, directive, type-ref)
```

---

### Task 21: SchemaBuilder::build() Orchestration

**Files:**
- Modify: `crates/libgraphql-core-v2/src/schema/schema_builder.rs`

- [ ] **Step 1: Implement the full build() method**

```rust
pub fn build(mut self) -> Result<Schema, SchemaErrors> {
    // 1. Apply pending type extensions
    self.apply_pending_extensions();

    // 2. Validate root operation types
    self.validate_root_operation_types();

    // 3. Run type validators
    for (_, graphql_type) in &self.types {
        match graphql_type {
            GraphQLType::Object(obj) => {
                let validator = ObjectOrInterfaceValidator::new(
                    obj.as_ref(), &self.types,
                );
                self.errors.extend(
                    validator.validate(&mut HashSet::new())
                        .into_iter()
                        .map(SchemaBuildError::TypeValidation),
                );
            },
            GraphQLType::Interface(iface) => { /* same pattern */ },
            GraphQLType::Union(union_type) => {
                let validator = UnionValidator::new(
                    union_type, &self.types,
                );
                self.errors.extend(
                    validator.validate()
                        .into_iter()
                        .map(SchemaBuildError::TypeValidation),
                );
            },
            GraphQLType::InputObject(input_obj) => { /* InputObjectValidator */ },
            _ => {},
        }
    }

    // 4. Run directive validators
    // 5. Run type-ref validators

    if !self.errors.is_empty() {
        return Err(SchemaErrors::new(self.errors));
    }

    let query_type_name = self.query_type_name
        .map(|(name, _)| name)
        .unwrap_or_else(|| TypeName::new("Query"));

    Ok(Schema {
        directive_defs: self.directive_defs,
        mutation_type_name: self.mutation_type_name.map(|(n, _)| n),
        query_type_name,
        source_maps: self.source_maps,
        subscription_type_name: self.subscription_type_name.map(|(n, _)| n),
        types: self.types,
    })
}
```

- [ ] **Step 2: Write end-to-end schema building tests**

Test the full pipeline: parse string -> load -> build -> query.

- [ ] **Step 3: Commit**

```
[libgraphql-core-v2] Implement SchemaBuilder::build() with full validation
```

---

### Task 22: Operation Types (Operation, Variable, SelectionSet)

**Files:**
- Create all files under `crates/libgraphql-core-v2/src/operation/`

- [ ] **Step 1: Implement Operation, OperationKind, Variable**

```rust
// operation.rs
pub struct Operation {
    pub(crate) def_location: Span,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) kind: OperationKind,
    pub(crate) name: Option<String>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) variables: IndexMap<VariableName, Variable>,
}

impl Operation {
    pub fn root_type_name(&self, schema: &Schema) -> &TypeName {
        match self.kind {
            OperationKind::Query => &schema.query_type_name,
            OperationKind::Mutation => schema.mutation_type_name.as_ref()
                .expect("validated at build time"),
            OperationKind::Subscription => schema.subscription_type_name.as_ref()
                .expect("validated at build time"),
        }
    }
    // ... accessors
}
```

- [ ] **Step 2: Implement SelectionSet with all_fields() iterator**

```rust
pub struct SelectionSet {
    pub(crate) selections: Vec<Selection>,
    pub(crate) span: Span,
}

impl SelectionSet {
    pub fn selections(&self) -> &[Selection] {
        &self.selections
    }

    pub fn field_selections(&self) -> impl Iterator<Item = &FieldSelection> {
        self.selections.iter().filter_map(|s| {
            if let Selection::Field(f) = s { Some(f) } else { None }
        })
    }
}
```

- [ ] **Step 3: Implement FieldSelection with pre-resolved metadata**

```rust
pub struct FieldSelection {
    pub(crate) alias: Option<FieldName>,
    pub(crate) arguments: IndexMap<FieldName, Value>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) field_name: FieldName,
    pub(crate) field_return_type_name: TypeName,
    pub(crate) parent_type_name: TypeName,
    pub(crate) requires_selection_set: bool,
    pub(crate) selection_set: Option<SelectionSet>,
    pub(crate) span: Span,
}
```

- [ ] **Step 4: Implement Fragment, FragmentSpread, InlineFragment, Selection**
- [ ] **Step 5: Commit**

```
[libgraphql-core-v2] Add Operation, SelectionSet, FieldSelection, Fragment types
```

---

### Task 23: OperationBuilder + FragmentRegistryBuilder

**Files:**
- Create: `crates/libgraphql-core-v2/src/operation/operation_builder.rs`
- Create: `crates/libgraphql-core-v2/src/operation/selection_set_builder.rs`
- Create: `crates/libgraphql-core-v2/src/operation/fragment_builder.rs`
- Create: `crates/libgraphql-core-v2/src/operation/fragment_registry_builder.rs`

- [ ] **Step 1: Implement OperationBuilder**

Follows v1's flow but simplified:
1. `from_ast(&Schema, Option<&FragmentRegistry>, ast, source_map_id)`
2. Validates variables, directives
3. Builds SelectionSet via SelectionSetBuilder
4. Returns `Operation`

- [ ] **Step 2: Implement SelectionSetBuilder**

Key validation during building:
- Fields exist on parent type (F1)
- `__typename` available on composite types (F2)
- Leaf fields must not have sub-selections (F4)
- Composite fields must have sub-selections (F5)
- Arguments correspond to field definition (A1)
- Required arguments provided (A3)
- Pre-resolves `field_return_type_name` and `requires_selection_set`

- [ ] **Step 3: Implement FragmentRegistryBuilder**

Port v1's cycle detection (DFS with phase normalization for deduplication). Add:
- Fragment type condition must be composite type (FR3)
- Fragment spread type applicability (FR7)

- [ ] **Step 4: Write operation building tests**
- [ ] **Step 5: Commit**

```
[libgraphql-core-v2] Add OperationBuilder, SelectionSetBuilder, FragmentRegistryBuilder
```

---

### Task 24: ExecutableDocument + Integration Tests

**Files:**
- Create: `crates/libgraphql-core-v2/src/operation/executable_document.rs`
- Create: `crates/libgraphql-core-v2/src/operation/executable_document_builder.rs`

- [ ] **Step 1: Implement ExecutableDocumentBuilder**
- [ ] **Step 2: Write full-pipeline integration tests**

Test: parse schema string -> build Schema -> parse operation string -> build Operation -> query results.

- [ ] **Step 3: Commit**

```
[libgraphql-core-v2] Add ExecutableDocumentBuilder + integration tests
```

---

### Task 25: Macro Runtime + Serde/Bincode

**Files:**
- Create: `crates/libgraphql-core-v2/src/schema/_macro_runtime.rs`

- [ ] **Step 1: Implement macro runtime**

```rust
use crate::schema::Schema;

pub fn build_from_macro_serialized(serialized_schema: &[u8]) -> Schema {
    bincode::serde::decode_from_slice::<Schema, _>(
        serialized_schema,
        bincode::config::standard(),
    ).expect("Failed to deserialize precompiled Schema").0
}
```

- [ ] **Step 2: Write round-trip serialization tests**

```rust
#[cfg(test)]
mod tests {
    // Verifies Schema survives bincode round-trip.
    // This is critical for the macro crate integration.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn schema_bincode_roundtrip() {
        let mut sb = SchemaBuilder::new();
        sb.load_str("type Query { hello: String }").unwrap();
        let schema = sb.build().unwrap();

        let bytes = bincode::serde::encode_to_vec(
            &schema, bincode::config::standard(),
        ).unwrap();
        let roundtripped = build_from_macro_serialized(&bytes);
        assert_eq!(schema, roundtripped);
    }
}
```

- [ ] **Step 3: Commit**

```
[libgraphql-core-v2] Add macro runtime + serde/bincode round-trip tests
```

---

### Task 26: Comprehensive Test Suite

**Files:**
- Create: `crates/libgraphql-core-v2/src/schema/tests/`
- Create: `crates/libgraphql-core-v2/src/types/tests/`
- Create: `crates/libgraphql-core-v2/src/operation/tests/`

- [ ] **Step 1: Port v1 schema tests**

Adapt tests from `/crates/libgraphql-core/src/schema/tests/` and `/crates/libgraphql-core/src/test/snapshot_tests/`. Use the `.graphql` fixture files from v1.

- [ ] **Step 2: Port v1 type tests**
- [ ] **Step 3: Port v1 operation tests**
- [ ] **Step 4: Add new tests for previously-missing validations**

Test each validation rule identified as MISSING in the spec audit:
- Root operation types must be Object types (S3/S4/S5)
- Empty types rejected (O1/I1/U3/IO7)
- `true`/`false`/`null` enum values rejected (E3)
- Subscription single root field (OP3)
- Leaf/composite sub-selection validation (F4/F5)
- Directive usage validation (DU1-DU5)
- Variable type validation (VA2/VA3)

- [ ] **Step 5: Run full test suite**

Run: `cargo test --package libgraphql-core-v2`

- [ ] **Step 6: Commit**

```
[libgraphql-core-v2] Add comprehensive test suite
```

---

## Validation Coverage Checklist

Validations that must be implemented, organized by where they run:

### During `register_type()` (early, structural):
- [ ] Type names must not start with `__`
- [ ] Duplicate type definition rejected
- [ ] Duplicate field names within a type rejected
- [ ] Duplicate enum values rejected
- [ ] Duplicate interface implementation declarations rejected
- [ ] Duplicate union members rejected
- [ ] Enum values must not be named `true`/`false`/`null`
- [ ] Field/param/directive argument names must not start with `__`
- [ ] Param names unique within a field

### During `build()` (cross-type validation):
- [ ] Root operation types must be Object types
- [ ] Root operation type names must resolve to defined types
- [ ] Query type must exist (or type named "Query" must exist)
- [ ] Object/Interface must define >= 1 field
- [ ] Union must define >= 1 member
- [ ] Interface implementation: field presence, param equivalence, return covariance
- [ ] Interface implementation: additional params must be optional
- [ ] Interface implementation: transitive (recursive)
- [ ] Union members must be Object types
- [ ] Input field types must be input types (Scalar/Enum/InputObject only)
- [ ] Output field types must be output types
- [ ] Parameter types must be input types
- [ ] Input object circular non-nullable reference detection
- [ ] All type references resolve to defined types
- [ ] Directive argument types must be input types
- [ ] Directive argument names must not start with `__`
- [ ] Extension of undefined types rejected
- [ ] Extension type kind mismatch rejected

### During operation building:
- [ ] Fields exist on parent type
- [ ] `__typename` selectable on composite types
- [ ] Leaf fields must not have sub-selections
- [ ] Composite fields must have sub-selections
- [ ] Arguments correspond to field/directive definition
- [ ] Required arguments provided
- [ ] Duplicate arguments rejected
- [ ] Fragment type condition on composite type
- [ ] Fragment cycle detection
- [ ] Undefined fragment reference rejected
- [ ] Duplicate fragment names rejected
- [ ] Variable names unique
- [ ] Variable types must be input types
- [ ] Subscription: single root field
- [ ] Directives used must be defined
- [ ] Non-repeatable directives applied at most once per location

### Deferred (future work, lower priority):
- [ ] Field selection merging (same response name compatibility)
- [ ] Variable usage defined + type compatible
- [ ] Value type coercion validation
- [ ] Input object literal field existence + uniqueness
- [ ] All variables must be used / All fragments must be used

---

## Critical Reference Files

| v1 File | What to port |
|---------|-------------|
| `crates/libgraphql-core/src/schema/schema_builder.rs` | Build flow, extension merging, built-in injection |
| `crates/libgraphql-core/src/types/object_or_interface_type_validator.rs` | Interface contract validation |
| `crates/libgraphql-core/src/types/union_type_validator.rs` | Union member validation |
| `crates/libgraphql-core/src/types/input_object_type_validator.rs` | Circular ref detection (**fix:** reject Interface/Union too) |
| `crates/libgraphql-core/src/types/types_map_builder.rs` | Cross-type validation orchestration |
| `crates/libgraphql-core/src/types/type_annotation.rs` | Subtype/equivalence logic |
| `crates/libgraphql-core/src/operation/fragment_registry_builder.rs` | Fragment cycle detection |
| `crates/libgraphql-core/src/operation/selection_set_builder.rs` | Selection validation |
| `crates/libgraphql-core/src/schema/_macro_runtime.rs` | Macro serialization |
| `crates/libgraphql-macros/src/emittable_schema.rs` | Macro emit pattern |
