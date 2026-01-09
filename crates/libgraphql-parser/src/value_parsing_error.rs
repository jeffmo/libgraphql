use crate::GraphQLStringParsingError;

/// Errors that occur when parsing literal values.
///
/// These errors occur when converting raw token text to semantic values.
/// For example, parsing the integer `9999999999999999999999` overflows i32.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ValueParsingError {
    /// Invalid string literal (bad escape sequence, unterminated, etc.).
    #[error("Invalid GraphQL string: {0}")]
    String(#[from] GraphQLStringParsingError),

    /// Invalid integer literal (overflow, invalid format).
    ///
    /// GraphQL integers must fit in a signed 32-bit integer (i32).
    #[error("Invalid GraphQL integer: {0}")]
    Int(String),

    /// Invalid float literal (infinity, NaN, invalid format).
    ///
    /// GraphQL floats must be finite f64 values.
    #[error("Invalid GraphQL float: {0}")]
    Float(String),
}
