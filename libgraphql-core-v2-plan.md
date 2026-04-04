# libgraphql-core-v2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a from-scratch rewrite of `libgraphql-core` that consumes `libgraphql-parser` AST directly, exposes public type builders, leverages Rust's type system for safety, and implements complete GraphQL September 2025 spec validation.

**Architecture:** Owned semantic types (no lifetime params) built from parser AST via public builders registered with `SchemaBuilder`. Name newtypes (`TypeName`, `FieldName`, etc.) prevent cross-domain confusion. A shared `HasFieldsAndInterfaces` trait enables generic validation over Object/Interface types. Kind-discriminator enums (`ScalarKind`, `DirectiveDefinitionKind`, `GraphQLTypeKind`) enable exhaustive matching without inflating data-carrying enum variant counts. `SchemaBuilder::build()` runs comprehensive cross-type validation and returns `Result<Schema, SchemaErrors>`. Operations are a single `Operation` type with `kind: OperationKind`. All types are serde-serializable for macro crate integration.

**Tech Stack:** Rust 2024 edition, `libgraphql-parser`, `inherent`, `serde`+`bincode`, `indexmap`, `thiserror`

---

## Architectural Decisions

### AD1. Name Newtypes — Prevent cross-domain string confusion

Each GraphQL name domain gets its own `#[repr(transparent)]` newtype, defined explicitly (no `macro_rules!`). Common behavior is shared via a private trait + `#[inherent]` delegation. Each type lives in its own file under `names/`.

```rust
pub struct TypeName(String);      // type names: User, Query, String
pub struct FieldName(String);     // field/param names: firstName, id
pub struct VariableName(String);  // variable names: userId (no $ prefix)
pub struct DirectiveName(String); // directive names: deprecated (no @ prefix)
pub struct EnumValueName(String); // enum value names: ACTIVE, ADMIN
pub struct FragmentName(String);  // fragment names: UserFields
```

### AD2. Shared Trait for Object/Interface Types

```rust
// In has_fields_and_interfaces.rs
pub trait HasFieldsAndInterfaces {
    fn def_location(&self) -> Span;
    fn description(&self) -> Option<&str>;
    fn directives(&self) -> &[DirectiveAnnotation];
    fn field(&self, name: &str) -> Option<&FieldDefinition>;
    fn fields(&self) -> &IndexMap<FieldName, FieldDefinition>;
    fn interface_names(&self) -> &[TypeName];
    fn name(&self) -> &TypeName;
}
```

Both `ObjectType` and `InterfaceType` wrap a shared `FieldedTypeData` struct and implement this trait. Validators are generic over `T: HasFieldsAndInterfaces`.

### AD3. DirectiveDefinition: Unified struct + kind discriminator

```rust
pub enum DirectiveDefinitionKind {
    Custom,
    Deprecated,
    Include,
    Skip,
    SpecifiedBy,
}

pub struct DirectiveDefinition {
    pub(crate) kind: DirectiveDefinitionKind,
    pub(crate) description: Option<String>,
    pub(crate) is_repeatable: bool,
    pub(crate) locations: Vec<DirectiveLocationKind>,
    pub(crate) name: DirectiveName,
    pub(crate) parameters: IndexMap<FieldName, ParameterDefinition>,
    pub(crate) span: Span,
}
```

Uniform data access, exhaustive matching via `.kind()`.

### AD4. GraphQLType: 6 data variants + ScalarKind discriminator

```rust
pub enum ScalarKind {
    Boolean,
    Custom,
    Float,
    ID,
    Int,
    String,
}

pub struct ScalarType {
    pub(crate) kind: ScalarKind,
    pub(crate) name: TypeName,
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) span: Span,
}

pub enum GraphQLType {
    Enum(Box<EnumType>),
    InputObject(Box<InputObjectType>),
    Interface(Box<InterfaceType>),
    Object(Box<ObjectType>),
    Scalar(Box<ScalarType>),
    Union(Box<UnionType>),
}

// 11 variants for exhaustive matching including built-in scalar identity
pub enum GraphQLTypeKind {
    Boolean, Enum, Float, ID, InputObject,
    Int, Interface, Object, Scalar, String, Union,
}
```

`GraphQLType` methods (`name()`, `def_location()`, etc.) have 6 arms. Exhaustive matching over all types including built-in scalar identity via `.type_kind()` -> `GraphQLTypeKind` (11 variants).

### AD5. Source Locations: Span = ByteSpan + SourceMapId

```rust
pub struct SourceMapId(u16);
pub struct Span { pub byte_span: ByteSpan, pub source_map_id: SourceMapId }
pub const BUILTIN_SOURCE_MAP_ID: SourceMapId = SourceMapId(0);
```

### AD6. SchemaSourceMap: owned, serializable subset of parser SourceMap

