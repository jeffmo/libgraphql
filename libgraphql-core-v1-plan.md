# libgraphql-core-v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

## Important: Source Code Naming Convention

The crate is developed as `libgraphql-core-v1` (Cargo package name) to coexist with the existing `libgraphql-core` during development. However, **all source code (rustdoc, doc examples, comments, error messages) must refer to `libgraphql-core` / `libgraphql_core`** — never `libgraphql-core-v1` / `libgraphql_core_v1`. The plan is to fully replace `/crates/libgraphql-core` with this rewrite once stabilized, bumping to `v1.0.0`. In the end this is the same crate, just developed adjacently until ready to ship.

---

## Execution Protocol

**Before starting each task:**
1. Read the **entirety** of the current state of `libgraphql-core-v1-plan.md` (this file) into the session. The plan is a living document — it may have been updated by prior tasks. Always work from the freshest version.

**After completing each task:**
1. Update this plan file with:
   - Mark completed steps with `[x]`
   - A brief "Completion Notes" block under the task summarizing what was actually done (especially any deviations from the plan)
   - Any adjustments to subsequent tasks that were discovered during implementation (e.g., "Task N will also need to handle X" or "The API shape for Y changed to Z")
2. Commit the plan file update as part of the task's commit (or as a follow-up commit in the same stack)
3. Push a `lgcore_v1_task${TASK_NUM}` branch to GitHub and create a PR with a clear, thorough description
4. Wait for the PR to be reviewed and merged to main
5. After merge: run `sl pull && sl up main` to move to the main commit, then proceed with the next task

This ensures the plan persistently tracks progress and evolving understanding across sessions, and each task is independently reviewed before building on it.

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
    fn description(&self) -> Option<&str>;
    fn directives(&self) -> &[DirectiveAnnotation];
    fn field(&self, name: &str) -> Option<&FieldDefinition>;
    fn fields(&self) -> &IndexMap<FieldName, FieldDefinition>;
    fn interfaces(&self) -> &[Located<TypeName>];
    fn name(&self) -> &TypeName;
    fn span(&self) -> Span;
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

`GraphQLType` methods (`name()`, `span()`, etc.) have 6 arms. Exhaustive matching over all types including built-in scalar identity via `.type_kind()` -> `GraphQLTypeKind` (11 variants).

### AD5. Source Locations: Span = ByteSpan + SourceMapId

```rust
pub struct SourceMapId(u16);
pub struct Span { pub byte_span: ByteSpan, pub source_map_id: SourceMapId }
pub const BUILTIN_SOURCE_MAP_ID: SourceMapId = SourceMapId(0);
```

### AD6. SchemaSourceMap: owned, serializable subset of parser SourceMap

`libgraphql_parser::SourceMap<'src>` borrows source text via `'src` and isn't serializable. `Schema` must be `'static` and serde-serializable (macro crate embeds it as bytes). `SchemaSourceMap` stores just line-start offsets and file path — enough to resolve byte offsets to line/column on demand. Columns are UTF-8 character offsets (not UTF-16 code unit offsets).

```rust
pub struct SchemaSourceMap {
    pub file_path: Option<PathBuf>,
    pub line_starts: Vec<u32>,
}
```

### AD6a. `Located<T>` for name occurrences with source spans

Name newtypes (`TypeName`, etc.) stay as pure string identity types — they work as `IndexMap` keys and carry no location. When a specific *occurrence* of a name needs its source span (e.g., which `implements Foo` reference triggered a validation error), use `Located<T>`:

```rust
pub struct Located<T> {
    pub value: T,
    pub span: Span,
}
```

`Located<T>` deliberately does NOT implement `Eq`/`Hash` — making accidental use as a map key a compile error. Used primarily in:
- `ObjectType.interfaces: Vec<Located<TypeName>>`
- `UnionType.members: Vec<Located<TypeName>>`

Places where the containing struct's span suffices (e.g., `NamedTypeAnnotation` already has its own span) use bare `TypeName`.

### AD6b. Drop TypeRef and DirectiveRef — TypeName/DirectiveName suffice

v0's `NamedRef<TSource, TRefLocation, TResource>` carried both a name and a source location, justifying a distinct wrapper. An earlier draft's planned `TypeRef(TypeName)` would have contained nothing beyond the name itself — it's redundant. Moreover, the GraphQL spec uses "type reference" to mean what we call `TypeAnnotation`, so `TypeRef` would be a confusing name collision. **Use `TypeName` directly everywhere** a reference to a type is needed. Same for `DirectiveName` — no `DirectiveRef` wrapper.

### AD7. SchemaErrors Newtype

`SchemaBuilder::build()` returns `Result<Schema, SchemaErrors>` where `SchemaErrors: Error + Display + IntoIterator<Item = SchemaBuildError>`, enabling `?` propagation.

### AD8. FieldDefinition (not "Field") for schema field definitions

Clear nominal distinction: `FieldDefinition` is a field defined on an Object/Interface type in the schema. `FieldSelection` is a field selected in an operation. Matches `libgraphql-parser`'s naming.

### AD9. Absorption via SchemaBuilder

`schema_builder.absorb_type(builder)` rather than `builder.register(&mut sb)`. "Absorb" conveys that the builder is consumed and its contents are incorporated into the schema being built.

### AD10. Builder from_ast() as methods, shared helpers as private module

`ObjectTypeBuilder::from_ast(&ast_node, source_map_id)` is a method on the builder. Shared conversion helpers (`type_annotation_from_ast()`, `value_from_ast()`, `directive_annotation_from_ast()`) live in a private `ast_helpers.rs` module.

### AD11. Pre-Resolved FieldSelection

`FieldSelection` stores pre-resolved metadata (`parent_type_name`, `field_return_type_name`, `requires_selection_set`) validated at build time, queryable without `&Schema`.

### AD12. Typed Schema Query API

`schema.object_type(&name)`, `schema.interface_types()`, `schema.types_implementing(&name)`, etc. — typed accessors and iterators as thin wrappers with zero storage cost.

### AD13. IndexMap keys must use name newtypes, not bare String

Established during PR #90 review. Anywhere an `IndexMap` key represents a GraphQL name (field names, argument names, object field names in values), use the appropriate name newtype (`FieldName`, `TypeName`, etc.) — never bare `String`. This applies to:
- `DirectiveAnnotation.arguments`: `IndexMap<FieldName, Value>` (not `String`)
- `Value::Object`: `IndexMap<FieldName, Value>` (not `String`)
- `FieldSelection.arguments`: `IndexMap<FieldName, Value>`
- All similar maps in builders and types

**Why:** Prevents cross-domain string confusion and centralizes name handling. The `Borrow<str>` impl on each name newtype still allows `map.get("foo")` lookups with `&str`.

### AD14. Value::Int uses i32, not i64

The GraphQL spec defines `Int` as a signed 32-bit integer. `libgraphql-parser` already parses int values as `i32`. Using `i64` would allow constructing values that are invalid by spec definition. Use `i32` everywhere.

### AD15. `#[inherent]` requires a single impl block, not two

The `#[inherent]` proc macro takes one `impl Trait for Type { ... }` block with full method bodies and generates both the trait impl and the inherent methods. Do **not** write a separate non-`#[inherent]` trait impl — this causes a conflicting-impl error. The plan's original code sketches (Task 2) showed two blocks; the correct pattern is a single `#[inherent]` block.

---

## File Structure

Each type in its own file, `mod.rs` files only for module declarations + re-exports.

```
crates/libgraphql-core-v1/
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
    error_note.rs                    -- ErrorNote, ErrorNoteKind (notes system)
    located.rs                       -- Located<T> wrapper (value + span)
    span.rs                          -- Span, SourceMapId, BUILTIN_SOURCE_MAP_ID
    schema_source_map.rs             -- SchemaSourceMap, LineCol
    value.rs                         -- Value enum
    directive_annotation.rs          -- DirectiveAnnotation (applied instance)

    // ---- Type system (immutable, validated) ----
    types/
      mod.rs
      has_fields_and_interfaces.rs   -- HasFieldsAndInterfaces trait
      fielded_type_data.rs           -- FieldedTypeData (shared Object/Interface data)
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
      directive_location_kind.rs     -- re-exports libgraphql_parser::ast::DirectiveLocationKind
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
      object_or_interface_type_validator.rs
      union_type_validator.rs
      input_object_type_validator.rs
      directive_definition_validator.rs
      type_reference_validator.rs

    // ---- Operations ----
    operation/
      mod.rs
      operation.rs                   -- Operation (single type w/ OperationKind)
      operation_kind.rs              -- OperationKind
      operation_builder.rs           -- OperationBuilder (generic)
      query_operation_builder.rs     -- QueryOperationBuilder (newtype)
      mutation_operation_builder.rs  -- MutationOperationBuilder (newtype)
      subscription_operation_builder.rs -- SubscriptionOperationBuilder (newtype)
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
3. **Span + Located + SchemaSourceMap** — foundational location types
4. **Value + DirectiveAnnotation** — shared value/annotation types
5. **TypeAnnotation** — `TypeAnnotation` + subtype/equivalence logic
6. **Scalar + Enum types** — `ScalarType`, `ScalarKind`, `EnumType`, `EnumValue`, `DeprecationState`
7. **FieldDefinition + ParameterDefinition** — schema field/param types
8. **HasFieldsAndInterfaces + ObjectType + InterfaceType** — shared trait, `FieldedTypeData`, concrete types
9. **InputObjectType + UnionType** — remaining type definitions
10. **DirectiveDefinition** — `DirectiveDefinition`, `DirectiveDefinitionKind`, `DirectiveLocationKind`
11. **GraphQLType + GraphQLTypeKind** — the main type enum + 11-variant kind
12. **Type builders** — all builder structs + `from_ast()` methods + `ast_helpers`
13. **Schema errors** — `SchemaBuildError`, `TypeValidationError`, `SchemaErrors`
14. **SchemaBuilder core** — registration, load_str/load_parse_result, built-in seeding
15. **Validators** — all 5 validators (object/interface, union, input object, directive, type-ref)
16. **SchemaBuilder::build()** — orchestration + Schema struct + typed query API
17. **Schema tests** — comprehensive schema building tests
18. **Operation types** — `Operation`, `SelectionSet`, `FieldSelection`, fragments, etc.
19. **Operation builders** — `OperationBuilder`, `SelectionSetBuilder`, `FragmentRegistryBuilder`
20. **Operation tests** — comprehensive operation tests
21. **Macro runtime + serde** — `_macro_runtime`, bincode round-trip tests

---

## Task Breakdown

### Task 1: Crate Scaffold

**Files:**
- Create: `crates/libgraphql-core-v1/Cargo.toml`
- Create: `crates/libgraphql-core-v1/src/lib.rs`
- Modify: `Cargo.toml` (workspace members)

- [x] Create `Cargo.toml` with deps: `libgraphql-parser`, `inherent`, `serde` (features=["derive"]), `bincode`, `indexmap` (features=["serde"]), `thiserror`. All using workspace versions.
- [x] Create stub `lib.rs` with crate-level rustdoc (follow `libgraphql-parser`'s doc style: overview paragraph, usage examples, links to spec).
- [x] Add `"crates/libgraphql-core-v1"` to workspace members in root `Cargo.toml`.
- [x] Verify: `cargo check --package libgraphql-core-v1`
- [x] Commit: `[libgraphql-core-v1] Scaffold new crate`

**Completion Notes:** All source code references use `libgraphql_core` (not `libgraphql_core_v1`) per the naming convention — this crate will replace the existing `libgraphql-core` at v1.0.0.

---

### Task 2: Name Newtypes

**Files:**
- Create: `crates/libgraphql-core-v1/src/names/mod.rs`
- Create: `crates/libgraphql-core-v1/src/names/graphql_name.rs`
- Create: `crates/libgraphql-core-v1/src/names/type_name.rs`
- Create: `crates/libgraphql-core-v1/src/names/field_name.rs`
- Create: `crates/libgraphql-core-v1/src/names/variable_name.rs`
- Create: `crates/libgraphql-core-v1/src/names/directive_name.rs`
- Create: `crates/libgraphql-core-v1/src/names/enum_value_name.rs`
- Create: `crates/libgraphql-core-v1/src/names/fragment_name.rs`

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

- [x] Define private `GraphQLName` trait in `graphql_name.rs`
- [x] Implement all 6 name types, each in its own file, with full rustdocs
- [x] Write tests (construction, display, serde round-trip, from-conversions)
- [x] Wire up `names/mod.rs` with re-exports
- [x] Add `pub mod names;` to `lib.rs`
- [x] Verify: `cargo test --package libgraphql-core-v1 -- names`
- [x] Commit: `[libgraphql-core-v1] Add name newtypes (TypeName, FieldName, etc.)`

**Completion Notes:** The plan's code sketch had two separate `impl GraphQLName` blocks (one real, one `#[inherent]`) per name type. `#[inherent]` generates both the trait impl and the inherent methods from a single block, so the duplicate was removed. Doctests use `ignore` since `libgraphql_core` doesn't resolve until the crate is renamed post-stabilization. All future tasks should use a single `#[inherent] impl` block, not the two-block pattern from the plan sketches.

---

### Task 3: Span + Located + SchemaSourceMap

**Files:**
- Create: `crates/libgraphql-core-v1/src/span.rs`
- Create: `crates/libgraphql-core-v1/src/located.rs`
- Create: `crates/libgraphql-core-v1/src/schema_source_map.rs`

**`located.rs`:**
```rust
use crate::span::Span;

/// A value paired with the [`Span`] of its occurrence in source.
///
/// Used for name references that need to trace back to their
/// source location — e.g., each interface name in an `implements`
/// clause, or each member name in a union definition. The inner
/// value provides identity (for lookups), while the span provides
/// location (for error reporting).
///
/// `Located<T>` deliberately does **not** implement `Eq` or
/// `Hash`. Use the inner `.value` for identity comparisons and
/// map lookups.
///
/// # Example
///
/// ```rust
/// # use libgraphql_core_v2::Located;
/// # use libgraphql_core_v2::names::TypeName;
/// # use libgraphql_core_v2::span::Span;
/// let located = Located {
///     value: TypeName::new("Node"),
///     span: Span::builtin(),
/// };
/// assert_eq!(located.value.as_str(), "Node");
/// ```
#[derive(Clone, Debug)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Located<T> {
    pub value: T,
    pub span: Span,
}

