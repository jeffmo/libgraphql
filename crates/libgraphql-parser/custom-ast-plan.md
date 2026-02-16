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
│  - GraphQLSourceSpan on every node                │
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

### Per-Node Span: `GraphQLSourceSpan`

Every AST node has a `pub span: GraphQLSourceSpan` field using the
existing `GraphQLSourceSpan` type (unchanged). This type already
carries start/end `SourcePosition` values (line, column, byte_offset)
and an optional file path, and is well-tested throughout the
parser/lexer pipeline.

No new span types are introduced. No `SourceMap` is required. The
span is directly accessible as a `pub` field on every node — no
accessor methods needed to read it.

> **Future optimization opportunity:** A compact `ByteSpan` (8 bytes)
> + `SourceMap` approach could reduce per-node span overhead from
> 104 bytes to 8 bytes. This is preserved as a detailed design in
> **Section 14: Future Optimization Opportunity (ByteSpan +
> SourceMap)** for potential exploration after the AST is working
> and profilable.

### `AstNode` Trait: Generic Source Access

All AST node types implement an `AstNode` trait via `#[inherent]`,
giving each node both inherent methods (no trait import needed) and a
trait bound for generic utilities (error formatters, linters, etc.).

The span is directly accessible as a `pub` field on every node, so
no span accessor methods are part of the trait. The trait provides
source reconstruction methods:

```rust
pub trait AstNode {
    /// Append this node's source representation to `sink`.
    /// Two reconstruction modes:
    /// - Source-slice mode: when `source` is `Some(s)`, slices
    ///   `&s[span.start_byte_offset..span.end_byte_offset]`
    ///   directly from the original source (zero-copy,
    ///   lossless).
    /// - Synthetic-formatting mode: when `source` is `None`,
    ///   reconstructs from semantic data (keywords, names,
    ///   values) with standard formatting (lossy but
    ///   semantically equivalent).
    fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    );

    /// Convenience: return this node as a source string.
    /// Default implementation delegates to append_source.
    fn to_source(
        &self,
        source: Option<&str>,
    ) -> String {
        let mut s = String::new();
        self.append_source(&mut s, source);
        s
    }
}
```

**Source reconstruction modes:**

- **Source-slice mode (fast, lossless):** When `source` is
  `Some(s)`, `append_source` slices
  `&s[span.start.byte_offset..span.end.byte_offset]`. This is the
  common path for `StrGraphQLTokenSource`. Zero allocation.
- **Synthetic-formatting mode (slower, lossy):** When `source` is
  `None` (e.g. `RustMacroGraphQLTokenSource`), `append_source`
  walks the AST and emits keywords, names, values, and punctuation
  with standard spacing. The output is semantically equivalent but
  not formatting-identical. Useful for debugging and proc-macro
  code generation.

**Why not a syntax-token walk mode?** A syntax-token walk would
reconstruct from `GraphQLToken` trivia and token text. However, it
only works correctly when ALL trivia types are enabled (including
whitespace). If `emit_whitespace_trivia = false`, the walk
produces tokens with no spacing. Since `append_source` cannot
inspect which trivia flags were set, this mode is unreliable.
The two cases where it would be needed — `source = None` with
whitespace trivia available — don't occur in practice
(`RustMacroGraphQLTokenSource` never has whitespace trivia). A
syntax-token walk mode can be revisited as a future enhancement
if a use case emerges.

Each struct node's `append_source` implementation is node-specific
in synthetic-formatting mode (each node type emits its own
keywords/punctuation/structure). The source-slice mode is uniform
across all types (slice from source text using span byte offsets):

```rust
#[inherent]
impl AstNode for ObjectTypeDefinition<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        // Source-slice or synthetic-formatting depending
        // on whether source is Some or None
    }
}
```

**Enum nodes** (e.g. `Definition`, `TypeDefinition`, `Value`,
`Selection`) implement `AstNode` via match-delegation to their
variant:

```rust
#[inherent]
impl AstNode for Definition<'_> {
    pub fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    ) {
        match self {
            Definition::SchemaDefinition(d) => {
                d.append_source(sink, source)
            },
            Definition::TypeDefinition(d) => {
                d.append_source(sink, source)
            },
            // ... etc for all variants
        }
    }
}
```

The `AstNode` trait enables generic utilities (formatters, linters,
etc.) that operate on any node via `append_source`/`to_source`.

**`#[inherent]` rationale:** The `inherent` crate (not yet a
dependency of `libgraphql-parser` — must be added in Phase 1)
makes trait methods callable as inherent methods on each concrete
type. Users calling `node.append_source(...)` or
`node.to_source(...)` directly don't need to import the `AstNode`
trait — they only import the trait when writing generic code
(`fn foo(x: &impl AstNode)`).

**Implementation note:** Each type gets an explicit
`#[inherent] impl AstNode` block rather than using `macro_rules!`
or a derive macro. This is more verbose but makes the codebase
easier to navigate — a reader can find any type's `AstNode` impl
directly without tracing through macro expansion.

### Trivia Storage: `SmallVec` Optimization

`GraphQLToken` continues to use
`SmallVec<[GraphQLTriviaToken<'src>; 2]>` (via the existing
`GraphQLTriviaTokenVec<'src>` type alias) for leading trivia
storage. With the addition of `Whitespace` trivia, typical
distribution is:

- 0 items (~5–10% of tokens): tokens immediately after others
- 1 item (~70–80%): most tokens just have whitespace before them
- 2 items (~15–20%): comma + whitespace, or comment + whitespace
- 3+ items (~2–5%): comma + comment + whitespace — rare

Capacity 2 covers ~95% of tokens inline (no heap allocation).
Increasing to 3 adds +88 bytes per token to cover only 2–5% of
cases — the heap allocation cost for those rare tokens is cheaper.
If profiling shows >15% heap spillage, capacity can be increased.

---

## 4. String Representation

### `Cow<'src, str>` for All String Data

All name identifiers, string literal values, descriptions, and enum
values use `Cow<'src, str>`:

```rust
pub struct Name<'src> {
    pub value: Cow<'src, str>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<NameSyntax<'src>>,
}

pub struct NameSyntax<'src> {
    pub token: GraphQLToken<'src>,
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
or via the syntax layer's `GraphQLToken` when retained.

#### StringValue

```rust
pub struct StringValue<'src> {
    /// The processed string value after escape-sequence
    /// resolution and block-string indentation stripping.
    /// Borrows from the source when no transformation was
    /// needed (simple quoted string with no escapes, or a
    /// block string whose processed result is a contiguous
    /// substring of the source text); owned when escapes
    /// were resolved or block-string stripping produced a
    /// non-contiguous result.
    pub value: Cow<'src, str>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<StringValueSyntax<'src>>,
}
```

#### IntValue

The [GraphQL spec](https://spec.graphql.org/September2025/#sec-Int)
defines Int as a signed 32-bit integer. The spec does not mandate
overflow behavior, but our parser treats overflow/underflow as a
`ParseError` and error-recovers by clamping to `i32::MAX` /
`i32::MIN` respectively so the AST is always structurally
complete. These are the only two failure modes — a lexed
`GraphQLTokenKind::Int` token is necessarily `-?[0-9]+` (leading
zeros already rejected by the lexer), so no other parse errors
are possible.

```rust
pub struct IntValue<'src> {
    /// The parsed 32-bit integer value. On overflow/underflow
    /// the parser emits a diagnostic and clamps to
    /// i32::MAX / i32::MIN.
    pub value: i32,
    pub span: GraphQLSourceSpan,
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
    pub span: GraphQLSourceSpan,
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
    pub span: GraphQLSourceSpan,
    pub syntax: Option<DocumentSyntax<'src>>,
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
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub root_operations: Vec<RootOperationTypeDefinition<'src>>,
    pub syntax: Option<SchemaDefinitionSyntax<'src>>,
}

pub struct RootOperationTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
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
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<ScalarTypeDefinitionSyntax<'src>>,
}

pub struct ObjectTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub implements: Vec<Name<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub syntax: Option<ObjectTypeDefinitionSyntax<'src>>,
}

pub struct InterfaceTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub implements: Vec<Name<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub syntax: Option<InterfaceTypeDefinitionSyntax<'src>>,
}

pub struct UnionTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub members: Vec<Name<'src>>,
    pub syntax: Option<UnionTypeDefinitionSyntax<'src>>,
}

pub struct EnumTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub values: Vec<EnumValueDefinition<'src>>,
    pub syntax: Option<EnumTypeDefinitionSyntax<'src>>,
}

pub struct InputObjectTypeDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<InputValueDefinition<'src>>,
    pub syntax:
        Option<InputObjectTypeDefinitionSyntax<'src>>,
}

pub struct DirectiveDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub arguments: Vec<InputValueDefinition<'src>>,
    pub repeatable: bool,
    pub locations: Vec<DirectiveLocation<'src>>,
    pub syntax: Option<DirectiveDefinitionSyntax<'src>>,
}

/// Directive location with span (unlike graphql_parser which
/// uses a plain enum).
pub struct DirectiveLocation<'src> {
    pub value: DirectiveLocationKind,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<DirectiveLocationSyntax<'src>>,
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
    pub span: GraphQLSourceSpan,
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
// minus description, plus span.

