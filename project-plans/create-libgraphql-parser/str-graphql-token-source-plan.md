# Implementation Plan: StrGraphQLTokenSource

**Goal:** Implement an exhaustively-tested `StrGraphQLTokenSource` lexer for GraphQL in the `libgraphql-parser` crate.

---

## Status Summary (Updated 2026-01-20)

| Step | Description | Status |
|------|-------------|--------|
| 0 | Refactor `GraphQLTokenKind` to use `Cow<'src, str>` | ✅ COMPLETE |
| 1 | Create Lexer Skeleton | ✅ COMPLETE |
| 2 | Whitespace and Basic Punctuators | ✅ COMPLETE |
| 3 | Comments and Ellipsis | ✅ COMPLETE |
| 4 | Names and Keywords | ✅ COMPLETE |
| 5 | Numeric Literals | ✅ COMPLETE |
| 6 | String Literals | ✅ COMPLETE |
| 7 | Invalid Character Handling | ✅ COMPLETE |
| 8 | Comprehensive Test Suite | 🟡 PARTIAL (unit tests done; integration/differential tests remain) |
| 9 | Vendored Test Porting | 🔲 TODO |
| 10 | Performance and Fuzzing | 🔲 TODO |

**Test Count:** 383 tests passing (as of 2026-01-20)

**Key Files:**
- Implementation: `crates/libgraphql-parser/src/token_source/str_to_graphql_token_source.rs` (~1130 lines)
- Tests: `crates/libgraphql-parser/src/token_source/tests/str_to_graphql_token_source_tests.rs` (~2160 lines)

---

## Architectural Decision: Zero-Copy with `Cow<'a, str>`

We will use `Cow<'a, str>` (Clone-on-Write) for string data in `GraphQLTokenKind`, enabling:
- **Zero-copy lexing** for `StrGraphQLTokenSource` (borrows from source string)
- **Owned strings** for `RustMacroGraphQLTokenSource` (proc_macro2 requires allocation)

This requires adding a lifetime parameter to `GraphQLTokenKind<'a>`, `GraphQLToken<'a>`, and related types.

### Why `Cow` instead of `&'a str`?

`RustMacroGraphQLTokenSource` cannot return borrowed strings because `proc_macro2::Ident::to_string()` returns an owned `String` — there's no contiguous source buffer to borrow from. `Cow` allows both borrowed and owned data in the same type.

---

## Context Summary

### Existing Infrastructure (Already Implemented)

| Component | Location | Purpose |
|-----------|----------|---------|
| `SourcePosition` | `src/source_position.rs` | Dual-column position tracking (UTF-8, optional UTF-16) |
| `GraphQLSourceSpan` | `src/graphql_source_span.rs` | Start/end positions with optional file path |
| `GraphQLToken` | `src/token/graphql_token.rs` | Token with kind, preceding_trivia, span |
| `GraphQLTokenKind` | `src/token/graphql_token_kind.rs` | 14 punctuators, literals, True/False/Null, Eof, Error |
| `GraphQLTriviaToken` | `src/token/graphql_trivia_token.rs` | Comment and Comma variants |
| `GraphQLTokenSource` | `src/token_source/graphql_token_source.rs` | Marker trait: `Iterator<Item = GraphQLToken>` |
| `parse_string_value()` | `src/token/graphql_token_kind.rs` | String escape processing (single-line & block) |
| `GraphQLParseError` | `src/graphql_parse_error.rs` | Parse errors with notes |
| `GraphQLErrorNote` | `src/graphql_error_note.rs` | Error notes (General, Help, Spec) |

### Reference Implementation

`RustMacroGraphQLTokenSource` in `libgraphql-macros` (~890 lines) provides a reference for:
- State machine for `.` → `..` → `...` handling
- Block string detection/combination
- Negative number handling (`-17` → single token)
- Error generation with helpful notes
- Trivia (comma) accumulation

### Key Differences from RustMacroGraphQLTokenSource

| Aspect | RustMacroGraphQLTokenSource | StrGraphQLTokenSource |
|--------|-----------------------------|-----------------------|
| Input | `proc_macro2::TokenStream` | `&str` |
| UTF-16 column | `None` (unavailable from proc_macro2) | `Some(value)` (computed) |
| Comments | Cannot preserve (Rust strips them) | Preserves `#` comments |
| Dot handling | Rust tokenizes separately, combined | Direct scanning |
| Negative numbers | Combine `-` + number | Direct scan as single token |

---

## Implementation Steps

### Step 0: Refactor GraphQLTokenKind to Use `Cow<'src, str>` ✅ COMPLETED

