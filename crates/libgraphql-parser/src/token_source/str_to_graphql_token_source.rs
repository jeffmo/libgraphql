//! A [`GraphQLTokenSource`] that lexes from a `&str` input.
//!
//! This lexer implements zero-copy lexing: token values borrow directly from
//! the source string using `Cow::Borrowed`, avoiding allocations for names,
//! numbers, and strings.
//!
//! # Features
//!
//! - **Zero-copy lexing**: Token values borrow from source text when possible
//! - **Dual column tracking**: Reports both UTF-8 character positions (for
//!   display) and UTF-16 code unit positions (for LSP compatibility)
//! - **Comment preservation**: GraphQL `#` comments are captured as trivia
//! - **Error recovery**: Invalid characters emit `Error` tokens, allowing the
//!   lexer to continue and report multiple errors
//!
//! # Usage
//!
//! ```rust
//! use libgraphql_parser::token_source::StrGraphQLTokenSource;
//!
//! let source = "{ name }";
//! let lexer = StrGraphQLTokenSource::new(source);
//! for token in lexer {
//!     println!("{:?}", token.kind);
//! }
//! // Output:
//! // CurlyBraceOpen
//! // Name(Borrowed("name"))
//! // CurlyBraceClose
//! // Eof
//! ```

use crate::smallvec;
use crate::token::GraphQLToken;
use crate::token::GraphQLTokenKind;
use crate::token::GraphQLTriviaToken;
use crate::token::GraphQLTriviaTokenVec;
use crate::GraphQLErrorNote;
use crate::GraphQLSourceSpan;
use crate::SourcePosition;
use std::borrow::Cow;
use std::path::Path;

/// A [`GraphQLTokenSource`] that lexes from a `&str` input.
///
/// This lexer produces [`GraphQLToken`]s with zero-copy string values where
/// possible. The `'src` lifetime ties token values to the source string.
///
/// See module documentation for details.
pub struct StrGraphQLTokenSource<'src> {
    /// The full source text being lexed.
    source: &'src str,

    /// Current byte offset from the start of `source`.
    ///
    /// The remaining text to lex is `&source[curr_byte_offset..]`.
    curr_byte_offset: usize,

    /// Current 0-based line number.
    curr_line: usize,

    /// Current UTF-8 character column (0-based).
    ///
    /// This counts characters, not bytes. For example, "🎉" (4 bytes) advances
    /// this by 1.
    curr_col_utf8: usize,

    /// Current UTF-16 code unit column (0-based).
    ///
    /// Characters outside the Basic Multilingual Plane (U+10000 and above)
    /// advance this by 2 (surrogate pair). For example, "🎉" (U+1F389) advances
    /// this by 2.
    curr_col_utf16: usize,

    /// Whether the previous character was `\r`.
    ///
    /// Used to handle `\r\n` as a single newline: when we see `\r`, we set
    /// this flag; if the next character is `\n`, we skip it without
    /// incrementing the line number again.
    last_char_was_cr: bool,

    /// Trivia (comments, commas) accumulated before the next token.
    pending_trivia: GraphQLTriviaTokenVec<'src>,

    /// Whether the EOF token has been emitted.
    finished: bool,

    /// Optional file path for error messages and spans.
    ///
    /// When present, this is included in `GraphQLSourceSpan::file_path`.
    /// Borrowed from the caller to avoid allocation.
    file_path: Option<&'src Path>,
}