impl<T> AsRef<T> for Located<T> {
    fn as_ref(&self) -> &T { &self.value }
}
```

`AsRef<T>` enables passing `&Located<TypeName>` to methods expecting `&TypeName` via `.as_ref()`, e.g. `schema.get_type(located_name.as_ref())`.

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
    #[inline]
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

    /// Resolves a byte offset to a 0-based line/column position.
    ///
    /// Returns both a byte-offset column and a UTF-8 character
    /// column. Computing the UTF-8 column requires the source
    /// text for the line slice (to count characters); if source
    /// text is unavailable, pass `None` and `col_utf8` will
    /// equal `col_linestart_byte_offset` (correct for ASCII).
    pub fn resolve_offset(
        &self,
        byte_offset: u32,
        source: Option<&str>,
    ) -> LineCol {
        let line = self.line_starts
            .partition_point(|&start| start <= byte_offset)
            .saturating_sub(1);
        let line_start = self.line_starts[line];
        let col_byte = byte_offset - line_start;
        let col_utf8 = match source {
            Some(src) => {
                let start = line_start as usize;
                let end = byte_offset as usize;
                if end <= src.len() && start <= end {
                    src[start..end].chars().count() as u32
                } else {
                    col_byte
                }
            },
            None => col_byte,
        };
        LineCol {
            line: line as u32,
            col_linestart_byte_offset: col_byte,
            col_utf8,
        }
    }
}

/// A resolved 0-based line and column position.
///
/// Provides two column representations:
/// - `col_utf8`: UTF-8 character count from line start (consistent
///   with [`SourcePosition::col_utf8()`](libgraphql_parser::SourcePosition::col_utf8))
/// - `col_linestart_byte_offset`: byte offset from line start
///
/// For ASCII-only content (the common case in GraphQL), both
/// values are equal.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LineCol {
    pub line: u32,
    pub col_linestart_byte_offset: u32,
    pub col_utf8: u32,
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

- [x] Implement `Span`, `SourceMapId`, `BUILTIN_SOURCE_MAP_ID` with rustdocs
- [x] Implement `Located<T>` with rustdocs (no Eq/Hash)
- [x] Implement `SchemaSourceMap` with `from_source()` and `resolve_offset()`
- [x] Implement `LineCol` with `col_utf8` and `col_linestart_byte_offset` fields
- [x] Write tests
- [x] Verify: `cargo test --package libgraphql-core-v1 -- span`
- [x] Commit: `[libgraphql-core-v1] Add Span, Located, SourceMapId, SchemaSourceMap`

**Completion Notes:** Added `serde::Deserialize`/`serde::Serialize` derives to `ByteSpan` in `libgraphql-parser` since `Span` contains a `ByteSpan` and needs to be serde-serializable. Tests are in the top-level `tests/` directory (`span_tests.rs`, `located_tests.rs`, `schema_source_map_tests.rs`) — not inline. Re-exports for `Located`, `SchemaSourceMap`, `LineCol`, and `Span` added to `lib.rs`.

---

### Task 4: Value + DirectiveAnnotation

**Files:**
- Create: `crates/libgraphql-core-v1/src/value.rs`
- Create: `crates/libgraphql-core-v1/src/directive_annotation.rs`

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
    Int(i32),
    List(Vec<Value>),
    Null,
    Object(IndexMap<FieldName, Value>),
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
    pub(crate) arguments: IndexMap<FieldName, Value>,
    pub(crate) name: DirectiveName,
    pub(crate) span: Span,
}

impl DirectiveAnnotation {
    pub fn arguments(&self) -> &IndexMap<FieldName, Value> {
        &self.arguments
    }

    #[inline]
    pub fn name(&self) -> &DirectiveName { &self.name }

    #[inline]
    pub fn span(&self) -> Span { self.span }
}
```

**Tests:** Value variant construction, DirectiveAnnotation accessors.

- [x] Implement `Value` enum with rustdocs
- [x] Implement `DirectiveAnnotation` with rustdocs
- [x] Write tests
- [x] Commit: `[libgraphql-core-v1] Add Value enum and DirectiveAnnotation`

**Completion Notes:** Straightforward implementation following the plan. Tests in `tests/value_tests.rs` and `tests/directive_annotation_tests.rs`.

---

### Task 5: TypeAnnotation

**Files:**
- Create: `crates/libgraphql-core-v1/src/types/mod.rs`
- Create: `crates/libgraphql-core-v1/src/types/type_annotation.rs`
- Create: `crates/libgraphql-core-v1/src/types/named_type_annotation.rs`
- Create: `crates/libgraphql-core-v1/src/types/list_type_annotation.rs`

**`type_annotation.rs`:** `TypeAnnotation` enum (Named | List) + `is_equivalent_to()`, `is_subtype_of()`, `innermost_named()`, `innermost_type_name()`, `nullable()`, `Display`. Full rustdocs with spec links to [Type References](https://spec.graphql.org/September2025/#sec-Type-References), [IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation()), and [IsSubType](https://spec.graphql.org/September2025/#IsSubType()).

**`type_annotation.rs`** (key logic — port from v0 `/crates/libgraphql-core/src/types/type_annotation.rs`):
```rust
use crate::names::TypeName;
use crate::span::Span;
use crate::types::named_type_annotation::NamedTypeAnnotation;
use crate::types::list_type_annotation::ListTypeAnnotation;

/// A GraphQL
/// [type reference](https://spec.graphql.org/September2025/#sec-Type-References)
/// (type annotation).
///
/// Represents the type of a field, argument, variable, or input
/// field — including nullability and list wrapping. Recursive:
/// `[String!]!` is `List(non-null, Named(non-null, "String"))`.
///
/// # Subtype and equivalence checks
///
/// [`is_equivalent_to()`](Self::is_equivalent_to) checks structural
/// identity (used for parameter type validation per
/// [IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation())).
///
/// [`is_subtype_of()`](Self::is_subtype_of) checks covariant subtyping
/// (used for field return type validation).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum TypeAnnotation {
    List(ListTypeAnnotation),
    Named(NamedTypeAnnotation),
}

impl TypeAnnotation {
    pub fn named(
        type_name: impl Into<TypeName>,
        nullable: bool,
    ) -> Self {
        Self::Named(NamedTypeAnnotation {
            nullable,
            span: Span::builtin(),
            type_name: type_name.into(),
        })
    }

    pub fn list(inner: TypeAnnotation, nullable: bool) -> Self {
        Self::List(ListTypeAnnotation {
            inner: Box::new(inner),
            nullable,
            span: Span::builtin(),
        })
    }

    #[inline]
    pub fn nullable(&self) -> bool {
        match self {
            Self::List(l) => l.nullable,
            Self::Named(n) => n.nullable,
        }
    }

    #[inline]
    pub fn span(&self) -> Span {
        match self {
            Self::List(l) => l.span,
            Self::Named(n) => n.span,
        }
    }

    /// Recursively unwrap list layers and return the innermost
    /// named type annotation.
    pub fn innermost_named(&self) -> &NamedTypeAnnotation {
        match self {
            Self::List(l) => l.inner.innermost_named(),
            Self::Named(n) => n,
        }
    }

    /// The name of the innermost type (convenience for
    /// `self.innermost_named().type_name()`).
    pub fn innermost_type_name(&self) -> &TypeName {
        &self.innermost_named().type_name
    }

    /// Structural equivalence check. Two annotations are
    /// equivalent if they have the same structure, nullability
    /// at every level, and the same innermost type name.
    ///
    /// Source locations are intentionally ignored.
    ///
    /// Useful for things like parameter type validation where
    /// exact type matching is required per
    /// [IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation()).
    pub fn is_equivalent_to(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Named(a), Self::Named(b)) => {
                a.nullable == b.nullable
                    && a.type_name == b.type_name
            },
            (Self::List(a), Self::List(b)) => {
                a.nullable == b.nullable
                    && a.inner.is_equivalent_to(&b.inner)
            },
            _ => false,
        }
    }

    /// Covariant subtype check per
    /// [IsSubType](https://spec.graphql.org/September2025/#IsSubType()).
    ///
    /// `self` is a valid subtype of `other` if it has equal or
    /// stricter nullability and the innermost type is the same
    /// or a subtype (union member, interface implementor).
    pub fn is_subtype_of(
        &self,
        types_map: &indexmap::IndexMap<
            TypeName,
            crate::types::GraphQLType,
        >,
        other: &Self,
    ) -> bool {
        match (self, other) {
            (Self::Named(a), Self::Named(b)) => {
                // Non-null is subtype of nullable (same name)
                if a.type_name == b.type_name {
                    return !a.nullable || b.nullable;
                }
                // Different names: only valid if self is
                // non-nullable or other is nullable, AND self's
                // type is a member/implementor of other's type.
                // (Abstract type subtyping — requires types_map.)
                (!a.nullable || b.nullable)
                    && is_type_subtype_of(
                        types_map,
                        &a.type_name,
                        &b.type_name,
                    )
            },
            (Self::List(a), Self::List(b)) => {
                (!a.nullable || b.nullable)
                    && a.inner.is_subtype_of(types_map, &b.inner)
            },
            _ => false,
        }
    }
}

/// Check if `sub` is a subtype of `super_` in the type hierarchy.
/// A type is a subtype of itself, or of an interface it implements,
/// or of a union it is a member of.
fn is_type_subtype_of(
    types_map: &indexmap::IndexMap<
        TypeName,
        crate::types::GraphQLType,
    >,
    sub: &TypeName,
    super_: &TypeName,
) -> bool {
    use crate::types::GraphQLType;
    use crate::types::HasFieldsAndInterfaces;

    if sub == super_ {
        return true;
    }
    let Some(super_type) = types_map.get(super_) else {
        return false;
    };
    match super_type {
        GraphQLType::Interface(_) => {
            // sub implements super_ as an interface?
            let Some(sub_type) = types_map.get(sub) else {
                return false;
            };
            match sub_type {
                GraphQLType::Object(obj) => {
                    obj.interfaces().iter().any(|l| &l.value == super_)
                },
                GraphQLType::Interface(iface) => {
                    iface.interfaces().iter().any(|l| &l.value == super_)
                },
                _ => false,
            }
        },
        GraphQLType::Union(union_type) => {
            union_type.members().iter().any(|m| m == sub)
        },
        _ => false,
    }
}

impl std::fmt::Display for TypeAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Named(n) => write!(
                f, "{}{}",
                n.type_name,
                if n.nullable { "" } else { "!" },
            ),
            Self::List(l) => write!(
                f, "[{}]{}",
                l.inner,
                if l.nullable { "" } else { "!" },
            ),
        }
    }
}
```

**`named_type_annotation.rs`:**
```rust
use crate::names::TypeName;
use crate::span::Span;

/// A named type reference with nullability
/// (e.g. `String`, `String!`).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct NamedTypeAnnotation {
    pub(crate) nullable: bool,
    pub(crate) span: Span,
    pub(crate) type_name: TypeName,
}

impl NamedTypeAnnotation {
    #[inline]
    pub fn nullable(&self) -> bool { self.nullable }
    #[inline]
    pub fn span(&self) -> Span { self.span }
    #[inline]
    pub fn type_name(&self) -> &TypeName { &self.type_name }
}
```

**`list_type_annotation.rs`:**
```rust
use crate::span::Span;
use crate::types::type_annotation::TypeAnnotation;

/// A list type wrapper with nullability
/// (e.g. `[String]`, `[String!]!`).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ListTypeAnnotation {
    pub(crate) inner: Box<TypeAnnotation>,
    pub(crate) nullable: bool,
    pub(crate) span: Span,
}

impl ListTypeAnnotation {
    #[inline]
    pub fn inner(&self) -> &TypeAnnotation { &self.inner }
    #[inline]
    pub fn nullable(&self) -> bool { self.nullable }
    #[inline]
    pub fn span(&self) -> Span { self.span }
}
```

**Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Verifies identical type annotations are equivalent.
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

    // Verifies nested list equivalence.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn equivalent_nested_lists() {
        let a = TypeAnnotation::list(
            TypeAnnotation::named("Int", false),
            true,
        );
        let b = TypeAnnotation::list(
            TypeAnnotation::named("Int", false),
            true,
        );
        assert!(a.is_equivalent_to(&b));
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
                TypeAnnotation::named("Int", false), true,
            ).to_string(),
            "[Int!]",
        );
        assert_eq!(
            TypeAnnotation::list(
                TypeAnnotation::named("Int", false), false,
            ).to_string(),
            "[Int!]!",
        );
    }

    // Verifies innermost_type_name unwraps nested lists.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn innermost_type_name_nested() {
        let annot = TypeAnnotation::list(
            TypeAnnotation::list(
                TypeAnnotation::named("User", false),
                true,
            ),
            false,
        );
        assert_eq!(annot.innermost_type_name().as_str(), "User");
    }
}
```

- [ ] Implement `TypeAnnotation`, `NamedTypeAnnotation`, `ListTypeAnnotation` (each in own file)
- [ ] Port subtype/equivalence logic from v0 `type_annotation.rs`, including abstract type subtyping
- [ ] Write thorough tests for equivalence, subtype logic, Display, innermost access
- [ ] Wire up `types/mod.rs` with re-exports
- [ ] Commit: `[libgraphql-core-v1] Add TypeAnnotation with subtype and equivalence logic`

---

### Task 6: ScalarType, ScalarKind, EnumType, EnumValue, DeprecationState

**Files:**
- Create: `crates/libgraphql-core-v1/src/types/scalar_kind.rs`
- Create: `crates/libgraphql-core-v1/src/types/scalar_type.rs`
- Create: `crates/libgraphql-core-v1/src/types/enum_type.rs`
- Create: `crates/libgraphql-core-v1/src/types/enum_value.rs`
- Create: `crates/libgraphql-core-v1/src/types/deprecation_state.rs`

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

**`scalar_type.rs`:**
```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::scalar_kind::ScalarKind;

/// A GraphQL [scalar type](https://spec.graphql.org/September2025/#sec-Scalars).
///
/// Both built-in scalars (`Boolean`, `Float`, `ID`, `Int`, `String`)
/// and custom scalars are represented by this struct. Use
/// [`kind()`](Self::kind) to distinguish them, and
/// [`is_builtin()`](Self::is_builtin) as a convenience check.
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ScalarType {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) kind: ScalarKind,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

impl ScalarType {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn is_builtin(&self) -> bool {
        !matches!(self.kind, ScalarKind::Custom)
    }
    pub fn kind(&self) -> ScalarKind { self.kind }
    pub fn name(&self) -> &TypeName { &self.name }
    pub fn span(&self) -> Span { self.span }
}
```

**`enum_value.rs`:**
```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::EnumValueName;
use crate::span::Span;

/// A single value within an [`EnumType`](crate::types::EnumType)
/// definition.
///
/// See
/// [Enum Values](https://spec.graphql.org/September2025/#EnumValuesDefinition).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct EnumValue {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: EnumValueName,
    pub(crate) parent_type_name: TypeName,
    pub(crate) span: Span,
}

impl EnumValue {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn name(&self) -> &EnumValueName { &self.name }
    /// The name of the [`EnumType`](crate::types::EnumType) that
    /// defines this value.
    pub fn parent_type_name(&self) -> &TypeName {
        &self.parent_type_name
    }
    pub fn span(&self) -> Span { self.span }
}
```

**`enum_type.rs`:**
```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::EnumValueName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::enum_value::EnumValue;
use indexmap::IndexMap;

