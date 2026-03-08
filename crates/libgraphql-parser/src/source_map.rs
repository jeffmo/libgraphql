use crate::ByteSpan;
use crate::SourceSpan;
use crate::SourcePosition;
use std::path::Path;
use std::path::PathBuf;

/// Internal storage mode for position resolution.
enum SourceMapData<'src> {
    /// Source-text mode: positions are resolved on demand by binary-searching
    /// `line_starts` and counting chars from the line start offset.
    SourceText {
        source: &'src str,
        line_starts: Vec<u32>,
    },

    /// Pre-computed columns mode: positions were pushed by the token source
    /// during lexing. Lookups binary-search the sorted offset table.
    PrecomputedColumns {
        /// Sorted by byte offset (first element of each tuple). Entries are
        /// inserted in lexing order, which is monotonically increasing.
        entries: Vec<(u32, SourcePosition)>,
    },
}

impl<'src> SourceMapData<'src> {
    /// Resolves a byte offset to a [`SourcePosition`], dispatching to the
    /// appropriate mode-specific implementation.
    ///
    /// Returns `None` if the offset cannot be resolved (e.g., offset out of
    /// bounds, or no pre-computed entries available).
    fn resolve_offset(
        &self,
        byte_offset: u32,
    ) -> Option<SourcePosition> {
        match self {
            Self::SourceText {
                source,
                line_starts,
            } => {
                let offset = byte_offset as usize;

                if offset > source.len() {
                    return None;
                }

                // partition_point returns the first index where
                // line_starts[i] > byte_offset, so the line index is
                // one less.
                let line_index =
                    line_starts.partition_point(|&ls| ls <= byte_offset);
                let line =
                    if line_index > 0 { line_index - 1 } else { 0 };
                let line_start = line_starts[line] as usize;

                // Count Unicode scalar values and UTF-16 code units
                // from line start to the target byte offset.
                //
                // TODO: `col_utf8` in SourcePosition counts Unicode
                // scalar values (what `str::chars()` yields), NOT
                // UTF-8 bytes. The name is inherited from
                // SourcePosition and is misleading — consider renaming
                // to `col_char` or `col_scalar` in a future cleanup.
                let line_slice = &source[line_start..offset];
                let mut col_utf8: usize = 0;
                let mut col_utf16: usize = 0;
                for ch in line_slice.chars() {
                    col_utf8 += 1;
                    col_utf16 += ch.len_utf16();
                }

                Some(SourcePosition::new(
                    line,
                    col_utf8,
                    Some(col_utf16),
                    offset,
                ))
            },
            Self::PrecomputedColumns { entries } => {
                if entries.is_empty() {
                    return None;
                }

                // Floor lookup: find the largest entry offset <=
                // byte_offset.
                let idx = entries
                    .partition_point(|&(off, _)| off <= byte_offset);

                if idx > 0 {
                    Some(entries[idx - 1].1)
                } else {
                    None
                }
            },
        }
    }

    /// Returns the source text, if this is source-text-mode data.
    fn source(&self) -> Option<&'src str> {
        match self {
            Self::SourceText { source, .. } => Some(source),
            Self::PrecomputedColumns { .. } => None,
        }
    }
}

