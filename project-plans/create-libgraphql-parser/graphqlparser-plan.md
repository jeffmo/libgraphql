# GraphQLParser Implementation Plan

## Overview

Implement `GraphQLParser<'src, S: GraphQLTokenSource<'src>>` - a recursive descent parser generic over token sources, enabling unified parsing from both `StrGraphQLTokenSource` and `RustMacroGraphQLTokenSource`.

## Current State

### Completed Infrastructure
- `GraphQLTokenKind<'src>`, `GraphQLToken<'src>`, `GraphQLTriviaToken<'src>` - with `Cow` for zero-copy
- `GraphQLTokenSource<'src>` trait - marker trait for token iterators
- `GraphQLTokenStream<'src, S>` - lookahead buffering, peek/consume
- `StrGraphQLTokenSource<'src>` - complete (~1130 lines), 60 tests
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

## Implementation Steps

### Step 1: Core Parser Infrastructure
**File:** `/crates/libgraphql-parser/src/graphql_parser.rs`

Create:
```rust
pub struct GraphQLParser<'src, S: GraphQLTokenSource<'src>> {
    token_stream: GraphQLTokenStream<'src, S>,
    errors: Vec<GraphQLParseError>,
    delimiter_stack: Vec<OpenDelimiter>,
}

struct OpenDelimiter {
    kind: char,
    span: GraphQLSourceSpan,
    context: DelimiterContext,
}

enum DelimiterContext {
    SchemaDefinition, TypeDefinition, EnumDefinition, InputObjectDefinition,
    SelectionSet, Arguments, VariableDefinitions, ListType, ListValue, ObjectValue,
    DirectiveLocations,
}
```

**Token access (delegate to `GraphQLTokenStream`):**
- `self.token_stream.peek()` - lookahead without consuming
- `self.token_stream.consume()` - advance to next token

**Parser helpers (on `GraphQLParser` - need `self.errors` and delimiter context):**
- `new(token_source: S) -> Self`
- `expect(&mut self, kind: &GraphQLTokenKind) -> Result<GraphQLToken, ()>`
- `expect_name(&mut self) -> Result<String, ()>` - **See important note below**
- `expect_keyword(&mut self, kw: &str) -> Result<(), ()>`
- `at_keyword(&mut self, kw: &str) -> bool`
- `is_at_end(&mut self) -> bool`
- `record_error(&mut self, err: GraphQLParseError)`
- `push_delimiter(&mut self, kind: char, span: GraphQLSourceSpan, ctx: DelimiterContext)`
- `pop_delimiter(&mut self) -> Option<OpenDelimiter>`
- `recover_to_next_definition(&mut self)`

#### Critical: `expect_name()` and `true`/`false`/`null` tokens

**Problem:** The lexer tokenizes `true`, `false`, `null` as distinct `GraphQLTokenKind` variants (`True`, `False`, `Null`), NOT as `Name`. However, per the GraphQL spec, these ARE valid names in most contexts (they match the `Name` regex `/[_A-Za-z][_0-9A-Za-z]*/`).

**Solution:** `expect_name()` must accept ALL of these as valid names:
```rust
fn expect_name(&mut self) -> Result<String, ()> {
    match self.peek()?.kind {
        GraphQLTokenKind::Name(s) => { self.consume(); Ok(s.into_owned()) }
        GraphQLTokenKind::True => { self.consume(); Ok("true".to_string()) }
        GraphQLTokenKind::False => { self.consume(); Ok("false".to_string()) }
        GraphQLTokenKind::Null => { self.consume(); Ok("null".to_string()) }
        _ => { self.record_error(...); Err(()) }
    }
}
```

**Contexts where `true`/`false`/`null` ARE valid names:**
- Type definition names: `type true { }`
- Field names: `type Foo { true: String }`, `{ true }` in selection sets
- Alias names: `{ true: actualField }`
- Argument names: `field(true: String!)`, `field(true: 123)`
- Operation names: `query true { }`
- Variable names: `query($true: String)` (after `$`)
- Directive names: `directive @true on FIELD`
- Union member names: `union U = true | false`
- Interface names: `type Foo implements true`
- Input field names: `input I { true: String }`
- Object value field names: `{ true: 123 }` in literals

**Contexts where names ARE reserved (produce AST + error):**
- Enum values: `true`, `false`, `null` reserved → `ReservedName { context: EnumValue }`
- Fragment names: `on` reserved → `ReservedName { context: FragmentName }`