/// A GraphQL [enum type](https://spec.graphql.org/September2025/#sec-Enums).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct EnumType {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
    pub(crate) values: IndexMap<EnumValueName, EnumValue>,
}

impl EnumType {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn name(&self) -> &TypeName { &self.name }
    pub fn span(&self) -> Span { self.span }
    pub fn value(&self, name: &str) -> Option<&EnumValue> {
        self.values.get(name)
    }
    pub fn values(&self) -> &IndexMap<EnumValueName, EnumValue> {
        &self.values
    }
}
```

**`deprecation_state.rs`:**
```rust
/// Deprecation status of a type, field, enum value, or argument,
/// derived from the presence of a
/// [`@deprecated`](https://spec.graphql.org/September2025/#sec--deprecated)
/// directive annotation.
#[derive(Clone, Debug, PartialEq)]
pub enum DeprecationState<'a> {
    Active,
    Deprecated { reason: Option<&'a str> },
}

impl DeprecationState<'_> {
    #[inline]
    pub fn is_deprecated(&self) -> bool {
        matches!(self, Self::Deprecated { .. })
    }
}
```

**Tests:**
```rust
// Verifies ScalarKind discriminates built-ins from custom.
// Written by Claude Code, reviewed by a human.
#[test]
fn scalar_kind_builtin_check() {
    assert!(!matches!(ScalarKind::Custom, ScalarKind::Boolean));
    assert!(matches!(ScalarKind::Boolean, ScalarKind::Boolean));
    let scalar = ScalarType {
        kind: ScalarKind::Boolean,
        name: TypeName::new("Boolean"),
        description: None,
        directives: vec![],
        span: Span::builtin(),
    };
    assert!(scalar.is_builtin());
}

// Verifies custom scalars are not built-in.
// Written by Claude Code, reviewed by a human.
#[test]
fn custom_scalar_not_builtin() {
    let scalar = ScalarType {
        kind: ScalarKind::Custom,
        name: TypeName::new("DateTime"),
        description: None,
        directives: vec![],
        span: Span::builtin(),
    };
    assert!(!scalar.is_builtin());
}
```

- [ ] Implement all 5 types, each in own file, with rustdocs
- [ ] Write tests for ScalarKind, ScalarType.is_builtin(), EnumType accessors
- [ ] Commit: `[libgraphql-core-v1] Add ScalarType, ScalarKind, EnumType, EnumValue, DeprecationState`

---

### Task 7: FieldDefinition + ParameterDefinition

**Files:**
- Create: `crates/libgraphql-core-v1/src/types/field_definition.rs`
- Create: `crates/libgraphql-core-v1/src/types/parameter_definition.rs`

**`parameter_definition.rs`:**
```rust
use crate::names::FieldName;
use crate::span::Span;
use crate::types::type_annotation::TypeAnnotation;
use crate::value::Value;

/// A parameter definition on a
/// [`FieldDefinition`](crate::types::FieldDefinition) or
/// [`DirectiveDefinition`](crate::types::DirectiveDefinition).
///
/// Referred to as an "argument definition" in the GraphQL spec
/// ([`InputValueDefinition`](https://spec.graphql.org/September2025/#InputValueDefinition)
/// in the grammar).
///
/// See [Field Arguments](https://spec.graphql.org/September2025/#sec-Field-Arguments).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ParameterDefinition {
    pub(crate) default_value: Option<Value>,
    pub(crate) description: Option<String>,
    pub(crate) name: FieldName,
    pub(crate) span: Span,
    pub(crate) type_annotation: TypeAnnotation,
}

impl ParameterDefinition {
    pub fn default_value(&self) -> Option<&Value> {
        self.default_value.as_ref()
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn name(&self) -> &FieldName { &self.name }
    pub fn span(&self) -> Span { self.span }
    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_annotation
    }
}
```

**`field_definition.rs`:**
```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::parameter_definition::ParameterDefinition;
use crate::types::type_annotation::TypeAnnotation;
use indexmap::IndexMap;

/// A field definition on an
/// [`ObjectType`](crate::types::ObjectType) or
/// [`InterfaceType`](crate::types::InterfaceType).
///
/// This is a *schema-level* field definition — the shape and type
/// of a field as declared in the schema. For a *selected* field
/// within an operation, see
/// [`FieldSelection`](crate::operation::FieldSelection).
///
/// See [Field Definitions](https://spec.graphql.org/September2025/#FieldsDefinition).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct FieldDefinition {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: FieldName,
    pub(crate) parameters: IndexMap<FieldName, ParameterDefinition>,
    pub(crate) parent_type_name: TypeName,
    pub(crate) span: Span,
    pub(crate) type_annotation: TypeAnnotation,
}

impl FieldDefinition {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn name(&self) -> &FieldName { &self.name }
    pub fn parameters(&self) -> &IndexMap<FieldName, ParameterDefinition> {
        &self.parameters
    }
    pub fn parent_type_name(&self) -> &TypeName {
        &self.parent_type_name
    }
    /// The name of the innermost type this field returns.
    /// Convenience for
    /// `self.type_annotation().innermost_type_name()`.
    pub fn return_type_name(&self) -> &TypeName {
        self.type_annotation.innermost_type_name()
    }
    pub fn span(&self) -> Span { self.span }
    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_annotation
    }
}
```

- [ ] Implement both types with full rustdocs
- [ ] Write tests for `return_type_name()` and parameter accessors
- [ ] Commit: `[libgraphql-core-v1] Add FieldDefinition and ParameterDefinition`

---

### Task 8: HasFieldsAndInterfaces + ObjectType + InterfaceType

**Files:**
- Create: `crates/libgraphql-core-v1/src/types/has_fields_and_interfaces.rs`
- Create: `crates/libgraphql-core-v1/src/types/fielded_type_data.rs`
- Create: `crates/libgraphql-core-v1/src/types/object_type.rs`
- Create: `crates/libgraphql-core-v1/src/types/interface_type.rs`

**`has_fields_and_interfaces.rs`:**
```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::field_definition::FieldDefinition;
use indexmap::IndexMap;

/// Shared behavior for types that define fields and implement
/// interfaces — [`ObjectType`](crate::types::ObjectType) and
/// [`InterfaceType`](crate::types::InterfaceType).
///
/// The GraphQL spec treats these as
/// "[composite output types](https://spec.graphql.org/September2025/#sec-Objects)"
/// with overlapping rules: both define fields, both can implement
/// interfaces, and both are validated by the same interface
/// implementation contract
/// ([IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation())).
///
/// This trait enables the validator and downstream consumers to
/// operate generically over both types without duplication.
pub trait HasFieldsAndInterfaces {
    fn description(&self) -> Option<&str>;
    fn directives(&self) -> &[DirectiveAnnotation];
    fn field(&self, name: &str) -> Option<&FieldDefinition>;
    fn fields(&self) -> &IndexMap<FieldName, FieldDefinition>;
    fn interfaces(&self) -> &[Located<TypeName>];
    fn name(&self) -> &TypeName;
    fn span(&self) -> Span;
}
```

**`fielded_type_data.rs`:**
```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::field_definition::FieldDefinition;
use indexmap::IndexMap;

/// Shared data for [`ObjectType`](crate::types::ObjectType) and
/// [`InterfaceType`](crate::types::InterfaceType).
///
/// Not part of the public API. Both types wrap this struct and
/// delegate via [`HasFieldsAndInterfaces`](crate::types::HasFieldsAndInterfaces).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct FieldedTypeData {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) fields: IndexMap<FieldName, FieldDefinition>,
    pub(crate) interfaces: Vec<Located<TypeName>>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}
```

**`object_type.rs`:**
```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::field_definition::FieldDefinition;
use crate::types::fielded_type_data::FieldedTypeData;
use crate::types::has_fields_and_interfaces::HasFieldsAndInterfaces;
use indexmap::IndexMap;

/// A GraphQL [object type](https://spec.graphql.org/September2025/#sec-Objects).
///
/// Object types are the primary composite output type in GraphQL.
/// They define a set of named fields, each of which yields a value
/// of a specific type. Object types may implement one or more
/// interfaces, committing to provide the fields those interfaces
/// specify.
///
/// # Shared behavior
///
/// Object types share their field and interface structure with
/// [`InterfaceType`](crate::types::InterfaceType) via the
/// [`HasFieldsAndInterfaces`] trait.
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(transparent)]
pub struct ObjectType(pub(crate) FieldedTypeData);

impl HasFieldsAndInterfaces for ObjectType {
    fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }
    fn directives(&self) -> &[DirectiveAnnotation] {
        &self.0.directives
    }
    fn field(&self, name: &str) -> Option<&FieldDefinition> {
        self.0.fields.get(name)
    }
    fn fields(&self) -> &IndexMap<FieldName, FieldDefinition> {
        &self.0.fields
    }
    fn interfaces(&self) -> &[Located<TypeName>] {
        &self.0.interfaces
    }
    fn name(&self) -> &TypeName { &self.0.name }
    fn span(&self) -> Span { self.0.span }
}
```

**`interface_type.rs`:** Identical pattern to ObjectType, differing only in struct name and rustdoc linking to [Interfaces](https://spec.graphql.org/September2025/#sec-Interfaces).

- [ ] Implement trait, shared data struct, and both types
- [ ] Write tests verifying trait method delegation (construct an ObjectType, access fields via trait)
- [ ] Commit: `[libgraphql-core-v1] Add HasFieldsAndInterfaces, ObjectType, InterfaceType`

---

### Task 9: InputObjectType, InputField, UnionType

**Files:**
- Create: `crates/libgraphql-core-v1/src/types/input_object_type.rs`
- Create: `crates/libgraphql-core-v1/src/types/input_field.rs`
- Create: `crates/libgraphql-core-v1/src/types/union_type.rs`

**`input_field.rs`:**
```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::type_annotation::TypeAnnotation;
use crate::value::Value;

/// A field on an
/// [`InputObjectType`](crate::types::InputObjectType).
///
/// Input fields differ from output
/// [`FieldDefinition`](crate::types::FieldDefinition)s: they
/// can have default values but cannot have parameters or
/// selection sets.
///
/// See [Input Object Fields](https://spec.graphql.org/September2025/#sec-Input-Objects).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct InputField {
    pub(crate) default_value: Option<Value>,
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) name: FieldName,
    pub(crate) parent_type_name: TypeName,
    pub(crate) span: Span,
    pub(crate) type_annotation: TypeAnnotation,
}

impl InputField {
    pub fn default_value(&self) -> Option<&Value> {
        self.default_value.as_ref()
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn name(&self) -> &FieldName { &self.name }
    pub fn parent_type_name(&self) -> &TypeName {
        &self.parent_type_name
    }
    pub fn span(&self) -> Span { self.span }
    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_annotation
    }
}
```

**`input_object_type.rs`:**
```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::types::input_field::InputField;
use indexmap::IndexMap;

/// A GraphQL [input object type](https://spec.graphql.org/September2025/#sec-Input-Objects).
///
/// Input objects are the composite input type — they define a
/// set of named input fields, each with a type that must itself
/// be an input type (scalar, enum, or another input object).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct InputObjectType {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) fields: IndexMap<FieldName, InputField>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

impl InputObjectType {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn field(&self, name: &str) -> Option<&InputField> {
        self.fields.get(name)
    }
    pub fn fields(&self) -> &IndexMap<FieldName, InputField> {
        &self.fields
    }
    pub fn name(&self) -> &TypeName { &self.name }
    pub fn span(&self) -> Span { self.span }
}
```

**`union_type.rs`:**
```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::TypeName;
use crate::span::Span;

/// A GraphQL [union type](https://spec.graphql.org/September2025/#sec-Unions).
///
/// Unions represent a value that could be one of several object
/// types. Unlike interfaces, unions do not define shared fields
/// — the member types are opaque until resolved via a type
/// condition.
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct UnionType {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) members: Vec<Located<TypeName>>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

impl UnionType {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    /// The union's member types, each carrying the span of its
    /// occurrence in the schema source.
    pub fn members(&self) -> &[Located<TypeName>] {
        &self.members
    }
    pub fn name(&self) -> &TypeName { &self.name }
    pub fn span(&self) -> Span { self.span }
}
```

- [ ] Implement all 3 types in own files with full rustdocs
- [ ] Write basic accessor tests
- [ ] Commit: `[libgraphql-core-v1] Add InputObjectType, InputField, UnionType`

---

### Task 10: DirectiveDefinition + DirectiveDefinitionKind + DirectiveLocationKind

**Files:**
- Create: `crates/libgraphql-core-v1/src/types/directive_definition.rs`
- Create: `crates/libgraphql-core-v1/src/types/directive_definition_kind.rs`
- Create: `crates/libgraphql-core-v1/src/types/directive_location_kind.rs`

**`directive_definition_kind.rs`:**
```rust
/// Identifies whether a directive definition is one of the
/// built-in GraphQL directives or a custom (user-defined)
/// directive.
///
/// See [Built-in Directives](https://spec.graphql.org/September2025/#sec-Type-System.Directives.Built-in-Directives).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum DirectiveDefinitionKind {
    Custom,
    Deprecated,
    Include,
    Skip,
    SpecifiedBy,
}

impl DirectiveDefinitionKind {
    pub fn is_builtin(&self) -> bool {
        !matches!(self, Self::Custom)
    }
}
```

**`directive_location_kind.rs`:** Re-export the parser's type directly rather than redefining:
```rust
/// Re-export of the parser's directive location kind.
///
/// See [Directive Locations](https://spec.graphql.org/September2025/#DirectiveLocations).
pub use libgraphql_parser::ast::DirectiveLocationKind;
```

Note: if `DirectiveLocationKind` needs `serde::Serialize`/`serde::Deserialize` (for Schema serialization), we should add those derives to `libgraphql-parser`'s definition. See the project-tracker task below.

**`directive_definition.rs`:**
```rust
use crate::names::DirectiveName;
use crate::names::FieldName;
use crate::span::Span;
use crate::types::directive_definition_kind::DirectiveDefinitionKind;
use crate::types::directive_location_kind::DirectiveLocationKind;
use crate::types::parameter_definition::ParameterDefinition;
use indexmap::IndexMap;

/// A directive definition in a GraphQL schema.
///
/// Unlike v0's asymmetric `Directive` enum (where built-in
/// variants were unit variants and custom carried data), v2 uses
/// a single struct for all directives. Built-ins are regular
/// entries distinguished by
/// [`kind()`](Self::kind) returning a non-`Custom` variant.
///
/// See [Type System Directives](https://spec.graphql.org/September2025/#sec-Type-System.Directives).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct DirectiveDefinition {
    pub(crate) description: Option<String>,
    pub(crate) is_repeatable: bool,
    pub(crate) kind: DirectiveDefinitionKind,
    pub(crate) locations: Vec<DirectiveLocationKind>,
    pub(crate) name: DirectiveName,
    pub(crate) parameters: IndexMap<FieldName, ParameterDefinition>,
    pub(crate) span: Span,
}

