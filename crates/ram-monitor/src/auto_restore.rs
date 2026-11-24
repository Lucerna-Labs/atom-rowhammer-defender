//! Opt-in automatic restoration of one selected file from captured trusted
//! bytes. Every action preserves the changed bytes first and verifies the
//! postcondition. The cognitive engine chooses a method, never whether the
//! hard integrity trigger should be honored.

use std::ffi::CString;
use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ram_monitor_cognitive::{CognitiveRepairSelector, RepairContext, RepairMethod, Selection};

use crate::file_events::{EventSignal, FileEventSource};
use crate::file_object::{reading_from_metadata, ObjectIdentity};
use crate::integrity::RECORD_BYTES;
use crate::integrity_watch::{IntegrityWatch, WatchCheck};

const MAX_EVIDENCE_BYTES: usize = 1024 * 1024;
fn source_origin(id: ObjectIdentity) -> u64 {
    (id.device ^ id.inode.rotate_left(17) ^ id.birth_ns as u64 ^ (id.birth_ns >> 64) as u64)
        .wrapping_mul(0x9e37_79b9_7f4a_7c15)
        | 1
}

#[derive(Debug)]
pub enum GuardError {
    InvalidPath(String),
    Read(String),
    Missing,
    Preserve(String),
    Race(String),
    Action(String),
    Postcondition(String),
    Cognitive(String),
}
impl fmt::Display for GuardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(v) => write!(f, "invalid guard path: {v}"),
            Self::Read(v) => write!(f, "guard read: {v}"),
            Self::Missing => f.write_str("selected file is missing"),
            Self::Preserve(v) => write!(f, "evidence preservation: {v}"),
            Self::Race(v) => write!(f, "file changed during response: {v}"),
            Self::Action(v) => write!(f, "restore action: {v}"),
            Self::Postcondition(v) => write!(f, "restore postcondition: {v}"),
            Self::Cognitive(v) => write!(f, "cognitive control: {v}"),
        }
    }
}
impl std::error::Error for GuardError {}

struct Snapshot {
    bytes: Vec<u8>,
    identity: ObjectIdentity,
    metadata: Metadata,
}

fn read_snapshot(path: &Path) -> Result<Snapshot, GuardError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                GuardError::Missing
            } else {
                GuardError::Read(error.to_string())
            }
        })?;
    let metadata = file
        .metadata()
        .map_err(|error| GuardError::Read(error.to_string()))?;
    if !metadata.is_file() || metadata.len() > MAX_EVIDENCE_BYTES as u64 {
        return Err(GuardError::Read(
            "expected a regular file at most 1 MiB".into(),
        ));
    }
    let identity = reading_from_metadata(&metadata)
        .map_err(|error| GuardError::Read(error.to_string()))?
        .identity;
    let mut bytes = Vec::new();
    file.take((MAX_EVIDENCE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| GuardError::Read(error.to_string()))?;
    if bytes.len() > MAX_EVIDENCE_BYTES || bytes.len() as u64 != metadata.len() {
        return Err(GuardError::Race("file size changed during snapshot".into()));
    }
    Ok(Snapshot {
        bytes,
        identity,
        metadata,
    })
}

#[derive(Clone, Debug)]
pub struct RestoreReceipt {
    pub signal: EventSignal,
    pub evidence: PathBuf,
    pub changed_bytes: usize,
    pub object_changed: bool,
    pub selected: Selection,
    pub executed: RepairMethod,
    pub first_attempt_failed: bool,
    pub cognitive_recorded: bool,
    pub cognitive_error: Option<String>,
    pub before: Option<ObjectIdentity>,
    pub after: ObjectIdentity,
    pub elapsed: Duration,
}

#[derive(Clone, Debug)]
pub enum GuardOutcome {
    Clean { signal: EventSignal },
    PendingWrite,
    Restored(Box<RestoreReceipt>),
}

pub struct AutomaticFileGuard {
    observed: PathBuf,
    evidence_dir: PathBuf,
    trusted: [u8; RECORD_BYTES],
    trusted_mode: u32,
    watch: IntegrityWatch,
    events: FileEventSource,
    cognitive: CognitiveRepairSelector,
    alerts: u32,
    sequence: u64,
}

