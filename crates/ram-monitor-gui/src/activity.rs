use std::collections::VecDeque;
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const LIMIT: u64 = 1024 * 1024;
const DISPLAY: usize = 200;

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn local_time(t: u64) -> String {
    let epoch = t as libc::time_t;
    let mut parts = std::mem::MaybeUninit::uninit();
    if unsafe { libc::localtime_r(&epoch, parts.as_mut_ptr()) }.is_null() {
        return t.to_string();
    }
    let p = unsafe { parts.assume_init() };
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        p.tm_year + 1900,
        p.tm_mon + 1,
        p.tm_mday,
        p.tm_hour,
        p.tm_min,
        p.tm_sec
    )
}

fn state_file(path: &Path) -> io::Result<File> {
    let f = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    let m = f.metadata()?;
    if !m.is_file()
        || m.uid() != unsafe { libc::getuid() }
        || m.mode() & 0o077 != 0
        || m.nlink() != 1
    {
        return Err(io::Error::other("unsafe alert history file"));
    }
    Ok(f)
}
#[derive(Default)]
pub struct Activity {
    dir: Option<PathBuf>,
    pub rows: VecDeque<String>,
    pub error: Option<String>,
}
impl Activity {
    pub fn open() -> Self {
        let mut this = Self::default();
        let root = std::env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/state")));
        let result = root
            .ok_or_else(|| io::Error::other("no state directory"))
            .and_then(|root| this.load(root.join("atom-defender")));
        if let Err(e) = result {
            this.error = Some(e.to_string());
        }
        this
    }
    fn load(&mut self, dir: PathBuf) -> io::Result<()> {
        DirBuilder::new().recursive(true).mode(0o700).create(&dir)?;
        let m = fs::symlink_metadata(&dir)?;
        if !m.is_dir() || m.mode() & 0o077 != 0 || m.uid() != unsafe { libc::getuid() } {
            return Err(io::Error::other("private alert history directory required"));
        }
        let lock = state_file(&dir.join("history.lock"))?;
        lock.lock()?;
        for name in ["alerts.previous", "alerts.log"] {
            let path = dir.join(name);
            if !path.try_exists()? {
                continue;
            }
            let mut f = state_file(&path)?;
            let offset = f.metadata()?.len().saturating_sub(LIMIT);
            f.seek(SeekFrom::Start(offset))?;
            let mut data = Vec::new();
            f.take(LIMIT).read_to_end(&mut data)?;
            for line in String::from_utf8_lossy(&data)
                .lines()
                .skip(usize::from(offset > 0))
            {
                self.push(line.to_owned());
            }
        }
        self.dir = Some(dir);
        Ok(())
    }
    fn push(&mut self, line: String) {
        self.rows.push_front(line);
        self.rows.truncate(DISPLAY);
    }
    pub fn record(&mut self, at: u64, module: &str, message: &str) {
        let safe: String = message
            .chars()
            .take(512)
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect();
        let line = format!("{}   {module}   {safe}", local_time(at));
        self.push(line.clone());
        if let Some(dir) = self.dir.as_ref() {
            let result = (|| -> io::Result<()> {
                let lock = state_file(&dir.join("history.lock"))?;
                lock.lock()?;
                let path = dir.join("alerts.log");
                let mut f = state_file(&path)?;
                if f.metadata()?.len() >= LIMIT {
                    fs::rename(&path, dir.join("alerts.previous"))?;
                    f = state_file(&path)?;
                }
                writeln!(f, "{line}")
            })();
            self.error = result.err().map(|e| e.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn history_reopens_in_order_and_rejects_a_link() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../target/keyboard-build/history-{}",
            std::process::id()
        ));
        let mut h = Activity::default();
        h.load(base.clone()).unwrap();
        for (m, t) in [("Keyboard Guard", 1), ("File Guard", 2), ("Rowhammer", 3)] {
            h.record(t, m, "sample\nmessage");
        }
        assert!(h.error.is_none());
        let mut reopened = Activity::default();
        reopened.load(base.clone()).unwrap();
        assert_eq!(h.rows, reopened.rows);
        assert!(h.rows[0].contains("Rowhammer   sample message"));
        let m = fs::metadata(base.join("alerts.log")).unwrap();
        assert_eq!(m.mode() & 0o777, 0o600);
        fs::rename(base.join("alerts.log"), base.join("proof")).unwrap();
        std::os::unix::fs::symlink(base.join("proof"), base.join("alerts.log")).unwrap();
        let before = fs::read(base.join("proof")).unwrap();
        h.record(4, "Keyboard Guard", "new event");
        assert!(h.error.is_some());
        assert_eq!(fs::read(base.join("proof")).unwrap(), before);
        fs::remove_dir_all(base).unwrap();
    }
}