impl DirectiveDefinition {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn is_builtin(&self) -> bool { self.kind.is_builtin() }
    pub fn is_repeatable(&self) -> bool { self.is_repeatable }
    pub fn kind(&self) -> DirectiveDefinitionKind { self.kind }
    pub fn locations(&self) -> &[DirectiveLocationKind] {
        &self.locations
    }
    pub fn name(&self) -> &DirectiveName { &self.name }
    pub fn parameters(
        &self,
    ) -> &IndexMap<FieldName, ParameterDefinition> {
        &self.parameters
    }
    pub fn span(&self) -> Span { self.span }
}
```

**Tests:**
```rust
// Verifies DirectiveDefinitionKind discriminates built-ins.
// Written by Claude Code, reviewed by a human.
#[test]
fn directive_kind_builtin() {
    assert!(DirectiveDefinitionKind::Deprecated.is_builtin());
    assert!(DirectiveDefinitionKind::Include.is_builtin());
    assert!(DirectiveDefinitionKind::Skip.is_builtin());
    assert!(DirectiveDefinitionKind::SpecifiedBy.is_builtin());
    assert!(!DirectiveDefinitionKind::Custom.is_builtin());
}
```

- [ ] Implement all 3 types with full rustdocs
- [ ] Write tests (kind matching, is_builtin, accessor methods)
- [ ] Commit: `[libgraphql-core-v1] Add DirectiveDefinition, DirectiveDefinitionKind, DirectiveLocationKind`

---

### Task 11: GraphQLType + GraphQLTypeKind

**Files:**
- Create: `crates/libgraphql-core-v1/src/types/graphql_type.rs`
- Create: `crates/libgraphql-core-v1/src/types/graphql_type_kind.rs`

**`graphql_type_kind.rs`:**
```rust
use crate::types::scalar_kind::ScalarKind;

/// Discriminates all GraphQL type categories, including
/// individual built-in scalar identities.
///
/// This enum has 11 variants — the 6 data-carrying categories
/// plus the 5 built-in scalars broken out from `Scalar`. Use
/// [`GraphQLType::type_kind()`](crate::types::GraphQLType::type_kind)
/// when you need exhaustive matching over all type identities.
///
/// See [Types](https://spec.graphql.org/September2025/#sec-Types).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum GraphQLTypeKind {
    Boolean,
    Enum,
    Float,
    ID,
    InputObject,
    Int,
    Interface,
    Object,
    Scalar,
    String,
    Union,
}

impl From<ScalarKind> for GraphQLTypeKind {
    fn from(kind: ScalarKind) -> Self {
        match kind {
            ScalarKind::Boolean => Self::Boolean,
            ScalarKind::Custom => Self::Scalar,
            ScalarKind::Float => Self::Float,
            ScalarKind::ID => Self::ID,
            ScalarKind::Int => Self::Int,
            ScalarKind::String => Self::String,
        }
    }
}
```

**`graphql_type.rs`:**
```rust
use crate::names::TypeName;
use crate::span::Span;
use crate::types::enum_type::EnumType;
use crate::types::graphql_type_kind::GraphQLTypeKind;
use crate::types::has_fields_and_interfaces::HasFieldsAndInterfaces;
use crate::types::input_object_type::InputObjectType;
use crate::types::interface_type::InterfaceType;
use crate::types::object_type::ObjectType;
use crate::types::scalar_type::ScalarType;
use crate::types::union_type::UnionType;

/// A defined GraphQL type.
///
/// This enum has 6 data-carrying variants — one per type
/// category. Built-in scalars (`Boolean`, `Float`, `ID`, `Int`,
/// `String`) are represented as
/// `Scalar(ScalarType { kind: ScalarKind::Boolean, .. })` rather
/// than separate enum variants, keeping accessor methods at 6
/// match arms instead of 11.
///
/// For exhaustive matching that distinguishes built-in scalar
/// identity, use [`type_kind()`](Self::type_kind) which returns
/// an 11-variant [`GraphQLTypeKind`].
///
/// See [Types](https://spec.graphql.org/September2025/#sec-Types).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum GraphQLType {
    Enum(Box<EnumType>),
    InputObject(Box<InputObjectType>),
    Interface(Box<InterfaceType>),
    Object(Box<ObjectType>),
    Scalar(Box<ScalarType>),
    Union(Box<UnionType>),
}

impl GraphQLType {
    #[inline]
    pub fn name(&self) -> &TypeName {
        match self {
            Self::Enum(t) => t.name(),
            Self::InputObject(t) => t.name(),
            Self::Interface(t) => HasFieldsAndInterfaces::name(t.as_ref()),
            Self::Object(t) => HasFieldsAndInterfaces::name(t.as_ref()),
            Self::Scalar(t) => t.name(),
            Self::Union(t) => t.name(),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Enum(t) => t.span(),
            Self::InputObject(t) => t.span(),
            Self::Interface(t) => HasFieldsAndInterfaces::span(t.as_ref()),
            Self::Object(t) => HasFieldsAndInterfaces::span(t.as_ref()),
            Self::Scalar(t) => t.span(),
            Self::Union(t) => t.span(),
        }
    }

    pub fn description(&self) -> Option<&str> {
        match self {
            Self::Enum(t) => t.description(),
            Self::InputObject(t) => t.description(),
            Self::Interface(t) => t.description(),
            Self::Object(t) => t.description(),
            Self::Scalar(t) => t.description(),
            Self::Union(t) => t.description(),
        }
    }

    /// Input types can appear in input positions (arguments,
    /// variables, input object fields): Scalar, Enum, InputObject.
    ///
    /// Note that Scalar and Enum are both input *and* output types,
    /// so `is_input_type()` and [`is_output_type()`](Self::is_output_type)
    /// are not opposites.
    ///
    /// See [Input and Output Types](https://spec.graphql.org/September2025/#sec-Input-and-Output-Types).
    pub fn is_input_type(&self) -> bool {
        matches!(
            self,
            Self::Enum(_) | Self::InputObject(_) | Self::Scalar(_),
        )
    }

    /// Output types can appear in output positions (field return
    /// types): Scalar, Enum, Object, Interface, Union.
    ///
    /// Note that Scalar and Enum are both input *and* output types,
    /// so `is_output_type()` and [`is_input_type()`](Self::is_input_type)
    /// are not opposites.
    ///
    /// See [Input and Output Types](https://spec.graphql.org/September2025/#sec-Input-and-Output-Types).
    pub fn is_output_type(&self) -> bool {
        matches!(
            self,
            Self::Enum(_) | Self::Interface(_) | Self::Object(_)
                | Self::Scalar(_) | Self::Union(_),
        )
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self, Self::Scalar(s) if s.is_builtin())
    }

    /// Composite types can have selection sets: Object,
    /// Interface, Union.
    pub fn is_composite_type(&self) -> bool {
        matches!(
            self,
            Self::Interface(_) | Self::Object(_) | Self::Union(_),
        )
    }

    /// Leaf types cannot have selection sets: Scalar, Enum.
    pub fn is_leaf_type(&self) -> bool {
        matches!(self, Self::Enum(_) | Self::Scalar(_))
    }

    /// Returns the fully-discriminated type kind, including
    /// built-in scalar identity (11 variants).
    pub fn type_kind(&self) -> GraphQLTypeKind {
        match self {
            Self::Enum(_) => GraphQLTypeKind::Enum,
            Self::InputObject(_) => GraphQLTypeKind::InputObject,
            Self::Interface(_) => GraphQLTypeKind::Interface,
            Self::Object(_) => GraphQLTypeKind::Object,
            Self::Scalar(s) => s.kind().into(),
            Self::Union(_) => GraphQLTypeKind::Union,
        }
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
}
```

**Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn builtin_scalar(kind: ScalarKind, name: &str) -> GraphQLType {
        GraphQLType::Scalar(Box::new(ScalarType {
            description: None,
            directives: vec![],
            kind,
            name: TypeName::new(name),
            span: Span::builtin(),
        }))
    }

    // Verifies is_input_type/is_output_type per the spec's
    // input/output type classification.
    // https://spec.graphql.org/September2025/#sec-Input-and-Output-Types
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn input_output_classification() {
        let scalar = builtin_scalar(ScalarKind::String, "String");
        assert!(scalar.is_input_type());
        assert!(scalar.is_output_type());

        // InputObject: input only
        let input_obj = GraphQLType::InputObject(Box::new(
            InputObjectType { /* ... */ },
        ));
        assert!(input_obj.is_input_type());
        assert!(!input_obj.is_output_type());
    }

    // Verifies type_kind() maps ScalarKind to correct
    // GraphQLTypeKind (11-variant exhaustive matching).
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn type_kind_mapping() {
        let boolean = builtin_scalar(ScalarKind::Boolean, "Boolean");
        assert_eq!(boolean.type_kind(), GraphQLTypeKind::Boolean);

        let custom = GraphQLType::Scalar(Box::new(ScalarType {
            kind: ScalarKind::Custom,
            name: TypeName::new("DateTime"),
            description: None,
            directives: vec![],
            span: Span::builtin(),
        }));
        assert_eq!(custom.type_kind(), GraphQLTypeKind::Scalar);
    }

    // Verifies is_builtin() only returns true for built-in scalars.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn is_builtin() {
        let builtin = builtin_scalar(ScalarKind::Int, "Int");
        assert!(builtin.is_builtin());

        let custom = GraphQLType::Scalar(Box::new(ScalarType {
            kind: ScalarKind::Custom,
            name: TypeName::new("BigInt"),
            description: None,
            directives: vec![],
            span: Span::builtin(),
        }));
        assert!(!custom.is_builtin());
    }
}
```

- [ ] Implement `GraphQLType` with all methods and full rustdocs
- [ ] Implement `GraphQLTypeKind` with `From<ScalarKind>` conversion
- [ ] Finalize all `types/mod.rs` re-exports
- [ ] Write thorough tests for input/output classification, kind mapping, is_builtin, downcasts
- [ ] Commit: `[libgraphql-core-v1] Add GraphQLType (6 variants) and GraphQLTypeKind (11 variants)`

---

### Task 12: Type Builders

**Files:** All files under `type_builders/`

Each builder follows the same pattern:
- `new(name, span)` constructor — fails immediately if name starts with `__`
- `set_*()` mutators returning `&mut Self` (infallible setters)
- `add_*()` mutators returning `Result<&mut Self, SchemaBuildError>` — **fail-fast**: check for duplicates, `__` prefix, and other deterministic errors at the point of the call, not deferred to `build()`
- `from_ast(ast_node, source_map_id)` class method converting parser AST to builder — collects all errors internally (since it processes the whole AST at once), errors surface via `absorb_type()` or `build()`
- `build_from_ast(ast_node, source_map_id)` convenience shortcut: `Self::from_ast(...).build()`
- All data `pub(crate)` for SchemaBuilder access

**`ast_helpers.rs`** — `pub(crate)` shared conversion functions:
```rust
use crate::names::DirectiveName;
use crate::names::EnumValueName;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::span::Span;
use crate::span::SourceMapId;
use crate::types::type_annotation::TypeAnnotation;
use crate::value::Value;
use libgraphql_parser::ByteSpan;
use libgraphql_parser::ast;

pub(crate) fn span_from_ast(
    byte_span: ByteSpan,
    source_map_id: SourceMapId,
) -> Span {
    Span::new(byte_span, source_map_id)
}

pub(crate) fn type_annotation_from_ast(
    ast_annot: &ast::TypeAnnotation<'_>,
    source_map_id: SourceMapId,
) -> TypeAnnotation {
    match ast_annot {
        ast::TypeAnnotation::Named(named) => {
            TypeAnnotation::Named(
                crate::types::named_type_annotation::NamedTypeAnnotation {
                    nullable: named.nullable(),
                    span: span_from_ast(named.span, source_map_id),
                    type_name: TypeName::new(
                        named.name.value.as_ref(),
                    ),
                },
            )
        },
        ast::TypeAnnotation::List(list) => {
            TypeAnnotation::List(
                crate::types::list_type_annotation::ListTypeAnnotation {
                    inner: Box::new(type_annotation_from_ast(
                        &list.element_type,
                        source_map_id,
                    )),
                    nullable: list.nullable(),
                    span: span_from_ast(list.span, source_map_id),
                },
            )
        },
    }
}

pub(crate) fn value_from_ast(
    ast_val: &ast::Value<'_>,
) -> Value {
    match ast_val {
        ast::Value::Boolean(v) => Value::Boolean(v.value),
        ast::Value::Enum(v) => Value::Enum(
            EnumValueName::new(v.value.as_ref()),
        ),
        ast::Value::Float(v) => Value::Float(v.value),
        ast::Value::Int(v) => Value::Int(v.value),
        ast::Value::List(v) => Value::List(
            v.values.iter().map(value_from_ast).collect(),
        ),
        ast::Value::Null(_) => Value::Null,
        ast::Value::Object(v) => Value::Object(
            v.fields.iter().map(|f| {
                (FieldName::new(f.name.value.as_ref()), value_from_ast(&f.value))
            }).collect(),
        ),
        ast::Value::String(v) => {
            Value::String(v.value.to_string())
        },
        ast::Value::Variable(v) => Value::VarRef(
            crate::names::VariableName::new(v.name.value.as_ref()),
        ),
    }
}

pub(crate) fn directive_annotation_from_ast(
    ast_dir: &ast::DirectiveAnnotation<'_>,
    source_map_id: SourceMapId,
) -> crate::directive_annotation::DirectiveAnnotation {
    crate::directive_annotation::DirectiveAnnotation {
        arguments: ast_dir.arguments.iter().map(|arg| {
            (FieldName::new(arg.name.value.as_ref()), value_from_ast(&arg.value))
        }).collect(),
        name: DirectiveName::new(ast_dir.name.value.as_ref()),
        span: span_from_ast(ast_dir.span, source_map_id),
    }
}
```

**Example builder** (`object_type_builder.rs`):
```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::TypeName;
use crate::span::Span;
use crate::span::SourceMapId;
use crate::type_builders::ast_helpers;
use crate::type_builders::field_def_builder::FieldDefBuilder;
use libgraphql_parser::ast;

/// Builder for constructing an [`ObjectType`](crate::types::ObjectType).
///
/// Use [`SchemaBuilder::absorb_type()`](crate::schema::SchemaBuilder::absorb_type)
/// to register a completed builder with the schema.
///
/// # Programmatic construction
///
/// ```ignore
/// let mut builder = ObjectTypeBuilder::new("User", span);
/// builder.add_field(field_def);
/// builder.add_implements("Node");
/// schema_builder.absorb_type(builder)?;
/// ```
///
/// # From parser AST
///
/// ```ignore
/// let builder = ObjectTypeBuilder::from_ast(&ast_obj, source_map_id);
/// schema_builder.absorb_type(builder)?;
/// ```
pub struct ObjectTypeBuilder {
    pub(crate) description: Option<String>,
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) errors: Vec<SchemaBuildError>,
    pub(crate) fields: Vec<FieldDefBuilder>,
    pub(crate) implements: Vec<Located<TypeName>>,
    pub(crate) name: TypeName,
    pub(crate) span: Span,
}