pub struct ScalarTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<ScalarTypeExtensionSyntax<'src>>,
}

pub struct ObjectTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub implements: Vec<Name<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub syntax: Option<ObjectTypeExtensionSyntax<'src>>,
}

pub struct InterfaceTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub implements: Vec<Name<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<FieldDefinition<'src>>,
    pub syntax:
        Option<InterfaceTypeExtensionSyntax<'src>>,
}

pub struct UnionTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub members: Vec<Name<'src>>,
    pub syntax: Option<UnionTypeExtensionSyntax<'src>>,
}

pub struct EnumTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub values: Vec<EnumValueDefinition<'src>>,
    pub syntax: Option<EnumTypeExtensionSyntax<'src>>,
}

pub struct InputObjectTypeExtension<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub fields: Vec<InputValueDefinition<'src>>,
    pub syntax:
        Option<InputObjectTypeExtensionSyntax<'src>>,
}
```

### 5.4 Executable Definitions

```rust
pub struct OperationDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
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
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub type_condition: TypeCondition<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub selection_set: SelectionSet<'src>,
    pub syntax: Option<FragmentDefinitionSyntax<'src>>,
}

pub struct VariableDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
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
    pub span: GraphQLSourceSpan,
    pub selections: Vec<Selection<'src>>,
    pub syntax: Option<SelectionSetSyntax<'src>>,
}

pub enum Selection<'src> {
    Field(Field<'src>),
    FragmentSpread(FragmentSpread<'src>),
    InlineFragment(InlineFragment<'src>),
}

pub struct Field<'src> {
    pub span: GraphQLSourceSpan,
    pub alias: Option<Name<'src>>,
    pub name: Name<'src>,
    pub arguments: Vec<Argument<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub selection_set: Option<SelectionSet<'src>>,
    pub syntax: Option<FieldSyntax<'src>>,
}

pub struct FragmentSpread<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<FragmentSpreadSyntax<'src>>,
}

pub struct InlineFragment<'src> {
    pub span: GraphQLSourceSpan,
    pub type_condition: Option<TypeCondition<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub selection_set: SelectionSet<'src>,
    pub syntax: Option<InlineFragmentSyntax<'src>>,
}
```

### 5.6 Shared Sub-Nodes

```rust
pub struct FieldDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub arguments: Vec<InputValueDefinition<'src>>,
    pub field_type: TypeAnnotation<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax: Option<FieldDefinitionSyntax<'src>>,
}

pub struct InputValueDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub value_type: TypeAnnotation<'src>,
    pub default_value: Option<Value<'src>>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
    pub syntax:
        Option<InputValueDefinitionSyntax<'src>>,
}

pub struct EnumValueDefinition<'src> {
    pub span: GraphQLSourceSpan,
    pub description: Option<StringValue<'src>>,
    pub name: Name<'src>,
    pub directives: Vec<DirectiveAnnotation<'src>>,
}

pub struct DirectiveAnnotation<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub arguments: Vec<Argument<'src>>,
    pub syntax: Option<DirectiveAnnotationSyntax<'src>>,
}

pub struct Argument<'src> {
    pub span: GraphQLSourceSpan,
    pub name: Name<'src>,
    pub value: Value<'src>,
    pub syntax: Option<ArgumentSyntax<'src>>,
}

pub struct TypeCondition<'src> {
    pub span: GraphQLSourceSpan,
    pub named_type: Name<'src>,
    pub syntax: Option<TypeConditionSyntax<'src>>,
}
```

### 5.7 Type Annotations

The spec grammar has three type productions (`NamedType`, `ListType`,
`NonNullType`), but `NonNullType` is purely a wrapper that adds `!`.
Rather than model it as a recursive enum variant — which would allow
redundant same-level wrapping like `NonNull(NonNull(...))` — we
flatten nullability into a `Nullability` field on each concrete type
annotation node. Multi-level NonNull (e.g. `[String!]!`) is fully
supported: the inner `String!` is the list's `element_type` (a
separate `TypeAnnotation` with its own `Nullability`), and the outer
`!` is on the `ListTypeAnnotation` — different nesting levels.

The `Nullability` enum owns the `!` token directly in its `NonNull`
variant, making it impossible for nullability semantics and syntax to
disagree (e.g. a non-null annotation missing its `!` token or a
nullable annotation carrying one). Splitting this into a separate
boolean + optional token would re-introduce the invalid-state
problem the design prevents.

- `NamedTypeAnnotation.span` covers the full annotation including `!`
  when present. The underlying name span is available via
  `NamedTypeAnnotation.name.span`.
- `ListTypeAnnotation.span` likewise covers brackets and trailing `!`.

```rust
pub enum Nullability<'src> {
    Nullable,
    NonNull {
        /// The `!` token. Present when syntax detail is retained.
        syntax: Option<GraphQLToken<'src>>,
    },
}

pub enum TypeAnnotation<'src> {
    Named(NamedTypeAnnotation<'src>),
    List(ListTypeAnnotation<'src>),
}

pub struct NamedTypeAnnotation<'src> {
    pub name: Name<'src>,
    pub nullability: Nullability<'src>,
    pub span: GraphQLSourceSpan,
}

