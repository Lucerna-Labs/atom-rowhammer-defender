//! Event-time Linux path observation. A wake is a reason to re-read the
//! selected file, never evidence of who changed it or a pre-write veto.
use std::ffi::CString;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const CHANGE_MASK: u32 = libc::IN_MODIFY
    | libc::IN_CLOSE_WRITE
    | libc::IN_MOVED_FROM
    | libc::IN_MOVED_TO
    | libc::IN_CREATE
    | libc::IN_DELETE
    | libc::IN_ATTRIB;
const LOSS_MASK: u32 = libc::IN_Q_OVERFLOW
    | libc::IN_IGNORED
    | libc::IN_UNMOUNT
    | libc::IN_DELETE_SELF
    | libc::IN_MOVE_SELF;
const HEADER_BYTES: usize = 16;
const BUFFER_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventError {
    InvalidPath,
    InvalidTimeout,
    MalformedEvent,
    Io(String),
}
impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath => f.write_str("expected a file name in an existing directory"),
            Self::InvalidTimeout => f.write_str("event wait must be at most 60 seconds"),
            Self::MalformedEvent => f.write_str("malformed inotify event; full recheck required"),
            Self::Io(error) => write!(f, "inotify: {error}"),
        }
    }
}
impl std::error::Error for EventError {}

/// An event is never an attacker verdict. `Uncertain` requires a full
/// recheck because the queue or directory watch may have been lost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSignal {
    Changed { mask: u32 },
    WritePending { mask: u32 },
    Uncertain { mask: u32 },
    Timeout,
}

