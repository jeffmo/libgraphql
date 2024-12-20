use std::path::Path;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, ReadContentError>;

pub fn read_content<P: AsRef<Path>>(file_path: P) -> Result<String> {
    let file_path = file_path.as_ref();
    if !file_path.is_file() {
        return Err(ReadContentError::PathIsNotAFile(file_path.to_path_buf()));
    }

    let bytes = std::fs::read(file_path)
        .map_err(|err| ReadContentError::FileReadError {
            file_path: file_path.to_path_buf(),
            err,
        })?;

    let content = String::from_utf8(bytes)
        .map_err(|err| ReadContentError::FileDecodeError {
            file_path: file_path.to_path_buf(),
            err,
        })?;

    Ok(content)
}

#[derive(Debug)]
pub enum ReadContentError {
    FileDecodeError {
        file_path: PathBuf,
        err: std::string::FromUtf8Error,
    },

    FileReadError {
        file_path: PathBuf,
        err: std::io::Error,
    },

    PathIsNotAFile(PathBuf),
}
impl std::cmp::PartialEq for ReadContentError {
    fn eq(&self, other: &Self) -> bool {
        use ReadContentError::*;
        match (&*self, &*other) {
            (FileDecodeError {
                file_path: self_file_path,
                err: self_err,
            }, FileDecodeError {
                file_path: other_file_path,
                err: other_err,
            }) => {
                self_file_path.eq(other_file_path)
                && self_err.eq(other_err)
            },

            (FileReadError {
                file_path: self_file_path,
                err: self_err,
            }, FileReadError {
                file_path: other_file_path,
                err: other_err,
            }) => {
                self_file_path == other_file_path
                && self_err.kind() == other_err.kind()
            },

            (PathIsNotAFile(self_path), PathIsNotAFile(other_path)) => {
                self_path.eq(other_path)
            },

            _ => false,
        }
    }
}