/// Maps byte offsets to line/column positions within a source text.
///
/// `SourceMap` supports two modes of operation, chosen at construction time:
///
/// # Source-Text Mode ([`SourceMap::new_with_source`])
///
/// Built via an O(n) pre-pass that scans the source string for line
/// terminators (`\n`, `\r`, `\r\n`) and records the byte offset of each line
/// start. Individual position lookups are then O(log n) via binary search on
/// the line-start table, plus a short char-counting walk from the line start
/// to the target byte offset to compute UTF-8 and UTF-16 column values.
///
/// This mode is used by
/// [`StrGraphQLTokenSource`](crate::token_source::StrGraphQLTokenSource),
/// which has direct access to the source string. It is memory-efficient
/// (only one `u32` per line) and avoids any per-token bookkeeping during
/// lexing — the lexer only tracks a single `curr_byte_offset` and defers
/// all line/column computation to resolution time.
///
/// # Pre-Computed Columns Mode ([`SourceMap::new_precomputed`])
///
/// Some token sources do not have access to the underlying source text at
/// resolution time. For example,
/// [`RustMacroGraphQLTokenSource`](https://docs.rs/libgraphql-macros) in
/// the `libgraphql-macros` crate produces tokens from a
/// `proc_macro2::TokenStream`. Each `proc_macro2::Span` carries line/column
/// information at the time the token is produced, but there is no contiguous
/// source `&str` to scan after the fact. In this mode, the token source
/// pushes pre-computed `(byte_offset, SourcePosition)` entries into the
/// `SourceMap` during lexing via
/// [`insert_computed_position()`](Self::insert_computed_position), and
/// lookups binary-search that table.
///
/// This mode uses more memory (one full `SourcePosition` per inserted
/// offset, rather than one `u32` per line), but lookups are O(log n) with
/// no char-counting walk — just a binary search and a direct return.
///
/// In the future, `StrGraphQLTokenSource` could also offer a
/// "pre-computed columns" knob: eagerly computing positions during lexing
/// (slightly slower parse throughput) in exchange for faster column lookups
/// afterward (no char-counting walk). This would let users trade parse
/// throughput for lookup speed depending on their workload — e.g., an IDE
/// doing many position lookups per parse might prefer pre-computed columns,
/// while a batch validator that rarely formats errors might prefer
/// source-text mode.
///
/// # Lifetime
///
/// The `'src` lifetime ties the `SourceMap` to the source text it was built
/// from (in source-text mode). In pre-computed columns mode, `'src` is
/// typically `'static` since no source text is borrowed.
///
/// # UTF-16 Column Recovery
///
/// In source-text mode, UTF-16 columns are computed on demand by iterating
/// chars from the line start to the target byte offset and summing
/// [`char::len_utf16()`]. In pre-computed columns mode, UTF-16 columns are
/// whatever the token source provided (or `None` if the token source cannot
/// compute them).
pub struct SourceMap<'src> {
    /// The resolution data — either source-text-backed or pre-computed.
    data: SourceMapData<'src>,

    /// Optional file path for the source text.
    file_path: Option<PathBuf>,
}

impl<'src> SourceMap<'src> {
    /// Builds a `SourceMap` in source-text mode by scanning `source` for
    /// line terminators.
    ///
    /// This is an O(n) pre-pass that identifies all line start byte offsets.
    /// Line terminators recognized: `\n`, `\r`, `\r\n` (the pair counts as
    /// one terminator).
    pub fn new_with_source(
        source: &'src str,
        file_path: Option<PathBuf>,
    ) -> Self {
        let line_starts = Self::compute_line_starts(source);
        Self {
            data: SourceMapData::SourceText {
                source,
                line_starts,
            },
            file_path,
        }
    }

    /// Creates a `SourceMap` in pre-computed columns mode.
    ///
    /// In this mode, the token source is responsible for pushing position
    /// entries via
    /// [`insert_computed_position()`](Self::insert_computed_position)
    /// during lexing. This is intended for token sources that know
    /// line/column information at lex time but do not have access to the
    /// underlying source text afterward.
    ///
    /// See the [type-level documentation](Self) for a detailed comparison
    /// of the two modes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut source_map = SourceMap::new_precomputed(None);
    /// // During lexing, for each token:
    /// source_map.insert_computed_position(byte_offset, position);
    /// ```
    pub fn new_precomputed(file_path: Option<PathBuf>) -> Self {
        Self {
            data: SourceMapData::PrecomputedColumns {
                entries: Vec::new(),
            },
            file_path,
        }
    }