Callers that need to reject reserved names should call `expect_name()` first, then check the returned name and record an error if reserved in that context.

### Step 2: ParseResult Type
**File:** `/crates/libgraphql-parser/src/parse_result.rs`

```rust
pub struct ParseResult<T> {
    pub ast: Option<T>,
    pub errors: Vec<GraphQLParseError>,
}

impl<T> ParseResult<T> {
    pub fn is_ok(&self) -> bool;
    pub fn has_errors(&self) -> bool;
    pub fn ast(&self) -> Option<&T>;
    pub fn take_ast(&mut self) -> Option<T>;
    pub fn format_errors(&self, source: Option<&str>) -> String;
}

impl<T> From<ParseResult<T>> for Result<T, Vec<GraphQLParseError>>
```

### Step 3: Value Parsing
Methods:
- `parse_value(&mut self, const_only: bool) -> Result<Value, ()>`
- `parse_list_value(&mut self, const_only: bool) -> Result<Value, ()>`
- `parse_object_value(&mut self, const_only: bool) -> Result<Value, ()>`

Handle: Variable, IntValue, FloatValue, StringValue, True, False, Null, EnumValue, ListValue, ObjectValue

`const_only: bool` rejects variables in default value positions.

**Note on object value field names:** Object literals like `{ true: 123, false: "abc" }` have field names that may be `true`/`false`/`null` tokens. Use `expect_name()` which handles these.

### Step 4: Type Annotation Parsing
Parses the type syntax used in field definitions, parameters, and variables.

Methods:
- `parse_type_annotation(&mut self) -> Result<Type, ()>` - Full type annotation (`String!`, `[Int]!`)
- `parse_named_type_annotation(&mut self) -> Result<NamedType, ()>` - Just the name (`String`, `User`)

Handle:
- **NamedTypeAnnotation**: `TypeName` (e.g., `String`, `User`)
- **ListTypeAnnotation**: `[TypeAnnotation]` (e.g., `[String]`, `[[Int]]`)
- **Non-null modifier**: `TypeAnnotation!` (e.g., `String!`, `[Int]!`)

**Note on special type names:** Type names may be `true`, `false`, or `null` (e.g., `field: true!`). Use `expect_name()` which handles these token kinds.

### Step 5: Directive Annotation Parsing
Parses directive usages/applications (not definitions). Directive definitions are parsed in Step 9.

Methods:
- `parse_directive_annotations(&mut self) -> Result<Vec<Directive>, ()>` - Zero or more annotations
- `parse_directive_annotation(&mut self) -> Result<Directive, ()>` - Single annotation

Pattern: `@DirectiveName Arguments?` (e.g., `@deprecated(reason: "use newField")`)

**Note on directive names and arguments:** Directive names (e.g., `@true`) and argument names (e.g., `@foo(true: 123)`) may be `true`/`false`/`null` tokens. Use `expect_name()` which handles these.

### Step 6: Selection Set Parsing
Methods:
- `parse_selection_set(&mut self) -> Result<SelectionSet, ()>`
- `parse_selection(&mut self) -> Result<Selection, ()>`
- `parse_field(&mut self) -> Result<Field, ()>`
- `parse_arguments(&mut self) -> Result<Vec<Argument>, ()>`
- `parse_fragment_spread(&mut self) -> Result<FragmentSpread, ()>`
- `parse_inline_fragment(&mut self) -> Result<InlineFragment, ()>`

Track `{` with `delimiter_stack` for error messages.

**Note on field/alias names:** Field names and alias names may be `true`/`false`/`null` tokens (e.g., `{ true }`, `{ myAlias: true }`). Use `expect_name()` which handles these.

### Step 7: Operation Parsing
Methods:
- `parse_operation_definition(&mut self) -> Result<OperationDefinition, ()>`
- `parse_variable_definitions(&mut self) -> Result<Vec<VariableDefinition>, ()>`
- `parse_variable_definition(&mut self) -> Result<VariableDefinition, ()>`

Handle: query/mutation/subscription with optional name, variables, directives, selection set. Also shorthand query (`{ ... }`).

**Note on operation/variable names:** Operation names (e.g., `query true { }`) and variable names after `$` (e.g., `$true`) may be `true`/`false`/`null` tokens. Use `expect_name()` which handles these.