**Files to modify:**
- `src/token/graphql_token_kind.rs`
- `src/token/graphql_token.rs`
- `src/token/mod.rs`
- `src/graphql_token_stream.rs`
- `libgraphql-macros/src/rust_macro_graphql_token_source.rs`

**Tasks:**
1. Update `GraphQLTokenKind` to use `Cow<'src, str>`:
   ```rust
   use std::borrow::Cow;

   #[derive(Clone, Debug, PartialEq)]
   pub enum GraphQLTokenKind<'src> {
       // Punctuators unchanged (no string data)
       Ampersand,
       At,
       // ... etc

       // String-carrying variants now use Cow
       Name(Cow<'src, str>),
       IntValue(Cow<'src, str>),
       FloatValue(Cow<'src, str>),
       StringValue(Cow<'src, str>),

       // Error uses plain String (typically dynamically constructed)
       Error {
           message: String,
           error_notes: GraphQLErrorNotes,
       },

       // Others unchanged
       True,
       False,
       Null,
       Eof,
   }
   ```

2. Update `GraphQLToken` to carry the lifetime:
   ```rust
   #[derive(Clone, Debug, PartialEq)]
   pub struct GraphQLToken<'src> {
       pub kind: GraphQLTokenKind<'src>,
       pub preceding_trivia: GraphQLTriviaTokenVec<'src>,
       pub span: GraphQLSourceSpan,
   }
   ```

3. Update `GraphQLTriviaToken` (Comment value uses Cow):
   ```rust
   pub enum GraphQLTriviaToken<'src> {
       Comment {
           value: Cow<'src, str>,
           span: GraphQLSourceSpan,
       },
       Comma {
           span: GraphQLSourceSpan,
       },
   }
   ```

4. Update `GraphQLTokenSource` trait:
   ```rust
   pub trait GraphQLTokenSource<'src>: Iterator<Item = GraphQLToken<'src>> {}
   impl<'src, T> GraphQLTokenSource<'src> for T where T: Iterator<Item = GraphQLToken<'src>> {}
   ```

5. Update `GraphQLTokenStream<S>` to handle lifetime:
   ```rust
   pub struct GraphQLTokenStream<'src, S: GraphQLTokenSource<'src>> {
       // ...
   }
   ```

6. Add helper constructors for convenience (before updating RustMacroGraphQLTokenSource):
   ```rust
   impl<'src> GraphQLTokenKind<'src> {
       /// Create a Name token from a borrowed string slice.
       pub fn name_borrowed(s: &'src str) -> Self {
           GraphQLTokenKind::Name(Cow::Borrowed(s))
       }

       /// Create a Name token from an owned String.
       pub fn name_owned(s: String) -> Self {
           GraphQLTokenKind::Name(Cow::Owned(s))
       }

       /// Create an IntValue token from a borrowed string slice.
       pub fn int_value_borrowed(s: &'src str) -> Self {
           GraphQLTokenKind::IntValue(Cow::Borrowed(s))
       }

       /// Create an IntValue token from an owned String.
       pub fn int_value_owned(s: String) -> Self {
           GraphQLTokenKind::IntValue(Cow::Owned(s))
       }

       /// Create a FloatValue token from a borrowed string slice.
       pub fn float_value_borrowed(s: &'src str) -> Self {
           GraphQLTokenKind::FloatValue(Cow::Borrowed(s))
       }

       /// Create a FloatValue token from an owned String.
       pub fn float_value_owned(s: String) -> Self {
           GraphQLTokenKind::FloatValue(Cow::Owned(s))
       }

       /// Create a StringValue token from a borrowed string slice.
       pub fn string_value_borrowed(s: &'src str) -> Self {
           GraphQLTokenKind::StringValue(Cow::Borrowed(s))
       }

       /// Create a StringValue token from an owned String.
       pub fn string_value_owned(s: String) -> Self {
           GraphQLTokenKind::StringValue(Cow::Owned(s))
       }

       /// Create an Error token (message is always owned).
       pub fn error(message: impl Into<String>, error_notes: GraphQLErrorNotes) -> Self {
           GraphQLTokenKind::Error {
               message: message.into(),
               error_notes,
           }
       }
   }
   ```

7. Update `RustMacroGraphQLTokenSource` to use helper constructors:
   ```rust
   // Before:
   GraphQLTokenKind::Name(ident.to_string())

   // After:
   GraphQLTokenKind::name_owned(ident.to_string())

   // Before:
   GraphQLTokenKind::IntValue(lit_str)

   // After:
   GraphQLTokenKind::int_value_owned(lit_str)

   // etc.
   ```

8. **Tests:**
   - All existing tests still pass
   - `RustMacroGraphQLTokenSource` works with helper constructors
   - `cargo clippy --tests` passes
   - `cargo test` passes

---

