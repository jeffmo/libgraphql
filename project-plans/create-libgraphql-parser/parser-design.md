# GraphQL Parser Design Document

## Overview

This document outlines the design for a spec-compliant GraphQL parser in
`libgraphql-parser`. The parser is parameterized over a `GraphQLTokenSource`
type, enabling both proc-macro and string-based parsing from a unified
implementation.

---

## Part 1: Type Renames and Relocations

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

## Part 2: Parse Error Design

### `GraphQLParseError`

```rust
/// A single parse error with span information.
#[derive(Debug, Clone)]
pub struct GraphQLParseError {
    /// Human-readable error message.
    message: String,
    /// Spans highlighting the error location(s).
    spans: Vec<GraphQLSourceSpan>,
    /// Categorized error kind for programmatic handling.
    kind: GraphQLParseErrorKind,
}

impl GraphQLParseError {
    /// Creates a new parse error.
    pub fn new(
        message: String,
        spans: Vec<GraphQLSourceSpan>,
        kind: GraphQLParseErrorKind,
    ) -> Self { ... }

    pub fn message(&self) -> &str { ... }
    pub fn spans(&self) -> &[GraphQLSourceSpan] { ... }
    pub fn primary_span(&self) -> Option<&GraphQLSourceSpan> { ... }
    pub fn kind(&self) -> &GraphQLParseErrorKind { ... }
}
```

### `GraphQLParseErrorKind`

```rust
/// Categorizes parse errors for programmatic handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphQLParseErrorKind {
    /// Expected specific tokens but found something else.
    UnexpectedToken {
        expected: Vec<String>,
        found: String,
    },

    /// Unexpected end of input.
    UnexpectedEof {
        expected: Vec<String>,
    },

    /// General syntax error.
    InvalidSyntax,

    /// Directive used in invalid location.
    InvalidDirectiveLocation,

    /// Invalid value (wraps cooking errors).
    InvalidValue(CookValueError),

    /// Unclosed delimiter.
    UnclosedDelimiter {
        delimiter: String,
        opening_span: Option<GraphQLSourceSpan>,
    },

    /// Mismatched delimiter.
    MismatchedDelimiter {
        expected: String,
        found: String,
    },
}
```

### `CookValueError`

This enum unifies value-cooking errors:

```rust
/// Errors that occur when processing ("cooking") literal values.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CookValueError {
    #[error("Invalid string: {0}")]
    String(#[from] CookGraphQLStringError),

    #[error("Invalid integer: {0}")]
    Int(String),  // e.g., "overflow", "invalid format"

    #[error("Invalid float: {0}")]
    Float(String),
}
```

**Removed from `GraphQLParseErrorKind`:**
- `DuplicateDefinition` - This is a validation error, not a parse error.
  Duplicate detection belongs in schema/document validators.

---

## Part 3: Parser Architecture

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

/// Result of parsing, containing both AST (if any) and errors.
pub struct ParseResult<T> {
    pub ast: Option<T>,
    pub errors: Vec<GraphQLParseError>,
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

### Step 1: Rename and Relocate Types
1. Rename `GraphQLTokenSpan` → `GraphQLSourceSpan`
2. Move to crate root
3. Update all references
4. Verify tests pass

### Step 2: Create Parse Error Types
1. Create `graphql_parse_error.rs` with new design
2. Create `CookValueError` type
3. Add tests

### Step 3: Create Parser Skeleton
1. Create `graphql_parser.rs` with generic structure
2. Implement `parse_schema_document()` stub
3. Implement `parse_executable_document()` stub
4. Implement `parse_mixed_document()` stub

### Step 4: Implement Value Parsing
1. Implement `parse_value()` and variants
2. Handle all value types per spec
3. Add comprehensive tests

### Step 5: Implement Type Parsing
1. Implement `parse_type()` and variants
2. Handle list, non-null wrapping
3. Add tests

### Step 6: Implement Directive Parsing
1. Implement `parse_directives()` and `parse_directive()`
2. Add tests

### Step 7: Implement Selection Set Parsing
1. Implement `parse_selection_set()` and related methods
2. Handle fields, fragment spreads, inline fragments
3. Add tests

### Step 8: Implement Operation Parsing
1. Implement `parse_operation_definition()`
2. Handle variable definitions
3. Add tests

### Step 9: Implement Fragment Parsing
1. Implement `parse_fragment_definition()`
2. Add tests

### Step 10: Implement Type Definition Parsing
1. Implement all type definition methods
2. Handle descriptions, directives, implements
3. Add tests for each type

### Step 11: Implement Type Extension Parsing
1. Implement all extension methods
2. Add tests

### Step 12: Complete Document Parsing
1. Wire up all methods in `parse_*_document()`
2. Implement error recovery
3. Add integration tests

### Step 13: Port and Vendor Tests
1. Port tests from graphql-js (after license verification)
2. Port tests from graphql-parser (after license verification)
3. Add differential testing against graphql_parser crate

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

## Open Questions

1. **AST Types:** Continue using `graphql_parser::schema` and
   `graphql_parser::query` AST types, or define new ones?
   - Current plan: Continue using for now, custom AST is future work

2. **Trivia in AST:** Should AST nodes carry their trivia?
   - Current plan: Trivia stays on tokens, not propagated to AST

3. **Span in AST:** Should AST nodes have `GraphQLSourceSpan`?
   - Current plan: Use existing `Pos` from `graphql_parser` for now
