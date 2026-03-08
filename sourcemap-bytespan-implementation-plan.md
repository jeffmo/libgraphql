# ByteSpan + SourceMap Optimization Plan

## Context

**Problem:** Every token, trivia token, AST node (~42 types), and parse error stores a `GraphQLSourceSpan` (~104 bytes: two `SourcePosition` values + `Option<PathBuf>`). The lexer eagerly computes line/col/utf16 at every character boundary (5 field writes per `consume()` call), and clones a `PathBuf` on every `make_span()` call (~26 sites). For a 1.2MB schema this means ~250K PathBuf clones.

**Why now:** Custom AST (Section 4.2 Phase 4d) is complete. 672 tests + benchmark suite provide a safety net. This is the natural time to attempt the optimization.

**Intended outcome:** Replace `GraphQLSourceSpan` with `ByteSpan` (8 bytes, `Copy`) everywhere. Build a shared `SourceMap` once via O(n) pre-pass. Resolve line/col lazily on demand. Both UTF-8 and UTF-16 columns losslessly recoverable. Net perf win confirmed by benchmarks before committing.

**Previous attempt failed** because it tried to collect `line_starts` inside lexer hot paths. This plan uses a pre-pass strategy that completely decouples newline scanning from lexing.

---

## Architecture After Change

```
  StrGraphQLTokenSource
  ┌────────────────────────────────┐
  │ source: &'src str              │
  │ source_map: SourceMap<'src>    │──build in ctor (pre-pass)
  │ curr_byte_offset: usize        │
  │ pending_trivia, finished       │
  └────────────────────────────────┘
       │ impl GraphQLTokenSource<'src>
       │   - Iterator<Item = GraphQLToken<'src>>
       │   - source_map(&self) -> &SourceMap<'src>
       │   - into_source_map(self) -> SourceMap<'src>
       │
       │ emits tokens with ByteSpan
       v
  GraphQLParser<S: GraphQLTokenSource<'src>>
       │
       │ consumes S → calls S.into_source_map()
       v
  ParseResult<'src, TAst>
  ┌─────────────────────────────────────┐
  │ Ok { ast, source_map }              │
  │ Recovered { ast, errors, source_map}│
  └─────────────────────────────────────┘
       │
       │ source_map.resolve_span(ByteSpan) → SourceSpan (transient, on demand)
       v
  Error formatting, IDE/LSP, etc.
```

---

## Naming Conventions

| Type | Role |
|------|------|
| `ByteSpan` | Compact storage span (8 bytes, `Copy`). Stored on all tokens, AST nodes, errors. |
| `SourceSpan` | Rich resolved span with line/col/file. Transient — produced on demand, never stored. Renamed from `GraphQLSourceSpan`. |
| `SourcePosition` | Single resolved position (line, col_utf8, col_utf16, byte_offset). Kept as-is. |
| `SourceMap<'src>` | Maps byte offsets → line/col. Built once per parse, shared across all lookups. |

**Resolution methods:**
- `ByteSpan::resolve_source_span(source_map: &SourceMap) → SourceSpan` — convenience on ByteSpan
- `SourceMap::resolve_span(ByteSpan) → SourceSpan` — resolve both endpoints + attach file_path
- `SourceMap::resolve_offset(u32) → SourcePosition` — resolve a single byte offset

---

## Key Design Decisions

1. **Pre-pass for line_starts** — `StrGraphQLTokenSource` scans source for `\n`/`\r`/`\r\n` in its constructor, building `line_starts` into its internal `SourceMap`. Other token sources build their SourceMaps differently (e.g. `SourceMap::empty()` for proc-macro). A future streaming token source would build `line_starts` incrementally as chunks arrive.
2. **SourceMap on the `GraphQLTokenSource` trait** — all token sources implement `source_map(&self) -> &SourceMap<'src>` and `into_source_map(self) -> SourceMap<'src>`. This ensures uniform line/col resolution for all consumers (error formatting, IDEs over token streams, IDEs over ASTs, etc). The blanket `impl<T: Iterator> GraphQLTokenSource for T` is removed — token sources become intentional implementations.
3. **SourceMap inside `ParseResult`** — `ParseResult<'src, TAst>` gains a lifetime and carries `source_map: SourceMap<'src>` on both variants. `TAst` already implicitly carries `'src` (e.g. `Document<'src>`), so this is a natural extension. The parser consumes the token source via `into_source_map()` and bundles the result.
4. **ByteSpan everywhere** — public API change. All types use `ByteSpan { start: u32, end: u32 }`.
5. **Column recovery on demand** — binary search `line_starts` → line, then count chars/UTF-16 units from line start to byte offset. Only happens on error formatting and IDE queries (cold path).
6. **`GraphQLSourceSpan` renamed to `SourceSpan`** — kept as a transient "resolved" type returned by `SourceMap::resolve_span()`. Not stored in any struct.

