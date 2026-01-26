# GraphQLParser Implementation Plan

## Overview

Implement `GraphQLParser<'src, S: GraphQLTokenSource<'src>>` - a recursive descent parser generic over token sources, enabling unified parsing from both `StrGraphQLTokenSource` and `RustMacroGraphQLTokenSource`.

## Implementation Status

| Step | Description | Status |
|------|-------------|--------|
| 1 | Core Parser Infrastructure | ✅ Complete |
| 2 | ParseResult Type | ✅ Complete |
| 3 | Value Parsing | ✅ Complete |
| 4 | Type Annotation Parsing | ✅ Complete |
| 5 | Directive Annotation Parsing | ✅ Complete |
| 6 | Selection Set Parsing | ✅ Complete |
| 7 | Operation Parsing | ✅ Complete |
| 8 | Fragment Parsing | ✅ Complete |
| 9 | Type Definition Parsing | ✅ Complete |
| 10 | Type Extension Parsing | ✅ Complete |
| 11 | Document Parsing | ✅ Complete |
| 12 | Wire Up Exports | ✅ Complete |

**Test Status:** 383 tests passing, 4 doc-tests passing

## Known Issues

_None — all known bugs have been fixed._

## Remaining Work

1. **`RustMacroGraphQLTokenSource` testing** — parser tests only cover `StrGraphQLTokenSource`; add tests verifying parser works with proc-macro token source
2. **Vendored tests** — port test cases from graphql-js (MIT) and graphql-parser (MIT) for comprehensive spec coverage
3. **Performance benchmarks** — add benchmarks comparing against `graphql_parser` crate
4. **Fuzz testing** — set up `cargo-fuzz` for security-critical lexer/parser code
5. **Custom AST** (future) — currently uses `graphql_parser` crate AST types; custom AST deferred per Open Questions

---

## Current State

### Completed Infrastructure
- `GraphQLTokenKind<'src>`, `GraphQLToken<'src>`, `GraphQLTriviaToken<'src>` - with `Cow` for zero-copy
- `GraphQLTokenSource<'src>` trait - marker trait for token iterators
- `GraphQLTokenStream<'src, S>` - lookahead buffering, peek/consume
- `StrGraphQLTokenSource<'src>` - complete (~1130 lines), 60+ tests
- Error types: `GraphQLParseError`, `GraphQLParseErrorKind`, `GraphQLErrorNote`, etc.
- Supporting types: `DefinitionKind`, `DocumentKind`, `ReservedNameContext`, `ValueParsingError`
- `SourcePosition`, `GraphQLSourceSpan` - with file path support

### AST Strategy
- Continue using `graphql_parser` crate re-exports temporarily (already in `ast.rs`)
- Custom AST is future work per design doc "Open Questions" section

### Terminology (aligned with libgraphql-core::types)
- **TypeAnnotation** - Type syntax on fields/params/vars (e.g., `String!`, `[Int]!`)
  - **NamedTypeAnnotation** - Named type ref with nullability (e.g., `String!`)
  - **ListTypeAnnotation** - List type wrapper (e.g., `[String]!`)
- **Directive (types::Directive)** - Directive definition (`directive @name on LOCATIONS`)
- **DirectiveAnnotation** - Directive usage/application (`@name(args)`)

---

## Implementation Steps (Completed)

### Step 1: Core Parser Infrastructure ✅
**File:** `/crates/libgraphql-parser/src/graphql_parser.rs`

**Implemented:**
```rust
pub struct GraphQLParser<'src, TTokenSource: GraphQLTokenSource<'src>> {
    token_stream: GraphQLTokenStream<'src, TTokenSource>,
    errors: Vec<GraphQLParseError>,
    delimiter_stack: SmallVec<[OpenDelimiter; 8]>,  // Changed: SmallVec for perf
}

struct OpenDelimiter {
    kind: char,
    span: GraphQLSourceSpan,
    context: DelimiterContext,
}

enum DelimiterContext {
    SchemaDefinition, ObjectTypeDefinition, InterfaceDefinition,
    EnumDefinition, InputObjectDefinition, SelectionSet,
    FieldArguments, DirectiveArguments, VariableDefinitions,
    ListType, ListValue, ObjectValue, ArgumentDefinitions,
}
```

**Changes from original plan:**
- Uses `SmallVec<[OpenDelimiter; 8]>` instead of `Vec` for typical nesting depths
- Added `ConstContext` enum for better error msgs in const value contexts
- `expect_name()` returns `Cow<'src, str>` instead of `String` (zero-copy)
- Added `expect_name_only()` variant that doesn't return span

