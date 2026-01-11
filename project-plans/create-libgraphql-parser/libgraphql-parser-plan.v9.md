# Implementation Plan: Unified GraphQL Parser Infrastructure

**Version 9** — Updated 2026-01-08

## Change History

| Version | Date | Changes |
|---------|------|---------|
| v9 | 2026-01-08 | Marked Phase 1 Steps 1.0-1.4 as COMPLETED; Updated Step 1.6 to reference new parser-design.md approach; Added current state summary |
| v8 | Previous | Initial comprehensive plan |

## Current State Summary

**Phase 1 Progress:**
- ✅ Step 1.0: libgraphql-parser crate created
- ✅ Step 1.1: ast module copied, SourcePosition created
- ✅ Step 1.2: GraphQLTokenSource trait, GraphQLToken/GraphQLTokenKind defined
- ✅ Step 1.3: GraphQLTokenStream moved and generalized
- ✅ Step 1.4: RustMacroGraphQLTokenSource implements trait
- ⏳ Step 1.5: Feature flag infrastructure (partially complete)
- ⏳ Step 1.6: **Revised approach** — Building GraphQLParser from scratch per parser-design.md (original migration approach abandoned)

**Phase 2:** Not started (StrToGraphQLTokenSource is a stub)
**Phase 3:** Not started (Parser extension covered by parser-design.md)
**Phase 4:** Not started

---

## Overview
Move from dual parsing approach (`graphql_parser` + `GraphQLSchemaParser`) to unified token-based parser that supports schema documents, executable documents, and mixed documents. The new parser will be implemented in its own crate `libgraphql-parser`.

**Key Decisions:**
- Use trait-based `GraphQLTokenSource` (not enum) for extensibility - allows third parties to implement custom token sources
- Create new `SourcePosition` struct with dual column tracking: `col_utf8` (always available, counts UTF-8 characters) and `col_utf16` (optional, for LSP compatibility), plus `line` and `byte_offset` fields (no dependency on `libgraphql-core`)
- Keep `graphql_parser` AST types for now (future work: custom AST)
- Three parser methods: `parse_schema_document()`, `parse_executable_document()`, `parse_mixed_document()`
- Feature flag (`use-libgraphql-parser`) to toggle between old and new implementation (temporary; will become unconditional once stable)
- **New crate structure:** `libgraphql-parser` crate is standalone and usable independent of `libgraphql-core`
- Copy `ast` module from `libgraphql-core` to `libgraphql-parser` (keeping original in core temporarily for feature flag support; will consolidate later)
- `RustMacroGraphQLTokenSource` stays in `libgraphql-macros` (implements `GraphQLTokenSource` trait from parser crate)
- One struct per file, filename matches struct name in `lower_snake_case`
- `GraphQLToken` uses explicit punctuator variants (not `Punctuator(String)`) for type safety and avoiding allocations
- Comprehensive test suite with exhaustive tests including vendored tests from graphql-js and graphql-parser is mandatory. All tests should give clear error information useful for debugging if they fail.
- Comments and commas are lexed as preceding-trivia on tokens (not skipped)
- All errors include helpful suggestions (called "error_notes") and "did-you-mean" hints when possible
- `True`, `False`, `Null` are distinct token kinds (not parsed as `Name` tokens) for type safety
- Negative numbers like `-123` are lexed as single tokens: `IntValue("-123")`

**Critical Files:**
- `/crates/libgraphql-parser/` - New standalone parser crate
- `/crates/libgraphql-parser/src/token/graphql_token.rs` - Token enum with explicit variants
- `/crates/libgraphql-parser/src/token_source/graphql_token_source.rs` - Trait for token sources
- `/crates/libgraphql-parser/src/graphql_parser.rs` - Generic parser implementation
- `/crates/libgraphql-parser/src/token_source/str_to_graphql_token_source.rs` - String-based lexer
- `/crates/libgraphql-parser/src/source_position.rs` - Position tracking
- `/crates/libgraphql-parser/src/token/graphql_token_span.rs` - Token span with start/end positions
- `/crates/libgraphql-macros/src/rust_macro_graphql_token_source.rs` - Proc-macro token source (implements trait)

---

## Phase 1: Foundation & Infrastructure
*Goal: Prepare architecture for unified parsing without breaking existing functionality*

**Note:** Each step must pass `cargo clippy --tests` and `cargo test` with no warnings or errors before proceeding.

### Step 1.0: Create libgraphql-parser Crate ✅ COMPLETED
**Outcome:** New crate structure ready for parser implementation

**Status:** Completed. The crate exists at `/crates/libgraphql-parser/` with full module structure.

**Tasks:**
1. Create new crate at `/crates/libgraphql-parser/`:
   ```bash
   cargo new --lib crates/libgraphql-parser --vcs none
   ```

2. Update workspace `Cargo.toml` to add `smallvec` dependency and new crate:
   ```toml
   [workspace]
   members = ["crates/libgraphql", "crates/libgraphql-core", "crates/libgraphql-macros", "crates/libgraphql-parser"]

   [workspace.dependencies]
   # ... existing dependencies ...
   smallvec = "1.13"
   ```

3. Add dependencies and metadata to `/crates/libgraphql-parser/Cargo.toml`:
   ```toml
   [package]
   name = "libgraphql-parser"
   version = "0.1.0"
   edition = "2024"
   license = "MIT"
   repository = "https://github.com/jeffmo/libgraphql"
   description = "A GraphQL parser library with support for schema, executable, and mixed documents"

   [dependencies]
   graphql-parser.workspace = true
   smallvec.workspace = true
   thiserror.workspace = true
   serde.workspace = true
   ```
   **Note:** `proc-macro2` is NOT a dependency here. Conversion from `proc_macro2::Span` to
   `SourcePosition` is handled in `libgraphql-macros` to keep this crate lightweight.

4. Update `/crates/libgraphql-core/Cargo.toml`:
   ```toml
   [features]
   use-libgraphql-parser = ["dep:libgraphql-parser"]

   [dependencies]
   libgraphql-parser = { path = "../libgraphql-parser", optional = true }
   ```

5. Update `/crates/libgraphql/Cargo.toml`:
   ```toml
   [features]
   default = ["macros"]
   macros = ["dep:libgraphql-macros"]
   use-libgraphql-parser = ["libgraphql-core/use-libgraphql-parser"]
   ```

6. **Tests:**
   - `cargo build` succeeds
   - `cargo test` passes

**Considerations:**
- `libgraphql-parser` is designed to be useful independent of `libgraphql-core` - no dependency on core
- Feature flag controls inclusion in `libgraphql-core` during experimental phase; will become unconditional once stable and feature flag will be removed
- Clean separation of concerns enables third-party use of parser alone
- **Module organization** - Public APIs are organized as follows:
  - `libgraphql_parser::token::` - Token types: `GraphQLToken`, `GraphQLTokenKind`, `GraphQLTriviaToken`, `GraphQLTriviaTokenVec`, `CookGraphQLStringError`
  - `libgraphql_parser::token_source::` - Token sources: `GraphQLTokenSource`, `StrToGraphQLTokenSource`
  - `libgraphql_parser::ast::` - AST types: `MixedDocument`, `MixedDocumentDefinition`
  - `libgraphql_parser::` (root) - `GraphQLParser`, `GraphQLTokenStream`, `SourcePosition`, `GraphQLSourceSpan`, `GraphQLErrorNotes`, `SmallVec`

---

### Step 1.1: Copy ast Module and Create SourcePosition ✅ COMPLETED
**Outcome:** Location tracking ready for all token sources; ast module in parser crate

**Status:** Completed. `SourcePosition` exists with dual-column tracking (UTF-8 and optional UTF-16). `GraphQLSourceSpan` includes `file_path: Option<PathBuf>`. The "cook" terminology has been renamed to "parse" (`GraphQLStringParsingError`, `parse_int_value()`, etc.).

**Tasks:**
1. Copy `/crates/libgraphql-core/src/ast.rs` → `/crates/libgraphql-parser/src/ast.rs`:
   - Keep all existing code unchanged
   - Update module exports in `/crates/libgraphql-parser/src/lib.rs`
   - **Note:** The original `ast.rs` remains in `libgraphql-core` temporarily to support the feature flag transition. Once `libgraphql-parser` becomes the default, we will either remove `ast.rs` from `libgraphql-core` entirely or replace it with a re-export from `libgraphql-parser`.

2. Add conditional re-export in `/crates/libgraphql-core/src/lib.rs`:
   ```rust
   #[cfg(feature = "use-libgraphql-parser")]
   pub use libgraphql_parser::ast;

   #[cfg(not(feature = "use-libgraphql-parser"))]
   pub mod ast;  // Keep existing ast module when feature disabled
   ```

3. Create `/crates/libgraphql-parser/src/source_position.rs`:
   ```rust
   use crate::ast::AstPos;  // AstPos is an alias for graphql_parser::Pos

   /// Source position information for parsing, with dual column tracking.
   ///
   /// This is a pure data struct with no mutation methods. Lexers are responsible
   /// for computing position values as they scan input.
   ///
   /// This is standalone with no dependency on libgraphql-core.
   /// All fields are private with accessor methods.
   ///
   /// # Indexing Convention
   ///
   /// **All position values are 0-based:**
   /// - `line`: 0 = first line of the document (0-based)
   /// - `col_utf8`: UTF-8 character count within the current line (0-based)
   /// - `col_utf16`: Optional UTF-16 code unit offset within the current line (0-based)
   /// - `byte_offset`: byte offset within the whole document (0-based)
   ///
   /// # Dual Column Tracking
   ///
   /// Two column representations are supported:
   /// - **`col_utf8`** (always available): Number of UTF-8 characters from the start
   ///   of the current line. Increments by 1 for each character regardless of its
   ///   byte representation. This is intuitive for users and matches what most
   ///   text editors display as "column".
   /// - **`col_utf16`** (optional): UTF-16 code unit offset within the line.
   ///   This aligns with LSP (Language Server Protocol) and many editors.
   ///   It is `Some` when the token source can provide it (e.g., `StrToGraphQLTokenSource`),
   ///   and `None` when it cannot (e.g., `RustMacroGraphQLTokenSource` in
   ///   `libgraphql-macros` which uses `proc_macro2::Span` that only provides
   ///   UTF-8 char-based positions).
   ///
   /// For ASCII text, both columns are equal. For text containing characters outside
   /// the Basic Multilingual Plane (e.g., emoji), they differ:
   /// - `col_utf8` advances by 1 for each UTF-8 character
   /// - `col_utf16` advances by the character's UTF-16 length (1 or 2 code units)
   #[derive(Clone, Debug, Eq, PartialEq)]
   pub struct SourcePosition {
       /// Line number (0-based: first line is 0)
       line: usize,

       /// UTF-8 character count within current line (0-based: first position is 0)
       col_utf8: usize,

       /// UTF-16 code unit offset within current line (0-based), if available.
       /// None when the token source cannot provide UTF-16 column information.
       col_utf16: Option<usize>,

       /// byte offset from start of document (0-based: first byte is 0)
       byte_offset: usize,
   }

   impl SourcePosition {
       /// Create a new SourcePosition.
       ///
       /// # Arguments
       /// - `line`: 0-based line number (0 = first line)
       /// - `col_utf8`: 0-based UTF-8 character count within current line
       /// - `col_utf16`: 0-based UTF-16 code unit offset within current line,
       ///   or `None` if not available (e.g., from `proc_macro2::Span`)
       /// - `byte_offset`: 0-based byte offset from document start
       pub fn new(
           line: usize,
           col_utf8: usize,
           col_utf16: Option<usize>,
           byte_offset: usize,
       ) -> Self {
           Self {
               line,
               col_utf8,
               col_utf16,
               byte_offset,
           }
       }

       /// Returns the 0-based line number.
       pub fn line(&self) -> usize { self.line }

       /// Returns the 0-based (UTF-8) character count within the current line.
       ///
       /// This increments by 1 for each character regardless of byte representation.
       /// For example, both 'a' (1 byte) and '🎉' (4 bytes) each add 1 to this count.
       pub fn col_utf8(&self) -> usize { self.col_utf8 }

       /// Returns the 0-based UTF-16 code unit offset within the current line,
       /// if available.
       ///
       /// This is `Some` when the token source can provide UTF-16 column
       /// information (e.g., `StrToGraphQLTokenSource`), and `None` when it
       /// cannot (e.g., `RustMacroGraphQLTokenSource` in `libgraphql-macros`).
       ///
       /// For LSP compatibility, prefer this method when available.
       pub fn col_utf16(&self) -> Option<usize> { self.col_utf16 }

       /// Returns the 0-based byte offset from document start.
       pub fn byte_offset(&self) -> usize { self.byte_offset }

       pub fn to_ast_pos(&self) -> AstPos {
           // AstPos uses 1-based line/column, convert from our 0-based
           // Always use character count for column (consistent, no fallback logic)
           AstPos {
               line: self.line + 1,
               column: self.col_utf8 + 1,
           }
       }
   }
   ```
   **Note:** `SourcePosition` is a pure data struct with no character-advancement logic.
   Line terminator handling (e.g., `\r\n` as single newline) is the responsibility of
   the lexer (`StrToGraphQLTokenSource`), which tracks its own state and constructs
   `SourcePosition` values with the correct offsets.

   **Note:** No `from_span()` method here. Conversion from `proc_macro2::Span` is handled
   in `libgraphql-macros` using `SourcePosition::new(..., None, ...)` since `proc_macro2::Span`
   only provides UTF-8 char-based positions, not UTF-16 columns.

