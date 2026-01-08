# Implementation Plan: Unified GraphQL Parser Infrastructure - Phase 1

This document contains the Phase 1 implementation details extracted from `libgraphql-parser-plan.v8.md`.

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
- All errors include helpful suggestions and "did-you-mean" hints when possible
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

### Step 1.0: Create libgraphql-parser Crate
**Outcome:** New crate structure ready for parser implementation

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
   smallvec = "1.15"
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
  - `libgraphql_parser::token::` - Token types: `GraphQLToken`, `GraphQLTokenKind`, `GraphQLTriviaToken`, `GraphQLTriviaTokenVec`, `GraphQLTokenSpan`, `CookGraphQLStringError`
  - `libgraphql_parser::token_source::` - Token sources: `GraphQLTokenSource`, `StrToGraphQLTokenSource`
  - `libgraphql_parser::ast::` - AST types: `MixedDocument`, `MixedDocumentDefinition`
  - `libgraphql_parser::` (root) - `GraphQLParser`, `GraphQLTokenStream`, `SourcePosition`, `GraphQLSuggestionVec`, `SmallVec`

---

### Step 1.1: Copy ast Module and Create SourcePosition
**Outcome:** Location tracking ready for all token sources; ast module in parser crate

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

### Step 1.2: Create GraphQLTokenSource Trait and Define GraphQLToken
**Outcome:** Trait-based abstraction for token sources with explicit token types

**Tasks:**
1. Create `/crates/libgraphql-parser/src/token/graphql_token.rs` with trivia-aware structure:
   ```rust
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
       pub span: GraphQLTokenSpan,
   }

   impl GraphQLToken {
       /// Convenience constructor for a token with no preceding trivia.
       pub fn new(kind: GraphQLTokenKind, span: GraphQLTokenSpan) -> Self {
           Self { kind, preceding_trivia: SmallVec::new(), span }
       }

       /// Convenience constructor for a token with preceding trivia.
       pub fn with_trivia(
           kind: GraphQLTokenKind,
           preceding_trivia: GraphQLTriviaTokenVec,
           span: GraphQLTokenSpan,
       ) -> Self {
           Self { kind, preceding_trivia, span }
       }
   }
   ```

2. Create `/crates/libgraphql-parser/src/token/graphql_token_kind.rs`:
   ```rust
   use crate::SmallVec;
   use crate::token::CookGraphQLStringError;
   use crate::token::GraphQLTokenSpan;
   use std::num::ParseFloatError;
   use std::num::ParseIntError;

   /// Type alias for error suggestions. Each suggestion is a message with an
   /// optional span indicating where the fix should be applied.
   /// Uses SmallVec since most errors have 0-2 suggestions.
   pub type GraphQLSuggestionVec = SmallVec<[(String, Option<GraphQLTokenSpan>); 2]>;

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
       /// TODO: Explore replacing suggestions with a richer `Diagnostic` structure
       /// that includes severity level and "fix action" for IDE integration.
       Error {
           message: String,
           suggestions: GraphQLSuggestionVec,
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
   ```

3. Create `/crates/libgraphql-parser/src/token/graphql_trivia_token.rs`:
   ```rust
   use crate::token::GraphQLTokenSpan;

   /// Trivia tokens that don't affect parsing but are preserved for tooling.
   #[derive(Clone, Debug, PartialEq)]
   pub enum GraphQLTriviaToken {
       Comment {
           value: String,
           span: GraphQLTokenSpan,
       },
       Comma {
           span: GraphQLTokenSpan,
       },
   }
   ```

4. Create `/crates/libgraphql-parser/src/token/cook_graphql_string_error.rs`:
   ```rust
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
   ```

5. Create `/crates/libgraphql-parser/src/token/graphql_token_span.rs`:
   ```rust
   use crate::SourcePosition;

   /// Represents the span of a token from start to end position.
   ///
   /// The span is a half-open interval: `[start_inclusive, end_exclusive)`.
   /// - `start_inclusive`: Position of the first character of the token
   /// - `end_exclusive`: Position immediately after the last character of the token
   ///
   /// Fields are public to allow third-party `GraphQLTokenSource` implementations
   /// to construct spans directly.
   #[derive(Clone, Debug, Eq, PartialEq)]
   pub struct GraphQLTokenSpan {
       pub start_inclusive: SourcePosition,
       pub end_exclusive: SourcePosition,
   }
   ```

6. Create `/crates/libgraphql-parser/src/token_source/graphql_token_source.rs` with trait definition:
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

7. Update all imports in macros crate

8. **Tests:**
   - Unit tests for GraphQLToken, GraphQLTokenKind, GraphQLTriviaToken
   - Unit tests for GraphQLTokenSpan
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Trivia (comments, commas) attached to following token - parser doesn't need to skip trivia
- Marker trait with `Iterator<Item = GraphQLToken>` - simpler than tuple, span is inside token
- Lexers accumulate trivia and attach to next token (not `GraphQLTokenStream`'s job)
- `GraphQLTokenKind::Eof` carries trailing trivia at end of document
- Explicit punctuator variants provide type safety and avoid string allocations
- `True`, `False`, `Null` are distinct token kinds for better type safety (not `Name` tokens)
- `GraphQLTokenSpan` has public fields so third-party token sources can construct spans directly
- `StrToGraphQLTokenSource` is a stub initially (Step 2.x implements it)
- Monomorphization is acceptable - common case uses only 1-2 token source types
- Negative numbers are single tokens (e.g., `IntValue("-123")`)
- TODO notes in code for future optimization experiments: `Cow<'a, str>`, `SmallVec` for trivia

---

### Step 1.3: Move and Generalize GraphQLTokenStream
**Outcome:** Token stream works with any `GraphQLTokenSource` implementation, centralizing all lookahead logic

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

### Step 1.4: Update RustMacroGraphQLTokenSource to Implement Trait
**Outcome:** `RustMacroGraphQLTokenSource` in macros crate implements `GraphQLTokenSource` trait

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
   use libgraphql_parser::token::GraphQLTokenSpan;
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

7. Convert `proc_macro2::Span` to `GraphQLTokenSpan`:
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

### Step 1.4.1: Position Accuracy Tests for RustMacroGraphQLTokenSource
**Outcome:** Verified that line/col_char/byte_offset values are accurate in various scenarios

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
   - Treat proc-macro byte_offset (from `RustMacroGraphQLTokenSource`) as "Rust-file byte offset" and Str byte_offset as "document byte offset" (different coordinate spaces).
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

### Step 1.5: Add Cargo Feature Flag Infrastructure
**Outcome:** Ability to toggle between old and new parser

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

### Step 1.6: Move GraphQLSchemaParser to libgraphql-parser
**Outcome:** Parser infrastructure in libgraphql-parser crate, ready for extension

**Tasks:**
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

## Phase 1 Completion Checklist

- [ ] All parser infrastructure in `libgraphql-parser` crate
- [ ] `GraphQLTokenSource` trait implemented and working
- [ ] `SourcePosition` with dual column tracking
- [ ] `RustMacroGraphQLTokenSource` implements trait
- [ ] Feature flag infrastructure in place
- [ ] `cargo clippy --tests` passes (no warnings)
- [ ] `cargo test` passes (all tests)
- [ ] `cargo test --features use-libgraphql-parser` passes
- [ ] All existing macro tests still work
