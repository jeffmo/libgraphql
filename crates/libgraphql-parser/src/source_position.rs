use crate::ast::AstPos;

/// Source position information for parsing, with dual column tracking.
///
/// This is a pure data struct with no mutation methods. Lexers are responsible
/// for computing position values as they scan input.
///
/// This is standalone with no dependency on libgraphql-core.
/// All fields are private with accessor methods.
///
/// # Indexing Convention
///
/// **All position values are 0-based:**
/// - `line`: 0 = first line of the document (0-based)
/// - `col_utf8`: UTF-8 character count within the current line (0-based)
/// - `col_utf16`: Optional UTF-16 code unit offset within the current line
///   (0-based)
/// - `byte_offset`: byte offset within the whole document (0-based)
///
/// # Dual Column Tracking
///
/// Two column representations are supported:
/// - **`col_utf8`** (always available): Number of UTF-8 characters from the
///   start of the current line. Increments by 1 for each character regardless
///   of its byte representation. This is intuitive for users and matches what
///   most text editors display as "column".
/// - **`col_utf16`** (optional): UTF-16 code unit offset within the line. This
///   aligns with LSP (Language Server Protocol) and many editors. It is `Some`
///   when the token source can provide it (e.g. `StrToGraphQLTokenSource`),
///   and `None` when it cannot (e.g. `RustMacroGraphQLTokenSource` in
///   `libgraphql-macros` which uses `proc_macro2::Span` that only provides
///   UTF-8 char-based positions).
///
/// For ASCII text, both columns are equal. For text containing characters
/// outside the Basic Multilingual Plane (e.g., emoji), they differ:
/// - `col_utf8` advances by 1 for each UTF-8 character
/// - `col_utf16` advances by the character's UTF-16 length (1 or 2 code units)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourcePosition {
    /// Line number (0-based: first line is 0)
    line: usize,

    /// UTF-8 character count within current line (0-based: first position is 0)
    col_utf8: usize,

    /// UTF-16 code unit offset within current line (0-based), if available.
    /// None when the token source cannot provide UTF-16 column information.
    col_utf16: Option<usize>,

    /// byte offset from start of document (0-based: first byte is 0)
    byte_offset: usize,
}

impl SourcePosition {
    /// Create a new SourcePosition.
    ///
    /// # Arguments
    /// - `line`: 0-based line number (0 = first line)
    /// - `col_utf8`: 0-based UTF-8 character count within current line
    /// - `col_utf16`: 0-based UTF-16 code unit offset within current line,
    ///   or `None` if not available (e.g., from `proc_macro2::Span`)
    /// - `byte_offset`: 0-based byte offset from document start
    pub fn new(
        line: usize,
        col_utf8: usize,
        col_utf16: Option<usize>,
        byte_offset: usize,
    ) -> Self {
        Self {
            line,
            col_utf8,
            col_utf16,
            byte_offset,
        }
    }

    /// Returns the 0-based line number.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the 0-based (UTF-8) character count within the current line.
    ///
    /// This increments by 1 for each character regardless of byte
    /// representation. For example, both 'a' (1 byte) and '🎉' (4 bytes) each
    /// add 1 to this count.
    pub fn col_utf8(&self) -> usize {
        self.col_utf8
    }

    /// Returns the 0-based UTF-16 code unit offset within the current line,
    /// if available.
    ///
    /// This is `Some` when the token source can provide UTF-16 column
    /// information (e.g., `StrToGraphQLTokenSource`), and `None` when it
    /// cannot (e.g., `RustMacroGraphQLTokenSource` in `libgraphql-macros`).
    ///
    /// For example, 'a' (1 UTF-16 code unit) adds 1 to this count, while '🎉'
    /// (a surrogate pair requiring 2 UTF-16 code units) adds 2 to this count.
    ///
    /// For LSP compatibility, prefer this method when available.
    pub fn col_utf16(&self) -> Option<usize> {
        self.col_utf16
    }

    /// Returns the 0-based byte offset from document start.
    pub fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    /// Convert to an `AstPos` for compatibility with `graphql_parser` types.
    ///
    /// Note: `AstPos` uses 1-based line and column numbers, so this method
    /// adds 1 to both. The column is always derived from `col_utf8`.
    pub fn to_ast_pos(&self) -> AstPos {
        AstPos {
            line: self.line + 1,
            column: self.col_utf8 + 1,
        }
    }
}
