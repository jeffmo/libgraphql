# Syntax Tree Evaluation Plan

**Purpose:** Evaluate whether `libgraphql-parser` should (A) adopt `apollo-parser`'s CST, (B) adopt it with modifications, or (C) design a wholly new structure. This document lays out the evaluation criteria, preliminary analysis, and a concrete plan for reaching a final recommendation.

**Referencing:** Section 4.2 of `crates/libgraphql-parser/project-tracker.md`

---

## 1. Goals and Constraints (from Project Tracker §4.2)

### 1.1 Functional Goals

| ID | Goal | Notes |
|----|------|-------|
| G1 | Spans on all nodes | Every AST/CST node must carry source-location info |
| G2 | Schema extension support | `extend schema @directive { ... }` must be representable |
| G3 | Trivia attachment | Whitespace/comments preserved for formatters |
| G4 | Serde serialization | Complete tree must be serializable/deserializable |

### 1.2 Design Constraints

| ID | Constraint | Notes |
|----|-----------|-------|
| C1 | **Superset requirement** | Must represent at least a superset of info in *both* `graphql-parser` and `apollo-parser` |
| C2 | **C/C++ FFI suitability** | Must map naturally to C structs/tagged unions; avoid deeply generic types, complex ownership |
| C3 | **Translation utilities** | Forward translation to `graphql-parser` AST required; `apollo-parser` translation required if we don't adopt their structure directly |

### 1.3 Evaluation Factors (from §4.2 Task 2)

| ID | Factor |
|----|--------|
| F1 | API ergonomics |
| F2 | Compatibility burden |
| F3 | Maintenance cost |
| F4 | Information preservation |
| F5 | Downstream consumer needs |
| F6 | Parser performance implications (allocations, node granularity) |
| F7 | Configurability (parser options, spec-version variations) |
| F8 | FFI suitability |

---

## 2. Current State

`libgraphql-parser` currently produces `graphql_parser` crate AST types (`graphql_parser::query::*`, `graphql_parser::schema::*`) re-exported through `ast.rs` as type aliases pinned to `<'static, String>`. The parser (`GraphQLParser<S>`) is a hand-written recursive descent parser generic over token source. It produces `ParseResult<TAst>` which carries `Option<TAst>` + `Vec<GraphQLParseError>` for IDE-friendly partial results.

**Key limitation:** The `graphql-parser` AST has no spans on `Value`, `Type`, `TypeCondition`, `DirectiveLocation`, or `Number` nodes; no trivia; no schema extension support; and cannot be serialized.

---

## 3. Three Candidates

### 3.1 Option A — Adopt `apollo-parser`'s CST Directly

**What it is:** The `apollo-parser` crate builds a lossless Concrete Syntax Tree using the `rowan` crate's red-green tree model. Every token (including whitespace and comments) is preserved as a leaf node. Typed accessor structs are code-generated from an `ungrammar` definition and provide zero-cost casts over the untyped `SyntaxNode` tree.

**Architecture summary:**
- **Green tree** (immutable, compact): stores `SyntaxKind` (u16), text width, children. Uses `ThinArc` for pointer management. Supports structural sharing.
- **Red tree** (lazy, on-demand): adds parent pointers and absolute offsets. Not persisted — reconstructed during traversal.
- **Typed wrappers** (`CstNode` / `CstToken` traits): zero-cost downcasts from `SyntaxNode` to typed structs like `ObjectTypeDefinition`, `Field`, etc. Accessor methods navigate children.
- **Trivia handling:** whitespace/comments are interleaved into the tree as tokens, making it fully lossless.
- **Error recovery:** parser never panics; accumulates errors alongside partial tree.

### 3.2 Option B — Adapt `apollo-parser`'s CST (Hybrid)

Keep the rowan-based CST as an internal representation but overlay it with a more ergonomic, FFI-friendly API layer. Potentially fork/vendor the rowan dependency to customize.

### 3.3 Option C — Design a New Structure

A traditional AST with struct-based nodes, direct field access, spans on every node, and optional trivia attachment. Designed from scratch to satisfy all constraints.

---

## 4. Preliminary Evaluation

### 4.1 Evaluation Matrix

