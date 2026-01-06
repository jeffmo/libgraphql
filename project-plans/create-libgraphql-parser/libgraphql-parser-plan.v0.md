# Implementation Plan: Unified GraphQL Parser Infrastructure

## Overview
Move from dual parsing approach (`graphql_parser` + `GraphQLSchemaParser`) to unified token-based parser that supports schema documents, executable documents, and mixed documents.

**Key Decisions:**
- Use enum-based `TokenAdapter` (not generics) to avoid monomorphization
- Extend existing `FilePosition` for location tracking (add `offset` field if needed)
- Keep `graphql_parser` AST types for now (future work: custom AST)
- Three parser methods: `parse_schema_document()`, `parse_executable_document()`, `parse_mixed_document()`
- Feature flag (`unified-parser`) to toggle between old and new implementation
- Comprehensive test suite from graphql-js and graphql-parser is mandatory

**Critical Files:**
- `/crates/libgraphql-core/src/loc.rs` - Location tracking
- `/crates/libgraphql-macros/src/graphql_schema_parser.rs` - Parser to move/extend
- `/crates/libgraphql-macros/src/graphql_token_stream.rs` - Token stream to generalize
- `/crates/libgraphql-macros/src/rust_to_graphql_token_adapter.rs` - Token adapter

---

## Phase 1: Foundation & Infrastructure
*Goal: Prepare architecture for unified parsing without breaking existing functionality*

### Step 1.1: Extend FilePosition and Create Location Abstraction
**Outcome:** Location tracking ready for both proc-macro and string-based parsing

**Tasks:**
1. Extend `FilePosition` in `/crates/libgraphql-core/src/loc.rs`:
   - Add `offset: usize` field for efficient string slicing
   - Add constructor methods: `new()`, `from_str_offset()`
   - Add `advance_line()`, `advance_col()`, `advance_offset()` methods for lexer use

2. Create `ParserLocation` enum in new file `/crates/libgraphql-core/src/parser/location.rs`:
   ```rust
   pub enum ParserLocation {
       ProcMacro(proc_macro2::Span),
       Source(FilePosition),
   }
   ```
   - Implement conversion methods: `to_file_position()`, `to_ast_pos()`
   - This bridges proc-macro context and string-based parsing

3. **Tests:**
   - Unit tests for FilePosition methods
   - Conversion tests between ParserLocation variants

**Considerations:**
- `proc_macro2::Span` doesn't expose line/col in stable Rust, so `ProcMacro` variant converts to dummy `FilePosition` when needed
- `offset` field enables efficient error reporting with source snippets

---

### Step 1.2: Create TokenAdapter Enum
**Outcome:** Abstraction layer for different token sources

**Tasks:**
1. Create new module `/crates/libgraphql-core/src/parser/token_adapter.rs`:
   - Define `TokenAdapter` enum:
     ```rust
     pub enum TokenAdapter {
         RustTokens(RustToGraphQLTokenAdapter),
         String(StringToGraphQLTokenAdapter),
     }
     ```
   - Implement `Iterator<Item = (GraphQLToken, ParserLocation)>` for `TokenAdapter`
   - Enum dispatch in `next()` method

2. Move `GraphQLToken` from `/crates/libgraphql-macros/src/rust_to_graphql_token_adapter.rs` to `/crates/libgraphql-core/src/parser/token.rs`:
   - Keep existing 5 variants: `Punctuator`, `Name`, `IntValue`, `FloatValue`, `StringValue`
   - Update all imports in macros crate

3. **Documentation:**
   - Document why enum was chosen over generics (code size, simplicity)
   - Document trade-offs and when to add new variants

**Considerations:**
- Enum is closed set but avoids monomorphization
- Only 2 adapters expected long-term
- `StringToGraphQLTokenAdapter` is a stub initially (Step 2.x implements it)

---

### Step 1.3: Move and Generalize GraphQLTokenStream
**Outcome:** Token stream works with TokenAdapter enum

**Tasks:**
1. Move `/crates/libgraphql-macros/src/graphql_token_stream.rs` → `/crates/libgraphql-core/src/parser/token_stream.rs`