---

## Phase Completion Protocol

Every phase (except Phase 0) ends with:
1. `cargo test` — full workspace (all tests pass)
2. `cargo clippy --tests` — full workspace (clean)
3. 15-minute fuzz test run (skip for Phase 1 since no behavioral changes)
4. `sl commit` with thorough description of the phase's changes

---

## Phases

### Phase 0: Baseline Benchmarks
- Run `cargo bench` 3x, save criterion output
- Record: schema_parse (github, shopify_admin), executable_parse (simple, complex), lexer (github)
- Verify <5% variance between runs

### Phase 1: Add ByteSpan + SourceMap Types (Additive Only)

No existing code modified. All 672 tests still pass unchanged.

**New files:**
- `src/byte_span.rs` — `ByteSpan { start: u32, end: u32 }`, `#[derive(Copy)]`, `#[repr(C)]`
- `src/source_map.rs` — `SourceMap<'src> { file_path, source, line_starts }`

**SourceMap key methods:**
- `new(source, file_path)` — O(n) pre-pass building `line_starts: Vec<u32>` from newlines
- `empty()` — for token sources without source text (e.g. proc-macro)
- `resolve_offset(byte_offset: u32) → SourcePosition` — binary search line_starts, count chars for col
- `resolve_span(ByteSpan) → SourceSpan` — resolve both endpoints + attach file_path
- `file_path() → Option<&Path>`
- `source() → Option<&'src str>`

**ByteSpan convenience method:**
- `resolve_source_span(source_map: &SourceMap) → SourceSpan` — calls `source_map.resolve_span(self)`

**Conversion bridge (temporary, for Phase 1 round-trip validation):**
- `GraphQLSourceSpan::to_byte_span() → ByteSpan`

**Unit tests for SourceMap:**
- Empty string, single line ASCII, multi-line `\n`/`\r`/`\r\n`/mixed
- BOM at start and mid-line (3 bytes → 1 col)
- Non-ASCII: emoji (4-byte UTF-8, 2 UTF-16 units), CJK, accented chars
- Offset at exact line boundary, at EOF
- **Round-trip validation harness:** parse with current lexer → build SourceMap → resolve every token's byte_offset → assert line/col/utf16 match original SourcePosition exactly

**Wire into `lib.rs`:** `mod byte_span; mod source_map;` + re-exports

**Phase completion:** `cargo test`, `cargo clippy --tests`, `sl commit`

### Phase 2: Rename `GraphQLSourceSpan` → `SourceSpan`

Rename the struct and update all references across the codebase. This is a mechanical rename phase — no behavioral changes, just naming alignment.

- Rename `GraphQLSourceSpan` → `SourceSpan` in `src/graphql_source_span.rs` (or rename file to `source_span.rs`)
- Update all imports and references across all files
- Update `SourceMap::resolve_span()` return type to `SourceSpan`

**Phase completion:** `cargo test`, `cargo clippy --tests`, `sl commit`

### Phase 3: Update `GraphQLTokenSource` Trait + Migrate Tokens + Lexer

This is the highest-impact phase. The trait gains SourceMap methods, the lexer simplifies.

**Update `GraphQLTokenSource` trait** (`src/token_source/graphql_token_source.rs`):
- Remove blanket impl `impl<T: Iterator<Item = GraphQLToken>> GraphQLTokenSource for T`
- Add required methods:
  ```rust
  pub trait GraphQLTokenSource<'src>: Iterator<Item = GraphQLToken<'src>> {
      /// Borrow the SourceMap. Available at any point during
      /// tokenization — useful for IDE-like tools that need
      /// line/col lookups mid-stream.
      fn source_map(&self) -> &SourceMap<'src>;

      /// Consume this token source and return the owned SourceMap.
      /// Called by the parser after consuming all tokens (EOF).
      fn into_source_map(self) -> SourceMap<'src>;
  }
  ```