Each cell rates the option against the criterion: ✅ strong fit, ⚠️ partial fit / trade-offs, ❌ poor fit.

| Criterion | Option A (adopt apollo CST) | Option B (adapt apollo CST) | Option C (new design) |
|-----------|----------------------------|-----------------------------|-----------------------|
| **G1: Spans on all nodes** | ✅ Every node has `text_range()` | ✅ Inherits from rowan | ✅ By design |
| **G2: Schema extensions** | ✅ apollo-parser supports them | ✅ Same | ✅ By design |
| **G3: Trivia attachment** | ✅ Fully lossless, all trivia preserved | ✅ Same | ⚠️ Must be explicitly designed; likely "opt-in trivia" rather than fully lossless |
| **G4: Serde serialization** | ❌ Rowan trees are graph-like (parent pointers, Arc sharing); no natural serde mapping | ⚠️ Would need a separate serializable "snapshot" format | ✅ Plain structs serialize trivially |
| **C1: Superset of both** | ⚠️ Superset of `graphql-parser` info, but `apollo-parser` CST itself doesn't preserve *semantic* info that `graphql-parser` computes (e.g., parsed int values) — those come from the typed layer | ⚠️ Same base issue | ✅ Can be designed to include all fields from both |
| **C2: C/C++ FFI** | ⚠️ Feasible via opaque-pointer API, but unnatural — see §4.3 for detailed analysis | ⚠️ Same core trade-offs as A; wrapping layer doesn't change the underlying model | ✅ Struct-based layout maps naturally to C structs/tagged unions |
| **C3: Translation utils** | ⚠️ No `apollo-parser` translation needed, but `graphql-parser` forward translation requires tree traversal + allocation | ⚠️ Same | ⚠️ Need translation for both, but straightforward struct-to-struct conversion |
| **F1: API ergonomics** | ⚠️ Accessor methods traverse children each call; no direct field access; unfamiliar model for most Rust users | ⚠️ Could add ergonomic layer but doubled complexity | ✅ Direct `node.field` access; familiar Rust struct pattern |
| **F2: Compatibility burden** | ⚠️ Locked to rowan's API surface and evolution; apollo-parser version coupling | ⚠️ Fork/vendor adds maintenance | ✅ No external API dependency |
| **F3: Maintenance cost** | ⚠️ Ungrammar codegen pipeline adds build complexity; must track apollo-parser upstream changes | ❌ Fork divergence is highest-maintenance option | ⚠️ More upfront work, but simpler ongoing maintenance |
| **F4: Info preservation** | ✅ Fully lossless | ✅ Same | ⚠️ Lossless if trivia attached; "almost lossless" if trivia optional |
| **F5: Downstream consumers** | ⚠️ Consumers must learn rowan tree navigation; niche API | ⚠️ Two layers to learn | ✅ Standard Rust structs; lowest learning curve |
| **F6: Performance** | ⚠️ Arc allocations per node; cache management overhead; lazy red tree construction on traversal. Good for IDE use (incremental), potentially worse for batch parsing. | ⚠️ Same base cost | ✅ Arena or Vec-based allocation; minimal overhead for batch use cases |
| **F7: Configurability** | ⚠️ SyntaxKind enum is fixed at compile time; adding parser options or spec-version flags means forking codegen | ⚠️ Same | ✅ Full control over node variants, can use cfg flags or generics |
| **F8: FFI suitability** | ⚠️ See C2 | ⚠️ See C2 | ✅ See C2 |

### 4.2 Summary of Key Trade-offs

**Option A (adopt apollo CST) — strongest at:** lossless trivia preservation, proven error recovery model, zero implementation cost for the tree structure itself.

**Option A — weakest at:** C/C++ FFI ergonomics (feasible but unnatural — see §4.3), serde serialization (G4), API ergonomics for non-IDE consumers, build pipeline complexity.

**Option B (adapt) — attempts to get the best of both worlds but in practice:** doubles the API surface, creates a fork-maintenance burden, and doesn't improve FFI ergonomics since the underlying data structure is still rowan.

**Option C (new design) — strongest at:** FFI suitability, serde, API ergonomics, configurability, performance for batch use cases, full control.

