# GraphQL Parser Design Document

**Last Updated:** 2026-01-10

## Overview

This document outlines the comprehensive design for a spec-compliant GraphQL
lexer and parser in `libgraphql-parser`. The parser is parameterized over a
`GraphQLTokenSource` type, enabling both proc-macro and string-based parsing
from a unified implementation.

**Related:** This document is part of the broader `libgraphql-parser` project.
See `libgraphql-parser-plan.v9.md` for the overall project plan. This document
covers:
- **Phase 2:** String Lexer Implementation (`StrGraphQLTokenSource`) — Part 2.5
- **Phase 3:** Parser Extension for Operations — Parts 3–9

See also: `str-graphql-token-source-plan.md` for the standalone lexer plan.

---

## Current Implementation Status

Many foundational components described in this document have already been
implemented. This section summarizes what exists and what remains to be built.

### ✅ Completed (Parts 1–2)

| Component | Status | Location |
|-----------|--------|----------|
| `GraphQLSourceSpan` (renamed from `GraphQLTokenSpan`) | ✅ Done | `src/graphql_source_span.rs` |
| `file_path` field in `GraphQLSourceSpan` | ✅ Done | Includes `Option<PathBuf>` |
| "Cook" → "Parse" terminology rename | ✅ Done | `parse_int_value()`, `GraphQLStringParsingError`, etc. |
| `GraphQLErrorNoteKind` | ✅ Done | `src/graphql_error_note_kind.rs` |
| `GraphQLErrorNote` and `GraphQLErrorNotes` | ✅ Done | `src/graphql_error_note.rs` |
| `DefinitionKind` | ✅ Done | `src/definition_kind.rs` |
| `DocumentKind` | ✅ Done | `src/document_kind.rs` |
| `ReservedNameContext` | ✅ Done | `src/reserved_name_context.rs` |
| `ValueParsingError` | ✅ Done | `src/value_parsing_error.rs` |
| `GraphQLParseErrorKind` (10 variants) | ✅ Done | `src/graphql_parse_error_kind.rs` |
| `GraphQLParseError` with formatting | ✅ Done | `src/graphql_parse_error.rs` |

### ⏳ Remaining Work

| Component                                         | Status       | Notes                            |
|---------------------------------------------------|--------------|----------------------------------|
| **Phase 2: Lexer (`StrGraphQLTokenSource`)**      |              |                                  |
| Step 0: `Cow<'src, str>` refactoring              | ✅ COMPLETED | Part 2.5 — Prerequisite          |
| Step 1-7: Lexer implementation                    | ✅ COMPLETED | Part 2.5 — Core lexer (~1130 lines) |
| Step 8-10: Vendored tests, benchmarks, fuzzing    | 🔲 TODO      | Part 2.5 — Extended validation   |
| **Phase 3: Parser (`GraphQLParser`)**             |              |                                  |
| `ParseResult<T>` struct                           | ✅ COMPLETED | Part 3 — ~200 lines              |
| `GraphQLParser<S>` struct                         | ✅ COMPLETED | Part 3 — ~3200 lines             |
| Parser methods (values, types, directives, etc.)  | ✅ COMPLETED | Parts 4–5                        |
| Error recovery implementation                     | ✅ COMPLETED | Part 6                           |
| Parser tests                                      | ✅ COMPLETED | 383 tests, 4 doc-tests           |
| **Remaining validation work**                     |              |                                  |
| Vendored tests from graphql-js/graphql-parser     | 🔲 TODO      | Part 8 — License-compatible      |
| `RustMacroGraphQLTokenSource` parser tests        | 🔲 TODO      | Verify proc-macro token source   |
| Performance benchmarks vs graphql_parser crate    | 🔲 TODO      | Part 10                          |
| Fuzz testing (cargo-fuzz)                         | 🔲 TODO      | Security-critical code           |

### Dependencies

**Completed Prerequisite:** ✅ Token types have been refactored to use
`Cow<'src, str>` (Step 0 of Part 2.5). This enables zero-copy lexing while
maintaining compatibility with `RustMacroGraphQLTokenSource`.

Parser development can now proceed using either `RustMacroGraphQLTokenSource`
or `StrGraphQLTokenSource` for testing.

---

## Part 1: Type Renames and Relocations ✅ COMPLETED

### Rename: `GraphQLTokenSpan` → `GraphQLSourceSpan`

**Rationale:** The span type represents a region of source code, not
specifically a token. It's used for:
- Token spans
- Parse error spans
- AST node location tracking

The name `GraphQLSourceSpan` better reflects this general purpose.

**Location Change:** Move from `libgraphql_parser::token::GraphQLTokenSpan` to
`libgraphql_parser::GraphQLSourceSpan` (crate root).

**File Changes:**
1. Rename `src/token/graphql_token_span.rs` → `src/graphql_source_span.rs`
2. Update `src/lib.rs` to export `GraphQLSourceSpan`
3. Update `src/token/mod.rs` to remove the export
4. Update all references in:
   - `src/token/graphql_token.rs`
   - `src/token/graphql_trivia_token.rs`
   - `crates/libgraphql-macros/src/rust_macro_graphql_token_source.rs`
   - Test files

---

## Part 2: Parse Error Design ✅ COMPLETED

**Status:** All error types described in this section have been implemented.

### Design Goals

The error system is designed to:
1. **Provide helpful messages** — Errors should guide users toward fixing issues
2. **Support related locations** — Primary span plus related locations (e.g., where
   an unclosed delimiter was opened)
3. **Include contextual notes** — "Did you mean?" hints, explanations, suggestions
4. **Enable programmatic handling** — Error kinds allow tools to categorize and
   respond to errors
5. **Integrate with existing infrastructure** — Reuse `GraphQLErrorNotes` from the
   lexer layer for consistency
6. **Follow Rust conventions** — Implement `std::error::Error` for interoperability

---

### Update to `GraphQLSourceSpan`

Add file path information to enable multi-file error reporting:

```rust
// In: /crates/libgraphql-parser/src/graphql_source_span.rs

use crate::SourcePosition;
use std::path::PathBuf;

/// Represents the span of source text from start to end position.
///
/// The span is a half-open interval: `[start_inclusive, end_exclusive)`.
/// - `start_inclusive`: Position of the first character of the source text
/// - `end_exclusive`: Position immediately after the last character
///
/// Optionally includes a file path for multi-file scenarios.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphQLSourceSpan {
    pub start_inclusive: SourcePosition,
    pub end_exclusive: SourcePosition,
    /// The file path this span refers to, if known.
    ///
    /// This is `Some` when parsing from a file, `None` when parsing from an
    /// in-memory string without associated path information.
    pub file_path: Option<PathBuf>,
}
```

**Implementation Note:** This change requires updating all existing construction
sites for `GraphQLSourceSpan`. See Step 2a below for the migration task.

---

### Supporting Types (Crate Root)

These enums are defined at the `libgraphql-parser` crate root for general use:

```rust
// In: /crates/libgraphql-parser/src/definition_kind.rs

/// The kind of definition found in a GraphQL document.
///
/// Used for error reporting and programmatic categorization of definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionKind {
    /// `schema { ... }` or `extend schema { ... }`
    Schema,

    /// Type definitions: `type`, `interface`, `union`, `enum`, `scalar`,
    /// `input`, or their `extend` variants.
    TypeDefinition,

    /// `directive @name on ...`
    DirectiveDefinition,

    /// Operations: `query`, `mutation`, `subscription`, or anonymous `{ ... }`
    Operation,

    /// `fragment Name on Type { ... }`
    Fragment,
}
```

```rust
// In: /crates/libgraphql-parser/src/document_kind.rs

/// The kind of GraphQL document being parsed.
///
/// Different document kinds allow different definition types:
/// - Schema documents: only type system definitions
/// - Executable documents: only operations and fragments
/// - Mixed documents: both type system and executable definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    /// Schema document: only type system definitions allowed
    /// (`schema`, `type`, `interface`, `directive`, etc.).
    Schema,

    /// Executable document: only operations and fragments allowed
    /// (`query`, `mutation`, `subscription`, `fragment`).
    Executable,

    /// Mixed document: both type system and executable definitions allowed.
    /// This is useful for tools that process complete GraphQL codebases.
    Mixed,
}
```

```rust
// In: /crates/libgraphql-parser/src/reserved_name_context.rs

/// Contexts where certain names are reserved in GraphQL.
///
/// Some names have special meaning in specific contexts and cannot be used
/// as identifiers there. This enum is used by `GraphQLParseErrorKind::ReservedName`
/// to indicate which context rejected the name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReservedNameContext {
    /// Fragment names cannot be `on` (it introduces the type condition).
    ///
    /// Invalid: `fragment on on User { ... }`
    /// The first `on` would be parsed as the fragment name, but `on` is
    /// reserved in this context.
    FragmentName,

    /// Enum values cannot be `true`, `false`, or `null`.
    ///
    /// Invalid: `enum Bool { true false }` or `enum Maybe { null some }`
    /// These would be ambiguous with boolean/null literals in value contexts.
    EnumValue,
}
```

---

### Error Note Types

The note system categorizes notes by their purpose, enabling appropriate
rendering in different output contexts (CLI, IDE, JSON):

