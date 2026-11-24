//! Linux open-permission gate for one pinned inode and one live application instance.
use nocopy_sensor_engine::{Arm, Direction, Engine, KeyCell, Observation, Order};
use std::{
    env,
    fs::{File, Metadata, OpenOptions},
    io::{self, Read, Write},
    os::fd::{AsRawFd, FromRawFd},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn checked(n: i32) -> io::Result<i32> {
    if n < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(n)
    }
}
fn identity(meta: &Metadata) -> (u64, u64) {
    (meta.dev(), meta.ino())
}
fn pin(path: &Path, flags: i32) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_PATH | flags)
        .open(path)
}

struct Tripwire {
    engine: Engine,
    cell: KeyCell,
    last: Option<Instant>,
    tick: u64,
    fingerprint: u64,
    arm: Arm,
}
impl Tripwire {
    fn new(fingerprint: u64, arm: Arm) -> Result<Self> {
        let mut engine = Engine::new(fingerprint, Order::A, arm);
        engine.set_profile(0.0, true);
        engine
            .observe(Observation {
                tick: 1,
                direction: Direction::Donor,
                site: 0,
                fingerprint,
                quantity: 1.0,
            })
            .map_err(|e| format!("Atom observation: {e:?}"))?;
        Ok(Self {
            engine,
            cell: KeyCell::new(vec![1])?,
            last: None,
            tick: 1,
            fingerprint,
            arm,
        })
    }
    fn observe(&mut self, now: Instant) -> Result<(f64, bool)> {
        if self
            .last
            .is_some_and(|old| now.duration_since(old) >= Duration::from_secs(1))
        {
            *self = Self::new(self.fingerprint, self.arm)?;
        }
        self.last = Some(now);
        self.tick += 1;
        self.engine
            .observe(Observation {
                tick: self.tick,
                direction: Direction::Recipient,
                site: 0,
                fingerprint: self.fingerprint,
                quantity: 1.0,
            })
            .map_err(|e| format!("Atom observation: {e:?}"))?;
        let reaction = self
            .engine
            .measurements()
            .map_err(|e| format!("Atom measurement: {e:?}"))?
            .reaction;
        self.cell.absorb(reaction)?;
        Ok((reaction, self.cell.measurement().ruptured))
    }
}