impl ObjectTypeBuilder {
    pub fn new(
        name: impl Into<TypeName>,
        span: Span,
    ) -> Result<Self, SchemaBuildError> {
        let name = name.into();
        if name.as_str().starts_with("__") {
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidDunderPrefixedTypeName {
                    type_name: name.to_string(),
                },
                span,
                vec![],
            ));
        }
        Ok(Self {
            description: None,
            directives: vec![],
            errors: vec![],
            fields: vec![],
            implements: vec![],
            name,
            span,
        })
    }

    pub fn from_ast(
        ast_obj: &ast::ObjectTypeDefinition<'_>,
        source_map_id: SourceMapId,
    ) -> Self {
        let span = ast_helpers::span_from_ast(
            ast_obj.span, source_map_id,
        );
        // from_ast collects errors internally instead of
        // propagating via Result — it processes the whole AST
        // node at once.
        let mut builder = Self {
            description: None,
            directives: vec![],
            errors: vec![],
            fields: vec![],
            implements: vec![],
            name: TypeName::new(ast_obj.name.value.as_ref()),
            span,
        };
        if builder.name.as_str().starts_with("__") {
            builder.errors.push(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidDunderPrefixedTypeName {
                    type_name: builder.name.to_string(),
                },
                span,
                vec![],
            ));
        }
        if let Some(desc) = &ast_obj.description {
            builder.set_description(desc.value.to_string());
        }
        for iface in &ast_obj.implements {
            let iface_span = ast_helpers::span_from_ast(
                iface.span, source_map_id,
            );
            // Collect errors instead of returning Result
            if let Err(e) = builder.add_implements(
                iface.value.as_ref(), iface_span,
            ) {
                builder.errors.push(e);
            }
        }
        for dir in &ast_obj.directives {
            builder.add_directive(
                ast_helpers::directive_annotation_from_ast(
                    dir, source_map_id,
                ),
            );
        }
        for field in &ast_obj.fields {
            if let Err(e) = builder.add_field(
                FieldDefBuilder::from_ast(field, source_map_id),
            ) {
                builder.errors.push(e);
            }
        }
        builder
    }

    pub fn set_description(
        &mut self,
        desc: impl Into<String>,
    ) -> &mut Self {
        self.description = Some(desc.into());
        self
    }

    /// Fails immediately if a field with the same name already
    /// exists or if the field name starts with `__`.
    pub fn add_field(
        &mut self,
        field: FieldDefBuilder,
    ) -> Result<&mut Self, SchemaBuildError> {
        if field.name.as_str().starts_with("__") {
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::InvalidDunderPrefixedFieldName {
                    field_name: field.name.to_string(),
                    type_name: self.name.to_string(),
                },
                field.span,
                vec![],
            ));
        }
        if self.fields.iter().any(|f| f.name == field.name) {
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateFieldNameDefinition {
                    field_name: field.name.to_string(),
                    type_name: self.name.to_string(),
                },
                field.span,
                vec![],
            ));
        }
        self.fields.push(field);
        Ok(self)
    }

    /// Fails immediately if this interface is already listed.
    pub fn add_implements(
        &mut self,
        iface: impl Into<TypeName>,
        span: Span,
    ) -> Result<&mut Self, SchemaBuildError> {
        let iface = iface.into();
        if self.implements.iter().any(|l| l.value == iface) {
            return Err(SchemaBuildError::new(
                SchemaBuildErrorKind::DuplicateInterfaceImplementsDeclaration {
                    type_name: self.name.to_string(),
                    interface_name: iface.to_string(),
                },
                span,
                vec![],
            ));
        }
        self.implements.push(Located { value: iface, span });
        Ok(self)
    }

    pub fn add_directive(
        &mut self,
        dir: DirectiveAnnotation,
    ) -> &mut Self {
        self.directives.push(dir);
        self
    }
}
```

The remaining 6 builders (`InterfaceTypeBuilder`, `UnionTypeBuilder`, `EnumTypeBuilder`, `ScalarTypeBuilder`, `InputObjectTypeBuilder`, `DirectiveBuilder`) follow the same pattern — `new()`, `from_ast()`, `set_*()`, `add_*()`. `InterfaceTypeBuilder` is nearly identical to `ObjectTypeBuilder`. `UnionTypeBuilder` has `add_member()`. `EnumTypeBuilder` has `add_value()`. `DirectiveBuilder` has `add_location()`, `set_repeatable()`, `add_parameter()`.

Builder-stage data types (`FieldDefBuilder`, `ParameterDefBuilder`, `InputFieldDefBuilder`, `EnumValueDefBuilder`) each in own file — these hold intermediate data before validation. Each has `from_ast()` delegating to `ast_helpers`.

**Tests:**
```rust
// Verifies from_ast() correctly converts a parsed object type.
// Written by Claude Code, reviewed by a human.
#[test]
fn object_type_from_ast() {
    let result = libgraphql_parser::parse_schema(
        "type User implements Node { id: ID!, name: String }",
    );
    let doc = result.ast();
    let def = match &doc.definitions[0] {
        libgraphql_parser::ast::Definition::TypeDefinition(
            libgraphql_parser::ast::TypeDefinition::Object(obj),
        ) => obj,
        _ => panic!("expected object type definition"),
    };
    let builder = ObjectTypeBuilder::from_ast(def, SourceMapId(1));
    assert!(builder.errors.is_empty());
    assert_eq!(builder.name.as_str(), "User");
    assert_eq!(builder.implements.len(), 1);
    assert_eq!(builder.implements[0].value.as_str(), "Node");
    assert_eq!(builder.fields.len(), 2);
    assert_eq!(builder.fields[0].name.as_str(), "id");
    assert_eq!(builder.fields[1].name.as_str(), "name");
}

// Verifies add_field() fails immediately on duplicate name.
// Written by Claude Code, reviewed by a human.
#[test]
fn add_field_rejects_duplicate() {
    let mut builder = ObjectTypeBuilder::new(
        "User", Span::builtin(),
    ).unwrap();
    builder.add_field(FieldDefBuilder {
        name: FieldName::new("id"),
        // ... other fields
    }).unwrap();
    let err = builder.add_field(FieldDefBuilder {
        name: FieldName::new("id"),
        // ... other fields
    }).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::DuplicateFieldNameDefinition { .. },
    ));
}

// Verifies new() fails immediately on __ prefix.
// https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
// Written by Claude Code, reviewed by a human.
#[test]
fn new_rejects_dunder_prefix() {
    let err = ObjectTypeBuilder::new(
        "__Bad", Span::builtin(),
    ).unwrap_err();
    assert!(matches!(
        err.kind(),
        SchemaBuildErrorKind::InvalidDunderPrefixedTypeName { .. },
    ));
}
```

- [ ] Implement `ast_helpers.rs` with shared conversion functions
- [ ] Implement all builder-stage data types (`FieldDefBuilder`, `ParameterDefBuilder`, `InputFieldDefBuilder`, `EnumValueDefBuilder`), each in own file
- [ ] Implement all 7 type builders with `from_ast()` methods
- [ ] Write `from_ast()` round-trip tests
- [ ] Commit: `[libgraphql-core-v1] Add type builders with from_ast() support`

---

### Task 13: Schema Errors

**Files:**
- Create: `crates/libgraphql-core-v1/src/schema/mod.rs`
- Create: `crates/libgraphql-core-v1/src/schema/schema_errors.rs`
- Create: `crates/libgraphql-core-v1/src/schema/schema_build_error.rs`
- Create: `crates/libgraphql-core-v1/src/schema/type_validation_error.rs`

Errors use a **structured error + `#[non_exhaustive]` kind enum** pattern inspired by `libgraphql-parser`'s error system. Each error is a struct with a primary `span`, a `kind` for programmatic matching, and a `notes` vector for secondary context (related locations, spec references, hints). This replaces v0's flat enums where each variant carried its own scattered span/location fields.

**`error_note.rs`** (adapted from parser):
```rust
use crate::span::Span;

/// Additional context attached to an error — secondary
/// locations, spec references, or suggested fixes.
#[derive(Clone, Debug)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ErrorNote {
    pub kind: ErrorNoteKind,
    pub message: String,
    pub span: Option<Span>,
}

/// The kind of additional context an [`ErrorNote`] provides.
#[derive(Clone, Copy, Debug)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum ErrorNoteKind {
    /// General context ("note: ...").
    General,
    /// A suggestion or fix ("help: ...").
    Help,
    /// A link to the relevant spec section ("spec: ...").
    Spec,
}
```

**`schema_build_error.rs`:**
```rust
use crate::error_note::ErrorNote;
use crate::span::Span;

/// An error encountered during schema construction.
///
/// Every error carries:
/// - A primary [`span`](Self::span) pointing to the source of
///   the error
/// - A [`kind`](Self::kind) for programmatic matching
/// - Optional [`notes`](Self::notes) with secondary locations,
///   spec references, and hints
///
/// The `kind` enum is `#[non_exhaustive]` — new error variants
/// can be added in future versions without breaking downstream
/// `match` expressions.
#[derive(Clone, Debug)]
pub struct SchemaBuildError {
    kind: SchemaBuildErrorKind,
    notes: Vec<ErrorNote>,
    span: Span,
}

impl SchemaBuildError {
    pub fn kind(&self) -> &SchemaBuildErrorKind { &self.kind }
    pub fn notes(&self) -> &[ErrorNote] { &self.notes }
    pub fn span(&self) -> Span { self.span }

    /// Format this error with source snippets and caret
    /// underlines, resolving spans via the provided source maps.
    pub fn format_detailed(
        &self,
        source_maps: &[crate::schema_source_map::SchemaSourceMap],
    ) -> String {
        // Resolve primary span to line/col
        // Render primary message from kind
        // Render each note with optional secondary span
        todo!()
    }
}

impl std::fmt::Display for SchemaBuildError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for SchemaBuildError {}

/// Categorized error kind for programmatic matching.
///
/// `#[non_exhaustive]` — new variants may be added in minor
/// releases. Always include a wildcard arm in `match` expressions.
#[derive(Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaBuildErrorKind {
    #[error("duplicate directive definition `@{name}`")]
    DuplicateDirectiveDefinition {
        name: String,
    },

    #[error("duplicate enum value `{value_name}` on `{type_name}`")]
    DuplicateEnumValueDefinition {
        type_name: String,
        value_name: String,
    },

    #[error("duplicate field `{field_name}` on `{type_name}`")]
    DuplicateFieldNameDefinition {
        type_name: String,
        field_name: String,
    },

    // ... port remaining ~18 variants from v0, plus add:

    #[error("`{type_name}` has no fields")]
    EmptyObjectOrInterfaceType { type_name: String },

    #[error("union `{type_name}` has no members")]
    EmptyUnionType { type_name: String },

    #[error("enum value `{value_name}` on `{type_name}` must not be `true`, `false`, or `null`")]
    InvalidEnumValueName {
        type_name: String,
        value_name: String,
    },

    #[error("root {operation} type `{type_name}` must be an object type, found {actual_kind:?}")]
    RootOperationTypeNotObjectType {
        operation: String,
        type_name: String,
        actual_kind: crate::types::graphql_type_kind::GraphQLTypeKind,
    },

    #[error("root {operation} type `{type_name}` is not defined")]
    RootOperationTypeNotDefined {
        operation: String,
        type_name: String,
    },

    #[error("{0}")]
    TypeValidation(TypeValidationError),
}
```

Note: the primary `span` and `notes` (including secondary spans like "first defined here") live on the outer `SchemaBuildError` struct, NOT on each kind variant. Variants carry only identity data (names, kind discriminants). Validators construct notes at error-creation time when they have access to all relevant spans.

**`type_validation_error.rs`:** Same struct+kind pattern:
```rust
#[derive(Clone, Debug)]
pub struct TypeValidationError {
    kind: TypeValidationErrorKind,
    notes: Vec<ErrorNote>,
    span: Span,
}

#[derive(Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TypeValidationErrorKind {
    // ... port ~15 variants from v0, fixing input-type check
}
```

**`schema_errors.rs`:** Same as before but now wrapping the structured `SchemaBuildError`:
```rust
/// A collection of errors from
/// [`SchemaBuilder::build()`](crate::schema::SchemaBuilder::build).
///
/// Implements [`std::error::Error`] and
/// [`Display`](std::fmt::Display) for `?` propagation.
/// Implements [`IntoIterator`] for access to individual errors.
#[derive(Debug)]
pub struct SchemaErrors {
    errors: Vec<SchemaBuildError>,
}

impl SchemaErrors {
    pub(crate) fn new(errors: Vec<SchemaBuildError>) -> Self {
        debug_assert!(!errors.is_empty());
        Self { errors }
    }

    pub fn errors(&self) -> &[SchemaBuildError] { &self.errors }
    pub fn len(&self) -> usize { self.errors.len() }
    pub fn is_empty(&self) -> bool { self.errors.is_empty() }

