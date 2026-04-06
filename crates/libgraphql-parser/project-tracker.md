# libgraphql-parser — Project Tracker & Remaining Work

**Last Updated:** 2026-03-03

This document consolidates all remaining work for the `libgraphql-parser` crate.
It supersedes any individual tracking documents under the root `/project-tracker.md` document.

## Document Maintenance Notes

When updating this document:

1. **Completed items:** Move wholly-completed plan items to the "Past Completed Work" section at the end of this document. Include a simple title and terse description only.
2. **Plan identifiers:** NEVER re-number existing plan items (e.g., 4.3, 2.1). This ensures references to plan IDs remain valid over time.
3. **Partial completion:** If a plan item is partially done, leave it in place and update its description to reflect remaining work.

---

## Current State Summary

**Test Status:** 1,077+ tests passing, 9 doc-tests passing (1 ignored)

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
- Two real-world schemas vendored as benchmark fixtures in `benches/fixtures/third-party/`:
  - `github-schema/` — GitHub GraphQL API (~1.2MB, MIT, from `octokit/graphql-schema`)
  - `starwars-schema/` — Star Wars example (~4KB, MIT, from `apollographql/apollo-tooling`)
- Each has a sibling LICENSE file with full MIT text
- Integrated into `schema_parse`, `lexer`, and `compare_schema_parse` benchmark groups
- Central `vendored/` directory (for broader workspace use) not yet created

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

**Purpose:** Analyze test suites from `graphql-js`, `graphql-parser`, and `apollo-parser` to identify test scenarios we don't currently cover, then write our own tests for those gaps.

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

