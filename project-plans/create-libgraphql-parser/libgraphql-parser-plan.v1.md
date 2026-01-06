# Implementation Plan: Unified GraphQL Parser Infrastructure

## Overview
Move from dual parsing approach (`graphql_parser` + `GraphQLSchemaParser`) to unified token-based parser that supports schema documents, executable documents, and mixed documents. The new parser will be implemented in its own crate `libgraphql-parser`.

**Key Decisions:**
- Use enum-based `TokenSource` (not generics) to avoid monomorphization
- Create new location structure wrapping `FilePosition` with additional fields like `offset`
- Keep `graphql_parser` AST types for now (future work: custom AST)
- Three parser methods: `parse_schema_document()`, `parse_executable_document()`, `parse_mixed_document()`
- Feature flag (`experimental-libgraphql-parser`) to toggle between old and new implementation
- **New crate structure:** `libgraphql-parser` crate, with `libgraphql-core` depending on it (feature-gated)
- Move `ast` module from `libgraphql-core` to `libgraphql-parser`
- Comprehensive test suite with exhaustive tests including vendored tests from graphql-js and graphql-parser is mandatory. All tests should give clear error information useful for debugging if they fail.
- Comments are lexed as tokens (not skipped)
- All errors include helpful suggestions and "did-you-mean" hints when possible

**Critical Files:**
- `/crates/libgraphql-core/src/loc.rs` - Location tracking (FilePosition)
- `/crates/libgraphql-macros/src/graphql_schema_parser.rs` - Parser to move/extend
- `/crates/libgraphql-macros/src/graphql_token_stream.rs` - Token stream to generalize
- `/crates/libgraphql-macros/src/rust_to_graphql_token_adapter.rs` - Token source
- `/crates/libgraphql-parser/` - New crate to create

---

## Phase 1: Foundation & Infrastructure
*Goal: Prepare architecture for unified parsing without breaking existing functionality*

**Note:** Each step must pass `cargo clippy --tests` and `cargo test` with no warnings or errors before proceeding.

### Step 1.0: Create libgraphql-parser Crate
**Outcome:** New crate structure ready for parser implementation

**Tasks:**
1. Create new crate at `/crates/libgraphql-parser/`:
   ```bash
   cargo new --lib crates/libgraphql-parser
   ```

2. Update workspace `Cargo.toml`:
   ```toml
   [workspace]
   members = ["crates/libgraphql", "crates/libgraphql-core", "crates/libgraphql-macros", "crates/libgraphql-parser"]
   ```