fn run() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: atom-credential-guard FILE APPROVED_PID (keep stdin open while armed)".into(),
        );
    }
    let path = PathBuf::from(&args[0]);
    let pid: i32 = args[1].to_str().ok_or("invalid PID")?.parse()?;
    if pid <= 1 || !path.is_absolute() {
        return Err("select an absolute file path and a live application PID above 1".into());
    }
    let uid = match env::var("PKEXEC_UID") {
        Ok(s) => s.parse()?,
        Err(_) => unsafe { libc::getuid() },
    };
    let fd = checked(unsafe {
        libc::fanotify_init(
            libc::FAN_CLASS_CONTENT | libc::FAN_CLOEXEC | libc::FAN_NONBLOCK,
            (libc::O_RDONLY | libc::O_CLOEXEC | libc::O_LARGEFILE) as u32,
        )
    })?;
    let mut group = unsafe { File::from_raw_fd(fd) };
    // Only creation of the permission group requires privilege. Resolve all
    // user-selected objects and handle all subsequent events as the operator.
    if unsafe { libc::geteuid() } == 0 && uid != 0 {
        let passwd = unsafe { libc::getpwuid(uid) };
        if passwd.is_null() {
            return Err("unknown operator".into());
        }
        checked(unsafe { libc::setgroups(0, std::ptr::null()) })?;
        checked(unsafe { libc::setgid((*passwd).pw_gid) })?;
        checked(unsafe { libc::setuid(uid) })?;
    }
    checked(unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) })?;
    checked(unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) })?;
    let target = pin(&path, libc::O_NOFOLLOW)?;
    let meta = target.metadata()?;
    if !meta.is_file() || meta.uid() != uid {
        return Err("select a regular file owned by the operator".into());
    }
    let pidfd = checked(unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) as i32 })?;
    let process = unsafe { File::from_raw_fd(pidfd) };
    let exe_path = PathBuf::from(format!("/proc/{pid}/exe"));
    // File metadata of /proc/PID is owned by the live process user.
    let task_dir = std::fs::metadata(format!("/proc/{pid}"))?;
    if task_dir.uid() != uid {
        return Err("approved application must belong to the operator".into());
    }
    let executable = pin(&exe_path, 0)?;
    let exe_id = identity(&executable.metadata()?);
    let file_id = identity(&meta);
    let mut sensor = Tripwire::new(file_id.0.rotate_left(32) ^ file_id.1, Arm::NoChaos)?;
    let pinned_path = std::ffi::CString::new(format!("/proc/self/fd/{}", target.as_raw_fd()))?;
    checked(unsafe {
        libc::fanotify_mark(
            fd,
            libc::FAN_MARK_ADD,
            libc::FAN_OPEN_PERM,
            libc::AT_FDCWD,
            pinned_path.as_ptr(),
        )
    })?;
    let mut polls = [
        libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: process.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: 0,
            events: libc::POLLIN,
            revents: 0,
        },
    ];
    println!("ARMED pid={pid} dev={} inode={}", file_id.0, file_id.1);
    // A stalled GUI must never stall kernel permission decisions. Receipts
    // fit in one atomic pipe write and the next one includes cumulative counts.
    let flags = checked(unsafe { libc::fcntl(1, libc::F_GETFL) })?;
    checked(unsafe { libc::fcntl(1, libc::F_SETFL, flags | libc::O_NONBLOCK) })?;
    let mut allowed = 0u64;
    let mut denied = 0u64;
    let mut last_report = Instant::now();
    let mut pending = None;
    loop {
        if let Err(e) = checked(unsafe { libc::poll(polls.as_mut_ptr(), 3, 200) }) {
            if e.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(e.into());
        }
        if polls[2].revents != 0 {
            return Ok(());
        }
        if polls[1].revents != 0 {
            return Err("approved process exited; guard disarmed".into());
        }
        if std::fs::symlink_metadata(&path).map(|m| identity(&m)).ok() != Some(file_id) {
            return Err("selected path changed; guard disarmed".into());
        }
        if polls[0].revents != 0 {
            // No variable-length reporting flags: one fixed metadata record per read.
            let mut bytes = [0u8; std::mem::size_of::<libc::fanotify_event_metadata>()];
            let n = match group.read(&mut bytes) {
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
                    ) =>
                {
                    continue
                }
                other => other?,
            };
            let event = unsafe {
                bytes
                    .as_ptr()
                    .cast::<libc::fanotify_event_metadata>()
                    .read_unaligned()
            };
            if n != bytes.len()
                || event.vers != libc::FANOTIFY_METADATA_VERSION
                || event.event_len as usize != n
                || event.fd < 0
                || event.mask != libc::FAN_OPEN_PERM
            {
                return Err("kernel event loss or unsupported record; guard disarmed".into());
            }
            let opened = unsafe { File::from_raw_fd(event.fd) };
            let allow = event.pid == pid
                && std::fs::metadata(&exe_path).map(|m| identity(&m)).ok() == Some(exe_id)
                && unsafe { libc::poll(&mut polls[1], 1, 0) } == 0;
            let mut response = [0u8; 8];
            response[..4].copy_from_slice(&opened.as_raw_fd().to_ne_bytes());
            response[4..].copy_from_slice(
                &(if allow {
                    libc::FAN_ALLOW
                } else {
                    libc::FAN_DENY
                })
                .to_ne_bytes(),
            );
            if group.write(&response)? != response.len() {
                return Err("incomplete permission response".into());
            }
            if allow {
                allowed += 1;
            } else {
                denied += 1;
            }
            let (reaction, ruptured) = sensor.observe(Instant::now())?;
            pending = Some((reaction, ruptured, event.pid));
        }
        if last_report.elapsed() >= Duration::from_millis(200) {
            if let Some((reaction, ruptured, reader)) = pending.take() {
                let line = format!("ACTIVE allowed={allowed} denied={denied} last_pid={reader} atom_reaction={reaction:.8} tripwire={ruptured}\n");
                let n = unsafe { libc::write(1, line.as_ptr().cast(), line.len()) };
                if n < 0 {
                    let error = io::Error::last_os_error();
                    if !matches!(
                        error.kind(),
                        io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
                    ) {
                        return Err(error.into());
                    }
                    pending = Some((reaction, ruptured, reader));
                }
            }
            last_report = Instant::now();
        }
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("OFF: {e}");
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compare_burst_and_reset_on_matched_sensor_arms() -> Result<()> {
        for arm in [Arm::NoChaos, Arm::Full] {
            for seed in [1, 7, 32, 400] {
                let now = Instant::now();
                let mut sensor = Tripwire::new(seed, arm)?;
                let first = sensor.observe(now)?;
                for _ in 0..16 {
                    sensor.observe(now)?;
                }
                assert!(sensor.cell.measurement().ruptured);
                assert_eq!(sensor.observe(now + Duration::from_secs(2))?, first);
            }
        }
        Ok(())
    }
}
