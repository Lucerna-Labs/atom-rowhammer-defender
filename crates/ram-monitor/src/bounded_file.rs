//! One-descriptor, bounded reads of operator-selected regular files.
use std::fmt;
use std::fs::Metadata;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadError {
    Io(String),
    NotRegular,
    TooLarge(u64),
    InvalidUtf8,
}
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => f.write_str(message),
            Self::NotRegular => f.write_str("expected a regular file"),
            Self::TooLarge(size) => write!(f, "file exceeds byte limit: {size} bytes"),
            Self::InvalidUtf8 => f.write_str("file is not UTF-8"),
        }
    }
}
impl std::error::Error for ReadError {}

pub fn read_regular_bytes(path: &Path, max_bytes: usize) -> Result<Vec<u8>, ReadError> {
    read_regular_bytes_with_metadata(path, max_bytes).map(|(bytes, _)| bytes)
}

/// Return bytes and metadata from the same opened regular-file descriptor.
pub fn read_regular_bytes_with_metadata(
    path: &Path,
    max_bytes: usize,
) -> Result<(Vec<u8>, Metadata), ReadError> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
        .map_err(|error| ReadError::Io(error.to_string()))?;
    let metadata = file
        .metadata()
        .map_err(|error| ReadError::Io(error.to_string()))?;
    if !metadata.is_file() {
        return Err(ReadError::NotRegular);
    }
    let max_u64 = u64::try_from(max_bytes).map_err(|_| ReadError::TooLarge(u64::MAX))?;
    if metadata.len() > max_u64 {
        return Err(ReadError::TooLarge(metadata.len()));
    }
    let limit = max_u64
        .checked_add(1)
        .ok_or(ReadError::TooLarge(u64::MAX))?;
    let mut bytes = Vec::new();
    file.take(limit)
        .read_to_end(&mut bytes)
        .map_err(|error| ReadError::Io(error.to_string()))?;
    if bytes.len() > max_bytes {
        return Err(ReadError::TooLarge(bytes.len() as u64));
    }
    Ok((bytes, metadata))
}

pub fn read_regular_text(path: &Path, max_bytes: usize) -> Result<String, ReadError> {
    String::from_utf8(read_regular_bytes(path, max_bytes)?).map_err(|_| ReadError::InvalidUtf8)
}