pub struct ListTypeAnnotation<'src> {
    pub element_type: Box<TypeAnnotation<'src>>,
    pub nullability: Nullability<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<ListTypeAnnotationSyntax<'src>>,
}
```

### 5.8 Values

```rust
pub enum Value<'src> {
    Variable(VariableValue<'src>),
    Int(IntValue<'src>),
    Float(FloatValue<'src>),
    String(StringValue<'src>),
    Boolean(BooleanValue<'src>),
    Null(NullValue<'src>),
    Enum(EnumValue<'src>),
    List(ListValue<'src>),
    Object(ObjectValue<'src>),
}

pub struct VariableValue<'src> {
    pub name: Name<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<VariableValueSyntax<'src>>,
}

pub struct BooleanValue<'src> {
    pub value: bool,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<BooleanValueSyntax<'src>>,
}

pub struct NullValue<'src> {
    pub span: GraphQLSourceSpan,
    pub syntax: Option<NullValueSyntax<'src>>,
}

pub struct EnumValue<'src> {
    pub value: Cow<'src, str>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<EnumValueSyntax<'src>>,
}

pub struct ListValue<'src> {
    pub values: Vec<Value<'src>>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<ListValueSyntax<'src>>,
}

pub struct ObjectValue<'src> {
    pub fields: Vec<ObjectField<'src>>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<ObjectValueSyntax<'src>>,
}

pub struct ObjectField<'src> {
    pub name: Name<'src>,
    pub value: Value<'src>,
    pub span: GraphQLSourceSpan,
    pub syntax: Option<ObjectFieldSyntax<'src>>,
}
```

---

## 6. Syntax Layer (Optional Trivia & Token Detail)

### Design

Each AST node has an `Option<XyzSyntax<'src>>` field. When the parser
is configured to retain syntax detail, this field is `Some(...)` and
contains all punctuation tokens, keywords, and trivia. When syntax
detail is disabled, the field is `None`.

### Syntax Detail Struct Pattern

```rust
/// A matched pair of delimiter tokens (parentheses, brackets,
/// or braces). Bundled into one struct so that an open
/// delimiter without a matching close is unrepresentable.
pub struct DelimiterPair<'src> {
    pub open: GraphQLToken<'src>,
    pub close: GraphQLToken<'src>,
}

/// Syntax tokens for an object type definition:
///   "type" Name ImplementsInterfaces? Directives?
///       FieldsDefinition?
pub struct ObjectTypeDefinitionSyntax<'src> {
    pub type_keyword: GraphQLToken<'src>,
    pub implements_keyword: Option<GraphQLToken<'src>>,
    pub leading_ampersand: Option<GraphQLToken<'src>>,
    pub ampersands: Vec<GraphQLToken<'src>>,
    pub braces: Option<DelimiterPair<'src>>,
}
```

### Trivia in Comma-Separated Lists

**Design principle:** Every source token in the document — including
value literals, names, keywords, and punctuation — has a
corresponding `GraphQLToken` somewhere in the syntax layer. This ensures
the leading-trivia model is perfectly consistent: trivia (whitespace,
comments, commas) always attaches as `leading_trivia` on the
`GraphQLToken` of the next source token in document order. No trivia is
ever orphaned.

For comma-separated constructs (list values, arguments, object fields,
etc.), this means commas appear as `GraphQLTriviaToken::Comma` items in
the `leading_trivia` of the following item's `GraphQLToken`. No special
`infix_commas` vec is needed.

To make this work, every semantic value node has a `*Syntax` struct
containing a `GraphQLToken` for its source token:

```rust
pub struct IntValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}
pub struct FloatValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}
pub struct StringValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}
pub struct BooleanValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}
pub struct NullValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}
pub struct EnumValueSyntax<'src> {
    pub token: GraphQLToken<'src>,
}
```

And container syntax structs only need their delimiter tokens:

```rust
pub struct ListValueSyntax<'src> {
    pub brackets: DelimiterPair<'src>,
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
            value: 1,
            span: GraphQLSourceSpan { /* bytes 1..2 */ },
            syntax: Some(IntValueSyntax {
                token: GraphQLToken {
                    span: GraphQLSourceSpan { /* bytes 1..2 */ },
                    leading_trivia: [],
                },
            }),
        }),
        Value::Int(IntValue {
            value: 2,
            span: GraphQLSourceSpan { /* bytes 4..5 */ },
            syntax: Some(IntValueSyntax {
                token: GraphQLToken {
                    span: GraphQLSourceSpan { /* bytes 4..5 */ },
                    // Comma + space before "2"
                    leading_trivia: [
                        GraphQLTriviaToken::Comma {
                            span: GraphQLSourceSpan {
                                /* bytes 2..3 */
                            },
                        },
                        GraphQLTriviaToken::Whitespace {
                            text: " ",
                            span: GraphQLSourceSpan {
                                /* bytes 3..4 */
                            },
                        },
                    ],
                },
            }),
        }),
        Value::Int(IntValue {
            value: 3,
            span: GraphQLSourceSpan { /* bytes 7..8 */ },
            syntax: Some(IntValueSyntax {
                token: GraphQLToken {
                    span: GraphQLSourceSpan { /* bytes 7..8 */ },
                    // Comma + space before "3"
                    leading_trivia: [
                        GraphQLTriviaToken::Comma {
                            span: GraphQLSourceSpan {
                                /* bytes 5..6 */
                            },
                        },
                        GraphQLTriviaToken::Whitespace {
                            text: " ",
                            span: GraphQLSourceSpan {
                                /* bytes 6..7 */
                            },
                        },
                    ],
                },
            }),
        }),
    ],
    span: GraphQLSourceSpan { /* bytes 0..9 */ },
    syntax: Some(ListValueSyntax {
        brackets: DelimiterPair {
            open: GraphQLToken {
                span: GraphQLSourceSpan { /* byte 0..1 */ },
                leading_trivia: [],
            },
            close: GraphQLToken {
                span: GraphQLSourceSpan { /* bytes 8..9 */ },
                leading_trivia: [],
            },
        },
    }),
}
```

Every token has exactly one `GraphQLToken` home. The commas at bytes 2
and 5 are `GraphQLTriviaToken::Comma` in the `leading_trivia` of the
next value's `GraphQLToken`. The spaces at bytes 3 and 6 follow the
commas in the same `leading_trivia` vec. The closing bracket has no
leading trivia because `3` is immediately followed by `]`.

#### Example 2: Argument list `(x: 1, y: 2)`

```
 Byte:  0  1  2  3  4  5  6  7  8  9  10  11
 Char:  (  x  :     1  ,     y  :     2   )
```

The relevant syntax structs:

```rust
pub struct ArgumentSyntax<'src> {
    pub colon: GraphQLToken<'src>,
    // The argument's name token lives at
    // argument.name.syntax.unwrap().token.
    // The argument's value carries its own *ValueSyntax
    // with a GraphQLToken — trivia before the value (e.g.,
    // the space between ":" and the value) lands there.
}
```

Suppose these arguments belong to a `Field`. The `FieldSyntax`
holds the parentheses; each `Argument`'s name carries its own
`NameSyntax` token; each `ArgumentSyntax` holds the colon; and
each argument's value holds its own value token:

```rust
// FieldSyntax (partial — just the argument delimiters):
FieldSyntax {
    parens: Some(DelimiterPair {
        open: GraphQLToken {
            span: GraphQLSourceSpan { /* bytes 0..1 */ },
            leading_trivia: [],
        },
        close: GraphQLToken {
            span: GraphQLSourceSpan { /* bytes 11..12 */ },
            leading_trivia: [],
        },
    }),
    // ...
}

// arguments[0]: x: 1
Argument {
    name: Name {
        value: "x",
        span: GraphQLSourceSpan { /* bytes 1..2 */ },
        syntax: Some(NameSyntax {
            token: GraphQLToken {
                span: GraphQLSourceSpan { /* bytes 1..2 */ },
                leading_trivia: [],
            },
        }),
    },
    value: Value::Int(IntValue {
        value: 1,
        span: GraphQLSourceSpan { /* bytes 4..5 */ },
        syntax: Some(IntValueSyntax {
            token: GraphQLToken {
                span: GraphQLSourceSpan { /* bytes 4..5 */ },
                // Space between ":" and "1"
                leading_trivia: [
                    GraphQLTriviaToken::Whitespace {
                        text: " ",
                        span: GraphQLSourceSpan {
                            /* bytes 3..4 */
                        },
                    },
                ],
            },
        }),
    }),
    syntax: Some(ArgumentSyntax {
        colon: GraphQLToken {
            span: GraphQLSourceSpan { /* bytes 2..3 */ },
            leading_trivia: [],
        },
    }),
}

// arguments[1]: y: 2
Argument {
    name: Name {
        value: "y",
        span: GraphQLSourceSpan { /* bytes 7..8 */ },
        syntax: Some(NameSyntax {
            token: GraphQLToken {
                span: GraphQLSourceSpan { /* bytes 7..8 */ },
                // Comma + space between "1" and "y"
                leading_trivia: [
                    GraphQLTriviaToken::Comma {
                        span: GraphQLSourceSpan {
                            /* bytes 5..6 */
                        },
                    },
                    GraphQLTriviaToken::Whitespace {
                        text: " ",
                        span: GraphQLSourceSpan {
                            /* bytes 6..7 */
                        },
                    },
                ],
            },
        }),
    },
    value: Value::Int(IntValue {
        value: 2,
        span: GraphQLSourceSpan { /* bytes 10..11 */ },
        syntax: Some(IntValueSyntax {
            token: GraphQLToken {
                span: GraphQLSourceSpan { /* bytes 10..11 */ },
                // Space between ":" and "2"
                leading_trivia: [
                    GraphQLTriviaToken::Whitespace {
                        text: " ",
                        span: GraphQLSourceSpan {
                            /* bytes 9..10 */
                        },
                    },
                ],
            },
        }),
    }),
    syntax: Some(ArgumentSyntax {
        colon: GraphQLToken {
            span: GraphQLSourceSpan { /* bytes 8..9 */ },
            leading_trivia: [],
        },
    }),
}
```

Same pattern: the comma at byte 5 is leading trivia on the second
argument's `NameSyntax` token. The space at byte 6 follows it. Trivia
between `:` and the value (bytes 3 and 9) is leading trivia on the
value's `IntValueSyntax.token`.

#### Summary

The invariant is simple: **every piece of trivia is leading trivia on
the `GraphQLToken` of the next source token in document order.** Because
every semantic node that corresponds to a source token has a
`*Syntax` struct with a `GraphQLToken`, no trivia is ever orphaned. This
generalizes to all comma-separated constructs (arguments, variable
definitions, enum values, object fields, etc.) without any special
`infix_commas` machinery.

### Syntax Tokens: Reuse `GraphQLToken` Directly

**No separate `GraphQLToken` type.** `*Syntax` structs store
`GraphQLToken<'src>` directly. The `kind` field is technically
redundant (the field name in the parent struct — e.g. `braces`,
`colon` — already identifies the token), but the overhead is
negligible for punctuator variants (zero-payload enum discriminant)
and actively useful for value tokens (carries the raw source text).
The big win: the parser can **move** each `GraphQLToken` straight
from the token stream into the syntax struct with zero conversion.

```rust
pub struct ArgumentSyntax<'src> {
    pub colon: GraphQLToken<'src>,
}
pub struct ListValueSyntax<'src> {
    pub brackets: DelimiterPair<'src>,
}
```

**Unified trivia model:** `GraphQLTriviaToken` is expanded with a
`Whitespace` variant:

```rust
pub enum GraphQLTriviaToken<'src> {
    Whitespace {
        /// The whitespace text (spaces, tabs, newlines).
        text: Cow<'src, str>,
        /// The source location of the whitespace.
        span: GraphQLSourceSpan,
    },
    Comment {
        /// The comment text (excluding the leading #).
        value: Cow<'src, str>,
        /// The source location of the comment.
        span: GraphQLSourceSpan,
    },
    Comma {
        /// The source location of the comma.
        span: GraphQLSourceSpan,
    },
}
```

The lexer currently emits `Comment` and `Comma` trivia but skips
whitespace. Trivia recording is controlled by **per-type flags** on
a dedicated `GraphQLTokenSourceConfig` struct:

```rust
/// Lexer-level configuration controlling which trivia types
/// are emitted. All flags default to `true`.
pub struct GraphQLTokenSourceConfig {
    /// When true, whitespace runs between tokens are recorded
    /// as `GraphQLTriviaToken::Whitespace`.
    pub emit_whitespace_trivia: bool,

    /// When true, `#`-comments are recorded as
    /// `GraphQLTriviaToken::Comment`.
    pub emit_comment_trivia: bool,

    /// When true, commas are recorded as
    /// `GraphQLTriviaToken::Comma`.
    pub emit_comma_trivia: bool,
}