2. Update `GraphQLTokenStream` to use `TokenAdapter`:
   - Change `adapter: RustToGraphQLTokenAdapter` → `adapter: TokenAdapter`
   - Update `buffer: VecDeque<(GraphQLToken, Span)>` → `buffer: VecDeque<(GraphQLToken, ParserLocation)>`
   - Update all methods to use `ParserLocation`

3. Update existing methods for compatibility:
   - Keep existing API surface
   - Add `current_location()` method that returns `&ParserLocation`

4. **Tests:**
   - Port existing token stream tests
   - Add tests with both adapter types (RustTokens, String stub)

**Considerations:**
- Existing code in `libgraphql-macros` continues to work
- Token stream is now adapter-agnostic

---

### Step 1.4: Move RustToGraphQLTokenAdapter
**Outcome:** Adapter moved to core, compatible with new infrastructure

**Tasks:**
1. Move `/crates/libgraphql-macros/src/rust_to_graphql_token_adapter.rs` → `/crates/libgraphql-core/src/parser/rust_token_adapter.rs`

2. Update to return `ParserLocation::ProcMacro(Span)` instead of raw `Span`

3. Update all imports in `libgraphql-macros`:
   - Re-export from `libgraphql-core` if needed for backward compatibility

4. **Tests:**
   - Port existing adapter tests
   - Verify all proc macro tests still pass

**Considerations:**
- Proc macros still work exactly as before
- This is purely a refactoring with no behavior change

---

### Step 1.5: Add Cargo Feature Flag Infrastructure
**Outcome:** Ability to toggle between old and new parser

**Tasks:**
1. Update `/Cargo.toml` workspace and `/crates/libgraphql-core/Cargo.toml`:
   ```toml
   [features]
   default = ["macros"]
   macros = ["dep:libgraphql-macros"]
   unified-parser = []  # New feature flag
   ```

2. Add conditional compilation in future parser call sites:
   ```rust
   #[cfg(feature = "unified-parser")]
   use crate::parser::GraphQLParser;

   #[cfg(not(feature = "unified-parser"))]
   use graphql_parser;
   ```

3. Update CI to test both feature configurations

4. **Documentation:**
   - Document feature flag in main README
   - Explain when to use which parser

**Considerations:**
- Default stays with `graphql_parser` until new parser is battle-tested
- Allows gradual migration and confidence building
- Users can opt into new parser early for testing

---

### Step 1.6: Move GraphQLSchemaParser to Core
**Outcome:** Parser infrastructure in libgraphql-core, ready for extension

**Tasks:**
1. Create `/crates/libgraphql-core/src/parser/` module structure:
   ```
   parser/
   ├── mod.rs
   ├── location.rs          (from 1.1)
   ├── token.rs             (from 1.2)
   ├── token_adapter.rs     (from 1.2)
   ├── token_stream.rs      (from 1.3)
   ├── rust_token_adapter.rs (from 1.4)
   ├── string_token_adapter.rs (stub, implemented in Phase 2)
   ├── parser.rs            (moved from macros)
   ├── error.rs             (moved from macros)
   └── tests/               (moved from macros)
   ```

2. Move parser files from `/crates/libgraphql-macros/src/`:
   - `graphql_schema_parser.rs` → `parser/parser.rs`
   - `graphql_parse_error.rs` → `parser/error.rs`
   - Related test files → `parser/tests/`

3. Rename `GraphQLSchemaParser` → `GraphQLParser`:
   - Keep schema parsing as `parse_schema_document()` method
   - Add stub methods for `parse_executable_document()` and `parse_mixed_document()` (implemented in Phase 3)

4. Update `libgraphql-macros` to re-export from core:
   ```rust
   pub use libgraphql_core::parser::GraphQLParser;
   ```

5. **Tests:**
   - All existing parser tests pass
   - Verify proc macros still work

**Considerations:**
- This is a large refactoring but purely organizational
- Existing functionality must remain unchanged
- No new features yet, just moving code

---

## Phase 2: String Lexer Implementation
*Goal: Implement StringToGraphQLTokenAdapter with full GraphQL spec compliance*