impl<'src> StrGraphQLTokenSource<'src> {
    /// Creates a new token source from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use libgraphql_parser::token_source::StrGraphQLTokenSource;
    /// let lexer = StrGraphQLTokenSource::new("{ name }");
    /// ```
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            curr_byte_offset: 0,
            curr_line: 0,
            curr_col_utf8: 0,
            curr_col_utf16: 0,
            last_char_was_cr: false,
            pending_trivia: smallvec![],
            finished: false,
            file_path: None,
        }
    }

    /// Creates a new token source with an associated file path.
    ///
    /// The file path is included in token spans for error reporting.
    pub fn with_file_path(source: &'src str, path: &'src Path) -> Self {
        Self {
            source,
            curr_byte_offset: 0,
            curr_line: 0,
            curr_col_utf8: 0,
            curr_col_utf16: 0,
            last_char_was_cr: false,
            pending_trivia: smallvec![],
            finished: false,
            file_path: Some(path),
        }
    }

    // =========================================================================
    // Position and scanning helpers
    // =========================================================================

    /// Returns the remaining source text to be lexed.
    fn remaining(&self) -> &'src str {
        &self.source[self.curr_byte_offset..]
    }

    /// Returns the current source position.
    fn curr_position(&self) -> SourcePosition {
        SourcePosition::new(
            self.curr_line,
            self.curr_col_utf8,
            Some(self.curr_col_utf16),
            self.curr_byte_offset,
        )
    }

    /// Peeks at the next character without consuming it.
    ///
    /// Returns `None` if at end of input.
    fn peek_char(&self) -> Option<char> {
        self.peek_char_nth(0)
    }

    /// Peeks at the nth character ahead without consuming.
    ///
    /// `peek_char_nth(0)` is equivalent to `peek_char()`.
    /// Returns `None` if there aren't enough characters remaining.
    fn peek_char_nth(&self, n: usize) -> Option<char> {
        self.remaining().chars().nth(n)
    }

    /// Consumes the next character and updates position tracking.
    ///
    /// Returns `None` if at end of input.
    ///
    /// This method handles:
    /// - Advancing byte offset by the character's UTF-8 length
    /// - Incrementing line number on newlines (`\n`, `\r`, `\r\n`)
    /// - Tracking UTF-8 character column and UTF-16 code unit column
    fn consume(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        let byte_len = ch.len_utf8();

        // Handle newlines
        if ch == '\n' {
            if self.last_char_was_cr {
                // This is the \n of a \r\n pair - we already incremented line
                // when we saw \r. Just reset the flag.
                self.last_char_was_cr = false;
            } else {
                // Regular \n newline
                self.curr_line += 1;
                self.curr_col_utf8 = 0;
                self.curr_col_utf16 = 0;
            }
        } else if ch == '\r' {
            // Carriage return - treat as newline
            self.curr_line += 1;
            self.curr_col_utf8 = 0;
            self.curr_col_utf16 = 0;
            self.last_char_was_cr = true;
        } else {
            // Regular character - advance columns
            self.curr_col_utf8 += 1;
            self.curr_col_utf16 += ch.len_utf16();
            self.last_char_was_cr = false;
        }

        self.curr_byte_offset += byte_len;
        Some(ch)
    }

    /// Creates a `GraphQLSourceSpan` from a start position to the current
    /// position.
    fn make_span(&self, start: SourcePosition) -> GraphQLSourceSpan {
        let end = self.curr_position();
        if let Some(path) = self.file_path {
            GraphQLSourceSpan::with_file(start, end, path.to_path_buf())
        } else {
            GraphQLSourceSpan::new(start, end)
        }
    }

    // =========================================================================
    // Token creation helpers
    // =========================================================================

    /// Creates a token with the accumulated trivia.
    fn make_token(
        &mut self,
        kind: GraphQLTokenKind<'src>,
        span: GraphQLSourceSpan,
    ) -> GraphQLToken<'src> {
        GraphQLToken {
            kind,
            preceding_trivia: std::mem::take(&mut self.pending_trivia),
            span,
        }
    }

    // =========================================================================
    // Lexer main loop
    // =========================================================================

    /// Advances to the next token, skipping whitespace and collecting trivia.
    fn next_token(&mut self) -> GraphQLToken<'src> {
        loop {
            // Skip whitespace
            self.skip_whitespace();

            let start = self.curr_position();

            match self.peek_char() {
                None => {
                    // End of input
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::Eof, span);
                }

                Some('#') => {
                    // Comment - collect as trivia and continue
                    self.lex_comment(start);
                    continue;
                }

                Some(',') => {
                    // Comma - collect as trivia and continue
                    self.consume();
                    let span = self.make_span(start);
                    self.pending_trivia
                        .push(GraphQLTriviaToken::Comma { span });
                    continue;
                }

                // Single-character punctuators
                Some('!') => {
                    self.consume();
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::Bang, span);
                }
                Some('$') => {
                    self.consume();
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::Dollar, span);
                }
                Some('&') => {
                    self.consume();
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::Ampersand, span);
                }
                Some('(') => {
                    self.consume();
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::ParenOpen, span);
                }
                Some(')') => {
                    self.consume();
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::ParenClose, span);
                }
                Some(':') => {
                    self.consume();
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::Colon, span);
                }
                Some('=') => {
                    self.consume();
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::Equals, span);
                }
                Some('@') => {
                    self.consume();
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::At, span);
                }
                Some('[') => {
                    self.consume();
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::SquareBracketOpen, span);
                }
                Some(']') => {
                    self.consume();
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::SquareBracketClose, span);
                }
                Some('{') => {
                    self.consume();
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::CurlyBraceOpen, span);
                }
                Some('}') => {
                    self.consume();
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::CurlyBraceClose, span);
                }
                Some('|') => {
                    self.consume();
                    let span = self.make_span(start);
                    return self.make_token(GraphQLTokenKind::Pipe, span);
                }

                // Ellipsis or dot error
                Some('.') => {
                    return self.lex_dot_or_ellipsis(start);
                }

                // String literals
                Some('"') => {
                    return self.lex_string(start);
                }

                // Names and keywords
                Some(c) if is_name_start(c) => {
                    return self.lex_name(start);
                }

                // Numbers (including negative)
                Some(c) if c == '-' || c.is_ascii_digit() => {
                    return self.lex_number(start);
                }

                // Invalid character
                Some(_) => {
                    return self.lex_invalid_character(start);
                }
            }
        }
    }

    // =========================================================================
    // Whitespace handling
    // =========================================================================

    /// Skips whitespace characters.
    ///
    /// Per the GraphQL spec, these are "ignored tokens":
    /// - Space (U+0020)
    /// - Tab (U+0009)
    /// - Line terminators: LF (U+000A), CR (U+000D), CRLF
    /// - BOM (U+FEFF) - Unicode BOM is ignored anywhere in the document
    ///
    /// See: <https://spec.graphql.org/September2025/#sec-Language.Source-Text.Unicode>
    ///
    /// Note: Comma is also whitespace in GraphQL but we handle it separately
    /// to preserve it as trivia.
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            match ch {
                ' ' | '\t' | '\n' | '\r' | '\u{FEFF}' => {
                    self.consume();
                }
                _ => break,
            }
        }
    }

    // =========================================================================
    // Comment lexing
    // =========================================================================

    /// Lexes a comment and adds it to pending trivia.
    ///
    /// A comment starts with `#` and extends to the end of the line.
    fn lex_comment(&mut self, start: SourcePosition) {
        // Consume the '#'
        self.consume();
        let content_start = self.curr_byte_offset;

        // Consume until end of line or EOF
        while let Some(ch) = self.peek_char() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.consume();
        }

        let content_end = self.curr_byte_offset;
        let content = &self.source[content_start..content_end];
        let span = self.make_span(start);

        self.pending_trivia.push(GraphQLTriviaToken::Comment {
            value: Cow::Borrowed(content),
            span,
        });
    }

    // =========================================================================
    // Dot / Ellipsis lexing
    // =========================================================================

    /// Lexes dots, producing either an Ellipsis token or an error.
    ///
    /// This implements a state machine for dot handling similar to
    /// `RustMacroGraphQLTokenSource`:
    /// - `...` (adjacent) → `Ellipsis`
    /// - `.` alone → Error (no hint - could be many things like `Foo.Bar`)
    /// - `..` (adjacent) → Error with help to add third dot
    /// - `. .` (spaced, same line) → Error with help about spacing
    /// - `.. .` (first two adjacent, third spaced) → Error with help about
    ///   spacing
    /// - `. ..` (first spaced, last two adjacent) → Error with help about
    ///   spacing
    /// - `. . .` (all spaced, same line) → Error with help about spacing
    /// - Dots on different lines → Separate errors
    ///
    /// TODO: Look for patterns like `{Name}.{Name}` and give a useful error
    /// hint (e.g., user may have been trying to use enum syntax incorrectly).
    fn lex_dot_or_ellipsis(&mut self, start: SourcePosition) -> GraphQLToken<'src> {
        let first_dot_line = self.curr_line;

        // Consume first dot
        self.consume();

        // Check for second dot (may be adjacent or spaced)
        self.skip_whitespace_same_line();

        match self.peek_char() {
            Some('.') if self.curr_line == first_dot_line => {
                let second_dot_start = self.curr_position();
                let first_two_adjacent = second_dot_start.byte_offset() == start.byte_offset() + 1;
                self.consume();

                // Check for third dot
                self.skip_whitespace_same_line();

                match self.peek_char() {
                    Some('.') if self.curr_line == first_dot_line => {
                        let third_dot_start = self.curr_position();
                        self.consume();
                        let span = self.make_span(start);

                        // Check if all three dots were adjacent (no whitespace)
                        let second_third_adjacent =
                            third_dot_start.byte_offset() == second_dot_start.byte_offset() + 1;

                        if first_two_adjacent && second_third_adjacent {
                            // All adjacent - valid ellipsis
                            self.make_token(GraphQLTokenKind::Ellipsis, span)
                        } else if first_two_adjacent {
                            // `.. .` - first two adjacent, third spaced
                            let kind = GraphQLTokenKind::Error {
                                message: "Unexpected `.. .`".to_string(),
                                error_notes: smallvec![GraphQLErrorNote::help(
                                    "This `.` may have been intended to complete a `...` spread \
                                     operator. Try removing the extra spacing between the dots."
                                )],
                            };
                            self.make_token(kind, span)
                        } else if second_third_adjacent {
                            // `. ..` - first spaced, last two adjacent
                            let kind = GraphQLTokenKind::Error {
                                message: "Unexpected `. ..`".to_string(),
                                error_notes: smallvec![GraphQLErrorNote::help(
                                    "These dots may have been intended to form a `...` spread \
                                     operator. Try removing the extra spacing between the dots."
                                )],
                            };
                            self.make_token(kind, span)
                        } else {
                            // `. . .` - all spaced
                            let kind = GraphQLTokenKind::Error {
                                message: "Unexpected `. . .`".to_string(),
                                error_notes: smallvec![GraphQLErrorNote::help(
                                    "These dots may have been intended to form a `...` spread \
                                     operator. Try removing the extra spacing between the dots."
                                )],
                            };
                            self.make_token(kind, span)
                        }
                    }
                    _ => {
                        // Only two dots found on this line
                        let span = self.make_span(start);

                        if first_two_adjacent {
                            // Adjacent `..` - suggest adding third dot
                            let kind = GraphQLTokenKind::Error {
                                message: "Unexpected `..` (use `...` for spread operator)"
                                    .to_string(),
                                error_notes: smallvec![GraphQLErrorNote::help(
                                    "Add one more `.` to form the spread operator `...`"
                                )],
                            };
                            self.make_token(kind, span)
                        } else {
                            // Spaced `. .` - suggest removing spacing
                            let kind = GraphQLTokenKind::Error {
                                message: "Unexpected `. .` (use `...` for spread operator)"
                                    .to_string(),
                                error_notes: smallvec![GraphQLErrorNote::help(
                                    "These dots may have been intended to form a `...` spread \
                                     operator. Try removing the extra spacing between the dots."
                                )],
                            };
                            self.make_token(kind, span)
                        }
                    }
                }
            }
            _ => {
                // Single dot (or dots on different lines)
                // Don't assume it was meant to be ellipsis - could be `Foo.Bar` style
                let span = self.make_span(start);
                let kind = GraphQLTokenKind::Error {
                    message: "Unexpected `.`".to_string(),
                    error_notes: smallvec![],
                };
                self.make_token(kind, span)
            }
        }
    }

    /// Skips whitespace but only on the same line.
    ///
    /// Used for dot consolidation - we only merge dots that are on the same
    /// line.
    fn skip_whitespace_same_line(&mut self) {
        while let Some(ch) = self.peek_char() {
            match ch {
                ' ' | '\t' | '\u{FEFF}' => {
                    self.consume();
                }
                _ => break,
            }
        }
    }

    // =========================================================================
    // Name lexing
    // =========================================================================

    /// Lexes a name or keyword.
    ///
    /// Names match the pattern: `/[_A-Za-z][_0-9A-Za-z]*/`
    ///
    /// Keywords `true`, `false`, and `null` are emitted as distinct token
    /// kinds.
    fn lex_name(&mut self, start: SourcePosition) -> GraphQLToken<'src> {
        let name_start = self.curr_byte_offset;

        // Consume the first character (already validated as name start)
        self.consume();

        // Consume remaining name characters
        while let Some(ch) = self.peek_char() {
            if is_name_continue(ch) {
                self.consume();
            } else {
                break;
            }
        }

        let name_end = self.curr_byte_offset;
        let name = &self.source[name_start..name_end];
        let span = self.make_span(start);

        // Check for keywords
        let kind = match name {
            "true" => GraphQLTokenKind::True,
            "false" => GraphQLTokenKind::False,
            "null" => GraphQLTokenKind::Null,
            _ => GraphQLTokenKind::name_borrowed(name),
        };

        self.make_token(kind, span)
    }

    // =========================================================================
    // Number lexing
    // =========================================================================

    /// Lexes an integer or float literal.
    ///
    /// Handles:
    /// - Optional negative sign: `-`
    /// - Integer part: `0` or `[1-9][0-9]*`
    /// - Optional decimal part: `.[0-9]+`
    /// - Optional exponent: `[eE][+-]?[0-9]+`
    fn lex_number(&mut self, start: SourcePosition) -> GraphQLToken<'src> {
        let num_start = self.curr_byte_offset;
        let mut is_float = false;

        // Optional negative sign
        if self.peek_char() == Some('-') {
            self.consume();
        }

        // Integer part
        match self.peek_char() {
            Some('0') => {
                self.consume();
                // Check for invalid leading zeros (e.g., 00, 01)
                if let Some(ch) = self.peek_char()
                    && ch.is_ascii_digit() {
                    // Invalid: leading zeros
                    return self.lex_number_error(
                        start,
                        num_start,
                        "Invalid number: leading zeros are not allowed",
                        Some("https://spec.graphql.org/September2025/#sec-Int-Value"),
                    );
                }
            }
            Some(ch) if ch.is_ascii_digit() => {
                // Non-zero start
                self.consume();
                while let Some(ch) = self.peek_char() {
                    if ch.is_ascii_digit() {
                        self.consume();
                    } else {
                        break;
                    }
                }
            }
            Some(_) | None => {
                // Just a `-` with no digits
                let span = self.make_span(start);
                let kind = GraphQLTokenKind::Error {
                    message: "Unexpected `-`".to_string(),
                    error_notes: smallvec![],
                };
                return self.make_token(kind, span);
            }
        }

        // Optional decimal part
        if self.peek_char() == Some('.') {
            // Check that the next character is a digit (not another dot for `...`)
            if let Some(ch) = self.peek_char_nth(1)
                && ch.is_ascii_digit() {
                is_float = true;
                self.consume(); // consume the '.'

                // Consume decimal digits
                while let Some(ch) = self.peek_char() {
                    if ch.is_ascii_digit() {
                        self.consume();
                    } else {
                        break;
                    }
                }
            }
        }

        // Optional exponent part
        if let Some(ch) = self.peek_char()
            && (ch == 'e' || ch == 'E') {
            is_float = true;
            self.consume();

            // Optional sign
            if let Some(ch) = self.peek_char()
                && (ch == '+' || ch == '-') {
                self.consume();
            }

            // Exponent digits (required)
            let has_exponent_digits = matches!(self.peek_char(), Some(ch) if ch.is_ascii_digit());
            if !has_exponent_digits {
                return self.lex_number_error(
                    start,
                    num_start,
                    "Invalid number: exponent must have at least one digit",
                    Some("https://spec.graphql.org/September2025/#sec-Float-Value"),
                );
            }

            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    self.consume();
                } else {
                    break;
                }
            }
        }

        let num_end = self.curr_byte_offset;
        let num_text = &self.source[num_start..num_end];
        let span = self.make_span(start);

        let kind = if is_float {
            GraphQLTokenKind::float_value_borrowed(num_text)
        } else {
            GraphQLTokenKind::int_value_borrowed(num_text)
        };

        self.make_token(kind, span)
    }

    /// Creates an error token for an invalid number.
    fn lex_number_error(
        &mut self,
        start: SourcePosition,
        num_start: usize,
        message: &str,
        spec_url: Option<&str>,
    ) -> GraphQLToken<'src> {
        // Consume remaining number-like characters to provide better error recovery
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' || ch == '+' || ch == '-'
            {
                self.consume();
            } else {
                break;
            }
        }

        let num_end = self.curr_byte_offset;
        let invalid_text = &self.source[num_start..num_end];
        let span = self.make_span(start);

        let mut error_notes = smallvec![];
        if let Some(url) = spec_url {
            error_notes.push(GraphQLErrorNote::spec(url));
        }

        let kind = GraphQLTokenKind::Error {
            message: format!("{message}: `{invalid_text}`"),
            error_notes,
        };

        self.make_token(kind, span)
    }

    // =========================================================================
    // String lexing
    // =========================================================================

    /// Lexes a string literal (single-line or block string).
    fn lex_string(&mut self, start: SourcePosition) -> GraphQLToken<'src> {
        let str_start = self.curr_byte_offset;

        // Check for block string (""")
        if self.remaining().starts_with("\"\"\"") {
            return self.lex_block_string(start, str_start);
        }

        // Single-line string
        self.consume(); // consume opening "

        loop {
            match self.peek_char() {
                None => {
                    // Unterminated string
                    let span = self.make_span(start.clone());
                    let kind = GraphQLTokenKind::Error {
                        message: "Unterminated string literal".to_string(),
                        error_notes: smallvec![
                            GraphQLErrorNote::general_with_span(
                                "String started here",
                                self.make_span(start),
                            ),
                            GraphQLErrorNote::help("Add closing `\"`"),
                        ],
                    };
                    return self.make_token(kind, span);
                }
                Some('\n') | Some('\r') => {
                    // Unescaped newline in single-line string - consume it so
                    // the span includes the newline character
                    self.consume();
                    // Also consume \n if this was \r\n
                    if self.last_char_was_cr && self.peek_char() == Some('\n') {
                        self.consume();
                    }
                    let span = self.make_span(start);
                    let kind = GraphQLTokenKind::Error {
                        message: "Unterminated string literal".to_string(),
                        error_notes: smallvec![
                            GraphQLErrorNote::general(
                                "Single-line strings cannot contain unescaped newlines"
                            ),
                            GraphQLErrorNote::help("Use a block string (triple quotes) for multi-line strings, or escape the newline with `\\n`"),
                        ],
                    };
                    return self.make_token(kind, span);
                }
                Some('"') => {
                    // End of string
                    self.consume();
                    break;
                }
                Some('\\') => {
                    // Escape sequence - consume backslash and next character
                    self.consume();
                    if self.peek_char().is_some() {
                        self.consume();
                    }
                }
                Some(_) => {
                    self.consume();
                }
            }
        }

        let str_end = self.curr_byte_offset;
        let string_text = &self.source[str_start..str_end];
        let span = self.make_span(start);

        self.make_token(GraphQLTokenKind::string_value_borrowed(string_text), span)
    }

    /// Lexes a block string literal.
    fn lex_block_string(&mut self, start: SourcePosition, str_start: usize) -> GraphQLToken<'src> {
        // Consume opening """
        self.consume(); // first "
        self.consume(); // second "
        self.consume(); // third "

        loop {
            match self.peek_char() {
                None => {
                    // Unterminated block string
                    let span = self.make_span(start.clone());
                    let kind = GraphQLTokenKind::Error {
                        message: "Unterminated block string".to_string(),
                        error_notes: smallvec![
                            GraphQLErrorNote::general_with_span(
                                "Block string started here",
                                self.make_span(start),
                            ),
                            GraphQLErrorNote::help("Add closing `\"\"\"`"),
                        ],
                    };
                    return self.make_token(kind, span);
                }
                Some('\\') => {
                    // Check for escaped triple quote: \"""
                    if self.remaining().starts_with("\\\"\"\"") {
                        self.consume(); // backslash
                        self.consume(); // first "
                        self.consume(); // second "
                        self.consume(); // third "
                    } else {
                        self.consume();
                    }
                }
                Some('"') => {
                    // Check for closing """
                    if self.remaining().starts_with("\"\"\"") {
                        self.consume(); // first "
                        self.consume(); // second "
                        self.consume(); // third "
                        break;
                    } else {
                        self.consume();
                    }
                }
                Some(_) => {
                    self.consume();
                }
            }
        }

        let str_end = self.curr_byte_offset;
        let string_text = &self.source[str_start..str_end];
        let span = self.make_span(start);

        self.make_token(GraphQLTokenKind::string_value_borrowed(string_text), span)
    }

    // =========================================================================
    // Invalid character handling
    // =========================================================================

    /// Lexes an invalid character, producing an error token.
    fn lex_invalid_character(&mut self, start: SourcePosition) -> GraphQLToken<'src> {
        let ch = self.consume().unwrap();
        let span = self.make_span(start);

        let kind = GraphQLTokenKind::Error {
            message: format!("Unexpected character {}", describe_char(ch)),
            error_notes: smallvec![],
        };

        self.make_token(kind, span)
    }
}