impl Default for GraphQLTokenSourceConfig {
    fn default() -> Self {
        Self {
            emit_whitespace_trivia: true,
            emit_comment_trivia: true,
            emit_comma_trivia: true,
        }
    }
}
```

The `GraphQLTokenSource` trait does not prescribe a constructor
signature — each token source defines its own constructor.
`StrGraphQLTokenSource::new()` accepts a
`GraphQLTokenSourceConfig`; `RustMacroGraphQLTokenSource::new()`
accepts a `TokenStream` (as it does today).

Each flag independently controls its trivia type:

- `emit_whitespace_trivia`: records whitespace runs (spaces, tabs,
  newlines, BOM) as `GraphQLTriviaToken::Whitespace`
- `emit_comment_trivia`: records `#`-comments as
  `GraphQLTriviaToken::Comment`
- `emit_comma_trivia`: records commas as `GraphQLTriviaToken::Comma`

All three flags default to `true` — all trivia is recorded unless
explicitly disabled. This is consistent with the current behavior
where `Comment` and `Comma` trivia are always emitted, and adds
`Whitespace` trivia recording by default. Callers who want leaner
tokens can set individual flags to `false`.

`RustMacroGraphQLTokenSource` does not accept a
`GraphQLTokenSourceConfig` — Rust's tokenizer strips comments
and whitespace, so trivia flags are inapplicable. It will
continue to emit comma trivia unconditionally (as it does
today). A future follow-on can add an optional config to its
`::new()` when whitespace synthesis support is implemented.

### Trivia Attachment Strategy

Trivia is attached as **leading trivia** on the following token (same
as the current `GraphQLToken::preceding_trivia` design). This means:

- Trivia before the first token of a node is stored on that token
- Trivia after the last token of a definition is stored on the first
  token of the *next* definition (or lost if at EOF)
- **Inter-definition trivia:** Comments and whitespace between two
  top-level definitions attach as leading trivia on the *following*
  definition's first token. This is simpler than heuristic-split
  approaches (which try to assign trailing comments to the preceding
  definition) and avoids ambiguity about whether a comment "belongs"
  to the definition above or below it. Formatters and codegen tools
  that need to associate comments with specific definitions can use
  span proximity heuristics at a higher layer.
- **EOF trivia:** Trailing trivia at end-of-file is stored on
  `DocumentSyntax.trailing_trivia` (inside `Document.syntax`)

### Source Reconstruction

With the syntax layer enabled, lossless source reconstruction is
possible by walking the AST and emitting:
1. Leading trivia of each syntax token
2. The token text (derived from span + source text, or from the
   semantic value for names/strings)
3. Repeat for all tokens in document order

Calling `doc.to_source(Some(source))` exercises this walk and serves
as a correctness test (round-trip: parse → `to_source` → compare).

**Zero-length span fallback:** Nodes with a zero-length byte-span
(i.e., `start_inclusive.byte_offset == end_exclusive.byte_offset`)
signal that the node was synthesized rather than parsed from source
text (e.g., created by `from_*` conversion or error recovery).
`append_source()` should detect this condition and fall back to
synthetic-formatting mode for such nodes, emitting text derived
from semantic values rather than slicing source text.

### Complete Syntax Struct Catalog

Every `syntax: Option<XyzSyntax<'src>>` field referenced in Section 5
has a corresponding struct defined here. Grouped by category.

`DelimiterPair<'src>` (defined earlier in this section) is used for
all matched open/close delimiter pairs (parentheses, brackets,
braces).

#### Type System Definition Syntax

```rust
pub struct DocumentSyntax<'src> {
    /// Trailing trivia at end-of-file (after the last
    /// definition). Trivia that would otherwise be lost.
    pub trailing_trivia: Vec<GraphQLTriviaToken<'src>>,
}

pub struct SchemaDefinitionSyntax<'src> {
    pub schema_keyword: GraphQLToken<'src>,
    pub braces: DelimiterPair<'src>,
}

pub struct RootOperationTypeDefinitionSyntax<'src> {
    pub colon: GraphQLToken<'src>,
}

pub struct ScalarTypeDefinitionSyntax<'src> {
    pub scalar_keyword: GraphQLToken<'src>,
}

/// Already shown as the example pattern earlier in this
/// section — included here for catalog completeness.
pub struct ObjectTypeDefinitionSyntax<'src> {
    pub type_keyword: GraphQLToken<'src>,
    pub implements_keyword: Option<GraphQLToken<'src>>,
    pub leading_ampersand: Option<GraphQLToken<'src>>,
    pub ampersands: Vec<GraphQLToken<'src>>,
    pub braces: Option<DelimiterPair<'src>>,
}

pub struct InterfaceTypeDefinitionSyntax<'src> {
    pub interface_keyword: GraphQLToken<'src>,
    pub implements_keyword: Option<GraphQLToken<'src>>,
    pub leading_ampersand: Option<GraphQLToken<'src>>,
    pub ampersands: Vec<GraphQLToken<'src>>,
    pub braces: Option<DelimiterPair<'src>>,
}

pub struct UnionTypeDefinitionSyntax<'src> {
    pub union_keyword: GraphQLToken<'src>,
    pub equals: Option<GraphQLToken<'src>>,
    pub leading_pipe: Option<GraphQLToken<'src>>,
    pub pipes: Vec<GraphQLToken<'src>>,
}

pub struct EnumTypeDefinitionSyntax<'src> {
    pub enum_keyword: GraphQLToken<'src>,
    pub braces: Option<DelimiterPair<'src>>,
}

pub struct InputObjectTypeDefinitionSyntax<'src> {
    pub input_keyword: GraphQLToken<'src>,
    pub braces: Option<DelimiterPair<'src>>,
}

pub struct DirectiveDefinitionSyntax<'src> {
    pub directive_keyword: GraphQLToken<'src>,
    pub at_sign: GraphQLToken<'src>,
    pub parens: Option<DelimiterPair<'src>>,
    pub repeatable_keyword: Option<GraphQLToken<'src>>,
    pub on_keyword: GraphQLToken<'src>,
}

pub struct DirectiveLocationSyntax<'src> {
    /// The `|` pipe token before this location (None for
    /// the first location).
    pub pipe: Option<GraphQLToken<'src>>,
    /// The location name token (e.g. `FIELD`, `QUERY`).
    pub token: GraphQLToken<'src>,
}
```

#### Type Extension Syntax

Each mirrors its definition counterpart (no description token),
with an additional `extend_keyword`.

```rust
pub struct SchemaExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub schema_keyword: GraphQLToken<'src>,
    pub braces: Option<DelimiterPair<'src>>,
}

pub struct ScalarTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub scalar_keyword: GraphQLToken<'src>,
}

pub struct ObjectTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub type_keyword: GraphQLToken<'src>,
    pub implements_keyword: Option<GraphQLToken<'src>>,
    pub leading_ampersand: Option<GraphQLToken<'src>>,
    pub ampersands: Vec<GraphQLToken<'src>>,
    pub braces: Option<DelimiterPair<'src>>,
}

pub struct InterfaceTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub interface_keyword: GraphQLToken<'src>,
    pub implements_keyword: Option<GraphQLToken<'src>>,
    pub leading_ampersand: Option<GraphQLToken<'src>>,
    pub ampersands: Vec<GraphQLToken<'src>>,
    pub braces: Option<DelimiterPair<'src>>,
}

pub struct UnionTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub union_keyword: GraphQLToken<'src>,
    pub equals: Option<GraphQLToken<'src>>,
    pub leading_pipe: Option<GraphQLToken<'src>>,
    pub pipes: Vec<GraphQLToken<'src>>,
}

pub struct EnumTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub enum_keyword: GraphQLToken<'src>,
    pub braces: Option<DelimiterPair<'src>>,
}

pub struct InputObjectTypeExtensionSyntax<'src> {
    pub extend_keyword: GraphQLToken<'src>,
    pub input_keyword: GraphQLToken<'src>,
    pub braces: Option<DelimiterPair<'src>>,
}
```

#### Executable Syntax

```rust
pub struct OperationDefinitionSyntax<'src> {
    /// The operation keyword (`query`, `mutation`,
    /// `subscription`). None for shorthand queries.
    pub operation_keyword: Option<GraphQLToken<'src>>,
    pub parens: Option<DelimiterPair<'src>>,
}

pub struct FragmentDefinitionSyntax<'src> {
    pub fragment_keyword: GraphQLToken<'src>,
    pub on_keyword: GraphQLToken<'src>,
}

pub struct VariableDefinitionSyntax<'src> {
    pub dollar: GraphQLToken<'src>,
    pub colon: GraphQLToken<'src>,
    pub equals: Option<GraphQLToken<'src>>,
}

pub struct SelectionSetSyntax<'src> {
    pub braces: DelimiterPair<'src>,
}
```

#### Selection Syntax

```rust
pub struct FieldSyntax<'src> {
    /// The colon between alias and field name. None when
    /// no alias is present.
    pub alias_colon: Option<GraphQLToken<'src>>,
    pub parens: Option<DelimiterPair<'src>>,
}

pub struct FragmentSpreadSyntax<'src> {
    pub ellipsis: GraphQLToken<'src>,
}

pub struct InlineFragmentSyntax<'src> {
    pub ellipsis: GraphQLToken<'src>,
}
```

#### Shared Sub-Node Syntax

