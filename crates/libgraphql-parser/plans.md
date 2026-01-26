# libgraphql-parser — Consolidated Plans & Remaining Work

**Last Updated:** 2026-01-20

This document consolidates all remaining work for the `libgraphql-parser` crate.
It supersedes any individual planning documents under the root `/plans.md` document.

## Document Maintenance Notes

When updating this document:

1. **Completed items:** Move wholly-completed plan items to the "Past Completed Work" section at the end of this document. Include a simple title and terse description only.
2. **Plan identifiers:** NEVER re-number existing plan items (e.g., 4.3, 2.1). This ensures references to plan IDs remain valid over time.
3. **Partial completion:** If a plan item is partially done, leave it in place and update its description to reflect remaining work.

---

## Current State Summary

**Test Status:** 383 tests passing, 4 doc-tests passing

**Core Implementation: ✅ COMPLETE**
- `StrGraphQLTokenSource` lexer (~1130 lines) — zero-copy with `Cow<'src, str>`
- `GraphQLParser<S>` parser (~3200 lines) — recursive descent, generic over token source
- `ParseResult<T>` — partial AST + errors for IDE-friendly parsing
- Error infrastructure — `GraphQLParseError`, `GraphQLParseErrorKind`, `GraphQLErrorNote`
- All GraphQL constructs parsed: values, types, directives, operations, fragments, type definitions, extensions

**Remaining Work Categories:**
1. Vendored GraphQL Documents (Section 1)
2. Testing Gaps (Section 2)
3. Performance & Fuzzing (Section 3)
4. Future Enhancements (Section 4)
5. libgraphql-core Integration (Section 5)
6. Documentation (Section 6)

---

## Section 1: Vendored GraphQL Documents

### Purpose
Create a central repository for third-party GraphQL documents (schemas, operations, test fixtures) that multiple crates in the `libgraphql` workspace can reference for testing, benchmarking, and validation.

**Note:** This is for GraphQL *documents* only (`.graphql` files), not test *code*. For coverage gaps identified in external test suites, we write our own tests (see Section 2.2).

### Current Progress
- No vendored documents exist yet
- License verification not yet performed

### Location
```
libgraphql/vendored/
├── README.md                     # License attributions, sources
├── schemas/
│   ├── github/                   # GitHub's public schema
│   │   ├── LICENSE
│   │   └── schema.graphql
│   ├── gitlab/                   # GitLab's public schema
│   │   ├── LICENSE
│   │   └── schema.graphql
│   └── examples/
│       └── star-wars.graphql     # Classic example schema
├── operations/                   # Complex operation examples
│   └── ...
└── test-fixtures/                # Test fixture documents from external projects
    ├── graphql-js/               # Fixture documents from graphql-js (MIT)
    │   ├── LICENSE
    │   └── *.graphql
    └── graphql-parser/           # Fixture documents from graphql-parser (MIT/Apache-2.0)
        ├── LICENSE
        └── *.graphql
```

### Tasks

1. **License verification (BLOCKING)**
   - Verify licenses for GitHub/GitLab public schemas
   - Verify graphql-js license (MIT) permits fixture vendoring
   - Verify graphql-parser license (MIT/Apache-2.0) permits fixture vendoring
   - Document attribution requirements in `README.md`

2. **Collect real-world schemas**
   - GitHub public schema
   - GitLab public schema
   - Star Wars example schema
   - Other large public GraphQL APIs (if licenses permit)

3. **Extract test fixture documents from graphql-js**
   - Identify `.graphql` files used as test fixtures
   - Copy fixture files (not test code) with license attribution

4. **Extract test fixture documents from graphql-parser**
   - Identify `.graphql` files used as test fixtures
   - Copy fixture files with license attribution

### Definition of Done
- [ ] All vendored content has documented license compliance
- [ ] `vendored/README.md` exists with attribution for each source
- [ ] At least 3 real-world schemas vendored
- [ ] At least one test/benchmark in `libgraphql-parser` uses vendored fixtures

---

## Section 2: Testing Gaps

### 2.1 Coverage-Driven Test Discovery

**Purpose:** Use test coverage data as a *discovery tool* to identify untested code paths that may represent edge cases, error-prone areas, or subtle spec nuances worth testing.

