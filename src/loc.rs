use std::path::Path;
use std::path::PathBuf;

/// Very similar to graphql_parser's [Pos](graphql_parser::Pos), except it
/// includes a PathBuf to the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePosition {
    pub col: usize,
    pub file: Option<PathBuf>,
    pub line: usize,
}
impl FilePosition {
    pub(crate) fn from_pos<P: AsRef<Path>>(
        file: Option<P>,
        pos: graphql_parser::Pos,
    ) -> Self {
        Self {
            col: pos.column,
            file: file.map(|f| f.as_ref().to_path_buf()),
            line: pos.line,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchemaDefLocation {
    GraphQLBuiltIn,
    Schema(FilePosition),
}
