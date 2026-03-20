use crate::ByteSpan;
use crate::SourceMap;
use crate::SourceSpan;

/// Append the source text for `span` to `sink` by slicing
/// directly from `source` via byte offsets (zero-copy,
/// lossless).
pub(crate) fn append_span_source_slice(
    span: ByteSpan,
    sink: &mut String,
    source: &str,
) {
    let start = span.start as usize;
    let end = span.end as usize;
    debug_assert!(
        start <= end,
        "append_span_source_slice: inverted span \
         (start {start} > end {end})",
    );
    debug_assert!(
        end <= source.len(),
        "append_span_source_slice: span byte range \
         {}..{} exceeds source length {}",
        start,
        end,
        source.len(),
    );
    sink.push_str(&source[start..end]);
}

/// Trait implemented by all AST node types. Provides source
/// reconstruction and span access methods.
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
///
/// # Span Access
///
/// Every AST node carries a [`ByteSpan`] recording its
/// byte-offset range in the source text.
/// [`byte_span()`](AstNode::byte_span) exposes this uniformly
/// across all node types, and
/// [`source_span()`](AstNode::source_span) resolves it to
/// line/column coordinates on demand via a [`SourceMap`].
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

    /// Returns this node's byte-offset span within the source
    /// text.
    ///
    /// The returned [`ByteSpan`] is a compact `[start, end)`
    /// byte range that can be resolved to line/column positions
    /// via [`source_span()`](AstNode::source_span) or
    /// [`ByteSpan::resolve()`].
    fn byte_span(&self) -> ByteSpan;

    /// Resolves this node's position to line/column coordinates
    /// using the given [`SourceMap`].
    ///
    /// Returns [`None`] if the byte offsets cannot be resolved
    /// (e.g. the node was synthetically constructed without
    /// valid span data).
    fn source_span(
        &self,
        source_map: &SourceMap,
    ) -> Option<SourceSpan> {
        self.byte_span().resolve(source_map)
    }

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
