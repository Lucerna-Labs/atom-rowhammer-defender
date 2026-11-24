//! One-time GUI arming of the bus-managed automatic file guard.
use std::path::Path;
use std::time::Duration;

use eframe::egui;
use ram_monitor::auto_restore::{AutomaticFileGuard, GuardOutcome};
use ram_monitor_bus::{BusEvent, GuardBus, GuardDelivery};

pub struct GuardUi {
    pub evidence_dir: String,
    pub status: String,
    pub restored: u64,
    pub alerts: Vec<String>,
    pub failed: bool,
    active: bool,
    stop_requested: bool,
    runtime: Option<GuardBus<GuardOutcome>>,
}

impl GuardUi {
    pub fn new() -> Self {
        Self {
            evidence_dir: String::new(),
            status: "Automatic restore is disarmed".into(),
            restored: 0,
            alerts: Vec::new(),
            failed: false,
            active: false,
            stop_requested: false,
            runtime: None,
        }
    }

    pub fn busy(&self) -> bool {
        self.runtime.is_some()
    }
    pub fn active(&self) -> bool {
        self.active
    }

    pub fn arm(
        &mut self,
        ctx: &egui::Context,
        reference: &Path,
        observed: &Path,
    ) -> Result<(), String> {
        if self.busy() {
            return Err("wait for the previous guard worker to stop".into());
        }
        let evidence = Path::new(self.evidence_dir.trim());
        if self.evidence_dir.trim().is_empty() {
            return Err("choose an existing private evidence directory".into());
        }
        let mut guard = AutomaticFileGuard::arm(reference, observed, evidence)
            .map_err(|error| error.to_string())?;
        let repaint = ctx.clone();
        let runtime = GuardBus::start_with_notify(
            move |duration| {
                guard
                    .wait_and_act(duration)
                    .map_err(|error| error.to_string())
            },
            Duration::from_millis(200),
            None,
            move || repaint.request_repaint(),
        )
        .map_err(|error| error.to_string())?;
        self.runtime = Some(runtime);
        self.active = true;
        self.stop_requested = false;
        self.failed = false;
        self.status = format!("ARMED via Spiderweb · evidence {}", evidence.display());
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(runtime) = &self.runtime {
            runtime.stop();
        }
        self.stop_requested = true;
        self.status = "Stopping automatic restore after current check".into();
    }

    fn apply_message(&mut self, message: BusEvent<GuardOutcome>) {
        match message {
            BusEvent::Delivery(GuardDelivery {
                outcome: Ok(GuardOutcome::Restored(receipt)),
                ..
            }) => {
                self.status = format!(
                    "RESTORED {} exact bytes · object changed {} · method {:?} · cognitive {:?} · evidence {} · {} µs",
                    receipt.changed_bytes, receipt.object_changed,
                    receipt.executed, receipt.selected.source,
                    receipt.evidence.display(), receipt.elapsed.as_micros());
                self.alerts.push(self.status.clone());
                self.restored += 1;
                self.failed = false;
            }
            BusEvent::Delivery(GuardDelivery {
                outcome: Ok(GuardOutcome::PendingWrite),
                ..
            }) => self.status = "Write in progress · waiting for selected file close".into(),
            BusEvent::Delivery(GuardDelivery {
                outcome: Err(error),
                ..
            })
            | BusEvent::Crashed(error) => {
                self.status = format!("GUARD FAILED: {error}");
                self.alerts.push(self.status.clone());
                self.failed = true;
                self.active = false;
            }
            BusEvent::Delivery(GuardDelivery {
                outcome: Ok(GuardOutcome::Clean { .. }),
                ..
            }) => {}
        }
    }

    pub fn poll(&mut self) {
        let queued: Vec<_> = self
            .runtime
            .as_ref()
            .map(|runtime| runtime.events().try_iter().collect())
            .unwrap_or_default();
        for message in queued {
            self.apply_message(message);
        }
        if self.runtime.as_ref().is_some_and(GuardBus::is_finished) {
            if let Some(mut runtime) = self.runtime.take() {
                let joined = runtime.join();
                for message in runtime.events().try_iter() {
                    self.apply_message(message);
                }
                match joined {
                    Err(error) => {
                        self.status = format!("GUARD FAILED: {error}");
                        self.failed = true;
                    }
                    Ok(report) if self.stop_requested && !self.failed => {
                        self.status = format!(
                            "Automatic restore disarmed · {} verified repairs · bus {}/{} deliveries",
                            self.restored, report.received, report.published
                        );
                    }
                    Ok(_) => {}
                }
            }
            self.active = false;
            self.stop_requested = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Instant;

    #[test]
    fn bus_guard_restores_then_disarms() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../target/gui-bus-guard-{}", std::process::id()));
        assert!(!root.exists(), "preserve prior failed fixture");
        fs::create_dir(&root).expect("generated root");
        let evidence = root.join("evidence");
        fs::create_dir(&evidence).expect("evidence");
        fs::set_permissions(&evidence, fs::Permissions::from_mode(0o700))
            .expect("private evidence");
        let reference = root.join("reference");
        let observed = root.join("observed");
        let trusted = [31_u8; 160];
        let changed = [47_u8; 160];
        fs::write(&reference, trusted).expect("reference");
        fs::write(&observed, trusted).expect("observed");
        let mut ui = GuardUi::new();
        ui.evidence_dir = evidence.display().to_string();
        let ctx = egui::Context::default();
        ui.arm(&ctx, &reference, &observed).expect("arm GUI guard");
        fs::write(&observed, changed).expect("first edit");
        let deadline = Instant::now() + Duration::from_secs(3);
        while ui.restored == 0 && Instant::now() < deadline {
            ui.poll();
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(ui.restored, 1, "{}", ui.status);
        assert_eq!(fs::read(&observed).expect("restored bytes"), trusted);
        assert_eq!(
            fs::read(
                evidence
                    .read_dir()
                    .expect("evidence listing")
                    .find_map(|entry| {
                        let path = entry.ok()?.path();
                        (path.extension()? == "changed").then_some(path)
                    })
                    .expect("changed copy")
            )
            .expect("changed bytes"),
            changed
        );
        ui.stop();
        let deadline = Instant::now() + Duration::from_secs(3);
        while ui.busy() && Instant::now() < deadline {
            ui.poll();
            thread::sleep(Duration::from_millis(10));
        }
        assert!(!ui.busy(), "bus did not join after disarm");
        assert!(!ui.failed, "{}", ui.status);
        fs::write(&observed, changed).expect("second edit after disarm");
        thread::sleep(Duration::from_millis(250));
        assert_eq!(fs::read(&observed).expect("second bytes"), changed);
        fs::remove_dir_all(root).expect("remove verified generated fixture");
    }

    #[test]
    fn final_bus_error_survives_join_and_disarm() {
        let mut ui = GuardUi::new();
        ui.runtime = Some(
            GuardBus::<GuardOutcome>::start(
                |_| Err("forced final guard failure".into()),
                Duration::from_millis(1),
                Some(1),
            )
            .expect("start bus"),
        );
        ui.active = true;
        ui.stop_requested = true;
        let deadline = Instant::now() + Duration::from_secs(3);
        while ui
            .runtime
            .as_ref()
            .is_some_and(|runtime| !runtime.is_finished())
            && Instant::now() < deadline
        {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(ui.runtime.as_ref().is_some_and(GuardBus::is_finished));
        ui.poll();
        assert!(ui.failed);
        assert!(
            ui.status.contains("forced final guard failure"),
            "{}",
            ui.status
        );
        assert!(!ui.busy());
    }
}