**Parser helpers implemented:**
- `new(token_source: S) -> Self`
- `expect(&mut self, kind: &GraphQLTokenKind) -> Result<GraphQLToken, ()>`
- `expect_name(&mut self) -> Result<(Cow<'src, str>, GraphQLSourceSpan), ()>`
- `expect_name_only(&mut self) -> Result<Cow<'src, str>, ()>`
- `expect_keyword(&mut self, kw: &str) -> Result<(), ()>`
- `peek_is_keyword(&mut self, kw: &str) -> bool`
- `peek_is(&mut self, kind: &GraphQLTokenKind) -> bool`
- `record_error(&mut self, err: GraphQLParseError)`
- `push_delimiter(&mut self, kind: char, span: GraphQLSourceSpan, ctx: DelimiterContext)`
- `pop_delimiter(&mut self) -> Option<OpenDelimiter>`
- `recover_to_next_definition(&mut self)`
- `looks_like_definition_start(&mut self, keyword: &str) -> bool`

#### Critical: `expect_name()` and `true`/`false`/`null` tokens ✅

**Implemented as planned:** `expect_name()` accepts `Name`, `True`, `False`, `Null` tokens as valid names.

### Step 2: ParseResult Type ✅
**File:** `/crates/libgraphql-parser/src/parse_result.rs`

**Implemented:**
```rust
pub struct ParseResult<TAst> {
    ast: Option<TAst>,           // Changed: private field
    pub errors: Vec<GraphQLParseError>,
}

impl<TAst> ParseResult<TAst> {
    pub(crate) fn ok(ast: TAst) -> Self;
    pub(crate) fn err(errors: Vec<GraphQLParseError>) -> Self;
    pub(crate) fn recovered(ast: TAst, errors: Vec<GraphQLParseError>) -> Self;
    pub fn valid_ast(&self) -> Option<&TAst>;     // Added: only if no errors
    pub fn ast(&self) -> Option<&TAst>;           // Best-effort
    pub fn into_valid_ast(self) -> Option<TAst>;  // Added
    pub fn into_ast(self) -> Option<TAst>;        // Added
    pub fn is_ok(&self) -> bool;
    pub fn has_errors(&self) -> bool;
    pub fn format_errors(&self, source: Option<&str>) -> String;
}

impl<TAst> From<ParseResult<TAst>> for Result<TAst, Vec<GraphQLParseError>>
```

**Changes from original plan:**
- `ast` field is private (access via methods)
- Added `valid_ast()` / `into_valid_ast()` for strict mode
- Added consuming variants `into_ast()` / `into_valid_ast()`
- Constructors are `pub(crate)` not public

### Step 3: Value Parsing ✅
**Methods implemented:**
- `parse_value(&mut self, context: ConstContext) -> Result<ast::Value, ()>`
- `parse_list_value(&mut self, context: ConstContext) -> Result<ast::Value, ()>`
- `parse_object_value(&mut self, context: ConstContext) -> Result<ast::Value, ()>`

**Changes from original plan:**
- Uses `ConstContext` enum instead of `const_only: bool` for better error messages
- `ConstContext` variants: `AllowVariables`, `VariableDefaultValue`, `DirectiveArgument`, `InputDefaultValue`

### Step 4: Type Annotation Parsing ✅
**Methods implemented:**
- `parse_type_annotation(&mut self) -> Result<ast::operation::Type, ()>`
- `parse_named_type_annotation(&mut self) -> Result<ast::operation::Type, ()>`
- `parse_list_type_annotation(&mut self) -> Result<ast::operation::Type, ()>`
- `parse_schema_type_annotation(&mut self) -> Result<ast::schema::Type, ()>`  // Added
- `parse_schema_list_type(&mut self) -> Result<ast::schema::Type, ()>`        // Added

**Changes from original plan:**
- Added separate schema type methods for schema definition context
- Added `parse_list_type_annotation()` helper

### Step 5: Directive Annotation Parsing ✅
**Methods implemented:**
- `parse_directive_annotations(&mut self) -> Result<Vec<ast::operation::Directive>, ()>`
- `parse_directive_annotation(&mut self) -> Result<ast::operation::Directive, ()>`
- `parse_const_directive_annotations(&mut self) -> Result<Vec<ast::schema::Directive>, ()>`  // Added
- `parse_const_directive_annotation(&mut self) -> Result<ast::schema::Directive, ()>`        // Added
- `parse_arguments(&mut self, context: ConstContext) -> Result<Vec<(String, ast::Value)>, ()>`
- `parse_const_arguments(&mut self) -> Result<Vec<(String, ast::Value)>, ()>`               // Added

**Changes from original plan:**
- Added const variants for schema definition contexts