    /// Returns the source text, if this is a source-text-mode `SourceMap`.
    pub fn source(&self) -> Option<&'src str> {
        self.data.source()
    }

    /// Returns the file path, if available.
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    /// Inserts a pre-computed position entry.
    ///
    /// Only valid in pre-computed columns mode. Entries must be inserted in
    /// monotonically increasing byte-offset order (which is naturally the
    /// case during lexing).
    ///
    /// # Panics (debug only)
    ///
    /// Debug-asserts that:
    /// - The `SourceMap` is in pre-computed columns mode.
    /// - The byte offset is >= the last inserted offset (monotonic ordering).
    pub fn insert_computed_position(
        &mut self,
        byte_offset: u32,
        position: SourcePosition,
    ) {
        match &mut self.data {
            SourceMapData::PrecomputedColumns { entries } => {
                debug_assert!(
                    entries.last().is_none_or(
                        |&(last, _)| byte_offset >= last,
                    ),
                    "insert_computed_position called with non-monotonic \
                     byte offset: {byte_offset} after {}",
                    entries.last().unwrap().0,
                );
                entries.push((byte_offset, position));
            },
            SourceMapData::SourceText { .. } => {
                debug_assert!(
                    false,
                    "insert_computed_position called on a \
                     source-text-mode SourceMap",
                );
            },
        }
    }

    /// Resolves a byte offset to a full [`SourcePosition`] (line, col_utf8,
    /// col_utf16, byte_offset).
    ///
    /// Returns `None` if the offset cannot be resolved — for example, if
    /// the byte offset is out of bounds (source-text mode) or if no
    /// pre-computed entries cover the requested offset.
    ///
    /// # Source-text mode
    ///
    /// Uses binary search on `line_starts` to find the line, then counts
    /// chars from the line start to compute columns.
    ///
    /// # Pre-computed columns mode
    ///
    /// Binary-searches the pre-computed entries for the largest byte offset
    /// `<=` the requested offset (floor lookup). If the requested offset
    /// falls between two entries, the earlier entry's position is returned
    /// (this handles lookups for byte offsets mid-token, returning the
    /// position of the nearest preceding entry).
    pub fn resolve_offset(
        &self,
        byte_offset: u32,
    ) -> Option<SourcePosition> {
        self.data.resolve_offset(byte_offset)
    }

    /// Resolves a [`ByteSpan`] to a full [`SourceSpan`] with
    /// line/column information and file path.
    ///
    /// Returns `None` if either endpoint of the span cannot be resolved.
    pub fn resolve_span(
        &self,
        span: ByteSpan,
    ) -> Option<SourceSpan> {
        let start = self.data.resolve_offset(span.start)?;
        let end = self.data.resolve_offset(span.end)?;
        Some(match &self.file_path {
            Some(path) => {
                SourceSpan::with_file(start, end, path.clone())
            },
            None => SourceSpan::new(start, end),
        })
    }

    // ── Line-start computation ─────────────────────────────

    /// Scans source text and returns the byte offset of the start of each
    /// line.
    ///
    /// Line terminators: `\n`, `\r`, `\r\n` (the pair counts as one).
    fn compute_line_starts(source: &str) -> Vec<u32> {
        let bytes = source.as_bytes();
        let len = bytes.len();

        // Pre-allocate: ~40 chars per line as a rough heuristic.
        let mut line_starts = Vec::with_capacity(1 + len / 40);
        line_starts.push(0); // First line always starts at byte 0

        let mut i = 0;
        while i < len {
            match bytes[i] {
                b'\n' => {
                    line_starts.push((i + 1) as u32);
                },
                b'\r' => {
                    // \r\n is a single line terminator
                    if i + 1 < len && bytes[i + 1] == b'\n' {
                        line_starts.push((i + 2) as u32);
                        i += 1; // skip the \n
                    } else {
                        line_starts.push((i + 1) as u32);
                    }
                },
                _ => {},
            }
            i += 1;
        }

        line_starts
    }
}
