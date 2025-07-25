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

    pub(crate) fn from_pos<P: AsRef<Path>>(
        file: P,
        pos: graphql_parser::Pos,
    ) -> Self {
        Self {
            col: pos.column,
            file: Box::new(file.as_ref().to_path_buf()),
            line: pos.line,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchemaDefLocation {
    GraphQLBuiltIn,
    Schema(FilePosition),
}
impl std::convert::From<FilePosition> for SchemaDefLocation {
    fn from(value: FilePosition) -> SchemaDefLocation {
        SchemaDefLocation::Schema(value)
    }
}