`libgraphql_parser::SourceMap<'src>` borrows source text via `'src` and isn't serializable. `Schema` must be `'static` and serde-serializable (macro crate embeds it as bytes). `SchemaSourceMap` stores just line-start offsets and file path — enough to resolve byte offsets to line/column on demand.

```rust
pub struct SchemaSourceMap {
    pub file_path: Option<PathBuf>,
    pub line_starts: Vec<u32>,
}
```

### AD7. SchemaErrors Newtype

`SchemaBuilder::build()` returns `Result<Schema, SchemaErrors>` where `SchemaErrors: Error + Display + IntoIterator<Item = SchemaBuildError>`, enabling `?` propagation.

### AD8. FieldDefinition (not "Field") for schema field definitions

Clear nominal distinction: `FieldDefinition` is a field defined on an Object/Interface type in the schema. `FieldSelection` is a field selected in an operation. Matches `libgraphql-parser`'s naming.

### AD9. Registration via SchemaBuilder

`schema_builder.register_type(builder)` (subject-verb-object) rather than `builder.register(&mut sb)`.

### AD10. Builder from_ast() as methods, shared helpers as private module

`ObjectTypeBuilder::from_ast(&ast_node, source_map_id)` is a method on the builder. Shared conversion helpers (`type_annotation_from_ast()`, `value_from_ast()`, `directive_annotation_from_ast()`) live in a private `ast_helpers.rs` module.

### AD11. Pre-Resolved FieldSelection

`FieldSelection` stores pre-resolved metadata (`parent_type_name`, `field_return_type_name`, `requires_selection_set`) validated at build time, queryable without `&Schema`.

### AD12. Typed Schema Query API

`schema.object_type(&name)`, `schema.interface_types()`, `schema.types_implementing(&name)`, etc. — typed accessors and iterators as thin wrappers with zero storage cost.

---

## File Structure

Each type in its own file, `mod.rs` files only for module declarations + re-exports.

```
crates/libgraphql-core-v2/
  Cargo.toml
  src/
    lib.rs

    // ---- Names (one file per newtype) ----
    names/
      mod.rs
      graphql_name.rs                -- Private GraphQLName trait (shared behavior)
      type_name.rs                   -- TypeName
      field_name.rs                  -- FieldName
      variable_name.rs               -- VariableName
      directive_name.rs              -- DirectiveName
      enum_value_name.rs             -- EnumValueName
      fragment_name.rs               -- FragmentName

    // ---- Foundational ----
    span.rs                          -- Span, SourceMapId, BUILTIN_SOURCE_MAP_ID
    schema_source_map.rs             -- SchemaSourceMap
    value.rs                         -- Value enum
    directive_annotation.rs          -- DirectiveAnnotation (applied instance)
    readonly_map.rs                  -- ReadOnlyMap
    file_reader.rs                   -- read_content()

    // ---- Type system (immutable, validated) ----
    types/
      mod.rs
      has_fields_and_interfaces.rs   -- HasFieldsAndInterfaces trait
      fielded_type_data.rs           -- FieldedTypeData (shared Object/Interface data)
      type_ref.rs                    -- TypeRef
      directive_ref.rs               -- DirectiveRef
      graphql_type.rs                -- GraphQLType (6 variants)
      graphql_type_kind.rs           -- GraphQLTypeKind (11 variants)
      scalar_kind.rs                 -- ScalarKind
      type_annotation.rs             -- TypeAnnotation (Named | List) + subtype logic
      named_type_annotation.rs       -- NamedTypeAnnotation
      list_type_annotation.rs        -- ListTypeAnnotation
      deprecation_state.rs           -- DeprecationState
      object_type.rs                 -- ObjectType
      interface_type.rs              -- InterfaceType
      union_type.rs                  -- UnionType
      enum_type.rs                   -- EnumType
      enum_value.rs                  -- EnumValue
      scalar_type.rs                 -- ScalarType (with ScalarKind)
      input_object_type.rs           -- InputObjectType
      input_field.rs                 -- InputField
      field_definition.rs            -- FieldDefinition (on Object/Interface)
      parameter_definition.rs        -- ParameterDefinition (field/directive args)
      directive_definition.rs        -- DirectiveDefinition (unified struct)
      directive_definition_kind.rs   -- DirectiveDefinitionKind
      directive_location_kind.rs     -- DirectiveLocationKind (re-export/adapt from parser)
      tests/

    // ---- Type builders (public) ----
    type_builders/
      mod.rs
      ast_helpers.rs                 -- pub(crate) shared AST->owned conversion helpers
      object_type_builder.rs         -- ObjectTypeBuilder (with from_ast())
      interface_type_builder.rs      -- InterfaceTypeBuilder (with from_ast())
      union_type_builder.rs          -- UnionTypeBuilder (with from_ast())
      enum_type_builder.rs           -- EnumTypeBuilder (with from_ast())
      scalar_type_builder.rs         -- ScalarTypeBuilder (with from_ast())
      input_object_type_builder.rs   -- InputObjectTypeBuilder (with from_ast())
      directive_builder.rs           -- DirectiveBuilder (with from_ast())
      field_def_builder.rs           -- FieldDefBuilder (builder-stage field data)
      input_field_def_builder.rs     -- InputFieldDefBuilder
      parameter_def_builder.rs       -- ParameterDefBuilder
      enum_value_def_builder.rs      -- EnumValueDefBuilder
      tests/

    // ---- Schema ----
    schema/
      mod.rs
      schema.rs                      -- Schema + typed query API
      schema_builder.rs              -- SchemaBuilder
      schema_errors.rs               -- SchemaErrors newtype
      schema_build_error.rs          -- SchemaBuildError enum
      type_validation_error.rs       -- TypeValidationError enum
      _macro_runtime.rs
      tests/

    // ---- Validators (private) ----
    validators/
      mod.rs
      object_or_interface_validator.rs
      union_validator.rs
      input_object_validator.rs
      directive_validator.rs
      type_ref_validator.rs

    // ---- Operations ----
    operation/
      mod.rs
      operation.rs                   -- Operation (single type w/ OperationKind)
      operation_kind.rs              -- OperationKind
      operation_builder.rs           -- OperationBuilder
      operation_build_error.rs       -- OperationBuildError
      variable.rs                    -- Variable
      selection_set.rs               -- SelectionSet + iterators
      selection_set_builder.rs       -- SelectionSetBuilder
      selection_set_build_error.rs   -- SelectionSetBuildError
      selection.rs                   -- Selection enum
      field_selection.rs             -- FieldSelection (pre-resolved metadata)
      fragment_spread.rs             -- FragmentSpread
      inline_fragment.rs             -- InlineFragment
      fragment.rs                    -- Fragment
      fragment_builder.rs            -- FragmentBuilder
      fragment_build_error.rs        -- FragmentBuildError
      fragment_registry.rs           -- FragmentRegistry
      fragment_registry_builder.rs   -- FragmentRegistryBuilder
      fragment_registry_build_error.rs
      executable_document.rs         -- ExecutableDocument
      executable_document_builder.rs -- ExecutableDocumentBuilder
      executable_document_build_error.rs
      tests/
