//! The privileged launcher is fixed to the package-owned absolute helper path.
use eframe::egui;
use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub struct CredentialUi {
    file: String,
    pid: String,
    child: Option<Child>,
    receipt: Arc<Mutex<String>>,
    status: String,
}
impl CredentialUi {
    fn arm(&mut self, ctx: &egui::Context) -> std::io::Result<()> {
        let mut child = Command::new("/usr/bin/pkexec")
            .arg("/opt/ram-monitor/atom-credential-guard")
            .args([self.file.as_str(), self.pid.trim()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        self.receipt = Arc::default();
        fn listen(
            pipe: impl std::io::Read + Send + 'static,
            receipt: Arc<Mutex<String>>,
            ctx: egui::Context,
        ) {
            std::thread::spawn(move || {
                for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                    if let Ok(mut current) = receipt.lock() {
                        *current = line;
                    }
                    ctx.request_repaint();
                }
            });
        }
        if let Some(out) = child.stdout.take() {
            listen(out, self.receipt.clone(), ctx.clone());
        }
        if let Some(err) = child.stderr.take() {
            listen(err, self.receipt.clone(), ctx.clone());
        }
        self.status = "Waiting for authorization and kernel confirmation".into();
        self.child = Some(child);
        Ok(())
    }
    pub fn summary(&self) -> Option<&'static str> {
        self.child.as_ref()?;
        if self.status == "Stopping" {
            return Some("Credential guard stopping");
        }
        let confirmed = self
            .receipt
            .lock()
            .is_ok_and(|s| s.starts_with("ARMED ") || s.starts_with("ACTIVE "));
        Some(if confirmed {
            "Credential guard armed"
        } else {
            "Credential guard starting"
        })
    }
    pub fn poll(&mut self) {
        if let Some(child) = self.child.as_mut() {
            if let Ok(Some(exit)) = child.try_wait() {
                self.status = format!("Off · helper exited ({exit})");
                self.child = None;
            }
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Credential file access", |ui| {
            ui.label("Protect new opens of one existing file. Select the PID of the running application allowed to open it. Previously open handles remain usable. Protection ends on stop, process exit, or file replacement.");
            let running = self.child.is_some();
            ui.add_enabled_ui(!running, |ui| {
                ui.label("Credential file (absolute path)");
                ui.text_edit_singleline(&mut self.file);
                ui.label("Approved application PID");
                ui.text_edit_singleline(&mut self.pid);
                if ui.add_enabled(std::path::Path::new(&self.file).is_absolute() && self.pid.trim().parse::<i32>().is_ok_and(|p|p>1), egui::Button::new("Arm credential guard")).clicked() {
                    if let Err(e) = self.arm(ui.ctx()) { self.status = format!("Off · {e}"); }
                }
            });
            if let Some(child) = self.child.as_mut() {
                if ui.button("Disarm credential guard").clicked() {
                    child.stdin.take();
                    self.status = "Stopping".into();
                }
            }
            let latest = self.receipt.lock().map(|s|s.clone()).unwrap_or_default();
            if running && !latest.is_empty() && self.status != "Stopping" { ui.monospace(latest); }
            else {
                ui.label(if self.status.is_empty() {"Off"} else {&self.status});
                if !running && !latest.is_empty() { ui.label(format!("Last receipt: {latest}")); }
            }
        });
    }
}
