use crate::ast;
use std::boxed::Box;
use std::path::Path;
use std::path::PathBuf;

/// Very similar to graphql_parser's [Pos](graphql_parser::Pos), except it
/// includes a PathBuf to the file.
#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchemaDefLocationDeprecated {
    GraphQLBuiltIn,
    Schema(FilePosition),
}
impl std::convert::From<FilePosition> for SchemaDefLocationDeprecated {
    fn from(value: FilePosition) -> SchemaDefLocationDeprecated {
        SchemaDefLocationDeprecated::Schema(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceLocation {
    GraphQLBuiltIn,
    ExecutableDocument,
    ExecutableDocumentFile(FilePosition),
    Schema,
    SchemaFile(FilePosition),
}
impl SourceLocation {
    pub(crate) fn from_execdoc_ast_position(
        file_path: Option<&Path>,
        ast_pos: &ast::AstPos,
    ) -> Self {
        if let Some(file_path) = file_path {
            Self::ExecutableDocumentFile(FilePosition {
                col: ast_pos.column,
                file: Box::new(file_path.to_path_buf()),
                line: ast_pos.line
            })
        } else {
            Self::ExecutableDocument
        }
    }

    pub(crate) fn from_schema_ast_position(
        file_path: Option<&Path>,
        ast_pos: &ast::AstPos,
    ) -> Self {
        if let Some(file_path) = file_path {
            Self::SchemaFile(FilePosition {
                col: ast_pos.column,
                file: Box::new(file_path.to_path_buf()),
                line: ast_pos.line
            })
        } else {
            Self::Schema
        }
    }

    pub(crate) fn with_ast_position(&self, ast_position: &ast::AstPos) -> Self {
        match self {
            Self::GraphQLBuiltIn =>
                Self::GraphQLBuiltIn,

            Self::ExecutableDocument =>
                Self::ExecutableDocument,

            Self::ExecutableDocumentFile(file_pos) =>
                Self::ExecutableDocumentFile(FilePosition {
                    col: ast_position.column,
                    file: file_pos.file.to_owned(),
                    line: ast_position.line,
                }),

            Self::Schema =>
                Self::Schema,

            Self::SchemaFile(file_pos) =>
                Self::SchemaFile(FilePosition {
                    col: ast_position.column,
                    file: file_pos.file.to_owned(),
                    line: ast_position.line,
                })
        }
    }
}