```rust
pub struct NameSyntax<'src> {
    pub token: GraphQLToken<'src>,
}

pub struct FieldDefinitionSyntax<'src> {
    pub colon: GraphQLToken<'src>,
    pub parens: Option<DelimiterPair<'src>>,
}

pub struct InputValueDefinitionSyntax<'src> {
    pub colon: GraphQLToken<'src>,
    pub equals: Option<GraphQLToken<'src>>,
}

pub struct DirectiveAnnotationSyntax<'src> {
    pub at_sign: GraphQLToken<'src>,
    pub parens: Option<DelimiterPair<'src>>,
}

pub struct TypeConditionSyntax<'src> {
    pub on_keyword: GraphQLToken<'src>,
}

pub struct ListTypeAnnotationSyntax<'src> {
    pub brackets: DelimiterPair<'src>,
}
```

#### Value Syntax

`IntValueSyntax`, `FloatValueSyntax`, `StringValueSyntax`,
`BooleanValueSyntax`, `NullValueSyntax`, `EnumValueSyntax`, and
`ListValueSyntax` are already defined earlier in this section.
The remaining value syntax structs:

```rust
pub struct ObjectValueSyntax<'src> {
    pub braces: DelimiterPair<'src>,
}

pub struct ObjectFieldSyntax<'src> {
    pub colon: GraphQLToken<'src>,
}

pub struct VariableValueSyntax<'src> {
    pub dollar: GraphQLToken<'src>,
}
```

---

## 7. Parser Flags / Configuration

Configuration is split into two structs reflecting the two layers
of the pipeline: **lexer** (token source) and **parser**.

### `GraphQLTokenSourceConfig` (lexer-level)

Defined in Section 6. Controls which trivia types the lexer emits.
All flags default to `true`.

### `GraphQLParserConfig` (parser-level)

```rust
/// Parser-level configuration. Controls AST construction
/// behavior that is independent of the token source.
pub struct GraphQLParserConfig {
    /// When true, the parser populates `syntax` fields on AST
    /// nodes with keyword/punctuation tokens and their trivia.
    /// Default: true.
    pub retain_syntax: bool,

    // Future expansion:
    // pub max_recursion_depth: Option<usize>,
    // pub max_string_literal_size: Option<usize>,
    // pub spec_version: SpecVersion,
}

impl Default for GraphQLParserConfig {
    fn default() -> Self {
        Self {
            retain_syntax: true,
        }
    }
}
```

### Parser Constructors

The parser has three constructors for different levels of control:

```rust
impl<'src> GraphQLParser<'src, StrGraphQLTokenSource<'src>> {
    /// Convenience constructor. Uses default configs, which
    /// give full-fidelity mode (all trivia flags and
    /// `retain_syntax` are `true`). Use `new_with_configs()`
    /// or `from_token_source()` to customize.
    pub fn new(source: &'src str) -> Self;

    /// Full control over both lexer and parser configuration.
    /// The parser creates a `StrGraphQLTokenSource` internally
    /// using the provided `token_source_config`.
    pub fn new_with_configs(
        source: &'src str,
        token_source_config: GraphQLTokenSourceConfig,
        parser_config: GraphQLParserConfig,
    ) -> Self;
}

impl<'src, S: GraphQLTokenSource<'src>>
    GraphQLParser<'src, S>
{
    /// Accepts a pre-configured token source directly.
    /// Use this when you need custom token source setup
    /// or when working with `RustMacroGraphQLTokenSource`.
    pub fn from_token_source(
        token_source: S,
        parser_config: GraphQLParserConfig,
    ) -> Self;
}
```

**Design rationale:** Trivia flags are a lexer concern
(`GraphQLTokenSourceConfig`), while `retain_syntax` is a parser
concern (`GraphQLParserConfig`). This separation means:
- Token sources can be configured and tested independently
- The parser doesn't need to know about lexer internals
- `from_token_source()` works with any pre-configured token
  source (including `RustMacroGraphQLTokenSource`)

---

## 8. FFI Strategy

### Principles

1. **Opaque types** with accessor functions (standard Rust FFI pattern)
2. **`#[repr(C)]` on leaf types** that cross the boundary directly
   (index types, enums without data). **Note:** The current AST
   uses `GraphQLSourceSpan`, which is not `#[repr(C)]`. The FFI
   layer will need a trivial conversion at the boundary: extract
   `byte_offset` from start/end `SourcePosition`s into a C-friendly
   `{ uint32_t start; uint32_t end; }` struct. Alternatively, the
   full ByteSpan migration from Section 14 can be pursued first,
   which would make spans directly `#[repr(C)]`
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
    //   (GraphQLSourceSpan lives inside Document nodes)
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

**DECIDED:** Two-handle API (`Source` + `Document`) for Phase 7.
`OwnedDocument` (self-referential owned wrapper) is a possible
follow-on if C users find two-handle lifetime management
error-prone.

### `repr(C)` Types

```c
// C header (auto-generated)
// Span converted from GraphQLSourceSpan at the FFI boundary
// (extracts byte_offset from start/end SourcePositions).
// If the ByteSpan optimization (Section 14) is pursued, this
// struct can be used directly without conversion.
typedef struct {
    uint32_t start;
    uint32_t end;
} GraphQLByteSpan;

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
GraphQLByteSpan graphql_document_definition_span(
    const GraphQLDocument* doc, size_t index
);
// ... etc for each node type and field ...
```

### FFI Code Generation

Consider using `cbindgen` to auto-generate C headers from Rust types
annotated with `#[repr(C)]`. For the accessor-function pattern,
a proc-macro or build script could generate the boilerplate.

### Phase 7 Planning Topics

The following design topics should be resolved at the start of Phase 7
(FFI implementation) before writing code:

- **Error handling patterns:** How do FFI accessor functions signal
  errors (null returns, result codes, error-info structs)? How are
  `ParseResult` errors exposed to C?
- **Thread safety:** What threading guarantees does the FFI layer
  provide? Is a parsed `Document` safe to read from multiple threads?
  (Likely yes — it's immutable after parsing.) Are there any
  `Send`/`Sync` concerns with the opaque pointers?

---

## 9. Conversion Layer

Each external parser's conversion utilities live in a standalone
`compat_*` module, gated by a versioned feature flag. This keeps
external parser dependencies optional and makes version upgrades
explicit.

### 9.1 Module & Feature Flag Structure

**`Cargo.toml` features:**

```toml
[features]
compat-graphql-parser-v0.4 = ["dep:graphql-parser"]
compat-apollo-parser-v0.8 = ["dep:apollo-parser"]
# Future:
# compat-graphql-query-v0.X = ["dep:graphql_query"]
```

**`lib.rs` (or crate root):**

```rust
#[cfg(feature = "compat-graphql-parser-v0.4")]
pub mod compat_graphql_parser_v0_4;

#[cfg(feature = "compat-apollo-parser-v0.8")]
pub mod compat_apollo_parser_v0_8;
```

### 9.2 `compat_graphql_parser_v0_4`

Feature: `compat-graphql-parser-v0.4`

```rust
// compat_graphql_parser_v0_4.rs

/// Convert our Document to a graphql_parser schema AST.
/// Drops: trivia, syntax tokens, variable directives,
///        schema extensions.
/// GraphQLSourceSpan → Pos conversion uses the span's
/// line/col fields directly.
pub fn to_graphql_parser_schema_ast<'src>(
    ast: &Document<'src>,
    source: &'src str,
) -> graphql_parser::schema::Document<'src, str>;

/// Convert our Document to a graphql_parser query AST.
/// Drops: trivia, syntax tokens.
/// GraphQLSourceSpan → Pos conversion uses the span's
/// line/col fields directly.
/// `source` is needed to produce borrowed `&'src str`
/// values for `Cow::Owned` strings in the AST.
pub fn to_graphql_parser_query_ast<'src>(
    ast: &Document<'src>,
    source: &'src str,
) -> graphql_parser::query::Document<'src, str>;

/// Convert a graphql_parser schema AST to our Document.
/// Best-effort: spans are partial (Pos → synthetic
/// GraphQLSourceSpan), trivia and syntax layer unavailable.
pub fn from_graphql_parser_schema_ast<'src>(
    ast: &graphql_parser::schema::Document<'src, str>,
) -> Document<'src>;

/// Convert a graphql_parser query AST to our Document.
/// Best-effort: spans are partial, trivia unavailable.
pub fn from_graphql_parser_query_ast<'src>(
    ast: &graphql_parser::query::Document<'src, str>,
) -> Document<'src>;
```

**Implementation notes:**
- `to_*`: `Cow<'src, str>` passes through directly (no
  `.into_owned()`); `GraphQLSourceSpan` → `Pos { line, column }`
  using the span's line/col fields directly
- `from_*`: `graphql_parser::Pos` provides 1-based line/column;
  `GraphQLSourceSpan` start is constructed from available Pos data;
  end is set to start (zero-width). String values are
  `Cow::Borrowed` from the input AST's `&'src str`
- Information that `graphql_parser` lacks (variable directives,
  schema extensions, trivia) is silently dropped on `to_*` and
  absent on `from_*`

**Optional overloads with source text for better spans:**

```rust
/// When source text is provided, byte offsets are computed
/// accurately from (line, col) pairs. Span end positions
/// are estimated by scanning the source for the extent of
/// each construct.
pub fn from_graphql_parser_schema_ast_with_source<'src>(
    ast: &graphql_parser::schema::Document<'src, str>,
    source: &'src str,
) -> Document<'src>;

