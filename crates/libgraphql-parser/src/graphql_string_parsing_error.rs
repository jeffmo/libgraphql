/// Error returned when parsing a GraphQL string value fails.
///
/// This error can occur during `GraphQLTokenKind::parse_string_value()` when
/// processing escape sequences.
#[derive(Clone, Debug, thiserror::Error)]
pub enum GraphQLStringParsingError {
    /// An invalid escape sequence was encountered (e.g. `\q`).
    #[error("Invalid escape sequence: `{0}`")]
    InvalidEscapeSequence(String),

    /// The string was not properly terminated.
    #[error("Unterminated string: missing closing quote")]
    UnterminatedString,

    /// An invalid Unicode escape sequence was encountered (e.g. `\u{ZZZZ}`).
    #[error("Invalid unicode escape: `{0}`")]
    InvalidUnicodeEscape(String),
}
