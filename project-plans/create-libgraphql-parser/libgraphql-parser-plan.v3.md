# Implementation Plan: Unified GraphQL Parser Infrastructure

## Overview
Move from dual parsing approach (`graphql_parser` + `GraphQLSchemaParser`) to unified token-based parser that supports schema documents, executable documents, and mixed documents. The new parser will be implemented in its own crate `libgraphql-parser`.

**Key Decisions:**
- Use trait-based `GraphQLTokenSource` (not enum) for extensibility - allows third parties to implement custom token sources
- Create new `SourcePosition` struct with `col`, `line`, `byte_offset` fields (no dependency on `libgraphql-core`)
- Keep `graphql_parser` AST types for now (future work: custom AST)
- Three parser methods: `parse_schema_document()`, `parse_executable_document()`, `parse_mixed_document()`
- Feature flag (`experimental-libgraphql-parser`) to toggle between old and new implementation (temporary; will become unconditional once stable)
- **New crate structure:** `libgraphql-parser` crate is standalone and usable independent of `libgraphql-core`
- Move `ast` module from `libgraphql-core` to `libgraphql-parser`
- `RustMacroGraphQLTokenSource` stays in `libgraphql-macros` (implements `GraphQLTokenSource` trait from parser crate)
- One struct per file, filename matches struct name in `lower_snake_case`
- `GraphQLToken` uses explicit punctuator variants (not `Punctuator(String)`) for type safety and avoiding allocations
- Comprehensive test suite with exhaustive tests including vendored tests from graphql-js and graphql-parser is mandatory. All tests should give clear error information useful for debugging if they fail.
- Comments are lexed as tokens (not skipped)
- All errors include helpful suggestions and "did-you-mean" hints when possible

**Critical Files:**
- `/crates/libgraphql-parser/` - New standalone parser crate
- `/crates/libgraphql-parser/src/graphql_token.rs` - Token enum with explicit variants
- `/crates/libgraphql-parser/src/graphql_token_source.rs` - Trait for token sources
- `/crates/libgraphql-parser/src/graphql_parser.rs` - Generic parser implementation
- `/crates/libgraphql-parser/src/str_to_graphql_token_source.rs` - String-based lexer
- `/crates/libgraphql-parser/src/source_position.rs` - Position tracking
- `/crates/libgraphql-parser/src/token_span.rs` - Token span with start/end positions
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

2. Update workspace `Cargo.toml`:
   ```toml
   [workspace]
   members = ["crates/libgraphql", "crates/libgraphql-core", "crates/libgraphql-macros", "crates/libgraphql-parser"]
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
   proc-macro2 = { workspace = true, features = ["span-locations"] }
   thiserror.workspace = true
   serde.workspace = true
   ```

4. Update `/crates/libgraphql-core/Cargo.toml`:
   ```toml
   [features]
   experimental-libgraphql-parser = ["dep:libgraphql-parser"]

   [dependencies]
   libgraphql-parser = { path = "../libgraphql-parser", optional = true }
   ```

5. Update `/crates/libgraphql/Cargo.toml`:
   ```toml
   [features]
   default = ["macros"]
   macros = ["dep:libgraphql-macros"]
   experimental-libgraphql-parser = ["libgraphql-core/experimental-libgraphql-parser"]
   ```

6. **Tests:**
   - `cargo build` succeeds
   - `cargo test` passes

**Considerations:**
- `libgraphql-parser` is designed to be useful independent of `libgraphql-core` - no dependency on core
- Feature flag controls inclusion in `libgraphql-core` during experimental phase; will become unconditional once stable and feature flag will be removed
- Clean separation of concerns enables third-party use of parser alone

---

### Step 1.1: Move ast Module and Create SourcePosition
**Outcome:** Location tracking ready for all token sources; ast module in parser crate

**Tasks:**
1. Move `/crates/libgraphql-core/src/ast.rs` → `/crates/libgraphql-parser/src/ast.rs`:
   - Keep all existing code unchanged
   - Update module exports in `/crates/libgraphql-parser/src/lib.rs`

2. Add conditional re-export in `/crates/libgraphql-core/src/lib.rs`:
   ```rust
   #[cfg(feature = "experimental-libgraphql-parser")]
   pub use libgraphql_parser::ast;

   #[cfg(not(feature = "experimental-libgraphql-parser"))]
   pub mod ast;  // Keep existing ast module when feature disabled
   ```