### Step 8: Fragment Parsing
Methods:
- `parse_fragment_definition(&mut self) -> Result<FragmentDefinition, ()>`
- `parse_type_condition(&mut self) -> Result<TypeCondition, ()>`

**Error recovery for reserved name `on`:** If a fragment is named `on`, still produce a `crate::ast::operation::FragmentDefinition { name: "on", .. }` AST node, but record a `ReservedName { name: "on", context: FragmentName }` error. The invalid AST is indicated by the presence of an error in `ParseResult.errors`. This allows downstream tooling to see the full structure even when invalid.

### Step 9: Type Definition Parsing
Parses schema type definitions (object types, interfaces, unions, enums, scalars, input objects, and directive definitions).

Methods:
- `parse_description(&mut self) -> Option<String>` - StringValue before definition
- `parse_schema_definition(&mut self) -> Result<SchemaDefinition, ()>`
- `parse_scalar_type_definition(&mut self) -> Result<ScalarTypeDefinition, ()>`
- `parse_object_type_definition(&mut self) -> Result<ObjectTypeDefinition, ()>`
- `parse_interface_type_definition(&mut self) -> Result<InterfaceTypeDefinition, ()>`
- `parse_union_type_definition(&mut self) -> Result<UnionTypeDefinition, ()>`
- `parse_enum_type_definition(&mut self) -> Result<EnumTypeDefinition, ()>`
- `parse_input_object_type_definition(&mut self) -> Result<InputObjectTypeDefinition, ()>`
- `parse_directive_definition(&mut self) -> Result<DirectiveDefinition, ()>` - Parses `directive @name(...) on LOCATIONS`
- `parse_implements_interfaces(&mut self) -> Result<Vec<NamedType>, ()>`
- `parse_fields_definition(&mut self) -> Result<Vec<FieldDefinition>, ()>`

**Note on names in type definitions:** Many names may be `true`/`false`/`null` tokens:
- Type definition names: `type true { }`, `interface false { }`, `scalar null`
- Field/argument/input field names: `type Foo { true: String }`, `field(false: Int)`
- Union member names: `union U = true | false`
- Interface names in implements: `type Foo implements true`
- Directive definition names: `directive @true on FIELD`

Use `expect_name()` which handles these token kinds.

**Directive Definition** parsing (distinct from directive annotation parsing in Step 5):
- Pattern: `Description? "directive" "@" Name ArgumentsDefinition? "repeatable"? "on" DirectiveLocations`
- Validates directive location names; suggests closest match on typo (edit-distance)

**Error recovery for reserved enum values:** If an enum value is named `true`, `false`, or `null`, still produce a `crate::ast::schema::EnumValue { name: "true"|"false"|"null", .. }` AST node, but record a `ReservedName { name, context: EnumValue }` error. The invalid AST is indicated by the presence of an error in `ParseResult.errors`. This allows the parser to continue and report multiple errors.

### Step 10: Type Extension Parsing
Methods:
- `parse_type_extension(&mut self) -> Result<TypeExtension, ()>`
- `parse_schema_extension(&mut self) -> ...`
- `parse_scalar_type_extension(&mut self) -> ...`
- `parse_object_type_extension(&mut self) -> ...`
- (etc. for all extension types)

Pattern: `extend` keyword followed by type definition (minus description).

### Step 11: Document Parsing
Public API:
```rust
impl<'src, S: GraphQLTokenSource<'src>> GraphQLParser<'src, S> {
    pub fn parse_schema_document(mut self) -> ParseResult<schema::Document>;
    pub fn parse_executable_document(mut self) -> ParseResult<operation::Document>;
    pub fn parse_mixed_document(mut self) -> ParseResult<MixedDocument>;
}
```

Internal:
- `parse_definition(&mut self, doc_kind: DocumentKind) -> Result<Definition, ()>`
- Error recovery: on error, call `recover_to_next_definition()`, skip to next definition keyword

Definition keywords: `type`, `interface`, `union`, `enum`, `scalar`, `input`, `directive`, `schema`, `extend`, `query`, `mutation`, `subscription`, `fragment`, `{`

### Step 12: Wire Up Exports
**File:** `/crates/libgraphql-parser/src/lib.rs`

Add:
```rust
mod graphql_parser;
mod parse_result;

pub use graphql_parser::GraphQLParser;
pub use parse_result::ParseResult;
```

## Error Handling Strategy

