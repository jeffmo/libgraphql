# libgraphql-macros — Consolidated Plans & Remaining Work

**Last Updated:** 2026-01-22

This document consolidates all remaining work for the `libgraphql-macros` crate.

## Document Maintenance Notes

When updating this document:

1. **Completed items:** Move wholly-completed plan items to the "Past Completed Work" section at the end of this document. Include a simple title and terse description only.
2. **Plan identifiers:** NEVER re-number existing plan items (e.g., 4.3, 2.1). This ensures references to plan IDs remain valid over time.
3. **Partial completion:** If a plan item is partially done, leave it in place and update its description to reflect remaining work.

---

## Current State Summary

**Test Status:** 163 tests passing, 4 doc-tests passing, 2 compile-fail tests passing

**Core Implementation: ✅ Functional**
- `graphql_schema!` proc macro for compile-time schema validation
- `RustMacroGraphQLTokenSource` for tokenizing Rust token streams as GraphQL
- Error reporting with proc-macro span integration
- AST equivalence with runtime parser

**Remaining Work Categories:**
1. Error Reporting Enhancements (Section 1)
2. Dead Code Cleanup (Section 2)

---

## Section 1: Error Reporting Enhancements

### 1.1 Secondary Span Notes

**Purpose:** Improve error diagnostics by emitting additional notes at secondary source locations.

**Current Progress:** TODO exists but not implemented.

**Priority:** LOW

#### Tasks

1. **Emit notes at secondary spans**
   - `graphql_parse_error.rs:56`: Emit additional notes at secondary spans
   - Would improve multi-location error context

### Definition of Done
- [ ] Secondary spans emit helpful notes
- [ ] Error messages show related locations

---

## Section 2: Dead Code Cleanup

### 2.1 Remove Unused Code

**Purpose:** Address compiler warnings for dead code.

**Current Progress:** Warnings reported by rustc.

**Priority:** LOW

#### Tasks

1. **ParseResult::err() unused**
   - `parse_result.rs:93`: Associated function `err` never used
   - Either use it or remove it

2. **RustMacroGraphQLTokenSource::new() unused**
   - `rust_macro_graphql_token_source.rs:138`: Associated function `new` never used
   - Either use it or remove it

### Definition of Done
- [ ] No dead_code warnings from rustc
- [ ] Unused functions either utilized or removed

---

## Priority Summary

**HIGH Priority:**
*(None currently)*

**MEDIUM Priority:**
*(None currently)*

**LOW Priority:**
- Secondary span notes (Section 1.1) — nice-to-have diagnostics improvement
- Dead code cleanup (Section 2.1) — compiler warning hygiene

---

## Past Completed Work

*Items moved here when wholly completed. Each entry includes a simple title and terse description.*

*(No items yet)*

---

## Appendix: Code TODOs

TODOs found in the codebase (auto-generated 2026-01-22):

| File                                 | Line | TODO                                        |
|--------------------------------------|------|---------------------------------------------|
| `graphql_parse_error.rs`             |   56 | Emit additional notes at secondary spans    |
| `parse_result.rs`                    |   93 | Dead code: `err` function never used        |
| `rust_macro_graphql_token_source.rs` |  138 | Dead code: `new` function never used        |
