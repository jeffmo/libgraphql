/// Trait implemented by all AST node types. Provides source
/// reconstruction methods.
///
/// All AST node types implement this trait via
/// `#[inherent] impl AstNode`, giving each node both inherent
/// methods (no trait import needed) and a trait bound for generic
/// utilities (error formatters, linters, etc.).
///
/// # Source Reconstruction Modes
///
/// - **Source-slice mode (fast, lossless):** When `source` is
///   `Some(s)`, [`append_source`](AstNode::append_source) slices
///   `&s[span.start.byte_offset..span.end.byte_offset]`. This is
///   the common path for string-based token sources. Zero
///   allocation.
///
/// - **Synthetic-formatting mode (slower, lossy):** When `source`
///   is `None`, [`append_source`](AstNode::append_source) walks the
///   AST and emits keywords, names, values, and punctuation with
///   standard spacing. The output is semantically equivalent but not
///   formatting-identical.
pub trait AstNode {
    /// Append this node's source representation to `sink`.
    ///
    /// When `source` is `Some(s)`, slices the original source text
    /// directly via byte offsets (zero-copy, lossless). When
    /// `source` is `None`, reconstructs from semantic data with
    /// standard formatting (lossy but semantically equivalent).
    fn append_source(
        &self,
        sink: &mut String,
        source: Option<&str>,
    );

    /// Return this node as a source string.
    ///
    /// Convenience wrapper around
    /// [`append_source`](AstNode::append_source).
    fn to_source(
        &self,
        source: Option<&str>,
    ) -> String {
        let mut s = String::new();
        self.append_source(&mut s, source);
        s
    }
}
