use crate::DefinitionKind;
use crate::DocumentKind;
use crate::ReservedNameContext;
use crate::ValueParsingError;

/// Categorizes parse errors for programmatic handling.
///
/// Each variant contains minimal data needed for programmatic decisions.
/// Human-readable context (suggestions, explanations) belongs in the
/// `notes` field of `GraphQLParseError`.
///
/// The `#[error(...)]` messages are concise/programmatic. Full human-readable
/// messages are in `GraphQLParseError.message`.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GraphQLParseErrorKind {
    /// Expected specific token(s) but found something else.
    ///
    /// This is the most common error type — the parser expected certain tokens
    /// based on grammar rules but encountered something unexpected.
    ///
    /// # Example
    /// ```text
    /// type User { name String }
    ///                  ^^^^^^ expected `:`, found `String`
    /// ```
    #[error("unexpected token: `{found}`")]
    UnexpectedToken {
        /// What tokens were expected (e.g., `[":"​, "{"​, "@"]`).
        expected: Vec<String>,
        /// Description of what was found (e.g., `"String"` or `"}"`).
        found: String,
    },

    /// Unexpected end of input while parsing.
    ///
    /// The document ended before a complete construct was parsed.
    ///
    /// # Example
    /// ```text
    /// type User {
    ///           ^ expected `}`, found end of input
    /// ```
    #[error("unexpected end of input")]
    UnexpectedEof {
        /// What was expected when EOF was encountered.
        expected: Vec<String>,
    },

    /// Lexer error encountered during parsing.
    ///
    /// The parser encountered a `GraphQLTokenKind::Error` token from the lexer.
    /// The lexer's error message and notes are preserved in the parent
    /// `GraphQLParseError`'s `message` and `notes` fields.
    ///
    /// # Example
    /// ```text
    /// type User { name: "unterminated string
    ///                   ^ unterminated string literal
    /// ```
    #[error("lexer error")]
    LexerError,

    /// Unclosed delimiter (bracket, brace, or parenthesis).
    ///
    /// A delimiter was opened but EOF was reached before finding the matching
    /// closing delimiter. The opening location is typically included in the
    /// error's `notes`.
    ///
    /// # Example
    /// ```text
    /// type User {
    ///     name: String
    /// # EOF here — missing `}`
    /// ```
    ///
    /// Note: This is distinct from `MismatchedDelimiter`, which occurs when a
    /// *wrong* closing delimiter is found (e.g., `[` closed with `)`).
    #[error("unclosed delimiter: `{delimiter}`")]
    UnclosedDelimiter {
        /// The unclosed delimiter (e.g., `"{"`, `"["`, `"("`).
        delimiter: String,
    },

    /// Mismatched delimiter.
    ///
    /// A closing delimiter was found that doesn't match the most recently
    /// opened delimiter. This indicates a structural nesting error.
    ///
    /// # Example
    /// ```text
    /// type User { name: [String) }
    ///                         ^ expected `]`, found `)`
    /// ```
    ///
    /// Note: This is distinct from `UnclosedDelimiter`, which occurs when EOF
    /// is reached without any closing delimiter.
    #[error("mismatched delimiter")]
    MismatchedDelimiter {
        /// The expected closing delimiter (e.g., `"]"`).
        expected: String,
        /// The actual closing delimiter found (e.g., `")"`).
        found: String,
    },

    /// Invalid value (wraps value parsing errors).
    ///
    /// Occurs when a literal value (string, int, float) cannot be parsed.
    ///
    /// # Example
    /// ```text
    /// query { field(limit: 99999999999999999999) }
    ///                      ^^^^^^^^^^^^^^^^^^^^ integer overflow
    /// ```
    #[error("invalid value")]
    InvalidValue(ValueParsingError),

    /// Reserved name used in a context where it's not allowed.
    ///
    /// Certain names have special meaning in specific contexts:
    /// - `on` cannot be a fragment name (it introduces type conditions)
    /// - `true`, `false`, `null` cannot be enum values (ambiguous with literals)
    ///
    /// # Example
    /// ```text
    /// fragment on on User { name }
    ///          ^^ fragment name cannot be `on`
    /// ```
    #[error("reserved name: `{name}`")]
    ReservedName {
        /// The reserved name that was used (e.g., `"on"`, `"true"`).
        name: String,
        /// The context where this name is not allowed.
        context: ReservedNameContext,
    },

    /// Definition kind not allowed in the document being parsed.
    ///
    /// When parsing with `parse_executable_document()`, schema definitions
    /// (types, directives) are not allowed. When parsing with
    /// `parse_schema_document()`, operations and fragments are not allowed.
    ///
    /// # Example
    /// ```text
    /// # Parsing as executable document:
    /// type User { name: String }
    /// ^^^^ type definition not allowed in executable document
    /// ```
    #[error("wrong document kind")]
    WrongDocumentKind {
        /// What kind of definition was found.
        found: DefinitionKind,
        /// What kind of document is being parsed.
        document_kind: DocumentKind,
    },

    /// Empty construct that requires content.
    ///
    /// Certain constructs cannot be empty per the GraphQL spec:
    /// - Selection sets: `{ }` is invalid (must have at least one selection)
    /// - Argument lists: `()` is invalid (omit parentheses if no arguments)
    ///
    /// # Example
    /// ```text
    /// query { user { } }
    ///              ^^^ selection set cannot be empty
    /// ```
    #[error("invalid empty construct: `{construct}`")]
    InvalidEmptyConstruct {
        /// What construct is empty (e.g., `"selection set"`).
        construct: String,
    },

    /// Invalid syntax that doesn't fit other categories.
    ///
    /// A catch-all for syntax errors without dedicated variants. The specific
    /// error is described in `GraphQLParseError.message`.
    #[error("invalid syntax")]
    InvalidSyntax,
}