pub fn from_graphql_parser_query_ast_with_source<'src>(
    ast: &graphql_parser::query::Document<'src, str>,
    source: &'src str,
) -> Document<'src>;
```

**Parse-and-convert wrappers** (parse with our parser, convert
output to `graphql_parser` types):

```rust
/// Parse source text and return a graphql_parser schema AST.
/// Uses our parser internally; returns ParseResult with
/// errors/warnings.
pub fn parse_schema<S: AsRef<str>>(
    input: S,
) -> ParseResult<
    graphql_parser::schema::Document<'static, String>,
>;

/// Parse source text and return a graphql_parser query AST.
pub fn parse_query<S: AsRef<str>>(
    input: S,
) -> ParseResult<
    graphql_parser::query::Document<'static, String>,
>;
```

### 9.3 `compat_apollo_parser_v0_8`

Feature: `compat-apollo-parser-v0.8`

```rust
// compat_apollo_parser_v0_8.rs

/// Convert our Document to an apollo_parser CST.
///
/// Requires the syntax layer to be populated
/// (retain_syntax = true) for lossless conversion.
/// Without the syntax layer, structural tokens are
/// synthesized with zero-width spans.
pub fn to_apollo_parser_cst<'src>(
    ast: &Document<'src>,
    source: &'src str,
) -> apollo_parser::cst::Document;

/// Convert an apollo_parser CST to our Document.
/// Lossless: apollo_parser's rowan CST preserves all
/// spans, trivia, and syntax tokens.
pub fn from_apollo_parser_cst<'src>(
    doc: &apollo_parser::cst::Document,
    source: &'src str,
) -> Document<'src>;

/// Parse source text and return an apollo_parser CST.
/// Uses our parser internally; returns ParseResult with
/// errors/warnings.
pub fn parse<S: AsRef<str>>(
    input: S,
) -> ParseResult<apollo_parser::cst::Document>;
```

**Implementation approach (to_apollo_parser_cst):**
1. Walk our AST depth-first
2. For each node, call `GreenNodeBuilder::start_node(SyntaxKind)`
3. For each syntax token (from the syntax layer), emit
   `GreenNodeBuilder::token(kind, text)`
4. For trivia, emit whitespace/comment tokens
5. `GreenNodeBuilder::finish_node()`

**Without syntax layer:** We can still produce a structurally valid
CST by synthesizing tokens from semantic values and spans. The CST
will lack trivia but will have correct node structure (lossy but
useful).

**What transfers (from_apollo_parser_cst):**
- All semantic structure
- **Spans (full):** CST nodes have precise byte-offset ranges via
  `text_range()` — map directly to `GraphQLSourceSpan`
- **Trivia (full):** All whitespace, comments, and commas preserved
  as tokens — convert to `GraphQLTriviaToken` values
- **Syntax layer (full):** All punctuation and keyword tokens
  present — syntax layer can be fully populated
- Only limitation: string values need re-extraction from source
  text via spans (CST stores token text, not parsed values)

### 9.4 Conversion Fidelity Summary

| Dimension                    | `compat_graphql_parser_v0_4`                  | `compat_apollo_parser_v0_8`      |
|------------------------------|-----------------------------------------------|----------------------------------|
| `to_*` drops                 | trivia, syntax, var directives, schema ext    | nothing (with syntax layer)      |
| `from_*` spans               | Partial (Pos → zero-width GraphQLSourceSpan)  | Full                             |
| `from_*` trivia              | Unavailable                                   | Full                             |
| `from_*` syntax              | Unavailable                                   | Full                             |
| `to_*` file path             | Dropped (graphql_parser has no file path)      | N/A (CST has text ranges)        |
| `to_*` UTF-16 columns        | Dropped (Pos is line/column only)              | N/A                              |
| `from_*` schema extensions   | Unavailable (graphql_parser lacks them)        | Full                             |
| `from_*` variable directives | Unavailable (graphql_parser lacks them)        | Full                             |

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

1. **Span on every node**: Enables mapping a text edit to the
   affected AST subtree(s) via byte offsets
2. **Immutable nodes**: Nodes are not mutated in place; "editing" means
   producing a new node (enables structural sharing later)
3. **Vec-based children**: Can be replaced wholesale when a subtree is
   re-parsed
4. **Full re-parse is the initial API**: The existing
   `GraphQLParser` constructors (Section 7) already serve this
   role — no separate `reparse()` function is needed

### Future Incremental Strategy (When Needed)

When incremental parsing becomes necessary, the recommended approach is
**subtree re-parse with splice**:

1. Receive a text edit: `(edit_start: usize, edit_end: usize, new_text: &str)`
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
2. **Pass `GraphQLParserConfig` through the parser** to control syntax
   layer population; pass `GraphQLTokenSourceConfig` to the token source
   to control trivia emission
3. **Move `GraphQLSourceSpan`** from consumed tokens directly into AST
   nodes (the span is already computed by the lexer — no extraction or
   conversion needed)
4. **Populate `Name<'src>`** directly from token `Cow<'src, str>`
   (zero-copy path preserved)
5. **Conditionally construct `Syntax` structs** based on config flags
6. **Return `ParseResult<Document<'src>>`**

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

### Error Recovery and Missing Syntax Tokens

When the parser encounters errors and performs error recovery, it may
produce partial ASTs where expected tokens are missing (e.g., an
opening brace without a matching close). The AST types — particularly
`DelimiterPair` — guarantee structural completeness, so the parser
must synthesize tokens for anything that is missing.

**Strategy:** The parser emits a `GraphQLToken` with
`kind = GraphQLTokenKind::Error` for any expected-but-absent token.
The synthesized token carries a zero-width span at the position
where the token was expected (typically the current parser position
or EOF). This keeps the AST structurally valid — every
`DelimiterPair` has both `open` and `close`, every colon field has
a token — while clearly marking synthetic entries through the token
kind. Downstream code that walks the syntax layer can check for
`GraphQLTokenKind::Error` to detect recovered/missing tokens.

**Scenarios requiring synthesized tokens:**

| Category                  | Example source            | Missing token           | Synthesized span                  |
|---------------------------|---------------------------|-------------------------|-----------------------------------|
| Unmatched open delimiter  | `type Foo {` (EOF)        | `}` close brace         | Zero-width at EOF                 |
| Unmatched close delimiter | `}` without open          | `{` open brace          | Zero-width at `}` position        |
| Missing colon             | `field String`            | `:`                     | Zero-width between name and type  |
| Missing `=` for default   | `(x: Int 5)`             | `=`                     | Zero-width before value           |
| Missing keyword           | `extend { }`              | `type`/`schema` keyword | Zero-width at `{` position        |
| Missing name              | `type { }`                | Name token              | Zero-width at `{` position        |
| Missing `@` in directive  | `deprecated` as directive | `@`                     | Zero-width before name            |
| Missing closing `"`       | `"unterminated` (EOF)     | End of string           | Zero-width at EOF                 |

**Design notes:**

- The parser already performs error recovery today (advancing past
  unexpected tokens, inserting expected tokens). This strategy
  formalizes where those synthetic tokens land in the new AST.
- When `retain_syntax = false`, the syntax layer is `None` and
  synthetic tokens are not stored — error recovery still works, it
  just doesn't produce syntax-layer artifacts. The semantic layer
  (names, fields, etc.) uses best-effort values (empty name, etc.)
  and the error is recorded in `ParseResult.errors`.
- The zero-width span convention means diagnostics pointing at a
  synthesized token highlight the correct source location (where
  the token was expected), not some arbitrary position.
- This approach matches what TypeScript's parser and rust-analyzer
  do: the tree is always structurally complete, errors are metadata
  on individual tokens rather than structural holes.

### Parser API

The parser exposes the new `Document<'src>` API directly — there is
no dual-API transition period. Downstream crates (libgraphql-core,
libgraphql-macros) are migrated to the new AST types as part of
Phase 5.

```rust
pub fn parse_schema_document(
    self,
) -> ParseResult<Document<'src>>;
```

---

## 12. Implementation Phases

### Phase 0: ~~Pre-AST Infrastructure Refactoring~~ **DEFERRED**

> **Status: DEFERRED.** The ByteSpan + SourceMap infrastructure
> originally planned here has been deferred. The custom AST uses
> `GraphQLSourceSpan` directly (the existing, working type). See
> **Section 14: Future Optimization Opportunity (ByteSpan +
> SourceMap)** for the preserved design and rationale for deferral.
>
> Steps 0a–0f are retained below (collapsed) for reference only.

<details>
<summary>Original Phase 0 steps (deferred)</summary>

**Step 0a: Introduce `ByteSpan`**
- Define `ByteSpan { start: u32, end: u32 }` with `#[repr(C)]`
- Add conversion: `GraphQLSourceSpan::byte_span() -> ByteSpan`
  (extracts byte offsets from start/end `SourcePosition`s)
- Unit tests for `ByteSpan`