4. **Tests:**
   - Unit tests for SourcePosition constructor and accessors
   - **Constructor tests:**
     - Verify `SourcePosition::new(0, 0, Some(0), 0)` represents the very start of a document
     - Verify `SourcePosition::new(1, 0, Some(0), 10)` represents first char of second line
     - Verify `SourcePosition::new(0, 5, None, 5)` creates position with `col_utf16() == None`
   - **Accessor tests:**
     - Verify `line()`, `col_utf8()`, `col_utf16()`, `byte_offset()` return correct values
   - **Equality tests:**
     - Verify two positions with same values are equal
     - Verify positions with different values are not equal
   - **to_ast_pos() tests:**
     - Verify conversion to 1-based AstPos is correct
     - Verify `to_ast_pos()` always uses `col_utf8` for the column value
   - `cargo clippy --tests` passes
   - `cargo test` passes

   **Note:** Line ending handling (`\r\n` as single newline) and character advancement
   are tested in Step 2.1 as part of `StrToGraphQLTokenSource` tests, since that logic
   lives in the lexer.

**Considerations:**
- `FilePosition` remains unchanged in `libgraphql-core` (no changes needed there)
- `SourcePosition` is standalone in `libgraphql-parser` - no dependency on `libgraphql-core` (avoids cyclic dependency)
- **All positions are 0-based** - this is documented in rustdoc and tested
- **Pure data struct:** `SourcePosition` has no mutation methods; lexers construct new instances
- **Dual column tracking:** `col_utf8()` always available (UTF-8 character count); `col_utf16()` is `Option<usize>` for LSP compatibility
- Conversion from `proc_macro2::Span` passes `None` for `col_utf16` (see Step 1.4)
- `byte_offset` field enables efficient error reporting with source snippets
- `AstPos` is defined in `ast.rs` as an alias for `graphql_parser::Pos`

---

### Step 1.2: Create GraphQLTokenSource Trait and Define GraphQLToken ✅ COMPLETED
**Outcome:** Trait-based abstraction for token sources with explicit token types

**Status:** Completed. Full token system implemented including:
- `GraphQLToken` with `kind`, `preceding_trivia`, and `span` fields
- `GraphQLTokenKind` with all 15 punctuators, literals, True/False/Null, Eof, and Error variants
- `GraphQLTriviaToken` (Comment and Comma variants)
- `GraphQLTokenSource` marker trait
- Parse error system: `GraphQLParseError`, `GraphQLParseErrorKind` (10 variants), `GraphQLErrorNote`, `GraphQLErrorNoteKind`
- Value parsing errors: `ValueParsingError`, `GraphQLStringParsingError`
- Supporting enums: `DefinitionKind`, `DocumentKind`, `ReservedNameContext`

**Tasks:**
0. Create `/crates/libgraphql-parser/src/graphql_error_notes.rs`:
   ```rust
   use crate::GraphQLSourceSpan;
   use crate::SmallVec;

   /// Type alias for error notes. Each note is a message with an optional span
   /// indicating where the note applies.
   /// Uses SmallVec since most errors have 0-2 notes.
   pub type GraphQLErrorNotes = SmallVec<[(String, Option<GraphQLSourceSpan>); 2]>;
   ```
1. Create `/crates/libgraphql-parser/src/token/graphql_token.rs` with trivia-aware structure:
   ```rust
   use crate::GraphQLErrorNotes;
   use crate::SmallVec;  // Re-exported from crate root for consistency with third-party implementors
   use std::num::ParseFloatError;
   use std::num::ParseIntError;

   /// Type alias for trivia storage. Uses SmallVec to avoid heap allocation
   /// for the common case of 0-2 trivia items per token.
   /// Re-exported for third-party `GraphQLTokenSource` implementations.
   pub type GraphQLTriviaTokenVec = SmallVec<[GraphQLTriviaToken; 2]>;

   /// A GraphQL token with attached preceding trivia (comments, commas).
   ///
   /// Trivia is attached to the *following* token, so parsers can simply
   /// call `peek()` and `consume()` without worrying about skipping trivia.
   /// Trivia is preserved for tooling (formatters, linters).
   #[derive(Clone, Debug, PartialEq)]
   pub struct GraphQLToken {
       /// The kind of token (including Error for lexer errors).
       pub kind: GraphQLTokenKind,
       /// Trivia (comments, commas) that precede this token.
       pub preceding_trivia: GraphQLTriviaTokenVec,
       /// The source location span of this token.
       pub span: GraphQLSourceSpan,
   }

   /// The kind of a GraphQL token.
   ///
   /// Literal values (IntValue, FloatValue, StringValue) store only the raw
   /// source text. Use the `cook_*` methods to parse/process values when needed.
   ///
   /// **Note on numeric literals:** Negative numbers like `-123` are lexed as
   /// single tokens (e.g., `IntValue("-123")`), not as separate minus and number
   /// tokens. This matches the GraphQL spec's grammar for IntValue/FloatValue.
   ///
   /// TODO: Currently uses `String` for Name/StringValue/etc. A future
   /// optimization experiment could explore using `Cow<'a, str>` to enable
   /// zero-copy lexing from string sources.
   #[derive(Clone, Debug, PartialEq)]
   pub enum GraphQLTokenKind {
       // Punctuators (no allocation needed)
       Ampersand,            // &
       At,                   // @
       Bang,                 // !
       Colon,                // :
       CurlyBraceClose,      // }
       CurlyBraceOpen,       // {
       Dollar,               // $
       Ellipsis,             // ...
       Equals,               // =
       ParenClose,           // )
       ParenOpen,            // (
       Pipe,                 // |
       SquareBracketClose,   // ]
       SquareBracketOpen,    // [

       // Literals (raw source text only; use cook_* methods to parse)
       Name(String),
       /// Raw source text of integer literal including optional negative sign (e.g., "-123", "0")
       IntValue(String),
       /// Raw source text of float literal including optional negative sign (e.g., "-1.23e-4", "0.5")
       FloatValue(String),
       /// Raw source text of string literal including quotes (e.g., "\"hello\\nworld\"")
       StringValue(String),

       // Boolean and null (distinct from Name for type safety)
       True,
       False,
       Null,

       // End of input (carries any trailing trivia)
       Eof,

       // Lexer error (allows error recovery)
       /// TODO: Explore replacing error_notes with a richer diagnostics
       /// structure that includes things like severity level and "fix action"
       /// for IDE integration.
       Error {
           message: String,
           error_notes: GraphQLErrorNotes,
       },
   }

   /// Error returned when cooking a GraphQL string value fails.
   ///
   /// This error can occur during `cook_string_value()` when processing
   /// escape sequences. It may be wrapped in a `ParseError` variant when
   /// the parser encounters an invalid string.
   #[derive(Clone, Debug, thiserror::Error)]
   pub enum CookGraphQLStringError {
       #[error("Invalid escape sequence: {0}")]
       InvalidEscapeSequence(String),
       #[error("Unterminated string")]
       UnterminatedString,
       #[error("Invalid unicode escape: {0}")]
       InvalidUnicodeEscape(String),
   }

   /// Trivia tokens that don't affect parsing but are preserved for tooling.
   #[derive(Clone, Debug, PartialEq)]
   pub enum GraphQLTriviaToken {
       Comment {
           value: String,
           span: GraphQLSourceSpan,
       },
       Comma {
           span: GraphQLSourceSpan,
       },
   }

   impl GraphQLTokenKind {
       pub fn is_punctuator(&self) -> bool { ... }
       pub fn as_punctuator_str(&self) -> Option<&'static str> { ... }
       pub fn is_value(&self) -> bool { ... }
       pub fn is_error(&self) -> bool { matches!(self, GraphQLTokenKind::Error { .. }) }

       /// Parse an IntValue's raw text to i64.
       /// Returns None if not an IntValue, Some(Err) if parsing fails.
       pub fn cook_int_value(&self) -> Option<Result<i64, ParseIntError>> {
           match self {
               GraphQLTokenKind::IntValue(raw) => Some(raw.parse()),
               _ => None,
           }
       }

       /// Parse a FloatValue's raw text to f64.
       /// Returns None if not a FloatValue, Some(Err) if parsing fails.
       pub fn cook_float_value(&self) -> Option<Result<f64, ParseFloatError>> {
           match self {
               GraphQLTokenKind::FloatValue(raw) => Some(raw.parse()),
               _ => None,
           }
       }

       /// Process a StringValue's raw text to unescaped content.
       /// Handles escape sequences per GraphQL spec.
       /// Returns None if not a StringValue, Some(Err) if unescaping fails.
       pub fn cook_string_value(&self) -> Option<Result<String, CookGraphQLStringError>> {
           match self {
               GraphQLTokenKind::StringValue(raw) => {
                   // Implementation: Strip quotes and process escape sequences.
                   // Handles both single-line ("...") and block strings ("""...""").
                   // For single-line: process \n, \r, \t, \\, \", \/, \b, \f,
                   //   \uXXXX (fixed 4-digit), and \u{X...} (variable length).
                   // For block strings: apply indentation stripping algorithm
                   //   per spec, then process \""" escape only.
                   Some(/* cook_graphql_string implementation */)
               }
               _ => None,
           }
       }
   }

   impl GraphQLToken {
       /// Convenience constructor for a token with no preceding trivia.
       pub fn new(kind: GraphQLTokenKind, span: GraphQLSourceSpan) -> Self {
           Self { kind, preceding_trivia: SmallVec::new(), span }
       }

       /// Convenience constructor for a token with preceding trivia.
       pub fn with_trivia(
           kind: GraphQLTokenKind,
           preceding_trivia: GraphQLTriviaTokenVec,
           span: GraphQLSourceSpan,
       ) -> Self {
           Self { kind, preceding_trivia, span }
       }
   }
   ```

2. Create `/crates/libgraphql-parser/src/token/graphql_token_span.rs`:
   ```rust
   use crate::SourcePosition;

   /// Represents the span of some source text from start to end position.
   ///
   /// The span is a half-open interval: `[start_inclusive, end_exclusive)`.
   /// - `start_inclusive`: Position of the first character of the source text
   /// - `end_exclusive`: Position immediately after the last character of the
   ///   source text
   #[derive(Clone, Debug, Eq, PartialEq)]
   pub struct GraphQLSourceSpan {
       pub start_inclusive: SourcePosition,
       pub end_exclusive: SourcePosition,
   }
   ```

3. Create `/crates/libgraphql-parser/src/token_source/graphql_token_source.rs` with trait definition:
   ```rust
   use crate::token::GraphQLToken;

   /// Marker trait for sources that produce GraphQL tokens.
   ///
   /// This trait enables extensibility - third parties can implement custom
   /// token sources (e.g., from different input formats or with custom
   /// preprocessing).
   ///
   /// # Implementing
   ///
   /// Implementors provide an `Iterator` that produces tokens one at a time.
   /// All lookahead, buffering, and peeking is handled by `GraphQLTokenStream`.
   ///
   /// Lexers are responsible for:
   /// - Skipping whitespace (an "ignored token" per the GraphQL spec)
   /// - Accumulating trivia (comments, commas) and attaching to the next token
   /// - Emitting `GraphQLToken::Error` for lexer errors (enables error recovery)
   /// - Emitting a final token with `GraphQLTokenKind::Eof` carrying trailing trivia
   pub trait GraphQLTokenSource: Iterator<Item = GraphQLToken> {}
   impl<T> GraphQLTokenSource for T where T: Iterator<Item = GraphQLToken> {}
   ```

4. Update all imports in macros crate

5. **Tests:**
   - Unit tests for GraphQLToken, GraphQLTokenKind, GraphQLTriviaToken
   - Unit tests for GraphQLSourceSpan
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Trivia (comments, commas) attached to following token - parser doesn't need to skip trivia
- Marker trait with `Iterator<Item = GraphQLToken>` - simpler than tuple, span is inside token
- Lexers accumulate trivia and attach to next token (not `GraphQLTokenStream`'s job)
- `GraphQLTokenKind::Eof` carries trailing trivia at end of document
- Explicit punctuator variants provide type safety and avoid string allocations
- `True`, `False`, `Null` are distinct token kinds for better type safety (not `Name` tokens)
- `GraphQLSourceSpan` has public fields so third-party token sources can construct spans directly
- `StrToGraphQLTokenSource` is a stub initially (Step 2.x implements it)
- Monomorphization is acceptable - common case uses only 1-2 token source types
- Negative numbers are single tokens (e.g., `IntValue("-123")`)
- TODO notes in code for future optimization experiments: `Cow<'a, str>`, `SmallVec` for trivia