    /// Format all errors with source snippets.
    pub fn format_detailed(
        &self,
        source_maps: &[crate::schema_source_map::SchemaSourceMap],
    ) -> String {
        self.errors.iter()
            .map(|e| e.format_detailed(source_maps))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

impl std::fmt::Display for SchemaErrors { /* ... */ }
impl std::error::Error for SchemaErrors {}
impl IntoIterator for SchemaErrors { /* ... */ }
```

Operation-level errors (`OperationBuildError`, `SelectionSetBuildError`, `FragmentBuildError`, etc.) follow the same struct+kind pattern.

- [ ] Implement `ErrorNote` + `ErrorNoteKind` in `error_note.rs`
- [ ] Implement `SchemaBuildError` struct + `SchemaBuildErrorKind` (~25 `#[non_exhaustive]` variants)
- [ ] Implement `TypeValidationError` struct + `TypeValidationErrorKind` (~15 variants, fixing v0's incomplete input-type checking)
- [ ] Implement `SchemaErrors` newtype with Error + Display + IntoIterator + format_detailed
- [ ] Ensure every validation error includes a `Spec` note with the relevant spec URL
- [ ] Commit: `[libgraphql-core-v1] Add structured error types with notes system`

---

### Task 14: SchemaBuilder Core

**Files:**
- Create: `crates/libgraphql-core-v1/src/schema/schema_builder.rs`

**`schema_builder.rs`:**
```rust
use crate::names::DirectiveName;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::schema::schema_build_error::SchemaBuildError;
use crate::schema::schema_errors::SchemaErrors;
use crate::schema_source_map::SchemaSourceMap;
use crate::span::Span;
use crate::span::SourceMapId;
use crate::types::GraphQLType;
use crate::types::directive_definition::DirectiveDefinition;
use crate::types::directive_definition_kind::DirectiveDefinitionKind;
use crate::types::directive_location_kind::DirectiveLocationKind;
use crate::types::parameter_definition::ParameterDefinition;
use crate::types::scalar_kind::ScalarKind;
use crate::types::scalar_type::ScalarType;
use crate::types::type_annotation::TypeAnnotation;
use crate::type_builders::object_type_builder::ObjectTypeBuilder;
use indexmap::IndexMap;

/// Accumulates GraphQL type definitions, directive definitions,
/// and schema metadata, then validates and produces an immutable
/// [`Schema`](crate::schema::Schema).
///
/// # Usage
///
/// ```rust
/// use libgraphql_core_v2::schema::SchemaBuilder;
///
/// let mut sb = SchemaBuilder::new();
/// sb.load_str("type Query { hello: String }")?;
/// let schema = sb.build()?;
/// ```
pub struct SchemaBuilder {
    directive_defs: IndexMap<DirectiveName, DirectiveDefinition>,
    errors: Vec<SchemaBuildError>,
    mutation_type_name: Option<(TypeName, Span)>,
    query_type_name: Option<(TypeName, Span)>,
    source_maps: Vec<SchemaSourceMap>,
    subscription_type_name: Option<(TypeName, Span)>,
    types: IndexMap<TypeName, GraphQLType>,
    // Type extensions deferred until build()
    // pending_extensions: Vec<...>,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        let mut builder = Self {
            directive_defs: IndexMap::new(),
            errors: vec![],
            mutation_type_name: None,
            query_type_name: None,
            source_maps: vec![SchemaSourceMap::builtin()],
            subscription_type_name: None,
            types: IndexMap::new(),
        };
        builder.seed_builtin_scalars();
        builder.seed_builtin_directives();
        builder
    }

    /// Register a type builder with the schema. Performs early
    /// validation (name checks, duplicate detection).
    pub fn absorb_type(
        &mut self,
        builder: impl Into<TypeBuilderKind>,
    ) -> Result<(), SchemaBuildError> {
        let kind: TypeBuilderKind = builder.into();
        let (name, span) = kind.name_and_span();

        // Reject __ prefix
        if name.as_str().starts_with("__") {
            return Err(SchemaBuildError::InvalidDunderPrefixedTypeName {
                type_name: name.to_string(),
                location: span,
            });
        }

        // Check duplicate
        if self.types.contains_key(&name) {
            return Err(SchemaBuildError::DuplicateTypeDefinition {
                type_name: name.to_string(),
                location: span,
            });
        }

        // Convert builder to GraphQLType and insert
        let graphql_type = kind.into_graphql_type()?;
        self.types.insert(name, graphql_type);
        Ok(())
    }

    /// Parse a string as a schema document and load all
    /// definitions.
    pub fn load_str(
        &mut self,
        source: &str,
    ) -> Result<(), Vec<SchemaBuildError>> {
        let parse_result =
            libgraphql_parser::parse_schema(source);
        self.load_parse_result(&parse_result, None)
    }

    /// Load from a parse result (bundles AST + source map).
    pub fn load_parse_result(
        &mut self,
        parse_result: &libgraphql_parser::ParseResult<
            '_,
            libgraphql_parser::ast::Document<'_>,
        >,
        file_path: Option<std::path::PathBuf>,
    ) -> Result<(), Vec<SchemaBuildError>> {
        // Register source map
        let source_text = parse_result
            .source_map()
            .source()
            .unwrap_or("");
        let source_map_id = SourceMapId(
            self.source_maps.len() as u16,
        );
        self.source_maps.push(
            SchemaSourceMap::from_source(source_text, file_path),
        );

        let mut errors = vec![];
        for def in &parse_result.ast().definitions {
            // Convert each definition to a builder and register
            // (uses from_ast() on each builder type)
            // ...
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    /// Convenience: parse a schema string and build in one step.
    pub fn build_from_str(
        source: &str,
    ) -> Result<crate::schema::Schema, SchemaErrors> {
        let mut sb = Self::new();
        sb.load_str(source).map_err(SchemaErrors::new)?;
        sb.build()
    }

    /// Convenience: build from a parse result in one step.
    pub fn build_from_parse_result(
        parse_result: &libgraphql_parser::ParseResult<
            '_,
            libgraphql_parser::ast::Document<'_>,
        >,
    ) -> Result<crate::schema::Schema, SchemaErrors> {
        let mut sb = Self::new();
        sb.load_parse_result(parse_result, None)
            .map_err(SchemaErrors::new)?;
        sb.build()
    }

    pub fn build(
        self,
    ) -> Result<crate::schema::Schema, SchemaErrors> {
        // See Task 16 for full implementation
        todo!()
    }

    fn seed_builtin_scalars(&mut self) {
        for (kind, name) in [
            (ScalarKind::Boolean, "Boolean"),
            (ScalarKind::Float, "Float"),
            (ScalarKind::ID, "ID"),
            (ScalarKind::Int, "Int"),
            (ScalarKind::String, "String"),
        ] {
            self.types.insert(
                TypeName::new(name),
                GraphQLType::Scalar(Box::new(ScalarType {
                    description: None,
                    directives: vec![],
                    kind,
                    name: TypeName::new(name),
                    span: Span::builtin(),
                })),
            );
        }
    }

    fn seed_builtin_directives(&mut self) {
        // @skip(if: Boolean!)
        self.directive_defs.insert(
            DirectiveName::new("skip"),
            DirectiveDefinition {
                description: None,
                is_repeatable: false,
                kind: DirectiveDefinitionKind::Skip,
                locations: vec![
                    DirectiveLocationKind::Field,
                    DirectiveLocationKind::FragmentSpread,
                    DirectiveLocationKind::InlineFragment,
                ],
                name: DirectiveName::new("skip"),
                parameters: IndexMap::from([(
                    FieldName::new("if"),
                    ParameterDefinition {
                        default_value: None,
                        description: None,
                        name: FieldName::new("if"),
                        span: Span::builtin(),
                        type_annotation: TypeAnnotation::named(
                            "Boolean", false,
                        ),
                    },
                )]),
                span: Span::builtin(),
            },
        );
        // @include, @deprecated, @specifiedBy follow same pattern
        // (see v0 /crates/libgraphql-core/src/types/directive.rs
        // for param definitions)
    }
}

/// Internal enum to accept any type builder via absorb_type().
/// Each builder implements Into<TypeBuilderKind>.
pub(crate) enum TypeBuilderKind {
    Object(ObjectTypeBuilder),
    // Interface(...), Union(...), Enum(...), Scalar(...),
    // InputObject(...), etc.
}
```

**Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Verifies a minimal schema with Query type builds.
    // https://spec.graphql.org/September2025/#sec-Root-Operation-Types
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn minimal_schema_from_str() {
        let mut sb = SchemaBuilder::new();
        sb.load_str(
            "type Query { hello: String }",
        ).unwrap();
        let schema = sb.build().unwrap();
        assert!(schema.query_type().is_some());
    }

    // Verifies built-in scalars are present after new().
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn builtin_scalars_seeded() {
        let mut sb = SchemaBuilder::new();
        sb.load_str("type Query { x: Int }").unwrap();
        let schema = sb.build().unwrap();
        for name in ["Boolean", "Float", "ID", "Int", "String"] {
            assert!(
                schema.get_type(&TypeName::new(name)).is_some(),
                "missing built-in scalar: {name}",
            );
        }
    }

    // Verifies duplicate type registration is rejected.
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn duplicate_type_rejected() {
        let mut sb = SchemaBuilder::new();
        let result = sb.load_str(
            "type Foo { x: Int }\ntype Foo { y: String }",
        );
        assert!(result.is_err());
    }

    // Verifies __-prefixed type names are rejected.
    // https://spec.graphql.org/September2025/#sec-Names.Reserved-Names
    // Written by Claude Code, reviewed by a human.
    #[test]
    fn dunder_prefix_rejected() {
        let mut sb = SchemaBuilder::new();
        let result = sb.load_str(
            "type __Bad { x: Int }\ntype Query { x: Int }",
        );
        assert!(result.is_err());
    }
}
```

- [ ] Implement `SchemaBuilder` with `new()`, `absorb_type()`, `load_str()`, `load_parse_result()`
- [ ] Implement `TypeBuilderKind` enum + `Into` impls for all builder types
- [ ] Implement built-in scalar and directive seeding
- [ ] Write registration, loading, and duplicate-rejection tests
- [ ] Commit: `[libgraphql-core-v1] Add SchemaBuilder with registration and loading`

---

### Task 15: Validators

**Files:** All files under `validators/`

Port all validators from v0, adapting to v1 types. Fix v0 bugs identified in the spec audit.

**`object_or_interface_type_validator.rs`:** Port from v0 (`/crates/libgraphql-core/src/types/object_or_interface_type_validator.rs`). Key change: generic over `T: HasFieldsAndInterfaces` instead of taking `ObjectOrInterfaceTypeData`. Uses `TypeName`/`FieldName` instead of `&str`. Validates per [IsValidImplementation](https://spec.graphql.org/September2025/#IsValidImplementation()):
- Implemented interfaces exist and are interface types
- Transitive interface implementation
- All interface fields present with compatible params and covariant return types
- Additional params must be optional

```rust
use crate::names::TypeName;
use crate::schema::type_validation_error::TypeValidationError;
use crate::types::GraphQLType;
use crate::types::has_fields_and_interfaces::HasFieldsAndInterfaces;
use indexmap::IndexMap;
use std::collections::HashSet;

pub(crate) struct ObjectOrInterfaceTypeValidator<'a, T: HasFieldsAndInterfaces> {
    errors: Vec<TypeValidationError>,
    type_: &'a T,
    types_map: &'a IndexMap<TypeName, GraphQLType>,
}

impl<'a, T: HasFieldsAndInterfaces> ObjectOrInterfaceTypeValidator<'a, T> {
    pub fn new(
        type_: &'a T,
        types_map: &'a IndexMap<TypeName, GraphQLType>,
    ) -> Self {
        Self { errors: vec![], type_, types_map }
    }

    pub fn validate(
        mut self,
        verified: &mut HashSet<TypeName>,
    ) -> Vec<TypeValidationError> {
        // Port logic from v0, adapted for:
        // - TypeName instead of &str
        // - HasFieldsAndInterfaces trait methods
        // - FieldDefinition instead of Field
        // - Span instead of SourceLocation
        // See v0 file for complete algorithm
        self.errors
    }
}
```

**`union_type_validator.rs`:** Port from v0. Add empty-union check.
```rust
pub(crate) struct UnionTypeValidator<'a> {
    errors: Vec<TypeValidationError>,
    type_: &'a UnionType,
    types_map: &'a IndexMap<TypeName, GraphQLType>,
}

impl<'a> UnionTypeValidator<'a> {
    pub fn validate(mut self) -> Vec<TypeValidationError> {
        if self.type_.members().is_empty() {
            // NEW: v0 missed this check
            self.errors.push(
                TypeValidationError::EmptyUnionType { /* ... */ },
            );
        }
        for member_name in self.type_.members() {
            let Some(member_type) = self.types_map.get(member_name)
                else {
                    self.errors.push(
                        TypeValidationError::UndefinedTypeName { /* ... */ },
                    );
                    continue;
                };
            if !matches!(member_type, GraphQLType::Object(_)) {
                self.errors.push(
                    TypeValidationError::InvalidUnionMemberTypeKind { /* ... */ },
                );
            }
        }
        self.errors
    }
}
```

**`input_object_type_validator.rs`:** Port from v0 (`/crates/libgraphql-core/src/types/input_object_type_validator.rs`). **Fix v0 bug:** use `!is_input_type()` instead of `as_object().is_some()` so Interface and Union types are also rejected as input field types.

**`directive_definition_validator.rs`:** **NEW** (entirely absent in v0):
```rust
/// Validates directive definitions per
/// [Type System Directives](https://spec.graphql.org/September2025/#sec-Type-System.Directives).
pub(crate) fn validate_directive_definitions(
    directive_defs: &IndexMap<DirectiveName, DirectiveDefinition>,
    types_map: &IndexMap<TypeName, GraphQLType>,
) -> Vec<TypeValidationError> {
    let mut errors = vec![];
    for (_, directive) in directive_defs {
        if directive.is_builtin() { continue; }
        for (param_name, param) in directive.parameters() {
            // Param names must not start with __
            if param_name.as_str().starts_with("__") {
                errors.push(/* ... */);
            }
            // Param types must be input types
            let type_name = param.type_annotation()
                .innermost_type_name();
            if let Some(graphql_type) = types_map.get(type_name) {
                if !graphql_type.is_input_type() {
                    errors.push(/* ... */);
                }
            }
        }
    }
    errors
}
```

**`type_reference_validator.rs`:** Validates all type annotations across all types resolve to defined types, output fields use output types, params use input types.

**Tests:** Comprehensive tests for each validator — at minimum:
- Interface impl with missing field -> error
- Interface impl with wrong param type -> error
- Interface impl with non-covariant return -> error
- Union with non-object member -> error
- Empty union -> error
- Input object with Object/Interface/Union field type -> error
- Input object circular non-nullable reference -> error
- __ prefixed directive arg -> error
- Undefined type reference -> error

- [ ] Port + fix `object_or_interface_type_validator.rs` (generic over `HasFieldsAndInterfaces`)
- [ ] Port + fix `union_type_validator.rs` (add empty union check)
- [ ] Port + fix `input_object_type_validator.rs` (use `!is_input_type()`)
- [ ] Implement new `directive_definition_validator.rs`
- [ ] Implement `type_reference_validator.rs`
- [ ] Write comprehensive validator tests (valid + invalid for each rule)
- [ ] Commit: `[libgraphql-core-v1] Add validators (object/interface, union, input, directive, type-ref)`

---

### Task 16: Schema Struct + SchemaBuilder::build()

**Files:**
- Create: `crates/libgraphql-core-v1/src/schema/schema.rs`
- Modify: `crates/libgraphql-core-v1/src/schema/schema_builder.rs` (add build())

**`schema.rs`:**
```rust
use crate::names::DirectiveName;
use crate::names::TypeName;
use crate::schema_source_map::SchemaSourceMap;
use crate::types::GraphQLType;
use crate::types::directive_definition::DirectiveDefinition;
use crate::types::enum_type::EnumType;
use crate::types::has_fields_and_interfaces::HasFieldsAndInterfaces;
use crate::types::input_object_type::InputObjectType;
use crate::types::interface_type::InterfaceType;
use crate::types::object_type::ObjectType;
use crate::types::scalar_type::ScalarType;
use crate::types::union_type::UnionType;
use indexmap::IndexMap;

/// A fully validated, immutable GraphQL schema.
///
/// Produced by [`SchemaBuilder::build()`](crate::schema::SchemaBuilder::build).
/// A `Schema` is guaranteed to satisfy all type-system validation
/// rules from the
/// [GraphQL specification](https://spec.graphql.org/September2025/).
///
/// All types, directives, and root operation type references have
/// been validated — it is not possible to obtain a `Schema` that
/// references undefined types or violates interface contracts.
///
/// `Schema` implements `serde::Serialize` and
/// `serde::Deserialize` for binary embedding by the
/// `libgraphql-macros` crate.
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Schema {
    pub(crate) directive_defs:
        IndexMap<DirectiveName, DirectiveDefinition>,
    pub(crate) mutation_type_name: Option<TypeName>,
    pub(crate) query_type_name: TypeName,
    pub(crate) source_maps: Vec<SchemaSourceMap>,
    pub(crate) subscription_type_name: Option<TypeName>,
    pub(crate) types: IndexMap<TypeName, GraphQLType>,
}

impl Schema {
    // ── Generic lookups ──

    pub fn get_type(
        &self,
        name: &TypeName,
    ) -> Option<&GraphQLType> {
        self.types.get(name)
    }

    pub fn get_directive(
        &self,
        name: &DirectiveName,
    ) -> Option<&DirectiveDefinition> {
        self.directive_defs.get(name)
    }

    // ── Typed lookups ──

    pub fn object_type(
        &self,
        name: &TypeName,
    ) -> Option<&ObjectType> {
        self.types.get(name).and_then(|t| t.as_object())
    }

    pub fn interface_type(
        &self,
        name: &TypeName,
    ) -> Option<&InterfaceType> {
        self.types.get(name).and_then(|t| t.as_interface())
    }

    pub fn enum_type(
        &self,
        name: &TypeName,
    ) -> Option<&EnumType> {
        self.types.get(name).and_then(|t| t.as_enum())
    }

    pub fn scalar_type(
        &self,
        name: &TypeName,
    ) -> Option<&ScalarType> {
        self.types.get(name).and_then(|t| t.as_scalar())
    }

    pub fn union_type(
        &self,
        name: &TypeName,
    ) -> Option<&UnionType> {
        self.types.get(name).and_then(|t| t.as_union())
    }

    pub fn input_object_type(
        &self,
        name: &TypeName,
    ) -> Option<&InputObjectType> {
        self.types.get(name).and_then(|t| t.as_input_object())
    }

    // ── Typed iterators ──

    pub fn object_types(
        &self,
    ) -> impl Iterator<Item = &ObjectType> {
        self.types.values().filter_map(|t| t.as_object())
    }

    pub fn interface_types(
        &self,
    ) -> impl Iterator<Item = &InterfaceType> {
        self.types.values().filter_map(|t| t.as_interface())
    }

    pub fn enum_types(
        &self,
    ) -> impl Iterator<Item = &EnumType> {
        self.types.values().filter_map(|t| t.as_enum())
    }

    /// All types that implement a given interface.
    pub fn types_implementing(
        &self,
        interface_name: &TypeName,
    ) -> impl Iterator<Item = &GraphQLType> {
        let name = interface_name.clone();
        self.types.values().filter(move |t| match t {
            GraphQLType::Object(obj) => {
                obj.interfaces().iter().any(|l| &l.value == &name)
            },
            GraphQLType::Interface(iface) => {
                iface.interfaces().iter().any(|l| &l.value == &name)
            },
            _ => false,
        })
    }

    // ── Root operation types ──

    /// The Query root operation type. Per the
    /// [spec](https://spec.graphql.org/September2025/#sec-Root-Operation-Types),
    /// all schemas must define a Query root type —
    /// `SchemaBuilder::build()` validates this.
    pub fn query_type(&self) -> &ObjectType {
        self.object_type(&self.query_type_name)
            .expect("validated at build time")
    }

    pub fn query_type_name(&self) -> &TypeName {
        &self.query_type_name
    }

    pub fn mutation_type(&self) -> Option<&ObjectType> {
        self.mutation_type_name
            .as_ref()
            .and_then(|n| self.object_type(n))
    }

    pub fn mutation_type_name(&self) -> Option<&TypeName> {
        self.mutation_type_name.as_ref()
    }

    pub fn subscription_type(&self) -> Option<&ObjectType> {
        self.subscription_type_name
            .as_ref()
            .and_then(|n| self.object_type(n))
    }

    pub fn subscription_type_name(&self) -> Option<&TypeName> {
        self.subscription_type_name.as_ref()
    }

    // ── Source map resolution ──

    pub fn source_maps(&self) -> &[SchemaSourceMap] {
        &self.source_maps
    }
}
```

**`build()` orchestration** (in `schema_builder.rs`):
```rust
pub fn build(mut self) -> Result<Schema, SchemaErrors> {
    // Apply pending type extensions
    // self.apply_pending_extensions();

    // Validate root operation types
    let query_type_name = self.resolve_root_query_type();
    self.validate_root_type_is_object(
        &query_type_name, "query",
    );
    if let Some((ref name, _)) = self.mutation_type_name {
        self.validate_root_type_is_object(name, "mutation");
    }
    if let Some((ref name, _)) = self.subscription_type_name {
        self.validate_root_type_is_object(
            name, "subscription",
        );
    }

    // Run cross-type validators
    let mut verified_ifaces = std::collections::HashSet::new();
    for (_, graphql_type) in &self.types {
        match graphql_type {
            GraphQLType::Object(obj) => {
                let v = ObjectOrInterfaceTypeValidator::new(
                    obj.as_ref(), &self.types,
                );
                self.errors.extend(
                    v.validate(&mut verified_ifaces)
                        .into_iter()
                        .map(SchemaBuildError::TypeValidation),
                );
            },
            GraphQLType::Interface(iface) => {
                let v = ObjectOrInterfaceTypeValidator::new(
                    iface.as_ref(), &self.types,
                );
                self.errors.extend(
                    v.validate(&mut verified_ifaces)
                        .into_iter()
                        .map(SchemaBuildError::TypeValidation),
                );
            },
            GraphQLType::Union(u) => {
                let v = UnionTypeValidator::new(u, &self.types);
                self.errors.extend(
                    v.validate()
                        .into_iter()
                        .map(SchemaBuildError::TypeValidation),
                );
            },
            GraphQLType::InputObject(io) => {
                let v = InputObjectTypeValidator::new(
                    io, &self.types,
                );
                self.errors.extend(
                    v.validate()
                        .into_iter()
                        .map(SchemaBuildError::TypeValidation),
                );
            },
            _ => {},
        }
    }

    // Run directive validator
    self.errors.extend(
        validate_directive_definitions(
            &self.directive_defs, &self.types,
        ).into_iter()
            .map(SchemaBuildError::TypeValidation),
    );

    if !self.errors.is_empty() {
        return Err(SchemaErrors::new(self.errors));
    }

    Ok(Schema {
        directive_defs: self.directive_defs,
        mutation_type_name: self.mutation_type_name
            .map(|(n, _)| n),
        query_type_name,
        source_maps: self.source_maps,
        subscription_type_name: self.subscription_type_name
            .map(|(n, _)| n),
        types: self.types,
    })
}
```

**Tests:** Full-pipeline: parse -> load -> build -> query results. Both valid and invalid.

- [ ] Implement `Schema` with typed query API and full rustdocs
- [ ] Implement `SchemaBuilder::build()` orchestrating all validators
- [ ] Write end-to-end schema building tests (valid schemas, invalid schemas with specific error assertions)
- [ ] Commit: `[libgraphql-core-v1] Add Schema struct and SchemaBuilder::build()`

---

### Task 17: Schema Test Suite

**Files:** Under `schema/tests/`

Port and expand v0 tests. Use v0's `.graphql` fixture files where applicable.

- [ ] Port v0 schema builder tests (valid schemas: simple, SWAPI, GitHub, etc.)
- [ ] Port v0 invalid schema tests (duplicates, __-prefix, enum no values, etc.)
- [ ] Add new tests for previously-missing validations (root types must be Object, empty types, enum value names, directive validation)
- [ ] Commit: `[libgraphql-core-v1] Add comprehensive schema test suite`

---

### Task 18: Operation Types

**Files:** All type files under `operation/`

**`operation_kind.rs`:**
```rust
/// The kind of GraphQL operation.
///
/// See [Operations](https://spec.graphql.org/September2025/#sec-Language.Operations).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum OperationKind {
    Mutation,
    Query,
    Subscription,
}
```

**`variable.rs`:**
```rust
use crate::names::VariableName;
use crate::span::Span;
use crate::types::type_annotation::TypeAnnotation;
use crate::value::Value;

/// A variable definition within an operation
/// (e.g. `$id: ID! = "default"`).
///
/// See [Variable Definitions](https://spec.graphql.org/September2025/#sec-Language.Variables).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Variable {
    pub(crate) default_value: Option<Value>,
    pub(crate) name: VariableName,
    pub(crate) span: Span,
    pub(crate) type_annotation: TypeAnnotation,
}

impl Variable {
    pub fn default_value(&self) -> Option<&Value> {
        self.default_value.as_ref()
    }
    pub fn name(&self) -> &VariableName { &self.name }
    pub fn span(&self) -> Span { self.span }
    pub fn type_annotation(&self) -> &TypeAnnotation {
        &self.type_annotation
    }
}
```

**`operation.rs`:**
```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::TypeName;
use crate::names::VariableName;
use crate::operation::operation_kind::OperationKind;
use crate::operation::selection_set::SelectionSet;
use crate::operation::variable::Variable;
use crate::schema::Schema;
use crate::span::Span;
use indexmap::IndexMap;

/// A validated GraphQL operation (query, mutation, or
/// subscription).
///
/// Unlike v0's separate `Query`/`Mutation`/`Subscription` types,
/// v2 uses a single struct with a
/// [`kind`](Self::kind) discriminator. Methods requiring schema
/// context take `&Schema` as a parameter rather than storing a
/// lifetime-bound reference.
///
/// See [Operations](https://spec.graphql.org/September2025/#sec-Language.Operations).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Operation {
    pub(crate) directives: Vec<DirectiveAnnotation>,
    pub(crate) kind: OperationKind,
    pub(crate) name: Option<String>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) span: Span,
    pub(crate) variables: IndexMap<VariableName, Variable>,
}

impl Operation {
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn kind(&self) -> OperationKind { self.kind }
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }
    pub fn span(&self) -> Span { self.span }
    pub fn variables(
        &self,
    ) -> &IndexMap<VariableName, Variable> {
        &self.variables
    }

    /// The name of this operation's root type in the schema.
    pub fn root_type_name(
        &self,
        schema: &Schema,
    ) -> &TypeName {
        match self.kind {
            OperationKind::Query => schema.query_type_name(),
            OperationKind::Mutation => schema
                .mutation_type_name()
                .expect("validated at build time"),
            OperationKind::Subscription => schema
                .subscription_type_name()
                .expect("validated at build time"),
        }
    }
}
```

**`selection.rs`:**
```rust
use crate::operation::field_selection::FieldSelection;
use crate::operation::fragment_spread::FragmentSpread;
use crate::operation::inline_fragment::InlineFragment;

/// A single selection within a selection set.
///
/// See [Selection Sets](https://spec.graphql.org/September2025/#sec-Selection-Sets).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum Selection {
    Field(FieldSelection),
    FragmentSpread(FragmentSpread),
    InlineFragment(InlineFragment),
}
```

**`field_selection.rs`:**
```rust
use crate::directive_annotation::DirectiveAnnotation;
use crate::names::FieldName;
use crate::names::TypeName;
use crate::operation::selection_set::SelectionSet;
use crate::span::Span;
use crate::value::Value;
use indexmap::IndexMap;

/// A field selection within an operation's selection set.
///
/// This is an *operation-level* field selection — a reference to
/// a field in a query/mutation/subscription. For the *schema-level*
/// field definition, see
/// [`FieldDefinition`](crate::types::FieldDefinition).
///
/// `FieldSelection` stores pre-resolved metadata
/// (`parent_type_name`, `field_return_type_name`,
/// `requires_selection_set`) that was validated at build time.
/// This enables common queries without needing `&Schema`.
///
/// See [Fields](https://spec.graphql.org/September2025/#sec-Language.Fields).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
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

impl FieldSelection {
    pub fn alias(&self) -> Option<&FieldName> {
        self.alias.as_ref()
    }
    pub fn arguments(&self) -> &IndexMap<FieldName, Value> {
        &self.arguments
    }
    pub fn directives(&self) -> &[DirectiveAnnotation] {
        &self.directives
    }
    pub fn field_name(&self) -> &FieldName { &self.field_name }
    pub fn field_return_type_name(&self) -> &TypeName {
        &self.field_return_type_name
    }
    pub fn parent_type_name(&self) -> &TypeName {
        &self.parent_type_name
    }
    pub fn requires_selection_set(&self) -> bool {
        self.requires_selection_set
    }
    /// The response key for this field (alias if present,
    /// otherwise field name).
    #[inline]
    pub fn response_key(&self) -> &FieldName {
        self.alias.as_ref().unwrap_or(&self.field_name)
    }
    pub fn selection_set(&self) -> Option<&SelectionSet> {
        self.selection_set.as_ref()
    }
    pub fn span(&self) -> Span { self.span }

    /// Look up the schema-level field definition for this
    /// selection.
    pub fn schema_field<'s>(
        &self,
        schema: &'s crate::schema::Schema,
    ) -> Option<&'s crate::types::FieldDefinition> {
        schema.object_type(&self.parent_type_name)
            .and_then(|obj| {
                crate::types::HasFieldsAndInterfaces::field(
                    obj, self.field_name.as_str(),
                )
            })
            .or_else(|| {
                schema.interface_type(&self.parent_type_name)
                    .and_then(|iface| {
                        crate::types::HasFieldsAndInterfaces::field(
                            iface, self.field_name.as_str(),
                        )
                    })
            })
    }
}
```

**`selection_set.rs`:**
```rust
use crate::operation::field_selection::FieldSelection;
use crate::operation::selection::Selection;
use crate::span::Span;

/// A set of selections within braces `{ ... }`.
///
/// See [Selection Sets](https://spec.graphql.org/September2025/#sec-Selection-Sets).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct SelectionSet {
    pub(crate) selections: Vec<Selection>,
    pub(crate) span: Span,
}

impl SelectionSet {
    #[inline]
    pub fn selections(&self) -> &[Selection] {
        &self.selections
    }

    pub fn span(&self) -> Span { self.span }

    /// Iterate over only the field selections (filtering out
    /// fragment spreads and inline fragments).
    pub fn field_selections(
        &self,
    ) -> impl Iterator<Item = &FieldSelection> {
        self.selections.iter().filter_map(|s| match s {
            Selection::Field(f) => Some(f),
            _ => None,
        })
    }
}
```

Remaining types (`FragmentSpread`, `InlineFragment`, `Fragment`, `FragmentRegistry`, `ExecutableDocument`) follow v0's patterns adapted for v1 — no lifetime params, `Span` instead of `SourceLocation`, `TypeName`/`FieldName`/`FragmentName` newtypes.

- [ ] Implement all operation types: `Operation`, `OperationKind`, `Variable`, `SelectionSet`, `Selection`, `FieldSelection`, `FragmentSpread`, `InlineFragment`, `Fragment`, `FragmentRegistry`, `ExecutableDocument`
- [ ] Write basic construction/accessor tests
- [ ] Commit: `[libgraphql-core-v1] Add operation types`

---

### Task 19: Operation Builders

**Files:** All builder files under `operation/`

**`operation_builder.rs`:**
```rust
use crate::names::VariableName;
use crate::operation::operation::Operation;
use crate::operation::operation_kind::OperationKind;
use crate::operation::variable::Variable;
use crate::schema::Schema;
use crate::span::Span;
use crate::span::SourceMapId;
use indexmap::IndexMap;
use libgraphql_parser::ast;

/// Builds a validated [`Operation`] from parser AST or
/// programmatic construction.
///
/// # From parser AST
///
/// ```ignore
/// let op = OperationBuilder::from_ast(
///     &schema, frag_reg.as_ref(), &ast_op, source_map_id,
/// )?.build()?;
/// ```
pub struct OperationBuilder<'s> {
    schema: &'s Schema,
    fragment_registry: Option<
        &'s crate::operation::fragment_registry::FragmentRegistry,
    >,
    kind: OperationKind,
    name: Option<String>,
    variables: IndexMap<VariableName, Variable>,
    directives: Vec<crate::directive_annotation::DirectiveAnnotation>,
    selection_set_builder: Option<SelectionSetBuilder<'s>>,
    span: Span,
}

impl<'s> OperationBuilder<'s> {
    pub fn from_ast(
        schema: &'s Schema,
        fragment_registry: Option<
            &'s crate::operation::fragment_registry::FragmentRegistry,
        >,
        ast_op: &ast::OperationDefinition<'_>,
        source_map_id: SourceMapId,
    ) -> Result<Self, OperationBuildError> {
        // Determine kind
        // Verify root type exists in schema
        // Parse variables (check types exist, no duplicates)
        // Parse directives
        // Build SelectionSet via SelectionSetBuilder
        todo!()
    }

    pub fn build(self) -> Result<Operation, OperationBuildError> {
        // Assemble Operation
        todo!()
    }
}
```

**Typed operation builders** (`query_operation_builder.rs`, `mutation_operation_builder.rs`, `subscription_operation_builder.rs`):

These are newtype wrappers around `OperationBuilder` that provide type-safe construction for a specific operation kind. They convey the operation kind at the Rust type level and enable kind-specific fail-fast validation.

```rust
/// Type-safe builder for
/// [query operations](https://spec.graphql.org/September2025/#sec-Language.Operations).
///
/// Wraps [`OperationBuilder`] with the kind pre-set to
/// [`OperationKind::Query`]. Since the spec requires every
/// schema to have a Query root type, `new()` is infallible
/// (given a valid `Schema`).
///
/// # Example
///
/// ```ignore
/// let op = QueryOperationBuilder::new(&schema)
///     .set_name("GetUser")
///     .build()?;
/// assert_eq!(op.kind(), OperationKind::Query);
/// ```
pub struct QueryOperationBuilder<'s>(OperationBuilder<'s>);

impl<'s> QueryOperationBuilder<'s> {
    pub fn new(schema: &'s Schema) -> Self {
        Self(OperationBuilder::new(
            schema, OperationKind::Query,
        ))
    }

    pub fn from_ast(
        schema: &'s Schema,
        fragment_registry: Option<&'s FragmentRegistry>,
        ast_op: &ast::OperationDefinition<'_>,
        source_map_id: SourceMapId,
    ) -> Result<Self, OperationBuildError> {
        let inner = OperationBuilder::from_ast(
            schema, fragment_registry, ast_op, source_map_id,
        )?;
        // Verify the AST operation is actually a query
        if inner.kind != OperationKind::Query {
            return Err(/* kind mismatch error */);
        }
        Ok(Self(inner))
    }

    // Delegates: set_name, add_variable, add_directive,
    // add_selection — all forwarded to self.0

    pub fn build(self) -> Result<Operation, OperationBuildError> {
        self.0.build()
    }
}

/// Type-safe builder for
/// [mutation operations](https://spec.graphql.org/September2025/#sec-Language.Operations).
///
/// Fails at `new()` if the schema has no Mutation root type.
pub struct MutationOperationBuilder<'s>(OperationBuilder<'s>);

impl<'s> MutationOperationBuilder<'s> {
    pub fn new(
        schema: &'s Schema,
    ) -> Result<Self, OperationBuildError> {
        if schema.mutation_type().is_none() {
            return Err(/* NoMutationTypeDefinedInSchema */);
        }
        Ok(Self(OperationBuilder::new(
            schema, OperationKind::Mutation,
        )))
    }

