# Exhaustive Unit Test Suite for GraphQL Parser and Lexer

This document enumerates an exhaustive suite of unit tests for `libgraphql-parser`'s `GraphQLParser` and `StrGraphQLTokenSource` components according to the project's testing guidelines.

**ABSOLUTELY CRITICAL**: For each unit test, ensure that the test is
corroborated by the GraphQL specification
(https://spec.graphql.org/September2025) and -- if a given test fails -- decide
if the implementation is wrong (via extensive specification research) or the
test code is wrong before proceeding with a fix for the failure.

**Note:** Tests marked ✅ already exist and should NOT be re-implemented.

---

## Part 1: StrGraphQLTokenSource (Lexer) Tests

### 1.1 Punctuators
**Spec:** https://spec.graphql.org/September2025/#sec-Punctuators

| Test | Summary | Status |
|------|---------|--------|
| ✅ `test_punctuators` | All 12 single-char punctuators | EXISTS |
| ✅ `test_ellipsis` | Valid `...` token | EXISTS |
| `punctuators_adjacent_without_whitespace` | `{}[]()` lexes as 6 separate tokens | NEW |
| `ellipsis_with_surrounding_whitespace` | `  ...  ` correctly lexes | NEW |
| ✅ `test_dot_pattern_*` (8 tests) | Various dot error patterns | EXISTS |
| `ellipsis_followed_by_dot` | `....` → `Ellipsis` + dot error | NEW |

### 1.2 Names
**Spec:** https://spec.graphql.org/September2025/#Name

| Test | Summary | Status |
|------|---------|--------|
| ✅ `test_names` | Basic names, underscore, double underscore, numbers | EXISTS |
| `name_uppercase` | `SCREAMING_CASE` lexes as Name | NEW |
| `name_mixed_case` | `camelCase`, `PascalCase` lex as Names | NEW |
| `name_cannot_start_with_digit` | `2fast` → IntValue then Name | NEW |
| `name_single_underscore` | `_` alone is valid name | NEW |
| `name_very_long` | 10000-char name stress test | NEW |
| `name_unicode_rejected` | `café`, `名前` produce errors | NEW |

### 1.3 Keywords (true/false/null)
**Spec:** https://spec.graphql.org/September2025/#sec-Boolean-Value, #sec-Null-Value

| Test | Summary | Status |
|------|---------|--------|
| ✅ `test_keywords` | `true`, `false`, `null` as tokens | EXISTS |
| ✅ `test_keywords_case_sensitive` | `True`, `FALSE`, `Null` as names | EXISTS |
| `keyword_case_sensitive_NULL` | `NULL` lexes as Name | NEW |
| `keyword_prefix_trueish` | `trueValue` lexes as single Name | NEW |
| `keyword_prefix_falsely` | `falsely` lexes as single Name | NEW |
| `keyword_prefix_nullable` | `nullable` lexes as single Name | NEW |

### 1.4 Integer Values
**Spec:** https://spec.graphql.org/September2025/#sec-Int-Value

| Test | Summary | Status |
|------|---------|--------|
| ✅ `test_int_values` | `0`, `123`, `-456` | EXISTS |
| ✅ `test_number_leading_zeros` | `007` error | EXISTS |
| ✅ `test_number_lone_minus` | `-` alone error | EXISTS |
| `int_negative_zero` | `-0` is valid IntValue | NEW |
| `int_negative_leading_zeros_error` | `-007` produces error | NEW |
| `int_max_i32` | `2147483647` parses OK | NEW |
| `int_min_i32` | `-2147483648` parses OK | NEW |
| `int_overflow_i32` | `2147483648` overflow error | NEW |
| `int_underflow_i32` | `-2147483649` underflow error | NEW |
| `int_i64_max` | Large i64 values handling | NEW |
| `int_followed_by_name` | `123abc` → IntValue + Name | NEW |
| `int_followed_by_dot_name` | `123.abc` → error handling | NEW |

### 1.5 Float Values
**Spec:** https://spec.graphql.org/September2025/#sec-Float-Value

| Test | Summary | Status |
|------|---------|--------|
| ✅ `test_float_values` | Basic floats with decimal/exponent | EXISTS |
| ✅ `test_number_exponent_no_digits` | `1e` error | EXISTS |
| `float_exponent_uppercase` | `1E10`, `2E3` lex as FloatValue | NEW |
| `float_exponent_positive` | `1e+10` lexes as FloatValue | NEW |
| `float_zero_decimal` | `0.0` lexes as FloatValue | NEW |
| `float_leading_zero_decimal` | `0.123` is valid | NEW |
| `float_no_leading_zero_error` | `.5` produces dot error | NEW |
| `float_trailing_dot_not_float` | `5.` followed by non-digit | NEW |
| `float_double_dot_error` | `1..5` produces error | NEW |
| `float_infinity_error` | Very large floats → error | NEW |
| `float_negative_zero` | `-0.0` is valid FloatValue | NEW |
| `float_very_small` | `1e-308` near f64 min | NEW |
| `float_subnormal` | `1e-324` subnormal handling | NEW |
| `float_exponent_sign_no_digits` | `1e+` produces error | NEW |

### 1.6 String Values (Single-Line)
**Spec:** https://spec.graphql.org/September2025/#sec-String-Value

| Test | Summary | Status |
|------|---------|--------|
| ✅ `test_single_line_strings` | Basic string lexing | EXISTS |
| ✅ `test_string_unterminated` | Unterminated string error | EXISTS |
| ✅ `test_string_unescaped_newline` | Newline in string error | EXISTS |
| `string_empty` | `""` lexes as StringValue | NEW |
| `string_escape_quote` | `"say \"hello\""` | NEW |
| `string_escape_backslash` | `"path\\to\\file"` | NEW |
| `string_escape_slash` | `"a\/b"` | NEW |
| `string_escape_backspace` | `"a\bb"` | NEW |
| `string_escape_formfeed` | `"a\fb"` | NEW |
| `string_escape_newline` | `"a\nb"` | NEW |
| `string_escape_carriage_return` | `"a\rb"` | NEW |
| `string_escape_tab` | `"a\tb"` | NEW |
| `string_escape_unicode_4digit` | `"\u0041"` → `A` | NEW |
| `string_escape_unicode_bmp` | `"\u00E9"` → `é` | NEW |
| `string_escape_unicode_surrogate_pair` | `"\uD83D\uDE00"` → 😀 | NEW |
| `string_escape_invalid_unicode` | `"\uXXXX"` error | NEW |
| `string_escape_incomplete_unicode` | `"\u00"` error | NEW |
| `string_control_chars_error` | Unescaped U+0000-U+001F | NEW |

### 1.7 Block Strings
**Spec:** https://spec.graphql.org/September2025/#sec-String-Value (BlockString)

| Test | Summary | Status |
|------|---------|--------|
| ✅ `test_block_strings` | Basic block string | EXISTS |
| ✅ `test_block_string_unterminated` | Unterminated error | EXISTS |
| ✅ `test_block_string_escaped_triple_quote` | `\"""` handling | EXISTS |
| `block_string_multiline` | Block string with newlines | NEW |
| `block_string_contains_quotes` | `"""contains " quote"""` | NEW |
| `block_string_contains_double_quotes` | `"""contains "" quotes"""` | NEW |
| `block_string_common_indent_removal` | Leading whitespace trimming | NEW |
| `block_string_empty` | `""""""` valid empty | NEW |
| `block_string_just_whitespace` | Only whitespace/newlines | NEW |
| `block_string_crlf_handling` | `\r\n` line endings | NEW |

### 1.8 Comments
**Spec:** https://spec.graphql.org/September2025/#sec-Comments

| Test | Summary | Status |
|------|---------|--------|
| ✅ `test_comments_as_trivia` | Comment attached as trivia | EXISTS |
| ✅ `test_multiple_comments_as_trivia` | Multiple comments | EXISTS |
| ✅ `test_trailing_comment_on_eof` | Comment on EOF | EXISTS |
| `comment_empty` | `#` alone (empty comment) | NEW |
| `comment_contains_hash` | `# contains # hash` | NEW |
| `comment_unicode` | `# 日本語コメント` | NEW |

### 1.9 Whitespace and Ignored Tokens
**Spec:** https://spec.graphql.org/September2025/#sec-Language.Source-Text.Unicode

| Test | Summary | Status |
|------|---------|--------|
| ✅ `test_bom_ignored` | BOM at start and middle | EXISTS |
| ✅ `test_comma_as_trivia` | Comma trivia | EXISTS |
| `whitespace_tab` | Tabs between tokens | NEW |
| `multiple_commas` | `field1,,, field2` trivia | NEW |

### 1.10 Position Tracking
| Test | Summary | Status |
|------|---------|--------|
| ✅ `test_position_single_line` | Single line positions | EXISTS |
| ✅ `test_position_multiple_lines` | Multiline positions | EXISTS |
| ✅ `test_position_crlf_newline` | CRLF handling | EXISTS |
| ✅ `test_position_cr_newline` | CR handling | EXISTS |
| ✅ `test_utf16_column_*` (3 tests) | UTF-16 column tracking | EXISTS |
| `position_byte_offset` | Byte offset tracking | NEW |
| `position_with_bom` | BOM affects byte offset | NEW |
| `position_with_file_path` | `with_file_path()` in spans | NEW |

### 1.11 Error Recovery and Invalid Characters
| Test | Summary | Status |
|------|---------|--------|
| ✅ `test_invalid_character_recovery` | `^` error + continues | EXISTS |
| `invalid_char_tilde` | `~` produces error | NEW |
| `invalid_char_backtick` | `` ` `` produces error | NEW |
| `invalid_char_question` | `?` produces error | NEW |
| `invalid_char_control` | Control chars descriptive errors | NEW |
| `invalid_char_zero_width` | Zero-width chars errors | NEW |
| `invalid_char_bidi` | Bidi control chars errors | NEW |
| `multiple_errors_collected` | Multiple invalid chars reported | NEW |

---

## Part 2: GraphQLParser Tests

### 2.1 Value Parsing
**Spec:** https://spec.graphql.org/September2025/#sec-Input-Values

| Test | Summary | Edge Case? |
|------|---------|------------|
| `value_int` | Parse integer value | No |
| `value_int_negative` | Parse negative integer | No |
| `value_int_overflow` | i32 overflow produces error | Yes |
| `value_float` | Parse float value | No |
| `value_float_infinity_error` | Infinity/NaN produces error | Yes |
| `value_string` | Parse string value | No |
| `value_string_with_escapes` | Escapes correctly processed | No |
| `value_string_invalid_escape_error` | Invalid escape produces error | Yes |
| `value_boolean_true` | `true` parses to Boolean(true) | No |
| `value_boolean_false` | `false` parses to Boolean(false) | No |
| `value_null` | `null` parses to Null | No |
| `value_enum` | `ACTIVE` parses as EnumValue | No |
| `value_enum_looks_like_keyword` | Enum value `type`, `query` allowed | Yes |
| `value_list_empty` | `[]` parses as empty List | No |
| `value_list_simple` | `[1, 2, 3]` parses as List | No |
| `value_list_nested` | `[[1], [2]]` parses as nested List | Yes |
| `value_list_mixed_types` | `[1, "two", true]` parses | No |
| `value_object_empty` | `{}` parses as empty Object | No |
| `value_object_simple` | `{key: "value"}` parses as Object | No |
| `value_object_multiple_fields` | `{a: 1, b: 2}` parses | No |
| `value_object_nested` | `{outer: {inner: 1}}` parses | Yes |
| `value_variable` | `$varName` parses as Variable in allowed contexts | No |
| `value_variable_in_const_error` | Variable in default value produces error | Yes |
| `value_variable_in_directive_arg_error` | Variable in const directive arg produces error | Yes |

### 2.2 Type Annotations
**Spec:** https://spec.graphql.org/September2025/#sec-Type-References

| Test | Summary | Edge Case? |
|------|---------|------------|
| `type_named` | `String`, `User` parse as NamedType | No |
| `type_named_null_allowed` | Types like `null`, `true` can be type names | Yes |
| `type_non_null` | `String!` parses as NonNullType | No |
| `type_list` | `[String]` parses as ListType | No |
| `type_list_non_null` | `[String]!` parses correctly | No |
| `type_non_null_list` | `[String!]` parses correctly | No |
| `type_non_null_list_non_null` | `[String!]!` parses correctly | No |
| `type_deeply_nested` | `[[String]]`, `[[[Int]]]` parse | Yes |
| `type_unclosed_bracket_error` | `[String` produces error | Yes |
| `type_double_bang_error` | `String!!` produces error | Yes |

### 2.3 Directive Annotations
**Spec:** https://spec.graphql.org/September2025/#sec-Language.Directives

| Test | Summary | Edge Case? |
|------|---------|------------|
| `directive_simple` | `@deprecated` parses | No |
| `directive_with_args` | `@deprecated(reason: "old")` parses | No |
| `directive_multiple` | `@a @b @c` all parsed | No |
| `directive_arg_list` | `@dir(a: 1, b: 2)` parses | No |
| `directive_empty_args_error` | `@dir()` produces error (empty args not allowed) | Yes |
| `directive_const_only` | In schema context, variables in args produce error | Yes |
| `directive_name_keyword` | `@type`, `@query` allowed as directive names | Yes |

### 2.4 Selection Sets
**Spec:** https://spec.graphql.org/September2025/#sec-Selection-Sets

| Test | Summary | Edge Case? |
|------|---------|------------|
| `selection_set_simple` | `{ name }` parses | No |
| `selection_set_multiple_fields` | `{ name age }` parses | No |
| `selection_set_nested` | `{ user { name } }` parses | No |
| `selection_set_empty_error` | `{ }` produces error | Yes |
| `selection_set_unclosed_error` | `{ name` produces error with hint | Yes |
| `field_simple` | `name` field parses | No |
| `field_with_alias` | `userName: name` parses | No |
| `field_with_args` | `user(id: 1)` parses | No |
| `field_with_directives` | `name @include(if: true)` parses | No |
| `field_with_nested_selection` | `user { name }` parses | No |
| `field_empty_args_error` | `field()` produces error | Yes |
| `fragment_spread` | `...UserFields` parses | No |
| `fragment_spread_with_directives` | `...UserFields @include(if: true)` parses | No |
| `inline_fragment_typed` | `... on User { name }` parses | No |
| `inline_fragment_untyped` | `... { name }` parses | No |
| `inline_fragment_with_directives` | `... on User @skip(if: $flag) { name }` parses | No |

### 2.5 Operations
**Spec:** https://spec.graphql.org/September2025/#sec-Language.Operations

| Test | Summary | Edge Case? |
|------|---------|------------|
| `operation_query_named` | `query GetUser { name }` parses | No |
| `operation_query_anonymous` | `query { name }` parses | No |
| `operation_query_shorthand` | `{ name }` parses as anonymous query | No |
| `operation_mutation` | `mutation CreateUser { createUser }` parses | No |
| `operation_subscription` | `subscription OnMessage { newMessage }` parses | No |
| `operation_with_variables` | `query($id: ID!) { user(id: $id) }` parses | No |
| `operation_with_directives` | `query @cached { name }` parses | No |
| `operation_empty_vars_error` | `query() { name }` produces error | Yes |
| `operation_var_default_value` | `query($limit: Int = 10)` parses | No |
| `operation_var_default_no_variables` | Default values cannot contain variables | Yes |
| `operation_var_directives` | Variable directives parsed (spec allows) | No |
| `operation_name_is_keyword` | `query query { }` allowed (name=query) | Yes |

### 2.6 Fragments
**Spec:** https://spec.graphql.org/September2025/#sec-Language.Fragments

| Test | Summary | Edge Case? |
|------|---------|------------|
| `fragment_definition_simple` | `fragment UserFields on User { name }` parses | No |
| `fragment_with_directives` | `fragment F on User @deprecated { }` parses | No |
| `fragment_name_on_error` | `fragment on on User { }` produces error | Yes |
| `fragment_missing_type_condition` | `fragment F { name }` produces error | Yes |
| `fragment_nested_selections` | Fragment with nested selection sets | No |
| `fragment_type_condition_keyword` | `fragment F on null { }` - `null` as type name | Yes |

### 2.7 Schema Definitions
**Spec:** https://spec.graphql.org/September2025/#sec-Schema

| Test | Summary | Edge Case? |
|------|---------|------------|
| `schema_simple` | `schema { query: Query }` parses | No |
| `schema_all_operations` | `schema { query: Q mutation: M subscription: S }` | No |
| `schema_with_directives` | `schema @deprecated { query: Query }` parses | No |
| `schema_unknown_operation_error` | `schema { foo: Bar }` produces error | Yes |
| `schema_unclosed_error` | `schema { query: Query` produces error | Yes |

### 2.8 Scalar Types
**Spec:** https://spec.graphql.org/September2025/#sec-Scalars

| Test | Summary | Edge Case? |
|------|---------|------------|
| `scalar_simple` | `scalar DateTime` parses | No |
| `scalar_with_description` | `"desc" scalar DateTime` parses | No |
| `scalar_with_directives` | `scalar JSON @specifiedBy(url: "...")` parses | No |
| `scalar_name_keyword` | `scalar type` allowed | Yes |

### 2.9 Object Types
**Spec:** https://spec.graphql.org/September2025/#sec-Objects

| Test | Summary | Edge Case? |
|------|---------|------------|
| `object_simple` | `type User { name: String }` parses | No |
| `object_with_description` | `"User type" type User { }` parses | No |
| `object_implements_one` | `type User implements Node { }` parses | No |
| `object_implements_multiple` | `type User implements Node & Entity { }` parses | No |
| `object_implements_leading_ampersand` | `type User implements & Node & Entity` parses | Yes |
| `object_with_directives` | `type User @deprecated { }` parses | No |
| `object_multiple_fields` | Object with many fields | No |
| `object_field_with_args` | `type Query { user(id: ID!): User }` parses | No |
| `object_field_description` | Field with description string | No |
| `object_field_directives` | Field with directives | No |
| `object_empty_fields` | `type User { }` - empty body is valid | Yes |
| `object_no_body` | `type User` - no body is valid | Yes |

### 2.10 Interface Types
**Spec:** https://spec.graphql.org/September2025/#sec-Interfaces

| Test | Summary | Edge Case? |
|------|---------|------------|
| `interface_simple` | `interface Node { id: ID! }` parses | No |
| `interface_implements` | `interface Named implements Node { }` parses | No |
| `interface_with_fields` | Interface with multiple fields | No |
| `interface_no_body` | `interface Node` - no body is valid | Yes |

### 2.11 Union Types
**Spec:** https://spec.graphql.org/September2025/#sec-Unions

| Test | Summary | Edge Case? |
|------|---------|------------|
| `union_simple` | `union SearchResult = User` parses | No |
| `union_multiple_members` | `union Result = User \| Post \| Comment` parses | No |
| `union_leading_pipe` | `union Result = \| User \| Post` parses | Yes |
| `union_with_directives` | `union Result @deprecated = User` parses | No |
| `union_no_members` | `union Empty` - no `=` clause is valid | Yes |

### 2.12 Enum Types
**Spec:** https://spec.graphql.org/September2025/#sec-Enums

| Test | Summary | Edge Case? |
|------|---------|------------|
| `enum_simple` | `enum Status { ACTIVE INACTIVE }` parses | No |
| `enum_with_description` | Enum with description | No |
| `enum_value_description` | Enum value with description | No |
| `enum_value_directives` | `enum S { ACTIVE @deprecated }` parses | No |
| `enum_value_true_error` | `enum Bool { true false }` produces error | Yes |
| `enum_value_null_error` | `enum Maybe { null some }` produces error | Yes |
| `enum_empty_body` | `enum Status { }` - empty is valid | Yes |
| `enum_no_body` | `enum Status` - no body is valid | Yes |

### 2.13 Input Object Types
**Spec:** https://spec.graphql.org/September2025/#sec-Input-Objects

| Test | Summary | Edge Case? |
|------|---------|------------|
| `input_simple` | `input CreateUserInput { name: String! }` parses | No |
| `input_with_defaults` | `input I { limit: Int = 10 }` parses | No |
| `input_field_directives` | Input field with directives | No |
| `input_empty_body` | `input I { }` is valid | Yes |
| `input_no_body` | `input I` is valid | Yes |
| `input_default_no_variables` | Default value cannot contain variables | Yes |

### 2.14 Directive Definitions
**Spec:** https://spec.graphql.org/September2025/#sec-Type-System.Directives

| Test | Summary | Edge Case? |
|------|---------|------------|
| `directive_def_simple` | `directive @deprecated on FIELD_DEFINITION` parses | No |
| `directive_def_multiple_locations` | `directive @d on FIELD \| OBJECT` parses | No |
| `directive_def_leading_pipe` | `directive @d on \| FIELD \| OBJECT` parses | Yes |
| `directive_def_with_args` | `directive @deprecated(reason: String) on FIELD` | No |
| `directive_def_repeatable` | `directive @tag repeatable on OBJECT` parses | No |
| `directive_def_all_locations` | Test all 18 directive locations | No |
| `directive_def_unknown_location_error` | `directive @d on FOOBAR` produces error | Yes |
| `directive_def_typo_suggestion` | `directive @d on FEILD` suggests `FIELD` | Yes |

### 2.15 Type Extensions
**Spec:** https://spec.graphql.org/September2025/#sec-Type-System-Extensions

| Test | Summary | Edge Case? |
|------|---------|------------|
| `extend_scalar` | `extend scalar DateTime @specifiedBy(...)` parses | No |
| `extend_type_add_fields` | `extend type User { age: Int }` parses | No |
| `extend_type_add_implements` | `extend type User implements NewInterface` parses | No |
| `extend_type_add_directives` | `extend type User @deprecated` parses | No |
| `extend_interface` | `extend interface Node { }` parses | No |
| `extend_union` | `extend union Result = NewType` parses | No |
| `extend_enum` | `extend enum Status { PENDING }` parses | No |
| `extend_input` | `extend input CreateUserInput { extra: String }` parses | No |
| `extend_schema_unsupported` | `extend schema` produces unsupported error | Yes |

### 2.16 Document Types
| Test | Summary | Edge Case? |
|------|---------|------------|
| `parse_schema_document` | Only type system definitions accepted | No |
| `parse_schema_rejects_operation` | Operation in schema doc produces error | Yes |
| `parse_schema_rejects_fragment` | Fragment in schema doc produces error | Yes |
| `parse_executable_document` | Only operations/fragments accepted | No |
| `parse_executable_rejects_type` | Type def in executable doc produces error | Yes |
| `parse_executable_rejects_directive_def` | Directive def in executable doc produces error | Yes |
| `parse_mixed_document` | Both type system and executable accepted | No |
| `parse_empty_document` | Empty string parses to empty document | Yes |
| `parse_whitespace_only` | Whitespace-only string parses to empty document | Yes |
| `parse_comments_only` | Comments-only string parses to empty document | Yes |

### 2.17 Error Recovery
| Test | Summary | Edge Case? |
|------|---------|------------|
| `recovery_continues_after_error` | After syntax error, continues to next definition | No |
| `recovery_multiple_errors` | Multiple errors collected in single parse | No |
| `recovery_unclosed_brace_hint` | Unclosed `{` error includes "opened here" note | No |
| `recovery_unclosed_paren_hint` | Unclosed `(` error includes "opened here" note | No |
| `recovery_unclosed_bracket_hint` | Unclosed `[` error includes "opened here" note | No |
| `recovery_skips_to_definition` | Recovery skips garbage to find next `type`, etc. | No |
| `recovery_delimiter_stack_cleared` | Recovery clears delimiter stack properly | Yes |
| `recovery_lexer_error_propagated` | Lexer errors become parse errors | No |

### 2.18 Edge Cases and Subtle Scenarios
| Test | Summary | Edge Case? |
|------|---------|------------|
| `name_true_false_null_as_names` | `true`, `false`, `null` valid as field/type names in most contexts | Yes |
| `keyword_as_argument_name` | `field(type: 1, query: 2)` - keywords as arg names | Yes |
| `keyword_as_field_name` | `{ type query mutation }` - keywords as field names | Yes |
| `deep_nesting` | Deeply nested selection sets (100 levels) | Yes |
| `very_long_document` | Large document (1MB+) stress test | Yes |
| `many_definitions` | 1000+ type definitions in one doc | Yes |
| `many_fields` | Type with 1000+ fields | Yes |
| `many_arguments` | Field with 100+ arguments | Yes |
| `unicode_in_names_rejected` | Non-ASCII in names produces error | Yes |
| `unicode_in_strings_allowed` | Unicode in string values works | No |
| `unicode_in_descriptions` | Unicode in descriptions works | No |
| `empty_string_description` | `"" type User { }` - empty description | Yes |
| `block_string_description` | `"""desc""" type User { }` | No |
| `consecutive_operations` | Multiple operations in one document | No |
| `consecutive_fragments` | Multiple fragments in one document | No |
| `fragment_before_operation` | Fragment defined before operation using it | No |
| `duplicate_field_names` | Same field selected twice (valid at parse level) | Yes |

---

## Part 3: Test Implementation Guidelines

Per CLAUDE.md:

1. Each test must include a clear English description of what it verifies
2. Include links to relevant GraphQL spec sections where applicable
3. All tests written by Claude should indicate "Written by Claude Code, reviewed by a human"
4. Place tests in `tests/` subdirectory with `*_tests.rs` naming
5. Use `#[cfg(test)]` annotations in `mod.rs`

### Suggested File Organization

```
crates/libgraphql-parser/src/
├── token_source/
│   └── tests/
│       ├── mod.rs
│       ├── str_to_graphql_token_source_tests.rs  (existing, expand)
│       ├── punctuator_tests.rs
│       ├── name_tests.rs
│       ├── number_tests.rs
│       ├── string_tests.rs
│       ├── comment_tests.rs
│       ├── whitespace_tests.rs
│       ├── position_tests.rs
│       └── error_recovery_tests.rs
├── tests/
│   └── (parser tests)
│       ├── mod.rs
│       ├── value_tests.rs
│       ├── type_tests.rs
│       ├── directive_tests.rs
│       ├── selection_set_tests.rs
│       ├── operation_tests.rs
│       ├── fragment_tests.rs
│       ├── schema_definition_tests.rs
│       ├── type_definition_tests.rs
│       ├── type_extension_tests.rs
│       ├── document_tests.rs
│       ├── error_recovery_tests.rs
│       └── edge_case_tests.rs
```

---

## Summary Statistics

| Category | New Tests | Existing |
|----------|-----------|----------|
| **Lexer - Punctuators** | 3 | 9 |
| **Lexer - Names** | 6 | 1 |
| **Lexer - Keywords** | 4 | 2 |
| **Lexer - Integers** | 10 | 3 |
| **Lexer - Floats** | 12 | 2 |
| **Lexer - Single-line Strings** | 15 | 3 |
| **Lexer - Block Strings** | 7 | 3 |
| **Lexer - Comments** | 3 | 3 |
| **Lexer - Whitespace** | 2 | 2 |
| **Lexer - Position Tracking** | 3 | 7 |
| **Lexer - Error Recovery** | 7 | 1 |
| **Parser - Values** | 23 | 0 |
| **Parser - Types** | 10 | 0 |
| **Parser - Directives** | 7 | 0 |
| **Parser - Selection Sets** | 16 | 0 |
| **Parser - Operations** | 12 | 0 |
| **Parser - Fragments** | 6 | 0 |
| **Parser - Schema Definitions** | 5 | 0 |
| **Parser - Scalar Types** | 4 | 0 |
| **Parser - Object Types** | 12 | 0 |
| **Parser - Interface Types** | 4 | 0 |
| **Parser - Union Types** | 5 | 0 |
| **Parser - Enum Types** | 8 | 0 |
| **Parser - Input Objects** | 6 | 0 |
| **Parser - Directive Definitions** | 8 | 0 |
| **Parser - Type Extensions** | 9 | 0 |
| **Parser - Document Types** | 10 | 0 |
| **Parser - Error Recovery** | 8 | 0 |
| **Parser - Edge Cases** | 19 | 0 |
| **NEW TESTS TOTAL** | **~229** | 36 |

---

## Decisions Made

1. ✅ Vendored tests from `graphql-js` / `graphql-parser` will be added in a **future task**
2. ✅ Fuzzing tests (`cargo-fuzz`) will be added in a **future task**
3. ✅ Focus is on exhaustive unit tests without duplicating existing tests