3. Create `/crates/libgraphql-parser/src/source_position.rs`:
   ```rust
   use crate::ast::AstPos;

   /// Source position information for parsing, including byte offset.
   /// This is standalone with no dependency on libgraphql-core.
   /// All fields are private with accessor methods.
   #[derive(Clone, Debug, Eq, PartialEq)]
   pub struct SourcePosition {
       line: usize,
       col: usize,
       byte_offset: usize,
   }

   impl SourcePosition {
       pub fn new(line: usize, col: usize, byte_offset: usize) -> Self { ... }

       pub fn line(&self) -> &usize { &self.line }
       pub fn col(&self) -> &usize { &self.col }
       pub fn byte_offset(&self) -> &usize { &self.byte_offset }

       pub fn advance_line(&mut self) { ... }
       pub fn advance_col(&mut self) { ... }
       pub fn advance_byte_offset(&mut self, bytes: usize) { ... }
       pub fn to_ast_pos(&self) -> AstPos { ... }

       /// Create from a proc_macro2::Span (requires span-locations feature)
       pub fn from_span(span: &proc_macro2::Span) -> Self {
           let start = span.start();
           Self::new(start.line, start.column, 0)  // byte_offset not available from Span
       }
   }
   ```

4. **Tests:**
   - Unit tests for SourcePosition methods
   - Test `from_span()` conversion
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- `FilePosition` remains unchanged in `libgraphql-core` (no changes needed there)
- `SourcePosition` is standalone in `libgraphql-parser` - no dependency on `libgraphql-core` (avoids cyclic dependency)
- No `ParserLocation` enum needed: `proc_macro2::Span` exposes line/col via `span.start()` when `span-locations` feature is enabled, so `RustMacroGraphQLTokenSource` can normalize all spans to `SourcePosition`
- `byte_offset` field enables efficient error reporting with source snippets (set to 0 when converted from Span since byte offset not available)

---

### Step 1.2: Create GraphQLTokenSource Trait and Define GraphQLToken
**Outcome:** Trait-based abstraction for token sources with explicit token types

**Tasks:**
1. Create `/crates/libgraphql-parser/src/graphql_token.rs` with explicit punctuator variants:
   ```rust
   /// A GraphQL lexical token with explicit variants for type safety.
   /// Using explicit punctuator variants avoids string allocations and enables
   /// better pattern matching.
   #[derive(Clone, Debug, PartialEq)]
   pub enum GraphQLToken {
       // Punctuators (no allocation needed)
       Bang,                 // !
       Dollar,               // $
       Ampersand,            // &
       ParenOpen,            // (
       ParenClose,           // )
       Ellipsis,             // ...
       Colon,                // :
       Equals,               // =
       At,                   // @
       SquareBracketOpen,    // [
       SquareBracketClose,   // ]
       CurlyBraceOpen,       // {
       CurlyBraceClose,      // }
       Pipe,                 // |
       Comma,                // , (emitted for tooling like linters/formatters)

       // Literals
       Name(String),
       IntValue(i64),
       FloatValue(f64),
       StringValue(String),

       // Boolean and null (distinct from Name for type safety)
       True,
       False,
       Null,

       // Comments (preserved for tooling)
       Comment(String),
   }

   impl GraphQLToken {
       pub fn is_punctuator(&self) -> bool { ... }
       pub fn as_punctuator_str(&self) -> Option<&'static str> { ... }
       pub fn is_value(&self) -> bool { ... }
   }
   ```

2. Create `/crates/libgraphql-parser/src/token_span.rs`:
   ```rust
   use crate::source_position::SourcePosition;

   /// Represents the span of a token from start to end position.
   #[derive(Clone, Debug, Eq, PartialEq)]
   pub struct TokenSpan {
       start: SourcePosition,
       end: SourcePosition,
   }

   impl TokenSpan {
       pub fn new(start: SourcePosition, end: SourcePosition) -> Self { ... }
       pub fn start(&self) -> &SourcePosition { &self.start }
       pub fn end(&self) -> &SourcePosition { &self.end }
   }
   ```

3. Create `/crates/libgraphql-parser/src/graphql_token_source.rs` with trait definition:
   ```rust
   use crate::graphql_token::GraphQLToken;
   use crate::token_span::TokenSpan;

   /// Trait for sources that produce GraphQL tokens.
   ///
   /// This trait enables extensibility - third parties can implement custom
   /// token sources (e.g., from different input formats or with custom
   /// preprocessing).
   ///
   /// # Implementing
   ///
   /// Implementors should produce a stream of tokens with their associated
   /// spans. Whitespace is skipped internally (an "ignored token" per the
   /// GraphQL spec), but commas are emitted as `GraphQLToken::Comma` to
   /// support tooling use cases like linters and formatters.
   pub trait GraphQLTokenSource: Iterator<Item = (GraphQLToken, TokenSpan)> {
       /// Returns true if there are no more tokens to consume
       fn is_at_end(&mut self) -> bool;

       /// Peek at the next token without consuming it
       fn peek(&mut self) -> Option<&(GraphQLToken, TokenSpan)>;

       /// Peek at the nth token ahead (0-indexed)
       fn peek_nth(&mut self, n: usize) -> Option<&(GraphQLToken, TokenSpan)>;
   }
   ```

