use crate::ast::ByteSpan;
use crate::ast::SourceMap;
use std::path::Path;
use std::path::PathBuf;

/// A position within a file, with 1-based line and column numbers.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FilePosition {
    pub col: usize,
    pub file: Box<PathBuf>,
    pub line: usize,
}
impl FilePosition {
    pub fn file(&self) -> &PathBuf {
        &self.file
    }

    #[cfg(test)]
    pub(crate) fn into_schema_source_location(self) -> SourceLocation {
        SourceLocation::SchemaFile(self)
    }
}
impl std::convert::From<SourceLocation> for Option<FilePosition> {
    fn from(src_loc: SourceLocation) -> Self {
        match src_loc {
            SourceLocation::GraphQLBuiltIn => None,
            SourceLocation::ExecutableDocument => None,
            SourceLocation::ExecutableDocumentFile(file_pos) => Some(file_pos),
            SourceLocation::Schema => None,
            SourceLocation::SchemaFile(file_pos) => Some(file_pos),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum SourceLocation {
    GraphQLBuiltIn,
    ExecutableDocument,
    ExecutableDocumentFile(FilePosition),
    Schema,
    SchemaFile(FilePosition),
}
impl SourceLocation {
    pub(crate) fn from_execdoc_span(
        file_path: Option<&Path>,
        span: ByteSpan,
        source_map: &SourceMap<'_>,
    ) -> Self {
        if let Some(file_path) = file_path {
            let (line, col) = resolve_line_col(span, source_map);
            Self::ExecutableDocumentFile(FilePosition {
                col,
                file: Box::new(file_path.to_path_buf()),
                line,
            })
        } else {
            Self::ExecutableDocument
        }
    }

    pub(crate) fn from_schema_span(
        file_path: Option<&Path>,
        span: ByteSpan,
        source_map: &SourceMap<'_>,
    ) -> Self {
        if let Some(file_path) = file_path {
            let (line, col) = resolve_line_col(span, source_map);
            Self::SchemaFile(FilePosition {
                col,
                file: Box::new(file_path.to_path_buf()),
                line,
            })
        } else {
            Self::Schema
        }
    }

    pub(crate) fn with_span(
        &self,
        span: ByteSpan,
        source_map: &SourceMap<'_>,
    ) -> Self {
        match self {
            Self::GraphQLBuiltIn => Self::GraphQLBuiltIn,

            Self::ExecutableDocument => Self::ExecutableDocument,

            Self::ExecutableDocumentFile(file_pos) => {
                let (line, col) = resolve_line_col(span, source_map);
                Self::ExecutableDocumentFile(FilePosition {
                    col,
                    file: file_pos.file.to_owned(),
                    line,
                })
            },

            Self::Schema => Self::Schema,

            Self::SchemaFile(file_pos) => {
                let (line, col) = resolve_line_col(span, source_map);
                Self::SchemaFile(FilePosition {
                    col,
                    file: file_pos.file.to_owned(),
                    line,
                })
            },
        }
    }
}

/// Resolve a [`ByteSpan`] to 1-based (line, col) via the given [`SourceMap`].
///
/// Falls back to (1, 1) if the span cannot be resolved.
fn resolve_line_col(span: ByteSpan, source_map: &SourceMap<'_>) -> (usize, usize) {
    if let Some(pos) = source_map.resolve_offset(span.start) {
        // SourcePosition is 0-based; FilePosition is 1-based
        (pos.line() + 1, pos.col_utf8() + 1)
    } else {
        // This can happen for synthetic/empty spans (e.g., from macro-generated
        // AST or default ByteSpan values). Defaulting to (1, 1) matches the
        // prior behavior of graphql-parser's Pos default.
        (1, 1)
    }
}