**Philosophy:** We are NOT aiming for a specific coverage percentage target. Coverage metrics are a means to find blind spots, not a goal in themselves. The objective is to ensure important, tricky, or error-prone code paths are tested — not to inflate a number. Coverage will naturally increase as a side effect of thorough testing.

**Current Coverage:** ~62% lines (useful as a baseline, not a target)

**Priority: HIGH**

#### Approach

1. Run coverage analysis to identify untested lines/branches
2. For each uncovered area, assess:
   - Is this an edge case worth testing?
   - Is this error handling that should be verified?
   - Is this a spec-compliance nuance?
   - Is this dead code that should be removed?
3. Write tests for areas that represent meaningful behavior
4. Remove dead code rather than writing tests for unreachable paths

#### Identified Areas Needing Tests

Based on coverage analysis, these areas warrant attention:

**Token & Lexer (Target: `token/tests/`)**

| Area | Why It Matters |
|------|----------------|
| `is_punctuator()` edge cases | Ensure all 14 punctuators correctly identified |
| `parse_int_value()` overflow | Integer overflow handling is error-prone |
| `parse_string_value()` escape sequences | Complex spec rules for `\uXXXX`, `\u{...}` |
| Unicode escape edge cases | Empty braces, out-of-range codepoints, invalid hex |

**Security-Critical String Escape Tests (Target: `token/tests/`)**

Ensure comprehensive testing of potentially dangerous escape sequences:

| Category | Test Cases | Why Security-Critical |
|----------|------------|----------------------|
| Surrogate pairs | `\uD800` (lone high), `\uDFFF` (lone low), `\uD83D\uDE00` (valid pair → 😀) | Malformed surrogates can cause crashes or undefined behavior |
| Unicode range validation | `\u{110000}`, `\u{FFFFFFFF}` (beyond U+10FFFF) | Out-of-range values must error, not produce invalid UTF-8 |
| Bidi control chars | `\u202E` (RTL Override), `\u200E` (LRM), `\u200F` (RLM) | Can be used for display spoofing attacks |
| Zero-width chars | `\u200B` (ZWSP), `\u200C` (ZWNJ), `\u200D` (ZWJ), `\uFEFF` (BOM) | Invisible chars can hide malicious content |
| NUL byte injection | `\u0000`, `"a\u0000b"` (embedded NUL) | NUL bytes can truncate strings in C-based systems |
| Line separators | `\u2028` (Line Sep), `\u2029` (Paragraph Sep) | Can break JSON parsers expecting `\n` only |
| Empty/malformed escapes | `\u{}`, `\u{GGGG}`, `\u{00000000000000041}` | Edge cases in escape parsing logic |
| Unterminated escapes | `"\"` (backslash at EOF), `"\u00"` (incomplete) | Must error gracefully, not panic |

**Parser Error Handling (Target: `tests/graphql_parser_tests.rs`)**

| Area | Why It Matters |
|------|----------------|
| Value overflow errors | Numeric limits are easy to get wrong |
| Unclosed delimiter recovery | Error recovery logic is subtle |
| Reserved name validation | `true`/`false`/`null` context rules per spec |
| Document kind enforcement | Schema vs executable distinction |

**ParseResult & Error Infrastructure**

| Area | Why It Matters |
|------|----------------|
| `ParseResult` state transitions | Recovered vs failed states affect API users |
| Error formatting | User-facing output should be verified |

### Definition of Done
- [ ] Coverage analysis run and uncovered areas catalogued
- [ ] Each uncovered area assessed for test-worthiness
- [ ] Tests written for meaningful uncovered behavior
- [ ] Dead code removed where found
- [ ] Tests follow CLAUDE.md conventions

---

### 2.2 External Test Suite Gap Analysis

**Purpose:** Analyze test suites from `graphql-js` and `graphql-parser` to identify test scenarios we don't currently cover, then write our own tests for those gaps.

**Approach:** We analyze external test *code* to find coverage gaps, but we write our own test implementations rather than vendoring test code. This avoids license complexity and keeps our tests consistent with project conventions.

**Current Progress:** Not started.

**Priority: HIGH**

#### Tasks

1. **Analyze graphql-js lexer tests**
   - Source: `graphql-js/src/__tests__/lexer-test.ts`
   - For each test case, check if we have equivalent coverage
   - Document any gaps found
   - Write our own tests for uncovered scenarios