4. Update all imports in macros crate

5. **Tests:**
   - Unit tests for GraphQLToken helper methods
   - Unit tests for TokenSpan
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Trait-based approach enables extensibility for third parties
- Explicit punctuator variants provide type safety and avoid string allocations
- `True`, `False`, `Null` are distinct tokens (not `Name`) for better type safety
- `Comma` is emitted as a token (not skipped) - enables tooling like linters/formatters
- Comments are lexed as tokens, not skipped - enables tooling like formatters
- `TokenSpan` provides start and end positions for each token
- `StrToGraphQLTokenSource` is a stub initially (Step 2.x implements it)
- Monomorphization is acceptable - common case uses only 1-2 token source types

---

### Step 1.3: Move and Generalize GraphQLTokenStream
**Outcome:** Token stream works with any `GraphQLTokenSource` implementation

**Tasks:**
1. Move `/crates/libgraphql-macros/src/graphql_token_stream.rs` → `/crates/libgraphql-parser/src/graphql_token_stream.rs`

2. Update `GraphQLTokenStream` to be generic over `GraphQLTokenSource`:
   ```rust
   use crate::graphql_token::GraphQLToken;
   use crate::graphql_token_source::GraphQLTokenSource;
   use crate::token_span::TokenSpan;
   use std::collections::VecDeque;

   /// Streaming token parser with bounded lookahead buffer.
   /// Generic over any token source implementing `GraphQLTokenSource`.
   pub struct GraphQLTokenStream<S: GraphQLTokenSource> {
       token_source: S,
       buffer: VecDeque<(GraphQLToken, TokenSpan)>,
       current_span: Option<TokenSpan>,
   }

   impl<S: GraphQLTokenSource> GraphQLTokenStream<S> {
       pub fn new(token_source: S) -> Self { ... }
       pub fn peek(&mut self) -> Option<&(GraphQLToken, TokenSpan)> { ... }
       pub fn peek_nth(&mut self, n: usize) -> Option<&(GraphQLToken, TokenSpan)> { ... }
       pub fn next(&mut self) -> Option<(GraphQLToken, TokenSpan)> { ... }
       pub fn current_span(&self) -> &TokenSpan { ... }
       pub fn is_at_end(&mut self) -> bool { ... }
       pub fn check_name(&mut self, name: &str) -> bool { ... }
       pub fn check_punctuator(&mut self, token: &GraphQLToken) -> bool { ... }
   }
   ```

3. Update existing methods for compatibility:
   - Keep existing API surface
   - Update `check_punctuator` to take `&GraphQLToken` instead of `&str`

4. **Tests:**
   - Port existing token stream tests
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Generic `S: GraphQLTokenSource` enables any token source implementation
- Existing code in `libgraphql-macros` continues to work after updating to implement trait
- Token stream is now source-agnostic
- File renamed to `graphql_token_stream.rs` to match struct name

---

### Step 1.4: Update RustMacroGraphQLTokenSource to Implement Trait
**Outcome:** `RustMacroGraphQLTokenSource` in macros crate implements `GraphQLTokenSource` trait

**Tasks:**
1. Rename `/crates/libgraphql-macros/src/rust_to_graphql_token_adapter.rs` → `rust_macro_graphql_token_source.rs`

2. Rename `RustToGraphQLTokenAdapter` → `RustMacroGraphQLTokenSource`

3. Add `libgraphql-parser` as dependency to `/crates/libgraphql-macros/Cargo.toml`:
   ```toml
   [dependencies]
   libgraphql-parser = { path = "../libgraphql-parser" }
   ```

4. Update `RustMacroGraphQLTokenSource` to implement `GraphQLTokenSource` trait:
   ```rust
   use libgraphql_parser::graphql_token::GraphQLToken;
   use libgraphql_parser::graphql_token_source::GraphQLTokenSource;
   use libgraphql_parser::source_position::SourcePosition;
   use libgraphql_parser::token_span::TokenSpan;

   impl Iterator for RustMacroGraphQLTokenSource {
       type Item = (GraphQLToken, TokenSpan);
       fn next(&mut self) -> Option<Self::Item> { ... }
   }

   impl GraphQLTokenSource for RustMacroGraphQLTokenSource {
       fn is_at_end(&mut self) -> bool { ... }
       fn peek(&mut self) -> Option<&(GraphQLToken, TokenSpan)> { ... }
       fn peek_nth(&mut self, n: usize) -> Option<&(GraphQLToken, TokenSpan)> { ... }
   }
   ```

5. Convert `proc_macro2::Span` to `TokenSpan` using `SourcePosition::from_span()`:
   - Use the span's start position for `TokenSpan::start`
   - Use the span's end position for `TokenSpan::end`

6. Update to emit new explicit `GraphQLToken` variants (e.g., `GraphQLToken::CurlyBraceOpen` instead of `GraphQLToken::Punctuator("{".to_string())`)