Per `graphql-parse-error.md`, use the existing `GraphQLParseError` infrastructure:

### Error Structure
```rust
GraphQLParseError {
    message: String,           // Human-readable primary message
    span: GraphQLSourceSpan,   // Primary error location
    kind: GraphQLParseErrorKind, // Categorized for programmatic handling
    notes: GraphQLErrorNotes,  // SmallVec<[GraphQLErrorNote; 2]>
}
```

### Error Kinds (GraphQLParseErrorKind)
| Kind | Usage |
|------|-------|
| `LexerError` | Wrap `GraphQLTokenKind::Error` tokens |
| `UnexpectedToken { expected, found }` | Expected specific token(s) but got something else |
| `UnexpectedEof` | Document ended before complete construct (no open delimiter) |
| `UnclosedDelimiter` | Open `{`, `[`, `(` without matching close |
| `MismatchedDelimiter` | Wrong close delimiter (e.g., `[` closed with `)`) |
| `InvalidValue` | Value parse errors (int overflow, bad string escape) |
| `ReservedName { name, context }` | Reserved name in wrong context (`on` as fragment name) |
| `WrongDocumentKind { found, document_kind }` | Definition not allowed in document type |
| `InvalidEmptyConstruct { construct }` | Empty `{}`, `()` where content required |
| `InvalidSyntax` | Catch-all for other syntax errors |

### Error Notes (GraphQLErrorNote)
| Kind | Prefix | Example |
|------|--------|---------|
| `General` | `= note:` | "Opening `{` here" |
| `Help` | `= help:` | "Did you mean: `userName: String`?" |
| `Spec` | `= spec:` | "https://spec.graphql.org/September2025/#..." |

### Recovery Strategy
- Skip tokens until next definition keyword: `type`, `interface`, `union`, `enum`, `scalar`, `input`, `directive`, `schema`, `extend`, `query`, `mutation`, `subscription`, `fragment`, `{`
- Continue parsing to collect multiple errors in one pass

## Files to Create/Modify

| File | Action |
|------|--------|
| `src/graphql_parser.rs` | Create - main parser |
| `src/parse_result.rs` | Create - result type |
| `src/lib.rs` | Modify - add exports |
| `src/tests/graphql_parser_tests.rs` | Create - unit tests |

## Verification

1. `cargo check --package libgraphql-parser` - compilation
2. `cargo clippy --package libgraphql-parser --tests` - linting
3. `cargo test --package libgraphql-parser` - all tests pass
4. Manual test: parse simple schema and query documents
5. Test error recovery: document with multiple errors reports all

## Test Strategy

- Unit tests for each parse method
- Integration tests for complete documents
- Error message tests (verify helpful messages)
- Test both token sources work identically (StrGraphQLTokenSource, RustMacroGraphQLTokenSource)

### Special Name Tests (required)
Since `true`, `false`, `null` are tokenized as distinct `GraphQLTokenKind` variants (not `Name`), we need comprehensive tests to ensure `expect_name()` handles them correctly in all contexts.

**Valid uses (no error expected):**
- Type definition names: `type true { id: ID }`, `interface false { }`, `scalar null`
- Field names: `type Foo { true: String }`, `{ true }` in selection sets
- Alias names: `{ true: actualField }`
- Argument names: `field(true: String!)`, `query { foo(true: 123) }`
- Operation names: `query true { id }`, `mutation false { }`
- Variable names: `query($true: String) { }`, `$false`, `$null`
- Directive names: `directive @true on FIELD`, `@false`, `@null`
- Union member names: `union U = true | false | null`
- Interface names: `type Foo implements true`
- Input field names: `input I { true: String }`
- Object value field names: `{ foo(arg: { true: 123 }) }`

**Invalid uses (produce AST + error):**
- Enum values: `enum E { true false null }` - three `ReservedName` errors
- Fragment name `on`: `fragment on on Type` - one `ReservedName` error

**Test pattern:** For each context, verify:
1. Parsing succeeds (returns AST)
2. For valid uses: `ParseResult.errors` is empty
3. For invalid uses: AST is produced AND appropriate error is in `ParseResult.errors`

## Notes

- AST uses `graphql_parser` crate types temporarily; custom AST is future work
- Trivia stays on tokens, not propagated to AST
- Parser does NOT validate semantic rules (forward refs, duplicates, directive locations)
- `MixedDocument` preserves definition order for formatters