3. Add dependencies to `/crates/libgraphql-parser/Cargo.toml`:
   ```toml
   [dependencies]
   graphql-parser = "0.4.0"
   proc-macro2 = "1.0"
   thiserror = "2.0"
   serde = { version = "1.0", features = ["derive"] }
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
- New crate isolates parser from core library
- Feature flag controls inclusion
- Clean separation of concerns

---

### Step 1.1: Move ast Module and Create Location Abstraction
**Outcome:** Location tracking ready for both proc-macro and string-based parsing; ast module in parser crate

**Tasks:**
1. Move `/crates/libgraphql-core/src/ast.rs` → `/crates/libgraphql-parser/src/ast.rs`:
   - Keep all existing code unchanged
   - Update module exports in `/crates/libgraphql-parser/src/lib.rs`

2. Add re-export in `/crates/libgraphql-core/src/lib.rs`:
   ```rust
   pub use libgraphql_parser::ast;
   ```

3. Create new location structure in `/crates/libgraphql-parser/src/location.rs`:
   ```rust
   use crate::ast::AstPos;
   use libgraphql_core::loc::FilePosition;
   use proc_macro2::Span;

   /// Extended location information for parsing, including byte offset
   pub struct ParserPosition {
       pub file_position: FilePosition,
       pub offset: usize,
   }

   impl ParserPosition {
       pub fn new(file_position: FilePosition, offset: usize) -> Self { ... }
       pub fn advance_line(&mut self) { ... }
       pub fn advance_col(&mut self) { ... }
       pub fn advance_offset(&mut self, bytes: usize) { ... }
       pub fn to_ast_pos(&self) -> AstPos { ... }
   }

   /// Location information from different token sources
   pub enum ParserLocation {
       ProcMacro(Span),
       Source(ParserPosition),
   }

   impl ParserLocation {
       pub fn to_ast_pos(&self) -> AstPos { ... }
       pub fn to_file_position(&self) -> Option<FilePosition> { ... }
   }
   ```

4. Make parser module conditionally public in `/crates/libgraphql-parser/src/lib.rs`:
   ```rust
   #[cfg(feature = "experimental-libgraphql-parser")]
   pub mod location;

   #[cfg(not(feature = "experimental-libgraphql-parser"))]
   mod location;
   ```

5. **Tests:**
   - Unit tests for ParserPosition methods
   - Conversion tests between ParserLocation variants
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- `FilePosition` remains unchanged in `libgraphql-core`
- New `ParserPosition` wraps `FilePosition` + additional fields
- Can merge later if `FilePosition` becomes unused
- `proc_macro2::Span` doesn't expose line/col in stable Rust
- `offset` field enables efficient error reporting with source snippets
- Parser module only public with feature flag

---

### Step 1.2: Create TokenSource Enum and Refine GraphQLToken
**Outcome:** Abstraction layer for different token sources with explicit token types

**Tasks:**
1. Move `GraphQLToken` from `/crates/libgraphql-macros/src/rust_to_graphql_token_adapter.rs` to `/crates/libgraphql-parser/src/token.rs`:
   - Keep existing 5 variants initially: `Punctuator`, `Name`, `IntValue`, `FloatValue`, `StringValue`
   - Update all imports in macros crate

2. **Consider refining GraphQLToken with explicit punctuator variants** (may be follow-up task after initial move):
   - Current: `GraphQLToken::Punctuator(String)` can hold any punctuator
   - Proposed: Explicit variants like:
     ```rust
     pub enum GraphQLToken {
         // Punctuators
         Bang,              // !
         Dollar,            // $
         Ampersand,         // &
         ParenOpen,         // (
         ParenClose,        // )
         Ellipsis,          // ...
         Colon,             // :
         Equals,            // =
         At,                // @
         BracketOpen,       // [
         BracketClose,      // ]
         BraceOpen,         // {
         BraceClose,        // }
         Pipe,              // |

         // Values
         Name(String),
         IntValue(i64),
         FloatValue(f64),
         StringValue(String),

         // Comments
         Comment(String),
     }

     impl GraphQLToken {
         pub fn is_punctuator(&self) -> bool { ... }
         pub fn as_punctuator_str(&self) -> Option<&'static str> { ... }
     }
     ```
   - **Benefits:** Type safety, pattern matching, clearer intent, no string allocation for punctuators
   - **Downsides:** Larger enum, more variants to handle
   - **Recommendation:** Do this refactoring; benefits outweigh costs

3. Create new module `/crates/libgraphql-parser/src/token_source.rs`:
   - Define `TokenSource` enum:
     ```rust
     pub enum TokenSource {
         RustTokens(RustToGraphQLTokenSource),
         Str(StrToGraphQLTokenSource),
     }
     ```
   - Implement `Iterator<Item = (GraphQLToken, ParserLocation)>` for `TokenSource`
   - Enum dispatch in `next()` method

4. **Documentation:**
   - Document why enum was chosen over generics (code size, simplicity)
   - Document trade-offs and when to add new variants
   - Note that only a small number of token sources are expected long-term

**Considerations:**
- Enum is closed set but avoids monomorphization
- Only a small number of token sources expected long-term
- `StrToGraphQLTokenSource` is a stub initially (Step 2.x implements it)
- Renamed from "Adapter" to "Source" for clarity (not just adapting, it's the source)
- Comments are lexed as tokens, not skipped
- `cargo clippy --tests` and `cargo test` must pass

---

### Step 1.3: Move and Generalize GraphQLTokenStream
**Outcome:** Token stream works with TokenSource enum

**Tasks:**
1. Move `/crates/libgraphql-macros/src/graphql_token_stream.rs` → `/crates/libgraphql-parser/src/token_stream.rs`

2. Update `GraphQLTokenStream` to use `TokenSource`:
   - Change `adapter: RustToGraphQLTokenAdapter` → `source: TokenSource`
   - Update `buffer: VecDeque<(GraphQLToken, Span)>` → `buffer: VecDeque<(GraphQLToken, ParserLocation)>`
   - Update all methods to use `ParserLocation`

3. Update existing methods for compatibility:
   - Keep existing API surface
   - Add `current_location()` method that returns `&ParserLocation`

4. **Tests:**
   - Port existing token stream tests
   - Add tests with both source types (RustTokens, Str stub)
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Existing code in `libgraphql-macros` continues to work
- Token stream is now source-agnostic
- Renamed from "adapter" to "source" for clarity

---

### Step 1.4: Move RustToGraphQLTokenSource
**Outcome:** Rust token source moved to parser crate, compatible with new infrastructure

**Tasks:**
1. Move `/crates/libgraphql-macros/src/rust_to_graphql_token_adapter.rs` → `/crates/libgraphql-parser/src/rust_token_source.rs`

2. Rename `RustToGraphQLTokenAdapter` → `RustToGraphQLTokenSource`

3. Update to return `ParserLocation::ProcMacro(Span)` instead of raw `Span`

4. Update all imports in `libgraphql-macros`:
   - Import from `libgraphql-parser` directly
   - **Do not re-export parser abstractions** from `libgraphql-macros` (breaking change is acceptable)
   - Remove any existing public parser exports from `libgraphql-macros`

5. **Tests:**
   - Port existing token source tests
   - Verify all proc macro tests still pass
   - `cargo clippy --tests` passes
   - `cargo test` passes

**Considerations:**
- Proc macros still work exactly as before
- This is purely a refactoring with no behavior change
- Breaking change to remove parser exports from macros crate is intentional
- Renamed from "Adapter" to "Source"

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
   ├── ast.rs               (from 1.1)
   ├── location.rs          (from 1.1)
   ├── token.rs             (from 1.2)
   ├── token_source.rs      (from 1.2)
   ├── token_stream.rs      (from 1.3)
   ├── rust_token_source.rs (from 1.4)
   ├── str_token_source.rs  (stub, implemented in Phase 2)
   ├── parser.rs            (moved from macros)
   ├── error.rs             (moved from macros)
   └── tests/               (moved from macros)
   ```