**Option C — weakest at:** upfront implementation effort, trivia preservation (requires explicit design rather than getting it "for free"), and the need for translation utilities to both external formats.

### 4.3 FFI Analysis: Rowan Is Feasible But Unnatural

Rowan-based trees *can* be exposed to C/C++ via the standard opaque-pointer pattern — this is not infeasible. Many successful C libraries (libxml2, CoreFoundation, GLib) use exactly this model. The question is whether it's the right trade-off for `libgraphql`.

#### How a rowan C FFI would work

```c
typedef struct LGSyntaxNode LGSyntaxNode;

uint16_t       lg_node_kind(const LGSyntaxNode* node);
LGTextRange    lg_node_text_range(const LGSyntaxNode* node);
LGSyntaxNode*  lg_node_first_child(const LGSyntaxNode* node);
LGSyntaxNode*  lg_node_next_sibling(const LGSyntaxNode* node);
LGSyntaxNode*  lg_node_parent(const LGSyntaxNode* node);
void           lg_node_free(LGSyntaxNode* node);
```

Each Rust function boxes a `SyntaxNode`, hands it across as an opaque pointer, and C calls `lg_node_free` when done. The `Arc`-based root reference inside each `SyntaxNode` keeps the green tree alive, so there's no use-after-free as long as C frees nodes properly. This works.

#### Cost 1: Per-navigation allocation overhead

Every call to `lg_node_first_child()` or `lg_node_next_sibling()` creates a new `SyntaxNode` on the Rust side, boxes it, and hands it to C. C must free each one. A traversal visiting N nodes does N heap allocations and N frees as intermediate "trampoline" objects that exist only to be read and discarded. Contrast with a struct-based AST where the entire tree is allocated once and C follows pointers — zero per-access overhead.

**Important nuance:** If a C/C++ tool expects a struct-based layout (as many do), it will traverse the rowan tree through opaque calls and build its own struct representation. The total work of "build a struct tree" is roughly equivalent either way — either we do it or the consumer does. But the rowan path adds concrete overhead from the intermediate `SyntaxNode` box/unbox cycle during that traversal, overhead that doesn't exist if we ship structs directly.

That said, this could be mitigated with a bulk `lg_materialize_tree()` function on the Rust side that builds a C-friendly struct tree in one pass without per-node FFI round-trips. So the performance cost is real but not fundamental.

#### Cost 2: No direct field access

With a struct-based AST:
```c
typedef struct {
    LGSpan       span;
    LGName       name;
    LGFieldDef*  fields;
    uint32_t     field_count;
    LGDirective* directives;
    uint32_t     directive_count;
} LGObjectTypeDef;

// Direct field reads — zero overhead, transparent layout
const char* name = def->name.text;
```

With rowan, every "field" is a function call that traverses children:
```c
LGSyntaxNode* name = lg_object_type_def_name(node);
const char* text = lg_node_text(name);
lg_node_free(name);
```

Not broken, but noisier and less natural for C/C++ consumers.

#### Cost 3: Large generated C API surface

Apollo-parser's typed Rust wrappers are code-generated from ungrammar. For a C API, you'd need a *second* codegen pass producing `lg_object_type_def_name()`, `lg_object_type_def_fields()`, etc. — one function per "field" per node type. This is substantial API surface to generate and maintain.

#### Cost 4: The "two representations" risk

If we adopt rowan internally but many C/C++ consumers end up materializing struct-based trees on their side anyway, we've effectively shipped the complexity of rowan without its benefits reaching the consumers who need them most. Building the struct-based tree once on our side and shipping it directly is simpler for everyone.

#### Summary

Rowan + C FFI is **feasible** (opaque-pointer APIs are standard and battle-tested) but **unnatural** (per-access allocation overhead, no direct field reads, large generated API surface). The performance difference from intermediate allocations is real but mitigable. The primary argument against it is API quality and the risk of pushing tree-materialization work onto every C/C++ consumer individually. Combined with the serde, ergonomics, and maintenance trade-offs described elsewhere, FFI considerations add meaningful weight toward Option C but are not independently decisive.

### 4.4 Trivia Preservation in Option C

The main area where Option C must be designed carefully is trivia preservation (G3). Two sub-approaches:

