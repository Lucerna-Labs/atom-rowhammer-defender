use super::activity::{now, Activity};
use atom_network_lightcone::{Message, Report};
use eframe::egui;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{sync_channel, Receiver, TryRecvError};

struct Run {
    child: Child,
    rx: Receiver<Result<Message, String>>,
    live: bool,
    finished_report: bool,
}
impl Drop for Run {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct NetworkUi {
    pack: String,
    capture: String,
    ether: String,
    run: Option<Run>,
    report: Option<Report>,
    status: &'static str,
    error: Option<String>,
}
impl Default for NetworkUi {
    fn default() -> Self {
        let pack = std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|p| p.join("lightcone-learn/host.pathpack"))
            .unwrap_or_default();
        Self {
            pack: pack.to_string_lossy().into_owned(),
            capture: String::new(),
            ether: "capture".into(),
            run: None,
            report: None,
            status: "Off",
            error: None,
        }
    }
}
impl NetworkUi {
    pub fn status(&self) -> &'static str {
        self.status
    }
    pub fn set_capture(&mut self, path: PathBuf) {
        if self.run.is_none() {
            self.capture = path.to_string_lossy().into_owned();
        }
    }
    fn start(&mut self, live: bool) -> Result<(), String> {
        self.report = None;
        self.error = None;
        if self.pack.trim().is_empty() {
            return Err("Choose a reference path pack.".into());
        }
        if !live && self.capture.trim().is_empty() {
            return Err("Choose a capture file.".into());
        }
        let exe = std::env::current_exe()
            .map_err(|e| e.to_string())?
            .with_file_name("atom-network-lightcone");
        let mut cmd = Command::new(exe);
        cmd.env_clear()
            .arg(if live { "live" } else { "replay-managed" })
            .arg(self.pack.trim());
        if !live {
            cmd.arg(self.capture.trim()).arg(&self.ether);
        }
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Start network worker: {e}"))?;
        let pipe = child.stdout.take().ok_or("worker output unavailable")?;
        let (tx, rx) = sync_channel(8);
        let reader = std::thread::Builder::new()
            .name("lightcone-reports".into())
            .spawn(move || {
                for line in BufReader::new(pipe).lines() {
                    let message = line
                        .map_err(|e| e.to_string())
                        .and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string()));
                    if tx.send(message).is_err() {
                        break;
                    }
                }
            });
        if let Err(e) = reader {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e.to_string());
        }
        self.run = Some(Run {
            child,
            rx,
            live,
            finished_report: false,
        });
        self.status = "Starting";
        Ok(())
    }
    fn fail(&mut self, msg: String, log: &mut Activity) {
        self.run.take();
        self.status = "Unavailable";
        log.record(now(), "Network Lightcone", &msg);
        self.error = Some(msg);
    }
    pub fn poll(&mut self, log: &mut Activity) {
        let Some(run) = self.run.as_mut() else {
            return;
        };
        loop {
            match run.rx.try_recv() {
                Ok(Ok(Message::Ready(r))) => {
                    if self.status != "Stopping" {
                        self.status = if run.live { "Monitoring" } else { "Replaying" };
                    }
                    self.report = Some(r);
                }
                Ok(Ok(Message::Progress(r))) => self.report = Some(r),
                Ok(Ok(Message::Complete(r))) => {
                    self.report = Some(r);
                    run.finished_report = true;
                }
                Ok(Ok(Message::Error(e))) | Ok(Err(e)) => {
                    self.fail(e, log);
                    return;
                }
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        match run.child.try_wait() {
            Ok(None) => {}
            Ok(Some(code)) if code.success() && run.finished_report => {
                self.status = if self.status == "Stopping" {
                    "Stopped"
                } else {
                    "Complete"
                };
                if let Some(r) = &self.report {
                    log.record(now(),"Network Lightcone",&format!("{} · {} receipts · {} exact matches · {} reverse-deny matches · {} truncated · kernel drops {}",self.status,r.samples,r.matched,r.denied,r.truncated,r.kernel_drops));
                }
                self.run.take();
            }
            Ok(Some(code)) => self.fail(
                format!("Network worker ended without a complete report ({code})."),
                log,
            ),
            Err(e) => self.fail(e.to_string(), log),
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui, log: &mut Activity) {
        ui.heading("Network Lightcone");
        ui.label("Compare observed packet paths against a reference pack. Traffic forwarding is controlled by the host.");
        ui.add_space(8.0);
        let busy = self.run.is_some();
        ui.label("Reference path pack");
        ui.add_enabled(
            !busy,
            egui::TextEdit::singleline(&mut self.pack).desired_width(530.0),
        );
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!busy, egui::Button::new("Start live monitoring"))
                .clicked()
            {
                if let Err(e) = self.start(true) {
                    self.fail(e, log);
                }
            }
            if ui
                .add_enabled(busy && self.status != "Stopping", egui::Button::new("Stop"))
                .clicked()
            {
                if let Some(run) = &mut self.run {
                    drop(run.child.stdin.take());
                    self.status = "Stopping";
                }
            }
            ui.strong(self.status);
        });
        ui.add_space(8.0);
        ui.label("Replay a capture");
        ui.add_enabled(
            !busy,
            egui::TextEdit::singleline(&mut self.capture)
                .hint_text("Path to a PCAP or PCAPNG file")
                .desired_width(530.0),
        );
        ui.horizontal_wrapped(|ui| {
            ui.add_enabled_ui(!busy, |ui| {
                egui::ComboBox::from_id_salt("lightcone_direction")
                    .selected_text(match self.ether.as_str() {
                        "inbound" => "Ethernet: inbound",
                        "outbound" => "Ethernet: outbound",
                        _ => "Use captured direction",
                    })
                    .show_ui(ui, |ui| {
                        for (key, name) in [
                            ("capture", "Use captured direction"),
                            ("inbound", "Ethernet: inbound"),
                            ("outbound", "Ethernet: outbound"),
                        ] {
                            ui.selectable_value(&mut self.ether, key.into(), name);
                        }
                    });
            });
            if ui
                .add_enabled(!busy, egui::Button::new("Analyze capture"))
                .clicked()
            {
                if let Err(e) = self.start(false) {
                    self.fail(e, log);
                }
            }
        });
        if let Some(e) = &self.error {
            ui.colored_label(super::theme::WARN, e);
        }
        if let Some(r) = &self.report {
            ui.separator();
            ui.strong(format!("{} · {} reference paths", r.role, r.trained_paths));
            ui.label(format!(
                "{} receipts · {} inbound · {} outbound",
                r.samples, r.inbound, r.outbound
            ));
            ui.label(format!(
                "{} exact matches · {} reverse-deny matches · {} other misses",
                r.matched,
                r.denied,
                r.samples.saturating_sub(r.matched + r.denied)
            ));
            ui.label(format!(
                "{} without a represented path · {} truncated · {} without link metadata",
                r.unrepresented, r.truncated, r.missing_link_metadata
            ));
            ui.label(format!(
                "{} kernel drops · {} interface drops reported",
                r.kernel_drops, r.interface_drops
            ));
            if let Some(last) = &r.last {
                let result = if last.admitted {
                    "exact match"
                } else if last.denied {
                    "reverse-deny match"
                } else {
                    "miss"
                };
                ui.collapsing("Latest native receipt", |ui| {
                    ui.label(format!(
                        "#{} · {} · {} · {} symbols · {result}",
                        last.sequence,
                        last.direction,
                        last.kind,
                        last.path.len()
                    ));
                    if !last.path.is_empty() {
                        ui.label("Exact path (field:hex value)");
                        ui.monospace(last.path.join(" "));
                    }
                    ui.label("Captured frame SHA-256");
                    ui.monospace(&last.query_sha256);
                });
            }
            ui.collapsing("Reference pack identity", |ui| {
                ui.monospace(&r.pack_sha256);
                ui.label(format!(
                    "{} reverse-deny paths · exact matching · unsealed",
                    r.deny_paths
                ));
            });
        }
    }
}