### Step 2.1: Basic String Lexer Structure
**Outcome:** Skeleton lexer that tokenizes simple cases

**Tasks:**
1. Create `/crates/libgraphql-core/src/parser/string_token_adapter.rs`:
   ```rust
   pub struct StringToGraphQLTokenAdapter {
       source: String,
       position: FilePosition,
       current_offset: usize,
       // ... lexer state
   }
   ```

2. Implement basic tokenization:
   - Whitespace skipping (space, tab, newline, comma per spec)
   - Single-character punctuation: `! $ & ( ) : = @ [ ] { | }`
   - Simple names (identifiers): `/[_A-Za-z][_0-9A-Za-z]*/`
   - Integer literals (basic cases, no validation yet)

3. Implement `Iterator<Item = (GraphQLToken, ParserLocation)>`:
   - Return `ParserLocation::Source(position)` for each token
   - Update position tracking as tokens are consumed

4. **Tests:**
   - Simple schema: `type Query { hello: String }`
   - Verify correct tokenization and positions

**Considerations:**
- This is a foundation; edge cases come in later steps
- Focus on correct position tracking from the start
- Error handling is minimal initially (return `None` on error)

---

### Step 2.2: Comments and Multi-Character Punctuation
**Outcome:** Handle GraphQL comments and spread operator