### Step 1: Create Lexer Skeleton ✅ COMPLETED

**File:** `/crates/libgraphql-parser/src/token_source/str_to_graphql_token_source.rs`

**Tasks:**
1. Define `StrGraphQLTokenSource<'src>` struct with:
   - `source: &'src str` — Full source text (for scanning and borrowing token text)
   - `curr_byte_offset: usize` — Current byte offset from start (use `&source[curr_byte_offset..]` for remaining text)
   - `curr_line: usize` — Current 0-based line
   - `curr_col_utf8: usize` — Current UTF-8 character column
   - `curr_col_utf16: usize` — Current UTF-16 code unit column
   - `curr_last_was_cr: bool` — For `\r\n` handling (was last char `\r`?)
   - `pending_trivia: GraphQLTriviaTokenVec<'src>` — Accumulated trivia
   - `finished: bool` — Has EOF been emitted
   - `file_path: Option<&'src Path>` — Optional file path for spans (borrowed, not owned)

2. Implement constructors:
   ```rust
   pub fn new(source: &'src str) -> Self
   pub fn with_file_path(source: &'src str, path: &'src Path) -> Self
   ```

3. Implement position helpers:
   ```rust
   fn remaining(&self) -> &'src str  // Returns &source[curr_byte_offset..]
   fn curr_position(&self) -> SourcePosition
   fn peek_char(&self) -> Option<char>  // self.remaining().chars().next()
   fn peek_char_nth(&self, n: usize) -> Option<char>
   fn consume(&mut self) -> Option<char>  // Handles line/column tracking
   fn make_span(&self, start: SourcePosition) -> GraphQLSourceSpan
   ```

4. Implement `Iterator<Item = GraphQLToken<'src>>` (stub returning `todo!()`)

5. **Tests:**
   - Constructor creates valid state (all `curr_*` fields initialized to 0)
   - `peek_char()` returns first char without consuming
   - `consume()` consumes and updates position correctly
   - **Position tracking tests:**
     - Single ASCII char: `curr_line=0`, `curr_col_utf8=1`, `curr_col_utf16=1`, `curr_byte_offset=1`
     - Multi-byte UTF-8 (é = 2 bytes): `curr_col_utf8=1`, `curr_col_utf16=1`, `curr_byte_offset=2`
     - Supplementary plane (🎉 = 4 bytes): `curr_col_utf8=1`, `curr_col_utf16=2`, `curr_byte_offset=4`
     - Newline: `curr_line` increments, columns reset to 0

---

### Step 2: Whitespace and Basic Punctuators ✅ COMPLETED

**Tasks:**
1. Implement whitespace skipping:
   - Space (U+0020)
   - Tab (U+0009)
   - Newlines: `\n` (U+000A), `\r` (U+000D), `\r\n` (as single newline)
   - BOM: U+FEFF (Unicode byte order mark - ignored at start)

2. Implement single-character punctuators:
   ```rust
   '!' => GraphQLTokenKind::Bang
   '$' => GraphQLTokenKind::Dollar
   '&' => GraphQLTokenKind::Ampersand
   '(' => GraphQLTokenKind::ParenOpen
   ')' => GraphQLTokenKind::ParenClose
   ':' => GraphQLTokenKind::Colon
   '=' => GraphQLTokenKind::Equals
   '@' => GraphQLTokenKind::At
   '[' => GraphQLTokenKind::SquareBracketOpen
   ']' => GraphQLTokenKind::SquareBracketClose
   '{' => GraphQLTokenKind::CurlyBraceOpen
   '}' => GraphQLTokenKind::CurlyBraceClose
   '|' => GraphQLTokenKind::Pipe
   ```

3. Handle comma as trivia:
   - `','` → `GraphQLTriviaToken::Comma` added to `pending_trivia`

4. Implement EOF:
   - When `curr_byte_offset >= source.len()`, emit `GraphQLTokenKind::Eof` with any trailing trivia

5. **Tests:**
   - Tokenize `{ }` → `CurlyBraceOpen`, `CurlyBraceClose`, `Eof`
   - Whitespace is skipped
   - Commas accumulated as trivia
   - **Position tracking tests:**
     - `{` at start: span is `(0,0)-(0,1)`, byte_offset `0-1`
     - `{\n}` second `}` at: line=1, col=0
     - `{\r\n}` second `}` at: line=1, col=0 (single newline for `\r\n`)
     - `{\r}` (legacy Mac): second `}` at line=1, col=0
     - Tab advances column by 1 (not 4/8)
     - BOM at start is skipped, doesn't affect position

---

### Step 3: Comments and Ellipsis ✅ COMPLETED