**Step 0b: Introduce `SourceMap<'src>`**
- Define `SourceMap<'src>` with `file_path: Option<&'src Path>`,
  `source: Option<&'src str>`,
  `line_starts: Vec<u32>`, `utf16_offsets: Option<Vec<Utf16LineInfo>>`
- Implement `line_col()`, `line_col_utf16()`,
  `resolve_source_span()`, `source_str()`
- Build `SourceMap<'src>` during lexing
- Add `into_source_map(self) -> SourceMap<'src>` to the
  `GraphQLTokenSource` trait
- Unit tests for `SourceMap`

**Step 0c: Refactor `GraphQLSourceSpan` → `GraphQLSourceSpan<'src>`
(transient only)**
- Change `file_path: Option<PathBuf>` to `file_path: Option<&'src Path>`
- Update all constructors and callers

**Step 0d: Migrate `GraphQLToken.span` to `byte_span: ByteSpan`**
- Rename `GraphQLToken.span` to `GraphQLToken.byte_span` (type:
  `ByteSpan`)
- Update `make_span()` in token sources

**Step 0e: Migrate `GraphQLParseError` to `ByteSpan`**
- Change span fields on errors to `ByteSpan`
- Error formatting gains `source_map: &SourceMap` parameter

**Step 0f: Benchmark**
- Compare before/after Phase 0 changes

</details>

### Phase 1: Core AST Types

- Rename existing `ast` module to `legacy_ast` (old type aliases
  continue to work via the new module name)
- Add `inherent` crate as a dependency of `libgraphql-parser`
- Define all ~48 node types in a new `ast/` module within
  libgraphql-parser
- Implement `Name`, `StringValue`, `IntValue`, `FloatValue` (all
  nodes use `GraphQLSourceSpan` directly — no new span types needed)
- Implement `AstNode` trait with `append_source()` and default
  `to_source()` for all node types via explicit
  `#[inherent] impl AstNode` blocks (enum nodes use
  match-delegation to variants). Each type gets its own explicit
  impl — no `macro_rules!` generation. Span is a `pub` field,
  not a trait accessor
- Implement `append_source` source-slice mode only (slices from
  source text via `Option<&str>`). Synthetic-formatting mode (for
  `source = None`) is deferred to Phase 4e
- Write unit tests for all node types (construction, accessors,
  source-slice round-trip)
- No parser integration yet; just the type definitions

### Phase 2: `compat_graphql_parser_v0_4`

Build the compatibility/conversion layer between the new AST and the
`graphql_parser` crate's types. This is promoted before parser
integration so that Phase 3 can verify existing tests via conversion.

- Add `compat-graphql-parser-v0.4` feature flag gating
  `dep:graphql-parser`
- Implement `to_graphql_parser_schema_ast()` and
  `to_graphql_parser_query_ast()`
- Implement `from_graphql_parser_schema_ast()` and
  `from_graphql_parser_query_ast()` (lossy reverse conversions)
- Implement `from_*_with_source()` overloads for better spans
- Implement drop-in `parse_schema()` / `parse_query()` wrappers

### Phase 3: Parser Integration

- Add `GraphQLParserConfig` to `GraphQLParser`
- Add `new_with_configs()` constructor (note:
  `from_token_source()` already exists)
- Modify all `parse_*` methods to produce new AST types
- Implement the semantic layer (syntax fields all `None`
  initially regardless of config — syntax struct population
  comes in Phase 4c)
- Ensure all 443+ existing tests pass via Phase 2's compat
  conversion layer (parse with new parser → convert to old AST
  → run existing assertions)

### Phase 4: Syntax Layer

**CRITICAL — Test convention:** All tests (except tests that
specifically verify config-flag behavior) must be updated to run
with all trivia flags enabled AND `retain_syntax = true`. The
majority of our test surface should exercise the full-fidelity
parser path. Config-flag tests are the exception: they verify that
turning individual flags on/off produces the expected behavior
(e.g., trivia absent when flag is off, present when on).

#### Phase 4a: Lexer Trivia Configuration
- Define `GraphQLTokenSourceConfig` struct with three per-type
  trivia flags (all default `true`)
- Add `Whitespace` variant to `GraphQLTriviaToken`
- Update `StrGraphQLTokenSource` to accept
  `GraphQLTokenSourceConfig` and only record each trivia type
  when its flag is on (all flags default to `true`, consistent
  with current always-on Comment/Comma behavior)
- `RustMacroGraphQLTokenSource` continues to emit comma trivia
  unconditionally (no config param; Rust's tokenizer strips
  comments and whitespace). **TODO (project-tracker.md):** Add a
  task to make `RustMacroGraphQLTokenSource` synthesize
  whitespace (with spaces) and accept an optional config
- Unit tests for each trivia flag independently (whitespace on/off,
  comment on/off, comma on/off)

#### Phase 4b: Parser Syntax Configuration
- Define `GraphQLParserConfig` struct with `retain_syntax: bool`
- Wire `retain_syntax` through the parser (all syntax structs
  remain `None` at this point regardless of config — this is
  a mid-way step; population comes in Phase 4c)
- Update parity utils for new `Whitespace` variant
- Unit tests for `retain_syntax` flag plumbing

#### Phase 4c: Syntax Struct Population
- Define all `*Syntax` struct types from the catalog (Section 6)
- Populate `*Syntax` structs when `retain_syntax = true`
- Move `GraphQLToken`s from the token stream directly into
  syntax structs (zero-copy)
- Unit tests for individual syntax structs (verify correct tokens
  land in correct fields)

#### Phase 4d: Test Migration
- **Agent-assisted:** This phase is mechanical (update test
  assertions to match the new AST format) and well-suited for
  AI agent assistance
- Update all existing parsing tests to validate against the new
  AST format directly (rather than going through the compat
  layer), ensuring each updated test still passes as we progress
- Update parsing tests (excluding those that specifically test
  trivia config flags or `retain_syntax` behavior) to use
  full-fidelity mode: all trivia on + `retain_syntax = true`
- Add config-flag-specific tests that verify each flag toggles
  its trivia type independently
- Add whitespace trivia tests

#### Phase 4e: Source Reconstruction & Benchmarking
- Implement synthetic-formatting mode for `append_source` (the
  `source = None` fallback that reconstructs from semantic data)
- Write round-trip test (parse → `to_source(Some(source))` →
  compare original)
- Write semantic-equivalence test for synthetic-formatting mode
- Update benchmarking script to run two flavors of all existing
  benchmarks (including parser-comparison benchmarks):
  (a) all trivia off / `retain_syntax = false` (lean mode)
  (b) all trivia on / `retain_syntax = true` (full-fidelity mode)

### Phase 5: Downstream Migration

All downstream consumers (`libgraphql-macros`, `libgraphql-core`) will
initially migrate by adopting the `compat_*` conversion utilities from
Phase 2. This keeps the migration mechanical and low-risk: each
consumer parses with the new parser, converts to the legacy AST types
via `compat_graphql_parser_v0_4`, and the rest of its code is
unchanged.

Porting downstream consumers to use the new AST directly (eliminating
the compat layer) is a separate, follow-on effort that will require
its own design plan — it touches type signatures, validation logic,
and error reporting throughout the codebase.

- Update `libgraphql-macros` to parse via new parser + compat layer
- Update `libgraphql-core` to parse via new parser + compat layer
  (behind feature flag)
- Wire `use-libgraphql-parser` feature flag to use new parser + compat
  layer

### Phase 6: `compat_apollo_parser_v0_8`

- Add `compat-apollo-parser-v0.8` feature flag gating
  `dep:apollo-parser`
- Implement `to_apollo_parser_cst()`
- Implement `from_apollo_parser_cst()` (lossless reverse)
- Test against apollo-parser's own test fixtures

### Phase 7: FFI Layer

- Define C API surface (accessor functions)
- Implement `OwnedDocument` or two-handle pattern
- Auto-generate C headers
- Write C integration tests

### Phase 8: Cleanup

- Remove old `legacy_ast.rs` type aliases
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

2. ~~**FFI ownership model:**~~ **RESOLVED.** Two-handle
   `Source`+`Document` for Phase 7. `OwnedDocument` is a possible
   follow-on.

3. ~~**SourceMap location:**~~ **DEFERRED.** No `SourceMap` is needed
   for the initial custom AST — nodes use `GraphQLSourceSpan`
   directly, which already carries line/col. If the ByteSpan +
   SourceMap optimization (Section 14) is pursued in the future,
   the SourceMap would be stored in `ParseResult` alongside the AST
   and errors.

4. ~~**Module naming:**~~ **RESOLVED.** Rename existing `ast` module
   to `legacy_ast` in Phase 1. New AST types live in a new `ast/`
   module that replaces the old module name. Old type aliases continue
   to work via the `legacy_ast` module name during migration.

5. ~~**Trivia: leading vs leading+trailing:**~~ **RESOLVED.**
   Leading-only. Trivia is attached as leading trivia on the following
   token (consistent with the current `GraphQLToken::preceding_trivia`
   design). Tools that need trailing-trivia association can compute
   it from positions.

6. ~~**`PhantomData` on lifetime-less nodes:**~~ **RESOLVED.** Every
   node has a `syntax: Option<...Syntax<'src>>` field, so all nodes
   naturally use `'src` and no `PhantomData` is needed anywhere.