2. **Analyze graphql-js parser tests**
   - Source: `graphql-js/src/__tests__/parser-test.ts`
   - Focus on edge cases, error handling, spec compliance
   - Document gaps and write our own tests

3. **Analyze graphql-parser tests**
   - Source: `graphql-rust/graphql-parser/tests/`
   - Focus on Rust-specific edge cases
   - Document gaps and write our own tests

4. **Create gap analysis document**
   - Track which external tests we've analyzed
   - Document which gaps were found
   - Link to our tests that fill each gap

#### Expected Gap Categories
Based on planning docs, likely gaps include:
- Lexer edge cases: adjacent punctuators, ellipsis variants, Unicode handling
- String escapes: surrogate pairs, control characters, edge cases
- Number edge cases: overflow, underflow, special floats
- Parser recovery: various unclosed delimiter scenarios
- Reserved names: `true`/`false`/`null` in various contexts

### Definition of Done
- [ ] graphql-js lexer tests analyzed, gaps identified and filled
- [ ] graphql-js parser tests analyzed, gaps identified and filled
- [ ] graphql-parser tests analyzed, gaps identified and filled
- [ ] Gap analysis documented (can be in this file or separate)
- [ ] All new tests follow CLAUDE.md conventions

---

### 2.3 RustMacroGraphQLTokenSource Parser Tests

**Purpose:** Verify the parser works correctly with the proc-macro token source, not just `StrGraphQLTokenSource`.

**Current Progress:** Parser tests only cover `StrGraphQLTokenSource`. No parser tests use `RustMacroGraphQLTokenSource`.

**Priority: MEDIUM**

**Test Location:** Most tests should live in the `libgraphql-macros` crate since they test the integration of `RustMacroGraphQLTokenSource` with the parser. Any tests that more directly test `libgraphql-parser` behavior (independent of the token source) can go in `libgraphql-parser`.

#### Tasks

1. **Create parser tests using RustMacroGraphQLTokenSource**
   - Location: `crates/libgraphql-macros/src/tests/`
   - Use `quote!{}` macro to generate token streams
   - Parse schema documents, executable documents, mixed documents

2. **Cross-validate token sequences**
   - Parse same GraphQL with both token sources
   - Compare token kinds and relative ordering
   - Document expected differences (byte_offset coordinate spaces, col_utf16 availability)

3. **Test edge cases specific to proc-macro tokenization**
   - Rust tokenizer quirks (comment stripping, whitespace handling)
   - Negative numbers (`-123` requires combining Punct + Literal)
   - Spread operator (`...` requires combining three Puncts)
   - Block strings (combining three string literals)

### Definition of Done
- [ ] At least 20 parser tests in `libgraphql-macros` using `RustMacroGraphQLTokenSource`
- [ ] Cross-validation test exists comparing both token sources
- [ ] Known differences documented

---

### 2.4 Position Accuracy Tests

**Purpose:** Verify line/col_utf8/col_utf16/byte_offset values are accurate across both token sources.

**Current Progress:**
- StrGraphQLTokenSource position tests are comprehensive (emoji surrogate pairs, BMP chars, all line endings, BOM handling)
- RustMacroGraphQLTokenSource has basic position test (`test_position_tracking`) but lacks comprehensive coverage
- Cross-validation between token sources not yet done

**Priority: MEDIUM**

#### Tasks

1. **StrGraphQLTokenSource position tests** ✅ COMPLETE
   - UTF-16 surrogate pair handling (emoji) ✅
   - Multi-byte UTF-8 (accented characters) ✅
   - Line endings: `\n`, `\r`, `\r\n` ✅
   - BOM handling at file start ✅

2. **RustMacroGraphQLTokenSource position tests** (remaining)
   - Verify `col_utf16()` returns `None`
   - Verify byte_offset is Rust-file-relative (not GraphQL-document-relative)
   - Tokens spanning multiple Rust tokens (spread, block strings)

3. **Document coordinate space differences**
   - `StrGraphQLTokenSource`: byte_offset relative to `&str` start
   - `RustMacroGraphQLTokenSource`: byte_offset relative to Rust source file