**Tasks:**
1. Implement comment lexing:
   - `#` starts a comment
   - Comment extends to end of line (or EOF)
   - Emit `GraphQLTriviaToken::Comment { value, span }`
   - Attach to `pending_trivia`

2. Implement spread operator with comprehensive dot handling:
   - `...` (adjacent) → `GraphQLTokenKind::Ellipsis`
   - `.` (single) → Error
   - `..` (adjacent) → Error with help note
   - `. .` (spaced, same line) → Error with help note about spacing
   - `. ..` (spaced, same line) → Error with help note
   - `.. .` (spaced, same line) → Error with help note
   - `. . .` (all spaced, same line) → Error with help note
   - Dots on different lines → Separate error tokens

3. **Error messages for dots (matching RustMacroGraphQLTokenSource patterns):**
   ```rust
   // Single dot
   GraphQLTokenKind::error(
       "Unexpected `.` (use `...` for spread operator)",
       smallvec![],
   )

   // Adjacent double dot (can still become `...` if followed by adjacent `.`)
   GraphQLTokenKind::error(
       "Unexpected `..` (use `...` for spread operator)",
       smallvec![GraphQLErrorNote::help(
           "Add one more `.` to form the spread operator `...`"
       )],
   )

   // Spaced `. .` (same line) - terminal, won't become `...`
   GraphQLTokenKind::error(
       "Unexpected `. .` (use `...` for spread operator)",
       smallvec![GraphQLErrorNote::help_with_span(
           "These dots may have been intended to form a `...` spread operator. \
            Try removing the extra spacing between the dots.",
           span_of_second_dot,
       )],
   )

   // Spaced `.. .` (same line)
   GraphQLTokenKind::error(
       "Unexpected `.. .`",
       smallvec![GraphQLErrorNote::help_with_span(
           "This `.` may have been intended to complete a `...` spread operator. \
            Try removing the extra spacing between the dots.",
           span_of_third_dot,
       )],
   )

   // Spaced `. . .` (same line)
   GraphQLTokenKind::error(
       "Unexpected `. . .`",
       smallvec![GraphQLErrorNote::help_with_span(
           "These dots may have been intended to form a `...` spread operator. \
            Try removing the extra spacing between the dots.",
           span_of_third_dot,
       )],
   )
   ```

4. **Tests:**
   - `# comment\nfield` → Comment attached to `Name("field")`
   - `...` → `Ellipsis`
   - `.` → Error "Unexpected `.`"
   - `..` → Error "Unexpected `..`" with help note
   - `. .` (same line) → Error "Unexpected `. .`" with help about spacing
   - `.. .` (same line) → Error "Unexpected `.. .`" with help
   - `. ..` (same line) → Error with help
   - `. . .` (same line) → Error "Unexpected `. . .`" with help
   - `.\n.` (different lines) → Two separate single-dot errors
   - Comments at EOF attached to `Eof` token
   - **Position tracking tests:**
     - Comment span includes `#` and content, excludes newline
     - `...` span covers all 3 characters
     - Spaced dot errors span from first dot to last dot

---

### Step 4: Names and Keywords ✅ COMPLETED

**Tasks:**
1. Implement name lexing per GraphQL spec:
   - Pattern: `/[_A-Za-z][_0-9A-Za-z]*/`
   - Start: `_` or letter
   - Continue: `_`, digit, or letter

2. Emit special keywords as distinct tokens:
   - `"true"` → `GraphQLTokenKind::True`
   - `"false"` → `GraphQLTokenKind::False`
   - `"null"` → `GraphQLTokenKind::Null`
   - All other names → `GraphQLTokenKind::name_borrowed(&source[start..end])`

3. Note: GraphQL keywords (`type`, `query`, etc.) are context-dependent and lexed as `Name` tokens.

4. **Tests:**
   - `hello` → `Name("hello")`
   - `_private` → `Name("_private")`
   - `type2` → `Name("type2")`
   - `__typename` → `Name("__typename")` (double underscore allowed)
   - `true` → `True`
   - `false` → `False`
   - `null` → `Null`
   - `query` → `Name("query")` (parser decides context)
   - `True` → `Name("True")` (case sensitive - only lowercase is keyword)
   - `NULL` → `Name("NULL")` (case sensitive)
   - **Position tracking tests:**
     - `hello` at start: span `(0,0)-(0,5)`
     - `  hello` with leading spaces: span `(0,2)-(0,7)`
     - Multi-line `{\nhello}`: `hello` at line=1, col=0

---

### Step 5: Numeric Literals ✅ COMPLETED

**Tasks:**
1. Implement integer lexing:
   - Optional negative sign: `-`
   - Zero: `0`
   - Non-zero: `[1-9][0-9]*`
   - Invalid: `00`, `01` (leading zeros)
   - Emit as `GraphQLTokenKind::int_value_borrowed(&source[start..end])`

