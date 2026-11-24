//! Read-only Linux file-object observation. These are conventional filesystem
//! facts for a future Atom input boundary, not an attack verdict.

use std::fmt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::UNIX_EPOCH;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObjectIdentity {
    pub device: u64,
    pub inode: u64,
    pub birth_ns: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityTokenError {
    WrongShape,
    NonDecimal,
    Overflow,
}
impl fmt::Display for IdentityTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongShape => f.write_str("expected DEVICE:INODE:BIRTH_NS"),
            Self::NonDecimal => f.write_str("identity token fields must be decimal digits"),
            Self::Overflow => f.write_str("identity token field exceeds its integer range"),
        }
    }
}
impl std::error::Error for IdentityTokenError {}

pub fn identity_token(identity: ObjectIdentity) -> String {
    format!(
        "{}:{}:{}",
        identity.device, identity.inode, identity.birth_ns
    )
}

pub fn parse_identity_token(token: &str) -> Result<ObjectIdentity, IdentityTokenError> {
    let mut parts = token.split(':');
    let device = parts.next().ok_or(IdentityTokenError::WrongShape)?;
    let inode = parts.next().ok_or(IdentityTokenError::WrongShape)?;
    let birth_ns = parts.next().ok_or(IdentityTokenError::WrongShape)?;
    if parts.next().is_some() {
        return Err(IdentityTokenError::WrongShape);
    }
    for field in [device, inode, birth_ns] {
        if field.is_empty() || !field.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(IdentityTokenError::NonDecimal);
        }
    }
    Ok(ObjectIdentity {
        device: device.parse().map_err(|_| IdentityTokenError::Overflow)?,
        inode: inode.parse().map_err(|_| IdentityTokenError::Overflow)?,
        birth_ns: birth_ns.parse().map_err(|_| IdentityTokenError::Overflow)?,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectReading {
    pub identity: ObjectIdentity,
    /// Link count can change while the same file object persists.
    pub links: u64,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectReadError {
    Io(String),
    NotRegularFile,
    BirthTimeUnavailable,
    ClockBeforeEpoch,
}
impl fmt::Display for ObjectReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => f.write_str(message),
            Self::NotRegularFile => f.write_str("expected a regular file"),
            Self::BirthTimeUnavailable => f.write_str("file birth time unavailable"),
            Self::ClockBeforeEpoch => f.write_str("file birth time predates Unix epoch"),
        }
    }
}
impl std::error::Error for ObjectReadError {}

/// Observe the file object currently reached by `path`, following a symlink.
/// The reading has no authentication or protection against later path swaps.
pub fn read_object(path: &Path) -> Result<ObjectReading, ObjectReadError> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| ObjectReadError::Io(format!("{}: {error}", path.display())))?;
    reading_from_metadata(&metadata)
}

/// Build an object token from metadata already obtained on an opened handle.
pub fn reading_from_metadata(
    metadata: &std::fs::Metadata,
) -> Result<ObjectReading, ObjectReadError> {
    if !metadata.is_file() {
        return Err(ObjectReadError::NotRegularFile);
    }
    let created = metadata
        .created()
        .map_err(|_| ObjectReadError::BirthTimeUnavailable)?;
    let birth_ns = created
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ObjectReadError::ClockBeforeEpoch)?
        .as_nanos();
    Ok(ObjectReading {
        identity: ObjectIdentity {
            device: metadata.dev(),
            inode: metadata.ino(),
            birth_ns,
        },
        links: metadata.nlink(),
        size: metadata.size(),
    })
}