```

---

## Commit Strategy

Organized for human review — each commit is independently reviewable:

1. **Crate scaffold** — `Cargo.toml`, `lib.rs`, workspace membership
2. **Name newtypes** — `names/` module with all 6 name types + private trait
3. **Span + SchemaSourceMap** — foundational location types
4. **Value + DirectiveAnnotation** — shared value/annotation types
5. **Utilities** — `ReadOnlyMap`, `file_reader`
6. **Type refs + annotations** — `TypeRef`, `DirectiveRef`, `TypeAnnotation` + subtype logic
7. **Scalar + Enum types** — `ScalarType`, `ScalarKind`, `EnumType`, `EnumValue`, `DeprecationState`
8. **FieldDefinition + ParameterDefinition** — schema field/param types
9. **HasFieldsAndInterfaces + ObjectType + InterfaceType** — shared trait, `FieldedTypeData`, concrete types
10. **InputObjectType + UnionType** — remaining type definitions
11. **DirectiveDefinition** — `DirectiveDefinition`, `DirectiveDefinitionKind`, `DirectiveLocationKind`
12. **GraphQLType + GraphQLTypeKind** — the main type enum + 11-variant kind
13. **Type builders** — all builder structs + `from_ast()` methods + `ast_helpers`
14. **Schema errors** — `SchemaBuildError`, `TypeValidationError`, `SchemaErrors`
15. **SchemaBuilder core** — registration, load_str/load_file/load_parse_result, built-in seeding
16. **Validators** — all 5 validators (object/interface, union, input object, directive, type-ref)
17. **SchemaBuilder::build()** — orchestration + Schema struct + typed query API
18. **Schema tests** — comprehensive schema building tests
19. **Operation types** — `Operation`, `SelectionSet`, `FieldSelection`, fragments, etc.
20. **Operation builders** — `OperationBuilder`, `SelectionSetBuilder`, `FragmentRegistryBuilder`
21. **Operation tests** — comprehensive operation tests
22. **Macro runtime + serde** — `_macro_runtime`, bincode round-trip tests

---

## Task Breakdown

### Task 1: Crate Scaffold

**Files:**
- Create: `crates/libgraphql-core-v2/Cargo.toml`
- Create: `crates/libgraphql-core-v2/src/lib.rs`
- Modify: `Cargo.toml` (workspace members)

- [ ] Create `Cargo.toml` with deps: `libgraphql-parser`, `inherent`, `serde` (features=["derive"]), `bincode`, `indexmap` (features=["serde"]), `thiserror`. All using workspace versions.
- [ ] Create stub `lib.rs` with crate-level rustdoc (follow `libgraphql-parser`'s doc style: overview paragraph, usage examples, links to spec).
- [ ] Add `"crates/libgraphql-core-v2"` to workspace members in root `Cargo.toml`.
- [ ] Verify: `cargo check --package libgraphql-core-v2`
- [ ] Commit: `[libgraphql-core-v2] Scaffold new crate`

---

### Task 2: Name Newtypes

**Files:**
- Create: `crates/libgraphql-core-v2/src/names/mod.rs`
- Create: `crates/libgraphql-core-v2/src/names/graphql_name.rs`
- Create: `crates/libgraphql-core-v2/src/names/type_name.rs`
- Create: `crates/libgraphql-core-v2/src/names/field_name.rs`
- Create: `crates/libgraphql-core-v2/src/names/variable_name.rs`
- Create: `crates/libgraphql-core-v2/src/names/directive_name.rs`
- Create: `crates/libgraphql-core-v2/src/names/enum_value_name.rs`
- Create: `crates/libgraphql-core-v2/src/names/fragment_name.rs`

Each name type is defined explicitly (no `macro_rules!`). Common behavior is constrained by a private supertrait + `#[inherent]` delegation. Each type lives in its own file under `names/`.

**`graphql_name.rs`** — private trait with upper bounds enforcing consistency:
```rust
use std::borrow::Borrow;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;

/// Constrains all GraphQL name newtypes to a consistent set of
/// capabilities. Every name type must be cloneable, hashable,
/// orderable, serializable, displayable, and convertible from
/// common string types.
///
/// This trait is `pub(crate)` — it enforces consistency at
/// definition time but is not part of the public API. Public
/// consumers interact with each name type's inherent methods
/// (delegated via `#[inherent]`).
pub(crate) trait GraphQLName:
    Clone
    + Debug
    + Display
    + Eq
    + Hash
    + Ord
    + AsRef<str>
    + Borrow<str>
    + From<String>
    + serde::Serialize
    + for<'de> serde::Deserialize<'de>
{
    fn new(s: impl Into<String>) -> Self;
    fn as_str(&self) -> &str;
}
```

**Each name file** (e.g. `type_name.rs`):
```rust
use crate::names::graphql_name::GraphQLName;
use inherent::inherent;
use std::borrow::Borrow;

/// A GraphQL [type name](https://spec.graphql.org/September2025/#sec-Names)
/// (e.g. `User`, `String`, `Query`).
///
/// Type names identify schema-defined types: object types, interfaces,
/// unions, enums, scalars, and input objects. Using a dedicated newtype
/// prevents accidental mixing with other name domains like
/// [`FieldName`](crate::names::FieldName) or
/// [`VariableName`](crate::names::VariableName).
///
/// # Construction
///
/// ```rust
/// use libgraphql_core_v2::names::TypeName;
///
/// let name = TypeName::new("User");
/// assert_eq!(name.as_str(), "User");
///
/// let from_str: TypeName = "Query".into();
/// let from_string: TypeName = String::from("Query").into();
/// assert_eq!(from_str, from_string);
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct TypeName(String);