2. Implement float lexing:
   - Decimal part: `.` followed by digits
   - Exponent part: `e`/`E` followed by optional `+`/`-` and digits
   - At least one of decimal or exponent required
   - Emit as `GraphQLTokenKind::float_value_borrowed(&source[start..end])`

3. Disambiguation logic:
   ```rust
   fn lex_number(&mut self, start_pos: SourcePosition) -> GraphQLToken<'src> {
       // 1. Optional negative sign
       // 2. Integer part (check for invalid leading zeros)
       // 3. Check for decimal point → float
       // 4. Check for exponent → float
       // 5. Return IntValue or FloatValue
   }
   ```

4. **Tests:**
   - `123` → `IntValue("123")`
   - `-456` → `IntValue("-456")`
   - `0` → `IntValue("0")`
   - `-0` → `IntValue("-0")` (valid per spec)
   - `1.5` → `FloatValue("1.5")`
   - `-3.14` → `FloatValue("-3.14")`
   - `1e10` → `FloatValue("1e10")`
   - `1E10` → `FloatValue("1E10")` (uppercase E)
   - `1e+10` → `FloatValue("1e+10")` (explicit plus)
   - `1.23e-4` → `FloatValue("1.23e-4")`
   - `0.0` → `FloatValue("0.0")`
   - `00` → Error (leading zeros)
   - `01` → Error (leading zeros)
   - `1.` → Error (no digits after decimal)
   - `.5` → Error (no digits before decimal - not valid GraphQL)
   - `1e` → Error (no digits after exponent)
   - **Position tracking tests:**
     - `-123` span includes the negative sign: `(0,0)-(0,4)`
     - `1.23e-4` span covers entire literal: 7 characters
     - `field: 42` the `42` starts at correct column after colon and space

---

### Step 6: String Literals ✅ COMPLETED

**Tasks:**
1. Implement single-line string lexing:
   - Delimited by `"`
   - Reject unescaped newlines
   - Reject unescaped control characters
   - Store raw source text (escape processing done by `parse_string_value()`)
   - Handle unterminated string error