impl AutomaticFileGuard {
    pub fn arm(reference: &Path, observed: &Path, evidence_dir: &Path) -> Result<Self, GuardError> {
        for path in [reference, observed, evidence_dir] {
            let info = fs::symlink_metadata(path)
                .map_err(|error| GuardError::InvalidPath(format!("{}: {error}", path.display())))?;
            if info.file_type().is_symlink() {
                return Err(GuardError::InvalidPath(format!(
                    "symlink refused: {}",
                    path.display()
                )));
            }
        }
        if !evidence_dir.is_dir() {
            return Err(GuardError::InvalidPath(
                "evidence path must be a directory".into(),
            ));
        }
        let evidence_meta = fs::metadata(evidence_dir)
            .map_err(|error| GuardError::InvalidPath(error.to_string()))?;
        if evidence_meta.uid() != unsafe { libc::geteuid() } || evidence_meta.mode() & 0o077 != 0 {
            return Err(GuardError::InvalidPath(
                "evidence directory must be owned by this user and mode 0700".into(),
            ));
        }
        let parent = observed
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent_dev = fs::metadata(parent)
            .map_err(|error| GuardError::InvalidPath(error.to_string()))?
            .dev();
        if evidence_meta.dev() != parent_dev {
            return Err(GuardError::InvalidPath(
                "evidence directory must share the observed file's filesystem".into(),
            ));
        }
        let events = FileEventSource::new(observed)
            .map_err(|error| GuardError::InvalidPath(error.to_string()))?;
        let watch = IntegrityWatch::start_with_object(reference, observed)
            .map_err(|error| GuardError::Read(error.to_string()))?;
        let trusted = *watch.trusted_reference();
        let trusted_mode = fs::metadata(observed)
            .map_err(|error| GuardError::Read(error.to_string()))?
            .permissions()
            .mode()
            & 0o777;
        let cognitive = CognitiveRepairSelector::new()
            .map_err(|error| GuardError::Cognitive(error.to_string()))?;
        Ok(Self {
            observed: observed.to_path_buf(),
            evidence_dir: evidence_dir.to_path_buf(),
            trusted,
            trusted_mode,
            watch,
            events,
            cognitive,
            alerts: 0,
            sequence: 0,
        })
    }

    pub fn successful_experiences(&self) -> u64 {
        self.cognitive.successful_experiences()
    }

    /// Event-time wake plus a periodic fallback. The response occurs in this
    /// call, before returning to the CLI or GUI caller.
    pub fn wait_and_act(&mut self, timeout: Duration) -> Result<GuardOutcome, GuardError> {
        let mut signal = self
            .events
            .wait(timeout)
            .map_err(|error| GuardError::Read(error.to_string()))?;
        for _ in 0..7 {
            match self.check_and_act(signal) {
                Err(GuardError::Race(_)) => {
                    signal = self
                        .events
                        .wait(Duration::from_millis(20))
                        .map_err(|error| GuardError::Read(error.to_string()))?;
                }
                result => return result,
            }
        }
        self.check_and_act(signal)
    }