impl GraphQLName for TypeName {
    fn new(s: impl Into<String>) -> Self { Self(s.into()) }
    fn as_str(&self) -> &str { &self.0 }
}

#[inherent]
impl GraphQLName for TypeName {
    pub fn new(s: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

impl AsRef<str> for TypeName {
    fn as_ref(&self) -> &str { &self.0 }
}

impl Borrow<str> for TypeName {
    fn borrow(&self) -> &str { &self.0 }
}

impl std::fmt::Display for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for TypeName {
    fn from(s: &str) -> Self { Self(s.to_owned()) }
}

impl From<String> for TypeName {
    fn from(s: String) -> Self { Self(s) }
}
```

The remaining 5 name types (`FieldName`, `VariableName`, `DirectiveName`, `EnumValueName`, `FragmentName`) follow this identical pattern, differing only in their struct name and rustdoc description.

**Tests** for each name type (in a `tests` submodule adjacent to the `names/` module):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Verifies TypeName basic construction and accessor.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn type_name_construction() {
        let name = TypeName::new("User");
        assert_eq!(name.as_str(), "User");
    }

    // Verifies Display formats as the inner string.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn type_name_display() {
        let name = TypeName::new("Query");
        assert_eq!(format!("{name}"), "Query");
    }

    // Verifies serde round-trip via bincode.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn type_name_serde_roundtrip() {
        let name = TypeName::new("User");
        let bytes = bincode::serde::encode_to_vec(
            &name,
            bincode::config::standard(),
        ).unwrap();
        let (deserialized, _): (TypeName, _) =
            bincode::serde::decode_from_slice(
                &bytes,
                bincode::config::standard(),
            ).unwrap();
        assert_eq!(name, deserialized);
    }

    // Verifies From<&str> and From<String> produce equal values.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn type_name_from_conversions() {
        let from_str: TypeName = "Query".into();
        let from_string: TypeName = String::from("Query").into();
        assert_eq!(from_str, from_string);
    }

    // Verifies Borrow<str> enables HashMap lookups with &str keys.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn type_name_borrow_lookup() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(TypeName::new("User"), 42);
        assert_eq!(map.get("User"), Some(&42));
    }
}
```

- [ ] Define private `GraphQLName` trait in `graphql_name.rs`
- [ ] Implement all 6 name types, each in its own file, with full rustdocs
- [ ] Write tests (construction, display, serde round-trip, from-conversions)
- [ ] Wire up `names/mod.rs` with re-exports
- [ ] Add `pub mod names;` to `lib.rs`
- [ ] Verify: `cargo test --package libgraphql-core-v2 -- names`
- [ ] Commit: `[libgraphql-core-v2] Add name newtypes (TypeName, FieldName, etc.)`

---

### Task 3: Span + SchemaSourceMap

**Files:**
- Create: `crates/libgraphql-core-v2/src/span.rs`
- Create: `crates/libgraphql-core-v2/src/schema_source_map.rs`

**`span.rs`:**
```rust
use libgraphql_parser::ByteSpan;

/// Identifies a source map within a
/// [`Schema`](crate::schema::Schema)'s collection of source maps.
///
/// Index `0` ([`BUILTIN_SOURCE_MAP_ID`]) is reserved for built-in
/// definitions that have no user-authored source.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct SourceMapId(pub(crate) u16);

/// The source map ID for built-in types and directives (`Boolean`,
/// `String`, `@skip`, `@include`, etc.).
pub const BUILTIN_SOURCE_MAP_ID: SourceMapId = SourceMapId(0);

/// A compact source location: a byte-offset range paired with the
/// [`SourceMapId`] of the source it belongs to.
///
/// At 12 bytes and `Copy`, `Span` is designed to be stored on every
/// AST-derived semantic node without significant memory overhead.
/// Line/column resolution is deferred until needed, via the
/// corresponding [`SchemaSourceMap`](crate::SchemaSourceMap) stored
/// on the [`Schema`](crate::schema::Schema).
///
/// See [`ByteSpan`](libgraphql_parser::ByteSpan) for the
/// underlying byte-offset representation.
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

**`schema_source_map.rs`:**
```rust
use std::path::PathBuf;

/// Owned, serializable source map data for resolving
/// [`ByteSpan`](libgraphql_parser::ByteSpan)s to line/column
/// positions within a [`Schema`](crate::schema::Schema).
///
/// # Why not use `libgraphql_parser::SourceMap` directly?
///
/// The parser's [`SourceMap<'src>`](libgraphql_parser::SourceMap)
/// borrows source text via `'src` and does not implement
/// `serde::Serialize`. A [`Schema`] must be `'static` and
/// serde-serializable (the `libgraphql-macros` crate embeds
/// schemas as binary at compile time). `SchemaSourceMap` stores
/// just the line-start byte offsets and optional file path —
/// the minimum data needed for deferred line/column resolution.
///
/// One `SchemaSourceMap` exists per source file or string loaded
/// into a [`SchemaBuilder`](crate::schema::SchemaBuilder).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct SchemaSourceMap {
    pub(crate) file_path: Option<PathBuf>,
    pub(crate) line_starts: Vec<u32>,
}

impl SchemaSourceMap {
    /// Creates a `SchemaSourceMap` by scanning `source` for line
    /// terminators to compute line-start byte offsets.
    ///
    /// This performs the same O(n) line-start scan that the parser's
    /// `SourceMap` does internally, but the result is fully owned and
    /// serializable.
    pub fn from_source(
        source: &str,
        file_path: Option<PathBuf>,
    ) -> Self {
        let bytes = source.as_bytes();
        let mut line_starts = vec![0u32];
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'\n' => {
                    line_starts.push((i + 1) as u32);
                    i += 1;
                },
                b'\r' => {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                        line_starts.push((i + 2) as u32);
                        i += 2;
                    } else {
                        line_starts.push((i + 1) as u32);
                        i += 1;
                    }
                },
                _ => i += 1,
            }
        }
        Self { file_path, line_starts }
    }

    /// Creates a synthetic source map for built-in definitions.
    pub fn builtin() -> Self {
        Self { file_path: None, line_starts: vec![0] }
    }

    pub fn file_path(&self) -> Option<&std::path::Path> {
        self.file_path.as_deref()
    }

    /// Resolves a byte offset to a 0-based `(line, column)` pair.
    pub fn resolve_offset(&self, byte_offset: u32) -> (u32, u32) {
        let line = self.line_starts
            .partition_point(|&start| start <= byte_offset)
            .saturating_sub(1);
        let col = byte_offset - self.line_starts[line];
        (line as u32, col)
    }
}
```

**Tests:**
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

    // Verifies builtin span has source_map_id 0 and empty byte span.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn builtin_span() {
        let span = Span::builtin();
        assert_eq!(span.source_map_id, BUILTIN_SOURCE_MAP_ID);
        assert!(span.byte_span.is_empty());
    }
}
```

