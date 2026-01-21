use crate::GraphQLErrorNotes;
use crate::GraphQLStringParsingError;
use std::borrow::Cow;
use std::num::ParseFloatError;
use std::num::ParseIntError;

/// The kind of a GraphQL token.
///
/// Literal values (`IntValue`, `FloatValue`, `StringValue`) store only the raw
/// source text.
///
/// # Lifetime Parameter
///
/// The `'src` lifetime enables zero-copy lexing: `StrGraphQLTokenSource` can
/// borrow string slices directly from the source text using `Cow::Borrowed`,
/// while `RustMacroGraphQLTokenSource` uses `Cow::Owned` since `proc_macro2`
/// doesn't expose contiguous source text.
///
/// # Negative Numeric Literals
///
/// Negative numbers like `-123` are lexed as single tokens (e.g.
/// `IntValue("-123")`), not as separate minus and number tokens. This matches
/// the GraphQL spec's grammar for `IntValue`/`FloatValue`.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum GraphQLTokenKind<'src> {
    // =========================================================================
    // Punctuators (no allocation needed)
    // =========================================================================
    /// `&`
    Ampersand,
    /// `@`
    At,
    /// `!`
    Bang,
    /// `:`
    Colon,
    /// `}`
    CurlyBraceClose,
    /// `{`
    CurlyBraceOpen,
    /// `$`
    Dollar,
    /// `...`
    Ellipsis,
    /// `=`
    Equals,
    /// `)`
    ParenClose,
    /// `(`
    ParenOpen,
    /// `|`
    Pipe,
    /// `]`
    SquareBracketClose,
    /// `[`
    SquareBracketOpen,

    // =========================================================================
    // Literals (raw source text only)
    // =========================================================================
    /// A GraphQL name/identifier.
    ///
    /// Uses `Cow<'src, str>` to enable zero-copy lexing from string sources.
    Name(Cow<'src, str>),

    /// Raw source text of an integer literal, including optional negative sign
    /// (e.g. `"-123"`, `"0"`).
    ///
    /// Use `parse_int_value()` to parse the raw text into an `i64`.
    /// Uses `Cow<'src, str>` to enable zero-copy lexing from string sources.
    IntValue(Cow<'src, str>),

    /// Raw source text of a float literal, including optional negative sign
    /// (e.g. `"-1.23e-4"`, `"0.5"`).
    ///
    /// Use `parse_float_value()` to parse the raw text into an `f64`.
    /// Uses `Cow<'src, str>` to enable zero-copy lexing from string sources.
    FloatValue(Cow<'src, str>),

    /// Raw source text of a string literal, including quotes
    /// (e.g. `"\"hello\\nworld\""`, `"\"\"\"block\"\"\""`)
    ///
    /// Use `parse_string_value()` to process escape sequences and get the
    /// unescaped content.
    /// Uses `Cow<'src, str>` to enable zero-copy lexing from string sources.
    StringValue(Cow<'src, str>),

    // =========================================================================
    // Boolean and null (distinct from Name for type safety)
    // =========================================================================
    /// The `true` literal.
    True,
    /// The `false` literal.
    False,
    /// The `null` literal.
    Null,

    // =========================================================================
    // End of input
    // =========================================================================
    /// End of input. The associated `GraphQLToken` may carry trailing trivia.
    Eof,

    // =========================================================================
    // Lexer error (allows error recovery)
    // =========================================================================
    /// A lexer error. This allows the parser to continue and collect multiple
    /// errors in a single pass.
    ///
    /// TODO: Explore replacing error_notes with a richer diagnostics structure
    /// that includes things like severity level and "fix action" for IDE
    /// integration.
    Error {
        /// A human-readable error message.
        message: String,
        /// Optional notes providing additional context or suggestions.
        error_notes: GraphQLErrorNotes,
    },
}

impl<'src> GraphQLTokenKind<'src> {
    // =========================================================================
    // Helper constructors for creating token kinds
    // =========================================================================

    /// Create a `Name` token from a borrowed string slice (zero-copy).
    ///
    /// Use this in `StrGraphQLTokenSource` where the source text can be
    /// borrowed directly.
    #[inline]
    pub fn name_borrowed(s: &'src str) -> Self {
        GraphQLTokenKind::Name(Cow::Borrowed(s))
    }

    /// Create a `Name` token from an owned `String`.
    ///
    /// Use this in `RustMacroGraphQLTokenSource` where the string must be
    /// allocated (e.g., from `ident.to_string()`).
    #[inline]
    pub fn name_owned(s: String) -> Self {
        GraphQLTokenKind::Name(Cow::Owned(s))
    }

    /// Create an `IntValue` token from a borrowed string slice (zero-copy).
    #[inline]
    pub fn int_value_borrowed(s: &'src str) -> Self {
        GraphQLTokenKind::IntValue(Cow::Borrowed(s))
    }

    /// Create an `IntValue` token from an owned `String`.
    #[inline]
    pub fn int_value_owned(s: String) -> Self {
        GraphQLTokenKind::IntValue(Cow::Owned(s))
    }

    /// Create a `FloatValue` token from a borrowed string slice (zero-copy).
    #[inline]
    pub fn float_value_borrowed(s: &'src str) -> Self {
        GraphQLTokenKind::FloatValue(Cow::Borrowed(s))
    }

    /// Create a `FloatValue` token from an owned `String`.
    #[inline]
    pub fn float_value_owned(s: String) -> Self {
        GraphQLTokenKind::FloatValue(Cow::Owned(s))
    }

    /// Create a `StringValue` token from a borrowed string slice (zero-copy).
    #[inline]
    pub fn string_value_borrowed(s: &'src str) -> Self {
        GraphQLTokenKind::StringValue(Cow::Borrowed(s))
    }

    /// Create a `StringValue` token from an owned `String`.
    #[inline]
    pub fn string_value_owned(s: String) -> Self {
        GraphQLTokenKind::StringValue(Cow::Owned(s))
    }

    /// Create an `Error` token.
    ///
    /// Error messages are always dynamically constructed, so they use plain
    /// `String` rather than `Cow`.
    #[inline]
    pub fn error(message: impl Into<String>, error_notes: GraphQLErrorNotes) -> Self {
        GraphQLTokenKind::Error {
            message: message.into(),
            error_notes,
        }
    }

    // =========================================================================
    // Query methods
    // =========================================================================

    /// Returns `true` if this token is a punctuator.
    pub fn is_punctuator(&self) -> bool {
        match self {
            GraphQLTokenKind::Ampersand
            | GraphQLTokenKind::At
            | GraphQLTokenKind::Bang
            | GraphQLTokenKind::Colon
            | GraphQLTokenKind::CurlyBraceClose
            | GraphQLTokenKind::CurlyBraceOpen
            | GraphQLTokenKind::Dollar
            | GraphQLTokenKind::Ellipsis
            | GraphQLTokenKind::Equals
            | GraphQLTokenKind::ParenClose
            | GraphQLTokenKind::ParenOpen
            | GraphQLTokenKind::Pipe
            | GraphQLTokenKind::SquareBracketClose
            | GraphQLTokenKind::SquareBracketOpen => true,

            GraphQLTokenKind::Name(_)
            | GraphQLTokenKind::IntValue(_)
            | GraphQLTokenKind::FloatValue(_)
            | GraphQLTokenKind::StringValue(_)
            | GraphQLTokenKind::True
            | GraphQLTokenKind::False
            | GraphQLTokenKind::Null
            | GraphQLTokenKind::Eof
            | GraphQLTokenKind::Error { .. } => false,
        }
    }

    /// Returns the string representation of this token if it is a punctuator.
    pub fn as_punctuator_str(&self) -> Option<&'static str> {
        match self {
            GraphQLTokenKind::Ampersand => Some("&"),
            GraphQLTokenKind::At => Some("@"),
            GraphQLTokenKind::Bang => Some("!"),
            GraphQLTokenKind::Colon => Some(":"),
            GraphQLTokenKind::CurlyBraceClose => Some("}"),
            GraphQLTokenKind::CurlyBraceOpen => Some("{"),
            GraphQLTokenKind::Dollar => Some("$"),
            GraphQLTokenKind::Ellipsis => Some("..."),
            GraphQLTokenKind::Equals => Some("="),
            GraphQLTokenKind::ParenClose => Some(")"),
            GraphQLTokenKind::ParenOpen => Some("("),
            GraphQLTokenKind::Pipe => Some("|"),
            GraphQLTokenKind::SquareBracketClose => Some("]"),
            GraphQLTokenKind::SquareBracketOpen => Some("["),

            GraphQLTokenKind::Name(_)
            | GraphQLTokenKind::IntValue(_)
            | GraphQLTokenKind::FloatValue(_)
            | GraphQLTokenKind::StringValue(_)
            | GraphQLTokenKind::True
            | GraphQLTokenKind::False
            | GraphQLTokenKind::Null
            | GraphQLTokenKind::Eof
            | GraphQLTokenKind::Error { .. } => None,
        }
    }

    /// Returns `true` if this token is a value literal (`IntValue`,
    /// `FloatValue`, `StringValue`, `True`, `False`, or `Null`).
    pub fn is_value(&self) -> bool {
        match self {
            GraphQLTokenKind::IntValue(_)
            | GraphQLTokenKind::FloatValue(_)
            | GraphQLTokenKind::StringValue(_)
            | GraphQLTokenKind::True
            | GraphQLTokenKind::False
            | GraphQLTokenKind::Null => true,

            GraphQLTokenKind::Ampersand
            | GraphQLTokenKind::At
            | GraphQLTokenKind::Bang
            | GraphQLTokenKind::Colon
            | GraphQLTokenKind::CurlyBraceClose
            | GraphQLTokenKind::CurlyBraceOpen
            | GraphQLTokenKind::Dollar
            | GraphQLTokenKind::Ellipsis
            | GraphQLTokenKind::Equals
            | GraphQLTokenKind::ParenClose
            | GraphQLTokenKind::ParenOpen
            | GraphQLTokenKind::Pipe
            | GraphQLTokenKind::SquareBracketClose
            | GraphQLTokenKind::SquareBracketOpen
            | GraphQLTokenKind::Name(_)
            | GraphQLTokenKind::Eof
            | GraphQLTokenKind::Error { .. } => false,
        }
    }

    /// Returns `true` if this token represents a lexer error.
    pub fn is_error(&self) -> bool {
        matches!(self, GraphQLTokenKind::Error { .. })
    }

    /// Parse an `IntValue`'s raw text to `i64`.
    ///
    /// Returns `None` if this is not an `IntValue`, or `Some(Err(...))` if
    /// parsing fails.
    pub fn parse_int_value(&self) -> Option<Result<i64, ParseIntError>> {
        match self {
            GraphQLTokenKind::IntValue(raw) => Some(raw.parse()),
            _ => None,
        }
    }

    /// Parse a `FloatValue`'s raw text to `f64`.
    ///
    /// Returns `None` if this is not a `FloatValue`, or `Some(Err(...))` if
    /// parsing fails.
    pub fn parse_float_value(&self) -> Option<Result<f64, ParseFloatError>> {
        match self {
            GraphQLTokenKind::FloatValue(raw) => Some(raw.parse()),
            _ => None,
        }
    }

    /// Parse a `StringValue`'s raw text to unescaped content.
    ///
    /// Handles escape sequences per the GraphQL spec:
    /// - For single-line strings (`"..."`): processes `\n`, `\r`, `\t`, `\\`,
    ///   `\"`, `\/`, `\b`, `\f`, `\uXXXX` (fixed 4-digit), and `\u{X...}`
    ///   (variable length).
    /// - For block strings (`"""..."""`): applies the indentation stripping
    ///   algorithm per spec, then processes `\"""` escape only.
    ///
    /// Returns `None` if this is not a `StringValue`, or `Some(Err(...))` if
    /// parsing fails.
    pub fn parse_string_value(&self) -> Option<Result<String, GraphQLStringParsingError>> {
        match self {
            GraphQLTokenKind::StringValue(raw) => Some(parse_graphql_string(raw)),
            _ => None,
        }
    }
}

/// Parse a raw GraphQL string literal into its unescaped content.
fn parse_graphql_string(raw: &str) -> Result<String, GraphQLStringParsingError> {
    // Check if this is a block string
    if raw.starts_with("\"\"\"") {
        parse_block_string(raw)
    } else {
        parse_single_line_string(raw)
    }
}

/// Parse a single-line string literal.
fn parse_single_line_string(raw: &str) -> Result<String, GraphQLStringParsingError> {
    // Strip surrounding quotes
    if !raw.starts_with('"') || !raw.ends_with('"') || raw.len() < 2 {
        return Err(GraphQLStringParsingError::UnterminatedString);
    }
    let content = &raw[1..raw.len() - 1];

    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('/') => result.push('/'),
                Some('b') => result.push('\u{0008}'),
                Some('f') => result.push('\u{000C}'),
                Some('u') => {
                    let unicode_char = parse_unicode_escape(&mut chars)?;
                    result.push(unicode_char);
                }
                Some(other) => {
                    return Err(GraphQLStringParsingError::InvalidEscapeSequence(format!(
                        "\\{other}"
                    )));
                }
                None => {
                    return Err(GraphQLStringParsingError::InvalidEscapeSequence(
                        "\\".to_string(),
                    ));
                }
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

/// Parse a Unicode escape sequence after seeing `\u`.
fn parse_unicode_escape(
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<char, GraphQLStringParsingError> {
    // Check for variable-length syntax: \u{...}
    if chars.peek() == Some(&'{') {
        chars.next(); // consume '{'
        let mut hex = String::new();
        loop {
            match chars.next() {
                Some('}') => break,
                Some(c) if c.is_ascii_hexdigit() => hex.push(c),
                Some(c) => {
                    return Err(GraphQLStringParsingError::InvalidUnicodeEscape(format!(
                        "\\u{{{hex}{c}"
                    )));
                }
                None => {
                    return Err(GraphQLStringParsingError::InvalidUnicodeEscape(format!(
                        "\\u{{{hex}"
                    )));
                }
            }
        }
        if hex.is_empty() {
            return Err(GraphQLStringParsingError::InvalidUnicodeEscape(
                "\\u{}".to_string(),
            ));
        }
        let code_point = u32::from_str_radix(&hex, 16).map_err(|_| {
            GraphQLStringParsingError::InvalidUnicodeEscape(format!("\\u{{{hex}}}"))
        })?;
        char::from_u32(code_point).ok_or_else(|| {
            GraphQLStringParsingError::InvalidUnicodeEscape(format!("\\u{{{hex}}}"))
        })
    } else {
        // Fixed 4-digit syntax: \uXXXX
        let mut hex = String::with_capacity(4);
        for _ in 0..4 {
            match chars.next() {
                Some(c) if c.is_ascii_hexdigit() => hex.push(c),
                Some(c) => {
                    return Err(GraphQLStringParsingError::InvalidUnicodeEscape(format!(
                        "\\u{hex}{c}"
                    )));
                }
                None => {
                    return Err(GraphQLStringParsingError::InvalidUnicodeEscape(format!(
                        "\\u{hex}"
                    )));
                }
            }
        }
        let code_point = u32::from_str_radix(&hex, 16).map_err(|_| {
            GraphQLStringParsingError::InvalidUnicodeEscape(format!("\\u{hex}"))
        })?;
        char::from_u32(code_point).ok_or_else(|| {
            GraphQLStringParsingError::InvalidUnicodeEscape(format!("\\u{hex}"))
        })
    }
}

/// Parse a block string literal per the GraphQL spec.
fn parse_block_string(raw: &str) -> Result<String, GraphQLStringParsingError> {
    // Strip surrounding triple quotes
    if !raw.starts_with("\"\"\"") || !raw.ends_with("\"\"\"") || raw.len() < 6 {
        return Err(GraphQLStringParsingError::UnterminatedString);
    }
    let content = &raw[3..raw.len() - 3];

    // Handle escaped triple quotes
    let content = content.replace("\\\"\"\"", "\"\"\"");

    // Split into lines
    let lines: Vec<&str> = content.lines().collect();

    // Find common indentation (excluding first line and blank lines)
    let common_indent = lines
        .iter()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    // Build result with indentation stripped
    let mut result_lines: Vec<String> = Vec::with_capacity(lines.len());

    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            result_lines.push(line.to_string());
        } else if line.len() >= common_indent {
            result_lines.push(line[common_indent..].to_string());
        } else {
            result_lines.push(line.to_string());
        }
    }

    // Remove leading blank lines
    while result_lines.first().is_some_and(|l| l.trim().is_empty()) {
        result_lines.remove(0);
    }

    // Remove trailing blank lines
    while result_lines.last().is_some_and(|l| l.trim().is_empty()) {
        result_lines.pop();
    }

    Ok(result_lines.join("\n"))
}