2. Implement block string lexing:
   - Delimited by `"""`
   - Allow newlines and unescaped quotes
   - Store raw source text
   - Handle unterminated block string error
   - Handle escaped triple-quote: `\"""` per spec (https://spec.graphql.org/September2025/#BlockStringCharacter)

3. **Error handling:**
   ```rust
   // Unterminated single-line string
   GraphQLTokenKind::error(
       "Unterminated string literal",
       smallvec![
           GraphQLErrorNote::general_with_span("String started here", start_span),
           GraphQLErrorNote::help("Add closing `\"`"),
       ],
   )

   // Unterminated block string
   GraphQLTokenKind::error(
       "Unterminated block string",
       smallvec![
           GraphQLErrorNote::general_with_span("Block string started here", start_span),
           GraphQLErrorNote::help("Add closing `\"\"\"`"),
       ],
   )
   ```

4. **Basic Tests:**
   - `"hello"` → `StringValue("\"hello\"")`
   - `"line1\nline2"` → `StringValue("\"line1\\nline2\"")`
   - `"""block"""` → `StringValue("\"\"\"block\"\"\"")`
   - `"""multi\nline"""` → Block string with newline
   - Unterminated string → Error with helpful notes
   - Unterminated block string → Error with helpful notes

5. **Block String Escape Test (per spec https://spec.graphql.org/September2025/#BlockStringCharacter):**
   - `"""\""""""` → `StringValue("\"\"\"\\\"\"\"\"\"\"")` — The `\"""` is an escaped triple-quote inside a block string, followed by `"""` which closes the block. The result is a block string containing a literal `"""`.

6. **Extensive String Escape Security Tests (CRITICAL):**

   **Single-line escape sequences:**
   - `"\n"` → newline
   - `"\r"` → carriage return
   - `"\t"` → tab
   - `"\\"` → backslash
   - `"\""` → double quote
   - `"\/"` → forward slash
   - `"\b"` → backspace (U+0008)
   - `"\f"` → form feed (U+000C)

   **Unicode escapes (fixed 4-digit):**
   - `"\u0041"` → `A`
   - `"\u0000"` → NUL character (allowed)
   - `"\u001F"` → Control character (allowed via escape)
   - `"\uD800"` → Lone surrogate (should error or produce replacement char?)
   - `"\uDFFF"` → Lone surrogate
   - `"\uD83D\uDE00"` → Surrogate pair for 😀 (test if supported)

   **Unicode escapes (variable length):**
   - `"\u{41}"` → `A`
   - `"\u{0}"` → NUL
   - `"\u{1F600}"` → 😀
   - `"\u{10FFFF}"` → Max valid code point
   - `"\u{110000}"` → Error (beyond Unicode range)
   - `"\u{FFFFFFFF}"` → Error (way beyond range)
   - `"\u{}"` → Error (empty)
   - `"\u{GGGG}"` → Error (invalid hex)

   **Edge cases and attack vectors:**
   - `"\u202E"` → Right-to-Left Override (security concern for display)
   - `"\u2028"` → Line Separator (might break JSON parsers)
   - `"\u2029"` → Paragraph Separator
   - `"\u0000"` → NUL byte injection
   - `"\"` → Error (unterminated - backslash at end)
   - `"\x41"` → Error (not a valid GraphQL escape)
   - `"\U00000041"` → Error (uppercase U not valid)
   - String with embedded NUL: `"a\u0000b"` → `a` + NUL + `b`
   - Very long escape sequence: `"\u{00000000000000041}"` → Test handling
   - `"\` at EOF → Error

   **Block string security:**
   - Block string with `"""` inside: `"""contains \"\"\" quotes"""`
   - Unescaped backslash in block string: `"""\ not an escape"""`
   - Only `\"""` is an escape in block strings
   - Block string with lone `\`: `"""\"""` → Contains literal `\`
   - Block string at EOF: `"""unterminated` → Error

   **Position tracking tests:**
   - `"hello"` span is `(0,0)-(0,7)` (includes quotes)
   - Multi-line block string: track line numbers through content
   - Escape sequence doesn't change span (raw text, not processed)

---

### Step 7: Invalid Character Handling ✅ COMPLETED

**Tasks:**
1. Implement `describe_char()` helper for error messages:
   ```rust
   /// Returns a human-readable description of a character for error messages.
   /// For printable characters, returns the character in backticks.
   /// For invisible/control characters, includes Unicode code point and name.
   fn describe_char(ch: char) -> String {
       if ch.is_control() || ch.is_whitespace() && ch != ' ' {
           // Invisible characters get detailed description
           let name = unicode_name(ch);
           format!("`{}` (U+{:04X}: {})", ch, ch as u32, name)
       } else {
           format!("`{}`", ch)
       }
   }
   ```

2. **Invisible character descriptions (must handle all of these):**
   | Code Point | Name | Description |
   |------------|------|-------------|
   | U+0000 | NUL | Null |
   | U+0001-U+001F | (various) | C0 Control Characters |
   | U+007F | DEL | Delete |
   | U+0080-U+009F | (various) | C1 Control Characters |
   | U+00A0 | NBSP | No-Break Space |
   | U+00AD | SHY | Soft Hyphen |
   | U+034F | CGJ | Combining Grapheme Joiner |
   | U+061C | ALM | Arabic Letter Mark |
   | U+115F | (HCF) | Hangul Choseong Filler |
   | U+1160 | (HJF) | Hangul Jungseong Filler |
   | U+17B4 | (KV) | Khmer Vowel Inherent Aq |
   | U+17B5 | (KV) | Khmer Vowel Inherent Aa |
   | U+180E | MVS | Mongolian Vowel Separator |
   | U+2000-U+200A | (various) | Whitespace variants |
   | U+200B | ZWSP | Zero Width Space |
   | U+200C | ZWNJ | Zero Width Non-Joiner |
   | U+200D | ZWJ | Zero Width Joiner |
   | U+200E | LRM | Left-to-Right Mark |
   | U+200F | RLM | Right-to-Left Mark |
   | U+202A | LRE | Left-to-Right Embedding |
   | U+202B | RLE | Right-to-Left Embedding |
   | U+202C | PDF | Pop Directional Formatting |
   | U+202D | LRO | Left-to-Right Override |
   | U+202E | RLO | Right-to-Left Override |
   | U+2060 | WJ | Word Joiner |
   | U+2061-U+2064 | (various) | Invisible operators |
   | U+2066 | LRI | Left-to-Right Isolate |
   | U+2067 | RLI | Right-to-Left Isolate |
   | U+2068 | FSI | First Strong Isolate |
   | U+2069 | PDI | Pop Directional Isolate |
   | U+206A-U+206F | (various) | Deprecated format chars |
   | U+3000 | IDEOGRAPHIC SPACE | CJK whitespace |
   | U+FEFF | BOM | Byte Order Mark (except at start) |
   | U+FFF9-U+FFFB | (various) | Interlinear annotation |
   | U+E0001 | (TAG) | Language Tag |
   | U+E0020-U+E007F | (TAG) | Tag characters |

   Consider referencing the `unicode_names2` crate for a comprehensive set of name lookups.

3. Catch-all for unrecognized characters:
   ```rust
   _ => {
       let ch = self.consume().unwrap();
       GraphQLTokenKind::error(
           format!("Unexpected character `{}`", describe_char(ch)),
           smallvec![GraphQLErrorNote::general(
               "GraphQL only allows: A-Z, a-z, 0-9, _, and specific punctuation"
           )],
       )
   }
   ```

4. Handle specific invalid patterns with better messages:
   - `#!` at start → "Shebangs are not valid GraphQL"
   - `\` outside string → "Unexpected character `\\`. Backslash is only valid inside strings."
   - `%` → "Unexpected character `%`. GraphQL does not support modulo operator."

5. **Tests:**
   - `^` → Error "Unexpected character `^`"
   - `€` → Error (non-ASCII printable, shows character)
   - `{ ^ }` → Recovery: `CurlyBraceOpen`, Error, `CurlyBraceClose`, `Eof`
   - **Invisible character tests:**
     - Zero-width space (U+200B) → Error "Unexpected character `` (U+200B: Zero Width Space)"
     - Arabic Letter Mark (U+061C) → Error "Unexpected character `؜` (U+061C: Arabic Letter Mark)"
     - Right-to-Left Override (U+202E) → Error with code point and name
     - NUL (U+0000) → Error "Unexpected character `` (U+0000: Null)"
     - BOM in middle of file → Error (BOM only allowed at start)
   - **Position tracking tests:**
     - Error token has correct span for the invalid character
     - Multi-byte invalid char: span byte_offset reflects actual bytes

---

### Step 8: Comprehensive Test Suite 🟡 PARTIALLY COMPLETE

**Completed Tasks:**
1. ✅ **Unit tests for each token type:**
   - All 14 punctuators
   - All value types (Int, Float, String, Bool, Null)
   - Names and keywords
   - Comments and trivia
   - Error tokens
   - **Status:** 100+ unit tests in `str_to_graphql_token_source_tests.rs`

2. ✅ **Position tracking tests:**
   - ASCII: verify col_utf8 == col_utf16
   - Multi-byte UTF-8 (é, 中): col_utf8 +1, col_utf16 +1
   - Supplementary plane (🎉): col_utf8 +1, col_utf16 +2
   - Line endings: `\n`, `\r`, `\r\n` all increment line correctly
   - byte_offset tracks actual bytes
   - **Status:** Comprehensive position tracking tests exist

3. ✅ **Error recovery tests:**
   - Multiple errors in one document
   - Errors don't consume valid tokens
   - Error positions are accurate
   - **Status:** Multiple error recovery tests exist

**Remaining Tasks:**
4. 🔲 **Integration tests with real schemas:**
   - GitHub GraphQL schema (if license permits)
   - Star Wars example schema
   - Complex nested operations

5. 🔲 **Differential tests against `graphql_parser` crate:**
   - Parse same input with both lexers
   - Compare token sequences (kinds match)
   - Document expected differences

6. 🔲 **Cross-validation with `RustMacroGraphQLTokenSource`:**
   - Parse equivalent GraphQL with both
   - Compare token kinds and relative ordering
   - Note: positions will differ (coordinate spaces)

---

### Step 9: Vendored Test Porting

**Tasks:**
1. **License verification (REQUIRED FIRST):**
   - graphql-js: MIT License ✓
   - graphql-parser: MIT/Apache-2.0 ✓
   - Both licenses permit test case adaptation

2. **Port graphql-js lexer tests:**
   - Source: `graphql-js/src/__tests__/lexer-test.ts`
   - Target: `src/tests/str_to_graphql_token_source_tests.rs`
   - Include license header in vendored file
   - Convert TypeScript assertions to Rust

3. **Port graphql-parser tests:**
   - Source: `graphql-parser/tests/`
   - Include license attribution
   - Focus on lexer-specific tests

4. **Test organization:**
   ```
   src/tests/
   ├── str_to_graphql_token_source_tests.rs  (main test file)
   ├── vendored/
   │   ├── graphql_js_lexer_tests.rs         (from graphql-js)
   │   └── graphql_parser_tests.rs           (from graphql-parser)
   └── utils.rs                               (test helpers)
   ```

---

### Step 10: Performance and Fuzzing

**Tasks:**
1. **Benchmark suite:**
   - Create benchmarks in `/crates/libgraphql-parser/benches/`
   - Compare against `graphql_parser` crate
   - Measure: tokens/second, memory allocations
   - Target: within 2x of `graphql_parser` performance

2. **Fuzzing with cargo-fuzz:**
   - Create fuzz targets
   - Ensure no panics on arbitrary input
   - Run for at least 1 hour before declaring success

3. **Optimization opportunities:**
   - `String::with_capacity()` for string literals
   - Consider `memchr` for fast character scanning
   - Profile hot paths

---

## Verification Checklist

**Step 0 (Cow Refactoring): ✅ COMPLETED**
- [x] `GraphQLTokenKind<'src>` uses `Cow<'src, str>` for Name/IntValue/FloatValue/StringValue
- [x] `GraphQLTokenKind<'src>::Error.message` uses plain `String` (always dynamically constructed)
- [x] `GraphQLToken<'src>` carries lifetime
- [x] `GraphQLTriviaToken<'src>` Comment uses `Cow<'src, str>`
- [x] `GraphQLTokenSource<'src>` trait updated
- [x] `GraphQLTokenStream<'src, S>` handles lifetime
- [x] Helper constructors added (`name_borrowed`, `name_owned`, `error`, etc.)
- [x] `RustMacroGraphQLTokenSource` uses helper constructors and still works
- [x] All existing tests pass after refactoring