### Step 6: Selection Set Parsing ✅
**Methods implemented:**
- `parse_selection_set(&mut self) -> Result<ast::operation::SelectionSet, ()>`
- `parse_selection(&mut self) -> Result<ast::operation::Selection, ()>`
- `parse_field(&mut self) -> Result<ast::operation::Field, ()>`
- `parse_arguments(&mut self, context: ConstContext) -> ...`
- `parse_fragment_spread(&mut self) -> Result<ast::operation::Selection, ()>`
- `parse_inline_fragment(&mut self) -> Result<ast::operation::Selection, ()>`

### Step 7: Operation Parsing ✅
**Methods implemented:**
- `parse_operation_definition(&mut self) -> Result<ast::operation::OperationDefinition, ()>`
- `parse_variable_definitions(&mut self) -> Result<Vec<ast::operation::VariableDefinition>, ()>`
- `parse_variable_definition(&mut self) -> Result<ast::operation::VariableDefinition, ()>`

### Step 8: Fragment Parsing ✅
**Methods implemented:**
- `parse_fragment_definition(&mut self) -> Result<ast::operation::FragmentDefinition, ()>`
- `parse_type_condition(&mut self) -> Result<ast::operation::TypeCondition, ()>`

Reserved name handling for `on` implemented as planned.

### Step 9: Type Definition Parsing ✅
**Methods implemented:**
- `parse_description(&mut self) -> Option<String>`
- `parse_schema_definition(&mut self) -> Result<ast::schema::SchemaDefinition, ()>`
- `parse_scalar_type_definition(&mut self, desc: Option<String>) -> Result<ast::schema::TypeDefinition, ()>`
- `parse_object_type_definition(&mut self, desc: Option<String>) -> Result<ast::schema::TypeDefinition, ()>`
- `parse_interface_type_definition(&mut self, desc: Option<String>) -> Result<ast::schema::TypeDefinition, ()>`
- `parse_union_type_definition(&mut self, desc: Option<String>) -> Result<ast::schema::TypeDefinition, ()>`
- `parse_enum_type_definition(&mut self, desc: Option<String>) -> Result<ast::schema::TypeDefinition, ()>`
- `parse_input_object_type_definition(&mut self, desc: Option<String>) -> Result<ast::schema::TypeDefinition, ()>`
- `parse_directive_definition(&mut self, desc: Option<String>) -> Result<ast::schema::DirectiveDefinition, ()>`
- `parse_implements_interfaces(&mut self) -> Result<Vec<String>, ()>`
- `parse_fields_definition(&mut self) -> Result<Vec<ast::schema::Field>, ()>`
- `parse_field_definition(&mut self) -> Result<ast::schema::Field, ()>`
- `parse_arguments_definition(&mut self) -> Result<Vec<ast::schema::InputValue>, ()>`
- `parse_input_fields_definition(&mut self) -> Result<Vec<ast::schema::InputValue>, ()>`
- `parse_input_value_definition(&mut self) -> Result<ast::schema::InputValue, ()>`
- `parse_enum_values_definition(&mut self) -> Result<Vec<ast::schema::EnumValue>, ()>`
- `parse_enum_value_definition(&mut self) -> Result<ast::schema::EnumValue, ()>`
- `parse_directive_locations(&mut self) -> Result<Vec<ast::schema::DirectiveLocation>, ()>`
- `parse_directive_location(&mut self) -> Result<ast::schema::DirectiveLocation, ()>`

**Changes from original plan:**
- Type def methods take description as param (already parsed by caller)
- Added many helper methods not in original plan

### Step 10: Type Extension Parsing ✅
**Methods implemented:**
- `parse_type_extension(&mut self) -> Result<ast::schema::TypeExtension, ()>`
- `parse_scalar_type_extension(&mut self) -> Result<ast::schema::TypeExtension, ()>`
- `parse_object_type_extension(&mut self) -> Result<ast::schema::TypeExtension, ()>`
- `parse_interface_type_extension(&mut self) -> Result<ast::schema::TypeExtension, ()>`
- `parse_union_type_extension(&mut self) -> Result<ast::schema::TypeExtension, ()>`
- `parse_enum_type_extension(&mut self) -> Result<ast::schema::TypeExtension, ()>`
- `parse_input_object_type_extension(&mut self) -> Result<ast::schema::TypeExtension, ()>`

**Note:** Schema extension (`extend schema`) handled within `parse_type_extension()`

### Step 11: Document Parsing ✅
**Public API implemented:**
```rust
impl<'src, S: GraphQLTokenSource<'src>> GraphQLParser<'src, S> {
    pub fn parse_schema_document(mut self) -> ParseResult<ast::schema::Document>;
    pub fn parse_executable_document(mut self) -> ParseResult<ast::operation::Document>;
    pub fn parse_mixed_document(mut self) -> ParseResult<ast::MixedDocument>;
}
```

