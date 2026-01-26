# Test Coverage Gap Analysis & Implementation Plan

## Overview
Analysis of `libgraphql-parser` test coverage identified meaningful gaps requiring new tests. Focus: verify spec-compliant behavior, not coverage numbers.

**Current Coverage:** 62.29% lines (10,486 total, 3,954 uncovered)

---

## Part 1: Token & Lexer Tests (Priority: High)

### 1.1 GraphQLTokenKind Public API Tests
**File:** `graphql_token_kind.rs` (59.30% → target 85%)
**Test Location:** New file `crates/libgraphql-parser/src/token/tests/graphql_token_kind_tests.rs`

| Test | Lines | Description | Spec Reference |
|------|-------|-------------|----------------|
| `is_punctuator_returns_true_for_punctuators` | 198-225 | Verify all 14 punctuators return true | [Punctuators](https://spec.graphql.org/September2025/#Punctuator) |
| `is_punctuator_returns_false_for_non_punctuators` | 198-225 | Keywords, values, names return false | — |
| `as_punctuator_str_returns_correct_strings` | 228-255 | Each punctuator → correct string | — |
| `is_value_returns_true_for_value_tokens` | 259-286 | IntValue, FloatValue, StringValue, True, False, Null | [Values](https://spec.graphql.org/September2025/#Value) |
| `is_error_returns_true_only_for_error_kind` | 289-291 | Error kind detection | — |
| `parse_int_value_valid_integers` | 297-302 | Parse valid i64 integers | [Int Value](https://spec.graphql.org/September2025/#sec-Int-Value) |
| `parse_int_value_overflow_returns_error` | 297-302 | Overflow produces ParseIntError | — |
| `parse_float_value_valid_floats` | 308-313 | Parse valid f64 floats | [Float Value](https://spec.graphql.org/September2025/#sec-Float-Value) |
| `parse_string_value_basic` | 326-331 | Parse simple strings | [String Value](https://spec.graphql.org/September2025/#StringValue) |

### 1.2 String Parsing Error Tests
**File:** `graphql_token_kind.rs` (lines 345-498)

| Test | Lines | Description | Spec Reference |
|------|-------|-------------|----------------|
| `parse_string_invalid_escape_sequence` | 370-378 | `\x`, `\!` → InvalidEscapeSequence | [EscapedCharacter](https://spec.graphql.org/September2025/#EscapedCharacter) |
| `parse_string_trailing_backslash` | 375-378 | Unterminated escape | — |
| `parse_unicode_escape_empty_braces` | 413-416 | `\u{}` → error | [EscapedUnicode](https://spec.graphql.org/September2025/#EscapedUnicode) |
| `parse_unicode_escape_invalid_hex` | 401-404 | `\u{XYZ}` → error | — |
| `parse_unicode_escape_out_of_range` | 443-444 | `\u{110000}` → error (>U+10FFFF) | — |
| `parse_block_string_indentation_edge_case` | 483-484 | Line shorter than common indent | [Block Strings](https://spec.graphql.org/September2025/#sec-String-Value) |

### 1.3 GraphQLToken Constructor Test
**File:** `graphql_token.rs` (0% → target 100%)

| Test | Lines | Description |
|------|-------|-------------|
| `graphql_token_new_creates_empty_trivia` | 39-45 | Verify `new()` creates token with empty trivia |

---

## Part 2: Parser Error Handling Tests (Priority: High)

### 2.1 Value Parsing Error Tests
**File:** `graphql_parser.rs` (lines 763-965)
**Test Location:** Extend `crates/libgraphql-parser/src/tests/graphql_parser_tests.rs`

| Test | Lines | Description | Spec Reference |
|------|-------|-------------|----------------|
| `value_int_overflow_i32_error` | 804-815 | `99999999999999999` → overflow error | [Int Value](https://spec.graphql.org/September2025/#sec-Int-Value) |
| `value_float_infinity_error` | 823-837 | Very large float → non-finite error | [Float Value](https://spec.graphql.org/September2025/#sec-Float-Value) |
| `value_variable_in_const_context_error` | 943-965 | `$var` in default value → error | [All Variable Usages](https://spec.graphql.org/September2025/#sec-All-Variable-Usages-Defined) |

### 2.2 Unclosed Delimiter Error Tests
**File:** `graphql_parser.rs` (lines 983-1109, 2128-2274)

| Test | Lines | Description | Spec Reference |
|------|-------|-------------|----------------|
| `list_value_unclosed_bracket_error` | 983-996 | `[1, 2` → unclosed `[` | — |
| `object_value_unclosed_brace_error` | 1029-1048 | `{a: 1` → unclosed `{` | — |
| `object_value_missing_colon_error` | 1074-1080 | `{field 1}` → missing `:` | — |
| `field_definition_unclosed_brace` | 2128-2129 | `type T { f: String` → unclosed | [ObjectTypeDefinition](https://spec.graphql.org/September2025/#ObjectTypeDefinition) |
| `input_object_unclosed_brace` | 2214-2216 | `input I { f: String` → unclosed | [InputObjectTypeDefinition](https://spec.graphql.org/September2025/#InputObjectTypeDefinition) |
| `enum_definition_unclosed_brace` | 2273-2274 | `enum E { A` → unclosed | [EnumTypeDefinition](https://spec.graphql.org/September2025/#EnumTypeDefinition) |

### 2.3 Reserved Name Validation Tests
**File:** `graphql_parser.rs` (lines 2292-2306)

| Test | Lines | Description | Spec Reference |
|------|-------|-------------|----------------|
| `enum_value_true_reserved_error` | 2292-2306 | `enum E { true }` → reserved | [Enum Value Uniqueness](https://spec.graphql.org/September2025/#sec-Enum-Value-Uniqueness) |
| `enum_value_false_reserved_error` | 2292-2306 | `enum E { false }` → reserved | — |
| `enum_value_null_reserved_error` | 2292-2306 | `enum E { null }` → reserved | — |

### 2.4 Directive Location Error Tests
**File:** `graphql_parser.rs` (lines 2353-2469)

| Test | Lines | Description | Spec Reference |
|------|-------|-------------|----------------|
| `directive_unknown_location_error` | 2353-2387 | `directive @d on UNKNOWN` → error | [DirectiveLocation](https://spec.graphql.org/September2025/#DirectiveLocation) |
| `directive_location_typo_suggestion` | 2428-2469 | `directive @d on FILD` → suggests FIELD | — |

### 2.5 Document Type Enforcement Tests
**File:** `graphql_parser.rs` (lines 3033-3127)

| Test | Lines | Description | Spec Reference |
|------|-------|-------------|----------------|
| `executable_rejects_type_with_description` | 3048-3088 | `"desc" type T {}` in executable → error | [Executable Documents](https://spec.graphql.org/September2025/#sec-Executable-Documents) |

### 2.6 Schema Extension (Unsupported) Test
**File:** `graphql_parser.rs` (lines 2528-2596)

| Test | Lines | Description | Spec Reference |
|------|-------|-------------|----------------|
| `schema_extension_unsupported_error` | 2528-2596 | `extend schema {}` → unsupported error | [Schema Extension](https://spec.graphql.org/September2025/#sec-Schema-Extension) |

---

## Part 3: ParseResult & Error Infrastructure Tests (Priority: Medium)

### 3.1 ParseResult State Tests
**File:** `parse_result.rs` (34.62% → target 90%)
**Test Location:** New file `crates/libgraphql-parser/src/tests/parse_result_tests.rs`

| Test | Lines | Description |
|------|-------|-------------|
| `parse_result_err_creates_failed_state` | 93-95 | `err()` creates result with no AST |
| `parse_result_valid_ast_returns_none_when_errors` | 116-122 | Recovered parse → `valid_ast()` returns None |
| `parse_result_ast_returns_some_when_recovered` | 133-135 | Recovered parse → `ast()` returns Some |
| `parse_result_into_valid_ast_consumes` | 140-146 | Ownership transfer for valid parse |
| `parse_result_into_ast_consumes` | 151-153 | Ownership transfer (any case) |
| `parse_result_from_conversion_ok` | 185-194 | `Ok` state → `Result::Ok` |
| `parse_result_from_conversion_recovered` | 185-194 | Recovered → `Result::Err` with errors |
| `parse_result_from_conversion_err` | 185-194 | Failed → `Result::Err` with errors |
| `parse_result_format_errors` | 171-177 | Aggregates and formats multiple errors |

### 3.2 GraphQLParseError Construction Tests
**File:** `graphql_parse_error.rs` (11.52% → target 60%)
**Test Location:** Extend `crates/libgraphql-parser/src/tests/graphql_parse_error.rs`

| Test | Lines | Description |
|------|-------|-------------|
| `parse_error_with_notes_constructor` | 59-71 | `with_notes()` creates error with pre-populated notes |
| `parse_error_from_lexer_error` | 78-89 | Converts lexer error to parse error |
| `parse_error_add_note` | 112-114 | `add_note()` appends general note |
| `parse_error_add_help` | 123-125 | `add_help()` appends help note |
| `parse_error_add_help_with_span` | 128-131 | `add_help_with_span()` appends help note with location |

### 3.3 Error Display Formatting Tests (Lower Priority)
**File:** `graphql_parse_error.rs` (lines 155-297)

| Test | Lines | Description |
|------|-------|-------------|
| `parse_error_format_oneline` | 207-218 | Single-line format: "file:line:col: message" |
| `parse_error_format_detailed_with_source` | 155-199 | Multi-line with source snippet |
| `parse_error_format_detailed_with_notes` | 266-297 | Renders notes with different kinds |

---

## Part 4: Error Recovery Tests (Priority: Medium)

### 4.1 Recovery Point Detection Tests
**File:** `graphql_parser.rs` (lines 243-391)

| Test | Lines | Description | Spec Reference |
|------|-------|-------------|----------------|
| `recovery_skips_to_description_before_type` | 271-293 | `broken "desc" type T {}` → recovers at description | — |
| `recovery_distinguishes_keyword_from_field_name` | 313-391 | `type: String` not recovery point | — |

### 4.2 Lexer Error Integration Test
**File:** `str_to_graphql_token_source.rs` (lines 780-862)

| Test | Lines | Description |
|------|-------|-------------|
| `unterminated_string_error_with_location` | 780-793 | EOF before `"` → error with span |
| `unescaped_crlf_in_string_error` | 795-813 | `"hello\r\nworld"` → error |
| `unterminated_block_string_error` | 849-862 | EOF before `"""` → error |

---

## Implementation Guidelines

1. **Follow CLAUDE.md conventions:**
   - One import per line, alphabetically sorted
   - Match expressions end with commas
   - Tests in `tests/` submodule with `#[cfg(test)]`
   - 80-column line limit

2. **Test documentation requirements:**
   - Each test must include English description of what it verifies
   - Include GraphQL spec link where applicable
   - Add "Written by Claude Code, reviewed by a human." comment

3. **Spec verification process:**
   - Before implementing test, verify behavior against spec
   - If test fails, research spec to determine if implementation or test is wrong
   - Document any discovered bugs as `#[ignore]` with explanation

4. **Test file organization:**
   - `token/tests/graphql_token_kind_tests.rs` (NEW)
   - `token/tests/graphql_token_tests.rs` (NEW)
   - `tests/parse_result_tests.rs` (NEW)
   - `tests/graphql_parser_tests.rs` (EXTEND)
   - `tests/graphql_parse_error.rs` (EXTEND)

---

## Verification Plan

1. Run `cargo test --package libgraphql-parser` after each test group
2. Run `./scripts/generate-test-coverage-report.sh` after all tests
3. Verify coverage improved in target areas
4. Check no regressions in existing tests

---

## Excluded from Plan (Coverage-only, no meaningful verification)

- Unicode character name lookup table (1000+ lines of static data)
- Simple accessor methods (`message()`, `span()`, `kind()`, `notes()`)
- Serde helpers in `ast.rs` (internal serialization plumbing)
- Dead code / unreachable branches
- Display trait implementations (unless testing user-facing output)

---

## Estimated Scope

| Category | New Tests | Lines Covered |
|----------|-----------|---------------|
| Token/Lexer | ~15 | ~90 |
| Parser Errors | ~20 | ~200 |
| ParseResult | ~9 | ~35 |
| Error Infrastructure | ~8 | ~60 |
| Error Recovery | ~5 | ~80 |
| **Total** | **~57** | **~465** |

Expected coverage improvement: 62% → ~70%
