# Plan: Migrate libgraphql-core from graphql-parser to libgraphql-parser

## Context

`libgraphql-core` depends on `graphql-parser` v0.4 for parsing and AST types. The
project has its own parser (`libgraphql-parser`) that is spec-compliant (September
2025), supports error recovery, and has richer source tracking. This migration removes
`graphql-parser` entirely and uses `libgraphql-parser` AST types **directly** — no
intermediate wrapper layer.

The key challenge is that `graphql-parser` provides `'static` owned types (via
`.into_static()`), while `libgraphql-parser` uses `'src` lifetimes borrowing from
source text. However, analysis shows this is workable because:

1. **Operation builders** store zero AST nodes — they consume immediately in `from_ast()`
2. **Schema type builders** only store AST nodes for **deferred type extensions** (when
   a type hasn't been defined yet). All other data is extracted into owned types
   immediately.
3. The `'src` lifetime only needs to live within individual method scopes (e.g.,
   `load_str()`), NOT on builder structs.

## Architecture Decision: Direct AST Consumption

**Approach**: Use `libgraphql_parser::ast::*` types directly in all builder methods.
Delete `ast.rs`. Thread `&SourceMap` into processing functions for position resolution.
For deferred type extensions, extract owned data at storage time instead of storing raw
AST nodes.

**Why**: No wrapper layer to maintain. Direct use of libgraphql-parser's richer type
system (flat nullability, structured arguments, ByteSpan positions). Cleaner long-term
architecture.

---

## Phase 1: Dependency and Module Prep

### Changes:
- `crates/libgraphql-core/Cargo.toml`:
  - Make `libgraphql-parser` a required (non-optional) dependency
  - Remove `use-libgraphql-parser` feature flag
  - Keep `graphql-parser` temporarily (removed in final phase)
- Delete `src/ast.rs`
- Remove `pub mod ast;` from `src/lib.rs` (or equivalent)
- Update all `use crate::ast` imports throughout the crate — these will be replaced
  with direct `use libgraphql_parser::ast::*` imports in subsequent phases

### Verification:
This phase will NOT compile. It establishes the dependency change and forces all
downstream code to be updated in subsequent phases.

---

## Phase 2: Update `loc.rs` — Position Resolution

### Current state:
`loc.rs` uses `ast::AstPos` (= `graphql_parser::Pos { line, column }`, 1-based).
Three methods access it: `from_execdoc_ast_position()`,
`from_schema_ast_position()`, `with_ast_position()`.

### New approach:
Replace `AstPos`-based helpers with `ByteSpan` + `SourceMap`-based helpers.

```rust
use libgraphql_parser::ByteSpan;
use libgraphql_parser::SourceMap;

impl SourceLocation {
    pub(crate) fn from_schema_span(
        file_path: Option<&Path>,
        span: ByteSpan,
        source_map: &SourceMap,
    ) -> Self {
        if let Some(file_path) = file_path {
            let (line, col) = source_map.resolve_offset(span.start)
                .map(|pos| (pos.line() + 1, pos.col_utf8() + 1))
                .unwrap_or((0, 0));
            Self::SchemaFile(FilePosition {
                col,
                file: Box::new(file_path.to_path_buf()),
                line,
            })
        } else {
            Self::Schema
        }
    }

    pub(crate) fn from_execdoc_span(
        file_path: Option<&Path>,
        span: ByteSpan,
        source_map: &SourceMap,
    ) -> Self {
        // Similar to above but returns ExecutableDocumentFile/ExecutableDocument
    }

    // Keep `with_ast_position` but rename to `with_span`:
    pub(crate) fn with_span(
        &self,
        span: ByteSpan,
        source_map: &SourceMap,
    ) -> Self { ... }
}
```

### Files:
- `crates/libgraphql-core/src/loc.rs`

---

## Phase 3: Update `value.rs` — Value Conversion

### Current state:
`Value::from_ast()` matches on `ast::Value` variants (`Variable(String)`,
`Int(Number)`, `Float(f64)`, etc.) where `ast::Value = graphql_parser::query::Value`.

### New approach:
Match on `libgraphql_parser::ast::Value<'src>` variants directly. Key differences:

| Old (`graphql_parser`) | New (`libgraphql_parser`) |
|---|---|
| `Value::Variable(String)` | `Value::Variable(VariableReference { name: Name, .. })` |
| `Value::Int(Number)` — `num.as_i64()` | `Value::Int(IntValue { value: i32, .. })` |
| `Value::Float(f64)` | `Value::Float(FloatValue { value: f64, .. })` |
| `Value::String(String)` | `Value::String(StringValue { value: Cow<'src, str>, .. })` |
| `Value::Boolean(bool)` | `Value::Boolean(BooleanValue { value: bool, .. })` |
| `Value::Null` | `Value::Null(NullValue { .. })` |
| `Value::Enum(String)` | `Value::Enum(EnumValue { value: Cow<'src, str>, .. })` |
| `Value::List(Vec<Value>)` | `Value::List(ListValue { values: Vec<Value>, .. })` |
| `Value::Object(BTreeMap<String, Value>)` | `Value::Object(ObjectValue { fields: Vec<ObjectField>, .. })` |

### Changes to `Value::from_ast()`:
```rust
pub(crate) fn from_ast(
    ast_value: &libgraphql_parser::ast::Value<'_>,
    position: &loc::SourceLocation,
) -> Self {
    match ast_value {
        lgp::Value::Variable(var_ref) =>
            Value::VarRef(Variable::named_ref(&var_ref.name.value, position.to_owned())),
        lgp::Value::Int(int_val) =>
            Value::Int(int_val.value as i64),
        lgp::Value::Float(float_val) =>
            Value::Float(float_val.value),
        lgp::Value::String(str_val) =>
            Value::String(str_val.value.to_string()),
        // ... etc
    }
}
```

### Also simplify `Number` / serde:
Replace `ast::Number` (opaque type with `.as_i64()`) with a plain `i64` in the `Value::Int`
variant. Update the serde adapter accordingly — the `SerdeNumber` remote derive can be
dropped in favor of direct `i64` serialization.

### Files:
- `crates/libgraphql-core/src/value.rs`

---

## Phase 4: Update `TypeAnnotation::from_ast_type()` — Flat Nullability

### Current state:
`type_annotation.rs` matches on recursive `ast::operation::Type` enum:
```rust
match ast_type {
    Type::ListType(inner) => ...,
    Type::NamedType(name) => ...,
    Type::NonNullType(inner) => Self::from_ast_type_impl(location, inner, false),
}
```

### New approach:
Match on `libgraphql_parser::ast::TypeAnnotation` with flat nullability:

```rust
pub(crate) fn from_ast_type(
    src_loc: &loc::SourceLocation,
    ast_type: &libgraphql_parser::ast::TypeAnnotation<'_>,
) -> Self {
    let nullable = ast_type.nullable();
    match ast_type {
        libgraphql_parser::ast::TypeAnnotation::List(list) =>
            Self::List(ListTypeAnnotation {
                inner_type_ref: Box::new(Self::from_ast_type(src_loc, &list.element_type)),
                nullable,
                ref_location: src_loc.to_owned(),
            }),
        libgraphql_parser::ast::TypeAnnotation::Named(named) =>
            Self::Named(NamedTypeAnnotation {
                nullable,
                type_ref: NamedGraphQLTypeRef::new(
                    named.name.value.as_ref(),
                    src_loc.clone(),
                ),
            }),
    }
}
```

This is actually simpler than the current recursive `NonNullType` unwrapping — the
`from_ast_type_impl` helper and `nullable` parameter threading can be removed.

### Files:
- `crates/libgraphql-core/src/types/type_annotation.rs`

---

## Phase 5: Update `DirectiveAnnotationBuilder`

### Current state:
Takes `&[ast::operation::Directive]` where `Directive` has
`arguments: Vec<(String, Value)>` (tuple pairs).

### New approach:
Takes `&[libgraphql_parser::ast::DirectiveAnnotation<'_>]` where arguments are
`Vec<Argument>` with `Argument { name: Name, value: Value }`:

```rust
pub fn from_ast(
    annotated_item_srcloc: &loc::SourceLocation,
    directives: &[libgraphql_parser::ast::DirectiveAnnotation<'_>],
    source_map: &libgraphql_parser::SourceMap<'_>,
) -> Vec<DirectiveAnnotation> {
    directives.iter().map(|ast_annot| {
        let annot_srcloc = annotated_item_srcloc.with_span(ast_annot.span, source_map);
        let mut arguments = IndexMap::new();
        for arg in &ast_annot.arguments {
            arguments.insert(
                arg.name.value.to_string(),
                Value::from_ast(&arg.value, &annot_srcloc),
            );
        }
        DirectiveAnnotation {
            arguments,
            directive_ref: NamedDirectiveRef::new(
                ast_annot.name.value.as_ref(),
                annot_srcloc,
            ),
        }
    }).collect()
}
```

### Files:
- `crates/libgraphql-core/src/directive_annotation_builder.rs`

---

## Phase 6: Update `Parameter::from_ast()`

### Current state:
Takes `&ast::schema::InputValue` with fields: `name`, `position`, `value_type`,
`default_value`, `directives`, `description`.

### New approach:
Takes `&libgraphql_parser::ast::InputValueDefinition<'_>` + `&SourceMap`. Key field
name differences:

| Old field | New field |
|---|---|
| `param.position` | `param.span` (ByteSpan, needs SourceMap) |
| `param.name` (String) | `param.name.value` (Cow<'src, str>) |
| `param.value_type` | `param.value_type` (TypeAnnotation, same name) |
| `param.default_value` | `param.default_value` (Option<Value>) |
| `param.directives` | `param.directives` (Vec<DirectiveAnnotation>) |

### Files:
- `crates/libgraphql-core/src/types/parameter.rs`

---

## Phase 7: Update Schema Type Builders + TypeBuilder Trait

This is the largest phase. It updates the `TypeBuilder` trait and all 6 concrete
builders to use `libgraphql-parser` AST types directly.

### 7a: Update `TypeBuilder` trait (GATs)

```rust
use libgraphql_parser::SourceMap;

pub trait TypeBuilder: Sized {
    type AstTypeDef<'src>;
    type AstTypeExtension<'src>;

    fn finalize(self, types_map_builder: &mut TypesMapBuilder) -> Result<()>;

    fn visit_type_def<'src>(
        &mut self,
        types_map_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        source_map: &SourceMap<'src>,
        def: &Self::AstTypeDef<'src>,
    ) -> Result<()>;

    fn visit_type_extension<'src>(
        &mut self,
        types_map_builder: &mut TypesMapBuilder,
        file_path: Option<&Path>,
        source_map: &SourceMap<'src>,
        def: &Self::AstTypeExtension<'src>,
    ) -> Result<()>;
}
```

Key changes:
- `&SourceMap<'src>` parameter added to `visit_type_def` and `visit_type_extension`
- `visit_type_extension` takes `&Self::AstTypeExtension<'src>` (reference, not value)
- Associated types use GATs: `type AstTypeDef<'src>; type AstTypeExtension<'src>;`

### 7b: Update `TypeBuilderHelpers`

`object_fielddefs_from_ast()` and `inputobject_fields_from_ast()` change to accept
`libgraphql_parser` types:
- `fields: &[libgraphql_parser::ast::FieldDefinition<'_>]` (was `&[ast::schema::Field]`)
- `input_fields: &[libgraphql_parser::ast::InputValueDefinition<'_>]` (was `&[ast::schema::InputValue]`)
- Add `source_map: &SourceMap<'_>` parameter

Field access changes:
| Old | New |
|---|---|
| `field.name` (String) | `field.name.value` (Cow) |
| `field.position` (Pos) | `field.span` (ByteSpan) |
| `field.description` (Option<String>) | `field.description` (Option<StringValue>) → `.as_ref().map(\|d\| d.value.to_string())` |
| `field.field_type` (Type) | `field.field_type` (TypeAnnotation) |
| `field.arguments` (Vec<InputValue>) | `field.parameters` (Vec<InputValueDefinition>) |
| `field.directives` (Vec<Directive>) | `field.directives` (Vec<DirectiveAnnotation>) |

### 7c: Deferred Extension Storage

Each type builder that stores extensions needs a **pending extension struct** to hold
pre-extracted owned data. Example for `ObjectTypeBuilder`:

**Current** (stores raw AST):
```rust
extensions: Vec<(Option<PathBuf>, ast::schema::ObjectTypeExtension)>,
```

**New** (stores pre-extracted owned data):
```rust
struct PendingObjectExtension {
    file_path: Option<PathBuf>,
    ext_srcloc: loc::SourceLocation,
    name: String,
    fields: IndexMap<String, Field>,     // already-built owned Fields
    directives: Vec<DirectiveAnnotation>, // already-built
}

extensions: Vec<PendingObjectExtension>,
```

In `visit_type_extension()`:
- If type exists → apply extension immediately (call `merge_type_extension` with
  owned data, same as today)
- If type doesn't exist → extract all data NOW (build `Field`s, `DirectiveAnnotation`s,
  etc.), store in `PendingObjectExtension`

In `finalize()`:
- Process `PendingObjectExtension` entries — insert pre-built fields/directives
  into the type (no AST access needed)

**Same pattern for all 6 type builders:**

| Builder | Pending Extension Struct | Key owned data to extract |
|---|---|---|
| `ObjectTypeBuilder` | `PendingObjectExtension` | `name`, `ext_srcloc`, `fields: IndexMap<String, Field>`, `directives` |
| `InterfaceTypeBuilder` | `PendingInterfaceExtension` | Same as Object |
| `EnumTypeBuilder` | `PendingEnumExtension` | `name`, `ext_srcloc`, `values: IndexMap<String, EnumValue>`, `directives` |
| `UnionTypeBuilder` | `PendingUnionExtension` | `name`, `ext_srcloc`, `members: Vec<String>`, `directives` |
| `InputObjectTypeBuilder` | `PendingInputObjectExtension` | `name`, `ext_srcloc`, `fields: IndexMap<String, InputField>`, `directives` |
| `ScalarTypeBuilder` | `PendingScalarExtension` | `name`, `ext_srcloc`, `directives` |

Note: `merge_type_extension()` currently accepts `&ast::schema::*Extension` and
extracts data. With the new approach, the "immediate merge" path can reuse the same
extraction logic, just applied to `libgraphql_parser` AST types directly.
`finalize()` changes to merge from the pending structs (pre-extracted data) instead
of from AST nodes.

### Files:
- `crates/libgraphql-core/src/types/type_builder.rs`
- `crates/libgraphql-core/src/types/object_type_builder.rs`
- `crates/libgraphql-core/src/types/interface_type_builder.rs`
- `crates/libgraphql-core/src/types/enum_type_builder.rs`
- `crates/libgraphql-core/src/types/union_type_builder.rs`
- `crates/libgraphql-core/src/types/input_object_type_builder.rs`
- `crates/libgraphql-core/src/types/scalar_type_builder.rs`

---

## Phase 8: Update `SchemaBuilder`

### 8a: Update `load_str()`
```rust
pub fn load_str(
    mut self,
    file_path: Option<&Path>,
    content: impl AsRef<str>,
) -> Result<Self> {
    let result = libgraphql_parser::parse_schema(content.as_ref());
    if result.has_errors() {
        return Err(SchemaBuildError::ParseError {
            file: file_path.map(|p| p.to_path_buf()),
            err: result.formatted_errors(),
        });
    }
    let source_map = result.source_map();
    for def in result.definitions() {
        self.visit_ast_def(file_path, def, source_map)?;
    }
    Ok(self)
}
```

### 8b: Update `load_ast()` (public API change)
`load_ast()` currently takes `ast::schema::Document` (owned graphql-parser doc). Two
options:
1. **Remove it** — it's a public API that exposes internal AST types. With the parser
   migration, external callers should use `load_str()` instead.
2. **Change signature** to take `&libgraphql_parser::ast::Document<'_>` +
   `&SourceMap<'_>`.

Recommendation: Option 2 for flexibility, but expose via `libgraphql_parser` types
(these are now a public dependency).

### 8c: Update `visit_ast_def()`, `visit_ast_type_def()`, `visit_ast_type_extension()`
```rust
fn visit_ast_def<'src>(
    &mut self,
    file_path: Option<&Path>,
    def: &libgraphql_parser::ast::Definition<'src>,
    source_map: &libgraphql_parser::SourceMap<'src>,
) -> Result<()> {
    use libgraphql_parser::ast::Definition;
    match def {
        Definition::SchemaDefinition(schema_def) =>
            self.visit_ast_schemablock_def(file_path, schema_def, source_map),
        Definition::TypeDefinition(type_def) =>
            self.visit_ast_type_def(file_path, type_def, source_map),
        Definition::TypeExtension(type_ext) =>
            self.visit_ast_type_extension(file_path, type_ext, source_map),
        Definition::DirectiveDefinition(directive_def) =>
            self.visit_ast_directive_def(file_path, directive_def, source_map),
        // Executable definitions in a schema document → ignore or error
        Definition::OperationDefinition(_) | Definition::FragmentDefinition(_) =>
            Ok(()), // or return an error
        Definition::SchemaExtension(_) =>
            Ok(()), // handle or error as appropriate
    }
}
```

### 8d: Update `visit_ast_schemablock_def()`
`libgraphql_parser::ast::SchemaDefinition` has `root_operations: Vec<RootOperationTypeDefinition>`
instead of separate `query/mutation/subscription: Option<String>` fields. Iterate
root operations and match on `operation_kind`.

### 8e: Update `visit_ast_directive_def()`
Takes `&libgraphql_parser::ast::DirectiveDefinition<'src>`. Field access:
- `def.name.value` (was `def.name`)
- `def.span` + source_map (was `def.position`)
- `def.description.as_ref().map(|d| d.value.to_string())` (was `def.description`)
- `def.arguments` → `Vec<InputValueDefinition>` (was `def.arguments` → `Vec<InputValue>`)

### 8f: Update `visit_ast_type_def()` and `visit_ast_type_extension()`
The `TypeDefinition` enum variants match 1:1 but with different names:
| Old | New |
|---|---|
| `TypeDefinition::Enum(EnumType)` | `TypeDefinition::Enum(EnumTypeDefinition)` |
| `TypeDefinition::InputObject(InputObjectType)` | `TypeDefinition::Input(InputObjectTypeDefinition)` |
| `TypeDefinition::Interface(InterfaceType)` | `TypeDefinition::Interface(InterfaceTypeDefinition)` |
| `TypeDefinition::Object(ObjectType)` | `TypeDefinition::Object(ObjectTypeDefinition)` |
| `TypeDefinition::Scalar(ScalarType)` | `TypeDefinition::Scalar(ScalarTypeDefinition)` |
| `TypeDefinition::Union(UnionType)` | `TypeDefinition::Union(UnionTypeDefinition)` |

Same for `TypeExtension` variants.

Pass `source_map` through to type builder methods.

### Files:
- `crates/libgraphql-core/src/schema/schema_builder.rs`

---

## Phase 9: Update Operation Builders

These are straightforward since they never store AST nodes. Each `from_ast()` method
changes its parameter types and field access patterns.

### 9a: `OperationBuilder::from_ast()`
**Major structural change**: `OperationDefinition` changes from enum
(`SelectionSet`/`Query`/`Mutation`/`Subscription`) to struct with
`operation_kind: OperationKind` + `shorthand: bool`.

```rust
pub fn from_ast(
    schema: &'schema Schema,
    fragment_registry: &'fragreg FragmentRegistry<'schema>,
    ast: &libgraphql_parser::ast::OperationDefinition<'_>,
    source_map: &libgraphql_parser::SourceMap<'_>,
    file_path: Option<&Path>,
) -> Result<Self> {
    let op_kind = match ast.operation_kind {
        lgp::OperationKind::Query => OperationKind::Query,
        lgp::OperationKind::Mutation => OperationKind::Mutation,
        lgp::OperationKind::Subscription => OperationKind::Subscription,
    };
    let op_type = match op_kind {
        OperationKind::Query => schema.query_type(),
        OperationKind::Mutation => schema.mutation_type().ok_or(...)?,
        OperationKind::Subscription => schema.subscription_type().ok_or(...)?,
    };
    // Extract name, directives, variables, selection_set from ast fields
    // (all fields are directly on the struct, not spread across variants)
}
```

The `LoadFromAstDetails` intermediate struct can be simplified or removed since all
fields come from one struct.

### 9b: `SelectionSetBuilder::from_ast()`
Selection variants change:
| Old | New |
|---|---|
| `Selection::Field(Field { position, alias, name, arguments, directives, selection_set })` | `Selection::Field(FieldSelection { span, alias, name, arguments, directives, selection_set })` |
| `Selection::FragmentSpread(FragmentSpread { position, fragment_name, directives })` | `Selection::Fragment(FragmentSpread { span, name, directives })` |
| `Selection::InlineFragment(InlineFragment { position, type_condition, directives, selection_set })` | `Selection::Inline(InlineFragment { span, type_condition, directives, selection_set })` |

Field access:
- `field.arguments` changes from `Vec<(String, Value)>` to `Vec<Argument>` — iterate
  with `arg.name.value` and `arg.value`
- `field.selection_set` changes from `SelectionSet` to `Option<SelectionSet>`

### 9c: Other operation builders
- `FragmentBuilder::from_ast()` — takes `&FragmentDefinition<'_>` + `&SourceMap`
- `FragmentRegistryBuilder` — parse calls change to `libgraphql_parser::parse_executable()`
- `ExecutableDocumentBuilder` — parse calls change similarly
- `QueryBuilder`, `MutationBuilder`, `SubscriptionBuilder` — thin wrappers, minimal
  changes (mostly updating the AST type in `from_ast()`)

### Files:
- `crates/libgraphql-core/src/operation/operation_builder.rs`
- `crates/libgraphql-core/src/operation/selection_set_builder.rs`
- `crates/libgraphql-core/src/operation/fragment_builder.rs`
- `crates/libgraphql-core/src/operation/fragment_registry_builder.rs`
- `crates/libgraphql-core/src/operation/executable_document_builder.rs`
- `crates/libgraphql-core/src/operation/query_builder.rs`
- `crates/libgraphql-core/src/operation/mutation_builder.rs`
- `crates/libgraphql-core/src/operation/subscription_builder.rs`

---

## Phase 10: Update Error Types

The following error types reference parse errors:
- `SchemaBuildError::ParseError` — currently wraps `graphql_parser::schema::ParseError`
- `OperationBuildError::ParseError` — wraps `graphql_parser::query::ParseError`
- Others: `ExecutableDocumentBuildError`, `SelectionSetBuildError`, `FragmentBuildError`

Change to wrap `Vec<libgraphql_parser::GraphQLParseError>` or a formatted error string.
The `SchemaBuildError::ParseError` variant already stores `{ file, err: String }` from
the current `.to_string()` call, so it may need minimal changes.

### Files:
- Error types in `schema/schema_builder.rs`, `operation/operation_builder.rs`, etc.

---

## Phase 11: Update Test Files

### Test helpers that call graphql-parser directly:
- `test/snapshot_tests/test_runner.rs` line 678: Replace
  `graphql_parser::query::parse_query::<String>()` with
  `libgraphql_parser::parse_executable()`

### Type builder tests:
- `types/tests/enum_type_builder_tests.rs`
- `types/tests/interface_type_builder_tests.rs`
- `types/tests/object_type_builder_tests.rs`
- `types/tests/test_utils.rs`
- Any test that constructs AST types directly needs updating

### Files:
- All test files under `crates/libgraphql-core/src/*/tests/`

---

## Phase 12: Remove graphql-parser Dependency

### Changes:
1. `crates/libgraphql-core/Cargo.toml`: Remove `graphql-parser = { workspace = true }`
2. Verify zero remaining `use graphql_parser` or `graphql_parser::` references
3. Workspace `Cargo.toml`: Check if `graphql-parser` can be removed from workspace
   dependencies (note: `libgraphql-parser` keeps its own dep for the compat module)

### Verification:
```bash
cargo check --workspace --tests
cargo test --workspace
cargo clippy --workspace --tests
```

---

## Structural Differences Cheat Sheet

| Concern | graphql-parser v0.4 | libgraphql-parser |
|---|---|---|
| **Strings** | `String` (owned) | `Cow<'src, str>` via `.name.value` |
| **Position** | `Pos { line, column }` (1-based) | `ByteSpan { start, end }` + `SourceMap` (0-based) |
| **Type annotations** | Recursive: `NamedType`/`ListType`/`NonNullType` | Flat: `Named`/`List` each with `Nullability` field |
| **Operations** | Enum: `SelectionSet`/`Query`/`Mutation`/`Subscription` | Struct with `operation_kind` + `shorthand: bool` |
| **Selection::Field** | `Selection::Field(Field { ... })` | `Selection::Field(FieldSelection { ... })` |
| **Selection::FragmentSpread** | `Selection::FragmentSpread(...)` | `Selection::Fragment(FragmentSpread { ... })` |
| **Selection::InlineFragment** | `Selection::InlineFragment(...)` | `Selection::Inline(InlineFragment { ... })` |
| **Directive annotation** | `Directive { name: String, arguments: Vec<(String, Value)> }` | `DirectiveAnnotation { name: Name, arguments: Vec<Argument> }` |
| **Schema field def** | `schema::Field { arguments: Vec<InputValue>, field_type }` | `FieldDefinition { parameters: Vec<InputValueDefinition>, field_type }` |
| **Input value** | `InputValue { name, value_type, ... }` | `InputValueDefinition { name: Name, value_type: TypeAnnotation, ... }` |
| **Description** | `Option<String>` | `Option<StringValue<'src>>` → `.value.to_string()` |
| **Document** | Separate schema/query docs | Unified `Document` with `Definition` enum (7 variants) |
| **SchemaDefinition** | `query: Option<String>`, `mutation: Option<String>`, `subscription: Option<String>` | `root_operations: Vec<RootOperationTypeDefinition>` |
| **Int values** | `Value::Int(Number)` → `.as_i64()` | `Value::Int(IntValue { value: i32 })` |
| **Object values** | `Value::Object(BTreeMap<String, Value>)` | `Value::Object(ObjectValue { fields: Vec<ObjectField> })` |
| **Parse errors** | Single `ParseError` | `Vec<GraphQLParseError>` (error recovery) |
| **Type def names** | `EnumType`, `ObjectType`, etc. | `EnumTypeDefinition`, `ObjectTypeDefinition`, etc. |
| **TypeExtension variant** | `TypeExtension::InputObject(...)` | `TypeExtension::Input(...)` |

---

## Critical Files Summary

| File | Change Scope |
|---|---|
| `src/ast.rs` | **Delete** |
| `src/loc.rs` | Moderate — new `from_*_span()` + `with_span()` helpers |
| `src/value.rs` | Moderate — new `Value::from_ast()` matching |
| `src/directive_annotation_builder.rs` | Moderate — new directive/argument access |
| `src/types/type_annotation.rs` | Moderate — flat nullability (simpler) |
| `src/types/type_builder.rs` | Moderate — GATs + SourceMap param + field access |
| `src/types/parameter.rs` | Minor — InputValueDefinition access |
| `src/types/object_type_builder.rs` | **Significant** — pending extension struct + field access |
| `src/types/interface_type_builder.rs` | **Significant** — same pattern as object |
| `src/types/enum_type_builder.rs` | Moderate — pending extension + field access |
| `src/types/union_type_builder.rs` | Moderate — pending extension |
| `src/types/input_object_type_builder.rs` | Moderate — pending extension |
| `src/types/scalar_type_builder.rs` | Minor — pending extension (minimal data) |
| `src/schema/schema_builder.rs` | **Significant** — parse calls, definition dispatch, schema block |
| `src/operation/operation_builder.rs` | **Significant** — struct-based OperationDefinition |
| `src/operation/selection_set_builder.rs` | **Significant** — selection variant names/fields |
| `src/operation/fragment_builder.rs` | Moderate |
| `src/operation/fragment_registry_builder.rs` | Moderate — parse calls |
| `src/operation/executable_document_builder.rs` | Moderate — parse calls |
| `src/operation/query_builder.rs` | Minor |
| `src/operation/mutation_builder.rs` | Minor |
| `src/operation/subscription_builder.rs` | Minor |
| `Cargo.toml` (core) | Minor — dep changes |
| Test files | Moderate — AST construction changes |

## Verification Plan

After each compilable phase:
```bash
cargo check --package libgraphql-core --tests
cargo test --package libgraphql-core
```

Final:
```bash
cargo check --workspace --tests
cargo test --workspace
cargo clippy --workspace --tests
```