2. Move parser files from `/crates/libgraphql-macros/src/`:
   - `graphql_schema_parser.rs` → `/crates/libgraphql-parser/src/parser.rs`
   - `graphql_parse_error.rs` → `/crates/libgraphql-parser/src/error.rs`
   - Related test files → `/crates/libgraphql-parser/src/tests/`

3. Rename `GraphQLSchemaParser` → `GraphQLParser`:
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

   // Re-export ast module for backward compatibility
   pub use libgraphql_parser::ast;
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

**Lexer Library Consideration:**
Before implementing from scratch, evaluate lexer generator libraries:
- **`logos`**: Fast lexer generator with proc macros, good error recovery
- **`winnow`**: Parser combinator library with streaming support
- **Hand-written**: Full control, potentially simpler for GraphQL's straightforward lexical rules

**Decision criteria:**
1. **Correctness first**: Must handle all GraphQL spec edge cases
2. **Performance second**: Should be competitive with graphql_parser
3. **Debuggability**: Clear error messages for test failures
4. **Maintainability**: Code clarity matters

**Recommendation**: Start with hand-written lexer for maximum control and clarity. If performance becomes an issue, consider `logos` as optimization.

---

### Step 2.1: Basic String Lexer Structure
**Outcome:** Skeleton lexer that tokenizes simple cases