    // Delegates: same as QueryOperationBuilder

    pub fn build(self) -> Result<Operation, OperationBuildError> {
        self.0.build()
    }
}

/// Type-safe builder for
/// [subscription operations](https://spec.graphql.org/September2025/#sec-Subscription-Operation-Definitions).
///
/// Fails at `new()` if the schema has no Subscription root type.
/// Enforces the single-root-field constraint: `build()` verifies
/// the selection set contains exactly one root field.
pub struct SubscriptionOperationBuilder<'s>(OperationBuilder<'s>);

impl<'s> SubscriptionOperationBuilder<'s> {
    pub fn new(
        schema: &'s Schema,
    ) -> Result<Self, OperationBuildError> {
        if schema.subscription_type().is_none() {
            return Err(/* NoSubscriptionTypeDefinedInSchema */);
        }
        Ok(Self(OperationBuilder::new(
            schema, OperationKind::Subscription,
        )))
    }

    // Delegates: same as QueryOperationBuilder

    pub fn build(
        self,
    ) -> Result<Operation, OperationBuildError> {
        // Verify single root field constraint per
        // https://spec.graphql.org/September2025/#sec-Single-root-field
        // before delegating to inner build()
        self.0.validate_subscription_single_root_field()?;
        self.0.build()
    }
}
```

The generic `OperationBuilder` remains for cases where the operation kind is determined at runtime (e.g., `from_ast()` on an `OperationDefinition` where the kind comes from the AST). The typed builders are the preferred API for programmatic construction where the kind is known statically.

**`selection_set_builder.rs`:** The core validation engine for operation building. Port from v0 (`/crates/libgraphql-core/src/operation/selection_set_builder.rs`), fixing bugs and adding missing validations:

```rust
pub(crate) struct SelectionSetBuilder<'s> {
    schema: &'s Schema,
    fragment_registry: Option<
        &'s crate::operation::fragment_registry::FragmentRegistry,
    >,
    parent_type: &'s crate::types::GraphQLType,
    selections: Vec<Selection>,
    errors: Vec<SelectionSetBuildError>,
}
```

Key validations during `from_ast()`:
- **Field existence:** Selected fields must exist on parent type. **NEW:** `__typename` must be selectable on all composite types including unions (v0 bug: rejected unions entirely)
- **Leaf/composite sub-selection:** Leaf type fields must NOT have sub-selections; composite type fields MUST have sub-selections (both missing in v0)
- **Argument validation:** Arguments must correspond to field definition, required args must be present (missing in v0)
- **Pre-resolution:** For each field selection, store `parent_type_name`, `field_return_type_name`, `requires_selection_set` on the `FieldSelection`
- **Recursive:** Nested selection sets validated recursively

**`fragment_registry_builder.rs`:** Port from v0 (`/crates/libgraphql-core/src/operation/fragment_registry_builder.rs`):
- Fragment cycle detection via DFS with phase-normalization for deduplication
- Undefined fragment reference validation
- Duplicate fragment name rejection
- **NEW:** Fragment type condition must be a composite type

**Operation error types** (each in own file): `OperationBuildError`, `SelectionSetBuildError`, `FragmentBuildError`, `FragmentRegistryBuildError`, `ExecutableDocumentBuildError`. Port variants from v0, add new variants for previously-missing validations.

**Tests:**
```rust
// Verifies a simple query builds via typed builder.
// Written by Claude Code, reviewed by a human.
#[test]
fn simple_query_via_typed_builder() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { hello: String }",
    ).unwrap();
    let mut qb = QueryOperationBuilder::new(&schema);
    // ... add selections
    let op = qb.build().unwrap();
    assert_eq!(op.kind(), OperationKind::Query);
}