---

### Step 1.3: Move and Generalize GraphQLTokenStream ✅ COMPLETED
**Outcome:** Token stream works with any `GraphQLTokenSource` implementation, centralizing all lookahead logic

**Status:** Completed. `GraphQLTokenStream<S: GraphQLTokenSource>` is fully implemented with:
- `peek()`, `peek_nth()`, `consume()`, `current_token()` methods
- Internal buffering with `compact_buffer()` for memory management
- `is_at_end()` helper for EOF detection

**Tasks:**
1. Move `/crates/libgraphql-macros/src/graphql_token_stream.rs` → `/crates/libgraphql-parser/src/graphql_token_stream.rs`

2. Update `GraphQLTokenStream` to be generic over `GraphQLTokenSource`:
   ```rust
   use crate::token::GraphQLToken;
   use crate::token_source::GraphQLTokenSource;

   /// Streaming token parser with lookahead buffer.
   /// Generic over any token source implementing `GraphQLTokenSource`.
   ///
   /// This struct centralizes buffering, peeking, and lookahead logic.
   /// Since trivia is already attached to tokens by the lexer, the parser
   /// can simply call `peek()` and `consume()` without worrying about trivia.
   ///
   /// # Buffer Management
   ///
   /// Tokens are stored in `buffer`. The `current_index` points to the most
   /// recently consumed token. Periodically, consumed tokens are compacted
   /// (removed from the front of the buffer) to prevent unbounded growth.
   ///
   /// `compact_buffer()` should be called whenever there may be unreferenceable
   /// tokens in the buffer (i.e., tokens before `current_index` that will never
   /// be accessed again). Typically this is after successfully parsing a complete
   /// top-level definition.
   ///
   /// # Future Configuration Options (TODO)
   ///
   /// In a future iteration, consider adding a `GraphQLTokenStreamOptions` struct
   /// to configure behavior:
   /// - `include_trivia: bool` - Whether to include preceding_trivia in tokens
   ///   (can be disabled for performance when trivia is not needed)
   /// - `max_tokens: Option<usize>` - Limit total tokens returned (DoS protection)
   pub struct GraphQLTokenStream<S: GraphQLTokenSource> {
       token_source: S,
       /// Buffer of tokens. Grows as needed for lookahead.
       buffer: Vec<GraphQLToken>,
       /// Index of the current (most recently consumed) token in buffer.
       /// None if no token has been consumed yet.
       current_index: Option<usize>,
   }

   impl<S: GraphQLTokenSource> GraphQLTokenStream<S> {
       pub fn new(token_source: S) -> Self { ... }

       /// Peek at the next token without consuming it.
       /// Returns the token at current_index + 1 (or index 0 if nothing consumed yet).
       pub fn peek(&mut self) -> Option<&GraphQLToken> { ... }

       /// Peek at the nth token ahead (0-indexed from next unconsumed token).
       /// `peek_nth(0)` is equivalent to `peek()`.
       pub fn peek_nth(&mut self, n: usize) -> Option<&GraphQLToken> { ... }

       /// Advance to the next token and return a reference to it.
       /// The token is retained in buffer for access via `current_token()`.
       pub fn consume(&mut self) -> Option<&GraphQLToken> { ... }

       /// Returns the most recently consumed token.
       /// Returns None if no token has been consumed yet.
       pub fn current_token(&self) -> Option<&GraphQLToken> {
           self.current_index.map(|i| &self.buffer[i])
       }

       /// Compact the buffer by removing tokens before current_index.
       ///
       /// Call this after parsing each top-level definition to prevent
       /// unbounded buffer growth. Should be called whenever there may be
       /// unreferenceable tokens in the buffer.
       pub fn compact_buffer(&mut self) { ... }
   }
   ```

3. Implement internal buffering:
   - `peek()` and `peek_nth()` call `self.token_source.next()` to fill buffer as needed
   - `consume()` advances `current_index` and returns reference to that token
   - Buffer uses `Vec<GraphQLToken>` with index-based access (no copies for `current_token`)
   - `compact_buffer()` removes tokens before `current_index` and adjusts the index
   - Compaction is called by parser after parsing each top-level definition

4. **Tests:**
   - Port existing token stream tests
   - Test lookahead buffering behavior
   - **Buffer length regression test**: Consume 10,000+ tokens and verify buffer length
     stays bounded (e.g., under 100 tokens) when `compact_buffer()` is called periodically
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Generic `S: GraphQLTokenSource` enables any token source implementation
- All lookahead/buffering logic centralized here
- No trivia filtering needed - trivia already attached by lexer
- `current_index` is an offset into buffer, avoiding token copies
- Buffer compaction prevents memory growth for large documents
- Existing code in `libgraphql-macros` continues to work after updating to implement trait

---

### Step 1.4: Update RustMacroGraphQLTokenSource to Implement Trait ✅ COMPLETED
**Outcome:** `RustMacroGraphQLTokenSource` in macros crate implements `GraphQLTokenSource` trait

**Status:** Completed. Full implementation (~890 lines) including:
- Implements `Iterator<Item = GraphQLToken>` (and thus `GraphQLTokenSource`)
- Complex dot handling state machine for `...` detection
- Block string detection and combination
- Negative number handling (`-17` → single token)
- Error generation with helpful notes
- Span handling using `proc_macro2::Span::start()/end()`
- Comma trivia accumulation (comments not available from proc_macro2)
- Documented limitations (no Rust comments, UTF-16 columns unavailable)

**Important Limitation:** `proc_macro2::TokenStream` does NOT contain Rust comments (they are
stripped by the Rust tokenizer) and has no whitespace tokens. This means:
- `RustMacroGraphQLTokenSource` **cannot produce `GraphQLTriviaToken::Comment`** trivia
- Rust comments (`// comment`) written in macro invocations are stripped and not recoverable
- **Commas ARE available** as `Punct` tokens, so `GraphQLTriviaToken::Comma` works normally
- Position information must be derived from `proc_macro2::Span` values

**Tasks:**
1. Rename `/crates/libgraphql-macros/src/rust_to_graphql_token_adapter.rs` → `rust_macro_graphql_token_source.rs`

2. Rename `RustToGraphQLTokenAdapter` → `RustMacroGraphQLTokenSource`

3. Add `libgraphql-parser` as dependency to `/crates/libgraphql-macros/Cargo.toml`:
   ```toml
   [dependencies]
   libgraphql-parser = { path = "../libgraphql-parser" }
   proc-macro2 = { version = "1.0.41", features = ["span-locations"] }
   ```

4. Create helper function in `libgraphql-macros` to convert `proc_macro2::Span` to `SourcePosition`:
   ```rust
   /// Convert a proc_macro2::Span to a SourcePosition.
   ///
   /// Note: Requires `proc_macro2` with `span-locations` feature enabled.
   /// The `span-locations` feature is required for `span.start()`, `span.end()`,
   /// and `span.byte_range()` to return meaningful values.
   ///
   /// **Important:** `proc_macro2::Span` only provides UTF-8 char-based column
   /// positions, not UTF-16 code unit offsets. We pass the char offset as
   /// `col_utf8` since it is the UTF-8 char offset, but we don't have
   /// the UTF-16 col offset so we do not pass it.
   fn source_position_from_span(span: &proc_macro2::Span) -> SourcePosition {
       let start = span.start();
       // proc_macro2 uses 1-based lines, we use 0-based
       // proc_macro2 column is already 0-based and is a UTF-8 char offset
       SourcePosition::new(
           start.line.saturating_sub(1),
           start.column,  // UTF-8 char offset, used as col_utf8 
           None,          // UTF-16 column not available from proc_macro2
           span.byte_range().start,
       )
   }
   ```
   This keeps the `proc_macro2` dependency in `libgraphql-macros` only (not in `libgraphql-parser`).