**Tasks:**
1. Create `/crates/libgraphql-parser/src/str_token_source.rs`:
   ```rust
   pub struct StrToGraphQLTokenSource<'a> {
       source: &'a str,
       position: ParserPosition,
       current_offset: usize,
       // ... lexer state
   }
   ```

2. Implement basic tokenization:
   - Whitespace skipping (space, tab, newline, comma per spec)
   - Single-character punctuation: `! $ & ( ) : = @ [ ] { | }` (or explicit enum variants if doing Task 2 of Step 1.2)
   - Simple names (identifiers): `/[_A-Za-z][_0-9A-Za-z]*/`
   - Integer literals (basic cases, no validation yet)

3. Implement `Iterator<Item = (GraphQLToken, ParserLocation)>`:
   - Return `ParserLocation::Source(position)` for each token
   - Update position tracking as tokens are consumed

4. **Error reporting:**
   - Emit helpful error messages with suggestions when possible
   - Include "did you mean?" hints for common mistakes

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
- Matches `RustToGraphQLTokenAdapter` behavior
- User experience: show all errors, not just first one
- Balance recovery quality with complexity

---

### Step 2.7: Comprehensive Lexer Testing
**Outcome:** Lexer is battle-tested and spec-compliant

**Tasks:**
1. Port test cases from graphql-js:
   - Clone https://github.com/graphql/graphql-js
   - Extract lexer tests from `src/__tests__/lexer-test.ts`
   - Convert to Rust test cases in `/crates/libgraphql-parser/src/tests/lexer_tests.rs`
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
                           "Unexpected schema definition in executable document",
                           /* location */,
                           /* suggestion: "Did you mean to use parse_mixed_document()?" */
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
- Errors include suggestions: "Did you mean to use parse_mixed_document()?"
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
               GraphQLToken::BraceOpen => {  // or Punctuator("{") if not using explicit variants
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
                       /* location, suggestion */
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

### Step 4.1: Integrate Parser in SchemaBuilder
**Outcome:** SchemaBuilder can use new parser when feature enabled

**Tasks:**
1. Update `/crates/libgraphql-core/src/schema/schema_builder.rs`:
   - Add conditional compilation with **same function name**:
     ```rust
     #[cfg(feature = "experimental-libgraphql-parser")]
     pub fn build_from_str(&mut self, source: &str) -> Result<&mut Self, SchemaBuildError> {
         use libgraphql_parser::GraphQLParser;
         use libgraphql_parser::ParserPosition;
         use libgraphql_parser::StrToGraphQLTokenSource;
         use libgraphql_parser::TokenSource;

         let position = ParserPosition::new(/* ... */, 0);
         let token_source = StrToGraphQLTokenSource::new(source, position);
         let source = TokenSource::Str(token_source);
         let parser = GraphQLParser::new(source);
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
       use libgraphql_parser::GraphQLParser;
       use libgraphql_parser::ParserPosition;
       use libgraphql_parser::StrToGraphQLTokenSource;
       use libgraphql_parser::TokenSource;

       let position = ParserPosition::new(/* ... */, 0);
       let token_source = StrToGraphQLTokenSource::new(source, position);
       let source = TokenSource::Str(token_source);
       let parser = GraphQLParser::new(source);
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
               use libgraphql_parser::GraphQLParser;
               use libgraphql_parser::ParserPosition;
               use libgraphql_parser::StrToGraphQLTokenSource;
               use libgraphql_parser::TokenSource;

               let position = ParserPosition::new(/* ... */, 0);
               let token_source = StrToGraphQLTokenSource::new(source, position);
               let source = TokenSource::Str(token_source);
               let parser = GraphQLParser::new(source);
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
- ✅ All parser infrastructure moved to `libgraphql-core`
- ✅ `TokenAdapter` enum implemented
- ✅ Feature flag infrastructure in place
- ✅ All existing tests pass

**Phase 2 Complete:**
- ✅ `StringToGraphQLTokenAdapter` fully implements GraphQL lexer spec
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
- ✅ (Eventually) `unified-parser` is the default

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
