# libgraphql-parser — Consolidated Plans & Remaining Work

**Last Updated:** 2026-01-30

This document consolidates all remaining work for the `libgraphql-parser` crate.
It supersedes any individual planning documents under the root `/plans.md` document.

## Document Maintenance Notes

When updating this document:

1. **Completed items:** Move wholly-completed plan items to the "Past Completed Work" section at the end of this document. Include a simple title and terse description only.
2. **Plan identifiers:** NEVER re-number existing plan items (e.g., 4.3, 2.1). This ensures references to plan IDs remain valid over time.
3. **Partial completion:** If a plan item is partially done, leave it in place and update its description to reflect remaining work.

---

## Current State Summary

**Test Status:** 443 tests passing, 4 doc-tests passing

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

**Superset Requirement:** Whatever syntax tree structure we adopt, it must contain at least a superset of the information in *both* the `graphql-parser` and `apollo-parser` structures. This ensures lossless forward-translation to either format.

**C/C++ Bindings Constraint:** Whatever syntax tree structure we choose must work well as a foundation for C-bindings that we eventually publish for calling `libgraphql-parser` from C and C++ code. This means the AST should be amenable to representation across an FFI boundary — e.g., avoiding deeply generic types, favoring layouts that map naturally to C structs/tagged unions, and keeping ownership semantics straightforward enough to expose via opaque pointers or value types.

**Translation Utilities:** Forward-translation facilities are needed for at least:
- `libgraphql` syntax tree → `graphql-parser` AST (for compatibility with `graphql-parser`-based tools)