- [ ] Implement `Span`, `SourceMapId`, `BUILTIN_SOURCE_MAP_ID` with rustdocs
- [ ] Implement `SchemaSourceMap` with `from_parser_source_map()` and `resolve_offset()`
- [ ] Write tests
- [ ] Verify: `cargo test --package libgraphql-core-v2 -- span`
- [ ] Commit: `[libgraphql-core-v2] Add Span, SourceMapId, SchemaSourceMap`

---

### Task 4: Value + DirectiveAnnotation

**Files:**
- Create: `crates/libgraphql-core-v2/src/value.rs`
- Create: `crates/libgraphql-core-v2/src/directive_annotation.rs`

**`value.rs`:**
```rust
use crate::names::EnumValueName;
use crate::names::VariableName;
use indexmap::IndexMap;

/// A GraphQL input value.
///
/// Represents all possible value literals as defined in the
/// [Input Values](https://spec.graphql.org/September2025/#sec-Input-Values)
/// section of the spec. Used for argument values, default values,
/// and variable values.
///
/// Variable references use [`VariableName`] and enum values use
/// [`EnumValueName`] — preventing accidental mixing with other
/// name domains.
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

**`directive_annotation.rs`:**
```rust
use crate::names::DirectiveName;
use crate::span::Span;
use crate::value::Value;
use indexmap::IndexMap;

/// An applied directive annotation on a definition, field, or
/// argument (e.g. `@deprecated(reason: "Use newField")`).
///
/// This represents a *usage* of a directive — not its
/// *definition*. For the schema-level directive definition, see
/// [`DirectiveDefinition`](crate::types::DirectiveDefinition).
///
/// See
/// [Directives](https://spec.graphql.org/September2025/#sec-Language.Directives)
/// in the spec.
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

    pub fn name(&self) -> &DirectiveName { &self.name }

    pub fn span(&self) -> Span { self.span }
}
```

**Tests:** Value variant construction, DirectiveAnnotation accessors.

- [ ] Implement `Value` enum with rustdocs
- [ ] Implement `DirectiveAnnotation` with rustdocs
- [ ] Write tests
- [ ] Commit: `[libgraphql-core-v2] Add Value enum and DirectiveAnnotation`

---

### Task 5: Utilities

**Files:**
- Create: `crates/libgraphql-core-v2/src/readonly_map.rs`
- Create: `crates/libgraphql-core-v2/src/file_reader.rs`

- [ ] Port `ReadOnlyMap` from v1, adapting for `IndexMap`. Add rustdocs.
- [ ] Port `file_reader` from v1. Add rustdocs.
- [ ] Verify: `cargo check --package libgraphql-core-v2`
- [ ] Commit: `[libgraphql-core-v2] Add ReadOnlyMap and file_reader`

---

### Task 6: Type References + TypeAnnotation

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/mod.rs`
- Create: `crates/libgraphql-core-v2/src/types/type_ref.rs`
- Create: `crates/libgraphql-core-v2/src/types/directive_ref.rs`
- Create: `crates/libgraphql-core-v2/src/types/type_annotation.rs`
- Create: `crates/libgraphql-core-v2/src/types/named_type_annotation.rs`
- Create: `crates/libgraphql-core-v2/src/types/list_type_annotation.rs`

**`type_ref.rs`:** `TypeRef(TypeName)` with `name()` accessor. Rustdoc explaining resolution via `Schema::get_type()`.

**`directive_ref.rs`:** `DirectiveRef(DirectiveName)` with `name()` accessor.