7. **Do not re-export parser abstractions** from `libgraphql-macros` (breaking change is acceptable)
   - Remove any existing public parser exports from `libgraphql-macros`

8. **Tests:**
   - Port existing token source tests
   - Verify all proc macro tests still pass
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- `RustMacroGraphQLTokenSource` stays in `libgraphql-macros` (where it belongs - it's proc-macro specific)
- It implements the `GraphQLTokenSource` trait from `libgraphql-parser`
- `libgraphql-macros` now depends on `libgraphql-parser` (not the other way around)
- Third parties can implement their own token sources following the same pattern
- Breaking change to remove parser exports from macros crate is intentional
- `SourcePosition::from_span()` normalizes `proc_macro2::Span` to the unified position type

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
     experimental-libgraphql-parser = ["libgraphql-core/experimental-libgraphql-parser"]
     ```
   - `/crates/libgraphql-core/Cargo.toml`:
     ```toml
     [features]
     experimental-libgraphql-parser = ["dep:libgraphql-parser"]
     ```

2. Add conditional compilation in future parser call sites (template for later steps):
   ```rust
   #[cfg(feature = "experimental-libgraphql-parser")]
   use libgraphql_parser::GraphQLParser;

   #[cfg(not(feature = "experimental-libgraphql-parser"))]
   use graphql_parser;
   ```

3. Update CI to test both feature configurations (see Step 4.6)

4. **Documentation:**
   - Document feature flag in main README
   - Explain when to use which parser
   - Note that experimental-libgraphql-parser is opt-in

5. **Tests:**
   - `cargo build` succeeds with and without feature
   - `cargo test` passes with and without feature
   - `cargo clippy --tests` passes both ways

**Considerations:**
- Default stays with `graphql_parser` until new parser is battle-tested
- Allows gradual migration and confidence building
- Users can opt into new parser early for testing
- Feature name "experimental-libgraphql-parser" signals it's not production-ready yet

---

### Step 1.6: Move GraphQLSchemaParser to libgraphql-parser
**Outcome:** Parser infrastructure in libgraphql-parser crate, ready for extension

**Tasks:**
1. Create `/crates/libgraphql-parser/src/` module structure:
   ```
   libgraphql-parser/src/
   ├── lib.rs
   ├── ast.rs                        (from 1.1)
   ├── source_position.rs            (from 1.1)
   ├── token_span.rs                 (from 1.2)
   ├── graphql_token.rs              (from 1.2)
   ├── graphql_token_source.rs       (from 1.2)
   ├── graphql_token_stream.rs       (from 1.3)
   ├── str_to_graphql_token_source.rs (stub, implemented in Phase 2)
   ├── graphql_parser.rs             (moved from macros)
   ├── graphql_parse_error.rs        (moved from macros)
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
   - Add stub methods for `parse_executable_document()` and `parse_mixed_document()` (implemented in Phase 3)

4. Update `libgraphql-macros` imports:
   - Import `GraphQLParser` from `libgraphql_parser`
   - **Do not re-export parser abstractions** from `libgraphql-macros`
   - Remove any existing public parser exports

5. Update `/crates/libgraphql/src/lib.rs`:
   ```rust
   #[cfg(feature = "experimental-libgraphql-parser")]
   pub use libgraphql_parser as parser;

   #[cfg(feature = "experimental-libgraphql-parser")]
   pub use libgraphql_parser::ast;

   #[cfg(not(feature = "experimental-libgraphql-parser"))]
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
**Outcome:** Skeleton lexer that tokenizes simple cases

**Tasks:**
1. Create `/crates/libgraphql-parser/src/str_to_graphql_token_source.rs`:
   ```rust
   use crate::graphql_token::GraphQLToken;
   use crate::graphql_token_source::GraphQLTokenSource;
   use crate::source_position::SourcePosition;
   use crate::token_span::TokenSpan;

   pub struct StrToGraphQLTokenSource<'a> {
       source: &'a str,
       position: SourcePosition,
       // ... lexer state
   }

   impl<'a> Iterator for StrToGraphQLTokenSource<'a> {
       type Item = (GraphQLToken, TokenSpan);
       fn next(&mut self) -> Option<Self::Item> { ... }
   }

   impl<'a> GraphQLTokenSource for StrToGraphQLTokenSource<'a> {
       fn is_at_end(&mut self) -> bool { ... }
       fn peek(&mut self) -> Option<&(GraphQLToken, TokenSpan)> { ... }
       fn peek_nth(&mut self, n: usize) -> Option<&(GraphQLToken, TokenSpan)> { ... }
   }
   ```

2. Implement basic tokenization:
   - Whitespace skipping (space, tab, newline per spec; comma is emitted as `Comma` token)
   - Single-character punctuation as explicit variants: `Bang`, `Dollar`, `Ampersand`, `ParenOpen`, `ParenClose`, `Colon`, `Equals`, `At`, `SquareBracketOpen`, `SquareBracketClose`, `CurlyBraceOpen`, `CurlyBraceClose`, `Pipe`, `Comma`
   - Simple names (identifiers): `/[_A-Za-z][_0-9A-Za-z]*/`
   - Integer literals (basic cases, no validation yet)

3. Implement `Iterator<Item = (GraphQLToken, TokenSpan)>`:
   - Track start position before tokenizing, end position after
   - Return `TokenSpan::new(start, end)` for each token
   - Update position tracking as tokens are consumed

4. **Error reporting with structured suggestions:**
   - Errors should include a `suggestions: Vec<String>` field for "did you mean?" hints
   - Suggestions target GraphQL authors (e.g., "Did you mean `String`?" when encountering `Stirng`)
   - Examples of suggestions:
     - Typos in type names
     - Missing punctuation (e.g., "Expected `:` after field name")
     - Common syntax mistakes

5. **Tests:**
   - Simple schema: `type Query { hello: String }`
   - Verify correct tokenization and positions
   - Tests should provide clear debugging info on failure
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- This is a foundation; edge cases come in later steps
- Focus on correct position tracking from the start
- Error handling includes helpful suggestions
- Renamed from StringToGraphQLTokenAdapter → StrToGraphQLTokenSource (operates on &str, not String)

---

### Step 2.2: Comments and Multi-Character Punctuation
**Outcome:** Handle GraphQL comments and spread operator

**Tasks:**
1. Add comment token handling:
   - Recognize `#` followed by anything until newline
   - **Emit `GraphQLToken::Comment(String)` tokens** to parser (don't skip)
   - Parser can choose to ignore or preserve comments
   - Update position tracking for comment content

2. Add multi-character punctuation:
   - Recognize `...` (spread operator)
   - Disambiguate from three separate `.` tokens

3. **Tests:**
   - Schema with comments: `# This is a comment\ntype Query { ... }`
   - Spread in fragments
   - Comments at EOF
   - Verify comment tokens are emitted
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Comments are lexed as tokens (not skipped like in graphql_parser)
- Parser decides whether to preserve comments for tooling (formatters, IDEs)
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
     - Unicode escapes: `\uXXXX` (4 hex digits)
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
   - Unicode: `"\u0041\u0042\u0043"` → `"ABC"`
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

**Tasks:**
1. Implement integer parsing:
   - Valid: `0`, `123`, `-456`
   - Invalid: `00`, `01`, leading zeros except `0` itself
   - Handle negative sign
   - Range validation: i64::MIN to i64::MAX

2. Implement float parsing:
   - Recognize patterns:
     - Decimal: `1.23`, `0.5`
     - Exponent: `1e10`, `1E10`, `1e+10`, `1e-10`
     - Both: `1.23e10`
   - Invalid: `1.` (must have digits after decimal)
   - Parse to f64

3. Disambiguate integer vs float:
   - Presence of `.` or `e`/`E` → float
   - Otherwise → integer

4. **Tests:**
   - Valid integers: `0`, `123`, `-456`
   - Invalid integers: `00`, `01`
   - Valid floats: `1.0`, `1e10`, `1.23e-5`
   - Invalid floats: `1.`, `.5`, `1e`
   - Boundary cases: i64::MAX, f64::MAX

**Considerations:**
- Use Rust's `parse::<i64>()` and `parse::<f64>()` after validation
- Careful with sign handling (already consumed `-` token?)
- Error messages must be clear

---

### Step 2.5: Name Validation and Keywords
**Outcome:** Proper name lexing with spec validation

**Tasks:**
1. Implement name lexing:
   - Must match `/[_A-Za-z][_0-9A-Za-z]*/`
   - Reject names starting with digits
   - Emit `GraphQLToken::Name(String)` for most names

2. **Consider distinct tokens for boolean/null literals:**
   - Current approach: `true`, `false`, `null` as `Name` tokens, parser interprets
   - Alternative: Distinct tokens `GraphQLToken::True`, `GraphQLToken::False`, `GraphQLToken::Null`
   - **Benefits of distinct tokens:**
     - Type safety: Can't accidentally use `null` as identifier
     - Clearer parsing logic
     - Better error messages ("expected boolean, got identifier")
   - **Downsides:**
     - Slightly more complex lexer
     - Need to handle contexts where these are valid identifiers (rare in GraphQL)
   - **Question for consideration:** Are there contexts where `true`/`false`/`null` can be field/type names?
     - No, these are always literals in GraphQL spec
   - **Recommendation:** Use distinct tokens for `true`, `false`, `null`

3. **Tests:**
   - Valid names: `hello`, `_private`, `type2`
   - Invalid names: `2type`, `hello-world`
   - Boolean literals: `true`, `false`
   - Null literal: `null`
   - Keywords as names: `type`, `query`, `mutation` (context-dependent, always Name tokens)
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Most GraphQL keywords are context-dependent
- `true`, `false`, `null` are NOT context-dependent; always literals
- Using distinct tokens for boolean/null improves type safety
- Name validation is straightforward regex
- Parser handles context, lexer emits appropriate token types

---

### Step 2.6: Error Recovery in Lexer
**Outcome:** Lexer recovers from errors and continues tokenizing

**Tasks:**
1. Add error token type:
   - `GraphQLToken::Error(String)` variant for invalid tokens
   - Emit error token instead of returning `None`

2. Implement recovery strategies:
   - Invalid character: Skip and continue
   - Unterminated string: Emit error, skip to newline or EOF
   - Invalid number: Emit error, skip to next whitespace

3. Update parser to handle error tokens:
   - Accumulate errors instead of failing immediately
   - Continue parsing to find multiple errors

4. **Tests:**
   - Multiple errors in single document
   - Error recovery doesn't skip valid tokens
   - Error positions are accurate

**Considerations:**
- Matches `RustMacroGraphQLTokenSource` behavior for consistency
- User experience: show all errors, not just first one
- Balance recovery quality with complexity

---

### Step 2.7: Comprehensive Lexer Testing
**Outcome:** Lexer is battle-tested and spec-compliant

**Tasks:**
1. Port test cases from graphql-js:
   - Clone https://github.com/graphql/graphql-js
   - Extract lexer tests from `src/__tests__/lexer-test.ts`
   - Convert to Rust test cases in `/crates/libgraphql-parser/src/tests/str_to_graphql_token_source_tests.rs`
   - **Vendor the tests** (include them in the repo) for reproducibility
   - Ensure 100% of graphql-js lexer tests pass
   - All tests must provide **clear error information useful for debugging** if they fail

2. Port test cases from graphql-parser:
   - Clone https://github.com/graphql-rust/graphql-parser
   - Extract relevant lexer/parser tests
   - **Vendor the tests** in the repo
   - Convert to Rust test cases
   - Ensure compatibility
   - Clear debugging output on test failures

3. Add fuzzing tests:
   - Use `cargo-fuzz` or `proptest`
   - Generate random GraphQL-like strings
   - Verify lexer doesn't panic
   - Document any interesting fuzz-discovered issues

4. Benchmark lexer performance:
   - Compare against `graphql_parser` on various inputs
   - Identify performance bottlenecks
   - Optimize hot paths (string scanning, allocation)
   - **Check benchmark fixture files into source control** (e.g., GitHub/GitLab schema files in `/crates/libgraphql-parser/benches/fixtures/`)

5. **Success criteria:**
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
1. Add operation parsing to `/crates/libgraphql-core/src/parser/parser.rs`:
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
                           ParseErrorSuggestions::new(vec![
                               "Remove type definitions from this document".to_string(),
                               "Move type definitions to a separate schema file".to_string(),
                           ]),
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
   pub struct MixedDocument {
       schema_definitions: Vec<ast::schema::Definition>,
       executable_definitions: Vec<ast::operation::Definition>,
   }

   impl MixedDocument {
       pub fn schema_definitions(&self) -> &Vec<ast::schema::Definition> {
           &self.schema_definitions
       }

       pub fn executable_definitions(&self) -> &Vec<ast::operation::Definition> {
           &self.executable_definitions
       }
   }
   ```

2. Add method to `GraphQLParser`:
   ```rust
   pub fn parse_mixed_document(self) -> ParseResult<MixedDocument> {
       let mut schema_defs = Vec::new();
       let mut exec_defs = Vec::new();
       let mut errors = Vec::new();

       while !self.tokens.is_at_end() {
           let (token, _) = self.tokens.peek();
           match token {
               GraphQLToken::Name(kw) if is_schema_keyword(kw) => {
                   match self.parse_schema_definition() {
                       Ok(def) => schema_defs.push(def),
                       Err(err) => {
                           errors.push(err);
                           self.recover_to_next_definition();
                       }
                   }
               }
               GraphQLToken::Name(kw) if is_exec_keyword(kw) => {
                   match self.parse_executable_definition() {
                       Ok(def) => exec_defs.push(def),
                       Err(err) => {
                           errors.push(err);
                           self.recover_to_next_definition();
                       }
                   }
               }
               GraphQLToken::CurlyBraceOpen => {
                   // Anonymous query (shorthand)
                   match self.parse_executable_definition() {
                       Ok(def) => exec_defs.push(def),
                       Err(err) => {
                           errors.push(err);
                           self.recover_to_next_definition();
                       }
                   }
               }
               _ => {
                   errors.push(ParseError::new(
                       "Unexpected token in document",
                       /* location */,
                       ParseErrorSuggestions::new(vec![
                           "Expected a type definition, operation, or fragment".to_string(),
                       ]),
                   ));
                   self.recover_to_next_definition();
               }
           }
       }

       ParseResult {
           ast: if schema_defs.is_empty() && exec_defs.is_empty() {
               None
           } else {
               Some(MixedDocument { schema_definitions: schema_defs, executable_definitions: exec_defs })
           },
           errors,
       }
   }
   ```

3. Implement keyword classification:
   - Schema keywords: `type`, `interface`, `union`, `enum`, `scalar`, `input`, `directive`, `schema`, `extend`
   - Executable keywords: `query`, `mutation`, `subscription`, `fragment`

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
- Errors include helpful suggestions

---

### Step 3.5: Comprehensive Parser Testing
**Outcome:** Parser is battle-tested for all document types

**Tasks:**
1. Port operation/fragment tests from graphql-js:
   - Extract parser tests from graphql-js test suite
   - Convert to Rust tests
   - Ensure 100% coverage of operation syntax

2. Port tests from graphql-parser:
   - Ensure compatibility with existing test expectations

3. Add edge case tests:
   - Deeply nested selections
   - Large documents (performance)
   - Documents with many errors (error recovery)
   - Mixed documents with various combinations

4. Add regression tests for discovered bugs

5. **Success criteria:**
   - 100% graphql-js parser tests pass (for covered syntax)
   - 100% graphql-parser tests pass
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
     #[cfg(feature = "experimental-libgraphql-parser")]
     pub fn build_from_str(&mut self, source: &str) -> Result<&mut Self, SchemaBuildError> {
         use libgraphql_parser::graphql_parser::GraphQLParser;
         use libgraphql_parser::source_position::SourcePosition;
         use libgraphql_parser::str_to_graphql_token_source::StrToGraphQLTokenSource;

         let position = SourcePosition::new(1, 1, 0);
         let token_source = StrToGraphQLTokenSource::new(source, position);
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

     #[cfg(not(feature = "experimental-libgraphql-parser"))]
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
   - `experimental-libgraphql-parser` feature selects new parser
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
   #[cfg(feature = "experimental-libgraphql-parser")]
   pub fn build_from_str(&mut self, source: &str) -> Result<...> {
       use libgraphql_parser::graphql_parser::GraphQLParser;
       use libgraphql_parser::source_position::SourcePosition;
       use libgraphql_parser::str_to_graphql_token_source::StrToGraphQLTokenSource;

       let position = SourcePosition::new(1, 1, 0);
       let token_source = StrToGraphQLTokenSource::new(source, position);
       let parser = GraphQLParser::new(token_source);
       let result = parser.parse_executable_document();

       // Handle ParseResult (errors + salvaged AST)
       // ...
   }

   #[cfg(not(feature = "experimental-libgraphql-parser"))]
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
   /// This functionality requires the `experimental-libgraphql-parser` feature flag.
   pub struct MixedDocumentBuilder {
       schema_builder: SchemaBuilder,
       executable_builder: ExecutableDocumentBuilder,
   }

   impl MixedDocumentBuilder {
       pub fn build_from_str(&mut self, source: &str) -> Result<...> {
           #[cfg(feature = "experimental-libgraphql-parser")]
           {
               use libgraphql_parser::graphql_parser::GraphQLParser;
               use libgraphql_parser::source_position::SourcePosition;
               use libgraphql_parser::str_to_graphql_token_source::StrToGraphQLTokenSource;

               let position = SourcePosition::new(1, 1, 0);
               let token_source = StrToGraphQLTokenSource::new(source, position);
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

           #[cfg(not(feature = "experimental-libgraphql-parser"))]
           compile_error!("Mixed documents require experimental-libgraphql-parser feature");
       }
   }
   ```

2. Expose in public API:
   - Add to `/crates/libgraphql/src/lib.rs`
   - Only available with `experimental-libgraphql-parser` feature
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
- Only available with new parser
- Should encourage adoption of experimental-libgraphql-parser feature
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

3. **Add performance regression test**:
   - Create test that fails if libgraphql-parser exceeds threshold vs graphql_parser
   - Threshold: 2x slower on key fixtures (GitHub/GitLab schemas)
   - Example:
     ```rust
     #[test]
     fn performance_regression_github_schema() {
         let schema_src = include_str!("fixtures/github_schema.graphql");

         let graphql_parser_time = benchmark_graphql_parser(schema_src);
         let libgraphql_parser_time = benchmark_libgraphql_parser(schema_src);

         let ratio = libgraphql_parser_time / graphql_parser_time;
         assert!(ratio < 2.0, "Performance regression: {}x slower than graphql_parser", ratio);
     }
     ```
   - Run in CI to catch regressions

4. Optimize hot paths:
   - String allocation
   - Token buffer management
   - AST node construction

5. Document results:
   - Add benchmark results to README
   - Note acceptable performance threshold (within 2x)

**Considerations:**
- Performance regression is acceptable if < 2x slower
- Functionality and correctness > raw speed
- Optimization is iterative; don't over-optimize initially
- Automated regression tests prevent performance degradation over time
- Real-world schemas (GitHub/GitLab) provide realistic benchmarks

---

### Step 4.5: Documentation and Examples
**Outcome:** Users understand how to use new parser

**Tasks:**
1. Update `/README.md`:
   - Document `unified-parser` feature flag
   - Explain benefits of new parser
   - Migration guide from `graphql_parser`

2. Update rustdoc:
   - Add module-level docs to `/crates/libgraphql-core/src/parser/mod.rs`
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
- Should encourage migration to unified-parser

---

### Step 4.6: CI/CD Updates
**Outcome:** Both parsers tested in CI, confidence in new parser

**Tasks:**
1. Update GitHub Actions workflow:
   - Current CI already runs `cargo test` (verify this)
   - Add job with `--features experimental-libgraphql-parser`
   - Add job without feature (default) - likely already exists
   - Matrix build: test both configurations

2. **Verify existing coverage** (likely already in place):
   - `cargo test` should cover most needs
   - Check if benchmark runs exist
   - Check if fuzzing runs exist

3. Add parser-specific jobs **if not already covered by cargo test**:
   - Lexer fuzzing (may need separate job)
   - Parser fuzzing (may need separate job)
   - Performance regression test (from Step 4.4)

4. Code coverage (if not already in place):
   - Ensure parser code has high coverage
   - Use `cargo-tarpaulin` or `cargo-llvm-cov`

5. **Success criteria:**
   - All tests pass with both feature flags
   - No performance regression > 2x
   - Code coverage > 90% for parser module

**Considerations:**
- CI validates both code paths
- Prevents regressions in either parser
- Builds confidence for eventual default switch
- Most needs likely covered by existing `cargo test` infrastructure
- Main addition: testing with feature flag enabled

---

### Step 4.7: Default Feature Flag Flip
**Outcome:** experimental-libgraphql-parser becomes the default

**Tasks:**
1. Update `/crates/libgraphql/Cargo.toml`:
   ```toml
   [features]
   default = ["macros", "experimental-libgraphql-parser"]
   macros = ["dep:libgraphql-macros"]
   experimental-libgraphql-parser = ["libgraphql-core/experimental-libgraphql-parser"]
   legacy-parser = []  # Opt-in to graphql_parser (disables experimental-libgraphql-parser)
   ```

2. Invert conditional compilation:
   - `#[cfg(not(feature = "legacy-parser"))]` → new parser
   - `#[cfg(feature = "legacy-parser")]` → old parser

3. Update documentation:
   - Note that `experimental-libgraphql-parser` is now default
   - `legacy-parser` available for backward compatibility
   - Plan to deprecate `legacy-parser` in future release

4. **Deprecation plan:**
   - Announce in release notes
   - `legacy-parser` supported for 2-3 releases
   - Eventually remove `graphql_parser` dependency entirely

**Considerations:**
- Only flip default after confidence is high (weeks/months of testing)
- Provide escape hatch for users who hit issues
- Clear communication in changelog

---

## Success Criteria

**Phase 1 Complete:**
- ✅ All parser infrastructure moved to `libgraphql-parser` crate
- ✅ `GraphQLTokenSource` trait implemented
- ✅ Feature flag infrastructure in place
- ✅ All existing tests pass

**Phase 2 Complete:**
- ✅ `StrToGraphQLTokenSource` fully implements GraphQL lexer spec
- ✅ 100% graphql-js lexer tests pass
- ✅ 100% graphql-parser lexer tests pass
- ✅ Performance within 2x of `graphql_parser`

**Phase 3 Complete:**
- ✅ Parser handles schema, executable, and mixed documents
- ✅ 100% graphql-js parser tests pass (for covered syntax)
- ✅ 100% graphql-parser parser tests pass

**Phase 4 Complete:**
- ✅ New parser integrated in all builders
- ✅ Mixed document support available
- ✅ Comprehensive benchmarks documented
- ✅ Documentation and examples complete
- ✅ CI validates both parsers
- ✅ (Eventually) `experimental-libgraphql-parser` is the default

---

## Timeline Estimate

**Phase 1:** 3-5 days (infrastructure, refactoring)
**Phase 2:** 7-10 days (lexer implementation, testing)
**Phase 3:** 5-7 days (parser extension, testing)
**Phase 4:** 5-7 days (integration, benchmarking, docs)

**Total:** ~3-4 weeks of focused development

**Note:** Testing and validation will be ongoing throughout. Don't rush; correctness is paramount.

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
- Parser error recovery improvements (better error messages)
- Incremental parsing for IDE support
- WASM compilation for browser use
- Streaming parser for very large documents