**Tasks:**
1. Add comment handling:
   - Recognize `#` followed by anything until newline
   - Skip comments entirely (don't emit tokens)
   - Update position tracking to skip comment content

2. Add multi-character punctuation:
   - Recognize `...` (spread operator)
   - Disambiguate from three separate `.` tokens

3. **Tests:**
   - Schema with comments: `# This is a comment\ntype Query { ... }`
   - Spread in fragments
   - Comments at EOF

**Considerations:**
- Comments are "ignored tokens" per spec
- Spread operator requires 2-character lookahead
- Position tracking must account for skipped comment content

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
   - Emit `GraphQLToken::Name(String)`

2. Handle keyword vs name distinction:
   - GraphQL has no reserved keywords; all keywords are contextual
   - `true`, `false`, `null` are values, not names
   - Parser handles context, lexer just emits Name tokens

3. **Tests:**
   - Valid names: `hello`, `_private`, `type2`
   - Invalid names: `2type`, `hello-world`
   - Keywords as names: `type`, `query`, `mutation` (valid in some contexts)

**Considerations:**
- GraphQL keywords are context-dependent
- Lexer doesn't need keyword table; parser handles semantics
- Name validation is straightforward regex

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
   - Convert to Rust test cases in `/crates/libgraphql-core/src/parser/tests/lexer_tests.rs`
   - Ensure 100% of graphql-js lexer tests pass

2. Port test cases from graphql-parser:
   - Clone https://github.com/graphql-rust/graphql-parser
   - Extract relevant lexer/parser tests
   - Convert to Rust test cases
   - Ensure compatibility

3. Add fuzzing tests:
   - Use `cargo-fuzz` or `proptest`
   - Generate random GraphQL-like strings
   - Verify lexer doesn't panic

4. Benchmark lexer performance:
   - Compare against `graphql_parser` on various inputs
   - Identify performance bottlenecks
   - Optimize hot paths (string scanning, allocation)

5. **Success criteria:**
   - 100% graphql-js lexer tests pass
   - 100% graphql-parser lexer tests pass
   - No panics on fuzz tests
   - Performance within 2x of `graphql_parser`

**Considerations:**
- This step is non-negotiable; comprehensive testing is mandatory
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
1. Add fragment parsing:
   - Recognize `fragment` keyword
   - Parse fragment name
   - Parse type condition: `on TypeName`
   - Parse directives
   - Parse selection set

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

**Considerations:**
- Fragments reference types from schema (validation is separate concern)
- Fragments can spread other fragments (no cycle detection in parser)

---

### Step 3.3: Implement parse_executable_document()
**Outcome:** Dedicated method for parsing operations and fragments

**Tasks:**
1. Add method to `GraphQLParser`:
   ```rust
   pub fn parse_executable_document(self) -> Result<ast::operation::Document, Errors> {
       let mut definitions = Vec::new();
       while !self.tokens.is_at_end() {
           match self.parse_executable_definition() {
               Ok(def) => definitions.push(def),
               Err(err) => self.errors.add(err),
           }
       }
       if self.errors.has_errors() {
           Err(self.errors)
       } else {
           Ok(ast::operation::Document { definitions })
       }
   }
   ```

2. Implement `parse_executable_definition()`:
   - Peek at token to determine type:
     - `query`/`mutation`/`subscription` → operation
     - `fragment` → fragment definition
     - `{` → anonymous query (shorthand)
   - Delegate to appropriate parser

3. **Tests:**
   - Document with single operation
   - Document with multiple operations
   - Document with operations and fragments
   - Anonymous queries
   - Error recovery across multiple definitions

**Considerations:**
- Similar structure to `parse_schema_document()`
- Error recovery: skip to next definition on parse error

---

### Step 3.4: Implement parse_mixed_document()
**Outcome:** Parse documents containing both schema and executable definitions

**Tasks:**
1. Define unified document type in `/crates/libgraphql-core/src/parser/mod.rs`:
   ```rust
   pub struct MixedDocument {
       pub schema_definitions: Vec<ast::schema::Definition>,
       pub executable_definitions: Vec<ast::operation::Definition>,
   }
   ```

2. Add method to `GraphQLParser`:
   ```rust
   pub fn parse_mixed_document(self) -> Result<MixedDocument, Errors> {
       let mut schema_defs = Vec::new();
       let mut exec_defs = Vec::new();

       while !self.tokens.is_at_end() {
           let (token, _) = self.tokens.peek();
           match token {
               GraphQLToken::Name(kw) if is_schema_keyword(kw) => {
                   schema_defs.push(self.parse_schema_definition()?);
               }
               GraphQLToken::Name(kw) if is_exec_keyword(kw) => {
                   exec_defs.push(self.parse_executable_definition()?);
               }
               GraphQLToken::Punctuator("{") => {
                   exec_defs.push(self.parse_executable_definition()?);
               }
               _ => return Err(error),
           }
       }

       Ok(MixedDocument { schema_definitions: schema_defs, executable_definitions: exec_defs })
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

**Considerations:**
- This is the primary use case driving the refactor
- Must handle interleaved definitions
- Error in schema definition shouldn't prevent parsing operations

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
   - Add conditional compilation:
     ```rust
     #[cfg(feature = "unified-parser")]
     pub fn build_from_str_new(&mut self, source: &str) -> Result<&mut Self, SchemaBuildError> {
         use crate::parser::{GraphQLParser, TokenAdapter, StringToGraphQLTokenAdapter};

         let adapter = TokenAdapter::String(
             StringToGraphQLTokenAdapter::new(source, self.current_file_path())
         );
         let parser = GraphQLParser::new(adapter);
         let document = parser.parse_schema_document()?;
         self.process_document(document)
     }

     #[cfg(not(feature = "unified-parser"))]
     pub fn build_from_str(&mut self, source: &str) -> Result<&mut Self, SchemaBuildError> {
         // Existing graphql_parser implementation
     }
     ```

2. Keep both implementations during transition:
   - `unified-parser` feature selects new parser
   - Default uses `graphql_parser`

3. **Tests:**
   - Run all existing schema builder tests with both feature flags
   - Ensure identical behavior

**Considerations:**
- Gradual migration path
- Can compare outputs between parsers
- Bug compatibility may differ (new parser may be more/less strict)

---

### Step 4.2: Integrate Parser in QueryBuilder/MutationBuilder/SubscriptionBuilder
**Outcome:** Operation builders use new parser when feature enabled

**Tasks:**
1. Update operation builders:
   - `/crates/libgraphql-core/src/operation/query_builder.rs`
   - `/crates/libgraphql-core/src/operation/mutation_builder.rs`
   - `/crates/libgraphql-core/src/operation/subscription_builder.rs`

2. Add conditional compilation similar to SchemaBuilder:
   ```rust
   #[cfg(feature = "unified-parser")]
   fn build_from_str_new(...) { /* new parser */ }

   #[cfg(not(feature = "unified-parser"))]
   fn build_from_str(...) { /* graphql_parser */ }
   ```

3. **Tests:**
   - Run all operation builder tests with both feature flags
   - Verify behavior parity

**Considerations:**
- Operation builders currently use `graphql_parser::query::parse_query`
- New parser should produce identical AST structures
- Any differences need investigation

---

### Step 4.3: Add Mixed Document Support
**Outcome:** New API for parsing mixed documents

**Tasks:**
1. Create new builder in `/crates/libgraphql-core/src/mixed_document_builder.rs`:
   ```rust
   pub struct MixedDocumentBuilder {
       schema_builder: SchemaBuilder,
       executable_builder: ExecutableDocumentBuilder,
   }

   impl MixedDocumentBuilder {
       pub fn build_from_str(&mut self, source: &str) -> Result<...> {
           #[cfg(feature = "unified-parser")]
           {
               let parser = GraphQLParser::new(/* ... */);
               let doc = parser.parse_mixed_document()?;
               self.schema_builder.process_definitions(doc.schema_definitions)?;
               self.executable_builder.process_definitions(doc.executable_definitions)?;
               Ok(self)
           }

           #[cfg(not(feature = "unified-parser"))]
           compile_error!("Mixed documents require unified-parser feature");
       }
   }
   ```

2. Expose in public API:
   - Add to `/crates/libgraphql/src/lib.rs`
   - Only available with `unified-parser` feature

3. **Tests:**
   - Parse mixed document
   - Validate schema types are accessible
   - Validate operations are accessible
   - Integration test with real use case

**Considerations:**
- This is net-new functionality (solving the original problem)
- Only available with new parser
- Should encourage adoption of unified-parser feature

---

### Step 4.4: Performance Benchmarking
**Outcome:** Quantified performance comparison vs graphql_parser

**Tasks:**
1. Create benchmarks in `/crates/libgraphql-core/benches/parser_bench.rs`:
   - Use `criterion` crate
   - Benchmark scenarios:
     - Small schema (10 types)
     - Medium schema (100 types)
     - Large schema (1000 types)
     - Small operation
     - Complex operation (deep nesting)
     - Mixed document

2. Compare performance:
   - `graphql_parser` baseline
   - New parser (both lexer and parser)
   - Identify bottlenecks

3. Optimize hot paths:
   - String allocation
   - Token buffer management
   - AST node construction

4. Document results:
   - Add benchmark results to README
   - Note acceptable performance threshold (within 2x)

**Considerations:**
- Performance regression is acceptable if < 2x slower
- Functionality and correctness > raw speed
- Optimization is iterative; don't over-optimize initially

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
1. Update GitHub Actions workflow (or equivalent):
   - Test with `--features unified-parser`
   - Test without feature (default)
   - Run benchmarks on performance regression

2. Add parser-specific test jobs:
   - Lexer fuzzing
   - Parser fuzzing
   - Compliance test suite (graphql-js, graphql-parser)

3. Code coverage:
   - Ensure parser code has high coverage
   - Use `cargo-tarpaulin` or `cargo-llvm-cov`

4. **Success criteria:**
   - All tests pass with both feature flags
   - No performance regression > 2x
   - Code coverage > 90% for parser module

**Considerations:**
- CI validates both code paths
- Prevents regressions in either parser
- Builds confidence for eventual default switch

---

### Step 4.7: Default Feature Flag Flip
**Outcome:** unified-parser becomes the default

**Tasks:**
1. Update `/Cargo.toml`:
   ```toml
   [features]
   default = ["macros", "unified-parser"]
   macros = ["dep:libgraphql-macros"]
   unified-parser = []
   legacy-parser = []  # Opt-in to graphql_parser
   ```

2. Invert conditional compilation:
   - `#[cfg(not(feature = "legacy-parser"))]` → new parser
   - `#[cfg(feature = "legacy-parser")]` → old parser

3. Update documentation:
   - Note that `unified-parser` is now default
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