**`type_annotation.rs`:** `TypeAnnotation` enum (Named | List) + `is_equivalent_to()`, `is_subtype_of()`, `innermost_named()`, `innermost_type_name()`, `nullable()`, `Display`. Full rustdocs with spec links to [Type References](https://spec.graphql.org/September2025/#sec-Type-References) and [IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation()).

**Tests:** Equivalence, non-equivalence (nullability, structure), Display formatting, innermost_type_name on nested lists.

- [ ] Implement `TypeRef`, `DirectiveRef` (each in own file)
- [ ] Implement `TypeAnnotation`, `NamedTypeAnnotation`, `ListTypeAnnotation` (each in own file)
- [ ] Port subtype/equivalence logic from v1 `type_annotation.rs`
- [ ] Write thorough tests for equivalence + subtype logic
- [ ] Wire up `types/mod.rs` with re-exports
- [ ] Commit: `[libgraphql-core-v2] Add TypeRef, DirectiveRef, TypeAnnotation`

---

### Task 7: ScalarType, ScalarKind, EnumType, EnumValue, DeprecationState

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/scalar_kind.rs`
- Create: `crates/libgraphql-core-v2/src/types/scalar_type.rs`
- Create: `crates/libgraphql-core-v2/src/types/enum_type.rs`
- Create: `crates/libgraphql-core-v2/src/types/enum_value.rs`
- Create: `crates/libgraphql-core-v2/src/types/deprecation_state.rs`

**`scalar_kind.rs`:**
```rust
/// Identifies whether a scalar type is one of the built-in GraphQL
/// scalars or a custom (user-defined) scalar.
///
/// See [Scalars](https://spec.graphql.org/September2025/#sec-Scalars).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum ScalarKind {
    Boolean,
    Custom,
    Float,
    ID,
    Int,
    String,
}
```

**`scalar_type.rs`:** `ScalarType { kind: ScalarKind, name, description, directives, span }` with `is_builtin()` method.

**Tests:** ScalarKind matching, is_builtin(), DeprecationState from directives.

- [ ] Implement all 5 types, each in own file, with rustdocs
- [ ] Write tests
- [ ] Commit: `[libgraphql-core-v2] Add ScalarType, ScalarKind, EnumType, EnumValue, DeprecationState`

---

### Task 8: FieldDefinition + ParameterDefinition

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/field_definition.rs`
- Create: `crates/libgraphql-core-v2/src/types/parameter_definition.rs`

**`field_definition.rs`:** `FieldDefinition { name: FieldName, type_annotation: TypeAnnotation, parameters: IndexMap<FieldName, ParameterDefinition>, description, directives, parent_type_name: TypeName, span }`. Includes `return_type_name() -> &TypeName` convenience (delegates to `type_annotation.innermost_type_name()`). Rustdoc clearly states this is a *schema* field definition, not an operation field selection.

**`parameter_definition.rs`:** `ParameterDefinition { name: FieldName, type_annotation: TypeAnnotation, default_value: Option<Value>, description, span }`.

- [ ] Implement both types with full rustdocs
- [ ] Write tests for accessor methods
- [ ] Commit: `[libgraphql-core-v2] Add FieldDefinition and ParameterDefinition`

---

### Task 9: HasFieldsAndInterfaces + ObjectType + InterfaceType

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/has_fields_and_interfaces.rs`
- Create: `crates/libgraphql-core-v2/src/types/fielded_type_data.rs`
- Create: `crates/libgraphql-core-v2/src/types/object_type.rs`
- Create: `crates/libgraphql-core-v2/src/types/interface_type.rs`

**`has_fields_and_interfaces.rs`:** The trait, with rustdoc explaining it enables generic validation and querying over Object and Interface types.

**`fielded_type_data.rs`:** `pub(crate) struct FieldedTypeData { name, description, directives, fields: IndexMap<FieldName, FieldDefinition>, interfaces: Vec<TypeName>, span }`. Shared data struct.

**`object_type.rs`:** `pub struct ObjectType(pub(crate) FieldedTypeData)` implementing `HasFieldsAndInterfaces`. Rustdoc links to [Objects](https://spec.graphql.org/September2025/#sec-Objects).

**`interface_type.rs`:** Same pattern, links to [Interfaces](https://spec.graphql.org/September2025/#sec-Interfaces).

- [ ] Implement trait, shared data struct, and both types
- [ ] Write tests verifying trait method delegation
- [ ] Commit: `[libgraphql-core-v2] Add HasFieldsAndInterfaces, ObjectType, InterfaceType`

---

### Task 10: InputObjectType, InputField, UnionType

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/input_object_type.rs`
- Create: `crates/libgraphql-core-v2/src/types/input_field.rs`
- Create: `crates/libgraphql-core-v2/src/types/union_type.rs`

Standard implementations with full rustdocs and spec links.

- [ ] Implement all 3 types in own files
- [ ] Write basic accessor tests
- [ ] Commit: `[libgraphql-core-v2] Add InputObjectType, InputField, UnionType`

---