5. Update `RustMacroGraphQLTokenSource` to implement `GraphQLTokenSource` trait:
   ```rust
   use libgraphql_parser::SourcePosition;
   use libgraphql_parser::token::GraphQLToken;
   use libgraphql_parser::token::GraphQLTokenKind;
   use libgraphql_parser::token::GraphQLSourceSpan;
   use libgraphql_parser::token::GraphQLTriviaToken;
   use libgraphql_parser::token_source::GraphQLTokenSource;

   // NOTE: Rust macros only report byte_offsets properly when built with Rust
   //       nightly toolchains. At the time of this writing stable rustc
   //       toolchains do not provide accurate or meaningful output for
   //       `proc_macro::Span::byte_range()`.
   //
   //       See: https://github.com/rust-lang/rust/issues/54725
   //
   //       TODO: It would be good to add something that emits a warning with
   //             a clear description of caveats when using `libgraphql-macros`
   //             with a non-nightly (or otherwise incompatible) Rust toolchain.
   //
   //             e.g. build_dependency on `rustc_version` -> build.rs file that
   //             uses `rustc_version::version_meta()` to emit 
   //             "cargo:rustc-cfg=libgraphql_rustc_nightly" when on nightly.
   pub struct RustMacroGraphQLTokenSource {
       tokens: Peekable<proc_macro2::token_stream::IntoIter>,
       pending_trivia: Vec<GraphQLTriviaToken>,  // For accumulating commas (comments not available)
       finished: bool,
   }

   impl Iterator for RustMacroGraphQLTokenSource {
       type Item = GraphQLToken;

       fn next(&mut self) -> Option<Self::Item> {
           // NOTE: proc_macro2::TokenStream does not preserve Rust comments,
           // so only commas will appear in preceding_trivia for this token source.
           // Accumulate commas in pending_trivia, attach to next non-trivia token.
           // On exhaustion, emit Eof with any remaining trivia.
           // ...
       }
   }

   impl GraphQLTokenSource for RustMacroGraphQLTokenSource {}
   ```

6. Implement trivia accumulation:
   - Accumulate **commas only** in `pending_trivia` (Rust comments not available from proc_macro2)
   - When encountering a non-trivia token, wrap it with accumulated trivia
   - On source exhaustion, emit `GraphQLTokenKind::Eof` with trailing trivia

7. Convert `proc_macro2::Span` to `GraphQLSourceSpan`:
   - Use `source_position_from_span()` helper for start and end positions
   - Derive end position from span end: `span.end()` for line/col_char, calculate byte_offset
   - Note: `col_utf16()` will be `None` for both start and end positions

8. Update to emit `GraphQLTokenKind` variants (e.g., `GraphQLTokenKind::CurlyBraceOpen`)

9. **Do not re-export parser abstractions** from `libgraphql-macros` (breaking change is acceptable)
   - Remove any existing public parser exports from `libgraphql-macros`

