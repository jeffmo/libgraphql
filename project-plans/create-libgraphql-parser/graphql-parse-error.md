# GraphQLParseError Infrastructure

> **Status:** Implemented
> **Location:** `crates/libgraphql-parser/src/`
> **Last Updated:** 2026-01-10

---

## TL;DR — High-Level Summary

- **Built comprehensive `GraphQLParseError` infrastructure** — A structured error system with categorized error kinds (`GraphQLParseErrorKind`), contextual notes (`GraphQLErrorNote`), and Rust-style diagnostic formatting with source snippets
- **Refactored `GraphQLTokenSpan` → `GraphQLSourceSpan`** — Promoted the span type from the `token` submodule to a top-level export to enable sharing across both lexer and parser error systems
- **Created detailed parser design documentation** — ~1,700 lines of design documentation in `parser-design.md` covering error handling philosophy, API design, and implementation strategy
- **Documented feature flags in README** — Added documentation for `macros` (default) and `use-libgraphql-parser` (experimental) feature flags
- **Updated `RustMacroGraphQLTokenSource`** to use the new `GraphQLSourceSpan` and `GraphQLErrorNote` types

---

## Things You Should Know While Reviewing

1. **The `GraphQLParseError` formatting methods are designed for Rustc-style output** — The `format_detailed()` method produces output similar to Rust compiler errors with source snippets, line numbers, and caret underlining.

2. **`GraphQLErrorNotes` uses `SmallVec<[GraphQLErrorNote; 2]>`** — Most errors have 0-2 notes, so this avoids heap allocation in the common case.

3. **The `parser-design.md` document is extensive (~1,700 lines)** — It contains detailed design rationale, API sketches, and implementation strategy. It's meant to guide implementation, not be read cover-to-cover for review.

4. **This PR reorganizes token module exports** — `GraphQLTokenSpan` was removed from `token/` and replaced with `GraphQLSourceSpan` at the crate root level. This affects internal structure but no public API changes.

5. **`CookGraphQLStringError` was renamed to `GraphQLStringParsingError`** — The new name better describes its purpose and follows the naming pattern of other error types.

6. **The error types are infrastructure only** — The parser itself hasn't been built yet; this PR establishes the error system that the parser will use.

---

## New Abstractions

### 1. `GraphQLParseError`

> **Location:** `crates/libgraphql-parser/src/graphql_parse_error.rs:13-41`

Comprehensive parse error with location info, categorized kind, and contextual notes.

| Field     | Type                    | Purpose                                          |
|-----------|-------------------------|--------------------------------------------------|
| `message` | `String`                | Human-readable primary error message             |
| `span`    | `GraphQLSourceSpan`     | Primary error location                           |
| `kind`    | `GraphQLParseErrorKind` | Categorized error type for programmatic handling |
| `notes`   | `GraphQLErrorNotes`     | Additional context, suggestions, spec references |

### 2. `GraphQLParseErrorKind`

> **Location:** `crates/libgraphql-parser/src/graphql_parse_error_kind.rs:15-179`

Enum categorizing parse errors for programmatic pattern-matching:

| Variant                 | Description                                            |
|-------------------------|--------------------------------------------------------|
| `UnexpectedToken`       | Expected specific token(s) but found something else   |
| `UnexpectedEof`         | Document ended before complete construct              |
| `LexerError`            | Wraps lexer error tokens                              |
| `UnclosedDelimiter`     | Opened delimiter without matching close               |
| `MismatchedDelimiter`   | Wrong closing delimiter (e.g., `[` closed with `)`)   |
| `InvalidValue`          | Value parsing error (overflow, bad escape)            |
| `ReservedName`          | Reserved name in wrong context (`on` as fragment name)|
| `WrongDocumentKind`     | Definition not allowed in document type               |
| `InvalidEmptyConstruct` | Empty construct that requires content                 |
| `InvalidSyntax`         | Catch-all for other syntax errors                     |

### 3. `GraphQLErrorNote`

> **Location:** `crates/libgraphql-parser/src/graphql_error_note.rs:13-25`

Individual note providing additional context about an error.

| Field     | Type                        | Purpose                         |
|-----------|-----------------------------|---------------------------------|
| `kind`    | `GraphQLErrorNoteKind`      | Note category (General, Help, Spec) |
| `message` | `String`                    | The note message                |
| `span`    | `Option<GraphQLSourceSpan>` | Optional related location       |