**Lexer Implementation (Steps 1-7): ✅ COMPLETED**
- [x] All 14 punctuators tokenize correctly
- [x] IntValue handles positive, negative, zero, and errors
- [x] FloatValue handles all valid formats and errors
- [x] StringValue handles single-line and block strings
- [x] True/False/Null are distinct token kinds
- [x] Names follow `/[_A-Za-z][_0-9A-Za-z]*/` pattern
- [x] Comments are captured as trivia
- [x] Commas are captured as trivia
- [x] Ellipsis (`...`) tokenizes correctly
- [x] Invalid characters emit Error tokens with helpful messages
- [x] Position tracking is accurate (line, col_utf8, col_utf16, byte_offset)
- [x] Line endings handled: `\n`, `\r`, `\r\n`
- [x] UTF-16 column computed correctly for BMP and supplementary chars
- [x] Error recovery continues after errors
- [x] `cargo clippy --tests` passes with no warnings
- [x] `cargo test` passes (383 tests passing as of 2026-01-20)

**Step 8 (Comprehensive Test Suite): 🟡 PARTIALLY COMPLETE**
- [x] Unit tests for each token type (punctuators, values, names, keywords, comments, trivia, errors)
- [x] Position tracking tests (ASCII, multi-byte UTF-8, supplementary plane, line endings, byte_offset)
- [x] Error recovery tests (multiple errors, error positions)
- [ ] Integration tests with real schemas (GitHub schema, Star Wars schema, complex nested operations)
- [ ] Differential tests against `graphql_parser` crate
- [ ] Cross-validation with `RustMacroGraphQLTokenSource`

