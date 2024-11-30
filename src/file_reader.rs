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