    pub fn check_and_act(&mut self, signal: EventSignal) -> Result<GuardOutcome, GuardError> {
        if matches!(signal, EventSignal::WritePending { .. }) {
            if let WatchCheck::ComparedWithObject(receipt, false) = self.watch.poll() {
                if receipt.different_bytes == 0 {
                    self.events.clear_pending_after_verified_clean();
                    return Ok(GuardOutcome::Clean { signal });
                }
            }
            return Ok(GuardOutcome::PendingWrite);
        }
        let checked = self.watch.poll().clone();
        if let WatchCheck::ComparedWithObject(receipt, false) = &checked {
            if receipt.different_bytes == 0 {
                return Ok(GuardOutcome::Clean { signal });
            }
        }
        let began = Instant::now();
        let snapshot = match read_snapshot(&self.observed) {
            Ok(snapshot) => snapshot,
            Err(GuardError::Missing) => return self.restore_missing(signal, began),
            Err(error) => return Err(error),
        };
        let actual_diff = if snapshot.bytes.len() == RECORD_BYTES {
            snapshot
                .bytes
                .iter()
                .zip(&self.trusted)
                .filter(|(a, b)| a != b)
                .count()
        } else {
            RECORD_BYTES
        };
        let actual_object_changed = self
            .watch
            .baseline_object_identity()
            .is_none_or(|baseline| baseline != snapshot.identity);
        let (different_bytes, object_changed, atom_long_mean) = match checked {
            WatchCheck::ComparedWithObject(receipt, changed) => {
                if receipt.different_bytes != actual_diff || changed != actual_object_changed {
                    return Err(GuardError::Race(
                        "sensor and response snapshots differ".into(),
                    ));
                }
                (receipt.different_bytes, changed, receipt.long_mean)
            }
            WatchCheck::Compared(_) => return Err(GuardError::Read("object check absent".into())),
            WatchCheck::ReadError(_) => {
                if actual_diff == 0 && !actual_object_changed {
                    return Err(GuardError::Race(
                        "read error resolved before response".into(),
                    ));
                }
                (actual_diff, actual_object_changed, 0.0)
            }
            WatchCheck::EngineError(error) => return Err(GuardError::Read(error)),
        };
        let evidence = self.preserve(&snapshot)?;
        let parent = self
            .observed
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent_mode = fs::metadata(parent)
            .map_err(|error| GuardError::Read(error.to_string()))?
            .permissions()
            .mode();
        let context = RepairContext {
            different_bytes,
            object_changed,
            file_mode: snapshot.metadata.permissions().mode(),
            parent_mode,
            atom_long_mean,
            prior_alerts: self.alerts,
            origin: source_origin(snapshot.identity),
        };
        let selected = self
            .cognitive
            .select(context)
            .map_err(|error| GuardError::Cognitive(error.to_string()))?;
        let first = selected.method;
        let attempted = self.restore(first, &snapshot, &evidence);
        let (executed, first_attempt_failed) = match attempted {
            Ok(()) => (first, false),
            Err(AttemptError::Race(error)) => return Err(GuardError::Race(error)),
            Err(AttemptError::BeforeMutation(error)) if !object_changed => {
                let other = match first {
                    RepairMethod::Atomic => RepairMethod::InPlace,
                    RepairMethod::InPlace => RepairMethod::Atomic,
                };
                self.restore(other, &snapshot, &evidence)
                    .map_err(|failure| match failure {
                        AttemptError::BeforeMutation(detail) => {
                            GuardError::Action(format!("first {error}; fallback {detail}"))
                        }
                        AttemptError::Race(detail) => GuardError::Race(detail),
                        AttemptError::AfterMutation(detail) => GuardError::Postcondition(detail),
                    })?;
                (other, true)
            }
            Err(AttemptError::BeforeMutation(error)) => return Err(GuardError::Action(error)),
            Err(AttemptError::AfterMutation(error)) => {
                return Err(GuardError::Postcondition(error))
            }
        };
        self.watch
            .rearm_verified_object()
            .map_err(|error| GuardError::Postcondition(error.to_string()))?;
        let after = read_snapshot(&self.observed)?;
        if after.bytes != self.trusted {
            return Err(GuardError::Postcondition(
                "trusted bytes not restored".into(),
            ));
        }
        self.alerts = self.alerts.saturating_add(1);
        let cognitive_result = self.cognitive.record_verified_success(context, executed);
        let cognitive_error = cognitive_result.err().map(|error| error.to_string());
        let receipt = RestoreReceipt {
            signal,
            evidence,
            changed_bytes: different_bytes,
            object_changed,
            selected,
            executed,
            first_attempt_failed,
            cognitive_recorded: cognitive_error.is_none(),
            cognitive_error,
            before: Some(snapshot.identity),
            after: after.identity,
            elapsed: began.elapsed(),
        };
        self.persist_result(&receipt)?;
        Ok(GuardOutcome::Restored(Box::new(receipt)))
    }