7. ~~**`GraphQLParseError` span type:**~~ **RESOLVED — no change
   from status quo.** `GraphQLParseError` continues to store
   `GraphQLSourceSpan`. If the ByteSpan + SourceMap optimization
   (Section 14) is pursued, errors would migrate to `ByteSpan` at
   that time.

8. ~~**`SourceMap` and `GraphQLSourceSpan` lifetime:**~~ **N/A
   (deferred).** No `SourceMap` is introduced for the initial custom
   AST. `GraphQLSourceSpan` is used unchanged (no new `'src`
   lifetime parameter on it). If the ByteSpan + SourceMap
   optimization (Section 14) is pursued, the lifetime design
   described there would apply.

9. ~~**`GraphQLToken.span` type:**~~ **RESOLVED — no change from
   status quo.** `GraphQLToken.span` continues to store
   `GraphQLSourceSpan`. If the ByteSpan + SourceMap optimization
   (Section 14) is pursued, tokens would migrate to `ByteSpan` at
   that time.

---

## 14. Future Optimization Opportunity: ByteSpan + SourceMap

> **Status:** Deferred. This section preserves the complete design
> for a compact span representation that was originally planned as
> Phase 0 pre-work. The custom AST uses `GraphQLSourceSpan`
> directly (Section 3). This optimization can be explored after the
> AST is working and profilable.

### 14.1 Preserved Design

#### `ByteSpan`

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

#### `SourceMap`

```rust
/// Maps byte offsets to line/column positions. Built once during
/// parsing, shared across all lookups.
///
/// The `'src` lifetime matches the source text lifetime. The file
/// path borrows at `'src` — the same lifetime as the source text
/// — because both are provided as input to the parser and are
/// expected to be kept alive for the lifetime of the AST.
/// This unifies the SourceMap's lifetime with the single `'src`
/// that already permeates the token/parser pipeline, avoiding a
/// second lifetime parameter.
pub struct SourceMap<'src> {
    /// Optional file path for the source text. Borrowed from
    /// the caller at the same `'src` lifetime as the source
    /// text. Included in `GraphQLSourceSpan` values returned
    /// by resolve methods.
    file_path: Option<&'src Path>,
    /// Optional reference to the original source text.
    /// `Some` for `StrGraphQLTokenSource` (always has source);
    /// `None` for `RustMacroGraphQLTokenSource` (proc-macro
    /// tokens don't have a meaningful source string).
    /// Used by `AstNode::append_source()` for zero-copy
    /// source reconstruction.
    source: Option<&'src str>,
    /// Sorted byte offsets of each line start (index 0 =
    /// line 0).
    line_starts: Vec<u32>,
    /// Optional: UTF-16 column offset table for LSP
    /// compatibility. Only populated when the token source
    /// provides col_utf16.
    utf16_offsets: Option<Vec<Utf16LineInfo>>,
}

/// UTF-16 column mapping for a single source line.
pub struct Utf16LineInfo {
    /// Sorted (byte_offset, utf16_column) pairs marking where
    /// UTF-8 and UTF-16 column indices diverge within this
    /// line. Binary search on byte_offset to find the nearest
    /// entry, then compute: utf16_col = entry.1 +
    /// (byte_offset - entry.0).
    pub utf16_columns: Vec<(u32, u32)>,
}

impl<'src> SourceMap<'src> {
    /// O(log n) lookup: byte offset → (line, col_utf8).
    pub fn line_col(
        &self,
        byte_offset: u32,
    ) -> (u32, u32);

    /// O(log n) lookup: byte offset → (line, col_utf16).
    /// Returns None if UTF-16 info was not collected.
    pub fn line_col_utf16(
        &self,
        byte_offset: u32,
    ) -> Option<(u32, u32)>;

    /// Convert a ByteSpan to a full GraphQLSourceSpan (with
    /// file path from this SourceMap, if set).
    pub fn resolve_source_span(
        &self,
        span: ByteSpan,
    ) -> GraphQLSourceSpan<'src>;

    /// Returns the original source text, if available.
    pub fn source_str(&self) -> Option<&'src str>;
}
```

#### `into_source_map()` Trait Extension

```rust
pub trait GraphQLTokenSource<'src>:
    Iterator<Item = GraphQLToken<'src>>
{
    // ... existing methods ...

    /// Consume this token source and return the SourceMap
    /// built during lexing. Must only be called after all
    /// tokens have been consumed (i.e. after EOF).
    fn into_source_map(self) -> SourceMap<'src>;
}
```

The parser would call `self.token_source.into_source_map()` after
consuming the EOF token and bundle the result into `ParseResult`.

#### `ParseResult` Lifetime Parameterization

`ParseResult<TAst>` would gain a lifetime parameter →
`ParseResult<'src, TAst>` and a new
`source_map: SourceMap<'src>` field so that all consumers can
resolve `ByteSpan` → line/col via the bundled source map.

#### `ByteSpan::to_source_span()` Convenience

```rust
impl ByteSpan {
    /// Resolve to a full GraphQLSourceSpan using a SourceMap.
    pub fn to_source_span<'src>(
        &self,
        source_map: &SourceMap<'src>,
    ) -> GraphQLSourceSpan<'src>;
}
```

#### Transient `GraphQLSourceSpan<'src>`

`GraphQLSourceSpan` would gain a `'src` lifetime parameter but
would never be stored in the AST, tokens, or errors — only
produced on demand via `SourceMap::resolve_source_span()` or
`ByteSpan::to_source_span()`:

```rust
/// Rich span with resolved line/column positions and optional
/// file path. Produced on demand from ByteSpan + SourceMap.
/// Not stored in the AST — use ByteSpan for storage.
pub struct GraphQLSourceSpan<'src> {
    pub start_inclusive: SourcePosition,
    pub end_exclusive: SourcePosition,
    pub file_path: Option<&'src Path>,
}
```

### 14.2 Rationale / Opportunity

- **8 bytes vs ~104 bytes** per span (~13x reduction)
- **Eliminates per-token PathBuf clone** in `make_span()`
  (currently ~250K clones for the GitHub schema)
- **Better L1 cache density:** ~8000 ByteSpans vs ~615
  GraphQLSourceSpans fit in 64KB L1
- **`#[repr(C)]`** for direct FFI access without conversion
- **Line/col only needed on error-formatting cold path,** not
  during parsing
- **Matches standard compiler architecture** (rustc, clang, swc,
  oxc all use byte-offset spans with separate position tables)

### 14.3 Known Implementation Complexity

These difficulties were discovered during the initial attempt to
implement ByteSpan + SourceMap as Phase 0 pre-work:

1. **`line_starts` collection in lexer hot paths.** The SourceMap
   requires a `line_starts: Vec<u32>` table built during lexing.
   The current lexer has optimized hot paths —
   `skip_whitespace()` and `lex_block_string()` — that advance
   through source bytes efficiently without tracking line
   positions. Adding line_starts recording to these paths either:
   - Adds branches to the tight inner loops (perf regression
     risk), or
   - Requires a separate post-lexing pass to scan for newlines
     (extra work)

   The lexer currently computes line/col incrementally as part of
   SourcePosition tracking — the work to *stop* doing that
   per-token and instead collect only line_starts proved to be a
   non-trivial refactor of the lexer's inner loop.

2. **UTF-8 column offset edge cases in SourceMap.** Converting a
   byte offset to a UTF-8 column requires knowing where the line
   starts (via line_starts) and then counting characters from line
   start to the target byte offset. But byte offsets can land
   mid-codepoint in UTF-8 sequences (e.g., a byte offset pointing
   to the 2nd byte of a 3-byte UTF-8 character). The SourceMap
   must handle this gracefully — either by snapping to the nearest
   codepoint boundary or by explicitly defining behavior for
   mid-codepoint offsets. This is subtle and needs thorough test
   coverage.

3. **UTF-16 column computation.** LSP clients need UTF-16 column
   offsets. Computing these from byte offsets requires knowing the
   UTF-8 → UTF-16 mapping for each line, stored in the
   `Utf16LineInfo` structure. Building this table during lexing
   adds further complexity to the hot path, or requires a separate
   pass over source text.

4. **Two-phase migration risk.** ByteSpan + SourceMap touches
   every layer of the stack simultaneously (lexer, tokens, trivia,
   errors, parser, ParseResult, all consumers). The old Phase 0
   had 6 sub-steps that all needed to land together, making
   incremental validation difficult.

### 14.4 Recommendation

This work should be treated as an **opportunistic performance
optimization experiment** rather than a planned improvement we will
definitely pursue. The questions to answer:

- Can we add line_starts tracking to
  `skip_whitespace()`/`lex_block_string()` without measurable
  hot-path regression?
- Can we handle UTF-8/UTF-16 column edge cases correctly and
  comprehensively?
- Does the memory savings actually matter for real-world GraphQL
  document sizes? (Most documents are <100KB where ~104 bytes/span
  is not a practical concern)
- Is the API complexity (SourceMap threading) worth the memory
  savings?

Profiling the working custom AST (once built) will provide concrete
data to answer these questions. Until then, this is speculative
optimization.
