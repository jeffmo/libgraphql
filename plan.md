# AST Design Plan for libgraphql-parser

## 1. Goals & Constraints

Design a custom AST for `libgraphql-parser` that replaces the current
`graphql_parser` crate type aliases. The new AST must satisfy:

1. **Zero-copy**: Parameterized over `'src` lifetime; borrows strings from
   source text via `Cow<'src, str>` (no allocations for
   `StrGraphQLTokenSource`, owned strings for `RustMacroGraphQLTokenSource`
   where `'src = 'static`)
2. **Transformer-friendly**: Efficient, simple conversions to/from
   `graphql_parser` AST, `apollo_parser` CST, `graphql_query` AST, and
   future external formats
3. **FFI-amenable**: Natural mapping to C structs/tagged unions; efficient
   access without deep indirection
4. **Tool-oriented**: Serve compilers, typecheckers, linters, formatters,
   IDEs, and LSP servers equally well
5. **Configurable fidelity**: Parser flags control inclusion of trivia
   (whitespace, comments) and syntactic tokens (punctuation, keywords)
6. **Incremental-ready**: Structure should not preclude future incremental
   re-parsing; ideally support partial AST replacement

---

## 2. Architecture Decision: Typed AST with Optional Syntax Layer

### Options Evaluated

**Option A — Typed structs (graphql-parser style, enhanced)**
Strongly-typed structs for each GraphQL construct. Each node has semantic
fields (name, fields, directives, etc.) plus a span. Simple, familiar,
directly maps to C structs.

**Option B — Arena-indexed typed nodes**
All nodes stored in typed arena vectors, referenced by index (`u32`).
Excellent FFI (indices are just integers), good cache locality, enables
structural sharing. More complex Rust API (every access goes through arena).

**Option C — Green/Red tree (Roslyn/rowan model)**
Position-independent "green" nodes (hash-consed, immutable) wrapped by
position-aware "red" nodes (computed on demand). Maximum incremental
reuse. Complex to implement; not FFI-natural; untyped nodes require
casting.

### Decision: Option A, with arena storage as a future optimization

**Rationale:**
- Typed structs are the most natural Rust API and the simplest to convert
  to/from other typed ASTs (graphql-parser, graphql_query)
- FFI is well-served by opaque pointers with accessor functions (standard
  Rust FFI pattern); the struct layouts themselves are secondary to the
  access API
- GraphQL documents are typically small (<100KB); at ~73 MiB/s parse
  throughput, full re-parse of even a 1MB schema takes ~14ms — making
  incremental parsing a nice-to-have, not a requirement
- The typed AST does not preclude a future arena-indexed or green-tree
  layer; those can wrap or replace internals without changing the public
  API
- Option B's ergonomic cost (arena-threaded access everywhere) is not
  justified until profiling shows it's needed
- Option C's complexity and untyped nature conflicts with the
  "simple transformers" and "FFI-amenable" constraints

### Two-Layer Design

The AST has two conceptual layers:

```
┌─────────────────────────────────────────────────┐
│  Semantic Layer (always present)                │
│  - Typed structs: ObjectTypeDefinition, Field,  │
│    Directive, Value, etc.                       │
│  - Cow<'src, str> names/values                  │
│  - ByteSpan on every node                       │
│  - Full GraphQL semantics                       │
└─────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────┐
│  Syntax Layer (optional, parser-flag-controlled) │
│  - Keyword/punctuation tokens with spans        │
│  - Trivia (whitespace runs, comments, commas)   │
│  - Enables lossless source reconstruction       │
│  - Stored in `Option<XyzSyntax>` fields         │
└─────────────────────────────────────────────────┘
```

When parser flags disable the syntax layer, the `Option<...Syntax>` fields
are `None`, and the AST is a lean semantic tree comparable to
`graphql_parser`. When enabled, the AST is a lossless representation
suitable for formatters and IDE tooling.

---

## 3. Span Design

### Per-Node Span: `ByteSpan`

```rust
/// Compact byte-offset span. 8 bytes per node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct ByteSpan {
    /// Byte offset of the first byte of this node in the source
    /// text (0-based, inclusive).
    pub start: u32,
    /// Byte offset one past the last byte of this node in the
    /// source text (0-based, exclusive).
    pub end: u32,
}
```

**Rationale:**
- 8 bytes vs 104+ bytes for `GraphQLSourceSpan` (includes
  `Option<PathBuf>`)
