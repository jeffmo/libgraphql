use std::path::PathBuf;

/// Owned, serializable source map data for resolving
/// [`ByteSpan`](libgraphql_parser::ByteSpan)s to line/column
/// positions within a [`Schema`](crate::schema::Schema).
///
/// # Why not re-use `libgraphql_parser::SourceMap`?
///
/// The parser's [`SourceMap<'src>`](libgraphql_parser::SourceMap)
/// borrows source text via `'src` and does not implement
/// `serde::Serialize`. A [`Schema`] must be `'static` and
/// serde-serializable (the `libgraphql-macros` crate embeds
/// schemas as binary at compile time). `SchemaSourceMap` stores
/// just the line-start byte offsets and optional file path —
/// the minimum data needed for deferred line/column resolution.
///
/// One `SchemaSourceMap` exists per source file or string loaded
/// into a [`SchemaBuilder`](crate::schema::SchemaBuilder).
#[derive(Clone, Debug, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct SchemaSourceMap {
    pub(crate) file_path: Option<PathBuf>,
    pub(crate) line_starts: Vec<u32>,
}

impl SchemaSourceMap {
    /// Creates a `SchemaSourceMap` by scanning `source` for line
    /// terminators to compute line-start byte offsets.
    ///
    /// This performs the same O(n) line-start scan that the parser's
    /// `SourceMap` does internally, but the result is fully owned and
    /// serializable.
    pub fn from_source(
        source: &str,
        file_path: Option<PathBuf>,
    ) -> Self {
        let bytes = source.as_bytes();
        let mut line_starts = vec![0u32];
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'\n' => {
                    line_starts.push((i + 1) as u32);
                    i += 1;
                },
                b'\r' => {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                        line_starts.push((i + 2) as u32);
                        i += 2;
                    } else {
                        line_starts.push((i + 1) as u32);
                        i += 1;
                    }
                },
                _ => i += 1,
            }
        }
        Self { file_path, line_starts }
    }

    /// Creates a synthetic source map for built-in definitions.
    pub fn builtin() -> Self {
        Self { file_path: None, line_starts: vec![0] }
    }

    pub fn file_path(&self) -> Option<&std::path::Path> {
        self.file_path.as_deref()
    }

    /// Resolves a byte offset to a 0-based line/column position.
    ///
    /// Returns both a byte-offset column and a UTF-8 character
    /// column. Computing the UTF-8 column requires the source
    /// text for the line slice (to count characters); if source
    /// text is unavailable, pass `None` and `col_utf8` will
    /// equal `col_linestart_byte_offset` (correct for ASCII).
    pub fn resolve_offset(
        &self,
        byte_offset: u32,
        source: Option<&str>,
    ) -> LineCol {
        let line = self.line_starts
            .partition_point(|&start| start <= byte_offset)
            .saturating_sub(1);
        let line_start = self.line_starts[line];
        let col_byte = byte_offset - line_start;
        let col_utf8 = match source {
            Some(src) => {
                let start = line_start as usize;
                let end = byte_offset as usize;
                match src.get(start..end) {
                    Some(line_slice) => line_slice.chars().count() as u32,
                    None => col_byte,
                }
            },
            None => col_byte,
        };
        LineCol {
            line: line as u32,
            col_linestart_byte_offset: col_byte,
            col_utf8,
        }
    }
}

/// A resolved 0-based line and column position.
///
/// Provides two column representations:
/// - `col_utf8`: UTF-8 character count from line start (consistent
///   with [`SourcePosition::col_utf8()`](libgraphql_parser::SourcePosition::col_utf8))
/// - `col_linestart_byte_offset`: byte offset from line start
///
/// For ASCII-only content (the common case in GraphQL), both
/// values are equal.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LineCol {
    pub line: u32,
    pub col_linestart_byte_offset: u32,
    pub col_utf8: u32,
}
