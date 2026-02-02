# libgraphql-macros — Consolidated Project Management & Remaining Work

**Last Updated:** 2026-02-01

This document consolidates all remaining work for the `libgraphql-macros` crate.

## Document Maintenance Notes

When updating this document:

1. **Completed items:** Move wholly-completed plan items to the "Past Completed Work" section at the end of this document. Include a simple title and terse description only.
2. **Plan identifiers:** NEVER re-number existing plan items (e.g., 4.3, 2.1). This ensures references to plan IDs remain valid over time.
3. **Partial completion:** If a plan item is partially done, leave it in place and update its description to reflect remaining work.

---

## Current State Summary

**Test Status:** 139 tests passing, 4 doc-tests passing, 2 compile-fail tests passing

**Core Implementation: ✅ Functional**
- `graphql_schema!` proc macro for compile-time schema validation
- `RustMacroGraphQLTokenSource` for tokenizing Rust token streams as GraphQL
- Error reporting with proc-macro span integration
- AST equivalence with runtime parser

**Remaining Work Categories:**
1. ~~Error Reporting Enhancements (Section 1)~~ — ✅ COMPLETE
2. ~~Dead Code Cleanup (Section 2)~~ — ✅ COMPLETE
3. Token Source UX Improvements (Section 3)

---

## Section 1: Error Reporting Enhancements

**✅ COMPLETE** — Moved to Past Completed Work.

---

## Section 2: Dead Code Cleanup

**✅ COMPLETE** — Moved to Past Completed Work.

---

## Section 3: Token Source UX Improvements

### 3.1 Detect Unescaped Quotes Inside Block Strings

**Purpose:** When a user writes `"""The "output" string."""` inside `graphql_schema!`, Rust's tokenizer splits on the unescaped `"`, breaking block string recombination. The result is confusing parse errors. `RustMacroGraphQLTokenSource` should detect this pattern and emit a helpful error suggesting `\"` escaping.

**Priority:** MEDIUM

#### Tasks

1. **Investigate detection heuristic** — after `try_combine_block_string` fails to recombine, check if the surrounding tokens look like a broken block string (e.g. `""`, Name/other, `""` with no intervening punctuators)
2. **Emit a targeted error** — `GraphQLTokenKind::Error` with a note like: `help: escape inner quotes with \", e.g. """The \"output\" string."""`
3. **Add tests** — cover single embedded quote, multiple embedded quotes, and the case where `""` legitimately appears near other strings (no false positives)

### Definition of Done
- [ ] Broken block string pattern detected
- [ ] Helpful error message emitted with `\"` suggestion
- [ ] No false positives for legitimate `""` usage
- [ ] Tests cover detection and non-detection cases

---

## Priority Summary

**HIGH Priority:**
*(None currently)*

**MEDIUM Priority:**
- Detect unescaped quotes in block strings (Section 3.1) — better DX for common mistake

**LOW Priority:**
- Secondary span notes (Section 1.1) — nice-to-have diagnostics improvement
- Dead code cleanup (Section 2.1) — compiler warning hygiene

---

## Past Completed Work

*Items moved here when wholly completed. Each entry includes a simple title and terse description.*

### Secondary Span Notes (Section 1.1) (2026-02-01)
`parse_error_converter.rs` emits `compile_error!` at both primary and note span locations. Old `graphql_parse_error.rs` replaced during PR #50 parser integration.

### Dead Code Cleanup (Section 2.1) (2026-02-01)
`parse_result.rs` and old `rust_macro_graphql_token_source.rs::new()` removed during PR #50 parser integration. Zero `dead_code` warnings from `cargo build`.

---

## Appendix: Code TODOs

TODOs found in the codebase (auto-generated 2026-02-01):

| File                                 | Line | TODO                                          |
|--------------------------------------|------|-----------------------------------------------|
| `rust_macro_graphql_token_source.rs` |   29 | Emit nightly toolchain warning for byte_offset |
