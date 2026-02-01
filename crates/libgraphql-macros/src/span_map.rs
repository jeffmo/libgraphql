use libgraphql_parser::SourcePosition;
use proc_macro2::Span;
use std::collections::HashMap;

/// Maps GraphQL source positions back to `proc_macro2::Span`s.
///
/// The keys are `(line, column)` pairs where both values are
/// **0-based** and the column is measured in **UTF-8 characters**
/// (not bytes, not UTF-16 code units).
///
/// This coordinate system matches `SourcePosition::line()` (0-based)
/// and `SourcePosition::col_utf8()` (0-based, UTF-8 char offset).
///
/// We index on `(line, col_utf8)` rather than `byte_offset` because
/// `proc_macro2::Span::byte_range()` only returns meaningful values
/// on nightly Rust toolchains. On stable toolchains, byte offsets are
/// unreliable/zeroed. In contrast, `Span::start()` and `Span::end()`
/// (stabilized in Rust 1.88.0) reliably provide line/column, making
/// `(line, col_utf8)` a stable key across all supported toolchains.
pub(crate) struct SpanMap(HashMap<(usize, usize), Span>);

impl SpanMap {
    pub fn new(map: HashMap<(usize, usize), Span>) -> Self {
        Self(map)
    }

    /// Looks up the `proc_macro2::Span` for a given
    /// `SourcePosition`.
    ///
    /// Returns `None` if no exact match is found. This is
    /// unexpected at runtime — error positions should always
    /// correspond to token start positions recorded during
    /// tokenization. If `None` is returned, the caller should
    /// fall back to `Span::call_site()` and consider emitting a
    /// diagnostic asking the user to report the issue on GitHub.
    pub fn lookup(&self, pos: &SourcePosition) -> Option<Span> {
        self.0.get(&(pos.line(), pos.col_utf8())).copied()
    }
}