    fn restore_missing(
        &mut self,
        signal: EventSignal,
        began: Instant,
    ) -> Result<GuardOutcome, GuardError> {
        let parent = self
            .observed
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent_mode = fs::metadata(parent)
            .map_err(|error| GuardError::Read(error.to_string()))?
            .permissions()
            .mode();
        self.sequence = self.sequence.saturating_add(1);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| GuardError::Preserve(error.to_string()))?
            .as_nanos();
        let evidence = self.evidence_dir.join(format!(
            "event-{}-{}-{stamp}.absent",
            std::process::id(),
            self.sequence
        ));
        let mut record = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&evidence)
            .map_err(|error| GuardError::Preserve(error.to_string()))?;
        record
            .write_all(b"selected path absent before restore\n")
            .and_then(|_| record.sync_all())
            .map_err(|error| GuardError::Preserve(error.to_string()))?;
        File::open(&self.evidence_dir)
            .and_then(|dir| dir.sync_all())
            .map_err(|error| GuardError::Preserve(error.to_string()))?;
        let context = RepairContext {
            different_bytes: RECORD_BYTES,
            object_changed: true,
            file_mode: 0,
            parent_mode,
            atom_long_mean: 0.0,
            prior_alerts: self.alerts,
            origin: 0,
        };
        let selected = self
            .cognitive
            .select(context)
            .map_err(|error| GuardError::Cognitive(error.to_string()))?;
        let staged = parent.join(format!(
            ".ram-monitor-missing-{}-{stamp}.tmp",
            std::process::id()
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&staged)
            .map_err(|error| GuardError::Action(error.to_string()))?;
        let prepared = file
            .write_all(&self.trusted)
            .and_then(|_| file.set_permissions(fs::Permissions::from_mode(self.trusted_mode)))
            .and_then(|_| file.sync_all())
            .and_then(|_| fs::hard_link(&staged, &self.observed));
        if let Err(error) = prepared {
            let _ = fs::remove_file(&staged);
            return Err(if error.kind() == std::io::ErrorKind::AlreadyExists {
                GuardError::Race("selected path reappeared before missing-file restore".into())
            } else {
                GuardError::Action(error.to_string())
            });
        }
        fs::remove_file(&staged).map_err(|error| GuardError::Postcondition(error.to_string()))?;
        File::open(parent)
            .and_then(|dir| dir.sync_all())
            .map_err(|error| GuardError::Postcondition(error.to_string()))?;
        self.watch
            .rearm_verified_object()
            .map_err(|error| GuardError::Postcondition(error.to_string()))?;
        let after = read_snapshot(&self.observed)?;
        if after.bytes != self.trusted {
            return Err(GuardError::Postcondition(
                "trusted bytes not restored".into(),
            ));
        }
        self.alerts = self.alerts.saturating_add(1);
        let cognitive_error = self
            .cognitive
            .record_verified_success(context, RepairMethod::Atomic)
            .err()
            .map(|error| error.to_string());
        let receipt = RestoreReceipt {
            signal,
            evidence,
            changed_bytes: RECORD_BYTES,
            object_changed: true,
            selected,
            executed: RepairMethod::Atomic,
            first_attempt_failed: false,
            cognitive_recorded: cognitive_error.is_none(),
            cognitive_error,
            before: None,
            after: after.identity,
            elapsed: began.elapsed(),
        };
        self.persist_result(&receipt)?;
        Ok(GuardOutcome::Restored(Box::new(receipt)))
    }

    fn persist_result(&self, receipt: &RestoreReceipt) -> Result<(), GuardError> {
        let path = receipt.evidence.with_extension("result");
        let content = format!(
            "signal={:?}\nselected={:?}\nsource={:?}\nexecuted={:?}\nfirst_attempt_failed={}\nchanged_bytes={}\nobject_changed={}\ntraces={:?}\nresponse_hash={}\ncognitive_recorded={}\ncognitive_error={:?}\nbefore={:?}\nafter={:?}\nelapsed_us={}\n",
            receipt.signal,
            receipt.selected.method,
            receipt.selected.source,
            receipt.executed,
            receipt.first_attempt_failed,
            receipt.changed_bytes,
            receipt.object_changed,
            receipt.selected.supporting_traces,
            receipt.selected.response_hash,
            receipt.cognitive_recorded,
            receipt.cognitive_error,
            receipt.before,
            receipt.after,
            receipt.elapsed.as_micros(),
        );
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&path)
            .map_err(|error| {
                GuardError::Postcondition(format!("restored, but result receipt failed: {error}"))
            })?;
        file.write_all(content.as_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|error| {
                GuardError::Postcondition(format!("restored, but result receipt failed: {error}"))
            })?;
        if fs::read(&path).map_err(|error| {
            GuardError::Postcondition(format!("restored, but result readback failed: {error}"))
        })? != content.as_bytes()
        {
            return Err(GuardError::Postcondition(
                "restored, but result receipt readback differs".into(),
            ));
        }
        File::open(&self.evidence_dir)
            .and_then(|dir| dir.sync_all())
            .map_err(|error| {
                GuardError::Postcondition(format!("restored, but result sync failed: {error}"))
            })
    }

    fn preserve(&mut self, snapshot: &Snapshot) -> Result<PathBuf, GuardError> {
        self.sequence = self.sequence.saturating_add(1);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| GuardError::Preserve(error.to_string()))?
            .as_nanos();
        let name = format!(
            "event-{}-{}-{stamp}.changed",
            std::process::id(),
            self.sequence
        );
        let path = self.evidence_dir.join(name);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&path)
            .map_err(|error| GuardError::Preserve(error.to_string()))?;
        file.write_all(&snapshot.bytes)
            .map_err(|error| GuardError::Preserve(error.to_string()))?;
        file.sync_all()
            .map_err(|error| GuardError::Preserve(error.to_string()))?;
        let retained = fs::read(&path).map_err(|error| GuardError::Preserve(error.to_string()))?;
        if retained != snapshot.bytes {
            return Err(GuardError::Preserve("evidence readback differs".into()));
        }
        let metadata_path = path.with_extension("meta");
        let mut sidecar = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&metadata_path)
            .map_err(|error| GuardError::Preserve(error.to_string()))?;
        let id = snapshot.identity;
        write!(
            sidecar,
            "device={}\ninode={}\nbirth_ns={}\nmode={:o}\nsize={}\n",
            id.device,
            id.inode,
            id.birth_ns,
            snapshot.metadata.mode(),
            snapshot.bytes.len()
        )
        .map_err(|error| GuardError::Preserve(error.to_string()))?;
        sidecar
            .sync_all()
            .map_err(|error| GuardError::Preserve(error.to_string()))?;
        File::open(&self.evidence_dir)
            .and_then(|dir| dir.sync_all())
            .map_err(|error| GuardError::Preserve(error.to_string()))?;
        Ok(path)
    }

    fn restore(
        &self,
        method: RepairMethod,
        snapshot: &Snapshot,
        evidence: &Path,
    ) -> Result<(), AttemptError> {
        match method {
            RepairMethod::InPlace => self.restore_in_place(snapshot),
            RepairMethod::Atomic => self.restore_atomic(snapshot, evidence),
        }
    }

    fn restore_in_place(&self, snapshot: &Snapshot) -> Result<(), AttemptError> {
        if snapshot.bytes.len() != RECORD_BYTES {
            return Err(AttemptError::BeforeMutation(
                "in-place requires exactly 160 bytes".into(),
            ));
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&self.observed)
            .map_err(|error| match error.kind() {
                std::io::ErrorKind::NotFound => AttemptError::Race(error.to_string()),
                _ => AttemptError::BeforeMutation(error.to_string()),
            })?;
        let metadata = file
            .metadata()
            .map_err(|error| AttemptError::BeforeMutation(error.to_string()))?;
        let identity = reading_from_metadata(&metadata)
            .map_err(|error| AttemptError::BeforeMutation(error.to_string()))?
            .identity;
        if identity != snapshot.identity || metadata.len() != RECORD_BYTES as u64 {
            return Err(AttemptError::Race("target identity or size changed".into()));
        }
        let mut current = [0_u8; RECORD_BYTES];
        file.read_exact(&mut current)
            .map_err(|error| AttemptError::BeforeMutation(error.to_string()))?;
        if current.as_slice() != snapshot.bytes {
            return Err(AttemptError::Race("target bytes changed".into()));
        }
        file.seek(SeekFrom::Start(0))
            .map_err(|error| AttemptError::BeforeMutation(error.to_string()))?;
        file.write_all(&self.trusted)
            .map_err(|error| AttemptError::AfterMutation(error.to_string()))?;
        file.sync_all()
            .map_err(|error| AttemptError::AfterMutation(error.to_string()))?;
        Ok(())
    }

    fn restore_atomic(&self, snapshot: &Snapshot, evidence: &Path) -> Result<(), AttemptError> {
        self.restore_atomic_with_hook(snapshot, evidence, || {})
    }

    fn restore_atomic_with_hook<F: FnOnce()>(
        &self,
        snapshot: &Snapshot,
        evidence: &Path,
        before_exchange: F,
    ) -> Result<(), AttemptError> {
        let parent = self
            .observed
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| AttemptError::BeforeMutation(error.to_string()))?
            .as_nanos();
        let staged = parent.join(format!(
            ".ram-monitor-restore-{}-{stamp}.tmp",
            std::process::id()
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&staged)
            .map_err(|error| AttemptError::BeforeMutation(error.to_string()))?;
        let prepared = (|| -> Result<(), AttemptError> {
            file.write_all(&self.trusted)
                .map_err(|error| AttemptError::BeforeMutation(error.to_string()))?;
            file.set_permissions(fs::Permissions::from_mode(self.trusted_mode))
                .map_err(|error| AttemptError::BeforeMutation(error.to_string()))?;
            file.sync_all()
                .map_err(|error| AttemptError::BeforeMutation(error.to_string()))?;
            let now = read_snapshot(&self.observed).map_err(|error| match error {
                GuardError::Race(_) | GuardError::Missing => AttemptError::Race(error.to_string()),
                _ => AttemptError::BeforeMutation(error.to_string()),
            })?;
            if now.identity != snapshot.identity || now.bytes != snapshot.bytes {
                return Err(AttemptError::Race(
                    "target changed before atomic replace".into(),
                ));
            }
            Ok(())
        })();
        if let Err(error) = prepared {
            let _ = fs::remove_file(&staged);
            return Err(error);
        }
        before_exchange();
        let from = CString::new(staged.as_os_str().as_bytes())
            .map_err(|_| AttemptError::BeforeMutation("stage path contains NUL".into()))?;
        let to = CString::new(self.observed.as_os_str().as_bytes())
            .map_err(|_| AttemptError::BeforeMutation("observed path contains NUL".into()))?;
        let exchanged = unsafe {
            libc::renameat2(
                libc::AT_FDCWD,
                from.as_ptr(),
                libc::AT_FDCWD,
                to.as_ptr(),
                libc::RENAME_EXCHANGE,
            )
        };
        if exchanged < 0 {
            let error = std::io::Error::last_os_error();
            let _ = fs::remove_file(&staged);
            return Err(AttemptError::BeforeMutation(error.to_string()));
        }
        let displaced = read_snapshot(&staged)
            .map_err(|error| AttemptError::AfterMutation(error.to_string()))?;
        let object_evidence = evidence.with_extension("original-object");
        fs::hard_link(&staged, &object_evidence).map_err(|error| {
            AttemptError::AfterMutation(format!(
                "displaced object remains at {}: {error}",
                staged.display()
            ))
        })?;
        fs::remove_file(&staged).map_err(|error| AttemptError::AfterMutation(error.to_string()))?;
        File::open(&self.evidence_dir)
            .and_then(|dir| dir.sync_all())
            .map_err(|error| AttemptError::AfterMutation(error.to_string()))?;
        if displaced.identity != snapshot.identity || displaced.bytes != snapshot.bytes {
            return Err(AttemptError::AfterMutation(
                "concurrent target preserved but differs from pre-action snapshot".into(),
            ));
        }
        File::open(parent)
            .and_then(|dir| dir.sync_all())
            .map_err(|error| AttemptError::AfterMutation(error.to_string()))?;
        Ok(())
    }
}