If we design our own syntax tree (rather than adopting `apollo-parser`'s CST), we additionally need:
- `libgraphql` syntax tree → `apollo-parser` CST (for compatibility with `apollo-rs`-based tools)

Reverse translations (external → `libgraphql`) will necessarily lack some information, but should also be provided wherever reasonably implementable and useful — even if lossy.

#### Tasks

1. **Evaluate external syntax tree structures**
   - Catalogue all types and fields in `graphql-parser` (`graphql_parser::query` module)
   - Catalogue all types and fields in `apollo-parser` (`apollo-parser`'s CST)
   - Produce a comparison document identifying: shared fields, fields unique to each, and information unique to neither that `libgraphql` should add (e.g., spans, trivia)

2. **Decide: adopt apollo-parser's CST vs. design a new syntax tree**
   - Weigh pros, cons, and trade-offs of adopting `apollo-parser`'s existing CST structure (lossless, IDE-friendly, already proven) vs. designing a new structure (more control, potentially simpler API, tailored to `libgraphql`'s needs)
   - Consider hybrid approaches (e.g., adopting `apollo-parser`'s CST model with modifications)
   - Key factors: API ergonomics, compatibility burden, maintenance cost, information preservation, downstream consumer needs, parser performance implications (e.g., allocation patterns, node granularity), configurability (how easily the structure accommodates parser options, spec-version variations, etc.), and FFI suitability (how naturally the structure can be exposed via C-bindings — see C/C++ Bindings Constraint above)
   - This decision gates subsequent tasks: if we adopt Apollo's CST, `apollo-parser` translation utilities are unnecessary; if we design our own (or a variation), they become required

3. **Design/adopt syntax tree types** (informed by task 2)
   - All nodes include `span: GraphQLSourceSpan`
   - Consider `Arc` for sharing in IDE scenarios
   - Ensure all fields from both `graphql-parser` and `apollo-parser` are represented

4. **Implement forward translation to graphql-parser AST**
   - `impl From<LibGraphQLAst> for graphql_parser::Ast` (or similar)
   - Translation must be lossless for `graphql-parser` fields

5. **Implement apollo-parser translation utilities** (only if task 2 chose a non-Apollo structure)
   - Forward: `libgraphql` → `apollo-parser` types (lossless for `apollo-parser` fields)
   - Evaluate feasibility of reverse: `apollo-parser` CST → `libgraphql`; implement if tenable — even if imperfect but still useful
   - If `apollo-parser`'s CST model makes translation impractical for certain constructs, document the limitation and provide the closest reasonable approximation

6. **Evaluate and implement reverse translation from graphql-parser**
   - `graphql-parser` AST → `libgraphql` (lossy: will lack spans, trivia, etc.)
   - Implement if reasonably useful; clearly document any information loss

7. **Update parser to produce new syntax tree**
8. **Update libgraphql-macros for new syntax tree**
   - `libgraphql-macros` depends on `libgraphql-parser`'s syntax tree types; any change will break it
   - Update `RustMacroGraphQLTokenSource` integration and all macro-generated code to use the new types
9. **Add serde support**
10. **Deprecate and remove `graphql_parser` re-exports**

#### Code References (TODOs in codebase)
- `graphql_parser.rs:1725`: "Track these when we have a custom AST"
- Multiple references to custom AST in planning docs

### Definition of Done
- [ ] External syntax tree evaluation document produced
- [ ] Adopt-vs-invent decision made and rationale documented
- [ ] Syntax tree types defined for all GraphQL constructs
- [ ] Syntax tree is a superset of both `graphql-parser` and `apollo-parser` information
- [ ] `libgraphql` → `graphql-parser` forward translation implemented
- [ ] `apollo-parser` translation utilities implemented if applicable (or adoption documented)
- [ ] Reverse translations implemented where feasible, with information loss documented
- [ ] Parser produces new syntax tree
- [ ] `libgraphql-macros` updated and passing all tests with new syntax tree
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

### Definition of Done
- [ ] Spec-edition grammar differences catalogued
- [ ] Feature-flag mechanism designed and prototyped
- [ ] All supported spec versions (Oct 2016 through Sep 2025) compile and parse correctly
- [ ] Latest spec is the default when no flag is specified
- [ ] Conflicting flags produce a clear compile-time error
- [ ] Zero runtime cost verified (no runtime branching for spec-version logic)
- [ ] CI tests all supported spec versions

---

### 4.9 C/C++ API Bindings (`libgraphql-parser.h`)

**Purpose:** Expose `libgraphql-parser`'s full public API — parser entry points, AST structures, error infrastructure, token/trivia types, and source positions — as a canonical, idiomatic C library with a C++ convenience header, enabling non-Rust consumers (C, C++, Python/ctypes, Swift, etc.) to lex, parse, and inspect GraphQL documents.

**Current Progress:** Not started. Design constraints captured in Section 4.2.

**Priority: LOW (depends on Custom AST — Section 4.2)**

**Hard Dependency:** Section 4.2 (Custom AST) must be substantially complete first. The current AST is re-exported from `graphql-parser` 0.4 and uses generic lifetime parameters (`<'static, String>`), `Vec`, `Option`, `Box`, nested enums, and other types that do not map cleanly to C structs. The custom AST (4.2) is specifically constrained to be "amenable to representation across an FFI boundary." Designing the C API against the current transient AST would create massive throwaway work.

#### 4.9.0 Scope & Guiding Principles

1. **Full coverage.** Every public type, enum, and function in `libgraphql-parser` gets a C counterpart. No "partial" binding that forces users to reach around the C API.
2. **Opaque-handle ownership model.** All heap-allocated objects are represented as opaque `typedef struct ... *` handles. The C user never sees struct layouts; only accessor functions. This allows the Rust side to evolve internals freely.
3. **Naming convention:** `graphql_<noun>_<verb>` (e.g., `graphql_parser_new`, `graphql_parse_result_get_ast`, `graphql_source_span_start_line`). All symbols prefixed with `graphql_` to avoid collisions.
4. **Error signalling:** Functions that can fail return a status enum (`GraphQLStatus`) or a nullable handle. No C exceptions, no `setjmp`/`longjmp`.
5. **No global/thread-local state.** All state is carried in explicit handles. Library is fully reentrant.
6. **C99 baseline.** Header is valid C99 (with `stdint.h`, `stdbool.h`). A separate `libgraphql_parser.hpp` C++ header adds RAII wrappers, `std::string_view` accessors, and range-based iteration.
7. **Single shared/static library artifact.** Build produces `libgraphql_parser.so`/`.dylib`/`.dll` (shared) and `libgraphql_parser.a` (static) via `cbindgen` + `crate-type = ["cdylib", "staticlib"]`.
8. **Minimize copies across FFI.** String accessors return `const char *` + `size_t` length pointing into Rust-owned memory (valid for the lifetime of the parent handle). Users who need to outlive the handle must copy.

#### 4.9.1 Crate & Build Infrastructure

**New crate:** `crates/libgraphql-parser-c/`

```
crates/libgraphql-parser-c/
├── Cargo.toml           # cdylib + staticlib, depends on libgraphql-parser
├── cbindgen.toml        # cbindgen configuration
├── build.rs             # (optional) auto-generate header via cbindgen
├── src/
│   ├── lib.rs           # Top-level re-exports, #[no_mangle] entry points
│   ├── handles.rs       # Opaque handle type machinery (Box-to-ptr, ptr-to-ref)
│   ├── status.rs        # GraphQLStatus enum (#[repr(C)])
│   ├── parser_api.rs    # graphql_parser_* functions
│   ├── parse_result_api.rs
│   ├── ast_api.rs       # AST node accessors (schema, operation, mixed)
│   ├── error_api.rs     # GraphQLParseError / GraphQLErrorNote accessors
│   ├── token_api.rs     # Token, TokenKind, TriviaToken accessors
│   ├── source_span_api.rs
│   └── string_api.rs    # GraphQLStringRef (ptr+len for borrowed strings)
├── include/
│   ├── libgraphql_parser.h    # C header (generated or hand-maintained)
│   └── libgraphql_parser.hpp  # C++ convenience header (RAII, iterators)
└── tests/
    ├── c/               # C test programs (compiled + run in CI)
    └── cpp/             # C++ test programs
```

**Cargo.toml essentials:**
```toml
[lib]
crate-type = ["cdylib", "staticlib"]

[dependencies]
libgraphql-parser = { path = "../libgraphql-parser" }

[build-dependencies]
cbindgen = "0.27"       # or latest
```

**Tasks:**
1. Create crate skeleton with `cdylib` + `staticlib` lib targets
2. Configure `cbindgen.toml` (C99, `graphql_` prefix, include guards, documentation passthrough)
3. Wire into workspace `Cargo.toml`
4. CI step: build C library, compile + link + run C/C++ test programs
5. `pkg-config` `.pc` file generation (via build script or install script)

---

#### 4.9.2 Handle & Lifetime Model

All Rust objects exposed to C are boxed and returned as opaque pointers. A small internal macro/trait stamps out the pattern:

```rust
// Conceptual (not literal code):
macro_rules! define_handle {
    ($RustType:ty => $CHandle:ident) => {
        pub type $CHandle = *mut $RustType;

        fn into_handle(val: $RustType) -> $CHandle {
            Box::into_raw(Box::new(val))
        }

        unsafe fn from_handle(h: $CHandle) -> &$RustType { ... }

        unsafe fn free_handle(h: $CHandle) { drop(Box::from_raw(h)); }
    };
}
```

**Opaque handles to define (one per exposed Rust type):**

| Rust Type | C Handle Typedef | Free Function |
|-----------|-----------------|---------------|
| `ParseResult<ast::schema::Document>` | `GraphQLSchemaParseResult` | `graphql_schema_parse_result_free` |
| `ParseResult<ast::operation::Document>` | `GraphQLExecutableParseResult` | `graphql_executable_parse_result_free` |
| `ParseResult<ast::MixedDocument>` | `GraphQLMixedParseResult` | `graphql_mixed_parse_result_free` |
| `ast::schema::Document` | `GraphQLSchemaDocument` | (owned by ParseResult) |
| `ast::operation::Document` | `GraphQLExecutableDocument` | (owned by ParseResult) |
| `ast::MixedDocument` | `GraphQLMixedDocument` | (owned by ParseResult) |
| `Vec<GraphQLParseError>` | `GraphQLParseErrors` | (owned by ParseResult) |
| `GraphQLParseError` | `GraphQLParseErrorRef` | (borrowed ptr) |
| `GraphQLErrorNote` | `GraphQLErrorNoteRef` | (borrowed ptr) |
| `GraphQLSourceSpan` | `GraphQLSourceSpanRef` | (borrowed ptr) |
| `SourcePosition` | `GraphQLSourcePositionRef` | (borrowed ptr) |
| Individual AST nodes | `GraphQL<NodeType>Ref` | (borrowed ptr, owned by document) |

**Ownership rules (documented in header):**
- Functions returning `*_free`-able handles transfer ownership to the caller. Caller MUST call the corresponding `*_free`.
- Functions returning `*Ref` handles borrow from a parent. Valid only while the parent handle is live. Caller MUST NOT free these.
- All `const char *` string returns borrow from the parent handle. Caller must copy if needed beyond parent lifetime.

---

#### 4.9.3 Core Parser API Functions

```c
/* ── Parsing entry points ── */

// Parse a GraphQL schema document from a UTF-8 string.
// `source` must be valid UTF-8; `source_len` is byte length.
// Returns an owned handle; caller must call
// graphql_schema_parse_result_free().
GraphQLSchemaParseResult
graphql_parse_schema(const char *source, size_t source_len);

GraphQLExecutableParseResult
graphql_parse_executable(const char *source, size_t source_len);

GraphQLMixedParseResult
graphql_parse_mixed(const char *source, size_t source_len);


/* ── ParseResult accessors (one set per document kind) ── */

// Returns true if parsing succeeded with no errors.
bool graphql_schema_parse_result_is_ok(
    const GraphQLSchemaParseResult result);

// Returns true if any errors were encountered.
bool graphql_schema_parse_result_has_errors(
    const GraphQLSchemaParseResult result);

// Returns the AST only if parsing was fully successful (no errors).
// Returns NULL if there were errors or no AST was produced.
// The returned pointer borrows from `result`.
const GraphQLSchemaDocument
graphql_schema_parse_result_valid_ast(
    const GraphQLSchemaParseResult result);

// Returns the AST if present, regardless of errors (best-effort).
// Returns NULL if no AST was produced at all.
const GraphQLSchemaDocument
graphql_schema_parse_result_ast(
    const GraphQLSchemaParseResult result);

// Number of parse errors.
size_t graphql_schema_parse_result_error_count(
    const GraphQLSchemaParseResult result);

// Access the nth error (borrowed; valid while `result` is live).
const GraphQLParseErrorRef
graphql_schema_parse_result_error_at(
    const GraphQLSchemaParseResult result, size_t index);

// Format all errors as a single string.
// `source` / `source_len`: optional original source for snippets
//                           (pass NULL/0 to omit).
// Caller must free returned string via graphql_string_free().
char *graphql_schema_parse_result_format_errors(
    const GraphQLSchemaParseResult result,
    const char *source, size_t source_len);

void graphql_schema_parse_result_free(
    GraphQLSchemaParseResult result);

// (Identical pattern for Executable and Mixed variants)
```

**Tasks:**
1. Implement `graphql_parse_schema`, `graphql_parse_executable`, `graphql_parse_mixed`
2. Implement `ParseResult` accessor family for each document kind
3. Implement `graphql_string_free` for caller-owned strings
4. Test round-trip: parse in C, check `is_ok`, access AST root, free

---

#### 4.9.4 AST Node Accessor API

Every AST node type needs accessor functions. The pattern is uniform:

```
const <ChildHandle> graphql_<parent>_get_<field>(const <ParentHandle> h);
size_t graphql_<parent>_<list_field>_count(const <ParentHandle> h);
const <ChildHandle> graphql_<parent>_<list_field>_at(
    const <ParentHandle> h, size_t index);
```

**Schema document AST types to expose (from `graphql-parser` 0.4, pending custom AST):**

| AST Node | C Handle | Key Accessors |
|----------|----------|---------------|
| `schema::Document` | `GraphQLSchemaDocument` | `definitions_count`, `definition_at` |
| `schema::Definition` (enum) | `GraphQLSchemaDefinitionRef` | `kind` → tag enum, `as_type_definition`, `as_schema_definition`, etc. |
| `schema::SchemaDefinition` | `GraphQLSchemaDefRef` | `query`, `mutation`, `subscription`, `directives_*` |
| `schema::TypeDefinition` (enum) | `GraphQLTypeDefinitionRef` | `kind` tag, `as_object`, `as_interface`, etc. |
| `schema::ObjectType` | `GraphQLObjectTypeRef` | `name`, `implements_count`, `implements_at`, `fields_count`, `field_at`, `directives_*`, `description` |
| `schema::InterfaceType` | `GraphQLInterfaceTypeRef` | same pattern as ObjectType |
| `schema::UnionType` | `GraphQLUnionTypeRef` | `name`, `members_count`, `member_at`, `directives_*` |
| `schema::EnumType` | `GraphQLEnumTypeRef` | `name`, `values_count`, `value_at`, `directives_*` |
| `schema::EnumValue` | `GraphQLEnumValueRef` | `name`, `description`, `directives_*` |
| `schema::ScalarType` | `GraphQLScalarTypeRef` | `name`, `directives_*`, `description` |
| `schema::InputObjectType` | `GraphQLInputObjectTypeRef` | `name`, `fields_count`, `field_at`, `directives_*` |
| `schema::Field` | `GraphQLFieldDefRef` | `name`, `field_type`, `arguments_count`, `argument_at`, `directives_*`, `description` |
| `schema::InputValue` | `GraphQLInputValueRef` | `name`, `value_type`, `default_value`, `directives_*`, `description` |
| `schema::DirectiveDefinition` | `GraphQLDirectiveDefRef` | `name`, `arguments_*`, `locations_*`, `repeatable`, `description` |
| `schema::Type` (enum) | `GraphQLTypeAnnotationRef` | `kind` tag, `named_type_name`, `list_inner_type`, `non_null_inner_type` |
| Type extensions (Object, Interface, Union, Enum, Scalar, InputObject) | `GraphQL<X>ExtensionRef` | mirror base type accessors |

**Operation/executable document AST types:**

| AST Node | C Handle | Key Accessors |
|----------|----------|---------------|
| `operation::Document` | `GraphQLExecutableDocument` | `definitions_count`, `definition_at` |
| `operation::Definition` (enum) | `GraphQLExecDefinitionRef` | `kind` tag, `as_operation`, `as_fragment` |
| `operation::OperationDefinition` (enum) | `GraphQLOperationRef` | `kind` tag → SelectionSet / Query / Mutation / Subscription |
| `operation::Query` | `GraphQLQueryRef` | `name`, `variable_definitions_*`, `directives_*`, `selection_set` |
| `operation::Mutation` | `GraphQLMutationRef` | same as Query |
| `operation::Subscription` | `GraphQLSubscriptionRef` | same as Query |
| `operation::SelectionSet` | `GraphQLSelectionSetRef` | `items_count`, `item_at` |
| `operation::Selection` (enum) | `GraphQLSelectionRef` | `kind` tag, `as_field`, `as_fragment_spread`, `as_inline_fragment` |
| `operation::Field` | `GraphQLFieldRef` | `alias`, `name`, `arguments_*`, `directives_*`, `selection_set` |
| `operation::FragmentSpread` | `GraphQLFragmentSpreadRef` | `fragment_name`, `directives_*` |
| `operation::InlineFragment` | `GraphQLInlineFragmentRef` | `type_condition`, `directives_*`, `selection_set` |
| `operation::FragmentDefinition` | `GraphQLFragmentDefRef` | `name`, `type_condition`, `directives_*`, `selection_set` |
| `operation::VariableDefinition` | `GraphQLVariableDefRef` | `name`, `var_type`, `default_value`, `directives_*` |
| `operation::Directive` | `GraphQLDirectiveRef` | `name`, `arguments_count`, `argument_at` |
| `Value` (enum) | `GraphQLValueRef` | `kind` tag, `as_int`, `as_float`, `as_string`, `as_boolean`, `as_null`, `as_enum`, `as_list`, `as_object` |
| `Number` | (inline `int64_t`) | direct return |

**Mixed document types:**

| AST Node | C Handle | Key Accessors |
|----------|----------|---------------|
| `MixedDocument` | `GraphQLMixedDocument` | `definitions_count`, `definition_at` |
| `MixedDefinition` (enum) | `GraphQLMixedDefinitionRef` | `kind` tag, `as_schema`, `as_executable` |

**Enum tags** (all `#[repr(C)]`):

Every Rust enum that appears in the AST needs a C-side tag enum:
- `GraphQLSchemaDefinitionKind` { SchemaDefinition, TypeDefinition, TypeExtension, DirectiveDefinition }
- `GraphQLTypeDefinitionKind` { Scalar, Object, Interface, Union, Enum, InputObject }
- `GraphQLTypeExtensionKind` { Scalar, Object, Interface, Union, Enum, InputObject }
- `GraphQLTypeAnnotationKind` { Named, List, NonNull }
- `GraphQLExecDefinitionKind` { Operation, Fragment }
- `GraphQLOperationKind` { Query, Mutation, Subscription, SelectionSet }
- `GraphQLSelectionKind` { Field, FragmentSpread, InlineFragment }
- `GraphQLValueKind` { Int, Float, String, Boolean, Null, Enum, List, Object, Variable }
- `GraphQLMixedDefinitionKind` { Schema, Executable }
- `GraphQLDirectiveLocation` (all 18 locations per spec)

**Tasks:**
1. Define all `#[repr(C)]` tag enums
2. Implement accessor functions for each AST node type
3. Ensure `NULL` is returned for `Option::None` fields
4. Test: parse a schema in C, walk entire AST tree, verify field values

---

#### 4.9.5 Error Infrastructure API

```c
/* ── GraphQLParseError accessors ── */

// Returns error message as borrowed string (valid while error is live).
GraphQLStringRef graphql_parse_error_message(
    const GraphQLParseErrorRef err);

// Returns the primary source span.
const GraphQLSourceSpanRef graphql_parse_error_span(
    const GraphQLParseErrorRef err);

// Returns the error kind tag.
GraphQLParseErrorKind graphql_parse_error_kind(
    const GraphQLParseErrorRef err);

// Notes count and indexed access.
size_t graphql_parse_error_notes_count(
    const GraphQLParseErrorRef err);
const GraphQLErrorNoteRef graphql_parse_error_note_at(
    const GraphQLParseErrorRef err, size_t index);

// Format as detailed multi-line diagnostic string.
// Caller must free via graphql_string_free().
char *graphql_parse_error_format_detailed(
    const GraphQLParseErrorRef err,
    const char *source, size_t source_len);

// Format as single-line summary.
// Caller must free via graphql_string_free().
char *graphql_parse_error_format_oneline(
    const GraphQLParseErrorRef err);


/* ── GraphQLParseErrorKind tag enum ── */

typedef enum {
    GRAPHQL_ERROR_UNEXPECTED_TOKEN,
    GRAPHQL_ERROR_UNEXPECTED_EOF,
    GRAPHQL_ERROR_LEXER_ERROR,
    GRAPHQL_ERROR_UNCLOSED_DELIMITER,
    GRAPHQL_ERROR_MISMATCHED_DELIMITER,
    GRAPHQL_ERROR_INVALID_VALUE,
    GRAPHQL_ERROR_RESERVED_NAME,
    GRAPHQL_ERROR_WRONG_DOCUMENT_KIND,
    GRAPHQL_ERROR_INVALID_EMPTY_CONSTRUCT,
    GRAPHQL_ERROR_INVALID_SYNTAX,
} GraphQLParseErrorKind;


/* ── GraphQLErrorNote accessors ── */

GraphQLErrorNoteKind graphql_error_note_kind(
    const GraphQLErrorNoteRef note);
GraphQLStringRef graphql_error_note_message(
    const GraphQLErrorNoteRef note);
// Returns NULL if note has no span.
const GraphQLSourceSpanRef graphql_error_note_span(
    const GraphQLErrorNoteRef note);

typedef enum {
    GRAPHQL_NOTE_GENERAL,
    GRAPHQL_NOTE_HELP,
    GRAPHQL_NOTE_SPEC,
} GraphQLErrorNoteKind;


/* ── SourcePosition / SourceSpan ── */

size_t graphql_source_position_line(
    const GraphQLSourcePositionRef pos);
size_t graphql_source_position_col_utf8(
    const GraphQLSourcePositionRef pos);
// Returns SIZE_MAX if col_utf16 is unavailable.
size_t graphql_source_position_col_utf16(
    const GraphQLSourcePositionRef pos);
size_t graphql_source_position_byte_offset(
    const GraphQLSourcePositionRef pos);

const GraphQLSourcePositionRef graphql_source_span_start(
    const GraphQLSourceSpanRef span);
const GraphQLSourcePositionRef graphql_source_span_end(
    const GraphQLSourceSpanRef span);
// Returns NULL if no file path. Borrowed string.
GraphQLStringRef graphql_source_span_file_path(
    const GraphQLSourceSpanRef span);
```

**Tasks:**
1. Implement all error/note/span/position accessors
2. Implement `graphql_string_free` for caller-owned strings
3. Test: parse invalid GraphQL in C, iterate errors, check messages/spans

---

#### 4.9.6 String Return Convention

Two patterns:
1. **Borrowed strings** (`GraphQLStringRef`): pointer + length into Rust-owned memory.
   ```c
   typedef struct {
       const char *data;  // UTF-8, NOT null-terminated
       size_t len;        // byte length
   } GraphQLStringRef;
   ```
   Valid for the lifetime of the parent handle. Caller MUST NOT free. Zero-copy.

2. **Owned strings** (`char *`): null-terminated, heap-allocated via Rust's allocator.
   Returned by formatting functions (`format_detailed`, `format_errors`).
   Caller MUST free via `graphql_string_free(char *s)`.

**Tasks:**
1. Define `GraphQLStringRef` struct in header
2. Implement `graphql_string_free` (calls `CString` drop or `dealloc`)
3. Document convention prominently in header comments

---

#### 4.9.7 C++ Convenience Header

`libgraphql_parser.hpp` wraps the C API with:

1. **RAII handle wrappers** (`graphql::SchemaParseResult`, etc.) — call `*_free` in destructor, non-copyable, movable
2. **`std::string_view` accessors** — convert `GraphQLStringRef` to `std::string_view`
3. **Range-based iteration** — `for (auto &def : doc.definitions()) { ... }`
4. **Type-safe enum classes** — wrap C tag enums as `enum class`
5. **`std::optional`** — for nullable fields (instead of NULL checks)
6. **`std::variant`** visitors — for sum-type AST nodes (e.g., `Definition`, `Selection`)

This is header-only, no additional compilation needed. Requires C++17 minimum.

**Tasks:**
1. RAII wrappers for all `*_free`-able handles
2. Iterator adapters for list accessors
3. `std::string_view` conversion helpers
4. `std::variant` / `std::visit` for enum nodes
5. Test: same parse+walk test as C tests but with C++ idioms

---

#### 4.9.8 Token & Trivia API (Optional / Stretch)

Expose the token-level API for tools that need token streams (syntax highlighters, formatters):

```c
/* ── Lexer / Token Stream ── */

// Create a token stream from source text.
GraphQLTokenStream
graphql_token_stream_new(const char *source, size_t source_len);

// Advance to next token. Returns false at end of input.
bool graphql_token_stream_next(GraphQLTokenStream stream);

// Access current token (borrowed; valid until next call to _next).
const GraphQLTokenRef
graphql_token_stream_current(const GraphQLTokenStream stream);

void graphql_token_stream_free(GraphQLTokenStream stream);


/* ── Token accessors ── */

GraphQLTokenKindTag graphql_token_kind(const GraphQLTokenRef tok);
const GraphQLSourceSpanRef graphql_token_span(
    const GraphQLTokenRef tok);
// For Name/IntValue/FloatValue/StringValue — raw text.
GraphQLStringRef graphql_token_raw_value(const GraphQLTokenRef tok);
// For StringValue — parsed/unescaped value. Caller frees.
char *graphql_token_parse_string_value(const GraphQLTokenRef tok);
// For IntValue — parsed i64.
bool graphql_token_parse_int_value(
    const GraphQLTokenRef tok, int64_t *out);
// For FloatValue — parsed f64.
bool graphql_token_parse_float_value(
    const GraphQLTokenRef tok, double *out);

// Preceding trivia.
size_t graphql_token_trivia_count(const GraphQLTokenRef tok);
const GraphQLTriviaTokenRef graphql_token_trivia_at(
    const GraphQLTokenRef tok, size_t index);

GraphQLTriviaKind graphql_trivia_kind(
    const GraphQLTriviaTokenRef trivia);
GraphQLStringRef graphql_trivia_value(
    const GraphQLTriviaTokenRef trivia);  // comment text
const GraphQLSourceSpanRef graphql_trivia_span(
    const GraphQLTriviaTokenRef trivia);

typedef enum {
    GRAPHQL_TOKEN_AMPERSAND,
    GRAPHQL_TOKEN_AT,
    GRAPHQL_TOKEN_BANG,
    GRAPHQL_TOKEN_COLON,
    GRAPHQL_TOKEN_CURLY_BRACE_CLOSE,
    GRAPHQL_TOKEN_CURLY_BRACE_OPEN,
    GRAPHQL_TOKEN_DOLLAR,
    GRAPHQL_TOKEN_ELLIPSIS,
    GRAPHQL_TOKEN_EQUALS,
    GRAPHQL_TOKEN_PAREN_CLOSE,
    GRAPHQL_TOKEN_PAREN_OPEN,
    GRAPHQL_TOKEN_PIPE,
    GRAPHQL_TOKEN_SQUARE_BRACKET_CLOSE,
    GRAPHQL_TOKEN_SQUARE_BRACKET_OPEN,
    GRAPHQL_TOKEN_NAME,
    GRAPHQL_TOKEN_INT_VALUE,
    GRAPHQL_TOKEN_FLOAT_VALUE,
    GRAPHQL_TOKEN_STRING_VALUE,
    GRAPHQL_TOKEN_TRUE,
    GRAPHQL_TOKEN_FALSE,
    GRAPHQL_TOKEN_NULL,
    GRAPHQL_TOKEN_EOF,
    GRAPHQL_TOKEN_ERROR,
} GraphQLTokenKindTag;

typedef enum {
    GRAPHQL_TRIVIA_COMMENT,
    GRAPHQL_TRIVIA_COMMA,
} GraphQLTriviaKind;
```

**Tasks:**
1. Wrap `StrGraphQLTokenSource` + `GraphQLTokenStream` behind opaque handle
2. Implement token/trivia accessors
3. Test: lex a document in C, verify token sequence

---

#### 4.9.9 Testing Strategy

Three tiers:

1. **Rust-side `#[test]` FFI tests** — call `#[no_mangle]` functions from Rust tests with raw pointers. Fastest feedback loop. Verify handle creation/freeing, NULL returns, accessor correctness.

2. **C test programs** (`tests/c/*.c`) — compiled with system C compiler, linked against `libgraphql_parser`. Run via `cargo test` build script or CI script. Cover:
   - Parse valid schema → walk full AST → verify names/types/field counts
   - Parse valid executable doc → walk operations/fragments
   - Parse invalid input → iterate errors → check messages and spans
   - Parse mixed document → check mixed definition kinds
   - Memory: parse → free → no leaks (valgrind/ASAN in CI)
   - Edge cases: empty input, huge input, deeply nested input

3. **C++ test programs** (`tests/cpp/*.cpp`) — same scenarios using C++ RAII wrappers. Verify:
   - Destructor calls `_free` correctly
   - Range-based for loops work
   - `std::string_view` accessors return correct data
   - Move semantics work, copy is deleted

**CI additions:**
- Build C library (`cargo build --package libgraphql-parser-c`)
- Compile C test with `cc -std=c99 -Wall -Werror`
- Compile C++ test with `c++ -std=c++17 -Wall -Werror`
- Link and run tests
- AddressSanitizer + valgrind passes on test programs

---

#### 4.9.10 Documentation

1. **`libgraphql_parser.h` header docs** — every function, type, enum value documented with `/** ... */` doxygen-style comments. Include ownership rules, lifetime constraints, NULL behavior.
2. **`README.md` in `libgraphql-parser-c/`** — build instructions, usage examples (C and C++), linking guide, platform notes.
3. **Example programs** — `examples/parse_schema.c`, `examples/walk_ast.c`, `examples/error_handling.c`.

---

#### 4.9.11 Implementation Order

| Phase | What | Depends On |
|-------|------|------------|
| **Phase 0** | Section 4.2 custom AST complete | — |
| **Phase 1** | Crate skeleton, handle machinery, `GraphQLStringRef`, `GraphQLStatus`, `graphql_string_free` | Phase 0 |
| **Phase 2** | Parser entry points (`graphql_parse_schema/executable/mixed`) + `ParseResult` accessors | Phase 1 |
| **Phase 3** | Error infrastructure accessors (`GraphQLParseError`, `GraphQLErrorNote`, `SourceSpan`, `SourcePosition`) | Phase 1 |
| **Phase 4** | AST node accessors — schema types first (most complex), then executable, then mixed | Phase 2 |
| **Phase 5** | C test programs (parse + walk + error cases + ASAN) | Phases 2-4 |
| **Phase 6** | C++ convenience header + C++ tests | Phase 5 |
| **Phase 7** | Token/Trivia API (stretch) | Phase 1 |
| **Phase 8** | Documentation, examples, `pkg-config` | Phase 6 |

---

#### 4.9.12 Unresolved Questions

1. **cbindgen vs. hand-maintained header?** `cbindgen` auto-generates from `#[repr(C)]` types + `#[no_mangle]` functions but can be finicky with complex generics. Opaque-handle pattern may require hand-maintained header. Investigate `cbindgen`'s support for our pattern; fall back to hand-maintained if needed.
2. **Version/ABI stability.** Since the custom AST (4.2) is itself in flux, when do we commit to a stable C ABI? Suggest: mark the initial C API as `0.x` (unstable) and reserve ABI-breaking changes until `1.0`.
3. **Allocator.** The Rust global allocator is used for all heap allocations. Should we provide a custom-allocator hook for C callers? Likely overkill for `0.x` — defer.
4. **Thread safety.** Individual parse results / AST trees should be `Send` (usable from any thread after creation). Concurrent mutation of a single handle is not supported. Document this.
5. **Serialization.** Should the C API expose `serde`-based JSON serialization of the AST? Useful for language bindings (Python, Node.js) that prefer JSON. Likely a stretch goal for post-1.0.
6. **Windows support.** `cdylib` on Windows produces `.dll`. Need to verify `__declspec(dllexport)` / `__declspec(dllimport)` handling. `cbindgen` may handle this; verify.
7. **`graphql-parser` crate AST re-exports.** The current AST (pre-custom) is a thin `type alias` over `graphql-parser` 0.4 types. These are deeply generic (`<'a, T>`) and heavily `Box`/`Vec`-based. The C API plan above assumes a custom AST. If Section 4.2 is delayed, a minimal "parse-and-serialize-to-JSON" C API could be an interim alternative.

### Definition of Done
- [ ] `libgraphql-parser-c` crate builds `cdylib` + `staticlib`
- [ ] `libgraphql_parser.h` header covers all public types and functions
- [ ] Parse entry points work from C: schema, executable, mixed
- [ ] Full AST walk possible from C (all node types accessible)
- [ ] Error iteration works from C (messages, spans, notes)
- [ ] C++ RAII wrapper header exists and is tested
- [ ] No memory leaks (ASAN/valgrind clean)
- [ ] CI compiles and runs C and C++ tests
- [ ] `pkg-config` `.pc` file generated
- [ ] README with build/link/usage instructions

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

## Appendix: Code TODOs

TODOs found in the codebase (auto-generated 2026-01-22):

| File                               | Line | TODO                                             |
|------------------------------------|------|--------------------------------------------------|
| `graphql_parser.rs`                |  431 | Eliminate clone (see docblock)                   |
| `graphql_parser.rs`                |  556 | Test expect_keyword("true") behavior             |
| `graphql_parser.rs`                |  604 | Test peek_is_keyword("true") behavior            |
| `graphql_parser.rs`                |  937 | Consider eliminating clone                       |
| `graphql_parser.rs`                | 1794 | Track variable directives (needs custom AST)     |
| `graphql_parser.rs`                | 2656 | Support schema extensions (needs custom AST)     |
| `graphql_parser.rs`                | 2669 | Support schema extensions (needs custom AST)     |
| `graphql_parser.rs`                | 2723 | Support schema extensions (needs custom AST)     |
| `graphql_token_kind.rs`            |  111 | Explore richer diagnostics structure             |
| `str_to_graphql_token_source.rs`   |  443 | Detect `{Name}.{Name}` patterns for better error |

---

## Priority Summary

**HIGH Priority:**
- ~~Fuzz testing (Section 3.1) — ✅ COMPLETE~~
- Coverage-driven test discovery (Section 2.1) — find important untested paths
- External test suite gap analysis (Section 2.2) — comprehensive coverage
- apollo-parser test suite audit (Section 2.7) — parity with apollo-rs test coverage

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
- Custom AST / syntax tree (Section 4.2)
- Spec-version feature flags (Section 4.8)
- All other Section 4 items
- C API / FFI bindings (Section 4.9) — depends on custom AST (4.2)
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

### Fuzz Testing (Section 3.1) — completed 2026-01-30

- `cargo-fuzz` infrastructure with 4 targets: lexer, schema parser, executable parser, mixed parser
- Seed corpus with 10 hand-crafted `.graphql` files
- Parallel runner script (`scripts/run-fuzz-tests.sh`, bash 3.2 compatible for macos)
- 10 bugs found and fixed: 7 infinite-loop/OOM, 1 stack overflow (recursion depth guard), 2 block string UTF-8 panics
- 15-min sustained run per target (25.5M total executions), zero crashes
- Results documented in `crates/libgraphql-parser/README.md`