10. **Document the comment limitation** in rustdoc:
    ```rust
    /// A GraphQL token source that reads from Rust proc-macro token streams.
    ///
    /// # Limitations
    ///
    /// Due to how Rust's tokenizer works, this token source has inherent limitations:
    ///
    /// - **No Rust comment preservation**: Rust strips comments (`// ...`) before tokens reach
    ///   proc macros, so `preceding_trivia` will only contain `Comma` tokens, never `Comment`
    ///   tokens. Note that GraphQL uses `#` for comments, but since GraphQL is embedded in Rust
    ///   macro syntax here, users might write Rust-style comments which are lost.
    /// - **No whitespace tokens**: Whitespace is not tokenized by Rust, so position information
    ///   is derived from `proc_macro2::Span` values rather than character-by-character scanning.
    ///
    /// For use cases requiring comment preservation (formatters, linters), use
    /// `StrToGraphQLTokenSource` with the original source text instead.
    ```

11. **Tests:**
    - Port existing token source tests
    - Verify all proc macro tests still pass
    - **Position accuracy tests** (see Step 1.4.1 below)
    - `cargo clippy --tests` passes
    - `cargo test` passes

---

### Step 1.4.1: Position Accuracy Tests for RustMacroGraphQLTokenSource ⏳ PARTIALLY COMPLETE
**Outcome:** Verified that line/col_char/byte_offset values are accurate in various scenarios

**Status:** Basic tests exist but comprehensive position accuracy test suite not yet implemented. Cross-validation tests against `StrToGraphQLTokenSource` blocked until Phase 2 completes.

**Tasks:**
1. Create comprehensive position tests in `/crates/libgraphql-macros/src/tests/token_source_position_tests.rs`:
   - **Baseline tests:**
     - Single-line schema: verify positions for each token
     - Multi-line schema: verify line numbers increment correctly
     - Verify byte_offset matches expected character positions
     - Verify `col_utf8()` returns expected column within line (note: derived from proc_macro2 character count)
     - Verify `col_utf16()` returns `None` (proc_macro2 doesn't provide UTF-16 columns)
   - **Edge case tests:**
     - Tokens immediately after newlines
     - Unicode characters in identifiers (if supported by Rust tokenizer) are not allowed in GraphQL
     - Very long lines (position doesn't overflow)
     - Mixed indentation (tabs vs spaces)
     - Tokens spanning multiple Rust tokens (e.g., `...` spread operator)
   - **Tricky scenarios:**
     - Nested braces with varying indentation
     - String literals containing newlines (block strings)
     - Tokens at the very start of input (line 0, col_utf8() == 0)
     - Tokens at EOF

2. **Cross-validate positions** against `StrToGraphQLTokenSource`:
   - Parse same GraphQL text with both token sources
   - Treat proc-macro byte_offset (from `RustMacroGraphQLTokenSource`) as “Rust-file byte offset” and Str byte_offset as “document byte offset” (different coordinate spaces).
   - Compare token kinds and relative ordering always.
   - Compare line/col within the GraphQL snippet only if you can establish a snippet origin (e.g., if the macro input is a string literal and you know its start span).
   - **Document expected differences:**
     - `RustMacroGraphQLTokenSource` returns `col_utf16() == None`, while `StrToGraphQLTokenSource` returns `col_utf16() == Some(value)`
     - `RustMacroGraphQLTokenSource` returns a byte_offset relative to the Rust file the macro was expanded in; `StrToGraphQLTokenSource` defines it as an offset relative to the start of the `&str` document

3. Document any known position discrepancies between token sources

**Considerations:**
- `RustMacroGraphQLTokenSource` stays in `libgraphql-macros` (where it belongs - it's proc-macro specific)
- It implements the `GraphQLTokenSource` trait from `libgraphql-parser`
- `libgraphql-macros` now depends on `libgraphql-parser` (not the other way around)
- Third parties can implement their own token sources following the same pattern
- Breaking change to remove parser exports from macros crate is intentional
- **`proc_macro2` dependency stays in `libgraphql-macros`** - not in `libgraphql-parser`
- **`proc_macro2` requires `span-locations` feature** for position information
- **`col_utf16()` returns `None`** for this token source since `proc_macro2::Span` only provides UTF-8 char col positions
- `preceding_trivia` will only contain `Comma` tokens for proc-macro sources - document this clearly

---

### Step 1.5: Add Cargo Feature Flag Infrastructure ⏳ PARTIALLY COMPLETE
**Outcome:** Ability to toggle between old and new parser

**Status:** Feature flags defined in Cargo.toml files but conditional compilation not yet wired into builders. Awaiting GraphQLParser implementation in Phase 3.

**Tasks:**
1. Feature flags already added in Step 1.0, but verify configuration:
   - `/crates/libgraphql/Cargo.toml`:
     ```toml
     [features]
     default = ["macros"]
     macros = ["dep:libgraphql-macros"]
     use-libgraphql-parser = ["libgraphql-core/use-libgraphql-parser"]
     ```
   - `/crates/libgraphql-core/Cargo.toml`:
     ```toml
     [features]
     use-libgraphql-parser = ["dep:libgraphql-parser"]
     ```

2. Add conditional compilation in future parser call sites (template for later steps):
   ```rust
   #[cfg(feature = "use-libgraphql-parser")]
   use libgraphql_parser::GraphQLParser;

   #[cfg(not(feature = "use-libgraphql-parser"))]
   use graphql_parser;
   ```

3. Update CI to test both feature configurations (see Step 4.6)

4. **Documentation:**
   - Document feature flag in main README
   - Explain when to use which parser
   - Note that use-libgraphql-parser is opt-in initially

5. **Tests:**
   - `cargo build` succeeds with and without feature
   - `cargo test` passes with and without feature
   - `cargo clippy --tests` passes both ways

**Considerations:**
- Default stays with `graphql_parser` until new parser is battle-tested
- Allows gradual migration and confidence building
- Users can opt into new parser early for testing
- Feature name "use-libgraphql-parser" - will become default once stable

---

### Step 1.6: Create GraphQLParser ⏳ IN PROGRESS (REVISED APPROACH)
**Outcome:** Parser infrastructure in libgraphql-parser crate, ready for extension

**Status:** The original approach (migrating `GraphQLSchemaParser` from libgraphql-macros) has been **replaced** with building `GraphQLParser` from scratch. This provides a cleaner design and avoids inheriting limitations from the macro-specific parser. See `parser-design.md` for the comprehensive implementation plan.

**Revised Approach:**
Rather than moving and renaming the existing macro parser, we are building `GraphQLParser` fresh based on the GraphQL specification. The new parser will:
- Be generic over `GraphQLTokenSource`
- Support all three document types: schema, executable, and mixed
- Use the parse error infrastructure already implemented (Step 1.2)
- Provide error recovery for IDE use cases

**Module Structure (Current):**
```
libgraphql-parser/src/
├── lib.rs                           (re-exports public APIs)
├── ast.rs                           (AST wrappers around graphql-parser)
├── source_position.rs               (dual-column position tracking)
├── graphql_source_span.rs           (spans with optional file path)
├── graphql_token_stream.rs          (streaming lexer with lookahead)
├── graphql_parse_error.rs           (parse errors with notes)
├── graphql_parse_error_kind.rs      (10 error kind variants)
├── graphql_error_note.rs            (error notes: General/Help/Spec)
├── graphql_error_note_kind.rs       (note kind enum)
├── graphql_string_parsing_error.rs  (string escape errors)
├── value_parsing_error.rs           (int/float/string parsing errors)
├── definition_kind.rs               (schema vs operation definitions)
├── document_kind.rs                 (schema vs executable documents)
├── reserved_name_context.rs         (fragment names, enum values)
├── token/
│   ├── mod.rs
│   ├── graphql_token.rs             (token with trivia and span)
│   ├── graphql_token_kind.rs        (token kind variants)
│   └── graphql_trivia_token.rs      (comments and commas)
├── token_source/
│   ├── mod.rs
│   ├── graphql_token_source.rs      (marker trait)
│   └── str_to_graphql_token_source.rs (STUB - Phase 2)
└── tests/
```

**What Still Needs To Be Built:**
1. `StrToGraphQLTokenSource` implementation (Phase 2)
2. `GraphQLParser` implementation (per parser-design.md)
3. Integration with `libgraphql-macros` (update macro to use new parser)

---

#### Original Step 1.6 Tasks (Superseded - Preserved for Reference)

The original plan was to move parser files from libgraphql-macros and rename `GraphQLSchemaParser` to `GraphQLParser`. This approach was abandoned because:
1. The macro parser had limitations specific to proc-macro contexts
2. Building fresh allows for better error recovery design
3. The new parser can be designed around the complete GraphQL spec from the start

**Original Tasks:**
1. Create `/crates/libgraphql-parser/src/` module structure:
   ```
   libgraphql-parser/src/
   ├── lib.rs                        (re-exports public APIs)
   ├── ast.rs                        (from 1.1)
   ├── source_position.rs            (from 1.1)
   ├── graphql_token_stream.rs       (from 1.3)
   ├── graphql_parser.rs             (moved from macros)
   ├── graphql_parse_error.rs        (moved from macros)
   ├── token/
   │   ├── mod.rs                    (re-exports token types)
   │   ├── graphql_token.rs          (from 1.2)
   │   └── graphql_token_span.rs     (from 1.2)
   ├── token_source/
   │   ├── mod.rs                    (re-exports token source types)
   │   ├── graphql_token_source.rs   (from 1.2)
   │   └── str_to_graphql_token_source.rs (stub, implemented in Phase 2)
   └── tests/                        (moved from macros)
   ```
   Note: `RustMacroGraphQLTokenSource` stays in `libgraphql-macros` (Step 1.4)

2. Move parser files from `/crates/libgraphql-macros/src/`:
   - `graphql_schema_parser.rs` → `/crates/libgraphql-parser/src/graphql_parser.rs`
   - `graphql_parse_error.rs` → `/crates/libgraphql-parser/src/graphql_parse_error.rs`
   - Related test files → `/crates/libgraphql-parser/src/tests/`

3. Rename `GraphQLSchemaParser` → `GraphQLParser`:
   - Make it generic over `S: GraphQLTokenSource`
   - Keep schema parsing as `parse_schema_document()` method
   - Add stub methods that return `todo!()` for `parse_executable_document()` and `parse_mixed_document()` (implemented in Phase 3)

4. Update `libgraphql-macros` imports:
   - Import `GraphQLParser` from `libgraphql_parser`
   - **Do not re-export parser abstractions** from `libgraphql-macros`
   - Remove any existing public parser exports

5. Update `/crates/libgraphql/src/lib.rs`:
   ```rust
   #[cfg(feature = "use-libgraphql-parser")]
   pub use libgraphql_parser as parser;

   #[cfg(feature = "use-libgraphql-parser")]
   pub use libgraphql_parser::ast;

   #[cfg(not(feature = "use-libgraphql-parser"))]
   pub mod ast;  // Keep existing when feature disabled
   ```

6. **Tests:**
   - All existing parser tests pass
   - Verify proc macros still work
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- This is a large refactoring but purely organizational
- Existing functionality must remain unchanged
- No new features yet, just moving code
- Parser now in dedicated crate with clear boundaries
- No parser re-exports from macros crate (breaking change is intentional)

---

## Phase 2: String Lexer Implementation
*Goal: Implement StrToGraphQLTokenSource with full GraphQL spec compliance*

**Note:** Each step must pass `cargo clippy --tests` and `cargo test` with no warnings or errors before proceeding.

**Lexer Implementation Approach:**
We will implement a hand-written lexer for maximum control, clarity, and maintainability. GraphQL's lexical rules are straightforward enough that a hand-written approach provides:
1. Full control over error recovery and reporting
2. Clear, debuggable code for handling spec edge cases
3. Good performance without the overhead of a lexer generator framework

---

### Step 2.1: Basic String Lexer Structure
**Outcome:** Skeleton lexer that tokenizes simple cases, providing both UTF-8 and UTF-16 column tracking

**Tasks:**
1. Create `/crates/libgraphql-parser/src/token_source/str_to_graphql_token_source.rs`:
   ```rust
   use crate::SourcePosition;
   use crate::token::GraphQLToken;
   use crate::token::GraphQLTokenKind;
   use crate::token::GraphQLSourceSpan;
   use crate::token::GraphQLTriviaTokenVec;
   use crate::token_source::GraphQLTokenSource;

   /// A token source that lexes GraphQL from a string slice.
   ///
   /// This source provides full position tracking with both column representations:
   /// - `col_utf8()`: Character count within the line (always available)
   /// - `col_utf16()`: UTF-16 code unit offset within the line (`Some` value)
   ///
   /// Unlike `RustMacroGraphQLTokenSource` in the `libgraphql-macros` crate,
   /// this source scans the text character-by-character and can therefore track
   /// both column representations accurately.
   pub struct StrToGraphQLTokenSource<'a> {
       source: &'a str,
       /// Remaining source text (slice advances as we consume characters)
       remaining: &'a str,
       /// Current position tracking (mutable as we scan)
       line: usize,
       col_char: usize,
       col_utf16: usize,
       byte_offset: usize,
       /// Tracks if last character was \r (for \r\n handling)
       last_was_cr: bool,
       pending_trivia: GraphQLTriviaTokenVec,
       finished: bool,
   }

   impl<'a> StrToGraphQLTokenSource<'a> {
       pub fn new(source: &'a str) -> Self {
           Self {
               source,
               remaining: source,
               line: 0,
               col_char: 0,
               col_utf16: 0,
               byte_offset: 0,
               last_was_cr: false,
               pending_trivia: GraphQLTriviaTokenVec::new(),
               finished: false,
           }
       }

       /// Returns the current position as a SourcePosition.
       fn current_position(&self) -> SourcePosition {
           SourcePosition::new(
               self.line,
               self.col_char,
               Some(self.col_utf16),
               self.byte_offset,
           )
       }

       /// Advance position by a single character.
       ///
       /// Handles line terminators per GraphQL spec:
       /// - `\n` (U+000A): New line
       /// - `\r` (U+000D) not followed by `\n`: New line (legacy Mac)
       /// - `\r\n`: Single new line (Windows)
       fn advance_by_char(&mut self, ch: char) {
           // Handle \r\n sequence: if last char was \r and this is \n,
           // don't increment line again (already did for \r)
           if self.last_was_cr && ch == '\n' {
               self.last_was_cr = false;
               let char_len = ch.len_utf8();
               self.byte_offset += char_len;
               self.remaining = &self.remaining[char_len..];
               return;
           }

           if ch == '\n' || ch == '\r' {
               self.line += 1;
               self.col_char = 0;
               self.col_utf16 = 0;
               self.last_was_cr = ch == '\r';
           } else {
               self.col_char += 1;
               self.col_utf16 += ch.len_utf16();
               self.last_was_cr = false;
           }
           self.byte_offset += ch.len_utf8();
           self.remaining = &self.remaining[ch.len_utf8()..];
       }
   }

   impl<'a> Iterator for StrToGraphQLTokenSource<'a> {
       type Item = GraphQLToken;

       fn next(&mut self) -> Option<Self::Item> {
           // Skip whitespace using advance_by_char()
           // Accumulate trivia (comments, commas) in pending_trivia
           // When a non-trivia token is found:
           //   - Record start position via current_position()
           //   - Consume token characters via advance_by_char()
           //   - Record end position via current_position()
           //   - Return token with span and attached trivia
           // On EOF, emit GraphQLTokenKind::Eof with trailing trivia
           // ...
       }
   }

   impl<'a> GraphQLTokenSource for StrToGraphQLTokenSource<'a> {}
   ```

2. Implement basic tokenization:
   - Whitespace skipping (space, tab, newline per spec)
   - Single-character punctuation as `GraphQLTokenKind` variants: `Bang`, `Dollar`, `Ampersand`, `ParenOpen`, `ParenClose`, `Colon`, `Equals`, `At`, `SquareBracketOpen`, `SquareBracketClose`, `CurlyBraceOpen`, `CurlyBraceClose`, `Pipe`
   - Commas accumulated as `GraphQLTriviaToken::Comma`
   - Simple names (identifiers): `/[_A-Za-z][_0-9A-Za-z]*/`
   - Integer literals (basic cases, no validation yet)

3. Implement trivia accumulation in `Iterator::next()`:
   - Accumulate comments and commas in `pending_trivia` (using `GraphQLTriviaTokenVec`)
   - When a non-trivia token is scanned, attach `pending_trivia` to it
   - Track start/end positions for `GraphQLSourceSpan`
   - On EOF, return `GraphQLToken { kind: Eof, preceding_trivia, ... }`

4. **Error reporting with structured suggestions:**
   - Errors use `GraphQLErrorNotes` for "did you mean?" hints with optional spans
   - Suggestions target GraphQL authors (e.g., "Did you mean `String`?" when encountering `Stirng`)
   - Examples of error_notes:
     - Typos in type names
     - Missing punctuation (e.g., "Expected `:` after field name")
     - Common syntax mistakes

5. **Tests:**
   - Simple schema: `type Query { hello: String }`
   - Verify correct tokenization and positions
   - Tests should provide clear debugging info on failure
   - **Line ending tests:**
     - Verify `\n` increments line and resets both columns
     - Verify `\r` alone increments line and resets both columns
     - Verify `\r\n` increments line only once (not twice)
     - Verify positions are correct after mixed line endings
   - **Dual column tracking tests:**
     - Verify ASCII characters advance both `col_utf8()` and `col_utf16()` equally
     - Verify multi-byte UTF-8 characters (e.g., `é`) advance `col_utf8()` by 1
       and `col_utf16()` by UTF-16 length
     - Verify characters outside BMP (e.g., emoji 🎉) advance `col_utf8()` by 1 and
       `col_utf16()` by 2 (surrogate pair)
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- This is a foundation; edge cases come in later steps
- Focus on correct position tracking from the start
- Error handling includes helpful suggestions and notes
- Renamed from StringToGraphQLTokenAdapter → StrToGraphQLTokenSource (operates on &str, not String)
- `last_was_cr` state is internal to the lexer, not exposed in `SourcePosition`

---

### Step 2.2: Comments and Multi-Character Punctuation
**Outcome:** Handle GraphQL comments and spread operator

**Tasks:**
1. Add comment handling:
   - Recognize `#` followed by anything until newline
   - Accumulate as `GraphQLTriviaToken::Comment` in `pending_trivia`
   - Comments are attached to the next non-trivia token
   - Update position tracking for comment content

2. Add spread operator (`...`):
   - Recognize `...` (three consecutive dots)
   - Emit as `GraphQLTokenKind::Ellipsis`
   - Note: A single `.` or `..` is a lexer error in GraphQL (there is no single-dot token)

3. **Tests:**
   - Schema with comments: `# This is a comment\ntype Query { ... }`
   - Spread in fragments
   - Comments at EOF (attached to Eof token)
   - Verify comments appear in `preceding_trivia`
   - Single `.` produces error token
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Comments accumulated as trivia, attached to following token
- EOF token carries any trailing comments
- Spread operator requires 2-character lookahead
- Position tracking must account for comment content

---

### Step 2.3: String Literal Lexing
**Outcome:** Full GraphQL string support including escape sequences