4. **Analyze apollo-parser tests**
   - Source: `apollo-rs/crates/apollo-parser/src/tests/` ([apollo-rs](https://github.com/apollographql/apollo-rs/tree/main/crates/apollo-parser))
   - Focus on: error recovery, partial parse results, diagnostic quality, IDE-oriented edge cases
   - `apollo-parser` is particularly strong in error-recovery testing; pay close attention to scenarios where it tests graceful degradation on malformed input
   - Document gaps and write our own tests

5. **Create gap analysis document**
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
- Error recovery and diagnostics: partial parse behavior on malformed input (especially from `apollo-parser` analysis)

### Definition of Done
- [ ] graphql-js lexer tests analyzed, gaps identified and filled
- [ ] graphql-js parser tests analyzed, gaps identified and filled
- [ ] graphql-parser tests analyzed, gaps identified and filled
- [ ] apollo-parser tests analyzed, gaps identified and filled
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

### 2.7 apollo-parser Test Suite Audit

**Purpose:** Audit the full test suite of the `apollo-parser` crate (from [apollo-rs](https://github.com/apollographql/apollo-rs/tree/main/crates/apollo-parser)) to identify any test scenarios present there that are missing or anemic in `libgraphql-parser`, then implement equivalent tests.

**Approach:** Review every test in `apollo-parser`'s test suite, compare against `libgraphql-parser`'s existing tests, and write our own implementations for any gaps found. As with Section 2.2, we write our own test code (not vendored) to stay consistent with project conventions and avoid license complexity.

**Current Progress:** Not started.

**Priority: HIGH**

#### Tasks

1. **Catalogue apollo-parser test files**
   - Source: `apollo-rs/crates/apollo-parser/src/tests/`
   - Identify all test modules and their focus areas (lexer, parser, error recovery, specific GraphQL constructs, etc.)

2. **Audit lexer/tokenizer tests**
   - Compare token-level tests against `libgraphql-parser`'s `token/tests/`
   - Pay special attention to: punctuator handling, string/block-string edge cases, numeric literal edge cases, comment handling, Unicode edge cases

3. **Audit parser tests**
   - Compare parser tests against `libgraphql-parser`'s `tests/graphql_parser_tests.rs`
   - Focus areas: type definitions, operations, fragments, directives, descriptions, extensions, error recovery, partial parse results

4. **Audit error recovery and diagnostics tests**
   - `apollo-parser` has strong error-recovery testing (it's also IDE-oriented)
   - Compare error messages, error spans, and recovery behavior
   - Identify any error-recovery scenarios `libgraphql-parser` doesn't test

5. **Implement missing tests**
   - Write `libgraphql-parser` tests for every identified gap
   - Follow CLAUDE.md test conventions (description comments, spec links, "Written by Claude Code, reviewed by a human" attribution where applicable)

6. **Document audit results**
   - Track which `apollo-parser` test areas were reviewed
   - Document which gaps were found and filled
   - Note any intentional differences (e.g., spec interpretation, strictness)

### Definition of Done
- [ ] All `apollo-parser` test modules catalogued and reviewed
- [ ] Lexer/tokenizer test gaps identified and filled
- [ ] Parser test gaps identified and filled
- [ ] Error recovery test gaps identified and filled
- [ ] Audit results documented (in this file or a linked document)
- [ ] All new tests follow CLAUDE.md conventions
- [ ] End result: every scenario tested by `apollo-parser` is, in one form or another, also tested in `libgraphql-parser`

---

## Section 3: Performance & Fuzzing

### 3.1 Fuzz Testing

**✅ COMPLETE** — Moved to Past Completed Work.

Remaining stretch goal: structured fuzzing with `arbitrary` crate.

---

### 3.2 Performance Benchmarks

**Purpose:** Establish performance baseline and ensure `libgraphql-parser` is competitive with `graphql_parser` and `apollo-parser` crates.

**Current Progress:** Benchmark suite implemented with criterion. 5 benchmark groups covering schema parsing, executable parsing, lexer throughput, and cross-parser comparisons. Uses synthetic `.graphql` fixtures (small ~1.5KB, medium ~106KB, large ~500KB) and vendored real-world schemas (Star Wars ~4KB from `apollographql/apollo-tooling`, GitHub ~1.2MB from `octokit/graphql-schema`) embedded via `include_str!`.

**Priority: HIGH (perf is a design goal)**

**Baseline results (2026-02-08, Apple M2 Max arm64, rustc 1.90.0-nightly):**
- Schema parsing: ~1.7x (small), ~2.4x (medium), ~2.5x (large) vs `graphql_parser`
- Executable parsing: **0.63x faster** (simple), ~1.5x (complex) vs `graphql_parser`
- Lexer throughput: ~73-78 MiB/s (consistent across input sizes)

**Post-SourcePosition-shrink results (2026-02-23):**
- Full-fidelity: 14-29% improvement across all benchmarks (e.g., github schema 25.3→20.2ms, shopify 45.1→36.4ms)
- Lean mode: 11-20% improvement (e.g., github lean 12.5→11.0ms)
- Lexer: 9-18% improvement (cache locality from smaller tokens)
- Lean-mode benchmark groups (`schema_parse_lean`, `executable_parse_lean`) added

#### Tasks

1. **Create benchmark suite** ✅
   - Location: `/crates/libgraphql-parser/benches/`
   - `criterion` crate, single `parse_benchmarks` target, 5 groups

2. **Benchmark scenarios** ✅
   - Small/medium/large schemas, simple/complex queries, nested depths, multi-operation docs

3. **Compare against graphql_parser crate** ✅
   - Target: within 2x of `graphql_parser` performance
   - Executable: within target. Schema: medium/large slightly exceed 2x.

4. **Compare against apollo-parser crate** ✅
   - Target: within 2x of `apollo_parser` performance
   - Executable: within target. Schema: medium/large slightly exceed 2x.

5. **Identify optimization opportunities**
   - Profile hot paths
   - Consider `memchr` for fast character scanning
   - Review allocation patterns

6. **Add vendored real-world schema benchmarks** ✅
   - GitHub (~1.2MB) and Star Wars (~4KB) schemas added to benchmark groups

### Definition of Done
- [x] Benchmark suite exists with `criterion`
- [x] At least 3 benchmark scenarios using vendored schemas (GitHub + Star Wars in schema_parse, lexer, compare_schema_parse = 6 scenarios)
- [ ] Performance within 2x of `graphql_parser` (executable ✅, schema needs optimization)
- [ ] Performance within 2x of `apollo_parser` (executable ✅, schema needs optimization)

---

## Section 4: Future Enhancements

### 4.1 Schema Extension Support

**✅ COMPLETE** — Moved to Past Completed Work.

---

### 4.2 Custom AST

**Purpose:** Replace `graphql_parser` crate AST types with custom types with spans, schema extension support, trivia attachment, and serde support.

**Current Progress:** Custom AST substantially implemented. See `custom-ast-plan.md` for full design.

- **Phase 1 (Core AST Types):** ✅ COMPLETE — 42 typed structs in `ast/` module, zero-copy `Cow<'src, str>`, `GraphQLSourceSpan` on every node, optional `*Syntax` layer
- **Phase 2 (Compat Layer):** ✅ COMPLETE — Bidirectional conversion between custom AST and `graphql_parser` v0.4 types in `parser_compat/graphql_parser_v0_4/`
- **Phase 3 (Parser Integration):** ✅ COMPLETE — Parser produces custom AST types. Convenience wrappers `parse_schema()`/`parse_query()`/`parse_mixed()` added
- **Phase 3b (Ground-Truth Tests):** ✅ COMPLETE — Compat output verified against `graphql_parser` crate's own parse results
- **Phase 4a (Lexer Trivia Config):** ✅ COMPLETE — `StrGraphQLTokenSourceConfig` with per-type trivia flags, `Whitespace` trivia variant
- **Phase 4b (Parser Syntax Config):** ✅ COMPLETE — `GraphQLParserConfig` with `retain_syntax` flag
- **Phase 4c (Syntax Population):** ✅ COMPLETE — All 42 `*Syntax` structs populated when `retain_syntax = true`
- **Phase 4d (Test Migration):** ✅ COMPLETE — 11 test files migrated to new AST types, 5 config-flag tests, 8 whitespace trivia tests, ~80 expanded position assertions. 672 tests pass

**Remaining work (Phase 4e and beyond):**

**Priority: MEDIUM**

#### Tasks

1. ~~**Phase 4d: Test Migration**~~ ✅ COMPLETE
2. **Phase 4e: Source Reconstruction** — Implement synthetic-formatting mode for `append_source`, round-trip tests, benchmark lean vs full-fidelity. Note: PBT synthetic round-trip tests (`to_source(None)`) were removed since `append_source` currently no-ops when `source` is `None`; re-add them after implementing synthetic reconstruction.
3. **Phase 5: Downstream Migration** — Update `libgraphql-macros` and `libgraphql-core` to use new parser + compat layer
4. **Phase 6: apollo-parser compat** — `to_apollo_parser_cst()` / `from_apollo_parser_cst()`
5. **Phase 7: FFI Layer** — C API, OwnedDocument, auto-generated headers
6. **Phase 8: Cleanup** — Remove `legacy_ast`, make `graphql-parser` dep optional, feature-gate compat modules
7. **Serde support** — Serialization for complete AST

### Definition of Done
- [x] Syntax tree types defined for all GraphQL constructs
- [x] All nodes have span information
- [x] `libgraphql` → `graphql-parser` forward translation implemented
- [x] Reverse translation from `graphql-parser` implemented
- [x] Parser produces new syntax tree
- [ ] `apollo-parser` translation utilities implemented
- [ ] `libgraphql-macros` updated to use new AST directly (currently uses compat layer)
- [ ] Serde serialization works
- [ ] `graphql_parser` dep removed / feature-gated

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

**✅ COMPLETE** — Moved to Past Completed Work.

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

### 4.8 Spec-Version Feature Flags (Zero Runtime Cost)

**Purpose:** Investigate and implement feature-flag-based spec-version selection so that enabling a specific feature flag (e.g., `spec-sep-25`, `spec-oct-21`) compiles `libgraphql-parser` as a parser that strictly adheres to the grammar and constraints of that specification edition — with zero runtime cost.

**Current Progress:** Not started. Investigation required.

**Priority: LOW**

#### Background

The GraphQL specification has evolved across multiple editions. Some users need strict adherence to a specific edition (e.g., for compliance or compatibility reasons). The goal is a compile-time mechanism where the user chooses a spec version and `libgraphql-parser` compiles with only that version's grammar and validation rules — no runtime branching.

The latest spec version should be the default (i.e., when no spec-version feature flag is explicitly enabled). Coverage should go back to approximately **October 2016**.

Known spec editions to consider:
- October 2016
- June 2018
- October 2021
- September 2025 (default)

#### Key Investigation Questions

1. **Cargo feature additivity:** Cargo features are additive by design — enabling a feature should add functionality, not remove it. Spec-version selection needs *mutually exclusive* behavior (e.g., `spec-oct-21` should reject Sep 2025 syntax). How do we reconcile this? Options include:
   - Mutually exclusive features with compile-time assertions (e.g., `compile_error!` if multiple spec features enabled)
   - A single `spec-version` cfg value set via build script rather than features
   - Other patterns used in the Rust ecosystem for this problem
2. **Granularity of spec differences:** What actually changes between spec editions at the parser/lexer level? Are differences limited to a small number of grammar rules, or are they pervasive?
3. **Implementation mechanism:** `#[cfg(feature = "...")]` on individual parser branches? Const generics? Trait-based strategy pattern resolved at compile time?
4. **Testing matrix:** How do we test all supported spec versions in CI without combinatorial explosion?

#### Tasks

1. **Catalogue spec-edition grammar differences**
   - Diff each consecutive pair of spec editions (Oct 2016 → Jun 2018 → Oct 2021 → Sep 2025)
   - Identify which grammar rules, keywords, and constructs changed
   - Assess scope: small number of targeted `#[cfg]` branches vs. pervasive changes

2. **Design feature-flag mechanism**
   - Propose a concrete approach that achieves zero runtime cost and mutual exclusivity
   - Prototype with one small spec difference to validate the approach
   - Document trade-offs and any Cargo-ecosystem limitations encountered

3. **Implement spec-version feature flags**
   - Wire up `#[cfg]` (or chosen mechanism) for each spec-edition difference
   - Default to latest spec when no flag is specified
   - Emit `compile_error!` (or equivalent) if conflicting flags are enabled

4. **Update test infrastructure**
   - Ensure tests can run against each spec version
   - Add spec-version-specific tests for grammar differences
   - CI configuration to test all supported versions

5. **Update README banner for multi-spec support**
   - The SVG banner (`assets/banner-gradient-v4-slate.v10.svg` or whichever becomes final) currently shows "September 2025 Specification"
   - When multi-spec ships, update to list supported specs with `·` separator (e.g., `Sep 2025 · Oct 2021`)
   - See the HTML comment in the SVG file for exact instructions
   - Also update any spec-version references in the README body text

### Definition of Done
- [ ] Spec-edition grammar differences catalogued
- [ ] Feature-flag mechanism designed and prototyped
- [ ] All supported spec versions (Oct 2016 through Sep 2025) compile and parse correctly
- [ ] Latest spec is the default when no flag is specified
- [ ] Conflicting flags produce a clear compile-time error
- [ ] Zero runtime cost verified (no runtime branching for spec-version logic)
- [ ] CI tests all supported spec versions
- [ ] README banner and body updated to reflect multi-spec support

---

### 4.9 ByteSpan + SourceMap Span Optimization

**Purpose:** Replace per-node `GraphQLSourceSpan` (~104 bytes) with a compact `ByteSpan` (8 bytes) + shared `SourceMap` for deferred line/col resolution. ~13x per-span memory reduction, better cache density, eliminates per-token PathBuf clone.

**Current Progress:** ALL PHASES COMPLETE (Mar 4-6, 2026). ByteSpan (8 bytes, Copy) replaces SourceSpan across all tokens, AST nodes, parser, errors, and proc-macro token source. Lexer throughput 2-3x improvement vs Feb 14 baseline. Schema parse 5-17% faster. Full plan in `/sourcemap-bytespan-implementation-plan.md`.

**Priority: ✅ COMPLETE**

**Depends on:** Custom AST (4.2) — was prerequisite, now satisfied.

#### Key Questions

Key questions from original plan have been answered:
- Pre-pass strategy successfully avoids hot-path regression
- SourceMap dual-mode design handles both source-text and pre-computed column modes
- API complexity (SourceMap threading) is manageable — ParseResult bundles it

### Definition of Done
- [x] Profiling data collected from working custom AST
- [x] Decision made: pursue or abandon based on data
- [x] ByteSpan, SourceMap, `into_source_map()` implemented per Section 14 design (Phases 1-5 complete)
- [x] RustMacroGraphQLTokenSource migrated to ByteSpan w/ synthetic offsets (Phase 6)
- [x] Cleanup: removed `to_byte_span()` bridge, `large_enum_variant` allows, stale TODOs (Phase 7)
- [x] Final benchmarks: lexer 2-3x faster, schema parse 5-17% faster vs Feb 14 baseline (Phase 7)
- [x] All 1,077 tests pass, clippy clean

---

### 4.5 AST Visitor Trait

**Purpose:** Standard visitor trait for traversing parsed AST nodes, enabling custom linting, code generation, schema introspection, formatting, and migration tooling without manual destructuring.

**Priority: MEDIUM**

**Context:** Identified during `libgraphql-core-v1` Task 12 (type builders). Each builder's `from_ast()` manually destructures one AST node level — a visitor wouldn't help there, but broader tooling use cases (multi-pass analysis, custom linters) would benefit from a centralized traversal API.

#### Tasks
1. Design visitor trait with `visit_*` methods for each AST node type
2. Implement default traversal (walk functions)
3. Consider mutable visitor variant for AST transformations

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

**Current Progress:** README.md created with API overview, usage examples, and fuzz testing instructions/results. Still needs error handling patterns and feature flags documentation.

**Priority: MEDIUM**

#### Tasks

1. **Create `crates/libgraphql-parser/README.md`**
   - Quick start examples
   - API overview
   - Error handling patterns
   - Feature flags documentation
   - Fuzz testing instructions (see Section 3.1)

### Definition of Done
- [x] README.md exists with examples
- [ ] Examples compile and are tested (doc-tests or examples/)
- [x] Fuzz testing instructions included

---

## Post-Parser-Integration TODOs

### Unit Tests

- **ScalarTypeExtension.directives non-empty** — Verify parser never produces empty `directives` (spec requires at least one)
- **SchemaDefinition.root_operations non-empty** — Verify never empty (spec: RootOperationTypeDefinition+)
- **Extension directives-only forms** — Verify Union/Enum/InputObject TypeExtension directives-only forms have non-empty `directives`
- **IntValue overflow/underflow** — Verify parser clamps to i32::MAX/MIN, emits diagnostic
- **FloatValue edge cases** — Verify scientific notation, negatives, overflow to INFINITY

### Revisit

- ~~**large_enum_variant allows**~~ — RESOLVED: removed all 4 allows (`Definition`, `Nullability`, `TypeAnnotation`, `Selection`), clippy clean
- ~~**`byte_span` redundancy on `GraphQLParseError`**~~ — RESOLVED: removed `byte_span` field entirely; all consumers now use `source_span` exclusively

---

## Appendix: Code TODOs

TODOs found in the codebase (auto-generated 2026-02-24):

| File                             | Line | TODO                                              |
|----------------------------------|------|---------------------------------------------------|
| `ast/mod.rs`                     |   22 | Update example once parser advances (Phase 3)     |
| `ast/nullability.rs`             |   14 | ✅ `large_enum_variant` allow removed |
| `ast/type_annotation.rs`         |   15 | ✅ `large_enum_variant` allow removed |
| `graphql_parser.rs`              |  661 | Test expect_keyword("true") behavior              |
| `graphql_parser.rs`              |  721 | Test peek_is_keyword("true") behavior             |
| `graphql_token_kind.rs`          |  111 | Explore richer diagnostics structure              |
| `str_to_graphql_token_source.rs` |  669 | Detect `{Name}.{Name}` patterns for better error |

---

## Priority Summary

**HIGH Priority:**
- ~~Fuzz testing (Section 3.1) — ✅ COMPLETE~~
- Performance benchmarks (Section 3.2)
- Spec-version feature flags (Section 4.8)
- Coverage-driven test discovery (Section 2.1) — find important untested paths
- External test suite gap analysis (Section 2.2) — comprehensive coverage
- apollo-parser test suite audit (Section 2.7) — parity with apollo-rs test coverage

**MEDIUM Priority:**
- Custom AST remaining phases (Section 4.2) — Phase 4e (source reconstruction), Phase 5 (downstream migration)
- ~~ByteSpan + SourceMap optimization (Section 4.9)~~ — ✅ ALL PHASES COMPLETE
- Vendored documents project (Section 1) — enables benchmarks and integration tests
- RustMacroGraphQLTokenSource tests (Section 2.3)
- Feature flag wiring (Section 5.1)
- Crate README (Section 6.1)
- Integration tests with real schemas (Section 2.5)

**LOW Priority:**
- Differential tests (Section 2.6)
- ~~Schema extension support (Section 4.1) — ✅ COMPLETE~~
- ~~Clone overhead reduction (Section 4.6) — ✅ COMPLETE~~
- All other Section 4 items
- ast module consolidation (Section 5.2)

---

## Past Completed Work

*Items moved here when wholly completed. Each entry includes a simple title and terse description.*

### Core Parser Implementation (pre-project-tracker.md)

Completed before this document was created:

- **StrGraphQLTokenSource lexer** (~1130 lines) — zero-copy lexer with `Cow<'src, str>`
- **GraphQLParser recursive descent** (~3200 lines) — generic over token source
- **ParseResult API** — partial AST + errors for IDE-friendly parsing
- **Error infrastructure** — `GraphQLParseError`, `GraphQLParseErrorKind`, `GraphQLErrorNote`
- **All GraphQL constructs** — values, types, directives, operations, fragments, type definitions, extensions
- **383 unit tests + 4 doc-tests** — comprehensive test coverage for core functionality

### Schema Extension Support (Section 4.1) — completed 2026-02-20

`parse_schema_extension()` implemented as part of custom AST Phase 3 Task 8. `extend schema` with directives and root operations now parses correctly.

### Clone Overhead Reduction (Section 4.6) — completed 2026-02-23

All clone-related TODOs in `graphql_parser.rs` eliminated. `SourcePosition` derived `Copy` (removing ~50 `.clone()` calls). Custom AST Phase 3 moved spans by value instead of cloning.

### Custom AST Phases 1-4c — completed 2026-02-22

Phase 1: 42 typed AST structs in `ast/` module, zero-copy `Cow<'src, str>`, `GraphQLSourceSpan` on every node, optional `*Syntax` layer. Phase 2: Bidirectional compat layer with `graphql_parser` v0.4. Phase 3: Parser produces custom AST; tests pass via compat bridge. Phase 3b: Ground-truth comparison tests. Phase 4a: Lexer trivia config (`GraphQLTokenSourceConfig`). Phase 4b: Parser syntax config (`GraphQLParserConfig`). Phase 4c: All 42 `*Syntax` structs populated when `retain_syntax = true`.

### Custom AST Phase 4d: Test Migration — completed 2026-03-03

11 test files migrated from legacy `graphql_parser` types to native `crate::ast` types. Parse helpers (`utils.rs`) and extraction helpers (`ast_utils.rs`) rewritten. 5 new `GraphQLParserConfig` tests (`retain_syntax` flag). 8 new whitespace/trivia tests. ~80 expanded position assertions (`byte_offset`, `end_exclusive`, sub-node spans) across 14 existing position tests. Total: 672 tests passing.

### SourcePosition Shrink + Copy Optimization — completed 2026-02-23

Shrank `SourcePosition` from 40→20 bytes (`usize`→`u32`, derived `Copy`). Removed ~50 unnecessary `.clone()` calls across workspace. Added lean-mode benchmarks. 14-29% improvement on full-fidelity parsing, 9-18% on lexer throughput.

### Fuzz Testing (Section 3.1) — completed 2026-01-30

- `cargo-fuzz` infrastructure with 4 targets: lexer, schema parser, executable parser, mixed parser
- Seed corpus with 10 hand-crafted `.graphql` files
- Parallel runner script (`scripts/run-fuzz-tests.sh`, bash 3.2 compatible for macos)
- 10 bugs found and fixed: 7 infinite-loop/OOM, 1 stack overflow (recursion depth guard), 2 block string UTF-8 panics
- 15-min sustained run per target (25.5M total executions), zero crashes
- Results documented in `crates/libgraphql-parser/README.md`
