use std::path::PathBuf;

/// Very similar to graphql_parser's [Pos](graphql_parser::Pos), except it
/// includes a PathBuf to the file.
#[derive(Clone, Debug)]
pub struct FilePosition {
    pub col: usize,
    pub file: PathBuf,
    pub line: usize,
}
impl FilePosition {
    pub(crate) fn from_pos(file: PathBuf, pos: graphql_parser::Pos) -> Self {
        Self {
            col: pos.column,
            file,
            line: pos.line,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SchemaDefLocation {
    GraphQLBuiltIn,
    SchemaFile(FilePosition),
}