**Modify `GraphQLToken.span` and `GraphQLTriviaToken` spans → `ByteSpan`**
- `src/token/graphql_token.rs`: `pub span: ByteSpan`
- `src/token/graphql_trivia_token.rs`: all variant spans → `ByteSpan`

**Simplify `StrGraphQLTokenSource`** (`src/token_source/str_to_graphql_token_source.rs`):

Add field: `source_map: SourceMap<'src>` (built in constructor via pre-pass)

Remove 4 fields from struct:
- `curr_line` (line tracking moves to SourceMap pre-pass)
- `curr_col_utf8` (column computed on demand from SourceMap)
- `curr_col_utf16` (same)
- `last_char_was_cr` (only needed for line tracking)

Remaining mutable state: `curr_byte_offset`, `pending_trivia`, `finished`

Implement trait methods:
- `source_map(&self) -> &SourceMap<'src>` → `&self.source_map`
- `into_source_map(self) -> SourceMap<'src>` → `self.source_map`

Simplify methods:
- **`consume()`**: becomes ~6 lines (just advance `curr_byte_offset` by char's UTF-8 byte len)
- **`curr_position()`**: removed entirely
- **`make_span(start: u32)`**: `ByteSpan::new(start, self.curr_byte_offset as u32)` — no PathBuf clone
- **`skip_whitespace()`**: ~20 lines (just advance past whitespace/newline bytes, no line/col tracking)
- **`lex_block_string()`**: ~30 lines (byte-scan for closing `"""`, no line/col tracking)
- **`lex_comment()`**: byte-scan to EOL, remove `compute_columns_for_span()` call
- **`lex_name()`**: byte-scan for name chars, `self.curr_byte_offset = i` (remove col updates)
- **`lex_dot_or_ellipsis(start: u32)`**: remove `first_dot_line` / `self.curr_line` checks — these are redundant because `skip_whitespace_same_line()` never crosses line boundaries, so dots separated by newlines always fall through to the default error case naturally
- **`skip_whitespace_same_line()`**: unchanged (already only uses `peek_char`/`consume`)
- All `let start = self.curr_position()` → `let start = self.curr_byte_offset as u32`

**Remove `compute_columns_for_span()`** — no longer called by any lexer method

**Update lexer tests:**
- Position tests: use `source_map.resolve_offset()` to verify line/col/utf16
- Token kind tests: update span type annotations

**Phase completion:** `cargo test`, `cargo clippy --tests`, 15min fuzz test, `sl commit`

**Benchmark checkpoint:** Run `cargo bench`, compare lexer/parse throughput vs Phase 0 baseline. Expect improvement. If regression >3%: investigate pre-pass overhead.

### Phase 4: Migrate Parser + ParseResult to ByteSpan

**Update `ParseResult`** (`src/parse_result.rs`):
- Add lifetime: `ParseResult<'src, TAst>`
- Both variants gain `source_map: SourceMap<'src>`:
  ```rust
  pub enum ParseResult<'src, TAst> {
      Ok {
          ast: TAst,
          source_map: SourceMap<'src>,
      },
      Recovered {
          ast: TAst,
          errors: Vec<GraphQLParseError>,
          source_map: SourceMap<'src>,
      },
  }
  ```
- Update all methods (`valid_ast()`, `into_valid_ast()`, `into_ast()`, `errors()`, `format_errors()`, `From` impl, etc.) to carry the lifetime and propagate `source_map`
- `format_errors()` uses the bundled `source_map` directly (no external parameter needed)

**Update `GraphQLParser`** (`src/graphql_parser.rs`, ~3350 lines):

Mechanical changes:
- `last_end_position: Option<SourcePosition>` → `Option<u32>`
- `make_span(start: SourceSpan) → SourceSpan` → `make_span(start: ByteSpan) → ByteSpan`
- `make_span_ref(&SourceSpan) → SourceSpan` → `make_span_ref(&ByteSpan) → ByteSpan`
- `eof_span()` / `document_span()` → return `ByteSpan`
- `OpenDelimiter.span` → `ByteSpan`
- 18 direct `SourceSpan::new(...)` calls → `ByteSpan::new(...)` (`.start`/`.end` instead of `.start_inclusive`/`.end_exclusive`)
- 44 `.span.clone()` calls → just `.span` (Copy)
- All `parse_*` methods: `start.start_inclusive` → `start.start`, `start.end_exclusive` → `start.end`

Parser completion: after consuming EOF token, call `self.token_stream.into_source_map()` and bundle into `ParseResult`:
```rust
let source_map = self.token_stream.into_source_map();
ParseResult::Ok { ast: document, source_map }
```

Note: `GraphQLTokenStream` wraps the token source for peek/consume. It will need an `into_source_map()` method that forwards to the underlying token source.

**Update public parse functions** (in `lib.rs` or convenience wrappers):
- `parse_schema_document(source)` → returns `ParseResult<'_, SchemaDocument<'_>>`
- Same for `parse_executable_document`, `parse_mixed_document`
- No separate wrapper needed — `ParseResult` carries the SourceMap

**Update parser tests** — same pattern as lexer tests

**Phase completion:** `cargo test`, `cargo clippy --tests`, 15min fuzz test, `sl commit`

### Phase 5: Migrate AST Nodes + Errors to ByteSpan

**AST nodes** (`src/ast/*.rs`, ~42 files):
- `pub span: SourceSpan` → `pub span: ByteSpan`
- Remove `use crate::SourceSpan` → `use crate::ByteSpan`

**Errors** (`src/graphql_parse_error.rs`):
- `GraphQLParseError.span` → `ByteSpan`
- `GraphQLErrorNote.span` → `Option<ByteSpan>`
- `format_detailed(&self, source_map: &SourceMap)` — resolve spans via SourceMap for display
- `format_oneline(&self, source_map: &SourceMap)` — same

**Compat layer** (`src/parser_compat/graphql_parser_v0_4/`):
- `pos_from_span` / `end_pos_from_span` / `type_ext_pos_from_span` → take `&SourceMap` parameter
- Thread SourceMap through all `to_*` conversion functions via a context struct
- `from_*` functions create `ByteSpan` from graphql_parser positions

**Phase completion:** `cargo test`, `cargo clippy --tests`, 15min fuzz test, `sl commit`

### Phase 6: Migrate RustMacroGraphQLTokenSource

In `crates/libgraphql-macros/`:

- Implement `GraphQLTokenSource` trait explicitly (was previously via blanket impl):
  - `source_map()` → returns `&self.source_map` (a `SourceMap::empty()`)
  - `into_source_map()` → returns `self.source_map`
- Emit `ByteSpan` from `proc_macro2::Span::byte_range()`
- `span_map: HashMap<(usize, usize), Span>` → `HashMap<u32, Span>` (keyed by byte offset, simpler)
- Error display via `compile_error!` with original `Span` — unaffected by SourceMap changes

**Phase completion:** `cargo test`, `cargo clippy --tests`, 15min fuzz test, `sl commit`

### Phase 7: Cleanup + Final Benchmarks

- Remove any deprecated shims / temporary conversion bridges
- Remove `compute_columns_for_span()` if still present
- Check `clippy::large_enum_variant` allows on `Nullability`/`TypeAnnotation` — may no longer be needed with 8-byte spans
- Run full `cargo bench`, compare all 7 groups vs Phase 0 baseline

**Success criteria:** net perf improvement across schema_parse and executable_parse benchmarks. No regression >3% on any benchmark.

**Abort criteria:** if overall performance regresses, revert and document findings.

**Phase completion:** `cargo test`, `cargo clippy --tests`, 15min fuzz test, `sl commit`

---

## Critical Files

| File | Change |
|------|--------|
| `crates/libgraphql-parser/src/byte_span.rs` | **NEW** — ByteSpan type |
| `crates/libgraphql-parser/src/source_map.rs` | **NEW** — SourceMap type |
| `crates/libgraphql-parser/src/graphql_source_span.rs` | Renamed to `source_span.rs`, struct renamed `GraphQLSourceSpan` → `SourceSpan` |
| `crates/libgraphql-parser/src/token_source/graphql_token_source.rs` | Remove blanket impl, add `source_map()` + `into_source_map()` methods |
| `crates/libgraphql-parser/src/token_source/str_to_graphql_token_source.rs` | **MAJOR** — add source_map field, remove 4 fields, simplify all hot paths, implement trait |
| `crates/libgraphql-parser/src/token/graphql_token.rs` | span → ByteSpan |
| `crates/libgraphql-parser/src/token/graphql_trivia_token.rs` | spans → ByteSpan |
| `crates/libgraphql-parser/src/graphql_parser.rs` | ~3350 lines: mechanical ByteSpan migration, last_end_position → u32, call into_source_map() |
| `crates/libgraphql-parser/src/parse_result.rs` | Add `'src` lifetime, carry `source_map: SourceMap<'src>` on both variants |
| `crates/libgraphql-parser/src/graphql_parse_error.rs` | span → ByteSpan, formatting takes &SourceMap |
| `crates/libgraphql-parser/src/source_position.rs` | kept for SourceMap::resolve_offset return type |
| `crates/libgraphql-parser/src/ast/*.rs` | ~42 files: span field → ByteSpan |
| `crates/libgraphql-parser/src/ast/ast_node.rs` | append_span_source_slice updated for ByteSpan |
| `crates/libgraphql-parser/src/lib.rs` | module renames + new declarations + re-exports |
| `crates/libgraphql-parser/src/parser_compat/graphql_parser_v0_4/*.rs` | thread SourceMap through compat layer |
| `crates/libgraphql-macros/src/rust_macro_graphql_token_source.rs` | explicit trait impl, emit ByteSpan, simplify span_map |
| `crates/libgraphql-parser/benches/parse_benchmarks.rs` | update for new ParseResult API |

---

## Reusable Existing Code

- `SourcePosition::new()` — reused as return type from `SourceMap::resolve_offset()`
- `SourceSpan::new()` / `::with_file()` — reused inside `SourceMap::resolve_span()` for transient resolved spans
- Benchmark infrastructure (`benches/parse_benchmarks.rs`) — existing 7 groups provide before/after comparison
- `is_name_continue_byte()` in lexer — unchanged, used in simplified `lex_name`
- Fuzz test infrastructure (`fuzz/`) — used for 15min fuzz runs at end of each phase

---

## Verification (per phase)

1. `cargo test` — full workspace, all tests pass
2. `cargo clippy --tests` — full workspace, clean
3. 15-minute fuzz test (Phase 3+): `./scripts/run-fuzz-tests.sh` or equivalent
4. Round-trip validation (Phase 1): SourceMap-resolved positions match original SourcePosition for every token in every test fixture
5. Benchmarks (Phase 3, Phase 7): `cargo bench --package libgraphql-parser` — compare all 7 groups vs Phase 0 baseline

---

## Complexity Warnings (from previous attempt)

| Risk | Mitigation |
|------|-----------|
| line_starts in hot paths | Pre-pass strategy: completely decoupled from lexer. Internal to StrGraphQLTokenSource ctor. |
| UTF-8 mid-codepoint byte offsets | Lexer always produces char-boundary-aligned offsets. SourceMap uses `str::chars()` from line start — which starts at a valid char boundary. Debug assert for safety. |
| UTF-16 column recovery | On-demand from source text via `char::len_utf16()` count. No pre-computed Utf16LineInfo table needed. |
| Two-phase migration | Phases are testable independently. Phase 1 is purely additive. Phase 3 can be benchmarked before continuing. |
| BOM handling | BOM (0xEF 0xBB 0xBF) is NOT a line terminator → not in line_starts. Column counting via `chars()` naturally handles BOM as 1 char (U+FEFF). |
| lex_dot_or_ellipsis same-line check | `skip_whitespace_same_line()` never crosses newlines → the `curr_line == first_dot_line` guard is redundant and can be safely removed. |
| Blanket impl removal | `GraphQLTokenSource` blanket impl removed. Both `StrGraphQLTokenSource` and `RustMacroGraphQLTokenSource` need explicit impls. This is intentional — token sources should be deliberate. |
| ParseResult lifetime | `ParseResult<'src, TAst>` adds `'src`. Since `TAst` already carries `'src` implicitly (e.g. `Document<'src>`), most callers already have the lifetime in scope. |
| Streaming token sources | Not blocked. A future streaming source implements the trait: builds SourceMap incrementally as chunks arrive, `source_map()` returns partial-but-correct map for content seen so far. |