```rust
// In: /crates/libgraphql-parser/src/graphql_error_note_kind.rs

/// The kind of an error note, determining how it is rendered.
///
/// Notes provide additional context beyond the primary error message.
/// Different kinds are rendered with different prefixes in CLI output
/// and may be handled differently by IDEs or other tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphQLErrorNoteKind {
    /// General context or explanation about the error.
    ///
    /// Rendered as `= note: ...` in CLI output.
    /// Example: "Opening `{` here" (with span pointing to the opener)
    General,

    /// Actionable suggestion for fixing the error.
    ///
    /// Rendered as `= help: ...` in CLI output.
    /// Example: "Did you mean: `userName: String`?"
    Help,

    /// Reference to the GraphQL specification.
    ///
    /// Rendered as `= spec: ...` in CLI output.
    /// Example: "https://spec.graphql.org/September2025/#FieldDefinition"
    Spec,
}
```

```rust
// In: /crates/libgraphql-parser/src/graphql_error_note.rs

use crate::GraphQLErrorNoteKind;
use crate::GraphQLSourceSpan;
use crate::SmallVec;

/// An error note providing additional context about an error.
///
/// Notes augment the primary error message with:
/// - Explanatory context (why the error occurred)
/// - Actionable suggestions (how to fix it)
/// - Specification references (where to learn more)
/// - Related source locations (e.g., where a delimiter was opened)
#[derive(Debug, Clone, PartialEq)]
pub struct GraphQLErrorNote {
    /// The kind of note (determines rendering prefix).
    pub kind: GraphQLErrorNoteKind,

    /// The note message.
    pub message: String,

    /// Optional span pointing to a related location.
    ///
    /// When present, the note is rendered with a source snippet
    /// pointing to this location.
    pub span: Option<GraphQLSourceSpan>,
}

impl GraphQLErrorNote {
    /// Creates a general note without a span.
    pub fn general(message: impl Into<String>) -> Self {
        Self {
            kind: GraphQLErrorNoteKind::General,
            message: message.into(),
            span: None,
        }
    }

    /// Creates a general note with a span.
    pub fn general_with_span(message: impl Into<String>, span: GraphQLSourceSpan) -> Self {
        Self {
            kind: GraphQLErrorNoteKind::General,
            message: message.into(),
            span: Some(span),
        }
    }

    /// Creates a help note without a span.
    pub fn help(message: impl Into<String>) -> Self {
        Self {
            kind: GraphQLErrorNoteKind::Help,
            message: message.into(),
            span: None,
        }
    }

    /// Creates a help note with a span.
    pub fn help_with_span(message: impl Into<String>, span: GraphQLSourceSpan) -> Self {
        Self {
            kind: GraphQLErrorNoteKind::Help,
            message: message.into(),
            span: Some(span),
        }
    }

    /// Creates a spec reference note.
    pub fn spec(url: impl Into<String>) -> Self {
        Self {
            kind: GraphQLErrorNoteKind::Spec,
            message: url.into(),
            span: None,
        }
    }
}

/// Type alias for error notes.
///
/// Uses SmallVec since most errors have 0-2 notes, avoiding heap
/// allocation in the common case.
pub type GraphQLErrorNotes = SmallVec<[GraphQLErrorNote; 2]>;
```

---

### `GraphQLParseError`

```rust
// In: /crates/libgraphql-parser/src/graphql_parse_error.rs

use crate::GraphQLErrorNotes;
use crate::GraphQLSourceSpan;

/// A parse error with location information and contextual notes.
///
/// This structure provides comprehensive error information for both
/// human-readable CLI output and programmatic handling by tools.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct GraphQLParseError {
    /// Human-readable primary error message.
    ///
    /// This is the main error description shown to users.
    /// Examples: "Expected `:` after field name", "Unclosed `{`"
    message: String,

    /// The primary span where the error was detected.
    ///
    /// This location is highlighted as the main error site in CLI output.
    /// - For "unexpected token" errors: the unexpected token's span
    /// - For "expected X" errors: where X should have appeared
    /// - For "unclosed delimiter" errors: the position where closing was expected
    span: GraphQLSourceSpan,

    /// Categorized error kind for programmatic handling.
    ///
    /// Enables tools to pattern-match on error types without parsing messages.
    kind: GraphQLParseErrorKind,

    /// Additional notes providing context, suggestions, and related locations.
    ///
    /// Each note has a kind (General, Help, Spec), message, and optional span:
    /// - With span: Points to a related location (e.g., "opening `{` here")
    /// - Without span: General advice not tied to a specific location
    ///
    /// Uses `GraphQLErrorNotes` for consistency with lexer errors.
    notes: GraphQLErrorNotes,
}

impl GraphQLParseError {
    /// Creates a new parse error with no notes.
    pub fn new(
        message: impl Into<String>,
        span: GraphQLSourceSpan,
        kind: GraphQLParseErrorKind,
    ) -> Self {
        Self {
            message: message.into(),
            span,
            kind,
            notes: GraphQLErrorNotes::new(),
        }
    }

    /// Creates a new parse error with notes.
    pub fn with_notes(
        message: impl Into<String>,
        span: GraphQLSourceSpan,
        kind: GraphQLParseErrorKind,
        notes: GraphQLErrorNotes,
    ) -> Self {
        Self {
            message: message.into(),
            span,
            kind,
            notes,
        }
    }

    /// Creates a parse error from a lexer error token.
    ///
    /// When the parser encounters a `GraphQLTokenKind::Error` token, this
    /// method converts it to a `GraphQLParseError`, preserving the lexer's
    /// message and notes.
    pub fn from_lexer_error(
        message: impl Into<String>,
        span: GraphQLSourceSpan,
        lexer_notes: GraphQLErrorNotes,
    ) -> Self {
        Self {
            message: message.into(),
            span,
            kind: GraphQLParseErrorKind::LexerError,
            notes: lexer_notes,
        }
    }

    pub fn message(&self) -> &str { &self.message }
    pub fn span(&self) -> &GraphQLSourceSpan { &self.span }
    pub fn kind(&self) -> &GraphQLParseErrorKind { &self.kind }
    pub fn notes(&self) -> &GraphQLErrorNotes { &self.notes }

    /// Adds a general note without a span.
    pub fn add_note(&mut self, message: impl Into<String>) {
        self.notes.push(GraphQLErrorNote::general(message));
    }

    /// Adds a general note with a span (pointing to a related location).
    pub fn add_note_with_span(
        &mut self,
        message: impl Into<String>,
        span: GraphQLSourceSpan,
    ) {
        self.notes.push(GraphQLErrorNote::general_with_span(message, span));
    }

    /// Adds a help note without a span.
    pub fn add_help(&mut self, message: impl Into<String>) {
        self.notes.push(GraphQLErrorNote::help(message));
    }

    /// Adds a help note with a span.
    pub fn add_help_with_span(
        &mut self,
        message: impl Into<String>,
        span: GraphQLSourceSpan,
    ) {
        self.notes.push(GraphQLErrorNote::help_with_span(message, span));
    }

    /// Adds a spec reference note.
    pub fn add_spec(&mut self, url: impl Into<String>) {
        self.notes.push(GraphQLErrorNote::spec(url));
    }
}
```

---

### `GraphQLParseErrorKind`

```rust
// In: /crates/libgraphql-parser/src/graphql_parse_error_kind.rs

use crate::DefinitionKind;
use crate::DocumentKind;
use crate::ReservedNameContext;

/// Categorizes parse errors for programmatic handling.
///
/// Each variant contains minimal data needed for programmatic decisions.
/// Human-readable context (suggestions, explanations) belongs in the
/// `notes` field of `GraphQLParseError`.
///
/// The `#[error(...)]` messages are concise/programmatic. Full human-readable
/// messages are in `GraphQLParseError.message`.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GraphQLParseErrorKind {
    /// Expected specific token(s) but found something else.
    ///
    /// This is the most common error type — the parser expected certain tokens
    /// based on grammar rules but encountered something unexpected.
    ///
    /// # Example
    /// ```text
    /// type User { name String }
    ///                  ^^^^^^ expected `:`, found `String`
    /// ```
    #[error("unexpected token")]
    UnexpectedToken {
        /// What tokens were expected (e.g., `[":"​, "{"​, "@"]`).
        expected: Vec<String>,
        /// Description of what was found (e.g., `"String"` or `"}"`).
        found: String,
    },

    /// Unexpected end of input while parsing.
    ///
    /// The document ended before a complete construct was parsed.
    ///
    /// # Example
    /// ```text
    /// type User {
    ///           ^ expected `}`, found end of input
    /// ```
    #[error("unexpected end of input")]
    UnexpectedEof {
        /// What was expected when EOF was encountered.
        expected: Vec<String>,
    },

    /// Lexer error encountered during parsing.
    ///
    /// The parser encountered a `GraphQLTokenKind::Error` token from the lexer.
    /// The lexer's error message and notes are preserved in the parent
    /// `GraphQLParseError`'s `message` and `notes` fields.
    ///
    /// # Example
    /// ```text
    /// type User { name: "unterminated string
    ///                   ^ unterminated string literal
    /// ```
    #[error("lexer error")]
    LexerError,

    /// Unclosed delimiter (bracket, brace, or parenthesis).
    ///
    /// A delimiter was opened but EOF was reached before finding the matching
    /// closing delimiter. The opening location is typically included in the
    /// error's `notes`.
    ///
    /// # Example
    /// ```text
    /// type User {
    ///     name: String
    /// # EOF here — missing `}`
    /// ```
    ///
    /// Note: This is distinct from `MismatchedDelimiter`, which occurs when a
    /// *wrong* closing delimiter is found (e.g., `[` closed with `)`).
    #[error("unclosed delimiter")]
    UnclosedDelimiter {
        /// The unclosed delimiter (e.g., `"{"`, `"["`, `"("`).
        delimiter: String,
    },

    /// Mismatched delimiter.
    ///
    /// A closing delimiter was found that doesn't match the most recently
    /// opened delimiter. This indicates a structural nesting error.
    ///
    /// # Example
    /// ```text
    /// type User { name: [String) }
    ///                         ^ expected `]`, found `)`
    /// ```
    ///
    /// Note: This is distinct from `UnclosedDelimiter`, which occurs when EOF
    /// is reached without any closing delimiter.
    #[error("mismatched delimiter")]
    MismatchedDelimiter {
        /// The expected closing delimiter (e.g., `"]"`).
        expected: String,
        /// The actual closing delimiter found (e.g., `")"`).
        found: String,
    },

    /// Invalid value (wraps value parsing errors).
    ///
    /// Occurs when a literal value (string, int, float) cannot be parsed.
    ///
    /// # Example
    /// ```text
    /// query { field(limit: 99999999999999999999) }
    ///                      ^^^^^^^^^^^^^^^^^^^^ integer overflow
    /// ```
    #[error("invalid value")]
    InvalidValue(ValueParsingError),

    /// Reserved name used in a context where it's not allowed.
    ///
    /// Certain names have special meaning in specific contexts:
    /// - `on` cannot be a fragment name (it introduces type conditions)
    /// - `true`, `false`, `null` cannot be enum values (ambiguous with literals)
    ///
    /// # Example
    /// ```text
    /// fragment on on User { name }
    ///          ^^ fragment name cannot be `on`
    /// ```
    #[error("reserved name")]
    ReservedName {
        /// The reserved name that was used (e.g., `"on"`, `"true"`).
        name: String,
        /// The context where this name is not allowed.
        context: ReservedNameContext,
    },

    /// Definition kind not allowed in the document being parsed.
    ///
    /// When parsing with `parse_executable_document()`, schema definitions
    /// (types, directives) are not allowed. When parsing with
    /// `parse_schema_document()`, operations and fragments are not allowed.
    ///
    /// # Example
    /// ```text
    /// # Parsing as executable document:
    /// type User { name: String }
    /// ^^^^ type definition not allowed in executable document
    /// ```
    #[error("wrong document kind")]
    WrongDocumentKind {
        /// What kind of definition was found.
        found: DefinitionKind,
        /// What kind of document is being parsed.
        document_kind: DocumentKind,
    },

    /// Empty construct that requires content.
    ///
    /// Certain constructs cannot be empty per the GraphQL spec:
    /// - Selection sets: `{ }` is invalid (must have at least one selection)
    /// - Argument lists: `()` is invalid (omit parentheses if no arguments)
    ///
    /// # Example
    /// ```text
    /// query { user { } }
    ///              ^^^ selection set cannot be empty
    /// ```
    #[error("invalid empty construct")]
    InvalidEmptyConstruct {
        /// What construct is empty (e.g., `"selection set"`).
        construct: String,
    },

    /// Invalid syntax that doesn't fit other categories.
    ///
    /// A catch-all for syntax errors without dedicated variants. The specific
    /// error is described in `GraphQLParseError.message`.
    #[error("invalid syntax")]
    InvalidSyntax,
}
```

---

### `ValueParsingError`

This enum unifies value parsing errors (formerly called "cooking" errors):

```rust
// In: /crates/libgraphql-parser/src/value_parsing_error.rs

use crate::GraphQLStringParsingError;

/// Errors that occur when parsing literal values.
///
/// These errors occur when converting raw token text to semantic values.
/// For example, parsing the integer `9999999999999999999999` overflows i64.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ValueParsingError {
    /// Invalid string literal (bad escape sequence, unterminated, etc.).
    #[error("Invalid GraphQL string: `{0}`")]
    String(#[from] GraphQLStringParsingError),

    /// Invalid integer literal (overflow, invalid format).
    ///
    /// GraphQL integers must fit in a signed 64-bit integer (i64).
    #[error("Invalid GraphQL integer: `{0}`")]
    Int(String),

    /// Invalid float literal (infinity, NaN, invalid format).
    ///
    /// GraphQL floats must be finite f64 values.
    #[error("Invalid GraphQL float: `{0}`")]
    Float(String),
}
```

---

### Error Formatting

The `Display` implementation (`#[error("{message}")]`) provides a simple string
representation suitable for logging and the `?` operator. For rich diagnostic
output (line numbers, source snippets, colored output), use dedicated formatting
functions.

#### Formatting Functions

```rust
// In: /crates/libgraphql-parser/src/graphql_parse_error.rs

impl GraphQLParseError {
    /// Formats this error as a diagnostic string for CLI output.
    ///
    /// Produces output like:
    /// ```text
    /// error: Expected `:` after field name
    ///   --> schema.graphql:5:12
    ///    |
    ///  5 |     userName String
    ///    |              ^^^^^^ expected `:`
    ///    |
    ///    = note: Field definitions require `:` between name and type
    ///    = help: Did you mean: `userName: String`?
    /// ```
    ///
    /// # Arguments
    /// - `source`: Optional source text for snippet extraction. If `None`,
    ///   snippets are omitted but line/column info is still shown.
    pub fn format_diagnostic(&self, source: Option<&str>) -> String {
        // Implementation details...
    }

    /// Formats this error as a single-line summary.
    ///
    /// Produces output like:
    /// ```text
    /// schema.graphql:5:12: error: Expected `:` after field name
    /// ```
    pub fn format_oneline(&self) -> String {
        // Implementation details...
    }
}
```

#### Future: `DiagnosticFormatter` Trait

For more flexible formatting (JSON, SARIF, IDE integration), a trait-based
approach may be added in the future:

```rust
pub trait DiagnosticFormatter {
    fn format(&self, error: &GraphQLParseError, source: Option<&str>) -> String;
}
```

---

### Design Decisions

**Why a single `span` instead of `Vec<GraphQLSourceSpan>`?**

The original `GraphQLParser` design had `spans: Vec<GraphQLSourceSpan>`, but
this creates ambiguity about which span is "primary" and how multiple spans
relate. Instead:
- The primary error location goes in `span`
- Related locations go in `notes` with explanatory text

This makes relationships explicit and is consistent with how
`RustMacroGraphQLTokenSource` already uses `error_notes`.

**Why add `file_path` to `GraphQLSourceSpan`?**

Multi-file scenarios are common (schema split across files, operations in
separate files). Adding `Option<PathBuf>` to spans allows:
- Notes to point to different files
- Error messages to show which file contains the error
- Consistent file tracking throughout the parsing pipeline

**Why reuse `GraphQLErrorNotes`?**

The lexer already uses `GraphQLErrorNotes` for helpful hints. Reusing this type:
1. Provides consistency across the parsing pipeline
2. Avoids defining redundant types
3. Allows formatting code to handle both lexer and parser errors uniformly

**Why have separate `LexerError` kind without fields?**

When the parser encounters a `GraphQLTokenKind::Error` token, it creates a
`GraphQLParseError` where:
- `message` ← lexer error's `message`
- `span` ← token's span
- `kind` ← `GraphQLParseErrorKind::LexerError` (marker only)
- `notes` ← lexer error's `error_notes`

The `LexerError` kind is a discriminant indicating origin; the actual data
flows into the parent struct, avoiding duplication.

**What belongs in `kind` vs `notes`?**

- `kind`: Data needed for programmatic decisions (matching, categorization)
- `notes`: Human-readable context (suggestions, explanations, related locations)

For example, `UnclosedDelimiter` has `delimiter: String` in the kind, but the
opening span goes in `notes` with text like "opening `{` here".

**Why keep `Display` simple and use separate formatting functions?**

- `Display` is used in contexts expecting brief output (logs, `?` operator)
- Diagnostic formatting needs access to source text (for snippets)
- Different consumers want different formats (CLI vs IDE vs JSON)

**Removed from `GraphQLParseErrorKind`:**

- `DuplicateDefinition` — This is a validation error, not a parse error.
  Duplicate detection belongs in schema/document validators.
- `InvalidDirectiveLocation` — Directive location validation is semantic, not
  syntactic. The parser accepts any `@name` directive; validation checks
  locations.

---

## Part 2.5: StrGraphQLTokenSource Implementation ✅ COMPLETE

This section details the implementation of `StrGraphQLTokenSource`, the
string-based lexer for GraphQL. This is Phase 2 of the overall project plan.

**Status:** Core lexer implementation is complete (~1130 lines, integrated with
parser tests). Steps 0-7 (Cow refactoring through invalid character handling)
are done. Remaining optional work: Steps 8-10 (vendored tests, benchmarks,
fuzzing) for extended validation.

**Full details:** See `str-graphql-token-source-plan.md` for comprehensive
implementation steps, tests, and verification checklist.

---

### Architectural Decision: Zero-Copy with `Cow<'src, str>`

We use `Cow<'src, str>` (Clone-on-Write) for string data in `GraphQLTokenKind`,
enabling:
- **Zero-copy lexing** for `StrGraphQLTokenSource` (borrows from source string)
- **Owned strings** for `RustMacroGraphQLTokenSource` (proc_macro2 requires
  allocation)

This requires adding a lifetime parameter `'src` to `GraphQLTokenKind<'src>`,
`GraphQLToken<'src>`, and related types.

**Why `Cow` instead of `&'src str`?**

`RustMacroGraphQLTokenSource` cannot return borrowed strings because
`proc_macro2::Ident::to_string()` returns an owned `String` — there's no
contiguous source buffer to borrow from. `Cow` allows both borrowed and owned
data in the same type.

**Exception:** `Error.message` uses plain `String` (not `Cow`) because error
messages are always dynamically constructed.

---

### Existing Infrastructure

| Component              | Location                                   | Purpose                              |
|------------------------|--------------------------------------------|--------------------------------------|
| `SourcePosition`       | `src/source_position.rs`                   | Dual-column tracking (UTF-8, UTF-16) |
| `GraphQLSourceSpan`    | `src/graphql_source_span.rs`               | Start/end positions with file path   |
| `GraphQLToken`         | `src/token/graphql_token.rs`               | Token with kind, trivia, span        |
| `GraphQLTokenKind`     | `src/token/graphql_token_kind.rs`          | 14 punctuators, literals, Eof, Error |
| `GraphQLTriviaToken`   | `src/token/graphql_trivia_token.rs`        | Comment and Comma variants           |
| `GraphQLTokenSource`   | `src/token_source/graphql_token_source.rs` | Marker trait for token iterators     |
| `parse_string_value()` | `src/token/graphql_token_kind.rs`          | String escape processing             |

---

### Reference: RustMacroGraphQLTokenSource

`RustMacroGraphQLTokenSource` in `libgraphql-macros` (~890 lines) provides
patterns for:
- State machine for `.` → `..` → `...` handling
- Block string detection/combination
- Negative number handling (`-17` → single token)
- Error generation with helpful notes
- Trivia (comma) accumulation

### Key Differences

| Aspect           | RustMacroGraphQLTokenSource        | StrGraphQLTokenSource             |
|------------------|------------------------------------|-----------------------------------|
| Input            | `proc_macro2::TokenStream`         | `&'src str`                       |
| UTF-16 column    | `None` (unavailable)               | `Some(value)` (computed)          |
| Comments         | Cannot preserve (Rust strips them) | Preserves `#` comments            |
| Dot handling     | Rust tokenizes separately          | Direct scanning                   |
| Negative numbers | Combine `-` + number               | Direct scan as single token       |

---

### Step 0: Refactor GraphQLTokenKind to Use `Cow<'src, str>`

**Files to modify:**
- `src/token/graphql_token_kind.rs`
- `src/token/graphql_token.rs`
- `src/token/graphql_trivia_token.rs`
- `src/token/mod.rs`
- `src/graphql_token_stream.rs`
- `src/token_source/graphql_token_source.rs`
- `libgraphql-macros/src/rust_macro_graphql_token_source.rs`

**Tasks:**

1. Update `GraphQLTokenKind` to use `Cow<'src, str>`:
   ```rust
   use std::borrow::Cow;

   #[derive(Clone, Debug, PartialEq)]
   pub enum GraphQLTokenKind<'src> {
       // Punctuators unchanged (no string data)
       Ampersand, At, /* ... */

       // String-carrying variants now use Cow
       Name(Cow<'src, str>),
       IntValue(Cow<'src, str>),
       FloatValue(Cow<'src, str>),
       StringValue(Cow<'src, str>),

       // Error uses plain String (always dynamically constructed)
       Error { message: String, error_notes: GraphQLErrorNotes },

       // Others unchanged
       True, False, Null, Eof,
   }
   ```

2. Update `GraphQLToken<'src>`, `GraphQLTriviaToken<'src>`,
   `GraphQLTokenSource<'src>`, and `GraphQLTokenStream<'src, S>` to carry the
   lifetime.

3. Add helper constructors:
   ```rust
   impl<'src> GraphQLTokenKind<'src> {
       pub fn name_borrowed(s: &'src str) -> Self { ... }
       pub fn name_owned(s: String) -> Self { ... }
       pub fn int_value_borrowed(s: &'src str) -> Self { ... }
       pub fn int_value_owned(s: String) -> Self { ... }
       pub fn float_value_borrowed(s: &'src str) -> Self { ... }
       pub fn float_value_owned(s: String) -> Self { ... }
       pub fn string_value_borrowed(s: &'src str) -> Self { ... }
       pub fn string_value_owned(s: String) -> Self { ... }
       pub fn error(msg: impl Into<String>, notes: GraphQLErrorNotes) -> Self { ... }
   }
   ```

4. Update `RustMacroGraphQLTokenSource` to use helper constructors.

---

### Step 1: Create Lexer Skeleton

**File:** `src/token_source/str_to_graphql_token_source.rs`

**Struct definition:**
```rust
pub struct StrGraphQLTokenSource<'src> {
    source: &'src str,              // Full source (for scanning and borrowing)
    curr_byte_offset: usize,        // Use &source[curr_byte_offset..] for remaining
    curr_line: usize,               // Current 0-based line
    curr_col_utf8: usize,           // Current UTF-8 character column
    curr_col_utf16: usize,          // Current UTF-16 code unit column
    curr_last_was_cr: bool,         // For \r\n handling
    pending_trivia: GraphQLTriviaTokenVec<'src>,
    finished: bool,                 // Has EOF been emitted
    file_path: Option<&'src Path>,  // Optional file path (borrowed)
}
```

**Position helpers:**
```rust
fn remaining(&self) -> &'src str      // &source[curr_byte_offset..]
fn curr_position(&self) -> SourcePosition
fn peek_char(&self) -> Option<char>   // self.remaining().chars().next()
fn peek_char_nth(&self, n: usize) -> Option<char>
fn consume(&mut self) -> Option<char> // Handles line/column tracking
fn make_span(&self, start: SourcePosition) -> GraphQLSourceSpan
```

---

### Steps 2–7: Lexer Implementation Summary

| Step | Topic                    | Key Points                                                                                       |
|------|--------------------------|--------------------------------------------------------------------------------------------------|
| 2    | Whitespace & Punctuators | Space, tab, newlines, BOM; 13 single-char punctuators                                            |
| 3    | Comments & Ellipsis      | `#` comments as trivia; `...` and dot error handling                                             |
| 4    | Names & Keywords         | `/[_A-Za-z][_0-9A-Za-z]*/`; `true`/`false`/`null`                                                |
| 5    | Numeric Literals         | Int and float with negative sign handling; **security-critical parsing, test exhaustively**      |
| 6    | String Literals          | Single-line and block strings; escape handling; **security-critical parsing, test exhaustively** |
| 7    | Invalid Characters       | `describe_char()` with Unicode names for invisibles                                              |

---

### Steps 8–10: Testing & Performance

| Step | Topic          | Key Points                                          |
|------|----------------|-----------------------------------------------------|
| 8    | Test Suite     | Unit tests, position tracking, error recovery       |
| 9    | Vendored Tests | Port from graphql-js (MIT) and graphql-parser (MIT) |
| 10   | Performance    | Benchmarks vs graphql_parser crate; cargo-fuzz      |

---

### Critical Files Summary

| File                                                       | Changes                                  |
|------------------------------------------------------------|------------------------------------------|
| `src/token/graphql_token_kind.rs`                          | Add `'src`, `Cow`, helper constructors   |
| `src/token/graphql_token.rs`                               | Add `'src` lifetime                      |
| `src/token/graphql_trivia_token.rs`                        | Add `'src`, Comment uses `Cow`           |
| `src/graphql_token_stream.rs`                              | Add `'src` lifetime                      |
| `src/token_source/graphql_token_source.rs`                 | Update trait with lifetime               |
| `src/token_source/str_to_graphql_token_source.rs`          | **Main implementation** (~500-700 lines) |
| `libgraphql-macros/src/rust_macro_graphql_token_source.rs` | Use helper constructors                  |

---

## Part 3: Parser Architecture ✅ COMPLETE

**Status:** Fully implemented in `src/graphql_parser.rs` (~3200 lines) with
`ParseResult<T>` in `src/parse_result.rs` (~200 lines). All three document
parsing methods (`parse_schema_document`, `parse_executable_document`,
`parse_mixed_document`) are functional with error recovery. Tests in
`src/tests/graphql_parser_tests.rs` (~2200 lines, 175+ tests).

### Design Principles

1. **Recursive Descent:** Simple, predictable, easy to debug
2. **Generic over Token Source:** `GraphQLParser<S: GraphQLTokenSource>`
3. **Error Recovery:** Continue parsing after errors to report multiple issues
4. **Spec Compliance:** Follow September 2025 GraphQL spec exactly
5. **Fresh Design:** Build from spec grammar, not by migrating existing code

### Core Parser Structure

```rust
/// A GraphQL parser that can parse schema, executable, or mixed documents.
pub struct GraphQLParser<S: GraphQLTokenSource> {
    tokens: GraphQLTokenStream<S>,
    errors: Vec<GraphQLParseError>,
    /// Stack of currently open delimiters for error recovery.
    /// Enables accurate "opening `{` here" notes in error messages.
    delimiter_stack: Vec<OpenDelimiter>,
}

/// Tracks an opened delimiter for error reporting.
#[derive(Debug, Clone)]
struct OpenDelimiter {
    /// The delimiter character (e.g., `{`, `[`, `(`).
    kind: char,
    /// The span of the opening delimiter.
    span: GraphQLSourceSpan,
    /// What construct opened this delimiter (for error context).
    context: DelimiterContext,
}

/// The syntactic construct that opened a delimiter.
///
/// Used in error messages like "unclosed `{` in type definition".
/// Internal to the parser module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DelimiterContext {
    /// `schema { ... }`
    SchemaDefinition,
    /// `type Foo { ... }`, `interface Bar { ... }`, etc.
    TypeDefinition,
    /// `enum Foo { ... }`
    EnumDefinition,
    /// `input Foo { ... }`
    InputObjectDefinition,
    /// `{ field ... }` in operations
    SelectionSet,
    /// `field(arg: value)`
    Arguments,
    /// `($var: Type)` in operations
    VariableDefinitions,
    /// `[Type]` in type references
    ListType,
    /// `[value, ...]` in list literals
    ListValue,
    /// `{ key: value }` in object literals
    ObjectValue,
    /// `directive @foo on ...` locations wrapped in parens (rare)
    DirectiveLocations,
}

impl<S: GraphQLTokenSource> GraphQLParser<S> {
    pub fn new(token_source: S) -> Self { ... }

    /// Parse a schema document (type definitions only).
    pub fn parse_schema_document(mut self) -> ParseResult<ast::schema::Document>;

    /// Parse an executable document (operations and fragments only).
    pub fn parse_executable_document(mut self) -> ParseResult<ast::operation::Document>;

    /// Parse a mixed document (both schema and executable definitions).
    pub fn parse_mixed_document(mut self) -> ParseResult<MixedDocument>;
}
```

### ParseResult Type

```rust
/// Result of parsing, containing both AST (if successful or partially-successful)
/// and all errors.
///
/// Even when errors occur, a partial AST may be available for tooling
/// (IDE error recovery, formatters that work on broken documents, etc.).
#[derive(Debug, Clone)]
pub struct ParseResult<T> {
    /// The parsed AST, if parsing succeeded or recovered enough to produce one.
    pub ast: Option<T>,
    /// All errors encountered during parsing (may be empty on full success).
    pub errors: Vec<GraphQLParseError>,
}

impl<T> ParseResult<T> {
    /// Returns `true` if parsing completed without errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty() && self.ast.is_some()
    }

    /// Returns `true` if any errors were encountered.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns the AST if available, regardless of errors.
    ///
    /// Useful for tooling that can work with partial/broken ASTs.
    pub fn ast(&self) -> Option<&T> {
        self.ast.as_ref()
    }

    /// Takes ownership of the AST, leaving `None` in its place.
    pub fn take_ast(&mut self) -> Option<T> {
        self.ast.take()
    }

    /// Formats all errors as a combined diagnostic string.
    pub fn format_errors(&self, source: Option<&str>) -> String {
        self.errors
            .iter()
            .map(|e| e.format_detailed(source))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

/// Converts `ParseResult<T>` to standard `Result`, discarding partial AST on error.
///
/// Use this when you need strict success/failure semantics.
impl<T> From<ParseResult<T>> for std::result::Result<T, Vec<GraphQLParseError>> {
    fn from(parse_result: ParseResult<T>) -> Self {
        if parse_result.errors.is_empty() {
            parse_result.ast.ok_or_else(Vec::new)
        } else {
            Err(parse_result.errors)
        }
    }
}
```

---

## Part 4: Grammar Coverage

Based on the GraphQL September 2025 specification, the parser must handle:

### Document Structure
```
Document := Definition+
Definition := ExecutableDefinition | TypeSystemDefinitionOrExtension

ExecutableDefinition := OperationDefinition | FragmentDefinition

TypeSystemDefinition := SchemaDefinition | TypeDefinition | DirectiveDefinition
TypeDefinition := ScalarTypeDefinition | ObjectTypeDefinition |
                  InterfaceTypeDefinition | UnionTypeDefinition |
                  EnumTypeDefinition | InputObjectTypeDefinition

TypeSystemExtension := SchemaExtension | TypeExtension
```

### Operations
```
OperationDefinition := OperationType Name? VariableDefinitions? Directives? SelectionSet
                     | SelectionSet  (shorthand query)
OperationType := "query" | "mutation" | "subscription"

VariableDefinitions := "(" VariableDefinition+ ")"
VariableDefinition := Variable ":" Type DefaultValue? Directives?
Variable := "$" Name
DefaultValue := "=" Value
```

### Selection Sets
```
SelectionSet := "{" Selection+ "}"
Selection := Field | FragmentSpread | InlineFragment

Field := Alias? Name Arguments? Directives? SelectionSet?
Alias := Name ":"
Arguments := "(" Argument+ ")"
Argument := Name ":" Value

FragmentSpread := "..." FragmentName Directives?
InlineFragment := "..." TypeCondition? Directives? SelectionSet
```

### Fragments
```
FragmentDefinition := "fragment" FragmentName TypeCondition Directives? SelectionSet
FragmentName := Name (but not "on")
TypeCondition := "on" NamedType
```

### Values
```
Value := Variable | IntValue | FloatValue | StringValue | BooleanValue |
         NullValue | EnumValue | ListValue | ObjectValue

BooleanValue := "true" | "false"
NullValue := "null"
EnumValue := Name (but not true, false, null)
ListValue := "[" "]" | "[" Value+ "]"
ObjectValue := "{" "}" | "{" ObjectField+ "}"
ObjectField := Name ":" Value
```

### Types
```
Type := NamedType | ListType | NonNullType
NamedType := Name
ListType := "[" Type "]"
NonNullType := NamedType "!" | ListType "!"
```

### Directives
```
Directives := Directive+
Directive := "@" Name Arguments?
```

### Type Definitions
```
SchemaDefinition := Description? "schema" Directives? "{" RootOperationTypeDefinition+ "}"
RootOperationTypeDefinition := OperationType ":" NamedType

ScalarTypeDefinition := Description? "scalar" Name Directives?

ObjectTypeDefinition := Description? "type" Name ImplementsInterfaces?
                        Directives? FieldsDefinition?
FieldsDefinition := "{" FieldDefinition+ "}"
FieldDefinition := Description? Name ArgumentsDefinition? ":" Type Directives?
ArgumentsDefinition := "(" InputValueDefinition+ ")"
InputValueDefinition := Description? Name ":" Type DefaultValue? Directives?

InterfaceTypeDefinition := Description? "interface" Name ImplementsInterfaces?
                           Directives? FieldsDefinition?

UnionTypeDefinition := Description? "union" Name Directives? UnionMemberTypes?
UnionMemberTypes := "=" "|"? NamedType ("|" NamedType)*

EnumTypeDefinition := Description? "enum" Name Directives? EnumValuesDefinition?
EnumValuesDefinition := "{" EnumValueDefinition+ "}"
EnumValueDefinition := Description? EnumValue Directives?

InputObjectTypeDefinition := Description? "input" Name Directives?
                             InputFieldsDefinition?
InputFieldsDefinition := "{" InputValueDefinition+ "}"

DirectiveDefinition := Description? "directive" "@" Name ArgumentsDefinition?
                       "repeatable"? "on" DirectiveLocations
DirectiveLocations := "|"? DirectiveLocation ("|" DirectiveLocation)*
```

### Type Extensions
```
SchemaExtension := "extend" "schema" Directives? "{" RootOperationTypeDefinition+ "}"
                 | "extend" "schema" Directives

ScalarTypeExtension := "extend" "scalar" Name Directives

ObjectTypeExtension := "extend" "type" Name ImplementsInterfaces? Directives? FieldsDefinition
                     | "extend" "type" Name ImplementsInterfaces? Directives
                     | "extend" "type" Name ImplementsInterfaces

InterfaceTypeExtension := (similar patterns)
UnionTypeExtension := (similar patterns)
EnumTypeExtension := (similar patterns)
InputObjectTypeExtension := (similar patterns)
```

---

## Part 5: Parser Methods (Internal)

The parser will be organized into method groups:

### Top-Level Parsing
```rust
fn parse_definition(&mut self) -> Result<Definition, GraphQLParseError>;
fn parse_executable_definition(&mut self) -> Result<ExecutableDefinition, GraphQLParseError>;
fn parse_type_system_definition(&mut self) -> Result<TypeSystemDefinition, GraphQLParseError>;
```

### Operations
```rust
fn parse_operation_definition(&mut self) -> Result<OperationDefinition, GraphQLParseError>;
fn parse_variable_definitions(&mut self) -> Result<Vec<VariableDefinition>, GraphQLParseError>;
fn parse_variable_definition(&mut self) -> Result<VariableDefinition, GraphQLParseError>;
```

### Selection Sets
```rust
fn parse_selection_set(&mut self) -> Result<SelectionSet, GraphQLParseError>;
fn parse_selection(&mut self) -> Result<Selection, GraphQLParseError>;
fn parse_field(&mut self) -> Result<Field, GraphQLParseError>;
fn parse_arguments(&mut self) -> Result<Vec<Argument>, GraphQLParseError>;
```

### Fragments
```rust
fn parse_fragment_definition(&mut self) -> Result<FragmentDefinition, GraphQLParseError>;
fn parse_fragment_spread(&mut self) -> Result<FragmentSpread, GraphQLParseError>;
fn parse_inline_fragment(&mut self) -> Result<InlineFragment, GraphQLParseError>;
fn parse_type_condition(&mut self) -> Result<TypeCondition, GraphQLParseError>;
```

### Values
```rust
fn parse_value(&mut self, const_only: bool) -> Result<Value, GraphQLParseError>;
fn parse_list_value(&mut self, const_only: bool) -> Result<Value, GraphQLParseError>;
fn parse_object_value(&mut self, const_only: bool) -> Result<Value, GraphQLParseError>;
```

### Types
```rust
fn parse_type(&mut self) -> Result<Type, GraphQLParseError>;
fn parse_named_type(&mut self) -> Result<NamedType, GraphQLParseError>;
```

### Directives
```rust
fn parse_directives(&mut self) -> Result<Vec<Directive>, GraphQLParseError>;
fn parse_directive(&mut self) -> Result<Directive, GraphQLParseError>;
```

### Type Definitions
```rust
fn parse_schema_definition(&mut self) -> Result<SchemaDefinition, GraphQLParseError>;
fn parse_scalar_type_definition(&mut self) -> Result<ScalarTypeDefinition, GraphQLParseError>;
fn parse_object_type_definition(&mut self) -> Result<ObjectTypeDefinition, GraphQLParseError>;
fn parse_interface_type_definition(&mut self) -> Result<InterfaceTypeDefinition, GraphQLParseError>;
fn parse_union_type_definition(&mut self) -> Result<UnionTypeDefinition, GraphQLParseError>;
fn parse_enum_type_definition(&mut self) -> Result<EnumTypeDefinition, GraphQLParseError>;
fn parse_input_object_type_definition(&mut self) -> Result<InputObjectTypeDefinition, GraphQLParseError>;
fn parse_directive_definition(&mut self) -> Result<DirectiveDefinition, GraphQLParseError>;
```

### Type Extensions
```rust
fn parse_type_extension(&mut self) -> Result<TypeExtension, GraphQLParseError>;
// ... specific extension methods
```

### Helpers
```rust
fn parse_description(&mut self) -> Option<String>;
fn parse_implements_interfaces(&mut self) -> Result<Vec<NamedType>, GraphQLParseError>;
fn parse_fields_definition(&mut self) -> Result<Vec<FieldDefinition>, GraphQLParseError>;
fn expect(&mut self, kind: &GraphQLTokenKind) -> Result<&GraphQLToken, GraphQLParseError>;
fn expect_name(&mut self) -> Result<String, GraphQLParseError>;
fn expect_keyword(&mut self, keyword: &str) -> Result<(), GraphQLParseError>;
```

---

## Part 6: Error Recovery Strategy

### Recovery Points

The parser recovers at definition boundaries:

1. **Top-level:** On error, skip to next definition keyword
   (`type`, `interface`, `union`, `enum`, `scalar`, `input`, `directive`,
   `schema`, `extend`, `query`, `mutation`, `subscription`, `fragment`, `{`)

2. **Within definitions:** Future enhancement - recover at field/member level

### Recovery Implementation

```rust
fn recover_to_next_definition(&mut self) {
    loop {
        match self.tokens.peek() {
            None | Some(GraphQLToken { kind: GraphQLTokenKind::Eof, .. }) => break,
            Some(t) if self.is_definition_start(&t.kind) => break,
            _ => { self.tokens.consume(); }
        }
    }
}

fn is_definition_start(&self, kind: &GraphQLTokenKind) -> bool {
    matches!(kind,
        GraphQLTokenKind::Name(n) if matches!(n.as_str(),
            "type" | "interface" | "union" | "enum" | "scalar" |
            "input" | "directive" | "schema" | "extend" |
            "query" | "mutation" | "subscription" | "fragment"
        ) | GraphQLTokenKind::CurlyBraceOpen
    )
}
```

---

## Part 7: Implementation Steps

### Step 1: Rename and Relocate Types ✅ COMPLETED
1. ✅ Rename `GraphQLTokenSpan` → `GraphQLSourceSpan`
2. ✅ Move to crate root (`/crates/libgraphql-parser/src/graphql_source_span.rs`)
3. ✅ Update all references
4. ✅ Verify tests pass

### Step 1a: Add `file_path` to `GraphQLSourceSpan` ✅ COMPLETED

✅ All tasks completed. `GraphQLSourceSpan` now includes `file_path: Option<PathBuf>`
with `new()` and `with_file()` constructors.

### Step 1b: Rename "Cook" Terminology to "Parse" ✅ COMPLETED

✅ All renames completed:
- `CookGraphQLStringError` → `GraphQLStringParsingError`
- `cook_int_value()` → `parse_int_value()`
- `cook_float_value()` → `parse_float_value()`
- `cook_string_value()` → `parse_string_value()`

### Step 2: Create Error Note Types ✅ COMPLETED

✅ All types created:
- `GraphQLErrorNoteKind` (General, Help, Spec)
- `GraphQLErrorNote` with factory methods
- `GraphQLErrorNotes` type alias

### Step 2a: Update Existing Lexer to Use New Note Types ✅ COMPLETED

✅ `RustMacroGraphQLTokenSource` updated to use `GraphQLErrorNote` structure.

### Step 2b: Create Parse Error Types ✅ COMPLETED

✅ All types created:
- `DefinitionKind`, `DocumentKind`, `ReservedNameContext`
- `ValueParsingError`
- `GraphQLParseErrorKind` (10 variants)
- `GraphQLParseError` with `format_detailed()` and `format_oneline()`

---

### Step 3: Create Parser Skeleton ✅ COMPLETED
1. ✅ Created `graphql_parser.rs` with generic structure (~3200 lines)
2. ✅ Implemented `parse_schema_document()`
3. ✅ Implemented `parse_executable_document()`
4. ✅ Implemented `parse_mixed_document()`

### Step 4: Implement Value Parsing ✅ COMPLETED
1. ✅ Implemented `parse_value()` with `ConstContext` enum (not `ValueContext`)
2. ✅ Handle all value types per spec
3. ✅ Comprehensive tests

### Step 5: Implement Type Parsing ✅ COMPLETED
1. ✅ Implemented `parse_type_annotation()` and variants
2. ✅ Handle list, non-null wrapping
3. ✅ Tests included

### Step 6: Implement Directive Parsing ✅ COMPLETED
1. ✅ Implemented `parse_directive_annotations()` and `parse_directive_annotation()`
2. ✅ Added const variants for schema contexts
3. ✅ Tests included

### Step 7: Implement Selection Set Parsing ✅ COMPLETED
1. ✅ Implemented `parse_selection_set()` and related methods
2. ✅ Handle fields, fragment spreads, inline fragments
3. ✅ Uses `delimiter_stack` for tracking `{` openers
4. ✅ Tests included

### Step 8: Implement Operation Parsing ✅ COMPLETED
1. ✅ Implemented `parse_operation_definition()`
2. ✅ Handle variable definitions
3. ✅ Tests included

### Step 9: Implement Fragment Parsing ✅ COMPLETED
1. ✅ Implemented `parse_fragment_definition()`
2. ✅ Enforce `on` reserved name restriction
3. ✅ Tests included

### Step 10: Implement Type Definition Parsing ✅ COMPLETED
1. ✅ Implement all type definition methods
2. ✅ Handle descriptions, directives, implements
3. ✅ Tests for each type

### Step 11: Implement Type Extension Parsing ✅ COMPLETED
1. ✅ Implement all extension methods
2. ✅ Tests included

### Step 12: Complete Document Parsing ✅ COMPLETED
1. ✅ Wire up all methods in `parse_*_document()`
2. ✅ Implement error recovery with `delimiter_stack`
3. ✅ Integration tests included

### Step 13: Port and Vendor Tests 🔲 TODO (deferred)
1. 🔲 Port tests from graphql-js (license verified: MIT)
2. 🔲 Port tests from graphql-parser (license verified: MIT/Apache-2.0)
3. 🔲 Add differential testing against graphql_parser crate

---

## Part 8: Test Strategy

### Unit Tests
- Each parsing method has dedicated tests
- Both success and error cases
- Edge cases from spec examples

### Integration Tests
- Complete documents of each type
- Mixed documents
- Error recovery scenarios

### Vendored Tests
- graphql-js lexer/parser tests (license permitting)
- graphql-parser tests (license permitting)

### Differential Tests
- Compare output with `graphql_parser` crate
- Document any intentional differences

---

## Part 9: Parsing Error Catalog (Non-Comprehensive)

This section enumerates a representative set of parsing errors to establish
conventions and patterns for error construction. This is **not** an exhaustive
list — the parser may emit additional errors not enumerated here. The purpose
is to explore the shape and conventions we want to build into our parsing
errors.

For each error, we specify:
- **Kind**: Which `GraphQLParseErrorKind` variant
- **Message**: Human-readable error message
- **Notes**: Bulleted list of notes to include (suggestions, related locations,
  spec links)

---

### Category 1: Lexer Errors (Pass-Through)

These errors originate from the lexer (`GraphQLTokenKind::Error`) and are
passed through to the parser error system.

| Error                   | Kind         | Message                             | Notes                                        |
|-------------------------|--------------|-------------------------------------|----------------------------------------------|
| Invalid character       | `LexerError` | "Unexpected character `{char}`"     | <ul>                                         |
|                         |              |                                     | <li>**Note:** "GraphQL allows: A-Z, a-z, 0-9, _, and specific punctuation"</li> |
|                         |              |                                     | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Punctuators</li> |
|                         |              |                                     | </ul>                                        |
|-------------------------|--------------|-------------------------------------|----------------------------------------------|
| Unterminated string     | `LexerError` | "Unterminated string literal"       | <ul>                                         |
|                         |              |                                     | <li>**Note:** "String started here" (span to opening quote)</li> |
|                         |              |                                     | <li>**Help:** "Add closing `\"`"</li>        |
|                         |              |                                     | <li>**Spec:** https://spec.graphql.org/September2025/#sec-String-Value</li> |
|                         |              |                                     | </ul>                                        |
|-------------------------|--------------|-------------------------------------|----------------------------------------------|
| Unterminated block str  | `LexerError` | "Unterminated block string"         | <ul>                                         |
|                         |              |                                     | <li>**Note:** "Block string started here" (span to opening `"""`)</li> |
|                         |              |                                     | <li>**Help:** "Add closing `\"\"\"`"</li>    |
|                         |              |                                     | <li>**Spec:** https://spec.graphql.org/September2025/#sec-String-Value</li> |
|                         |              |                                     | </ul>                                        |
|-------------------------|--------------|-------------------------------------|----------------------------------------------|
| Invalid escape sequence | `LexerError` | "Invalid escape sequence `\\{seq}`" | <ul>                                         |
|                         |              |                                     | <li>**Note:** "Valid escapes: `\\n`, `\\r`, `\\t`, `\\\\`, `\\\"`, `\\/`, `\\b`, `\\f`, `\\uXXXX`, `\\u{...}`"</li> |
|                         |              |                                     | <li>**Spec:** https://spec.graphql.org/September2025/#EscapedCharacter</li> |
|                         |              |                                     | </ul>                                        |
|-------------------------|--------------|-------------------------------------|----------------------------------------------|
| Invalid unicode escape  | `LexerError` | "Invalid unicode escape `\\u{...}`" | <ul>                                         |
|                         |              |                                     | <li>**Note:** "Unicode escapes must be valid code points (0-10FFFF)"</li> |
|                         |              |                                     | <li>**Spec:** https://spec.graphql.org/September2025/#EscapedUnicode</li> |
|                         |              |                                     | </ul>                                        |
|-------------------------|--------------|-------------------------------------|----------------------------------------------|
| Invalid number format   | `LexerError` | "Invalid number `{text}`"           | <ul>                                         |
|                         |              |                                     | <li>**Note:** Context-specific (e.g., "Leading zeros not allowed")</li> |
|                         |              |                                     | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Int-Value</li> |
|                         |              |                                     | </ul>                                        |
|-------------------------|--------------|-------------------------------------|----------------------------------------------|
| Single dot              | `LexerError` | "Unexpected `.`"                    | <ul>                                         |
|                         |              |                                     | <li>**Help:** "Did you mean `...` (spread operator)?" — **only include if the `.` is adjacent to or on the same line as other `.` tokens; omit if surrounded by non-`.` tokens**</li> |
|                         |              |                                     | </ul>                                        |
|-------------------------|--------------|-------------------------------------|----------------------------------------------|
| Spaced dots             | `LexerError` | "Unexpected `. .`"                  | <ul>                                         |
|                         |              |                                     | <li>**Help:** "Remove spacing to form `...` spread operator"</li> |
|                         |              |                                     | <li>**Spec:** https://spec.graphql.org/September2025/#Punctuator</li> |
|                         |              |                                     | </ul>                                        |

---

### Category 2: Token Expectation Errors

These errors occur when the parser expects specific tokens based on grammar
rules.

| Context                         | Kind              | Message                                | Notes                                     |
|---------------------------------|-------------------|----------------------------------------|-------------------------------------------|
| Field definition missing `:`    | `UnexpectedToken` | "Expected `:` after field name"        | <ul>                                      |
|                                 |                   |                                        | <li>**Note:** "Field definitions: `name: Type`"</li> |
|                                 |                   |                                        | <li>**Help (conditional):** If next token is `Name` on same line: "Did you mean: `{prevName}: {nextName}`?"</li> |
|                                 |                   |                                        | <li>**Spec:** https://spec.graphql.org/September2025/#FieldDefinition</li> |
|                                 |                   |                                        | </ul>                                     |
|---------------------------------|-------------------|----------------------------------------|-------------------------------------------|
| Argument missing `:`            | `UnexpectedToken` | "Expected `:` after argument name"     | <ul>                                      |
|                                 |                   |                                        | <li>**Note:** "Arguments: `name: value`"</li> |
|                                 |                   |                                        | <li>**Help (conditional):** If next token is `Name` on same line: "Did you mean: `{prevName}: {nextName}`?"</li> |
|                                 |                   |                                        | <li>**Spec:** https://spec.graphql.org/September2025/#Argument</li> |
|                                 |                   |                                        | </ul>                                     |
|---------------------------------|-------------------|----------------------------------------|-------------------------------------------|
| Variable definition missing `:` | `UnexpectedToken` | "Expected `:` after variable"          | <ul>                                      |
|                                 |                   |                                        | <li>**Note:** "Variable definitions: `$name: Type`"</li> |
|                                 |                   |                                        | <li>**Help (conditional):** If next token is `Name` on same line: "Did you mean: `${prevName}: {nextName}`?"</li> |
|                                 |                   |                                        | <li>**Spec:** https://spec.graphql.org/September2025/#VariableDefinition</li> |
|                                 |                   |                                        | </ul>                                     |
|---------------------------------|-------------------|----------------------------------------|-------------------------------------------|
| Missing type condition          | `UnexpectedToken` | "Expected `on` for type condition"     | <ul>                                      |
|                                 |                   |                                        | <li>**Note (context-specific):** For inline fragments: "Inline fragments: `... on Type { }`". For fragment definitions: "Fragment definitions: `fragment Name on Type { }`"</li> |
|                                 |                   |                                        | <li>**Spec:** https://spec.graphql.org/September2025/#TypeCondition</li> |
|                                 |                   |                                        | </ul>                                     |
|---------------------------------|-------------------|----------------------------------------|-------------------------------------------|
| Missing directive `@`           | `UnexpectedToken` | "Expected `@` before directive name"   | <ul>                                      |
|                                 |                   |                                        | <li>**Note:** "Directives start with `@`"</li> |
|                                 |                   |                                        | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Language.Directives</li> |
|                                 |                   |                                        | </ul>                                     |
|---------------------------------|-------------------|----------------------------------------|-------------------------------------------|
| Missing selection set `{`       | `UnexpectedToken` | "Expected `{` to start selection set"  | <ul>                                      |
|                                 |                   |                                        | <li>**Spec:** https://spec.graphql.org/September2025/#SelectionSet</li> |
|                                 |                   |                                        | </ul>                                     |
|---------------------------------|-------------------|----------------------------------------|-------------------------------------------|
| Missing argument list `(`       | `UnexpectedToken` | "Expected `(` for arguments"           | <ul>                                      |
|                                 |                   |                                        | <li>**Spec:** https://spec.graphql.org/September2025/#Arguments</li> |
|                                 |                   |                                        | </ul>                                     |
|---------------------------------|-------------------|----------------------------------------|-------------------------------------------|
| General unexpected              | `UnexpectedToken` | "Expected {expected}, found `{found}`" | <ul>                                      |
|                                 |                   |                                        | <li>(Context-dependent)</li>              |
|                                 |                   |                                        | </ul>                                     |

---

### Category 3: Delimiter Errors

**Best-Effort Closing Location Detection:** For all `UnclosedDelimiter` errors,
the parser should attempt to identify where the missing delimiter was probably
intended to go based on:
- Indentation levels (a line with less indentation than the opener likely
  indicates where the block should have ended)
- Surrounding same-line context (tokens that typically end constructs)

If a probable location can be identified with reasonable confidence, include a
note with a span suggesting: "Consider adding `}` here".

| Scenario             | Kind                  | Message                   | Notes                                     |
|----------------------|-----------------------|---------------------------|-------------------------------------------|
| Unclosed `{` at EOF  | `UnclosedDelimiter`   | "Unclosed `{`"            | <ul>                                      |
|                      |                       |                           | <li>**Note:** "Opening `{` here" (span to opener)</li> |
|                      |                       |                           | <li>**Help (conditional):** If location detected: "Consider adding `}` here" (span to suggested location)</li> |
|                      |                       |                           | </ul>                                     |
|----------------------|-----------------------|---------------------------|-------------------------------------------|
| Unclosed `[` at EOF  | `UnclosedDelimiter`   | "Unclosed `[`"            | <ul>                                      |
|                      |                       |                           | <li>**Note:** "Opening `[` here" (span to opener)</li> |
|                      |                       |                           | <li>**Help (conditional):** If location detected: "Consider adding `]` here" (span to suggested location)</li> |
|                      |                       |                           | </ul>                                     |
|----------------------|-----------------------|---------------------------|-------------------------------------------|
| Unclosed `(` at EOF  | `UnclosedDelimiter`   | "Unclosed `(`"            | <ul>                                      |
|                      |                       |                           | <li>**Note:** "Opening `(` here" (span to opener)</li> |
|                      |                       |                           | <li>**Help (conditional):** If location detected: "Consider adding `)` here" (span to suggested location)</li> |
|                      |                       |                           | </ul>                                     |
|----------------------|-----------------------|---------------------------|-------------------------------------------|
| `{` closed with `)`  | `MismatchedDelimiter` | "Expected `}`, found `)`" | <ul>                                      |
|                      |                       |                           | <li>**Note:** "Opening `{` here" (span to opener)</li> |
|                      |                       |                           | </ul>                                     |
|----------------------|-----------------------|---------------------------|-------------------------------------------|
| `[` closed with `}`  | `MismatchedDelimiter` | "Expected `]`, found `}`" | <ul>                                      |
|                      |                       |                           | <li>**Note:** "Opening `[` here" (span to opener)</li> |
|                      |                       |                           | </ul>                                     |
|----------------------|-----------------------|---------------------------|-------------------------------------------|
| `(` closed with `]`  | `MismatchedDelimiter` | "Expected `)`, found `]`" | <ul>                                      |
|                      |                       |                           | <li>**Note:** "Opening `(` here" (span to opener)</li> |
|                      |                       |                           | </ul>                                     |

---

### Choosing Between `UnclosedDelimiter` and `UnexpectedEof`

When EOF is reached during parsing, there's often ambiguity about which error
to emit. Use this decision framework:

**Emit `UnclosedDelimiter` when:**
- There is an explicitly opened delimiter (`{`, `[`, `(`) that was never closed
- The parser has context about which delimiter is missing

**Emit `UnexpectedEof` when:**
- No unclosed delimiter exists, but parsing is incomplete
- The parser expected a specific token (not necessarily a delimiter) that wasn't
  found before EOF

**Examples:**

```graphql
type Foo {
  field1: {
    subfield1,
    subfield2,
}
```
→ **`UnclosedDelimiter`** for the inner `{` — there's a clear unclosed
delimiter. Include note: "Opening `{` here" pointing to line 2.

```graphql
type Foo {
```
→ **`UnclosedDelimiter`** for `{` — there's an explicit unclosed delimiter,
even though the definition is also incomplete.

```graphql
type Foo
```
→ **`UnexpectedEof`** with `expected: ["{", "implements", "@"]` — no delimiter
was opened; we just expected more tokens.

```graphql
directive @example on
```
→ **`UnexpectedEof`** with `expected: ["directive location"]` — incomplete
construct but no unclosed delimiter.

**Rule of thumb:** If there's an unclosed delimiter, emit `UnclosedDelimiter`.
Otherwise, emit `UnexpectedEof`.

---

### Category 4: Reserved Name Errors

| Scenario             | Kind           | Message                        | Notes                                     |
|----------------------|----------------|--------------------------------|-------------------------------------------|
| Fragment named `on`  | `ReservedName` | "Fragment name cannot be `on`" | <ul>                                      |
|                      |                |                                | <li>**Note:** "`on` is reserved for type conditions"</li> |
|                      |                |                                | <li>**Spec:** https://spec.graphql.org/September2025/#FragmentName</li> |
|                      |                |                                | </ul>                                     |
|----------------------|----------------|--------------------------------|-------------------------------------------|
| Enum value `true`    | `ReservedName` | "Enum value cannot be `true`"  | <ul>                                      |
|                      |                |                                | <li>**Note:** "Would be ambiguous with boolean literal"</li> |
|                      |                |                                | <li>**Spec:** https://spec.graphql.org/September2025/#EnumValue</li> |
|                      |                |                                | </ul>                                     |
|----------------------|----------------|--------------------------------|-------------------------------------------|
| Enum value `false`   | `ReservedName` | "Enum value cannot be `false`" | <ul>                                      |
|                      |                |                                | <li>**Note:** "Would be ambiguous with boolean literal"</li> |
|                      |                |                                | <li>**Spec:** https://spec.graphql.org/September2025/#EnumValue</li> |
|                      |                |                                | </ul>                                     |
|----------------------|----------------|--------------------------------|-------------------------------------------|
| Enum value `null`    | `ReservedName` | "Enum value cannot be `null`"  | <ul>                                      |
|                      |                |                                | <li>**Note:** "Would be ambiguous with null literal"</li> |
|                      |                |                                | <li>**Spec:** https://spec.graphql.org/September2025/#EnumValue</li> |
|                      |                |                                | </ul>                                     |

---

### Category 5: Value Parsing Errors

**Note:** GraphQL `Int` is specified as a signed 32-bit integer, not 64-bit.
See https://spec.graphql.org/September2025/#sec-Int

| Scenario           | Kind                   | Message                    | Notes                                     |
|--------------------|------------------------|----------------------------|-------------------------------------------|
| Integer overflow   | `InvalidValue(Int)`    | "Integer value too large"  | <ul>                                      |
|                    |                        |                            | <li>**Note:** "Maximum: 2147483647 (i32::MAX)"</li> |
|                    |                        |                            | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Int</li> |
|                    |                        |                            | </ul>                                     |
|--------------------|------------------------|----------------------------|-------------------------------------------|
| Integer underflow  | `InvalidValue(Int)`    | "Integer value too small"  | <ul>                                      |
|                    |                        |                            | <li>**Note:** "Minimum: -2147483648 (i32::MIN)"</li> |
|                    |                        |                            | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Int</li> |
|                    |                        |                            | </ul>                                     |
|--------------------|------------------------|----------------------------|-------------------------------------------|
| Float infinity     | `InvalidValue(Float)`  | "Float value is infinite"  | <ul>                                      |
|                    |                        |                            | <li>**Note:** "Value exceeds f64 range"</li> |
|                    |                        |                            | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Float</li> |
|                    |                        |                            | </ul>                                     |
|--------------------|------------------------|----------------------------|-------------------------------------------|
| Float NaN          | `InvalidValue(Float)`  | "Float value is NaN"       | <ul>                                      |
|                    |                        |                            | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Float</li> |
|                    |                        |                            | </ul>                                     |
|--------------------|------------------------|----------------------------|-------------------------------------------|
| String escape err  | `InvalidValue(String)` | "Invalid string escape"    | <ul>                                      |
|                    |                        |                            | <li>**Note:** (Specific escape error from lexer)</li> |
|                    |                        |                            | <li>**Spec:** https://spec.graphql.org/September2025/#sec-String-Value</li> |
|                    |                        |                            | </ul>                                     |

---

### Category 6: Document Kind Errors

| Scenario                 | Kind                | Message                                                   | Notes                                     |
|--------------------------|---------------------|-----------------------------------------------------------|-------------------------------------------|
| Type in executable doc   | `WrongDocumentKind` | "Type definition not allowed in executable document"      | <ul>                                      |
|                          |                     |                                                           | <li>**Note:** "Executable documents contain only operations and fragments"</li> |
|                          |                     |                                                           | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Executable-Definitions</li> |
|                          |                     |                                                           | </ul>                                     |
|--------------------------|---------------------|-----------------------------------------------------------|-------------------------------------------|
| Schema in executable doc | `WrongDocumentKind` | "Schema definition not allowed in executable document"    | <ul>                                      |
|                          |                     |                                                           | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Executable-Definitions</li> |
|                          |                     |                                                           | </ul>                                     |
|--------------------------|---------------------|-----------------------------------------------------------|-------------------------------------------|
| Directive def in exec    | `WrongDocumentKind` | "Directive definition not allowed in executable document" | <ul>                                      |
|                          |                     |                                                           | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Executable-Definitions</li> |
|                          |                     |                                                           | </ul>                                     |
|--------------------------|---------------------|-----------------------------------------------------------|-------------------------------------------|
| Operation in schema doc  | `WrongDocumentKind` | "Operation not allowed in schema document"                | <ul>                                      |
|                          |                     |                                                           | <li>**Note:** "Schema documents contain only type system definitions"</li> |
|                          |                     |                                                           | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Type-System</li> |
|                          |                     |                                                           | </ul>                                     |
|--------------------------|---------------------|-----------------------------------------------------------|-------------------------------------------|
| Fragment in schema doc   | `WrongDocumentKind` | "Fragment not allowed in schema document"                 | <ul>                                      |
|                          |                     |                                                           | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Type-System</li> |
|                          |                     |                                                           | </ul>                                     |

---

### Category 7: Empty Construct Errors

| Scenario                   | Kind                    | Message                                | Notes                                     |
|----------------------------|-------------------------|----------------------------------------|-------------------------------------------|
| Empty selection set        | `InvalidEmptyConstruct` | "Selection set cannot be empty"        | <ul>                                      |
|                            |                         |                                        | <li>**Help:** "Add at least one field or fragment spread"</li> |
|                            |                         |                                        | <li>**Spec:** https://spec.graphql.org/September2025/#SelectionSet</li> |
|                            |                         |                                        | </ul>                                     |
|----------------------------|-------------------------|----------------------------------------|-------------------------------------------|
| Empty argument list        | `InvalidEmptyConstruct` | "Argument list cannot be empty"        | <ul>                                      |
|                            |                         |                                        | <li>**Help:** "Omit `()` if no arguments"</li> |
|                            |                         |                                        | <li>**Spec:** https://spec.graphql.org/September2025/#Arguments</li> |
|                            |                         |                                        | </ul>                                     |
|----------------------------|-------------------------|----------------------------------------|-------------------------------------------|
| Empty variable definitions | `InvalidEmptyConstruct` | "Variable definitions cannot be empty" | <ul>                                      |
|                            |                         |                                        | <li>**Help:** "Omit `()` if no variables"</li> |
|                            |                         |                                        | <li>**Spec:** https://spec.graphql.org/September2025/#VariableDefinitions</li> |
|                            |                         |                                        | </ul>                                     |

---

### Category 8: EOF Errors

| Scenario               | Kind            | Message                                           | Notes                                     |
|------------------------|-----------------|---------------------------------------------------|-------------------------------------------|
| EOF in selection set   | `UnexpectedEof` | "Unexpected end of input in selection set"        | <ul>                                      |
|                        |                 |                                                   | <li>**Note:** "Expected `}` to close selection set"</li> |
|                        |                 |                                                   | </ul>                                     |
|------------------------|-----------------|---------------------------------------------------|-------------------------------------------|
| EOF in type definition | `UnexpectedEof` | "Unexpected end of input in type definition"      | <ul>                                      |
|                        |                 |                                                   | <li>**Note:** (Context-dependent)</li>    |
|                        |                 |                                                   | </ul>                                     |
|------------------------|-----------------|---------------------------------------------------|-------------------------------------------|
| EOF after `extend`     | `UnexpectedEof` | "Unexpected end of input after `extend`"          | <ul>                                      |
|                        |                 |                                                   | <li>**Note:** "Expected `type`, `interface`, `schema`, etc."</li> |
|                        |                 |                                                   | </ul>                                     |
|------------------------|-----------------|---------------------------------------------------|-------------------------------------------|
| EOF in directive def   | `UnexpectedEof` | "Unexpected end of input in directive definition" | <ul>                                      |
|                        |                 |                                                   | <li>**Note:** "Expected `on` followed by locations"</li> |
|                        |                 |                                                   | </ul>                                     |
|------------------------|-----------------|---------------------------------------------------|-------------------------------------------|
| General EOF            | `UnexpectedEof` | "Unexpected end of input"                         | <ul>                                      |
|                        |                 |                                                   | <li>**Note:** "Expected {expected}"</li>  |
|                        |                 |                                                   | </ul>                                     |

---

### Category 9: Other Syntax Errors

**Note on Directive Locations:** For invalid directive location errors, use
edit-distance matching (similar to rustc's `find_best_match_for_name` in
[`rustc_span::edit_distance`](https://doc.rust-lang.org/beta/nightly-rustc/src/rustc_span/edit_distance.rs.html#166-172))
to suggest the closest valid location. For example, if the user writes `FEILD`,
suggest `FIELD`.

| Scenario                  | Kind            | Message                                            | Notes                                     |
|---------------------------|-----------------|----------------------------------------------------|-------------------------------------------|
| Variable in const context | `InvalidSyntax` | "Variables not allowed in default value positions" | <ul>                                      |
|                           |                 |                                                    | <li>**Note:** "Default values must be constants"</li> |
|                           |                 |                                                    | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Values.Input-Coercion</li> |
|                           |                 |                                                    | </ul>                                     |
|---------------------------|-----------------|----------------------------------------------------|-------------------------------------------|
| Invalid directive location| `InvalidSyntax` | "Invalid directive location `{location}`"          | <ul>                                      |
|                           |                 |                                                    | <li>**Note:** "Valid locations: QUERY, MUTATION, FIELD, ..."</li> |
|                           |                 |                                                    | <li>**Help (conditional):** If edit-distance match found: "Did you mean `{suggestion}`?"</li> |
|                           |                 |                                                    | <li>**Spec:** https://spec.graphql.org/September2025/#DirectiveLocation</li> |
|                           |                 |                                                    | </ul>                                     |
|---------------------------|-----------------|----------------------------------------------------|-------------------------------------------|
| Extension with no changes | `InvalidSyntax` | "Type extension must add something"                | <ul>                                      |
|                           |                 |                                                    | <li>**Help:** "Add interfaces, directives, or fields"</li> |
|                           |                 |                                                    | <li>**Spec:** https://spec.graphql.org/September2025/#sec-Type-Extensions</li> |
|                           |                 |                                                    | </ul>                                     |

**Note:** Checks like "multiple anonymous operations" are intentionally omitted
from the parser. This allows parsing concatenated documents that will be
managed separately. This validation belongs in `ExecutableDocumentBuilder`.

---

### CLI Output Format

Errors are formatted for CLI output using:

```rust
impl GraphQLParseError {
    pub fn format_detailed(&self, src_text: impl AsRef<str>) -> String {
        // ...
    }
}
```

The `GraphQLErrorNoteKind` determines the prefix used in output:
- `GraphQLErrorNoteKind::General` → `= note: ...`
- `GraphQLErrorNoteKind::Help` → `= help: ...`
- `GraphQLErrorNoteKind::Spec` → `= spec: ...`

**Single error with suggestion:**

```text
error: Expected `:` after field name
  --> schema.graphql:5:12
   |
 5 |     userName String
   |              ^^^^^^ expected `:`
   |
   = note: Field definitions require a colon between the name and type
   = help: Did you mean: `userName: String`?
   = spec: https://spec.graphql.org/September2025/#FieldDefinition
```

**Unclosed delimiter with related spans:**

```text
error: Unclosed `{`
   --> schema.graphql:10:1
    |
  3 |   type User {
    |             - opening brace here
 ...
 10 |
    | ^ expected `}` to close type definition
    |
    = help: Consider adding `}` after line 9
```

**Error with multiple notes:**

```text
error: Integer value too large
  --> query.graphql:3:15
   |
 3 |   field(limit: 9999999999)
   |                ^^^^^^^^^^ value exceeds i32::MAX
   |
   = note: Maximum: 2147483647 (i32::MAX)
   = note: GraphQL Int is a signed 32-bit integer
   = spec: https://spec.graphql.org/September2025/#sec-Int
```

---

## Open Questions

1. **AST Types:** Continue using `graphql_parser::schema` and
   `graphql_parser::query` AST types, or define new ones?
   - **Decision:** Continue using for now; custom AST is future work
   - **Rationale:** Allows faster initial development; custom AST can be added
     later without changing the parser's public API significantly

2. **Trivia in AST:** Should AST nodes carry their trivia?
   - **Decision:** Trivia stays on tokens, not propagated to AST
   - **Rationale:** Keeps AST clean for semantic analysis; trivia is available
     in `GraphQLToken.preceding_trivia` for tools that need it

3. **Span in AST:** Should AST nodes have `GraphQLSourceSpan`?
   - **Decision:** Use existing `Pos` from `graphql_parser` for now
   - **Rationale:** `graphql_parser` AST only stores start position. Full span
     tracking will come with custom AST types.
   - **Note:** Conversion via `SourcePosition::to_ast_pos()` loses:
     - UTF-16 column information
     - Byte offset
     - File path
     - End position

4. **MixedDocument Ordering:** How to handle forward references?
   - **Decision:** Parser does NOT validate definition dependencies
   - **Rationale:** A type can reference another type defined later in the
     document. Forward references are resolved during schema building, not
     parsing. `MixedDocument` preserves definition order for formatters and
     error reporting.
