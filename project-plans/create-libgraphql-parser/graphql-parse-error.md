# GraphQLParseError Infrastructure & Parser Implementation

> **Status:** Complete (error infrastructure + full parser)
> **Location:** `crates/libgraphql-parser/src/`
> **Last Updated:** 2026-01-20

---

## TL;DR — High-Level Summary

### ✅ Originally Planned (Error Infrastructure) — COMPLETE

- **Built comprehensive `GraphQLParseError` infrastructure** — A structured error system with categorized error kinds (`GraphQLParseErrorKind`), contextual notes (`GraphQLErrorNote`), and Rust-style diagnostic formatting with source snippets
- **Refactored `GraphQLTokenSpan` → `GraphQLSourceSpan`** — Promoted the span type from the `token` submodule to a top-level export to enable sharing across both lexer and parser error systems
- **Created detailed parser design documentation** — ~2,000 lines of design documentation in `parser-design.md` covering error handling philosophy, API design, and implementation strategy
- **Updated `RustMacroGraphQLTokenSource`** to use the new `GraphQLSourceSpan` and `GraphQLErrorNote` types

### ✅ Implemented Beyond Original Scope — COMPLETE

- **Full `GraphQLParser<T>` implementation** — 3,236-line recursive descent parser with 56 `parse_*` methods
- **`ParseResult<T>` struct** — Enables partial AST + errors for IDE-friendly parsing
- **Schema document parsing** — `parse_schema_document()` handles all type system definitions
- **Executable document parsing** — `parse_executable_document()` handles operations/fragments
- **Mixed document parsing** — `parse_mixed_document()` handles interleaved schema + executable definitions
- **Error recovery** — Delimiter stack tracking, recovery-to-next-definition logic
- **Comprehensive test suite** — 183 tests covering values, types, directives, operations, fragments, type definitions, extensions, error cases

### 🔲 Remaining Work

- **Vendored tests from graphql-js/graphql-parser** — Port test cases from reference implementations
- **Fuzzing infrastructure** — Add fuzz testing for parser robustness
- **README documentation** — Add crate-level README with usage examples

---

## Things You Should Know While Reviewing

1. **The `GraphQLParseError` formatting methods are designed for Rustc-style output** — The `format_detailed()` method produces output similar to Rust compiler errors with source snippets, line numbers, and caret underlining.

2. **`GraphQLErrorNotes` uses `SmallVec<[GraphQLErrorNote; 2]>`** — Most errors have 0-2 notes, so this avoids heap allocation in the common case.

3. **The `parser-design.md` document is extensive (~2,000 lines)** — It contains detailed design rationale, API sketches, and implementation strategy.

4. **`GraphQLTokenSpan` → `GraphQLSourceSpan` rename** — Promoted from `token/` to crate root level. This affects internal structure but no public API changes.

5. **`CookGraphQLStringError` → `GraphQLStringParsingError` rename** — The new name better describes its purpose and follows the naming pattern of other error types.

6. **The parser is fully implemented** — `GraphQLParser<T>` provides `parse_schema_document()`, `parse_executable_document()`, and `parse_mixed_document()` methods with error recovery.

7. **`ParseResult<T>` enables partial parsing** — Unlike `Result<T, E>`, it can contain both a partial AST and errors, enabling IDE features even when parsing fails.

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
| `SourcePosition`            | `source_position.rs:39-52`        | Position with line, col_utf8, col_utf16 (optional), byte_offset             |

### 6. `ParseResult<T>` (New)

> **Location:** `crates/libgraphql-parser/src/parse_result.rs`

Result type enabling partial parsing — can contain both AST and errors.

| Method          | Returns          | Purpose                                              |
|-----------------|------------------|------------------------------------------------------|
| `valid_ast()`   | `Option<&TAst>`  | AST only if parsing was error-free                   |
| `ast()`         | `Option<&TAst>`  | AST if present, regardless of errors (for IDE use)   |
| `is_ok()`       | `bool`           | True if has AST and no errors                        |
| `has_errors()`  | `bool`           | True if any errors were recorded                     |
| `format_errors()`| `String`        | Formats all errors for display                       |

### 7. `GraphQLParser<'src, T>` (New)

> **Location:** `crates/libgraphql-parser/src/graphql_parser.rs` (3,236 lines)

Recursive descent parser generic over token source. Key public methods:

| Method                       | Returns                              | Purpose                              |
|------------------------------|--------------------------------------|--------------------------------------|
| `new(token_source)`          | `Self`                               | Create parser from token source      |
| `parse_schema_document()`    | `ParseResult<ast::schema::Document>` | Parse type system definitions only   |
| `parse_executable_document()`| `ParseResult<ast::operation::Document>` | Parse operations/fragments only   |
| `parse_mixed_document()`     | `ParseResult<ast::MixedDocument>`    | Parse both interleaved              |

Internal features:
- **56 `parse_*` methods** covering all GraphQL constructs (values, types, directives, selections, operations, fragments, type definitions, extensions)
- **Delimiter stack** for tracking `{`, `[`, `(` for error recovery
- **`recover_to_next_definition()`** for multi-error collection

### 8. `GraphQLTokenStream<'src, T>` (New)

> **Location:** `crates/libgraphql-parser/src/graphql_token_stream.rs`

Buffered token stream with lookahead support.

| Method          | Purpose                                           |
|-----------------|---------------------------------------------------|
| `peek()`        | Look at current token without consuming           |
| `peek_nth(n)`   | Look ahead n tokens                               |
| `consume()`     | Advance to next token                             |
| `current_token()` | Get the most recently consumed token            |

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

### 7. `ParseResult<T>` Design

Unlike `Result<T, E>`, `ParseResult` can hold both a partial AST and errors simultaneously:
- **IDE integration**: Show syntax errors while still providing completions from partial AST
- **Batch error reporting**: Report all syntax errors in one pass
- **Graceful degradation**: Process as much of a document as possible

Two access patterns: `valid_ast()` for strict mode (compilers), `ast()` for best-effort mode (IDE/formatters).

### 8. Parser Error Recovery Strategy

The parser uses delimiter tracking and definition-boundary recovery:
- **Delimiter stack**: Tracks `{`, `[`, `(` with context (e.g., "selection set", "list value")
- **Recovery**: On error, skip tokens until next definition keyword (`type`, `query`, etc.)
- **Multi-error collection**: Continues parsing after recovery to find additional errors

### 9. Generic Token Source

`GraphQLParser<'src, T: GraphQLTokenSource<'src>>` enables:
- **String input**: `StrGraphQLTokenSource` for runtime parsing
- **Proc-macro input**: `RustMacroGraphQLTokenSource` for compile-time parsing
- Shared parser logic with different lexer backends

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
- `ParseResult`
- `GraphQLParser`
- `GraphQLTokenStream`
- `SourcePosition`

### Run Parser Tests

```bash
cargo test --package libgraphql-parser
```

**Current coverage:** 383 tests passing (183 parser tests + token/lexer tests)

Test categories:
- Value parsing (int, float, string, boolean, null, enum, list, object)
- Type annotations (named, list, non-null)
- Directives (with/without arguments, const contexts)
- Selection sets (fields, aliases, fragments, inline fragments)
- Operations (query, mutation, subscription, variables)
- Fragment definitions
- Schema definitions (all type kinds)
- Type extensions (all extension kinds)
- Error cases (unclosed delimiters, unexpected tokens, reserved names)

### Verify `RustMacroGraphQLTokenSource` Still Works

```bash
cargo test --package libgraphql-macros rust_macro_graphql_token_source
```

All existing tests should continue to pass after the refactoring to use `GraphQLSourceSpan`.