enum AttemptError {
    Race(String),
    BeforeMutation(String),
    AfterMutation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exchange_retains_concurrent_replacement() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../target/cognitive-race-{}",
            std::process::id()
        ));
        assert!(!root.exists(), "preserve prior failed fixture");
        fs::create_dir(&root).expect("generated fixture dir");
        let evidence_dir = root.join("evidence");
        fs::create_dir(&evidence_dir).expect("evidence dir");
        fs::set_permissions(&evidence_dir, fs::Permissions::from_mode(0o700))
            .expect("secure evidence");
        let reference = root.join("reference");
        let observed = root.join("observed");
        let trusted = [11_u8; RECORD_BYTES];
        let first = [22_u8; RECORD_BYTES];
        let second = [33_u8; RECORD_BYTES];
        fs::write(&reference, trusted).expect("reference");
        fs::write(&observed, trusted).expect("clean arming object");
        let mut guard = AutomaticFileGuard::arm(&reference, &observed, &evidence_dir).expect("arm");
        fs::write(&observed, first).expect("first changed object");
        let snapshot = read_snapshot(&observed).expect("snapshot");
        let evidence = guard.preserve(&snapshot).expect("preserve first");
        let attacker = root.join("attacker");
        let result = guard.restore_atomic_with_hook(&snapshot, &evidence, || {
            fs::write(&attacker, second).expect("second object");
            fs::rename(&attacker, &observed).expect("concurrent path replacement");
        });
        assert!(matches!(result, Err(AttemptError::AfterMutation(_))));
        assert_eq!(fs::read(&observed).expect("restored"), trusted);
        assert_eq!(fs::read(&evidence).expect("first preserved"), first);
        assert_eq!(
            fs::read(evidence.with_extension("original-object")).expect("second preserved"),
            second
        );
        fs::remove_dir_all(root).expect("remove verified generated fixture");
    }
}