### Definition of Done
- [ ] Position tests for both token sources pass
- [ ] Coordinate space differences documented in rustdoc

---

### 2.5 Integration Tests with Real Schemas

**Purpose:** Verify parser handles real-world, large, complex GraphQL schemas correctly.

**Current Progress:** No integration tests with real schemas.

**Priority: MEDIUM**

**Depends on:** Section 1 (vendored documents)

#### Tasks

1. **Parse GitHub schema** (once vendored)
2. **Parse GitLab schema** (once vendored)
3. **Parse Star Wars example schema**
4. **Stress tests**
   - Very deeply nested types (100+ levels)
   - Types with 1000+ fields
   - Documents with 1000+ definitions

### Definition of Done
- [ ] At least 3 real-world schemas parse successfully
- [ ] No panics on malformed input
- [ ] Stress tests complete in reasonable time (<5s)

---

### 2.6 Differential Tests Against graphql_parser Crate

**Purpose:** Build confidence that `libgraphql-parser` is a compatible replacement for the `graphql_parser` crate.

**Current Progress:** No differential tests exist.

**Priority: LOW (confidence-building, not blocking)**

#### Tasks

1. **Create test harness**
   - Parse same input with both parsers
   - Compare success/failure outcomes
   - Compare AST "shape" (structure, not spans)

2. **Run on vendored document corpus**
   - Track any discrepancies
   - Investigate each: spec violation in `graphql_parser` or bug in `libgraphql-parser`

3. **Document known differences**
   - `libgraphql-parser` may be stricter or more lenient in some cases
   - Note any intentional spec compliance differences

4. **Upstream bug fixes to graphql-parser**
   - When we discover bugs in `graphql-parser`, document each bug
   - Create upstream PRs to fix issues in `graphql-parser` as we encounter them
   - Track PR status in this document or a linked issue

### Definition of Done
- [ ] Differential test harness exists
- [ ] 95%+ agreement on test corpus
- [ ] Discrepancies documented with rationale
- [ ] Any `graphql-parser` bugs found have upstream PRs submitted

---

## Section 3: Performance & Fuzzing

### 3.1 Fuzz Testing

**Purpose:** Ensure lexer and parser don't panic on arbitrary/malformed input. Security-critical for parsing untrusted GraphQL.

**Current Progress:** No fuzz testing infrastructure.

**Priority: HIGH (security)**

#### Tasks

1. **Set up cargo-fuzz**
   - Create `/crates/libgraphql-parser/fuzz/`
   - Fuzz target for `StrGraphQLTokenSource`
   - Fuzz target for `GraphQLParser::parse_schema_document()`
   - Fuzz target for `GraphQLParser::parse_executable_document()`
   - Fuzz target for `GraphQLParser::parse_mixed_document()`

2. **Run fuzzer**
   - Minimum 1 hour of fuzzing before declaring success
   - Fix any crashes discovered
   - Add regression tests for crash inputs

3. **Consider structured fuzzing**
   - Use `arbitrary` crate to generate valid-ish GraphQL
   - More likely to find deep parser bugs

4. **Document fuzzing in README.md**
   - Clear instructions on how to run fuzz tests
   - Required tools and setup steps
   - Track the longest continuous fuzzing duration completed
   - Document any crashes found during that period (and their resolution status)

### Definition of Done
- [ ] Fuzz targets exist for lexer and all 3 parser entry points
- [ ] 1+ hour of fuzzing completed with no crashes
- [ ] Any discovered crashes fixed and regression-tested
- [ ] README.md documents how to run fuzz tests
- [ ] README.md tracks longest fuzzing duration and results

---

### 3.2 Performance Benchmarks

**Purpose:** Establish performance baseline and ensure `libgraphql-parser` is competitive with `graphql_parser` crate.

**Current Progress:** No benchmarks exist.

**Priority: LOW (optimization can come later)**

**Depends on:** Section 1 (vendored documents for benchmark fixtures)

#### Tasks

1. **Create benchmark suite**
   - Location: `/crates/libgraphql-parser/benches/`
   - Use `criterion` crate

2. **Benchmark scenarios**
   - Small schema (~10 types)
   - Medium schema (~100 types, e.g., Star Wars)
   - Large schema (~500+ types, e.g., GitHub)
   - Complex operations with deep nesting