pub struct FileEventSource {
    file: File,
    watch_descriptor: i32,
    basename: Vec<u8>,
    path: PathBuf,
    write_pending: bool,
}
impl FileEventSource {
    pub fn new(path: &Path) -> Result<Self, EventError> {
        let basename = path.file_name().ok_or(EventError::InvalidPath)?;
        if basename.is_empty() {
            return Err(EventError::InvalidPath);
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent = parent
            .canonicalize()
            .map_err(|error| EventError::Io(error.to_string()))?;
        if !parent.is_dir() {
            return Err(EventError::InvalidPath);
        }
        let parent_c =
            CString::new(parent.as_os_str().as_bytes()).map_err(|_| EventError::InvalidPath)?;
        let fd = unsafe { libc::inotify_init1(libc::IN_CLOEXEC | libc::IN_NONBLOCK) };
        if fd < 0 {
            return Err(EventError::Io(io::Error::last_os_error().to_string()));
        }
        // SAFETY: inotify_init1 returned a fresh owned descriptor above.
        let file = unsafe { File::from_raw_fd(fd) };
        let mask = CHANGE_MASK | LOSS_MASK | libc::IN_ONLYDIR;
        let watch_descriptor =
            unsafe { libc::inotify_add_watch(file.as_raw_fd(), parent_c.as_ptr(), mask) };
        if watch_descriptor < 0 {
            return Err(EventError::Io(io::Error::last_os_error().to_string()));
        }
        Ok(Self {
            file,
            watch_descriptor,
            basename: basename.as_bytes().to_vec(),
            path: parent.join(basename),
            write_pending: false,
        })
    }

    pub fn selected_path(&self) -> &Path {
        &self.path
    }

    pub fn clear_pending_after_verified_clean(&mut self) {
        self.write_pending = false;
    }

    /// Wait for a selected-name change, queue/watch loss, or a bounded
    /// fallback timeout. Unrelated siblings are ignored.
    pub fn wait(&mut self, timeout: Duration) -> Result<EventSignal, EventError> {
        if timeout > Duration::from_secs(60) {
            return Err(EventError::InvalidTimeout);
        }
        let started = Instant::now();
        loop {
            let remaining = timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Ok(if self.write_pending {
                    EventSignal::WritePending { mask: 0 }
                } else {
                    EventSignal::Timeout
                });
            }
            let timeout_ms = remaining.as_millis().min(60_000) as i32;
            let mut descriptor = libc::pollfd {
                fd: self.file.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            let ready = unsafe { libc::poll(&mut descriptor, 1, timeout_ms) };
            if ready < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(EventError::Io(error.to_string()));
            }
            if ready == 0 {
                return Ok(if self.write_pending {
                    EventSignal::WritePending { mask: 0 }
                } else {
                    EventSignal::Timeout
                });
            }
            if descriptor.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
                return Ok(EventSignal::Uncertain { mask: 0 });
            }
            let mut changed = 0_u32;
            let mut pending = 0_u32;
            let mut uncertain = 0_u32;
            let mut bytes = [0_u8; BUFFER_BYTES];
            loop {
                match self.file.read(&mut bytes) {
                    Ok(0) => return Ok(EventSignal::Uncertain { mask: 0 }),
                    Ok(length) => {
                        let mut offset = 0;
                        while offset < length {
                            if length - offset < HEADER_BYTES {
                                return Err(EventError::MalformedEvent);
                            }
                            let header = &bytes[offset..offset + HEADER_BYTES];
                            let wd =
                                i32::from_ne_bytes([header[0], header[1], header[2], header[3]]);
                            let mask =
                                u32::from_ne_bytes([header[4], header[5], header[6], header[7]]);
                            let name_len = u32::from_ne_bytes([
                                header[12], header[13], header[14], header[15],
                            ]) as usize;
                            let end = offset
                                .checked_add(HEADER_BYTES + name_len)
                                .ok_or(EventError::MalformedEvent)?;
                            if end > length {
                                return Err(EventError::MalformedEvent);
                            }
                            let name = bytes[offset + HEADER_BYTES..end]
                                .split(|byte| *byte == 0)
                                .next()
                                .unwrap_or(&[]);
                            if mask & LOSS_MASK != 0 {
                                uncertain |= mask & LOSS_MASK;
                            } else if wd == self.watch_descriptor && name == self.basename {
                                pending |= mask & (libc::IN_MODIFY | libc::IN_CREATE);
                                if pending != 0 {
                                    self.write_pending = true;
                                }
                                let complete = mask
                                    & (libc::IN_CLOSE_WRITE
                                        | libc::IN_MOVED_FROM
                                        | libc::IN_MOVED_TO
                                        | libc::IN_DELETE);
                                if complete != 0 {
                                    self.write_pending = false;
                                    changed |= complete;
                                }
                                if mask & libc::IN_ATTRIB != 0 {
                                    changed |= libc::IN_ATTRIB;
                                }
                            }
                            offset = end;
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) => return Err(EventError::Io(error.to_string())),
                }
            }
            if uncertain != 0 {
                return Ok(EventSignal::Uncertain { mask: uncertain });
            }
            if self.write_pending {
                return Ok(EventSignal::WritePending { mask: pending });
            }
            if changed != 0 {
                return Ok(EventSignal::Changed { mask: changed });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, OpenOptions};
    use std::io::Write;

    #[test]
    fn selected_write_waits_for_close() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../target/event-close-{}", std::process::id()));
        assert!(!root.exists(), "preserve prior failed fixture");
        fs::create_dir(&root).expect("generated fixture");
        let observed = root.join("observed");
        fs::write(&observed, b"trusted").expect("initial file");
        let mut events = FileEventSource::new(&observed).expect("watch");
        let mut writer = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&observed)
            .expect("open writer");
        assert!(matches!(
            events
                .wait(Duration::from_millis(100))
                .expect("truncate event"),
            EventSignal::WritePending { .. }
        ));
        writer.write_all(b"changed").expect("write");
        assert!(matches!(
            events
                .wait(Duration::from_millis(100))
                .expect("modify event"),
            EventSignal::WritePending { .. }
        ));
        drop(writer);
        assert!(matches!(
            events.wait(Duration::from_millis(100)).expect("close event"),
            EventSignal::Changed { mask } if mask & libc::IN_CLOSE_WRITE != 0
        ));
        fs::remove_dir_all(root).expect("remove verified generated fixture");
    }
}
