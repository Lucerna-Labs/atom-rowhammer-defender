//! Consumer of the installed fixed-purpose keyboard metadata helper.
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::MetadataExt;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const HELPER: &str = "/opt/ram-monitor/atom-keyboard-guard";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opening {
    pub at: u64,
    pub pid: u32,
    pub uid: u32,
    pub exe_dev: u32,
    pub ino: u64,
    pub device: String,
    pub executable: String,
}
impl Opening {
    pub fn expected_system_reader(&self) -> bool {
        [
            ("/usr/lib/systemd/systemd-logind", 0),
            ("/usr/bin/cosmic-comp", unsafe { libc::getuid() }),
        ]
        .iter()
        .any(|(path, uid)| {
            self.uid == *uid
                && fs::metadata(path).is_ok_and(|m| {
                    let dev = (libc::major(m.dev()) << 20) | libc::minor(m.dev());
                    m.uid() == 0
                        && m.mode() & 0o022 == 0
                        && dev == self.exe_dev
                        && m.ino() == self.ino
                })
        })
    }
    pub fn identity(&self) -> (u32, u32, u64) {
        (self.uid, self.exe_dev, self.ino)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Idle,
    Ready,
    Open(Opening),
    Lost(u64),
}

fn event_unix_second(boot_ns: u64) -> Result<u64, String> {
    let mut boot = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    if unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut boot) } != 0 {
        return Err("boot clock unavailable".into());
    }
    let now = Duration::new(boot.tv_sec as u64, boot.tv_nsec as u32);
    let age = now
        .checked_sub(Duration::from_nanos(boot_ns))
        .ok_or("invalid event time")?;
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .checked_sub(age)
        .ok_or("invalid wall clock")?
        .as_secs())
}

pub fn decode(line: &str) -> Result<Event, String> {
    if let Some(message) = line.strip_prefix("ERROR ") {
        return Err(message.to_owned());
    }
    let parts = line.split_ascii_whitespace().collect::<Vec<_>>();
    let bad = || "invalid keyboard metadata".to_string();
    match parts.as_slice() {
        ["READY", "1"] => Ok(Event::Ready),
        ["LOST", n] => Ok(Event::Lost(n.parse().map_err(|_| bad())?)),
        ["OPEN", time, pid, uid, dev, ino, device, exe] => {
            let number = device.strip_prefix("event").ok_or_else(bad)?;
            if number.parse::<u32>().is_err()
                || !number.bytes().all(|b| b.is_ascii_digit())
                || !exe
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b))
            {
                return Err(bad());
            }
            Ok(Event::Open(Opening {
                at: event_unix_second(time.parse().map_err(|_| bad())?)?,
                pid: pid.parse().map_err(|_| bad())?,
                uid: uid.parse().map_err(|_| bad())?,
                exe_dev: dev.parse().map_err(|_| bad())?,
                ino: ino.parse().map_err(|_| bad())?,
                device: format!("/dev/input/{device}"),
                executable: exe.to_string(),
            }))
        }
        _ => Err(bad()),
    }
}

pub struct Observer {
    child: Child,
    messages: Receiver<Result<Event, String>>,
}
impl Observer {
    pub fn start() -> Result<Self, String> {
        let mut child = Command::new(HELPER)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Cannot start the installed keyboard observer ({e})"))?;
        let stdout = child.stdout.take().ok_or("missing keyboard event pipe")?;
        let (tx, messages) = sync_channel(128);
        let worker = std::thread::Builder::new()
            .name("keyboard-metadata".into())
            .spawn(move || {
                for line in BufReader::new(stdout).lines() {
                    let event = line
                        .map_err(|e| e.to_string())
                        .and_then(|line| decode(&line));
                    let failed = event.is_err();
                    if tx.send(event).is_err() || failed {
                        break;
                    }
                }
            });
        if let Err(e) = worker {
            drop(child.stdin.take());
            let _ = child.wait();
            return Err(e.to_string());
        }
        Ok(Self { child, messages })
    }
    pub fn next(&mut self, timeout: Duration) -> Result<Event, String> {
        match self.messages.recv_timeout(timeout) {
            Ok(event) => event,
            Err(RecvTimeoutError::Timeout) => Ok(Event::Idle),
            Err(RecvTimeoutError::Disconnected) => Err("keyboard observer disconnected".into()),
        }
    }
}
impl Drop for Observer {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        // The helper exits on pipe closure. Terminate a blocked writer as well.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_frames_and_malformed_records() {
        assert_eq!(decode("READY 1"), Ok(Event::Ready));
        assert_eq!(decode("LOST 7"), Ok(Event::Lost(7)));
        assert!(
            matches!(decode("OPEN 0 213 1000 12 32 event5 compositor"),Ok(Event::Open(o))
            if o.pid==213 && o.device=="/dev/input/event5" && o.executable=="compositor")
        );
        for bad in [
            "READY 0",
            "OPEN 0 12 3 4 5 ../event1 x",
            "OPEN 0 12 3 4 5 event1 x extra",
            "OPEN 0 12 3 4 5 event1 <script>",
            "LOST -1",
        ] {
            assert!(decode(bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn name_alone_never_grants_an_exception() {
        let Event::Open(o) = decode("OPEN 0 1 0 0 0 event1 systemd-logind").unwrap() else {
            panic!()
        };
        assert!(!o.expected_system_reader());
    }
}