### 4. `GraphQLErrorNoteKind`

> **Location:** `crates/libgraphql-parser/src/graphql_error_note_kind.rs:7-25`

| Variant   | Rendering     | Example                             |
|-----------|---------------|-------------------------------------|
| `General` | `= note: ...` | "Opening `{` here"                  |
| `Help`    | `= help: ...` | "Did you mean: `userName: String`?" |
| `Spec`    | `= spec: ...` | "https://spec.graphql.org/..."      |

### 5. Supporting Types

| Type                        | Location                          | Purpose                                                                      |
|-----------------------------|-----------------------------------|------------------------------------------------------------------------------|
| `GraphQLSourceSpan`         | `graphql_source_span.rs:12-17`    | Half-open span with optional file path (promoted from `token::GraphQLTokenSpan`) |
| `DefinitionKind`            | `definition_kind.rs:4-21`         | Categorizes definition types (Schema, TypeDefinition, Operation, Fragment)  |
| `DocumentKind`              | `document_kind.rs:8-20`           | Document parsing mode (Schema, Executable, Mixed)                           |
| `ReservedNameContext`       | `reserved_name_context.rs:7-20`   | Context for reserved name errors (FragmentName, EnumValue)                  |
| `ValueParsingError`         | `value_parsing_error.rs:8-24`     | Value parsing errors (String, Int, Float)                                   |
| `GraphQLStringParsingError` | `graphql_string_parsing_error.rs` | Renamed from `CookGraphQLStringError`                                       |

---

## Key Design Points & Choices

### 1. Separation of `message` and `kind` in `GraphQLParseError`

The error stores both a human-readable `message` and a programmatic `kind`:
- `message`: "Expected `:` after field name" — shown to users
- `kind`: `UnexpectedToken { expected: [":"], found: "String" }` — for tools

This enables rich CLI output while supporting programmatic error handling (e.g., IDE quick-fixes).

### 2. Notes System with Three Categories

Notes are separated into `General`, `Help`, and `Spec` kinds:
- **General**: Contextual info ("Opening `{` here")
- **Help**: Actionable suggestions ("Did you mean: `userName: String`?")
- **Spec**: GraphQL specification links

This mirrors Rustc's error output and enables tools to filter/transform notes appropriately.

### 3. `SmallVec` for Notes

`GraphQLErrorNotes` uses `SmallVec<[GraphQLErrorNote; 2]>` because most errors have 0-2 notes. This avoids heap allocation in the common case.

### 4. Promotion of `GraphQLTokenSpan` to `GraphQLSourceSpan`

The span type was moved from `::token::GraphQLTokenSpan` to `::GraphQLSourceSpan` because:
- The parser error system needs spans
- Span is a general concept, not token-specific
- Enables consistent span type across lexer and parser errors

### 5. Rustc-Style Diagnostic Formatting

`GraphQLParseError::format_detailed()` produces output like:

```text
error: Expected `:` after field name
  --> schema.graphql:5:12
   |
 5 |     userName String
   |              ^^^^^^ expected `:`
   |
   = note: Field definitions require `:` between name and type
   = help: Did you mean: `userName: String`?
```

### 6. Document Kind Validation

The `WrongDocumentKind` error variant enables the parser to enforce that:
- Schema documents only contain type system definitions
- Executable documents only contain operations and fragments
- Mixed documents allow both (for tooling that processes complete codebases)

---

## Test Plan

### Build Verification

```bash
# Build all crates
cargo build

# Type-check including tests
cargo check --tests

# Run clippy
cargo clippy --tests
```

### Run All Tests

```bash
cargo test
```

### Verify New Types Export

```bash
# Check that new types are exported correctly
cargo doc --package libgraphql-parser --no-deps
```

New public exports to verify:
- `GraphQLParseError`
- `GraphQLParseErrorKind`
- `GraphQLErrorNote`
- `GraphQLErrorNotes`
- `GraphQLErrorNoteKind`
- `GraphQLSourceSpan`
- `DefinitionKind`
- `DocumentKind`
- `ReservedNameContext`
- `ValueParsingError`
- `GraphQLStringParsingError`

### Verify `RustMacroGraphQLTokenSource` Still Works

```bash
cargo test --package libgraphql-macros rust_macro_graphql_token_source
```

All 26 existing tests should continue to pass after the refactoring to use `GraphQLSourceSpan`.