**Tasks:**
1. Implement single-line strings:
   - Recognize `"..."` delimiters
   - Handle escape sequences per spec §2.9.4:
     - `\"`, `\\`, `\/`, `\b`, `\f`, `\n`, `\r`, `\t`
     - Unicode escapes (fixed 4-digit): `\uXXXX`
     - Unicode escapes (variable length): `\u{X}` to `\u{XXXXXX}`
   - Reject unescaped line breaks
   - Reject unescaped control characters

2. Implement block strings (triple-quoted):
   - Recognize `"""..."""` delimiters
   - Allow unescaped quotes, line breaks
   - Implement indentation stripping algorithm (spec §2.9.4):
     ```
     1. Split into lines
     2. Find common indentation (excluding first/last line)
     3. Strip common indentation
     4. Remove leading/trailing blank lines
     ```
   - Handle escaped triple-quotes: `\"""`

3. **Tests (comprehensive):**
   - Basic strings: `"hello"`, `"hello world"`
   - Escapes: `"line1\nline2"`, `"quote: \"hi\""`
   - Fixed-length unicode: `"\u0041\u0042\u0043"` → `"ABC"`
   - Variable-length unicode: `"\u{1F600}"` → emoji
   - Block strings with indentation
   - Block strings with blank lines
   - Edge cases from graphql-js test suite

**Considerations:**
- This is the most complex lexing logic
- String processing is performance-critical (consider `String::with_capacity`)
- Thorough testing is essential; many edge cases
- Reference: graphql-js lexer implementation

---

### Step 2.4: Numeric Literal Lexing
**Outcome:** Spec-compliant integer and float parsing

**Note:** Negative numbers (e.g., `-123`) are lexed as single tokens, not as separate
minus and number tokens. The raw string stored in `IntValue`/`FloatValue` includes the
optional negative sign.

**Tasks:**
1. Implement integer parsing:
   - Valid: `0`, `123`, `-456`
   - Invalid: `00`, `01`, leading zeros except `0` itself
   - Handle negative sign as part of the token
   - Range validation: i64::MIN to i64::MAX
   - Emit as `IntValue("-123")` (single token including sign)

2. Implement float parsing:
   - Recognize patterns:
     - Decimal: `1.23`, `0.5`, `-1.5`
     - Exponent: `1e10`, `1E10`, `1e+10`, `1e-10`
     - Both: `1.23e10`, `-1.23e-10`
   - Invalid: `1.` (must have digits after decimal)
   - Parse to f64
   - Emit as `FloatValue("-1.23e10")` (single token including sign)

3. Disambiguate integer vs float:
   - Presence of `.` or `e`/`E` → float
   - Otherwise → integer

4. **Tests:**
   - Valid integers: `0`, `123`, `-456`
   - Invalid integers: `00`, `01`
   - Valid floats: `1.0`, `1e10`, `1.23e-5`, `-3.14`
   - Invalid floats: `1.`, `.5`, `1e`
   - Boundary cases: i64::MAX, f64::MAX
   - Verify negative numbers are single tokens

**Considerations:**
- Use Rust's `parse::<i64>()` and `parse::<f64>()` after validation
- Negative sign is consumed as part of the number token (not separate)
- Error messages must be clear

---

### Step 2.5: Name Validation and Keywords
**Outcome:** Proper name lexing with spec validation

**Tasks:**
1. Implement name lexing:
   - Must match `/[_A-Za-z][_0-9A-Za-z]*/`
   - Reject names starting with digits
   - Emit `GraphQLTokenKind::Name(String)` for most names

2. Emit distinct tokens for `true`, `false`, `null`:
   - `true` → `GraphQLTokenKind::True`
   - `false` → `GraphQLTokenKind::False`
   - `null` → `GraphQLTokenKind::Null`
   - These are _usually_ literals in GraphQL, but are sometimes also valid as identifiers (e.g. field names).
   - Since we expect these specific tokens to be common, we model them as a known token (to minimize string-allocs during lexing) and interpret them as either identifiers or "keywords" in the parser depending on parsing context.

3. **Tests:**
   - Valid names: `hello`, `_private`, `type2`
   - Invalid names: `2type`, `hello-world`
   - Boolean literals: `true`, `false` → distinct token kinds
   - Null literal: `null` → distinct token kind
   - Keywords as names: `type`, `query`, `mutation` (context-dependent, always Name tokens)
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Most GraphQL keywords are context-dependent (e.g., `type` can be a field name)
- `true`, `false`, `null` ARE context-dependent; parser decides based on context how to interpret them
- Using distinct tokens for boolean/null improves type safety
- Name validation is straightforward regex
- Parser handles context, lexer emits appropriate token types

---

### Step 2.6: Error Recovery in Lexer
**Outcome:** Lexer recovers from errors and continues tokenizing

**Tasks:**
1. Emit `GraphQLTokenKind::Error` for invalid input:
   - Include `message` and `error_notes` fields
   - Error tokens also carry `preceding_trivia`
   - Continue lexing after error (don't return `None` early)

2. Implement recovery strategies:
   - Invalid character: Emit error token, skip character, continue
   - Unterminated string: Emit error, skip to newline or EOF
   - Invalid number: Emit error, skip to next whitespace

3. Parser handles error tokens:
   - Accumulate errors instead of failing immediately
   - Continue parsing to find multiple errors

4. **Tests:**
   - Multiple errors in single document
   - Error recovery doesn't skip valid tokens
   - Error positions are accurate
   - Errors have preceding trivia attached
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Matches `RustMacroGraphQLTokenSource` behavior for consistency
- User experience: show all errors, not just first one
- Balance recovery quality with complexity

---

### Step 2.7: Comprehensive Lexer Testing
**Outcome:** Lexer is battle-tested and spec-compliant

**Tasks:**
1. **License review for graphql-js tests:**
   - Check graphql-js license (https://github.com/graphql/graphql-js/blob/main/LICENSE)
   - Verify license permits copying/adapting test cases
   - Document the license in vendored test files (e.g., header comment with attribution)
   - If license has restrictions, evaluate alternatives:
     - Write equivalent tests from scratch based on GraphQL spec
     - Use only test *inputs* (which may not be copyrightable) and write own assertions
   - **Do not proceed with vendoring until license is verified as compatible**

2. Port test cases from graphql-js (after license approval):
   - Clone https://github.com/graphql/graphql-js
   - Extract lexer tests from `src/__tests__/lexer-test.ts`
   - Convert to Rust test cases in `/crates/libgraphql-parser/src/tests/str_to_graphql_token_source_tests.rs`
   - **Vendor the tests** (include them in the repo) for reproducibility
   - Include license header/attribution in vendored files
   - Ensure 100% of graphql-js lexer tests pass
   - All tests must provide **clear error information useful for debugging** if they fail

3. **License review for graphql-parser tests:**
   - Check graphql-parser license (https://github.com/graphql-rust/graphql-parser)
   - Verify license permits copying/adapting test cases
   - Document the license in vendored test files
   - **Do not proceed with vendoring until license is verified as compatible**

4. Port test cases from graphql-parser (after license approval):
   - Clone https://github.com/graphql-rust/graphql-parser
   - Extract relevant lexer/parser tests
   - **Vendor the tests** in the repo with license attribution
   - Convert to Rust test cases
   - Ensure compatibility
   - Clear debugging output on test failures

5. Add fuzzing tests:
   - Use `cargo-fuzz` or `proptest`
   - Generate random GraphQL-like strings
   - Verify lexer doesn't panic
   - Document any interesting fuzz-discovered issues

6. Benchmark lexer performance:
   - Compare against `graphql_parser` on various inputs
   - Identify performance bottlenecks
   - Optimize hot paths (string scanning, allocation)
   - **Check benchmark fixture files into source control** (e.g., GitHub/GitLab schema files in `/crates/libgraphql-parser/benches/fixtures/`)
   - **Verify license compatibility** before vendoring real-world schemas

7. **Success criteria:**
   - 100% graphql-js lexer tests pass
   - 100% graphql-parser lexer tests pass
   - No panics on fuzz tests
   - Performance within 2x of `graphql_parser`
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- This step is non-negotiable; comprehensive testing with exhaustive test suite is mandatory
- Vendored tests ensure reproducibility and offline development
- Clear error messages in test failures aid debugging
- May uncover edge cases requiring lexer fixes
- Performance baseline for future optimization

---

## Phase 3: Parser Extension for Operations
*Goal: Extend parser to handle executable documents and mixed documents*

**Note:** Each step must pass `cargo clippy --tests` and `cargo test` with no warnings or errors before proceeding.

### Step 3.1: Parse Operation Definitions
**Outcome:** Parser can handle query/mutation/subscription operations

**Tasks:**
1. Add operation parsing to `/crates/libgraphql-parser/src/graphql_parser.rs`:
   - Recognize `query`, `mutation`, `subscription` keywords
   - Parse operation name (optional)
   - Parse variable definitions: `($var: Type = default)`
   - Parse directives on operations
   - Parse selection sets

2. Implement selection set parsing:
   - Field selections: `fieldName(args) @directives { ... }`
   - Fragment spreads: `...FragmentName`
   - Inline fragments: `... on Type { ... }`

3. Parse field arguments:
   - `field(arg1: value1, arg2: value2)`
   - Support all value types: int, float, string, boolean, null, enum, list, object

4. Parse variable definitions:
   - `($varName: TypeRef = defaultValue @directives)`
   - Type references: `Type`, `Type!`, `[Type]`, `[Type!]!`

5. **Tests:**
   - Simple query: `{ hello }`
   - Named query: `query GetUser { user { name } }`
   - Variables: `query GetUser($id: ID!) { user(id: $id) { name } }`
   - Directives: `query @cached { ... }`
   - Nested selections

**Considerations:**
- Operations use `ast::operation::*` types (from graphql_parser)
- Parser must handle both schema and operation syntax
- Selection sets can nest deeply (stack depth consideration)

**Future Configuration Options (TODO):**
Add a `GraphQLParserOptions` struct in a future iteration to configure:
- `max_selection_depth: Option<usize>` - Limit nesting depth (DoS protection)
- `max_string_literal_size: Option<usize>` - Limit string literal length
- `max_list_literal_size: Option<usize>` - Limit list literal elements
- `max_input_object_fields: Option<usize>` - Limit input object fields

---

### Step 3.2: Parse Fragment Definitions
**Outcome:** Parser can handle fragment definitions

**Tasks:**
1. Add fragment parsing to produce `ast::operation::FragmentDefinition`:
   - Recognize `fragment` keyword
   - Parse fragment name
   - Parse type condition: `on TypeName`
   - Parse directives
   - Parse selection set
   - Return `ast::operation::FragmentDefinition` structure

2. Handle fragment spreads in selections:
   - `...FragmentName` with optional directives

3. Handle inline fragments in selections:
   - `... on TypeName @directives { ... }`
   - `... @directives { ... }` (no type condition)

4. **Tests:**
   - Fragment definition: `fragment UserFields on User { id name }`
   - Fragment spread: `{ user { ...UserFields } }`
   - Inline fragment: `{ user { ... on Admin { role } } }`
   - Nested fragments
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Fragments parse to the existing `ast::operation::FragmentDefinition` structure from graphql_parser
- Fragments reference types from schema (validation is separate concern)
- Fragments can spread other fragments (no cycle detection in parser)

---

### Step 3.3: Implement parse_executable_document()
**Outcome:** Dedicated method for parsing operations and fragments

**Tasks:**
1. Define result type that includes errors AND salvaged AST:
   ```rust
   pub struct ParseResult<T> {
       pub ast: Option<T>,
       pub errors: Vec<ParseError>,
   }
   ```

   **Note on error types:**
   - `ParseError` is an enum emitted by the parser for syntax errors
   - Each variant represents a specific error (parsing can produce multiple `ParseError`s)
   - `CookGraphQLStringError` can be wrapped in a `ParseError` variant for string processing failures
   - Existing builder errors (`SchemaBuildError`, `QueryBuildError`, etc.) in `libgraphql-core` remain separate; they handle semantic validation after parsing

2. Add method to `GraphQLParser`:
   ```rust
   pub fn parse_executable_document(self) -> ParseResult<ast::operation::Document> {
       let mut definitions = Vec::new();
       let mut errors = Vec::new();

       while !self.tokens.is_at_end() {
           match self.parse_executable_definition() {
               Ok(def) => definitions.push(def),
               Err(err) => {
                   errors.push(err);
                   // If we encounter non-executable syntax (schema definitions),
                   // emit error and skip to next definition
                   if self.is_schema_definition() {
                       errors.push(ParseError::new(
                           "Unexpected type definition in executable document",
                           /* location */,
                           vec![
                               "Remove type definitions from this document".to_string(),
                               "Move type definitions to a separate schema file".to_string(),
                           ],
                       ));
                       self.skip_schema_definition();
                   } else {
                       self.recover_to_next_definition();
                   }
               }
           }
       }

       ParseResult {
           ast: if definitions.is_empty() { None } else { Some(ast::operation::Document { definitions }) },
           errors,
       }
   }
   ```

3. Implement `parse_executable_definition()`:
   - Peek at token to determine type:
     - `query`/`mutation`/`subscription` → operation
     - `fragment` → fragment definition
     - `{` → anonymous query (shorthand)
   - Delegate to appropriate parser
   - **Error on schema definitions**: Emit error with helpful message

4. Implement error recovery:
   - `skip_schema_definition()`: Skip non-executable syntax
   - Continue parsing after error
   - **TODO: Explore "synchronization sets per grammar region"** for smarter recovery:
     ```
     Instead of always skipping to next definition, define context-aware sync tokens:
     - Top level: sync on `schema`/`type`/`interface`/`union`/`enum`/`scalar`/
       `input`/`directive`/`extend`/`query`/`mutation`/`subscription`/`fragment`/`{`
     - Inside selection sets: sync on `...`, Name, `}`
     - Inside input objects: sync on Name, `}`
     - Inside argument lists: sync on Name, `)`
     - Inside type references: sync on `]`, `!`, `)`, `=`, `@`, `{`
     - Inside directive arguments: sync on Name, `)`, `@`
     ```
     This enables recovery at finer granularity (e.g., recover within a single
     field definition rather than discarding the entire type).