**Internal methods:**
- `parse_schema_definition_item(&mut self) -> Result<ast::schema::Definition, ()>`
- `parse_executable_definition_item(&mut self) -> Result<ast::operation::Definition, ()>`
- `parse_mixed_definition_item(&mut self) -> Result<ast::MixedDefinition, ()>`

**Changes from original plan:**
- Internal methods use `*_item` suffix instead of generic `parse_definition()`

### Step 12: Wire Up Exports ✅
**File:** `/crates/libgraphql-parser/src/lib.rs`

**Exports:**
```rust
pub use graphql_parser::GraphQLParser;
pub use parse_result::ParseResult;
pub use definition_kind::DefinitionKind;
pub use document_kind::DocumentKind;
pub use graphql_error_note::GraphQLErrorNote;
pub use graphql_error_note::GraphQLErrorNotes;
pub use graphql_error_note_kind::GraphQLErrorNoteKind;
pub use graphql_parse_error::GraphQLParseError;
pub use graphql_parse_error_kind::GraphQLParseErrorKind;
pub use graphql_source_span::GraphQLSourceSpan;
pub use graphql_string_parsing_error::GraphQLStringParsingError;
pub use graphql_token_stream::GraphQLTokenStream;
pub use reserved_name_context::ReservedNameContext;
pub use source_position::SourcePosition;
pub use value_parsing_error::ValueParsingError;
// Plus pub mod: ast, token, token_source
```

---

## Error Handling Strategy

Per `graphql-parse-error.md`, uses existing `GraphQLParseError` infrastructure:

### Error Structure
```rust
GraphQLParseError {
    message: String,
    span: GraphQLSourceSpan,
    kind: GraphQLParseErrorKind,
    notes: GraphQLErrorNotes,  // SmallVec<[GraphQLErrorNote; 2]>
}
```

### Error Kinds (GraphQLParseErrorKind)
| Kind | Usage |
|------|-------|
| `LexerError` | Wrap `GraphQLTokenKind::Error` tokens |
| `UnexpectedToken { expected, found }` | Expected specific token(s) but got something else |
| `UnexpectedEof` | Document ended before complete construct |
| `UnclosedDelimiter` | Open `{`, `[`, `(` without matching close |
| `MismatchedDelimiter` | Wrong close delimiter |
| `InvalidValue` | Value parse errors (int overflow, bad string escape) |
| `ReservedName { name, context }` | Reserved name in wrong context |
| `WrongDocumentKind { found, document_kind }` | Definition not allowed in document type |
| `InvalidEmptyConstruct { construct }` | Empty `{}`, `()` where content required |
| `InvalidSyntax` | Catch-all |

### Recovery Strategy
- Skip tokens until next definition keyword
- Continue parsing to collect multiple errors in one pass

---

## Files Created/Modified

| File | Status |
|------|--------|
| `src/graphql_parser.rs` | ✅ Created (~3200 lines) |
| `src/parse_result.rs` | ✅ Created (~200 lines) |
| `src/lib.rs` | ✅ Modified |
| `src/ast.rs` | ✅ Created (AST re-exports + MixedDocument) |
| `src/tests/graphql_parser_tests.rs` | ✅ Created (~2200 lines, 175 tests) |
| `src/tests/parse_result_tests.rs` | ✅ Created |

---

## Verification ✅

1. ✅ `cargo check --package libgraphql-parser` - compiles
2. ✅ `cargo clippy --package libgraphql-parser --tests` - no warnings
3. ✅ `cargo test --package libgraphql-parser` - 379 pass, 1 ignored
4. ✅ Manual test: parse schema/query documents (doc-tests)
5. ✅ Error recovery: multiple errors reported in one pass

---

## Test Coverage

### Implemented Test Categories
- Value parsing (int, float, string, bool, null, enum, list, object, variable)
- Type annotations (named, non-null, list, nested)
- Directive annotations (simple, with args, multiple)
- Selection sets (fields, aliases, args, nested, fragments)
- Operations (query, mutation, subscription, variables, directives)
- Fragment definitions
- Schema definitions (all type kinds)
- Type extensions (all extension kinds)
- Document type enforcement
- Error recovery
- Lexer error integration
- Edge cases (keywords as names, Unicode, etc.)

### Special Name Tests ✅
All `true`/`false`/`null` name context tests implemented:
- ✅ Field names in selection sets
- ✅ Reserved enum value errors
- ✅ Reserved fragment name `on` error

---

## Notes

- AST uses `graphql_parser` crate types temporarily; custom AST is future work
- Trivia stays on tokens, not propagated to AST
- Parser does NOT validate semantic rules (forward refs, duplicates, directive locations)
- `MixedDocument` preserves definition order for formatters