// Verifies MutationOperationBuilder fails if schema has
// no mutation type.
// Written by Claude Code, reviewed by a human.
#[test]
fn mutation_builder_fails_without_mutation_type() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { hello: String }",
    ).unwrap();
    let err = MutationOperationBuilder::new(&schema)
        .unwrap_err();
    // assert kind is NoMutationTypeDefinedInSchema
}

// Verifies subscription enforces single root field.
// https://spec.graphql.org/September2025/#sec-Single-root-field
// Written by Claude Code, reviewed by a human.
#[test]
fn subscription_rejects_multiple_root_fields() {
    // ... schema with subscription type
    // ... build subscription with 2 root fields
    // ... assert error
}

// Verifies a simple query builds via generic builder.
// Written by Claude Code, reviewed by a human.
#[test]
fn simple_query_from_ast() {
    let schema = SchemaBuilder::build_from_str(
        "type Query { hello: String }",
    ).unwrap();

    let parse = libgraphql_parser::parse_executable(
        "query { hello }",
    );
    let op = OperationBuilder::from_ast(
        &schema, None,
        /* extract OperationDefinition from parse result */
        SourceMapId(1),
    ).unwrap().build().unwrap();
    assert_eq!(op.kind(), OperationKind::Query);
}

// Verifies selecting a non-existent field is rejected.
// https://spec.graphql.org/September2025/#sec-Field-Selections
// Written by Claude Code, reviewed by a human.
#[test]
fn undefined_field_rejected() {
    // ... build schema, try to select nonexistent field,
    // assert specific error
}

// Verifies leaf fields reject sub-selections.
// https://spec.graphql.org/September2025/#sec-Leaf-Field-Selections
// Written by Claude Code, reviewed by a human.
#[test]
fn leaf_field_rejects_subselection() {
    // type Query { name: String }
    // query { name { x } }  -> error
}

// Verifies subscription single root field constraint.
// https://spec.graphql.org/September2025/#sec-Single-root-field
// Written by Claude Code, reviewed by a human.
#[test]
fn subscription_multiple_root_fields_rejected() {
    // subscription { a, b }  -> error
}
```

- [ ] Implement `OperationBuilder` (generic) with `from_ast()` and `build()`
- [ ] Implement `QueryOperationBuilder`, `MutationOperationBuilder`, `SubscriptionOperationBuilder` (newtype wrappers)
- [ ] Implement `SelectionSetBuilder` with all field/selection validations
- [ ] Implement `FragmentRegistryBuilder` with cycle detection
- [ ] Implement `ExecutableDocumentBuilder`
- [ ] Implement all operation error types (each in own file)
- [ ] Write operation building tests (valid + invalid)
- [ ] Commit: `[libgraphql-core-v1] Add operation builders`

---

### Task 20: Operation Test Suite

- [ ] Port v0 operation tests
- [ ] Add tests for previously-missing validations (subscription root field, leaf/composite, argument validation, variable type validation)
- [ ] Commit: `[libgraphql-core-v1] Add comprehensive operation test suite`

---

### Task 21: Macro Runtime + Serde/Bincode

**Files:**
- Create: `crates/libgraphql-core-v1/src/schema/_macro_runtime.rs`

- [ ] Implement `build_from_macro_serialized()`
- [ ] Write bincode round-trip test (build schema from string -> serialize -> deserialize -> verify equality)
- [ ] Test with realistic schemas
- [ ] Commit: `[libgraphql-core-v1] Add macro runtime and serde/bincode support`

---

### Addendum: libgraphql-parser Project Tracker Item

As part of this plan's execution, add the following item to `libgraphql-parser`'s `project-tracker.md`:

> **Investigate adding `serde::Serialize`/`serde::Deserialize` to all AST nodes.** `libgraphql-core-v1` re-exports `DirectiveLocationKind` and needs it serializable for Schema bincode embedding. Investigate making all parser AST nodes serde-serializable — this would enable AST caching, serialized test fixtures, and cross-process AST transfer. Consider gating behind a `serde` feature flag to avoid adding the dependency for consumers who don't need it.

---

## Validation Coverage Checklist

### During `absorb_type()` (early, structural):
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

| v0 File | What to port |
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

## `#[inline]` Placement Guide

Only the following 18 functions should receive `#[inline]`. All others should be left to rustc's heuristics. These were selected for being trivial (1-2 expressions), frequently called in hot paths, and benefiting from cross-crate inlining.

**Tier 1 — Definitely inline (15):**
- All 6 name newtypes: `as_str(&self) -> &str`
- `Span::builtin() -> Self`
- `NamedTypeAnnotation::nullable()`, `type_name()`, `span()`
- `ListTypeAnnotation::inner()`, `nullable()`, `span()`
- `DirectiveAnnotation::name()`, `span()`
- `DeprecationState::is_deprecated()`
- `FieldSelection::response_key()`
- `SelectionSet::selections()`

**Tier 2 — Borderline but worth it (3):**
- `TypeAnnotation::nullable()`, `span()` (2-arm match returning Copy/field)
- `GraphQLType::name()` (6-arm match, each arm a single field ref — hottest accessor in traversal)

**Everything else: NO `#[inline]`.** In particular: constructors (`new()`), recursive functions (`innermost_named()`), methods with non-trivial logic (`resolve_offset()`, `is_subtype_of()`), validation-path-only accessors (`description()`, `directives()`, `is_builtin()`), and schema-lookup chains (`schema_field()`).

---

## Code Style Reminders

- All lines ≤ 100 columns (per CLAUDE.md)
- One `use` per line, alphabetically sorted, `crate`-rooted (no `super`)
- All `match` arms end with comma
- Enum variants alphabetically sorted
- Opening `{` never on its own line
- No "Step N:" style comments
- Thorough rustdoc on all public items, matching `libgraphql-parser` quality
- **Every semantic type** (`ObjectType`, `InterfaceType`, `UnionType`, `EnumType`, `ScalarType`, `InputObjectType`, `FieldDefinition`, `ParameterDefinition`, `EnumValue`, `DirectiveDefinition`, `Operation`, `SelectionSet`, `FieldSelection`, `Fragment`, etc.) must include a `[link](https://spec.graphql.org/September2025/#...)` to the relevant section of the September 2025 GraphQL spec in its rustdoc
- Tests include English description + spec link + "Written by Claude Code, reviewed by a human"
- **All validation logic must have significant unit test coverage.** Every validator, every error path, every boundary condition. The only code exempt from this is trivially simple code that wouldn't benefit from testing (e.g., a function that just returns a field).
- Validators that validate a `*Type` are named `*TypeValidator` (e.g., `ObjectOrInterfaceTypeValidator`, `UnionTypeValidator`, `InputObjectTypeValidator`)