5. **Tests:**
   - Document with single operation
   - Document with multiple operations
   - Document with operations and fragments
   - Anonymous queries
   - **Mixed document to executable parser**: Should error but recover
   - Error recovery across multiple definitions
   - Verify salvaged AST is usable even with errors
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Similar structure to `parse_schema_document()`
- Error recovery: skip to next definition on parse error
- Suggestions target GraphQL authors, not API users (e.g., "Move type definitions to a separate schema file")
- Returns both errors and salvaged AST for partial success

---

### Step 3.4: Implement parse_mixed_document()
**Outcome:** Parse documents containing both schema and executable definitions

**Tasks:**
1. Define unified document type in `/crates/libgraphql-parser/src/mixed_document.rs`:
   ```rust
   /// A definition in a mixed document - either schema or executable.
   pub enum MixedDocumentDefinition {
       Schema(ast::schema::Definition),
       Executable(ast::operation::Definition),
   }

   /// A document containing both schema and executable definitions.
   /// Preserves original ordering for tools like formatters and printers.
   pub struct MixedDocument {
       definitions: Vec<MixedDocumentDefinition>,
   }

   impl MixedDocument {
       pub fn definitions(&self) -> &[MixedDocumentDefinition] {
           &self.definitions
       }

       /// Returns only schema definitions (loses interleaving order).
       pub fn schema_definitions(&self) -> impl Iterator<Item = &ast::schema::Definition> {
           self.definitions.iter().filter_map(|d| match d {
               MixedDocumentDefinition::Schema(def) => Some(def),
               _ => None,
           })
       }

       /// Returns only executable definitions (loses interleaving order).
       pub fn executable_definitions(&self) -> impl Iterator<Item = &ast::operation::Definition> {
           self.definitions.iter().filter_map(|d| match d {
               MixedDocumentDefinition::Executable(def) => Some(def),
               _ => None,
           })
       }
   }
   ```

2. Add helper functions for keyword classification:
   ```rust
   /// Returns true if the name is a schema definition keyword.
   fn is_schema_keyword(name: &str) -> bool {
       matches!(name, "type" | "interface" | "union" | "enum" | "scalar" |
                      "input" | "directive" | "schema" | "extend")
   }

   /// Returns true if the name is an executable definition keyword.
   fn is_exec_keyword(name: &str) -> bool {
       matches!(name, "query" | "mutation" | "subscription" | "fragment")
   }
   ```

3. Add method to `GraphQLParser`:
   ```rust
   pub fn parse_mixed_document(self) -> ParseResult<MixedDocument> {
       let mut definitions = Vec::new();
       let mut errors = Vec::new();

       while !self.tokens.is_at_end() {
           match self.tokens.peek() {
               Some(GraphQLToken { kind: GraphQLTokenKind::Name(kw), .. })
                   if is_schema_keyword(kw) =>
               {
                   match self.parse_schema_definition() {
                       Ok(def) => definitions.push(MixedDocumentDefinition::Schema(def)),
                       Err(err) => {
                           errors.push(err);
                           self.recover_to_next_definition();
                       }
                   }
               }
               Some(GraphQLToken { kind: GraphQLTokenKind::Name(kw), .. })
                   if is_exec_keyword(kw) =>
               {
                   match self.parse_executable_definition() {
                       Ok(def) => definitions.push(MixedDocumentDefinition::Executable(def)),
                       Err(err) => {
                           errors.push(err);
                           self.recover_to_next_definition();
                       }
                   }
               }
               Some(GraphQLToken { kind: GraphQLTokenKind::CurlyBraceOpen, .. }) => {
                   // Anonymous query (shorthand)
                   match self.parse_executable_definition() {
                       Ok(def) => definitions.push(MixedDocumentDefinition::Executable(def)),
                       Err(err) => {
                           errors.push(err);
                           self.recover_to_next_definition();
                       }
                   }
               }
               Some(GraphQLToken { kind: GraphQLTokenKind::Eof, .. }) => {
                   break;
               }
               _ => {
                   errors.push(ParseError::new(
                       "Unexpected token in document",
                       /* location */,
                       vec!["Expected a type definition, operation, or fragment".to_string()],
                   ));
                   self.recover_to_next_definition();
               }
           }
       }

       ParseResult {
           ast: if definitions.is_empty() { None } else { Some(MixedDocument { definitions }) },
           errors,
       }
   }
   ```

4. **Tests:**
   - Mixed document: schema types + operations
   - Mixed document: schema types + fragments
   - Mixed document: operations + fragments + schema
   - Error handling in mixed documents
   - Salvaged AST with partial errors
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- This is the primary use case driving the refactor
- Must handle interleaved definitions
- Error in schema definition shouldn't prevent parsing operations
- Fields are private with accessor methods
- Returns both errors and salvaged AST
- Errors include helpful error_notes

---

### Step 3.5: Comprehensive Parser Testing
**Outcome:** Parser is battle-tested for all document types

**Tasks:**
1. **Verify license compatibility** (if not already done in Step 2.7):
   - License checking for graphql-js and graphql-parser should have been completed in Step 2.7
   - If skipped earlier, complete license review before proceeding
   - Ensure all vendored tests include proper license attribution

2. Port operation/fragment tests from graphql-js (license permitting):
   - Extract parser tests from graphql-js test suite
   - Convert to Rust tests
   - Include license attribution headers
   - Ensure 100% coverage of operation syntax

3. Port tests from graphql-parser (license permitting):
   - Ensure compatibility with existing test expectations
   - Include license attribution headers

4. **Differential testing against `graphql_parser`**:
   - Parse same inputs with both `libgraphql-parser` and `graphql_parser`
   - Compare success/failure outcomes (both should succeed or both should fail)
   - Compare AST "shape" (structure should match, ignoring span differences)
   - Use a corpus of real-world schemas (GitHub, GitLab, etc.) and operations
   - Any discrepancies should be investigated and documented:
     - If `graphql_parser` is wrong, document the spec violation
     - If `libgraphql-parser` is wrong, fix it
   - This builds confidence that the new parser is a drop-in replacement

5. Add edge case tests:
   - Deeply nested selections
   - Large documents (performance)
   - Documents with many errors (error recovery)
   - Mixed documents with various combinations

6. Add regression tests for discovered bugs

7. **Success criteria:**
   - 100% graphql-js parser tests pass (for covered syntax)
   - 100% graphql-parser tests pass
   - Differential tests show equivalent behavior
   - No panics on malformed input

**Considerations:**
- Parser testing is as critical as lexer testing
- Test both success and error cases
- Test error recovery quality

---

## Phase 4: Integration & Migration
*Goal: Wire up new parser in SchemaBuilder/QueryBuilder, enable feature flag*

**Note:** Each step must pass `cargo clippy --tests` and `cargo test` with no warnings or errors before proceeding.

### Step 4.1: Integrate Parser in SchemaBuilder
**Outcome:** SchemaBuilder can use new parser when feature enabled

**Tasks:**
1. Update `/crates/libgraphql-core/src/schema/schema_builder.rs`:
   - Add conditional compilation with **same function name**:
     ```rust
     #[cfg(feature = "use-libgraphql-parser")]
     pub fn build_from_str(&mut self, source: &str) -> Result<&mut Self, SchemaBuildError> {
         use libgraphql_parser::GraphQLParser;
         use libgraphql_parser::token_source::StrToGraphQLTokenSource;

         let token_source = StrToGraphQLTokenSource::new(source);
         let parser = GraphQLParser::new(token_source);
         let result = parser.parse_schema_document();

         // Handle ParseResult (errors + salvaged AST)
         if !result.errors.is_empty() {
             // Convert to SchemaBuildError
         }
         if let Some(document) = result.ast {
             self.process_document(document)
         } else {
             Err(/* error */)
         }
     }

     #[cfg(not(feature = "use-libgraphql-parser"))]
     pub fn build_from_str(&mut self, source: &str) -> Result<&mut Self, SchemaBuildError> {
         // Existing graphql_parser implementation
         use crate::ast;

         let document = ast::schema::parse(source)?;
         self.process_document(document)
     }
     ```

2. **Use statement style**:
   - Each `use` statement imports exactly one symbol
   - Sorted alphabetically by module path
   - No compound `use` statements with curly braces

3. Keep both implementations during transition:
   - `use-libgraphql-parser` feature selects new parser
   - Default uses `graphql_parser`

4. **Tests:**
   - Run all existing schema builder tests with both feature flags
   - **Add equivalence tests**: Verify both parsers produce identical output for same input
   - Tests are comprehensive and will be useful until we remove graphql_parser
   - `cargo clippy --tests` passes
   - `cargo test` passes (both with and without feature)