- `u32` supports documents up to 4 GiB (sufficient for any GraphQL
  document; the largest known public schema — GitHub's — is ~1.2 MB)
- `#[repr(C)]` for direct FFI access
- Byte offsets are the most fundamental span representation; all other
  position info can be derived from them

### Line/Column Recovery: `SourceMap`

```rust
/// Maps byte offsets to line/column positions. Built once during
/// parsing, shared across all lookups.
pub struct SourceMap {
    /// Sorted byte offsets of each line start (index 0 = line 0).
    line_starts: Vec<u32>,
    /// Optional: UTF-16 column offset table for LSP compatibility.
    /// Only populated when the token source provides col_utf16.
    utf16_offsets: Option<Vec<Utf16LineInfo>>,
}

impl SourceMap {
    /// O(log n) lookup: byte offset → (line, col_utf8).
    pub fn line_col(&self, byte_offset: u32) -> (u32, u32);

    /// O(log n) lookup: byte offset → (line, col_utf16).
    /// Returns None if UTF-16 info was not collected.
    pub fn line_col_utf16(
        &self,
        byte_offset: u32,
    ) -> Option<(u32, u32)>;

    /// Convert a ByteSpan to a full SourcePosition pair.
    pub fn resolve_span(
        &self,
        span: ByteSpan,
    ) -> (SourcePosition, SourcePosition);
}
```

**Rationale:**
- Line-start tables are compact (~1 entry per source line) and enable
  O(log n) position lookups
- Separating position info from spans saves ~56 bytes per node
- The `SourceMap` is built during lexing (the lexer already tracks line
  positions) at near-zero marginal cost
- UTF-16 column info is optional because `RustMacroGraphQLTokenSource`
  cannot provide it
- Matches standard compiler architecture (rustc, clang, swc, oxc)

### Convenience: Rich Position On Demand

```rust
impl ByteSpan {
    /// Resolve to full source positions using a SourceMap.
    pub fn resolve(
        &self,
        source_map: &SourceMap,
    ) -> ResolvedSpan;
}

pub struct ResolvedSpan {
    pub start: SourcePosition,
    pub end: SourcePosition,
}
```

### Preserving File Path

File path is stored once on the `ParseResult` / `Document`, not on every
span. Nodes inherit the file path from their containing document.

---

## 4. String Representation

### `Cow<'src, str>` for All String Data

All name identifiers, string literal values, descriptions, and enum
values use `Cow<'src, str>`:

```rust
pub struct Name<'src> {
    pub value: Cow<'src, str>,
    pub span: ByteSpan,
}
```

**How this works across token sources:**

| Token Source                   | `'src`               | String storage                                   |
|--------------------------------|----------------------|--------------------------------------------------|
| `StrGraphQLTokenSource<'src>`  | Borrows `&'src str`  | `Cow::Borrowed` (zero-copy)                      |
| `RustMacroGraphQLTokenSource`  | `'static`            | `Cow::Owned` (allocated from proc_macro2 tokens) |

The parser is already generic over `GraphQLTokenSource<'src>`, so the
AST type parameter flows naturally:

```rust
impl<'src, S: GraphQLTokenSource<'src>> GraphQLParser<'src, S> {
    pub fn parse_schema_document(
        self,
    ) -> ParseResult<'src, Document<'src>>;
}
```

### Scalar Value Cooking

The parser must fully process ("cook") every scalar literal during
parsing in order to validate it and produce diagnostics. Since the
work is already done, we store the cooked value directly in the AST
node rather than discarding it and recomputing on access.

All fields are `pub` — no `OnceLock`, no private fields, no lazy
`.value()` methods. Raw source text is available via `span` + source
or via the syntax layer's `AstToken` when retained.

#### StringValue

```rust
pub struct StringValue<'src> {
    /// The processed string value after escape-sequence
    /// resolution and block-string indentation stripping.
    /// Borrows from the source when no transformation was
    /// needed (simple quoted string with no escapes);
    /// owned when escapes or block-string stripping produced
    /// a new string.
    pub value: Cow<'src, str>,
    pub span: ByteSpan,
    pub syntax: Option<StringValueSyntax<'src>>,
}
```

#### IntValue

The GraphQL spec constrains Int to signed 32-bit range. The parser
validates this and emits a diagnostic on overflow/underflow, error-
recovering to `i32::MAX` / `i32::MIN` respectively. These are the
only two failure modes — a lexed `GraphQLTokenKind::Int` token is
necessarily `-?[0-9]+` (leading zeros already rejected by the
lexer), so no other parse errors are possible.

```rust
pub struct IntValue<'src> {
    /// The parsed 32-bit integer value. On overflow/underflow
    /// the parser emits a diagnostic and clamps to
    /// i32::MAX / i32::MIN.
    pub value: i32,
    pub span: ByteSpan,
    pub syntax: Option<IntValueSyntax<'src>>,
}

impl IntValue<'_> {
    /// Widen to i64 (infallible).
    pub fn as_i64(&self) -> i64;
}
```

#### FloatValue

```rust
pub struct FloatValue<'src> {
    /// The parsed f64 value. On overflow the parser emits a
    /// diagnostic and stores f64::INFINITY / f64::NEG_INFINITY.
    pub value: f64,
    pub span: ByteSpan,
    pub syntax: Option<FloatValueSyntax<'src>>,
}
```

**Rationale:** The parser must cook every value for validation
anyway, so storing the result avoids double computation (validate
at parse time, then recompute on access). This also eliminates the
`OnceLock`-based lazy cache that was previously planned for
`StringValue`, removing the sole private field from AST nodes and
the associated `Send + Sync` concern.

---

## 5. Node Catalog

### 5.1 Document-Level Nodes

```rust
/// Root AST node for any GraphQL document.
pub struct Document<'src> {
    pub definitions: Vec<Definition<'src>>,
    pub span: ByteSpan,
}

/// A top-level definition in a GraphQL document.
pub enum Definition<'src> {
    // ---- Type System ----
    SchemaDefinition(SchemaDefinition<'src>),
    TypeDefinition(TypeDefinition<'src>),
    DirectiveDefinition(DirectiveDefinition<'src>),
    SchemaExtension(SchemaExtension<'src>),
    TypeExtension(TypeExtension<'src>),

    // ---- Executable ----
    OperationDefinition(OperationDefinition<'src>),
    FragmentDefinition(FragmentDefinition<'src>),
}
```

**Note:** A single unified `Definition` enum replaces the current
separate `schema::Definition` / `operation::Definition` enums. This
naturally supports mixed documents (schema + executable interleaved)
without a separate `MixedDocument` type. Filtering to "schema only"
or "executable only" is a method on `Document`:

```rust
impl<'src> Document<'src> {
    pub fn schema_definitions(
        &self,
    ) -> impl Iterator<Item = &Definition<'src>>;
    pub fn executable_definitions(
        &self,
    ) -> impl Iterator<Item = &Definition<'src>>;
}
```

### 5.2 Type System Definitions

```rust
pub struct SchemaDefinition<'src> {
    pub span: ByteSpan,
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub root_operations: Vec<RootOperationTypeDefinition<'src>>,
    pub syntax: Option<SchemaDefinitionSyntax<'src>>,
}

pub struct RootOperationTypeDefinition<'src> {
    pub span: ByteSpan,
    pub operation_type: OperationType,
    pub named_type: Name<'src>,
    pub syntax: Option<RootOperationTypeDefinitionSyntax<'src>>,
}

pub enum OperationType { Query, Mutation, Subscription }

pub enum TypeDefinition<'src> {
    Scalar(ScalarTypeDefinition<'src>),
    Object(ObjectTypeDefinition<'src>),
    Interface(InterfaceTypeDefinition<'src>),
    Union(UnionTypeDefinition<'src>),
    Enum(EnumTypeDefinition<'src>),
    InputObject(InputObjectTypeDefinition<'src>),
}

pub struct ScalarTypeDefinition<'src> {
    pub span: ByteSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<ScalarTypeDefinitionSyntax<'src>>,
}

pub struct ObjectTypeDefinition<'src> {
    pub span: ByteSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub implements_interfaces: Vec<Name<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub syntax: Option<ObjectTypeDefinitionSyntax<'src>>,
}

pub struct InterfaceTypeDefinition<'src> {
    pub span: ByteSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub implements_interfaces: Vec<Name<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub syntax: Option<InterfaceTypeDefinitionSyntax<'src>>,
}

pub struct UnionTypeDefinition<'src> {
    pub span: ByteSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub members: Vec<Name<'src>>,
    pub syntax: Option<UnionTypeDefinitionSyntax<'src>>,
}

pub struct EnumTypeDefinition<'src> {
    pub span: ByteSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub values: Vec<EnumValueDefinition<'src>>,
    pub syntax: Option<EnumTypeDefinitionSyntax<'src>>,
}

pub struct InputObjectTypeDefinition<'src> {
    pub span: ByteSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<InputValueDefinition<'src>>,
    pub syntax:
        Option<InputObjectTypeDefinitionSyntax<'src>>,
}

pub struct DirectiveDefinition<'src> {
    pub span: ByteSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub arguments: Vec<InputValueDefinition<'src>>,
    pub repeatable: bool,
    pub locations: Vec<DirectiveLocation<'src>>,
    pub syntax: Option<DirectiveDefinitionSyntax<'src>>,
}

/// Directive location with span (unlike graphql_parser which
/// uses a plain enum).
pub struct DirectiveLocation {
    pub value: DirectiveLocationKind,
    pub span: ByteSpan,
}

pub enum DirectiveLocationKind {
    // Executable
    Query, Mutation, Subscription, Field,
    FragmentDefinition, FragmentSpread,
    InlineFragment, VariableDefinition,
    // Type System
    Schema, Scalar, Object, FieldDefinition,
    ArgumentDefinition, Interface, Union, Enum,
    EnumValue, InputObject, InputFieldDefinition,
}
```

### 5.3 Type Extensions

```rust
/// NEW: Schema extension support (currently unsupported by parser).
pub struct SchemaExtension<'src> {
    pub span: ByteSpan,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub root_operations:
        Vec<RootOperationTypeDefinition<'src>>,
    pub syntax: Option<SchemaExtensionSyntax<'src>>,
}

pub enum TypeExtension<'src> {
    Scalar(ScalarTypeExtension<'src>),
    Object(ObjectTypeExtension<'src>),
    Interface(InterfaceTypeExtension<'src>),
    Union(UnionTypeExtension<'src>),
    Enum(EnumTypeExtension<'src>),
    InputObject(InputObjectTypeExtension<'src>),
}

// Each extension type mirrors its definition counterpart
// minus description and name, plus span. Example:
pub struct ObjectTypeExtension<'src> {
    pub span: ByteSpan,
    pub name: Name<'src>,
    pub implements_interfaces: Vec<Name<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub syntax: Option<ObjectTypeExtensionSyntax<'src>>,
}
// ... similar patterns for other extension types ...
```

### 5.4 Executable Definitions

```rust
pub struct OperationDefinition<'src> {
    pub span: ByteSpan,
    pub operation_type: OperationType,
    pub name: Option<Name<'src>>,
    pub variable_definitions:
        Vec<VariableDefinition<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub selection_set: SelectionSet<'src>,
    pub syntax:
        Option<OperationDefinitionSyntax<'src>>,
}

pub struct FragmentDefinition<'src> {
    pub span: ByteSpan,
    pub name: Name<'src>,
    pub type_condition: TypeCondition<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub selection_set: SelectionSet<'src>,
    pub syntax: Option<FragmentDefinitionSyntax<'src>>,
}

pub struct VariableDefinition<'src> {
    pub span: ByteSpan,
    pub variable: Name<'src>,
    pub var_type: TypeAnnotation<'src>,
    pub default_value: Option<Value<'src>>,
    /// NEW: Variable directives (per Sep 2025 spec).
    /// Currently lost by graphql_parser AST.
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<VariableDefinitionSyntax<'src>>,
}
```

### 5.5 Selection Sets

```rust
pub struct SelectionSet<'src> {
    pub span: ByteSpan,
    pub selections: Vec<Selection<'src>>,
    pub syntax: Option<SelectionSetSyntax<'src>>,
}

pub enum Selection<'src> {
    Field(Field<'src>),
    FragmentSpread(FragmentSpread<'src>),
    InlineFragment(InlineFragment<'src>),
}

pub struct Field<'src> {
    pub span: ByteSpan,
    pub alias: Option<Name<'src>>,
    pub name: Name<'src>,
    pub arguments: Vec<Argument<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub selection_set: Option<SelectionSet<'src>>,
    pub syntax: Option<FieldSyntax<'src>>,
}

pub struct FragmentSpread<'src> {
    pub span: ByteSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<FragmentSpreadSyntax<'src>>,
}

pub struct InlineFragment<'src> {
    pub span: ByteSpan,
    pub type_condition: Option<TypeCondition<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub selection_set: SelectionSet<'src>,
    pub syntax: Option<InlineFragmentSyntax<'src>>,
}
```

### 5.6 Shared Sub-Nodes

```rust
pub struct FieldDefinition<'src> {
    pub span: ByteSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub arguments: Vec<InputValueDefinition<'src>>,
    pub field_type: TypeAnnotation<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<FieldDefinitionSyntax<'src>>,
}

pub struct InputValueDefinition<'src> {
    pub span: ByteSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub value_type: TypeAnnotation<'src>,
    pub default_value: Option<Value<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax:
        Option<InputValueDefinitionSyntax<'src>>,
}

pub struct EnumValueDefinition<'src> {
    pub span: ByteSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax:
        Option<EnumValueDefinitionSyntax<'src>>,
}

pub struct DirectiveAnnotation<'src> {
    pub span: ByteSpan,
    pub name: Name<'src>,
    pub arguments: Vec<Argument<'src>>,
    pub syntax: Option<DirectiveAnnotationSyntax<'src>>,
}

pub struct Argument<'src> {
    pub span: ByteSpan,
    pub name: Name<'src>,
    pub value: Value<'src>,
    pub syntax: Option<ArgumentSyntax<'src>>,
}

pub struct TypeCondition<'src> {
    pub span: ByteSpan,
    pub named_type: Name<'src>,
    pub syntax: Option<TypeConditionSyntax<'src>>,
}
```

### 5.7 Type Annotations

The spec grammar has three type productions (`NamedType`, `ListType`,
`NonNullType`), but `NonNullType` is purely a wrapper that adds `!`.
Rather than model it as a recursive enum variant — which would allow
illegal states like `NonNull(NonNull(...))` — we flatten nullability
into a `Nullability` field on each concrete type annotation node.

The `Nullability` enum owns the `!` token directly in its `NonNull`
variant, making it impossible for nullability semantics and syntax to
disagree (e.g. a non-null annotation missing its `!` token or a
nullable annotation carrying one).

- `NamedTypeAnnotation.span` covers the full annotation including `!`
  when present. The underlying name span is available via
  `NamedTypeAnnotation.name.span`.
- `ListTypeAnnotation.span` likewise covers brackets and trailing `!`.

```rust
pub enum Nullability<'src> {
    Nullable,
    NonNull {
        /// The `!` token. Present when syntax detail is retained.
        syntax: Option<AstToken<'src>>,
    },
}

pub enum TypeAnnotation<'src> {
    Named(NamedTypeAnnotation<'src>),
    List(ListTypeAnnotation<'src>),
}

pub struct NamedTypeAnnotation<'src> {
    pub name: Name<'src>,
    pub nullability: Nullability<'src>,
    pub span: ByteSpan,
}

pub struct ListTypeAnnotation<'src> {
    pub element_type: Box<TypeAnnotation<'src>>,
    pub nullability: Nullability<'src>,
    pub span: ByteSpan,
    pub syntax: Option<ListTypeAnnotationSyntax<'src>>,
}
```

### 5.8 Values

```rust
pub enum Value<'src> {
    Variable(Variable<'src>),
    Int(IntValue<'src>),
    Float(FloatValue<'src>),
    String(StringValue<'src>),
    Boolean(BooleanValue<'src>),
    Null(NullValue<'src>),
    Enum(EnumValue<'src>),
    List(ListValue<'src>),
    Object(ObjectValue<'src>),
}

pub struct Variable<'src> {
    pub name: Name<'src>,
    pub span: ByteSpan,
    pub syntax: Option<VariableSyntax<'src>>,
}

pub struct BooleanValue<'src> {
    pub value: bool,
    pub span: ByteSpan,
    pub syntax: Option<BooleanValueSyntax<'src>>,
}

pub struct NullValue<'src> {
    pub span: ByteSpan,
    pub syntax: Option<NullValueSyntax<'src>>,
}

pub struct EnumValue<'src> {
    pub value: Cow<'src, str>,
    pub span: ByteSpan,
    pub syntax: Option<EnumValueSyntax<'src>>,
}

pub struct ListValue<'src> {
    pub values: Vec<Value<'src>>,
    pub span: ByteSpan,
    pub syntax: Option<ListValueSyntax<'src>>,
}

pub struct ObjectValue<'src> {
    pub fields: Vec<ObjectField<'src>>,
    pub span: ByteSpan,
    pub syntax: Option<ObjectValueSyntax<'src>>,
}

pub struct ObjectField<'src> {
    pub name: Name<'src>,
    pub value: Value<'src>,
    pub span: ByteSpan,
    pub syntax: Option<ObjectFieldSyntax<'src>>,
}
```

### 5.9 Summary: Node Count

| Category                                 | Count  |
|------------------------------------------|--------|
| Document/Definition enums                | 4      |
| Type system definitions                  | 8      |
| Type extensions                          | 7      |
| Executable definitions                   | 2      |
| Selection/Field types                    | 4      |
| Shared sub-nodes                         | 6      |
| Type annotation nodes                    | 2      |
| Value nodes                              | 9      |
| Terminal nodes (Name, StringValue, etc.) | 5      |
| **Total**                                | **~47** |

This is a superset of both `graphql_parser` (~44 types) and
`apollo_parser` CST node kinds, covering the full Sep 2025 spec
including schema extensions and variable directives.

---

## 6. Syntax Layer (Optional Trivia & Token Detail)

### Design

Each AST node has an `Option<XyzSyntax<'src>>` field. When the parser
is configured to retain syntax detail, this field is `Some(...)` and
contains all punctuation tokens, keywords, and trivia. When syntax
detail is disabled, the field is `None`.

### Syntax Detail Struct Pattern

```rust
/// Syntax tokens for an object type definition:
///   "type" Name ImplementsInterfaces? Directives?
///       FieldsDefinition?
pub struct ObjectTypeDefinitionSyntax<'src> {
    pub type_keyword: AstToken<'src>,
    pub implements_keyword: Option<AstToken<'src>>,
    pub first_ampersand: Option<AstToken<'src>>,
    pub ampersands: Vec<AstToken<'src>>,
    pub open_brace: Option<AstToken<'src>>,
    pub close_brace: Option<AstToken<'src>>,
}
```

### Trivia in Comma-Separated Lists

**Design principle:** Every source token in the document — including
value literals, names, keywords, and punctuation — has a
corresponding `AstToken` somewhere in the syntax layer. This ensures
the leading-trivia model is perfectly consistent: trivia (whitespace,
comments, commas) always attaches as `leading_trivia` on the
`AstToken` of the next source token in document order. No trivia is
ever orphaned.

For comma-separated constructs (list values, arguments, object fields,
etc.), this means commas appear as `AstTokenTrivia::Comma` items in
the `leading_trivia` of the following item's `AstToken`. No special
`infix_commas` vec is needed.

To make this work, every semantic value node has a `*Syntax` struct
containing an `AstToken` for its source token:

```rust
pub struct IntValueSyntax<'src> {
    pub token: AstToken<'src>,
}
pub struct FloatValueSyntax<'src> {
    pub token: AstToken<'src>,
}
pub struct StringValueSyntax<'src> {
    pub token: AstToken<'src>,
}
pub struct BooleanValueSyntax<'src> {
    pub token: AstToken<'src>,
}
pub struct NullValueSyntax<'src> {
    pub token: AstToken<'src>,
}
pub struct EnumValueSyntax<'src> {
    pub token: AstToken<'src>,
}
```

And container syntax structs only need their delimiter tokens:

```rust
pub struct ListValueSyntax<'src> {
    pub open_bracket: AstToken<'src>,
    pub close_bracket: AstToken<'src>,
}
```

#### Example 1: List value `[1, 2, 3]`

```
 Byte:  0  1  2  3  4  5  6  7  8
 Char:  [  1  ,     2  ,     3  ]
```

Full AST (semantic + syntax layers interleaved):

```rust
ListValue {
    values: [
        Value::Int(IntValue {
            raw: "1",
            span: ByteSpan { start: 1, end: 2 },
            syntax: Some(IntValueSyntax {
                token: AstToken {
                    span: ByteSpan { start: 1, end: 2 },
                    leading_trivia: [],
                },
            }),
        }),
        Value::Int(IntValue {
            raw: "2",
            span: ByteSpan { start: 4, end: 5 },
            syntax: Some(IntValueSyntax {
                token: AstToken {
                    span: ByteSpan { start: 4, end: 5 },
                    // Comma + space before "2"
                    leading_trivia: [
                        AstTokenTrivia::Comma {
                            span: ByteSpan {
                                start: 2, end: 3,
                            },
                        },
                        AstTokenTrivia::Whitespace {
                            text: " ",
                            span: ByteSpan {
                                start: 3, end: 4,
                            },
                        },
                    ],
                },
            }),
        }),
        Value::Int(IntValue {
            raw: "3",
            span: ByteSpan { start: 7, end: 8 },
            syntax: Some(IntValueSyntax {
                token: AstToken {
                    span: ByteSpan { start: 7, end: 8 },
                    // Comma + space before "3"
                    leading_trivia: [
                        AstTokenTrivia::Comma {
                            span: ByteSpan {
                                start: 5, end: 6,
                            },
                        },
                        AstTokenTrivia::Whitespace {
                            text: " ",
                            span: ByteSpan {
                                start: 6, end: 7,
                            },
                        },
                    ],
                },
            }),
        }),
    ],
    span: ByteSpan { start: 0, end: 9 },
    syntax: Some(ListValueSyntax {
        open_bracket: AstToken {
            span: ByteSpan { start: 0, end: 1 },
            leading_trivia: [],
        },
        close_bracket: AstToken {
            span: ByteSpan { start: 8, end: 9 },
            leading_trivia: [],
        },
    }),
}
```

Every token has exactly one `AstToken` home. The commas at bytes 2
and 5 are `AstTokenTrivia::Comma` in the `leading_trivia` of the
next value's `AstToken`. The spaces at bytes 3 and 6 follow the
commas in the same `leading_trivia` vec. The `close_bracket` has no
leading trivia because `3` is immediately followed by `]`.

#### Example 2: Argument list `(x: 1, y: 2)`

```
 Byte:  0  1  2  3  4  5  6  7  8  9  10  11
 Char:  (  x  :     1  ,     y  :     2   )
```

The relevant syntax structs:

```rust
pub struct ArgumentSyntax<'src> {
    pub name: AstToken<'src>,
    pub colon: AstToken<'src>,
    // The argument's value carries its own *ValueSyntax
    // with an AstToken — trivia before the value (e.g.,
    // the space between ":" and the value) lands there.
}
```

Suppose these arguments belong to a `Field`. The `FieldSyntax`
holds the parentheses; each `Argument`'s syntax holds its name and
colon tokens; each argument's value holds its own value token:

```rust
// FieldSyntax (partial — just the argument delimiters):
FieldSyntax {
    open_paren: Some(AstToken {
        span: ByteSpan { start: 0, end: 1 },
        leading_trivia: [],
    }),
    close_paren: Some(AstToken {
        span: ByteSpan { start: 11, end: 12 },
        leading_trivia: [],
    }),
    // ...
}

// arguments[0]: x: 1
Argument {
    name: Name {
        value: "x",
        span: ByteSpan { start: 1, end: 2 },
    },
    value: Value::Int(IntValue {
        raw: "1",
        span: ByteSpan { start: 4, end: 5 },
        syntax: Some(IntValueSyntax {
            token: AstToken {
                span: ByteSpan { start: 4, end: 5 },
                // Space between ":" and "1"
                leading_trivia: [
                    AstTokenTrivia::Whitespace {
                        text: " ",
                        span: ByteSpan {
                            start: 3, end: 4,
                        },
                    },
                ],
            },
        }),
    }),
    syntax: Some(ArgumentSyntax {
        name: AstToken {
            span: ByteSpan { start: 1, end: 2 },
            leading_trivia: [],
        },
        colon: AstToken {
            span: ByteSpan { start: 2, end: 3 },
            leading_trivia: [],
        },
    }),
}

// arguments[1]: y: 2
Argument {
    name: Name {
        value: "y",
        span: ByteSpan { start: 7, end: 8 },
    },
    value: Value::Int(IntValue {
        raw: "2",
        span: ByteSpan { start: 10, end: 11 },
        syntax: Some(IntValueSyntax {
            token: AstToken {
                span: ByteSpan { start: 10, end: 11 },
                // Space between ":" and "2"
                leading_trivia: [
                    AstTokenTrivia::Whitespace {
                        text: " ",
                        span: ByteSpan {
                            start: 9, end: 10,
                        },
                    },
                ],
            },
        }),
    }),
    syntax: Some(ArgumentSyntax {
        name: AstToken {
            span: ByteSpan { start: 7, end: 8 },
            // Comma + space between "1" and "y"
            leading_trivia: [
                AstTokenTrivia::Comma {
                    span: ByteSpan {
                        start: 5, end: 6,
                    },
                },
                AstTokenTrivia::Whitespace {
                    text: " ",
                    span: ByteSpan {
                        start: 6, end: 7,
                    },
                },
            ],
        },
        colon: AstToken {
            span: ByteSpan { start: 8, end: 9 },
            leading_trivia: [],
        },
    }),
}
```

Same pattern: the comma at byte 5 is leading trivia on the second
argument's `name` AstToken. The space at byte 6 follows it. Trivia
between `:` and the value (bytes 3 and 9) is leading trivia on the
value's `IntValueSyntax.token`.

#### Summary

The invariant is simple: **every piece of trivia is leading trivia on
the `AstToken` of the next source token in document order.** Because
every semantic node that corresponds to a source token has a
`*Syntax` struct with an `AstToken`, no trivia is ever orphaned. This
generalizes to all comma-separated constructs (arguments, variable
definitions, enum values, object fields, etc.) without any special
`infix_commas` machinery.

### AstToken: Compact Token + Trivia

**Why not reuse `GraphQLToken<'src>`?** `GraphQLToken` is a *lexer
output* type carrying three fields: `kind: GraphQLTokenKind<'src>`,
`preceding_trivia: GraphQLTriviaTokenVec<'src>`, and
`span: GraphQLSourceSpan`. In the AST's syntax layer, each
`AstToken` is stored in a named field that already identifies what
token it is (e.g., `open_brace`, `type_keyword`), making the `kind`
discriminant redundant. `GraphQLToken` also uses `GraphQLSourceSpan`
(104+ bytes including `Option<PathBuf>`) while the AST uses `ByteSpan`
(8 bytes). Reusing `GraphQLToken` would add ~100 bytes of unnecessary
overhead per structural token. `AstToken` is a separate, lean
*AST storage* type:

```rust
/// A syntactic token preserved in the AST for lossless
/// source reconstruction. Unlike GraphQLToken (the lexer
/// output type), this omits the token kind (implied by the
/// field name in the parent Syntax struct) and uses the
/// compact ByteSpan rather than GraphQLSourceSpan.
pub struct AstToken<'src> {
    pub span: ByteSpan,
    pub leading_trivia: SmallVec<[AstTokenTrivia<'src>; 2]>,
    // Trailing trivia is the leading trivia of the *next*
    // token — not stored here to avoid duplication.
}

pub enum AstTokenTrivia<'src> {
    Whitespace {
        /// The whitespace text (spaces, tabs, newlines).
        text: Cow<'src, str>,
        span: ByteSpan,
    },
    Comment {
        /// The comment text (excluding the leading #).
        value: Cow<'src, str>,
        span: ByteSpan,
    },
    Comma {
        span: ByteSpan,
    },
}
```

**Note:** The current token layer stores comments and commas as trivia
but does NOT store whitespace. Adding whitespace to trivia requires
lexer changes (the lexer currently skips whitespace without recording
it). This is why the syntax layer is controlled by a parser flag — the
lexer can conditionally record whitespace runs.

### Trivia Attachment Strategy

Trivia is attached as **leading trivia** on the following token (same
as the current `GraphQLToken::preceding_trivia` design). This means:

- Trivia before the first token of a node is stored on that token
- Trivia after the last token of a definition is stored on the first
  token of the *next* definition (or lost if at EOF)
- **EOF trivia:** Trailing trivia at end-of-file is stored on a
  dedicated `Document.trailing_trivia` field

### Source Reconstruction

With the syntax layer enabled, lossless source reconstruction is
possible by walking the AST and emitting:
1. Leading trivia of each syntax token
2. The token text (derived from span + source text, or from the
   semantic value for names/strings)
3. Repeat for all tokens in document order

A `print_source(doc: &Document, source: &str) -> String` utility
function demonstrates this and serves as a correctness test.

---

## 7. Parser Flags / Configuration

```rust
pub struct ParserConfig {
    /// When true, the parser populates `syntax` fields on AST
    /// nodes with keyword/punctuation tokens and their trivia.
    /// Default: false.
    pub retain_syntax_tokens: bool,

    /// When true AND retain_syntax_tokens is true, whitespace
    /// runs between tokens are recorded as Trivia::Whitespace.
    /// When false, only comments and commas are trivia.
    /// Default: false.
    pub retain_whitespace_trivia: bool,

    // Future expansion:
    // pub max_recursion_depth: Option<usize>,
    // pub max_string_literal_size: Option<usize>,
    // pub spec_version: SpecVersion,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            retain_syntax_tokens: false,
            retain_whitespace_trivia: false,
        }
    }
}
```

**Parser API with config:**

```rust
impl<'src> GraphQLParser<'src, StrGraphQLTokenSource<'src>> {
    pub fn new(source: &'src str) -> Self;
    pub fn with_config(
        source: &'src str,
        config: ParserConfig,
    ) -> Self;
}
```

---

## 8. FFI Strategy

### Principles

1. **Opaque types** with accessor functions (standard Rust FFI pattern)
2. **`#[repr(C)]` on leaf types** that cross the boundary directly
   (`ByteSpan`, index types, enums without data)
3. **Owned wrapper** that bundles source text + AST to solve the
   lifetime/self-referential problem
4. **Flat accessor pattern**: C code calls `graphql_document_definition_count(doc)`,
   `graphql_document_definition_at(doc, i)`, etc.

### Self-Referential Ownership

The core challenge: `Document<'src>` borrows from source text, but C
needs a single opaque pointer.

```rust
/// Opaque type exposed to C. Owns both source and AST.
/// Uses `self_cell` crate (or manual unsafe) to safely
/// create a self-referential struct.
pub struct OwnedDocument {
    // Conceptually:
    //   source: String,
    //   document: Document<'source>,  // borrows from source
    //   source_map: SourceMap,
    //
    // Implemented via self_cell or ouroboros crate.
}
```

**Alternative (simpler, no self-referential struct):**

The C API takes both a source handle and a document handle. The user
is responsible for keeping the source alive while the document exists.
This matches C's manual lifetime management and avoids self-referential
complexity:

```c
GraphQLSource* src = graphql_source_new(text, len);
GraphQLDocument* doc = graphql_parse_schema(src);
// ... use doc (borrows from src) ...
graphql_document_free(doc);  // must free doc first
graphql_source_free(src);    // then free source
```

**[DECISION NEEDED]:** Self-referential owned wrapper (easier C API,
more Rust complexity) vs. two-handle API (simpler Rust implementation,
C user manages lifetimes manually). Recommendation: start with
two-handle API; add owned wrapper later if C users find it error-prone.

### `repr(C)` Types

```c
// C header (auto-generated)
typedef struct { uint32_t start; uint32_t end; } ByteSpan;

typedef enum {
    GRAPHQL_DEFINITION_SCHEMA = 0,
    GRAPHQL_DEFINITION_TYPE = 1,
    GRAPHQL_DEFINITION_DIRECTIVE = 2,
    GRAPHQL_DEFINITION_SCHEMA_EXTENSION = 3,
    GRAPHQL_DEFINITION_TYPE_EXTENSION = 4,
    GRAPHQL_DEFINITION_OPERATION = 5,
    GRAPHQL_DEFINITION_FRAGMENT = 6,
} GraphQLDefinitionKind;

// Accessor functions
size_t graphql_document_definition_count(
    const GraphQLDocument* doc
);
GraphQLDefinitionKind graphql_document_definition_kind(
    const GraphQLDocument* doc, size_t index
);
ByteSpan graphql_document_definition_span(
    const GraphQLDocument* doc, size_t index
);
// ... etc for each node type and field ...
```

### FFI Code Generation

Consider using `cbindgen` to auto-generate C headers from Rust types
annotated with `#[repr(C)]`. For the accessor-function pattern,
a proc-macro or build script could generate the boilerplate.

---

## 9. Conversion Layer

### 9.1 To `graphql_parser` AST (Primary)

```rust
impl<'src> Document<'src> {
    /// Convert to a graphql_parser schema document.
    /// Drops: spans (reduced to Pos), trivia, syntax tokens,
    ///        variable directives, schema extensions.
    pub fn to_graphql_parser_schema_document(
        &self,
        source_map: &SourceMap,
    ) -> graphql_parser::schema::Document<'static, String>;

    /// Convert to a graphql_parser executable document.
    /// Drops: spans (reduced to Pos), trivia, syntax tokens.
    pub fn to_graphql_parser_executable_document(
        &self,
        source_map: &SourceMap,
    ) -> graphql_parser::query::Document<'static, String>;
}
```

**Implementation notes:**
- `Cow<'src, str>` → `String` via `.into_owned()` or `.to_string()`
- `ByteSpan.start` → `Pos { line, column }` via `source_map.line_col()`
- Our `Definition` enum → discriminate into `schema::Definition` or
  `query::Definition` based on variant
- Information that `graphql_parser` lacks (variable directives, schema
  extensions, trivia) is silently dropped

**`SourceMap` parameter:** Required because `graphql_parser::Pos` needs
line/column, which we derive from byte offsets. This is the one place
where the `SourceMap` is mandatory for conversion. If this is too
cumbersome, we can store `SourceMap` inside `Document` (or alongside it
in `ParseResult`).

### 9.2 To `apollo_parser` CST

Apollo-parser uses a `rowan`-based CST (via `apollo_parser::cst`
module). Converting requires building a `rowan::GreenNode` tree.

```rust
impl<'src> Document<'src> {
    /// Convert to an apollo_parser-compatible CST.
    ///
    /// Requires the syntax layer to be populated
    /// (retain_syntax_tokens = true) for lossless conversion.
    /// Without the syntax layer, structural tokens are
    /// synthesized with zero-width spans.
    pub fn to_apollo_cst(
        &self,
        source: &str,
    ) -> apollo_parser::cst::Document;
}
```

**Implementation approach:**
1. Walk our AST depth-first
2. For each node, call `GreenNodeBuilder::start_node(SyntaxKind)`
3. For each syntax token (from the syntax layer), emit
   `GreenNodeBuilder::token(kind, text)`
4. For trivia, emit whitespace/comment tokens
5. `GreenNodeBuilder::finish_node()`

**Without syntax layer:** We can still produce a structurally valid CST
by synthesizing tokens from semantic values and spans. The CST will lack
trivia but will have correct node structure. This is a lossy but useful
conversion.

### 9.3 To `graphql_query` AST

The `graphql_query` crate uses a typed AST similar to `graphql_parser`
but with some differences in naming and structure. Conversion follows
the same pattern as 9.1.

### 9.4 From External ASTs (Reverse, Best-Effort)

Reverse conversions should translate as much information as each source
format provides. Some information will inevitably be absent (e.g.,
`graphql_parser` has no trivia), but what *is* available should be
faithfully carried over rather than discarded.

#### From `graphql_parser`

```rust
impl Document<'static> {
    pub fn from_graphql_parser_schema_document(
        doc: &graphql_parser::schema::Document<'static, String>,
    ) -> Document<'static>;

    pub fn from_graphql_parser_executable_document(
        doc: &graphql_parser::query::Document<'static, String>,
    ) -> Document<'static>;
}
```

**What transfers:**
- All semantic structure (definitions, fields, types, values, etc.)
- **Spans (partial):** `graphql_parser::Pos` provides 1-based
  line/column. We can convert these to `ByteSpan` if the original
  source text is also provided (compute byte offsets from line/col);
  without source text, `ByteSpan` start is set from a synthetic offset
  derived from `(line, col)` and end is set to start (zero-width)
- String values (owned, `Cow::Owned`)
- Descriptions, directives, arguments

**What is unavailable:**
- Trivia (whitespace, comments, commas) — `graphql_parser` discards
  these entirely
- Syntax layer tokens — no punctuation/keyword position info
- Variable directives — `graphql_parser` AST has no field for them
- Schema extensions with directives — not representable in
  `graphql_parser`
- Byte-accurate end positions — `graphql_parser::Pos` only marks
  start positions

**Overloaded API with source text for better spans:**

```rust
impl Document<'static> {
    /// When source text is provided, byte offsets are computed
    /// accurately from (line, col) pairs. Span end positions
    /// are estimated by scanning the source for the extent of
    /// each construct.
    pub fn from_graphql_parser_schema_document_with_source(
        doc: &graphql_parser::schema::Document<
            'static, String,
        >,
        source: &str,
    ) -> Document<'static>;
}
```

#### From `apollo_parser` CST

```rust
impl Document<'static> {
    pub fn from_apollo_cst(
        doc: &apollo_parser::cst::Document,
        source: &str,
    ) -> Document<'static>;
}
```

**What transfers:**
- All semantic structure
- **Spans (full):** `apollo_parser` CST nodes have precise byte-offset
  ranges via `text_range()` — these map directly to `ByteSpan`
- **Trivia (full):** The rowan-based CST preserves all whitespace,
  comments, and commas as tokens — these can be converted to our
  `AstTokenTrivia` types and attached to `AstToken`s
- **Syntax layer (full):** All punctuation and keyword tokens are
  present in the CST — the syntax layer can be fully populated
- String values, descriptions, directives, arguments

**What is unavailable:**
- Nothing major — `apollo_parser`'s CST is lossless. The conversion
  should produce a fully-populated AST including the syntax layer.
  The only limitation is that string values need to be re-extracted
  from source text via spans (the CST stores token text, not parsed
  values)

#### From `graphql_query`

Similar to `graphql_parser` — typed AST with positions but no trivia.
Best-effort span conversion applies.

#### Summary

| Source Format    | Spans          | Trivia         | Syntax Layer   |
|------------------|----------------|----------------|----------------|
| `graphql_parser` | Partial (Pos)  | Unavailable    | Unavailable    |
| `apollo_parser`  | Full           | Full           | Full           |
| `graphql_query`  | Partial        | Unavailable    | Unavailable    |

### 9.5 Compatibility API (Drop-In Replacement)

For the smoothest migration path, provide a compatibility module:

```rust
pub mod compat {
    pub mod graphql_parser_compat {
        /// Drop-in replacement for
        /// graphql_parser::schema::parse_schema.
        pub fn parse_schema<S: AsRef<str>>(
            input: S,
        ) -> Result<
            graphql_parser::schema::Document<'static, String>,
            Vec<GraphQLParseError>,
        >;

        /// Drop-in replacement for
        /// graphql_parser::query::parse_query.
        pub fn parse_query<S: AsRef<str>>(
            input: S,
        ) -> Result<
            graphql_parser::query::Document<'static, String>,
            Vec<GraphQLParseError>,
        >;
    }
}
```

This lets users switch parsers with a one-line import change while
keeping their existing code that operates on `graphql_parser` types.

---

## 10. Incremental Parsing: Exploration & Trade-Offs

### Context

The requirement is to support IDE scenarios where a user edits a portion
of a document and the AST should be updated without full re-parse.

### Assessment of Necessity

| Document Size              | Full Parse Time (est.) | Incremental Value |
|----------------------------|------------------------|-------------------|
| 1 KB (typical query)       | ~0.01 ms               | Negligible        |
| 10 KB (complex operation)  | ~0.13 ms               | Negligible        |
| 100 KB (large schema)      | ~1.3 ms                | Low               |
| 1 MB (GitHub schema)       | ~14 ms                 | Moderate          |
| 10 MB (hypothetical)       | ~137 ms                | High              |

For documents under ~1 MB (which covers nearly all real-world GraphQL),
full re-parse is fast enough for interactive use (< 16ms frame budget).
Incremental parsing becomes valuable only for very large schemas.

### Approach: Design for Future Incremental, Implement Full Re-Parse Now

The AST structure should not *preclude* incremental parsing, but we
should not implement it now. Specific design choices that preserve this
option:

1. **ByteSpan on every node**: Enables mapping a text edit to the
   affected AST subtree(s)
2. **Immutable nodes**: Nodes are not mutated in place; "editing" means
   producing a new node (enables structural sharing later)
3. **Vec-based children**: Can be replaced wholesale when a subtree is
   re-parsed
4. **Document-level re-parse is the initial API**:

```rust
/// Re-parse the document from scratch. This is the initial
/// and simplest API.
pub fn reparse(
    source: &'src str,
    config: &ParserConfig,
) -> ParseResult<'src, Document<'src>>;
```

### Future Incremental Strategy (When Needed)

When incremental parsing becomes necessary, the recommended approach is
**subtree re-parse with splice**:

1. Receive a text edit: `(edit_range: ByteSpan, new_text: &str)`
2. Apply the edit to the source text, producing new source
3. Identify the smallest enclosing definition(s) affected by the edit
   using byte spans
4. Re-parse only those definitions from the new source text
5. Replace the affected `Definition` nodes in the `Document.definitions`
   vector

This works because GraphQL documents are a flat list of top-level
definitions, and edits rarely span multiple definitions. The cost is
proportional to the size of the affected definition, not the whole
document.

**Finer-grained incremental** (re-parsing individual fields within a
type definition) is possible with the same approach applied recursively
but adds complexity. This is the "Phase 2" of incremental support if
the coarser approach proves insufficient.

### Alternative: Red-Green Tree (Phase 3, If Ever)

If even definition-level incremental is too coarse, the nuclear option is
adopting a green/red tree model (à la `rowan`/rust-analyzer). This would
require:
- Replacing `Vec<Child>` with `GreenNode` children
- Hash-consing identical subtrees
- Introducing a `RedNode` cursor API for position-aware traversal

This is a significant rewrite. The typed AST we're designing can serve as
the "red" (typed) layer over a green tree, but the green tree internals
would be a new data structure. **Recommendation: do not pursue this
unless/until the simpler approaches prove inadequate.**

### Summary of Incremental Strategy

| Phase      | Approach                | Complexity | When                          |
|------------|-------------------------|------------|-------------------------------|
| 0 (now)    | Full re-parse           | Trivial    | Default                       |
| 1 (future) | Definition-level splice | Moderate   | When >1MB schemas are common  |
| 2 (future) | Sub-definition splice   | High       | When Phase 1 is too slow      |
| 3 (future) | Green/red tree          | Very high  | Probably never for GraphQL    |

---

## 11. Parser Integration Plan

### How the Parser Changes

The parser (`graphql_parser.rs`) currently constructs `graphql_parser`
crate types. With the new AST:

1. **Replace all `graphql_parser::*` type references** with our new types
2. **Pass `ParserConfig` through the parser** to control syntax layer
   population
3. **Construct `ByteSpan`** from `GraphQLSourceSpan` (extract byte
   offsets)
4. **Populate `Name<'src>`** directly from token `Cow<'src, str>`
   (zero-copy path preserved)
5. **Conditionally construct `Syntax` structs** based on config flags
6. **Build `SourceMap`** during lexing (line-start offset table)
7. **Return `ParseResult<'src, Document<'src>>`** with source map

### Key Parser Method Changes

```rust
// Before:
fn parse_object_type_definition(
    &mut self,
    description: Option<String>,
) -> Result<ast::schema::TypeDefinition, ()>

// After:
fn parse_object_type_definition(
    &mut self,
    description: Option<StringValue<'src>>,
) -> Result<TypeDefinition<'src>, ()>
```

The parse methods become simpler in some ways (no `into_owned()` calls
for names when the target AST uses `Cow`) and slightly more complex in
others (conditionally building syntax structs).

### Preserving the Old AST API

During migration, the old `ast.rs` type aliases remain. The new AST
lives in a new module (e.g., `ast2.rs` or `typed_ast.rs`), and the
parser gains a second set of parse methods:

```rust
// Old API (deprecated, delegates to new + conversion):
pub fn parse_schema_document(
    self,
) -> ParseResult<ast::schema::Document>;

// New API:
pub fn parse_schema_document_v2(
    self,
) -> ParseResult<'src, Document<'src>>;
```

Once downstream crates (libgraphql-core, libgraphql-macros) are
migrated, the old API is removed and the new API is renamed to drop
the `_v2` suffix.

---

## 12. Implementation Phases

### Phase 1: Core AST Types

- Define all ~48 node types in a new `ast/` module within
  libgraphql-parser
- Implement `ByteSpan`, `Name`, `StringValue`, `IntValue`, `FloatValue`
- Implement `SourceMap` with line/column lookup
- Write unit tests for all node types (construction, accessors)
- No parser integration yet; just the type definitions

### Phase 2: Parser Integration

- Add `ParserConfig` to `GraphQLParser`
- Modify all `parse_*` methods to produce new AST types
- Build `SourceMap` during lexing
- Implement the semantic layer (syntax fields all `None` initially)
- Ensure all 443+ existing tests pass (via conversion to old AST)

### Phase 3: Syntax Layer

- Extend lexer to optionally record whitespace trivia
- Populate `Syntax` structs when `retain_syntax_tokens` is true
- Implement `AstToken` and `AstTokenTrivia` types
- Write source-reconstruction test (round-trip: parse → print → compare)

### Phase 4: Conversion Layer

- Implement `to_graphql_parser_schema_document()`
- Implement `to_graphql_parser_executable_document()`
- Implement compatibility API (`compat::graphql_parser_compat`)
- Implement `from_graphql_parser_*()` reverse conversions (lossy)

### Phase 5: Downstream Migration

- Update `libgraphql-macros` to use new AST
- Update `libgraphql-core` to use new AST (behind feature flag)
- Wire `use-libgraphql-parser` feature flag to use new parser + AST

### Phase 6: Apollo CST Conversion

- Implement `to_apollo_cst()` conversion
- Test against apollo-parser's own test fixtures

### Phase 7: FFI Layer

- Define C API surface (accessor functions)
- Implement `OwnedDocument` or two-handle pattern
- Auto-generate C headers
- Write C integration tests

### Phase 8: Cleanup

- Remove old `ast.rs` type aliases
- Remove `graphql_parser` crate dependency
- Rename `_v2` APIs
- Update documentation

---

## 13. Open Questions / Decisions Needed

1. ~~**StringValue storage:**~~ **RESOLVED.** All scalar values
   (string, int, float) are cooked eagerly during parsing and stored
   directly in the AST node. No `OnceLock`, no private fields, no
   lazy `.value()`. Parser must validate anyway, so storing the
   result avoids double computation. `StringValue.value` uses
   `Cow<'src, str>` (borrows when no transformation needed).
   `IntValue.value` is `i32` (clamped on overflow/underflow with
   diagnostic). `FloatValue.value` is `f64`.

2. **FFI ownership model:** Self-referential `OwnedDocument` (easier C
   API) vs two-handle `Source`+`Document` (simpler Rust implementation)?
   (Recommendation: two-handle initially.)

3. **SourceMap location:** Stored inside `Document` (convenient but
   increases document size) vs alongside in `ParseResult` (leaner
   documents but user must thread it through)?
   (Recommendation: stored in `ParseResult` alongside document; the
   `ParseResult` already bundles AST + errors.)

4. **Module naming:** `ast` (replace existing) vs `ast2` / `typed_ast`
   (coexist during migration)? (Recommendation: new `ast` module in a
   sub-directory `ast/`, old aliases moved to `legacy_ast.rs` during
   migration.)

5. **Trivia: leading vs leading+trailing:** Current design attaches
   trivia as leading-only (on the following token). Some tools prefer
   leading+trailing (e.g., trailing comment on same line belongs to the
   preceding node). Should we support trailing trivia?
   (Recommendation: leading-only for simplicity and consistency with
   current token layer; tools that need trailing-trivia association can
   compute it from positions.)

6. ~~**`PhantomData` on lifetime-less nodes:**~~ **RESOLVED.** Most
   nodes use `'src` via their `syntax` field, so no `PhantomData`
   needed. Nodes with no `'src`-using field (e.g. `DirectiveLocation`)
   simply drop the lifetime parameter entirely.