**C-i: Always-attached trivia (lossless by default)**
- Every node carries `leading_trivia: Vec<Trivia>` and `trailing_trivia: Vec<Trivia>` where `Trivia` is `{ kind: TriviaKind, span: Span, text: String }`.
- Pro: Truly lossless; formatters and printers always have full info.
- Con: Memory overhead for consumers who don't need trivia; every node is larger.

**C-ii: Optional trivia (lossless when configured)**
- Parser accepts a configuration flag (`preserve_trivia: bool`). When enabled, trivia is attached to nodes. When disabled, trivia is discarded and nodes are smaller.
- Pro: Pay-for-what-you-use; batch consumers get lean nodes.
- Con: Two "shapes" of the same tree type (with/without trivia); slightly more complex API.

**Recommendation for evaluation:** Prototype **C-i** first (always-attached trivia) and measure memory overhead on a large schema (e.g., GitHub's ~30K-line schema). If overhead is unacceptable, fall back to C-ii.

---

## 5. Preliminary Recommendation

**Option C (design a new structure)** is the strongest candidate given the stated constraints. No single factor is a knockout blow against Options A/B, but the cumulative weight of FFI unnaturalness, serde difficulty, API ergonomics, codegen pipeline maintenance, and less familiar Rust API tilts the balance decisively toward Option C.

However, this recommendation needs validation through concrete prototyping before it becomes final. The evaluation plan below is designed to either confirm or revise this recommendation.

---

## 6. Evaluation Plan

### Phase 1: Catalogue and Compare (Research)

**Deliverable:** A comparison document mapping every field in both `graphql-parser` and `apollo-parser` to the proposed new structure, identifying shared fields, unique fields, and new fields `libgraphql` should add.

#### 1.1 Catalogue `graphql-parser` types

- [ ] List every type in `graphql_parser::query` and `graphql_parser::schema`
- [ ] For each type, record: name, fields, which fields have position info, which don't
- [ ] Note fields that are "lossy" (e.g., `Number` wraps `i64` — raw text is lost)

**Status: DONE** (captured in research for this document; formalize into comparison table)

#### 1.2 Catalogue `apollo-parser` types

- [ ] List every `SyntaxKind` variant
- [ ] List every typed CST node and its accessor methods
- [ ] For each node, record: what info it preserves, how trivia is attached, what the span model is
- [ ] Note how `apollo-parser` handles things `graphql-parser` doesn't (schema extensions, trivia, error nodes)

**Approach:** Fetch `apollo-parser`'s `graphql.ungram` file and `syntax_kind.rs` from the [apollo-rs repo](https://github.com/apollographql/apollo-rs). These two files define the complete CST shape.

#### 1.3 Produce comparison table

- [ ] Side-by-side table: `graphql-parser` field ↔ `apollo-parser` equivalent ↔ proposed `libgraphql` field
- [ ] Identify gaps: info in one but not the other
- [ ] Identify new info `libgraphql` should add (full spans, trivia, parsed+raw values, etc.)

### Phase 2: Prototype the New Structure (Design)

**Deliverable:** A Rust module (`crates/libgraphql-parser/src/syntax_tree/`) with type definitions for all GraphQL constructs, implementing the superset requirement.

#### 2.1 Design span model

- [ ] Define `Span { start: Position, end: Position }` where `Position { line: u32, column_utf8: u32, column_utf16: Option<u32>, byte_offset: u32 }`
- [ ] Ensure every node type carries a `span: Span` field
- [ ] Ensure this is compatible with `GraphQLSourceSpan` already in the crate

#### 2.2 Design trivia model

- [ ] Define `Trivia { kind: TriviaKind, text: String, span: Span }` where `TriviaKind = Whitespace | LineComment | BlockComment | Comma`
- [ ] Decide on attachment strategy: leading trivia on each node, or leading+trailing
- [ ] Prototype on 2-3 node types to validate ergonomics

#### 2.3 Design node types

For each GraphQL construct, define a Rust struct with:
- All fields from `graphql-parser`'s equivalent type
- All additional info from `apollo-parser`'s equivalent CST node
- `span: Span` on every node
- Optional trivia fields
- Raw text preservation where `graphql-parser` discards it (e.g., raw number text alongside parsed `i64`)

Node categories to define:
- [ ] Document types: `SchemaDocument`, `ExecutableDocument`, `MixedDocument`
- [ ] Schema definitions: `SchemaDefinition`, `ObjectTypeDefinition`, `InterfaceTypeDefinition`, `UnionTypeDefinition`, `EnumTypeDefinition`, `ScalarTypeDefinition`, `InputObjectTypeDefinition`, `DirectiveDefinition`
- [ ] Schema extensions: `SchemaExtension`, `ObjectTypeExtension`, `InterfaceTypeExtension`, `UnionTypeExtension`, `EnumTypeExtension`, `ScalarTypeExtension`, `InputObjectTypeExtension`
- [ ] Operations: `OperationDefinition`, `FragmentDefinition`
- [ ] Selections: `SelectionSet`, `Field`, `FragmentSpread`, `InlineFragment`
- [ ] Variables: `VariableDefinition`
- [ ] Values: `Value` enum with spans on each variant
- [ ] Types: `TypeReference` enum with spans on each variant
- [ ] Directives: `Directive`, `DirectiveLocation`
- [ ] Arguments: `Argument` (with span), `InputValueDefinition`
- [ ] Descriptions: `Description` (with span, preserving raw text)
- [ ] Names: `Name` (with span)

#### 2.4 Validate FFI suitability

- [ ] For each node type, sketch the corresponding C struct / tagged union
- [ ] Verify no `Arc`, `Box<dyn Trait>`, closures, or generics that can't cross FFI
- [ ] Verify ownership model: can the tree be passed as an opaque pointer with accessor functions?

#### 2.5 Validate serde round-trip

- [ ] Derive `Serialize`/`Deserialize` on all node types
- [ ] Write a test: parse → serialize → deserialize → compare
- [ ] Verify trivia survives round-trip

### Phase 3: Translation Utilities (Compatibility)

**Deliverable:** Implemented and tested forward translations; feasibility assessment for reverse translations.

#### 3.1 `libgraphql` → `graphql-parser` forward translation

- [ ] Implement `From<LibgraphqlSchemaDoc> for graphql_parser::schema::Document<'static, String>`
- [ ] Implement `From<LibgraphqlExecDoc> for graphql_parser::query::Document<'static, String>`
- [ ] Test: parse with `libgraphql-parser` → translate → compare with `graphql_parser::parse_*` output
- [ ] Document info loss (trivia, extra spans discarded)

#### 3.2 `libgraphql` → `apollo-parser` forward translation

- [ ] Assess feasibility: can we construct a rowan `SyntaxNode` tree from our AST?
- [ ] If feasible, implement; if not, document why and what the closest approximation is
- [ ] Note: this may require the `rowan` crate as an optional dependency

#### 3.3 Reverse translations (external → `libgraphql`)

- [ ] `graphql-parser` → `libgraphql`: implement (lossy: no trivia, limited spans — `graphql-parser` only has start position, not end)
- [ ] `apollo-parser` → `libgraphql`: assess feasibility; implement if tenable (should be mostly lossless since `apollo-parser` preserves more info)
- [ ] Document all information loss for each reverse translation

### Phase 4: Parser Integration (Implementation)

**Deliverable:** `GraphQLParser` updated to produce the new syntax tree types.

#### 4.1 Update parser to produce new types

- [ ] Modify `parse_schema_document()`, `parse_executable_document()`, `parse_mixed_document()` to return new types
- [ ] Thread span tracking through all parse methods (the parser already tracks positions via `GraphQLSourceSpan`)
- [ ] Add trivia collection to lexer/parser pipeline
- [ ] Ensure `ParseResult<T>` works with new types

#### 4.2 Update `libgraphql-macros`

- [ ] Update `RustMacroGraphQLTokenSource` integration
- [ ] Update `graphql_schema!` macro codegen for new types
- [ ] Verify all macro tests pass

#### 4.3 Update `libgraphql-core`

- [ ] Update the `use-libgraphql-parser` feature-gated code paths
- [ ] Update builders (`SchemaBuilder`, `QueryBuilder`, etc.)
- [ ] Verify all `libgraphql-core` tests pass

### Phase 5: Validation (Confidence)

#### 5.1 Superset verification

- [ ] For every field in `graphql-parser` AST, confirm the new structure has an equivalent
- [ ] For every accessor in `apollo-parser` CST, confirm the new structure has an equivalent
- [ ] Document any intentional omissions with rationale

#### 5.2 Performance measurement

- [ ] Benchmark parse time: new structure vs. current `graphql-parser` AST production
- [ ] Measure memory usage on large schema (GitHub schema)
- [ ] Measure trivia overhead specifically

#### 5.3 FFI proof-of-concept

- [ ] Write a minimal C header file for 3-5 representative node types
- [ ] Implement the C accessor functions in Rust (via `#[no_mangle] extern "C"`)
- [ ] Write a small C program that parses a schema and traverses the tree
- [ ] Confirm the FFI model is viable before committing to the full binding surface

---

## 7. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Trivia model adds unacceptable memory overhead | Performance regression | Phase 2.2 prototyping + Phase 5.2 measurement; fall back to optional trivia |
| Translation to `apollo-parser` CST is infeasible | Can't interop with `apollo-rs` ecosystem | Document limitation; provide "best-effort" translation or export to GraphQL text and re-parse |
| New structure is too large / too many types | Maintenance burden | Keep 1:1 with GraphQL spec constructs; avoid over-engineering; use codegen if >40 types |
| Parser changes break `libgraphql-macros` | Regression | Phase 4.2 runs macro test suite; keep `graphql-parser` AST production as a fallback behind feature flag during transition |

---

## 8. Decision Gate

After completing **Phase 2** (prototype), we should have enough concrete information to make the final adopt/adapt/new decision. The gate criteria:

1. **Prototype compiles** and covers all GraphQL constructs
2. **FFI sketch** (Phase 2.4) confirms C-friendliness
3. **Serde round-trip** (Phase 2.5) works
4. **Trivia overhead** measured and acceptable (or optional-trivia fallback validated)
5. **No showstoppers** found that would make Option A or B preferable after all

If any gate criterion fails, re-evaluate and document why the recommendation should change.

---

## Appendix A: `graphql-parser` Types Missing Position Info

These types currently lack any source position — the new structure must add spans:

| Type | What it represents |
|------|--------------------|
| `Value` enum | All value literals (int, float, string, bool, null, enum, list, object) |
| `Type` enum | Type references (`String`, `[String]`, `String!`) |
| `TypeCondition` | Fragment type conditions (`on User`) |
| `DirectiveLocation` | Where a directive can be applied |
| `Number` | Integer values (also loses raw text) |

## Appendix B: Information `apollo-parser` Has That `graphql-parser` Lacks

| Info | apollo-parser | graphql-parser |
|------|---------------|----------------|
| Full spans (start + end) on all nodes | ✅ via `text_range()` | ⚠️ Start-only `Pos` on most; none on values/types |
| Trivia (whitespace, comments, commas) | ✅ Preserved in CST | ❌ Discarded |
| Error nodes in tree | ✅ Error tokens are CST leaves | ❌ Errors are separate |
| Schema extensions (`extend schema`) | ✅ Supported | ❌ Not supported |
| Raw text of all tokens | ✅ CST leaves store text | ❌ Parsed/converted values only |
| Descriptions as separate nodes with span | ✅ `Description` node | ⚠️ `Option<String>` field, no span |

## Appendix C: Information `graphql-parser` Has That `apollo-parser` Lacks

| Info | graphql-parser | apollo-parser |
|------|---------------|---------------|
| Parsed `i64` from int literals | ✅ `Number(i64)` | ❌ Raw text only; semantic value computed by consumer |
| Parsed `f64` from float literals | ✅ `Float(f64)` | ❌ Raw text only |
| `BTreeMap` for object values | ✅ Ordered map | ❌ Sequence of key-value pairs (order preserved differently) |
| `into_static()` lifetime conversion | ✅ Built-in | ❌ Not applicable (no lifetime parameter) |

*Note: `apollo-parser`'s approach of keeping raw text is arguably superior for a parser library — semantic interpretation belongs in downstream consumers. The new `libgraphql` structure should preserve raw text AND offer parsed convenience accessors.*