// =============================================================================
// Iterator implementation
// =============================================================================

impl<'src> Iterator for StrGraphQLTokenSource<'src> {
    type Item = GraphQLToken<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let token = self.next_token();

        if matches!(token.kind, GraphQLTokenKind::Eof) {
            self.finished = true;
        }

        Some(token)
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Returns `true` if `ch` can start a GraphQL name.
///
/// Per the GraphQL spec, names start with `NameStart`:
/// <https://spec.graphql.org/September2025/#NameStart>
fn is_name_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

/// Returns `true` if `ch` can continue a GraphQL name.
///
/// Per the GraphQL spec, names continue with `NameContinue`:
/// <https://spec.graphql.org/September2025/#NameContinue>
fn is_name_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

/// Returns a human-readable description of a character for error messages.
///
/// For printable characters, returns the character in backticks.
/// For invisible/control characters, includes Unicode code point description.
fn describe_char(ch: char) -> String {
    if ch.is_control() || (ch.is_whitespace() && ch != ' ') {
        // Invisible characters get detailed description
        let name = unicode_char_name(ch);
        if let Some(name) = name {
            format!("`{}` (U+{:04X}: {})", ch, ch as u32, name)
        } else {
            format!("`{}` (U+{:04X})", ch, ch as u32)
        }
    } else {
        format!("`{ch}`")
    }
}

/// Returns the Unicode name for well-known invisible/control characters.
///
/// This provides meaningful names for commonly encountered invisible
/// characters. Returns `None` for characters without a known name.
fn unicode_char_name(ch: char) -> Option<&'static str> {
    match ch {
        // C0 control characters (U+0000 - U+001F)
        '\u{0000}' => Some("NULL"),
        '\u{0001}' => Some("START OF HEADING"),
        '\u{0002}' => Some("START OF TEXT"),
        '\u{0003}' => Some("END OF TEXT"),
        '\u{0004}' => Some("END OF TRANSMISSION"),
        '\u{0005}' => Some("ENQUIRY"),
        '\u{0006}' => Some("ACKNOWLEDGE"),
        '\u{0007}' => Some("BELL"),
        '\u{0008}' => Some("BACKSPACE"),
        '\u{0009}' => Some("HORIZONTAL TAB"),
        '\u{000A}' => Some("LINE FEED"),
        '\u{000B}' => Some("VERTICAL TAB"),
        '\u{000C}' => Some("FORM FEED"),
        '\u{000D}' => Some("CARRIAGE RETURN"),
        '\u{000E}' => Some("SHIFT OUT"),
        '\u{000F}' => Some("SHIFT IN"),
        '\u{0010}' => Some("DATA LINK ESCAPE"),
        '\u{0011}' => Some("DEVICE CONTROL ONE"),
        '\u{0012}' => Some("DEVICE CONTROL TWO"),
        '\u{0013}' => Some("DEVICE CONTROL THREE"),
        '\u{0014}' => Some("DEVICE CONTROL FOUR"),
        '\u{0015}' => Some("NEGATIVE ACKNOWLEDGE"),
        '\u{0016}' => Some("SYNCHRONOUS IDLE"),
        '\u{0017}' => Some("END OF TRANSMISSION BLOCK"),
        '\u{0018}' => Some("CANCEL"),
        '\u{0019}' => Some("END OF MEDIUM"),
        '\u{001A}' => Some("SUBSTITUTE"),
        '\u{001B}' => Some("ESCAPE"),
        '\u{001C}' => Some("FILE SEPARATOR"),
        '\u{001D}' => Some("GROUP SEPARATOR"),
        '\u{001E}' => Some("RECORD SEPARATOR"),
        '\u{001F}' => Some("UNIT SEPARATOR"),

        // C1 control characters and special (U+007F - U+00A0)
        '\u{007F}' => Some("DELETE"),
        '\u{0080}' => Some("PADDING CHARACTER"),
        '\u{0081}' => Some("HIGH OCTET PRESET"),
        '\u{0082}' => Some("BREAK PERMITTED HERE"),
        '\u{0083}' => Some("NO BREAK HERE"),
        '\u{0084}' => Some("INDEX"),
        '\u{0085}' => Some("NEXT LINE"),
        '\u{0086}' => Some("START OF SELECTED AREA"),
        '\u{0087}' => Some("END OF SELECTED AREA"),
        '\u{0088}' => Some("CHARACTER TABULATION SET"),
        '\u{0089}' => Some("CHARACTER TABULATION WITH JUSTIFICATION"),
        '\u{008A}' => Some("LINE TABULATION SET"),
        '\u{008B}' => Some("PARTIAL LINE FORWARD"),
        '\u{008C}' => Some("PARTIAL LINE BACKWARD"),
        '\u{008D}' => Some("REVERSE LINE FEED"),
        '\u{008E}' => Some("SINGLE SHIFT TWO"),
        '\u{008F}' => Some("SINGLE SHIFT THREE"),
        '\u{0090}' => Some("DEVICE CONTROL STRING"),
        '\u{0091}' => Some("PRIVATE USE ONE"),
        '\u{0092}' => Some("PRIVATE USE TWO"),
        '\u{0093}' => Some("SET TRANSMIT STATE"),
        '\u{0094}' => Some("CANCEL CHARACTER"),
        '\u{0095}' => Some("MESSAGE WAITING"),
        '\u{0096}' => Some("START OF GUARDED AREA"),
        '\u{0097}' => Some("END OF GUARDED AREA"),
        '\u{0098}' => Some("START OF STRING"),
        '\u{0099}' => Some("SINGLE GRAPHIC CHARACTER INTRODUCER"),
        '\u{009A}' => Some("SINGLE CHARACTER INTRODUCER"),
        '\u{009B}' => Some("CONTROL SEQUENCE INTRODUCER"),
        '\u{009C}' => Some("STRING TERMINATOR"),
        '\u{009D}' => Some("OPERATING SYSTEM COMMAND"),
        '\u{009E}' => Some("PRIVACY MESSAGE"),
        '\u{009F}' => Some("APPLICATION PROGRAM COMMAND"),
        '\u{00A0}' => Some("NO-BREAK SPACE"),
        '\u{00AD}' => Some("SOFT HYPHEN"),

        // General punctuation - spaces (U+2000 - U+200A)
        '\u{2000}' => Some("EN QUAD"),
        '\u{2001}' => Some("EM QUAD"),
        '\u{2002}' => Some("EN SPACE"),
        '\u{2003}' => Some("EM SPACE"),
        '\u{2004}' => Some("THREE-PER-EM SPACE"),
        '\u{2005}' => Some("FOUR-PER-EM SPACE"),
        '\u{2006}' => Some("SIX-PER-EM SPACE"),
        '\u{2007}' => Some("FIGURE SPACE"),
        '\u{2008}' => Some("PUNCTUATION SPACE"),
        '\u{2009}' => Some("THIN SPACE"),
        '\u{200A}' => Some("HAIR SPACE"),

        // Zero-width and formatting characters (U+200B - U+200F)
        '\u{200B}' => Some("ZERO WIDTH SPACE"),
        '\u{200C}' => Some("ZERO WIDTH NON-JOINER"),
        '\u{200D}' => Some("ZERO WIDTH JOINER"),
        '\u{200E}' => Some("LEFT-TO-RIGHT MARK"),
        '\u{200F}' => Some("RIGHT-TO-LEFT MARK"),

        // Bidirectional text formatting (U+202A - U+202F)
        '\u{202A}' => Some("LEFT-TO-RIGHT EMBEDDING"),
        '\u{202B}' => Some("RIGHT-TO-LEFT EMBEDDING"),
        '\u{202C}' => Some("POP DIRECTIONAL FORMATTING"),
        '\u{202D}' => Some("LEFT-TO-RIGHT OVERRIDE"),
        '\u{202E}' => Some("RIGHT-TO-LEFT OVERRIDE"),
        '\u{202F}' => Some("NARROW NO-BREAK SPACE"),

        // More formatting (U+2060 - U+206F)
        '\u{2060}' => Some("WORD JOINER"),
        '\u{2061}' => Some("FUNCTION APPLICATION"),
        '\u{2062}' => Some("INVISIBLE TIMES"),
        '\u{2063}' => Some("INVISIBLE SEPARATOR"),
        '\u{2064}' => Some("INVISIBLE PLUS"),
        '\u{2066}' => Some("LEFT-TO-RIGHT ISOLATE"),
        '\u{2067}' => Some("RIGHT-TO-LEFT ISOLATE"),
        '\u{2068}' => Some("FIRST STRONG ISOLATE"),
        '\u{2069}' => Some("POP DIRECTIONAL ISOLATE"),
        '\u{206A}' => Some("INHIBIT SYMMETRIC SWAPPING"),
        '\u{206B}' => Some("ACTIVATE SYMMETRIC SWAPPING"),
        '\u{206C}' => Some("INHIBIT ARABIC FORM SHAPING"),
        '\u{206D}' => Some("ACTIVATE ARABIC FORM SHAPING"),
        '\u{206E}' => Some("NATIONAL DIGIT SHAPES"),
        '\u{206F}' => Some("NOMINAL DIGIT SHAPES"),

        // Other special spaces
        '\u{2028}' => Some("LINE SEPARATOR"),
        '\u{2029}' => Some("PARAGRAPH SEPARATOR"),
        '\u{205F}' => Some("MEDIUM MATHEMATICAL SPACE"),
        '\u{3000}' => Some("IDEOGRAPHIC SPACE"),

        // Special characters
        '\u{034F}' => Some("COMBINING GRAPHEME JOINER"),
        '\u{061C}' => Some("ARABIC LETTER MARK"),
        '\u{115F}' => Some("HANGUL CHOSEONG FILLER"),
        '\u{1160}' => Some("HANGUL JUNGSEONG FILLER"),
        '\u{17B4}' => Some("KHMER VOWEL INHERENT AQ"),
        '\u{17B5}' => Some("KHMER VOWEL INHERENT AA"),
        '\u{180E}' => Some("MONGOLIAN VOWEL SEPARATOR"),

        // BOM and special
        '\u{FEFF}' => Some("BYTE ORDER MARK"),
        '\u{FFFE}' => Some("NONCHARACTER"),
        '\u{FFFF}' => Some("NONCHARACTER"),

        // Interlinear annotation
        '\u{FFF9}' => Some("INTERLINEAR ANNOTATION ANCHOR"),
        '\u{FFFA}' => Some("INTERLINEAR ANNOTATION SEPARATOR"),
        '\u{FFFB}' => Some("INTERLINEAR ANNOTATION TERMINATOR"),

        // Tag characters (U+E0000 - U+E007F)
        '\u{E0001}' => Some("LANGUAGE TAG"),
        '\u{E0020}' => Some("TAG SPACE"),

        _ => None,
    }
}