**Step 9 (Vendored Test Porting): 🔲 TODO**
- [ ] License verification (graphql-js MIT, graphql-parser MIT/Apache-2.0)
- [ ] Port graphql-js lexer tests
- [ ] Port graphql-parser tests
- [ ] Test organization with vendored/ directory

**Step 10 (Performance and Fuzzing): 🔲 TODO**
- [ ] Benchmark suite in `/crates/libgraphql-parser/benches/`
- [ ] Fuzzing with cargo-fuzz
- [ ] Optimization opportunities (memchr, profiling)

---

## Critical Files (Implementation Status)

| File | Status | Notes |
|------|--------|-------|
| `src/token/graphql_token_kind.rs` | ✅ Done | `<'src>` lifetime, `Cow<'src, str>`, helper constructors |
| `src/token/graphql_token.rs` | ✅ Done | `<'src>` lifetime to struct |
| `src/token/graphql_trivia_token.rs` | ✅ Done | `<'src>` lifetime, Comment uses `Cow` |
| `src/token/mod.rs` | ✅ Done | Exports with lifetime |
| `src/graphql_token_stream.rs` | ✅ Done | `<'src>` lifetime parameter |
| `src/token_source/graphql_token_source.rs` | ✅ Done | Trait with lifetime |
| `src/token_source/str_to_graphql_token_source.rs` | ✅ Done | **Main implementation** (~1130 lines) |
| `src/token_source/mod.rs` | ✅ Done | Exports `StrGraphQLTokenSource` |
| `libgraphql-macros/src/rust_macro_graphql_token_source.rs` | ✅ Done | Uses helper constructors (`name_owned`, etc.) |
| `src/token_source/tests/str_to_graphql_token_source_tests.rs` | ✅ Done | ~2160 lines of unit tests |
| `src/tests/vendored/` | 🔲 TODO | Vendored tests from graphql-js and graphql-parser |
| `Cargo.toml` | N/A | Didn't add `unicode_names2` (hand-coded unicode char names instead) |
| `/crates/libgraphql-parser/benches/` | 🔲 TODO | Benchmark suite (Step 10) |