**Considerations:**
- Gradual migration path
- Can compare outputs between parsers via equivalence tests
- Bug compatibility may differ (new parser may be more/less strict)
- Same function name across feature flags (mutually exclusive compilation)
- Proper use statement style per codebase conventions

---

### Step 4.2: Integrate Parser in QueryBuilder/MutationBuilder/SubscriptionBuilder
**Outcome:** Operation builders use new parser when feature enabled

**Tasks:**
1. Update operation builders:
   - `/crates/libgraphql-core/src/operation/query_builder.rs`
   - `/crates/libgraphql-core/src/operation/mutation_builder.rs`
   - `/crates/libgraphql-core/src/operation/subscription_builder.rs`

2. Add conditional compilation with **same function name**:
   ```rust
   #[cfg(feature = "use-libgraphql-parser")]
   pub fn build_from_str(&mut self, source: &str) -> Result<...> {
       use libgraphql_parser::GraphQLParser;
       use libgraphql_parser::token_source::StrToGraphQLTokenSource;

       let token_source = StrToGraphQLTokenSource::new(source);
       let parser = GraphQLParser::new(token_source);
       let result = parser.parse_executable_document();

       // Handle ParseResult (errors + salvaged AST)
       // ...
   }

   #[cfg(not(feature = "use-libgraphql-parser"))]
   pub fn build_from_str(&mut self, source: &str) -> Result<...> {
       use crate::ast;

       let document = ast::operation::parse(source)?;
       // ... existing implementation
   }
   ```

3. **Use statement style**:
   - Each `use` statement imports exactly one symbol
   - Sorted alphabetically by module path
   - No compound `use` statements

4. **Tests:**
   - Run all operation builder tests with both feature flags
   - **Add equivalence tests**: Comprehensive suite verifying both parsers produce identical output
   - Tests useful for confidence until graphql_parser is removed
   - `cargo clippy --tests` passes
   - `cargo test` passes (both with and without feature)

**Considerations:**
- Operation builders currently use `graphql_parser::query::parse_query`
- New parser should produce identical AST structures
- Any differences need investigation
- Same function name across feature flags
- Equivalence tests build confidence in new parser

---

### Step 4.3: Add Mixed Document Support
**Outcome:** New API for parsing mixed documents

**Tasks:**
1. Create new builder in `/crates/libgraphql-core/src/mixed_document_builder.rs`:
   ```rust
   #[cfg(feature = "use-libgraphql-parser")]
   mod mixed_document_builder {
       /// Builder for parsing GraphQL documents containing both schema and executable definitions.
       ///
       /// This builder allows parsing documents that mix type definitions, directive definitions,
       /// operations, and fragments in a single file. This is useful for GraphQL tooling that
       /// processes complete GraphQL codebases.
       ///
       /// # Examples
       ///
       /// ```rust
       /// use libgraphql::MixedDocumentBuilder;
       ///
       /// let mut builder = MixedDocumentBuilder::new();
       /// builder.build_from_str(r#"
       ///     type Query {
       ///         hello: String
       ///     }
       ///
       ///     query GetHello {
       ///         hello
       ///     }
       /// "#)?;
       /// ```
       ///
       /// # Feature Flag
       ///
       /// This functionality requires the `use-libgraphql-parser` feature flag.
       pub struct MixedDocumentBuilder {
           schema_builder: SchemaBuilder,
           executable_builder: ExecutableDocumentBuilder,
       }

       impl MixedDocumentBuilder {
           pub fn build_from_str(&mut self, source: &str) -> Result<...> {
               use libgraphql_parser::GraphQLParser;
               use libgraphql_parser::token_source::StrToGraphQLTokenSource;

               let token_source = StrToGraphQLTokenSource::new(source);
               let parser = GraphQLParser::new(token_source);
               let result = parser.parse_mixed_document();

               // Handle errors
               if !result.errors.is_empty() {
                   // ...
               }

               if let Some(doc) = result.ast {
                   self.schema_builder.process_definitions(doc.schema_definitions())?;
                   self.executable_builder.process_definitions(doc.executable_definitions())?;
                   Ok(self)
               } else {
                   Err(/* error */)
               }
           }
       }
   }

   #[cfg(feature = "use-libgraphql-parser")]
   pub use mixed_document_builder::MixedDocumentBuilder;
   ```

2. Expose in public API:
   - Add to `/crates/libgraphql/src/lib.rs` with `#[cfg(feature = "use-libgraphql-parser")]`
   - Only available with `use-libgraphql-parser` feature
   - **Include clear and extensive rustdoc comments** for docs.rs

3. **Tests:**
   - Parse mixed document
   - Validate schema types are accessible
   - Validate operations are accessible
   - Integration test with real use case
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- This is net-new functionality (solving the original problem)
- Only available with new parser (guarded by `#[cfg(feature = "use-libgraphql-parser")]`)
- Should encourage adoption of use-libgraphql-parser feature
- Rustdoc provides clear documentation for users
- Use correct use statement style

---

### Step 4.4: Performance Benchmarking and Regression Tests
**Outcome:** Quantified performance comparison vs graphql_parser with automated regression detection

**Tasks:**
1. Create benchmarks in `/crates/libgraphql-parser/benches/parser_bench.rs`:
   - Use `criterion` crate
   - Benchmark scenarios:
     - Small schema (10 types)
     - Medium schema (100 types)
     - Large schema (1000 types) - use real schemas (GitHub, GitLab GraphQL APIs)
     - Small operation
     - Complex operation (deep nesting)
     - Mixed document

2. Compare performance:
   - `graphql_parser` baseline
   - New parser (both lexer and parser)
   - Identify bottlenecks

3. **Create benchmarks for local development**:
   - Benchmarks are for local profiling and optimization work
   - **Do NOT run benchmarks in CI** - CI environments have too much variability for reliable performance measurement
   - Developers can run `cargo bench` locally when investigating performance

4. Optimize hot paths:
   - String allocation
   - Token buffer management
   - AST node construction

5. Document results:
   - Add benchmark results to README
   - Note acceptable performance threshold (within 2x)

**Considerations:**
- Performance regression is acceptable if < 2x slower (verified locally, not in CI)
- Functionality and correctness > raw speed
- Optimization is iterative; don't over-optimize initially
- Real-world schemas (GitHub/GitLab) provide realistic benchmark fixtures
- **Verify license compatibility** before vendoring real-world schemas

---

### Step 4.5: Documentation and Examples
**Outcome:** Users understand how to use new parser

**Tasks:**
1. Update `/README.md`:
   - Document `use-libgraphql-parser` feature flag
   - Explain benefits of new parser
   - Migration guide from `graphql_parser`

2. Update rustdoc:
   - Add module-level docs to `/crates/libgraphql-parser/src/lib.rs`
   - Explain architecture (lexer → token stream → parser)
   - Code examples for each parser method

3. Add examples:
   - `/examples/parse_schema.rs` - Parse schema with new parser
   - `/examples/parse_query.rs` - Parse operation with new parser
   - `/examples/parse_mixed.rs` - Parse mixed document (showcase feature)

4. Update CLAUDE.md:
   - Document parser architecture
   - Document when to use which parser method
   - Add to testing conventions

**Considerations:**
- Documentation is critical for adoption
- Examples show real-world usage
- Should encourage migration to libgraphql-parser

---

### Step 4.6: CI/CD Updates
**Outcome:** Both parsers tested in CI, confidence in new parser

**Tasks:**
1. Update GitHub Actions workflow:
   - Current CI already runs `cargo test` (verify this)
   - Add job with `--features use-libgraphql-parser`
   - Add job without feature (default) - likely already exists
   - Matrix build: test both configurations

2. **Verify existing coverage** (likely already in place):
   - `cargo test` should cover most needs
   - Check if fuzzing runs exist

3. Add parser-specific jobs **if not already covered by cargo test**:
   - Lexer fuzzing (may need separate job)
   - Parser fuzzing (may need separate job)
   - **Note:** Benchmarks are NOT run in CI (too variable); developers run locally

4. Code coverage (if not already in place):
   - Ensure parser code has high coverage
   - Use `cargo-tarpaulin` or `cargo-llvm-cov`

5. **Success criteria:**
   - All tests pass with both feature flags
   - Code coverage > 90% for parser module

**Considerations:**
- CI validates both code paths
- Prevents regressions in either parser
- Builds confidence for eventual default switch
- Most needs likely covered by existing `cargo test` infrastructure
- Main addition: testing with feature flag enabled

---

### Step 4.7: Default Feature Flag Flip
**Outcome:** use-libgraphql-parser becomes the default

**Tasks:**
1. Update `/crates/libgraphql/Cargo.toml`:
   ```toml
   [features]
   default = ["macros", "use-libgraphql-parser"]
   macros = ["dep:libgraphql-macros"]
   use-libgraphql-parser = ["libgraphql-core/use-libgraphql-parser"]
   ```

2. Update documentation:
   - Note that `use-libgraphql-parser` is now default
   - Users can disable with `default-features = false` if needed
   - Plan to remove `graphql_parser` dependency in future release

3. **Deprecation plan:**
   - Announce in release notes
   - Support disabling for 2-3 releases
   - Eventually remove `graphql_parser` dependency entirely

**Considerations:**
- Only flip default after confidence is high (weeks/months of testing)
- Provide escape hatch for users who hit issues
- Clear communication in changelog

---

## Success Criteria

**Phase 1 Complete:**
- All parser infrastructure moved to `libgraphql-parser` crate
- `GraphQLTokenSource` trait implemented
- Feature flag infrastructure in place
- All existing tests pass

**Phase 2 Complete:**
- `StrToGraphQLTokenSource` fully implements GraphQL lexer spec
- 100% graphql-js lexer tests pass
- 100% graphql-parser lexer tests pass
- Performance within 2x of `graphql_parser`

**Phase 3 Complete:**
- Parser handles schema, executable, and mixed documents
- 100% graphql-js parser tests pass (for covered syntax)
- 100% graphql-parser parser tests pass

**Phase 4 Complete:**
- New parser integrated in all builders
- Mixed document support available
- Comprehensive benchmarks documented
- Documentation and examples complete
- CI validates both parsers
- (Eventually) `libgraphql-parser` is the default

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Lexer bugs causing incorrect parses | High | Comprehensive test suite from graphql-js/graphql-parser |
| Performance regression | Medium | Benchmarking, optimization, 2x threshold acceptable |
| AST incompatibility with graphql_parser | High | Extensive integration testing, feature flag escape hatch |
| Breaking changes during migration | Medium | Feature flag allows gradual migration |
| Maintenance burden of two parsers | Low | Deprecation plan for legacy parser |

---

## Future Work (Out of Scope)

- Custom AST types (replace `graphql_parser` types entirely)
- **Synchronization sets per grammar region** - Context-aware error recovery that syncs on different tokens depending on parsing context (selection sets, argument lists, type refs, etc.) for finer-grained recovery
- Richer diagnostics structure for error_notes (spans, severity, fix actions)
- Incremental parsing for IDE support
- WASM compilation for browser use
- Streaming parser for very large documents
- Zero-copy lexing with `Cow<'a, str>` optimization