3. **Compare against graphql_parser crate**
   - Target: within 2x of `graphql_parser` performance
   - Document any significant differences

4. **Identify optimization opportunities**
   - Profile hot paths
   - Consider `memchr` for fast character scanning
   - Review allocation patterns

### Definition of Done
- [ ] Benchmark suite exists with `criterion`
- [ ] At least 3 benchmark scenarios using vendored schemas
- [ ] Performance within 2x of `graphql_parser`

---

## Section 4: Future Enhancements

### 4.1 Schema Extension Support

**Purpose:** The `extend schema` construct is currently unsupported; parser emits "unsupported" error.

**Current Progress:** Error handling exists but schema extensions are not parsed.

**Priority: LOW (rarely used)**

**Blocking:** Requires custom AST (the `graphql_parser` crate AST doesn't fully support schema extensions with directives).

#### Tasks

1. **Implement `parse_schema_extension()`**
2. **Add to `parse_schema_document()` dispatch**
3. **Tests for schema extension syntax**

#### Code References (TODOs in codebase)
- `graphql_parser.rs:2524`
- `graphql_parser.rs:2531`
- `graphql_parser.rs:2582`

### Definition of Done
- [ ] `extend schema @directive { ... }` parses correctly
- [ ] Tests for schema extensions pass
- [ ] Schema extensions appear in AST

---

### 4.2 Custom AST

**Purpose:** Replace `graphql_parser` crate AST types with custom types that:
- Include span information on all nodes
- Support schema extensions properly
- Enable trivia attachment for formatters
- Allow serde serialization of complete AST

**Current Progress:** Uses `graphql_parser` AST types via re-exports in `ast.rs`. Custom AST is explicitly deferred.

**Priority: LOW (significant effort, not blocking current use cases)**

#### Design Constraints

**Superset Requirement:** Any custom `libgraphql` AST must contain at least a superset of the information in the corresponding `graphql-parser` AST structures. This ensures lossless conversion.

**One-Way Translation:** The custom AST must include a facility to translate `libgraphql` AST → `graphql-parser` AST for compatibility with tools built on `graphql-parser`. Since `libgraphql` AST will contain additional information (spans, trivia, etc.), the reverse translation (`graphql-parser` → `libgraphql`) may not be possible or will result in information loss.

#### Tasks

1. **Design custom AST types**
   - One type per GraphQL construct
   - All nodes include `span: GraphQLSourceSpan`
   - Consider `Arc` for sharing in IDE scenarios
   - Ensure all `graphql-parser` AST fields are represented

2. **Implement translation to graphql-parser AST**
   - `impl From<LibGraphQLAst> for graphql_parser::Ast` (or similar)
   - Translation must be lossless for `graphql-parser` fields
   - Document that reverse translation is not supported

3. **Update parser to produce custom AST**
4. **Add serde support**
5. **Deprecate and remove `graphql_parser` re-exports**

#### Code References (TODOs in codebase)
- `graphql_parser.rs:1725`: "Track these when we have a custom AST"
- Multiple references to custom AST in planning docs

### Definition of Done
- [ ] Custom AST types defined for all GraphQL constructs
- [ ] AST is a superset of `graphql-parser` AST information
- [ ] `libgraphql` AST → `graphql-parser` AST translation implemented
- [ ] Parser produces custom AST
- [ ] All nodes have span information
- [ ] Serde serialization works

---

### 4.3 Richer Diagnostics Structure

**Purpose:** Enhance error tokens with severity levels, fix actions for IDE integration.

**Current Progress:** `GraphQLTokenKind::Error` has `message` and `error_notes`. Planning doc mentions richer structure.

**Priority: LOW (nice-to-have for IDE integration)**

#### Code Reference
- `graphql_token_kind.rs:111`: "TODO: Explore replacing error_notes with a richer diagnostics structure"

#### Tasks

1. **Design diagnostic structure**
   - Severity: Error, Warning, Info, Hint
   - Fix actions for IDE quick-fix support
   - Related locations for multi-span errors

2. **Implement in lexer error tokens**
3. **Implement in parser errors**

### Definition of Done
- [ ] Diagnostic structure defined
- [ ] At least lexer errors support new structure
- [ ] IDE fix actions work for common errors

---

### 4.4 Parser Configuration Options

**Purpose:** Allow configuring parser behavior for DoS protection and specific use cases.

**Current Progress:** Not implemented. Mentioned in planning docs.

**Priority: LOW**

#### Potential Options
- `max_selection_depth: Option<usize>` — Limit nesting depth
- `max_string_literal_size: Option<usize>` — Limit string length
- `max_list_literal_size: Option<usize>` — Limit list elements
- `max_input_object_fields: Option<usize>` — Limit object fields

### Definition of Done
- [ ] `GraphQLParserOptions` struct exists
- [ ] Parser respects configured limits
- [ ] Errors clearly explain when limits exceeded

---

### 4.5 Improved Dot-Pattern Error Messages

**Purpose:** When encountering patterns like `{Name}.{Name}`, provide helpful error suggesting the user may have meant something else.

**Current Progress:** Not implemented. TODO exists in lexer.

**Priority: LOW**

#### Code Reference
- `str_to_graphql_token_source.rs:443`: "TODO: Look for patterns like `{Name}.{Name}` and give a useful error"

### Definition of Done
- [ ] `foo.bar` pattern detected and helpful error emitted
- [ ] Suggestion mentions GraphQL doesn't have member access syntax

---

### 4.6 Clone Overhead Reduction

**Purpose:** Reduce unnecessary string clones in parser hot paths.

**Current Progress:** `Cow<'src, str>` already used. Some clones remain for AST compatibility.

**Priority: LOW**

#### Code References
- `graphql_parser.rs:407`: "TODO: Reduce clone overhead"
- `graphql_parser.rs:438`: "TODO: See docblock above about eliminating this clone"
- `graphql_parser.rs:944`: "TODO: Consider if we can eliminate this clone"

### Definition of Done
- [ ] Profiling identifies clone hot spots
- [ ] Clones reduced where possible without custom AST
- [ ] (Full fix likely requires custom AST)

---

### 4.7 Rustc Nightly Toolchain Warning

**Purpose:** Warn users when `byte_offset` values from `RustMacroGraphQLTokenSource` may be inaccurate due to Rust toolchain limitations.

**Background:** `proc_macro::Span::byte_range()` only returns accurate values on Rust nightly toolchains. Stable rustc does not provide meaningful byte offsets. This affects error reporting and source snippet display when using the `graphql_schema!` macro.

**Crate:** `libgraphql-macros` (not `libgraphql-parser`)

**Priority: LOW**

#### Approach

Use `rustc_version` as a build dependency to detect the toolchain and emit a compile-time warning or cfg flag.

```rust
// In libgraphql-macros/build.rs:
// Use rustc_version::version_meta() to detect nightly
// Emit "cargo:rustc-cfg=libgraphql_rustc_nightly" when on nightly
// In code, conditionally emit warnings about byte_offset accuracy
```

### Definition of Done
- [ ] `build.rs` in `libgraphql-macros` detects Rust toolchain version
- [ ] Warning emitted when using stable toolchain with `graphql_schema!` macro
- [ ] Warning clearly explains that byte offsets may be inaccurate
- [ ] Documented in crate-level rustdoc

---

## Section 5: libgraphql-core Integration

### 5.1 Feature Flag Wiring

**Purpose:** Allow `libgraphql-core` to use `libgraphql-parser` instead of `graphql_parser` crate via feature flag.

**Current Progress:** Feature flag defined in Cargo.toml (`use-libgraphql-parser`). The `ast` module re-export is conditionally wired (uses `libgraphql-parser::ast` when feature enabled). Builders (SchemaBuilder, QueryBuilder, etc.) are NOT yet wired to use the new parser.

**Priority: MEDIUM (enables gradual migration)**

#### Tasks

1. **Update SchemaBuilder**
   - Add `#[cfg(feature = "use-libgraphql-parser")]` branch
   - Use `StrGraphQLTokenSource` and `GraphQLParser`
   - Convert `ParseResult` errors to `SchemaBuildError`

2. **Update QueryBuilder/MutationBuilder/SubscriptionBuilder**
   - Same pattern as SchemaBuilder
   - Use `parse_executable_document()`

3. **Add MixedDocumentBuilder** (new API, feature-gated)

4. **CI configuration**
   - Test with and without feature flag
   - Ensure both paths pass

### Definition of Done
- [ ] SchemaBuilder uses new parser when feature enabled
- [ ] Operation builders use new parser when feature enabled
- [ ] MixedDocumentBuilder API exists (feature-gated)
- [ ] CI tests both feature configurations
- [ ] All existing tests pass with both configurations

---

### 5.2 ast Module Consolidation

**Purpose:** After feature flag migration, consolidate the `ast` module to avoid duplication.

**Current Progress:** `ast.rs` exists in both `libgraphql-core` and `libgraphql-parser`. This is intentional during transition.

**Priority: LOW (after feature flag is default)**

#### Tasks

1. **Make `use-libgraphql-parser` the default**
2. **Remove `ast.rs` from `libgraphql-core`**
3. **Re-export from `libgraphql-parser`**
4. **Remove feature flag (becomes unconditional)**

### Definition of Done
- [ ] Single source of truth for `ast` module
- [ ] Feature flag removed
- [ ] `graphql_parser` crate dependency removed from `libgraphql-core`

---

## Section 6: Documentation

### 6.1 Crate README

**Purpose:** Provide usage examples and overview for crate users.

**Current Progress:** No README exists in `crates/libgraphql-parser/`.

**Priority: MEDIUM**

#### Tasks

1. **Create `crates/libgraphql-parser/README.md`**
   - Quick start examples
   - API overview
   - Error handling patterns
   - Feature flags documentation
   - Fuzz testing instructions (see Section 3.1)

### Definition of Done
- [ ] README.md exists with examples
- [ ] Examples compile and are tested (doc-tests or examples/)
- [ ] Fuzz testing instructions included

---

## Appendix: Code TODOs

TODOs found in the codebase (for reference):

| File | Line | TODO |
|------|------|------|
| `graphql_token_kind.rs` | 111 | Explore richer diagnostics structure |
| `graphql_token_stream.rs` | 35 | Future TODOs for configuration options |
| `str_to_graphql_token_source.rs` | 443 | Detect `{Name}.{Name}` patterns |
| `graphql_parser.rs` | 407 | Reduce clone overhead |
| `graphql_parser.rs` | 438 | Eliminate clone |
| `graphql_parser.rs` | 563 | Test expect_keyword("true") |
| `graphql_parser.rs` | 611 | Test peek_is_keyword("true") |
| `graphql_parser.rs` | 944 | Consider eliminating clone |
| `graphql_parser.rs` | 1725 | Track directives with custom AST |
| `graphql_parser.rs` | 2524 | Support schema extensions |
| `graphql_parser.rs` | 2531 | Support schema extensions |
| `graphql_parser.rs` | 2582 | Support schema extensions |

---

## Priority Summary

**HIGH Priority:**
- Fuzz testing (Section 3.1) — security-critical
- Coverage-driven test discovery (Section 2.1) — find important untested paths
- External test suite gap analysis (Section 2.2) — comprehensive coverage

**MEDIUM Priority:**
- Vendored documents project (Section 1) — enables benchmarks and integration tests
- RustMacroGraphQLTokenSource tests (Section 2.3)
- Feature flag wiring (Section 5.1)
- Crate README (Section 6.1)
- Integration tests with real schemas (Section 2.5)

**LOW Priority:**
- Differential tests (Section 2.6)
- Performance benchmarks (Section 3.2)
- Schema extension support (Section 4.1)
- Custom AST (Section 4.2)
- All other Section 4 items
- ast module consolidation (Section 5.2)

---

## Past Completed Work

*Items moved here when wholly completed. Each entry includes a simple title and terse description.*

### Core Parser Implementation (pre-plans.md)

Completed before this document was created:

- **StrGraphQLTokenSource lexer** (~1130 lines) — zero-copy lexer with `Cow<'src, str>`
- **GraphQLParser recursive descent** (~3200 lines) — generic over token source
- **ParseResult API** — partial AST + errors for IDE-friendly parsing
- **Error infrastructure** — `GraphQLParseError`, `GraphQLParseErrorKind`, `GraphQLErrorNote`
- **All GraphQL constructs** — values, types, directives, operations, fragments, type definitions, extensions
- **383 unit tests + 4 doc-tests** — comprehensive test coverage for core functionality