### Task 11: DirectiveDefinition + DirectiveDefinitionKind + DirectiveLocationKind

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/directive_definition.rs`
- Create: `crates/libgraphql-core-v2/src/types/directive_definition_kind.rs`
- Create: `crates/libgraphql-core-v2/src/types/directive_location_kind.rs`

**`directive_definition_kind.rs`:** Enum with `Custom`, `Deprecated`, `Include`, `Skip`, `SpecifiedBy` + `is_builtin()` method.

**`directive_location_kind.rs`:** Can re-export/adapt `libgraphql_parser::ast::DirectiveLocationKind` or define our own (owned, serializable) version. 19 variants matching the spec.

**`directive_definition.rs`:** Unified struct with all data fields + kind discriminator. Rustdoc explaining this replaces v1's asymmetric enum approach. Links to [Type-System Directives](https://spec.graphql.org/September2025/#sec-Type-System.Directives).

- [ ] Implement all 3 types
- [ ] Write tests (kind matching, is_builtin, accessor methods)
- [ ] Commit: `[libgraphql-core-v2] Add DirectiveDefinition, DirectiveDefinitionKind, DirectiveLocationKind`

---

### Task 12: GraphQLType + GraphQLTypeKind

**Files:**
- Create: `crates/libgraphql-core-v2/src/types/graphql_type.rs`
- Create: `crates/libgraphql-core-v2/src/types/graphql_type_kind.rs`

**`graphql_type.rs`:** 6-variant enum with methods: `name()`, `def_location()`, `description()`, `is_input_type()`, `is_output_type()`, `is_builtin()`, `is_composite_type()`, `is_leaf_type()`, `requires_selection_set()`, `type_kind()`, typed downcasts (`as_object()`, `as_interface()`, etc.). Full rustdocs with spec links.

**`graphql_type_kind.rs`:** 11-variant enum. `GraphQLType::type_kind()` maps `Scalar(s)` -> `s.kind.into()` (so `ScalarKind::Boolean` -> `GraphQLTypeKind::Boolean`, `ScalarKind::Custom` -> `GraphQLTypeKind::Scalar`).

**Tests:** `is_input_type`/`is_output_type` classification, `type_kind()` mapping, typed downcasts.

- [ ] Implement `GraphQLType` with all methods and rustdocs
- [ ] Implement `GraphQLTypeKind` with `From<ScalarKind>` conversion
- [ ] Finalize all `types/mod.rs` re-exports
- [ ] Write thorough tests for input/output classification and kind mapping
- [ ] Commit: `[libgraphql-core-v2] Add GraphQLType (6 variants) and GraphQLTypeKind (11 variants)`

---

### Task 13: Type Builders

**Files:** All files under `type_builders/`

Each builder follows the same pattern:
- `new(name, span)` constructor
- `set_*()` / `add_*()` mutators returning `&mut Self`
- `from_ast(ast_node, source_map_id)` class method
- All data `pub(crate)` for SchemaBuilder access

**`ast_helpers.rs`:** `pub(crate)` functions shared by multiple builders:
- `type_annotation_from_ast(&parser::ast::TypeAnnotation, SourceMapId) -> TypeAnnotation`
- `value_from_ast(&parser::ast::Value) -> Value`
- `directive_annotation_from_ast(&parser::ast::DirectiveAnnotation, SourceMapId) -> DirectiveAnnotation`
- `span_from_ast(ByteSpan, SourceMapId) -> Span`

Builder-stage data types (`FieldDefBuilder`, `ParameterDefBuilder`, etc.) each in own file — these are intermediate representations used during building, distinct from the validated types.

**Tests:** `from_ast()` round-trip tests parsing GraphQL strings and verifying builder fields.

- [ ] Implement `ast_helpers.rs` with shared conversion functions
- [ ] Implement all builder-stage data types (FieldDefBuilder, etc.)
- [ ] Implement all 7 type builders with `from_ast()` methods
- [ ] Write `from_ast()` round-trip tests
- [ ] Commit: `[libgraphql-core-v2] Add type builders with from_ast() support`

---

### Task 14: Schema Errors

**Files:**
- Create: `crates/libgraphql-core-v2/src/schema/mod.rs`
- Create: `crates/libgraphql-core-v2/src/schema/schema_errors.rs`
- Create: `crates/libgraphql-core-v2/src/schema/schema_build_error.rs`
- Create: `crates/libgraphql-core-v2/src/schema/type_validation_error.rs`

Port all error variants from v1, replacing `SourceLocation` with `Span`. Add new variants for previously-missing validations (see Validation Checklist below). `SchemaErrors` implements `Error + Display + IntoIterator`.

- [ ] Implement `SchemaBuildError` (~25 variants, covering all known validations)
- [ ] Implement `TypeValidationError` (~15 variants, fixing v1's incomplete input-type checking)
- [ ] Implement `SchemaErrors` newtype
- [ ] Commit: `[libgraphql-core-v2] Add SchemaBuildError, TypeValidationError, SchemaErrors`

---

### Task 15: SchemaBuilder Core

**Files:**
- Create: `crates/libgraphql-core-v2/src/schema/schema_builder.rs`

SchemaBuilder with:
- `new()` — seeds built-in scalars (Boolean/Float/ID/Int/String as `ScalarType` with appropriate `ScalarKind`) + built-in directives (@skip/@include/@deprecated/@specifiedBy as `DirectiveDefinition` with appropriate `DirectiveDefinitionKind`, locations, parameters)
- `register_type(builder)` — validates name, checks duplicates, inserts
- `register_directive(builder)` — validates name, checks duplicates, inserts
- `load_parse_result(&ParseResult)` — iterates definitions, creates builders, registers
- `load_str(source)` — parses + delegates to load_parse_result
- `load_file(path)` — reads file + delegates to load_str
- Pending type extensions stored for deferred application

**Tests:** Basic registration, duplicate rejection, load_str, built-in presence.

- [ ] Implement SchemaBuilder with all methods
- [ ] Write registration + loading tests
- [ ] Commit: `[libgraphql-core-v2] Add SchemaBuilder with registration and loading`

---

### Task 16: Validators

**Files:** All files under `validators/`

Port all validators from v1, adapting to v2 types. Fix v1 bugs identified in the spec audit.

**`object_or_interface_validator.rs`:** Generic over `T: HasFieldsAndInterfaces`. Validates interface implementation contracts per [IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation()).

**`union_validator.rs`:** Members must be Object types, must exist, union must have >=1 member.

**`input_object_validator.rs`:** **Fix v1 bug:** Use `!is_input_type()` instead of `as_object().is_some()`. Circular non-nullable ref detection.

**`directive_validator.rs`:** **NEW** (entirely absent in v1): directive argument types must be input types, argument names must not start with `__`, argument names unique.

**`type_ref_validator.rs`:** All type annotations resolve to defined types, output fields use output types, params use input types.

**Tests:** Comprehensive tests for each validator covering valid and invalid cases.

- [ ] Port + fix `object_or_interface_validator.rs`
- [ ] Port + fix `union_validator.rs` (add empty union check)
- [ ] Port + fix `input_object_validator.rs` (fix input-type check)
- [ ] Implement new `directive_validator.rs`
- [ ] Implement `type_ref_validator.rs`
- [ ] Write comprehensive validator tests
- [ ] Commit: `[libgraphql-core-v2] Add validators (object/interface, union, input, directive, type-ref)`

---

### Task 17: Schema Struct + SchemaBuilder::build()

**Files:**
- Create: `crates/libgraphql-core-v2/src/schema/schema.rs`
- Modify: `crates/libgraphql-core-v2/src/schema/schema_builder.rs` (add build())

**`schema.rs`:** `Schema` with typed query API:
- `get_type(&TypeName)`, `get_directive(&DirectiveName)` — generic lookups
- `object_type(&TypeName)`, `interface_type(&TypeName)`, etc. — typed lookups
- `object_types()`, `interface_types()`, etc. — typed iterators
- `types_implementing(&TypeName)` — relationship query
- `query_type()`, `mutation_type()`, `subscription_type()` — root operation types

**`build()` orchestration:**
1. Apply pending type extensions
2. Validate root operation types exist and are Object types
3. Run all validators (collect all errors)
4. Return `Schema` or `SchemaErrors`

**Tests:** Full-pipeline tests: parse string -> load -> build -> query. Both valid and invalid schemas.

- [ ] Implement `Schema` with typed query API and full rustdocs
- [ ] Implement `SchemaBuilder::build()` orchestrating all validators
- [ ] Write end-to-end schema building tests (valid + invalid)
- [ ] Commit: `[libgraphql-core-v2] Add Schema struct and SchemaBuilder::build()`

---

### Task 18: Schema Test Suite

**Files:** Under `schema/tests/`

Port and expand v1 tests. Use v1's `.graphql` fixture files where applicable.

- [ ] Port v1 schema builder tests (valid schemas: simple, SWAPI, GitHub, etc.)
- [ ] Port v1 invalid schema tests (duplicates, __-prefix, enum no values, etc.)
- [ ] Add new tests for previously-missing validations (root types must be Object, empty types, enum value names, directive validation)
- [ ] Commit: `[libgraphql-core-v2] Add comprehensive schema test suite`

---

### Task 19: Operation Types

**Files:** All type files under `operation/`

**`operation.rs`:** Single `Operation` struct with `kind: OperationKind`. Methods that need schema context take `&Schema`. `root_type_name(&Schema) -> &TypeName`.

**`field_selection.rs`:** Pre-resolved metadata: `parent_type_name: TypeName`, `field_return_type_name: TypeName`, `requires_selection_set: bool`. Full `schema_field(&self, schema: &Schema) -> Option<&FieldDefinition>` accessor.

**`selection_set.rs`:** `SelectionSet` with `selections()`, `field_selections()` iterator.

Other types follow v1's patterns adapted for v2 (no lifetime params, `Span` instead of `SourceLocation`).

- [ ] Implement all operation types (Operation, Variable, SelectionSet, Selection, FieldSelection, FragmentSpread, InlineFragment, Fragment, FragmentRegistry, ExecutableDocument)
- [ ] Write basic construction/accessor tests
- [ ] Commit: `[libgraphql-core-v2] Add operation types`

---

### Task 20: Operation Builders

**Files:** All builder files under `operation/`

**`selection_set_builder.rs`:** Key validation during building:
- Fields exist on parent type
- `__typename` available on composite types
- Leaf fields must not have sub-selections
- Composite fields must have sub-selections
- Arguments correspond to field definition
- Required arguments provided
- Pre-resolves field metadata

**`fragment_registry_builder.rs`:** Cycle detection (DFS with phase normalization), reference validation.

**`operation_builder.rs`:** Validates variables, directives, subscription single root field.

- [ ] Implement all operation builders
- [ ] Implement all operation error types (in own files)
- [ ] Write operation building tests (valid + invalid)
- [ ] Commit: `[libgraphql-core-v2] Add operation builders`

---

### Task 21: Operation Test Suite

- [ ] Port v1 operation tests
- [ ] Add tests for previously-missing validations (subscription root field, leaf/composite, argument validation, variable type validation)
- [ ] Commit: `[libgraphql-core-v2] Add comprehensive operation test suite`

---

### Task 22: Macro Runtime + Serde/Bincode

**Files:**
- Create: `crates/libgraphql-core-v2/src/schema/_macro_runtime.rs`

- [ ] Implement `build_from_macro_serialized()`
- [ ] Write bincode round-trip test (build schema from string -> serialize -> deserialize -> verify equality)
- [ ] Test with realistic schemas
- [ ] Commit: `[libgraphql-core-v2] Add macro runtime and serde/bincode support`

---

## Validation Coverage Checklist

### During `register_type()` (early, structural):
- [ ] Type names must not start with `__`
- [ ] Duplicate type definition rejected
- [ ] Duplicate field names within a type rejected
- [ ] Duplicate enum values rejected
- [ ] Duplicate interface implementation declarations rejected
- [ ] Duplicate union members rejected
- [ ] Enum values must not be named `true`/`false`/`null`
- [ ] Field/param names must not start with `__`
- [ ] Param names unique within a field
- [ ] Directive argument names must not start with `__`
- [ ] Directive argument names unique within a definition

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
- [ ] Input field types must be input types (not Object/Interface/Union)
- [ ] Output field types must be output types
- [ ] Parameter types must be input types
- [ ] Input object circular non-nullable reference detection
- [ ] All type references resolve to defined types
- [ ] Directive argument types must be input types
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

### Deferred (future work):
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

---

## Code Style Reminders

- All lines ≤ 100 columns (per CLAUDE.md)
- One `use` per line, alphabetically sorted, `crate`-rooted (no `super`)
- All `match` arms end with comma
- Enum variants alphabetically sorted
- Opening `{` never on its own line
- No "Step N:" style comments
- Thorough rustdoc on all public items, matching `libgraphql-parser` quality
- Link to September 2025 GraphQL spec where applicable
- Tests include English description + spec link + "Written by Claude Code, reviewed by a human"
