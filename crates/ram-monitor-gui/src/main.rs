//! Atom Rowhammer Defender — operator console (by Lucerna Labs, a division of
//! Lucerna Media). Package/binary id stays `ram-monitor-gui`; the display name
//! lives here and in the desktop entries.
//!
//! Left panel: operator-adjustable simulation settings and portable trace
//! upload. Main panel: status, counter
//! cards, an emission-rate sparkline, and the numerical trip log.
//! Rowhammer simulation uses generated or uploaded observations. The separate
//! trusted-file watch reads one operator-chosen local file without changing it.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

mod guard_ui;
mod credential_ui;
mod activity;
mod keyboard_ui;
mod network_ui;
mod overview;
use overview::View;
use activity::{Activity, now};
use keyboard_ui::KeyboardUi;
use guard_ui::GuardUi;

use eframe::egui;
use ram_monitor::{file_object::{identity_token, read_object, ObjectReading}, integrity::{compare_files, IntegrityReceipt}, pressure::{evaluate as evaluate_pressure, read_trace as read_pressure_trace, PressureReceipt}, pressure_hop::{evaluate as evaluate_pressure_hop, PressureHopReceipt}, psi_coupling::{evaluate as evaluate_psi_coupling, read_trace as read_psi_trace, PsiReceipt}, rhythm::{evaluate as evaluate_rhythm, read_trace as read_rhythm_trace, RhythmReceipt}, row_phase::{evaluate as evaluate_row_phase, read_upload as read_row_phase_upload, RowPhaseReceipt}, read_trace, Controller, Settings};
use ram_monitor::integrity_watch::{IntegrityWatch, WatchCheck};

// ---- theme: the Ordo AI agent palette (ordo-studio index.css + dashboard) ----
// inks #0a0c10/#0e1117, card #13161c (raised #181c24), parchment text #f4ecd8,
// gold primary #f4c95d, teal #0d9488, status: mint #6fd1b4, red #d86a63, amber #d4a258
mod theme {
    use eframe::egui;
    pub const BG: egui::Color32 = egui::Color32::from_rgb(0x0a, 0x0c, 0x10);
    pub const PANEL: egui::Color32 = egui::Color32::from_rgb(0x0e, 0x11, 0x17);
    pub const CARD: egui::Color32 = egui::Color32::from_rgb(0x13, 0x16, 0x1c);
    pub const CARD_RAISED: egui::Color32 = egui::Color32::from_rgb(0x18, 0x1c, 0x24);
    pub const BORDER: egui::Color32 = egui::Color32::from_rgba_premultiplied(255, 255, 255, 18);
    pub const TEXT: egui::Color32 = egui::Color32::from_rgb(0xf4, 0xec, 0xd8);
    pub const TEXT_DIM: egui::Color32 = egui::Color32::from_rgba_premultiplied(244, 236, 216, 140);
    pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(0xf4, 0xc9, 0x5d);
    pub const ACCENT_SOFT: egui::Color32 = egui::Color32::from_rgba_premultiplied(244, 201, 93, 40);
    pub const TEAL: egui::Color32 = egui::Color32::from_rgb(0x0d, 0x94, 0x88);
    pub const OK: egui::Color32 = egui::Color32::from_rgb(0x6f, 0xd1, 0xb4);
    pub const DANGER: egui::Color32 = egui::Color32::from_rgb(0xd8, 0x6a, 0x63);
    pub const WARN: egui::Color32 = egui::Color32::from_rgb(0xd4, 0xa2, 0x58);

    pub fn install(ctx: &egui::Context) {
        ctx.set_theme(egui::ThemePreference::Dark);
        ctx.style_mut_of(egui::Theme::Dark, |style| {
            let v = &mut style.visuals;
            v.panel_fill = PANEL;
            v.extreme_bg_color = BG;
            v.faint_bg_color = CARD;
            v.hyperlink_color = ACCENT;
            v.selection.bg_fill = ACCENT_SOFT;
            v.selection.stroke = egui::Stroke::new(1.0, ACCENT);
            let w = &mut v.widgets;
            for wid in [
                &mut w.noninteractive,
                &mut w.inactive,
                &mut w.hovered,
                &mut w.active,
                &mut w.open,
            ] {
                wid.corner_radius = egui::CornerRadius::same(8);
                wid.bg_stroke = egui::Stroke::new(1.0, BORDER);
            }
            w.noninteractive.bg_fill = CARD;
            w.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_DIM);
            w.inactive.bg_fill = CARD;
            w.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT);
            w.hovered.bg_fill = CARD_RAISED;
            w.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT);
            w.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT);
            w.active.bg_fill = ACCENT_SOFT;
            w.active.fg_stroke = egui::Stroke::new(1.0, TEXT);
            w.active.bg_stroke = egui::Stroke::new(1.0, ACCENT);
            w.open.bg_fill = CARD;
            w.open.fg_stroke = egui::Stroke::new(1.0, TEXT);
        });
    }
}

const APP_TITLE: &str = "Atom Defender";
const APP_BYLINE: &str = "by Lucerna Labs, a division of Lucerna Media";

struct App {
    view: View,
    activity: Activity,
    keyboard: KeyboardUi,
    network: network_ui::NetworkUi,
    file_alert_active: bool,
    controller: Controller,
    draft: Settings,
    applied: Settings,
    /// emission-rate history (observed frames per 200 ms tick), for the sparkline
    history: VecDeque<f64>,
    last_frames: u64,
    trace_path: String,
    upload_status: String,
    integrity_reference_path: String,
    integrity_observed_path: String,
    integrity_status: String,
    integrity_receipt: Option<IntegrityReceipt>,
    integrity_watch: Option<IntegrityWatch>,
    integrity_watch_last_check: Option<Instant>,
    integrity_watch_object: bool,
    guard: GuardUi,
    credential: credential_ui::CredentialUi,
    rhythm_path: String,
    rhythm_status: String,
    rhythm_receipt: Option<RhythmReceipt>,
    pressure_path: String,
    pressure_status: String,
    pressure_receipt: Option<PressureReceipt>,
    pressure_hop_path: String,
    pressure_hop_status: String,
    pressure_hop_receipt: Option<PressureHopReceipt>,
    object_path: String,
    object_status: String,
    object_baseline: Option<ObjectReading>,
    psi_path: String,
    psi_status: String,
    psi_receipt: Option<PsiReceipt>,
    row_phase_path: String,
    row_phase_status: String,
    row_phase_receipt: Option<RowPhaseReceipt>,
}

impl App {
    fn new() -> Self {
        // Demo mode is generated simulation only. Normal startup has no source
        // until the operator uploads a portable trace.
        let demo = std::env::var("RAM_MONITOR_GUI_DEMO").as_deref() == Ok("replay");
        let controller = if demo {
            Controller::spawn(|_| {
                let addrs: Vec<u64> = (0..256u64).map(|k| k % 64).collect();
                vec![Box::new(ram_monitor::LoopSource::new(addrs))
                    as Box<dyn ram_monitor::AccessSource>]
            })
        } else {
            Controller::spawn(|_| Vec::new())
        };
        let draft = Settings::default();
        Self {
            controller,
            applied: draft.clone(),
            draft,
            history: VecDeque::with_capacity(240),
            last_frames: 0,
            view: View::Overview,
            activity: Activity::open(),
            keyboard: KeyboardUi::default(),
            network: network_ui::NetworkUi::default(),
            file_alert_active: false,
            trace_path: String::new(),
            upload_status: "No trace loaded".to_string(),
            integrity_reference_path: String::new(),
            integrity_observed_path: String::new(),
            integrity_status: "No file pair compared".to_string(),
            integrity_receipt: None,
            integrity_watch: None,
            integrity_watch_last_check: None,
            integrity_watch_object: false,
            guard: GuardUi::new(),
            credential: Default::default(),
            rhythm_path: String::new(),
            rhythm_status: "No timing trace evaluated".to_string(),
            rhythm_receipt: None,
            pressure_path: String::new(),
            pressure_status: "No pressure trace evaluated".to_string(),
            pressure_receipt: None,
            pressure_hop_path: String::new(),
            pressure_hop_status: "No short-episode pressure trace evaluated".to_string(),
            pressure_hop_receipt: None,
            object_path: String::new(),
            object_status: "No file object captured".to_string(),
            object_baseline: None,
            psi_path: String::new(),
            psi_status: "No paired PSI trace evaluated".to_string(),
            psi_receipt: None,
            row_phase_path: String::new(),
            row_phase_status: "No fast row-phase trace evaluated".to_string(),
            row_phase_receipt: None,
        }
    }

    fn load_trace_path(&mut self, path: &std::path::Path) {
        match read_trace(path) {
            Ok(samples) => {
                let count = samples.len();
                let name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("uploaded trace")
                    .to_string();
                let was_enabled = self.controller.snapshot().enabled;
                self.controller.load_trace(samples, name.clone());
                self.upload_status = if was_enabled {
                    format!("Loaded {name}: {count} samples — running")
                } else {
                    format!("Loaded {name}: {count} samples — OFF")
                };
                self.trace_path = path.display().to_string();
            }
            Err(error) => self.upload_status = format!("Trace rejected: {error}"),
        }
    }

    fn compare_integrity_paths(&mut self) {
        let reference = std::path::Path::new(self.integrity_reference_path.trim());
        let observed = std::path::Path::new(self.integrity_observed_path.trim());
        match compare_files(reference, observed) {
            Ok(receipt) => {
                self.integrity_status = format!(
                    "Compared 160 bytes: {} exact byte differences (conventional count)",
                    receipt.different_bytes,
                );
                self.integrity_receipt = Some(receipt);
            }
            Err(error) => {
                self.integrity_status = format!("Comparison rejected: {error}");
                self.integrity_receipt = None;
            }
        }
    }

    fn show_watch_check(&mut self) {
        let Some(watch) = self.integrity_watch.as_ref() else { return; };
        let (status, receipt) = match watch.current() {
            WatchCheck::Compared(receipt) => {
                let status = if receipt.different_bytes == 0 {
                    format!("Watching: check {} · exact match · prior alerts {}",
                        watch.checks(), watch.ever_alerted())
                } else {
                    format!("ALERT: check {} · {} exact bytes changed · Atom long {:.9}",
                        watch.checks(), receipt.different_bytes, receipt.long_mean)
                };
                (status, Some(receipt.clone()))
            }
            WatchCheck::ComparedWithObject(receipt, changed) => {
                let status = if *changed || receipt.different_bytes > 0 {
                    format!("ALERT: check {} · {} exact bytes changed · object changed {} · Atom long {:.9}",
                        watch.checks(), receipt.different_bytes, changed, receipt.long_mean)
                } else {
                    format!("Watching: check {} · exact match · same file object · prior alerts {}",
                        watch.checks(), watch.ever_alerted())
                };
                (status, Some(receipt.clone()))
            }
            WatchCheck::ReadError(error) =>
                (format!("ALERT: check {} · observed file unavailable: {error}", watch.checks()), None),
            WatchCheck::EngineError(error) =>
                (format!("ALERT: check {} · numeric comparison failed: {error}", watch.checks()), None),
        };
        let alert = watch.current().alert();
        if alert && !self.file_alert_active { self.activity.record(now(), "File Guard", &status); }
        self.file_alert_active = alert;
        self.integrity_status = status;
        self.integrity_receipt = receipt;
    }

    fn start_integrity_watch(&mut self) {
        self.file_alert_active = false;
        let reference = std::path::Path::new(self.integrity_reference_path.trim());
        let observed = std::path::Path::new(self.integrity_observed_path.trim());
        let started = if self.integrity_watch_object {
            IntegrityWatch::start_with_object(reference, observed)
        } else {
            IntegrityWatch::start(reference, observed)
        };
        match started {
            Ok(watch) => {
                self.integrity_watch = Some(watch);
                self.integrity_watch_last_check = Some(Instant::now());
                self.show_watch_check();
            }
            Err(error) => {
                self.integrity_watch = None;
                self.integrity_watch_last_check = None;
                self.integrity_status = format!("Watch rejected: {error}");
                self.integrity_receipt = None;
            }
        }
    }

    fn poll_integrity_watch(&mut self) {
        let should_poll = self.integrity_watch_last_check
            .is_some_and(|last| last.elapsed() >= Duration::from_secs(1));
        if should_poll {
            if let Some(watch) = self.integrity_watch.as_mut() {
                watch.poll();
                self.integrity_watch_last_check = Some(Instant::now());
                self.show_watch_check();
            }
        }
    }

    fn evaluate_rhythm_path(&mut self) {
        let path = std::path::Path::new(self.rhythm_path.trim());
        let result = read_rhythm_trace(path).and_then(|events| evaluate_rhythm(&events));
        match result {
            Ok(receipt) => {
                self.rhythm_status = format!(
                    "Evaluated 64 events; first numerical crossing: {}",
                    receipt.first_crossing.map_or("none".to_string(), |n| n.to_string())
                );
                self.rhythm_receipt = Some(receipt);
            }
            Err(error) => {
                self.rhythm_status = format!("Timing trace rejected: {error}");
                self.rhythm_receipt = None;
            }
        }
    }

    fn evaluate_pressure_path(&mut self) {
        let path = std::path::Path::new(self.pressure_path.trim());
        let result = read_pressure_trace(path).and_then(|records| evaluate_pressure(&records));
        match result {
            Ok(receipt) => {
                self.pressure_status = format!(
                    "Evaluated 96 intervals; first numerical crossing: {}",
                    receipt.first_crossing.map_or("none".to_string(), |n| n.to_string())
                );
                self.pressure_receipt = Some(receipt);
            }
            Err(error) => {
                self.pressure_status = format!("Pressure trace rejected: {error}");
                self.pressure_receipt = None;
            }
        }
    }

    fn evaluate_pressure_hop_path(&mut self) {
        let path = std::path::Path::new(self.pressure_hop_path.trim());
        let result = read_pressure_trace(path).and_then(|records| evaluate_pressure_hop(&records));
        match result {
            Ok(receipt) => {
                self.pressure_hop_status = format!(
                    "Evaluated 96 intervals; first two-hop crossing: {}",
                    receipt.first_crossing.map_or("none".to_string(), |n| n.to_string())
                );
                self.pressure_hop_receipt = Some(receipt);
            }
            Err(error) => {
                self.pressure_hop_status = format!("Short-episode trace rejected: {error}");
                self.pressure_hop_receipt = None;
            }
        }
    }

    fn capture_object_path(&mut self) {
        let path = std::path::Path::new(self.object_path.trim());
        match read_object(path) {
            Ok(reading) => {
                self.object_status = format!(
                    "Captured object {} · links {} · size {}",
                    identity_token(reading.identity), reading.links, reading.size
                );
                self.object_baseline = Some(reading);
            }
            Err(error) => {
                self.object_status = format!("Capture rejected: {error}");
                self.object_baseline = None;
            }
        }
    }

    fn check_object_path(&mut self) {
        let Some(baseline) = self.object_baseline else {
            self.object_status = "Capture a file object first".to_string();
            return;
        };
        let path = std::path::Path::new(self.object_path.trim());
        match read_object(path) {
            Ok(current) => {
                let relation = if current.identity == baseline.identity {
                    "Same file object"
                } else {
                    "Different file object"
                };
                self.object_status = format!(
                    "{relation} · baseline {} · current {} · links {} · size {}",
                    identity_token(baseline.identity), identity_token(current.identity),
                    current.links, current.size
                );
            }
            Err(error) => {
                self.object_status = format!("Check rejected: {error}");
            }
        }
    }

    fn evaluate_psi_path(&mut self) {
        let path = std::path::Path::new(self.psi_path.trim());
        let result = read_psi_trace(path).and_then(|records| evaluate_psi_coupling(&records));
        match result {
            Ok(receipt) => {
                self.psi_status = format!(
                    "Evaluated 96 paired intervals; first joint crossing: {}",
                    receipt.first_crossing.map_or("none".to_string(), |n| n.to_string())
                );
                self.psi_receipt = Some(receipt);
            }
            Err(error) => {
                self.psi_status = format!("Paired PSI trace rejected: {error}");
                self.psi_receipt = None;
            }
        }
    }

    fn evaluate_row_phase_path(&mut self) {
        let path = std::path::Path::new(self.row_phase_path.trim());
        let result = read_row_phase_upload(path).and_then(|samples| evaluate_row_phase(&samples));
        match result {
            Ok(receipt) => {
                self.row_phase_status = format!(
                    "Evaluated 4096 mathematical addresses; first phase crossing: {}",
                    receipt.first_crossing.map_or("none".to_string(), |n| n.to_string())
                );
                self.row_phase_receipt = Some(receipt);
            }
            Err(error) => {
                self.row_phase_status = format!("Row phase trace rejected: {error}");
                self.row_phase_receipt = None;
            }
        }
    }

    fn file_controls(&mut self, ui: &mut egui::Ui) {
                self.credential.show(ui);
                ui.collapsing("File object continuity (read only)", |ui| {
                    ui.label("Capture a regular file's object identity, then check the current path. Same-content replacement changes the object; in-place edits do not. This is metadata, not an attacker verdict.");
                    ui.text_edit_singleline(&mut self.object_path);
                    ui.horizontal(|ui| {
                        if ui.button("Capture identity").clicked() {
                            self.capture_object_path();
                        }
                        if ui.button("Check object").clicked() {
                            self.check_object_path();
                        }
                    });
                    ui.label(&self.object_status);
                });

                egui::CollapsingHeader::new("Selected file (160 bytes)").default_open(true).show(ui, |ui| {
                    ui.label("Choose a trusted reference and an observed file. Compare once or watch read-only about every second. Exact bytes, optional object changes, or read errors alert; Atom material is a separate numeric receipt. The reference is captured when watching starts.");
                    let watching = self.integrity_watch.is_some();
                    let guarded = self.guard.busy();
                    let locked = watching || guarded;
                    ui.label("Reference file");
                    ui.add_enabled(!locked, egui::TextEdit::singleline(&mut self.integrity_reference_path));
                    ui.label("Observed file");
                    ui.add_enabled(!locked, egui::TextEdit::singleline(&mut self.integrity_observed_path));
                    ui.add_enabled(!locked, egui::Checkbox::new(&mut self.integrity_watch_object,
                        "Also watch file object identity (needs birth time)"));
                    if ui.add_enabled(!locked, egui::Button::new("Compare files")).clicked() {
                        self.compare_integrity_paths();
                    }
                    if watching {
                        if ui.button("Stop watch").clicked() {
                            let checks = self.integrity_watch.as_ref().map_or(0, IntegrityWatch::checks);
                            self.integrity_watch = None;
                            self.integrity_watch_last_check = None;
                            self.integrity_status = format!("Stopped after {checks} checks. {}", self.integrity_status);
                        }
                    } else if ui.add_enabled(!guarded, egui::Button::new("Start watch")).clicked() {
                        self.start_integrity_watch();
                    }
                    let alert = self.integrity_watch.as_ref().is_some_and(|watch| watch.current().alert());
                    ui.colored_label(if alert { theme::DANGER } else { theme::TEXT }, &self.integrity_status);
                    ui.separator();
                    ui.label("Automatic protection restores the captured trusted bytes after an event and saves changed contents first. It modifies only the selected observed file when armed.");
                    ui.label("Private evidence directory (owned by you, mode 0700)");
                    ui.add_enabled(!locked, egui::TextEdit::singleline(&mut self.guard.evidence_dir));
                    if self.guard.active() {
                        if ui.button("Disarm automatic restore").clicked() { self.guard.stop(); }
                    } else if ui.add_enabled(!locked, egui::Button::new("ARM AUTOMATIC RESTORE")).clicked() {
                        let reference = std::path::Path::new(self.integrity_reference_path.trim());
                        let observed = std::path::Path::new(self.integrity_observed_path.trim());
                        if let Err(error) = self.guard.arm(ui.ctx(), reference, observed) {
                            self.guard.status = format!("Guard rejected: {error}");
                            self.guard.failed = true;
                        }
                    }
                    ui.colored_label(if self.guard.failed { theme::DANGER } else { theme::OK }, &self.guard.status);
                    if let Some(receipt) = &self.integrity_receipt {
                        ui.label(format!(
                            "Atom material mean: long {:.9}, original {:.9}, peak {:.9}",
                            receipt.long_mean, receipt.original_mean, receipt.peak_mean
                        ));
                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            for tick in &receipt.ticks {
                                ui.monospace(format!(
                                    "{:03} {:.5} {:.5} {:.5} {:.5} | mean {:.5}",
                                    tick.index, tick.materials[0], tick.materials[1],
                                    tick.materials[2], tick.materials[3], tick.mean
                                ));
                            }
                        });
                    }
                });

    }

    fn card(ui: &mut egui::Ui, label: &str, value: impl ToString, accent: egui::Color32) {
        egui::Frame::new()
            .fill(theme::CARD)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .corner_radius(egui::CornerRadius::same(10))
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.set_min_width(150.0);
                ui.label(egui::RichText::new(label).size(11.0).color(theme::TEXT_DIM));
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(value.to_string())
                        .size(24.0)
                        .strong()
                        .color(accent),
                );
            });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_integrity_watch();
        self.guard.poll();
        self.credential.poll();
        for alert in self.guard.alerts.drain(..) { self.activity.record(now(), "File Guard", &alert); }
        for alert in self.controller.take_alerts() { self.activity.record(now(), "Rowhammer", &alert); }
        self.keyboard.poll(&mut self.activity);
        self.network.poll(&mut self.activity);
        let dropped: Vec<std::path::PathBuf> = ui.ctx().input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_path_buf())
                .collect()
        });
        for path in dropped {
            if self.view == View::Network || path.extension().and_then(|x| x.to_str()).is_some_and(|s| ["pcap","pcapng","cap"].iter().any(|e| s.eq_ignore_ascii_case(e))) {
                self.view=View::Network;
                self.network.set_capture(path);
                continue;
            }
            self.view = View::Rowhammer;
            self.load_trace_path(&path);
        }
        let snap = self.controller.snapshot();
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(200));

        // sparkline history: observed frames per tick
        let frames = snap.frames;
        self.history
            .push_back(frames.saturating_sub(self.last_frames) as f64);
        if self.history.len() > 240 {
            self.history.pop_front();
        }
        self.last_frames = frames;

        self.navigation(ui);
        if self.view != View::Rowhammer {
            self.show_module(ui, &snap);
            return;
        }
        // ---- left: settings ----
        egui::Panel::left("settings")
            .frame(egui::Frame::new().fill(theme::PANEL).inner_margin(egui::Margin::symmetric(14, 12)))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("SETTINGS").size(13.0).strong().color(theme::ACCENT));
                ui.label(egui::RichText::new("applied only via the Apply button — never inferred from traffic").size(10.5).color(theme::TEXT_DIM));
                ui.add_space(10.0);

                ui.label(egui::RichText::new("Sensitivity").size(12.0).strong().color(theme::TEXT));
                ui.label("window (samples)");
                ui.add(egui::Slider::new(&mut self.draft.window, 8..=1024).logarithmic(true));
                ui.label("trip threshold (top region share)");
                ui.add(egui::Slider::new(&mut self.draft.thresh, 1..=self.draft.window));
                ui.label(egui::RichText::new(format!(
                    "{:.0}% of the window must be one region",
                    100.0 * self.draft.thresh as f64 / self.draft.window as f64
                )).size(10.5).color(theme::TEXT_DIM));
                ui.add_space(10.0);

                ui.label(egui::RichText::new("Row geometry").size(12.0).strong().color(theme::TEXT));
                ui.horizontal(|ui| {
                    ui.label("line shift");
                    ui.add(egui::Slider::new(&mut self.draft.row_shift, 6..=12));
                    ui.label("region bits");
                    ui.add(egui::Slider::new(&mut self.draft.row_bits, 4..=16));
                });
                ui.add_space(10.0);

                ui.label(egui::RichText::new("Portable trace").size(12.0).strong().color(theme::TEXT));
                ui.label(egui::RichText::new("Drop a trace file anywhere, or enter its path.").size(10.5).color(theme::TEXT_DIM));
                ui.text_edit_singleline(&mut self.trace_path);
                if ui.button("Load trace").clicked() {
                    let path = std::path::PathBuf::from(self.trace_path.trim());
                    self.load_trace_path(&path);
                }
                let upload_status = match self.upload_status.rsplit_once(" — ") {
                    Some((prefix, _)) if prefix.starts_with("Loaded ") => {
                        let state = if snap.halted {
                            "HALTED"
                        } else if snap.enabled {
                            "ON"
                        } else {
                            "OFF"
                        };
                        format!("{prefix} — {state}")
                    }
                    _ => self.upload_status.clone(),
                };
                ui.label(egui::RichText::new(upload_status).size(10.5).color(theme::ACCENT));
                let toggle_text = if snap.enabled { "TURN OFF" } else { "TURN ON" };
                let toggle_color = if snap.enabled { theme::DANGER } else { theme::OK };
                let toggle = ui.add(
                    egui::Button::new(
                        egui::RichText::new(toggle_text)
                            .size(16.0)
                            .strong()
                            .color(theme::BG),
                    )
                    .fill(toggle_color)
                    .min_size(egui::vec2(230.0, 40.0)),
                );
                if toggle.clicked() {
                    self.controller.set_enabled(!snap.enabled);
                }
                ui.label(
                    egui::RichText::new("Uploads are staged only. ON starts from the beginning; OFF stops immediately.")
                        .size(10.5)
                        .color(theme::TEXT_DIM),
                );
                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    let apply = ui.add(egui::Button::new("Apply settings").min_size(egui::vec2(110.0, 30.0)));
                    if apply.clicked() {
                        self.controller.apply(self.draft.clone());
                        self.applied = self.draft.clone();
                    }
                    if ui.add(egui::Button::new("Reset detector").min_size(egui::vec2(110.0, 30.0))).clicked() {
                        self.controller.reset();
                    }
                });
                if self.draft != self.applied {
                    ui.colored_label(theme::WARN, "● unapplied changes");
                }
                ui.add_space(10.0);
                ui.label(egui::RichText::new(
                    "False flags? Raise the threshold or enlarge the window, then Apply. Reset re-arms the detector.",
                ).size(10.5).color(theme::TEXT_DIM));
            });

        // ---- main panel ----
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::symmetric(18, 14)),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("main_panel")
                    .show(ui, |ui| {
                // header
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Rowhammer · trace analysis")
                            .size(26.0)
                            .strong()
                            .color(theme::TEXT),
                    );
                    let (text, color) = if snap.halted {
                        ("HALTED", theme::DANGER)
                    } else if snap.enabled {
                        ("ON", theme::OK)
                    } else {
                        ("OFF", theme::TEXT_DIM)
                    };
                    egui::Frame::new()
                        .fill(color.gamma_multiply(0.15))
                        .stroke(egui::Stroke::new(1.5, color))
                        .corner_radius(egui::CornerRadius::same(14))
                        .inner_margin(egui::Margin::symmetric(14, 6))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(text).size(16.0).strong().color(color));
                        });
                });
                ui.label(
                    egui::RichText::new(APP_BYLINE)
                        .size(12.0)
                        .color(theme::TEXT_DIM),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "simulation trace: {} — numerical receipts",
                        snap.source_name
                    ))
                    .size(11.0)
                    .color(theme::TEXT_DIM),
                );
                ui.add_space(12.0);

                ui.label(
                    egui::RichText::new(
                        "Rowhammer: uploaded traces only, no live hardware input. File watch: selected local file.",
                    )
                    .color(theme::WARN),
                );
                ui.label(
                    egui::RichText::new(&snap.evidence)
                        .size(10.5)
                        .color(theme::TEXT_DIM),
                );

                let narrow = ui.available_width() < 560.0;
                if let Some(result) = snap.model_report {
                    match result {
                        Ok(report) => {
                            let r = report.reading;
                            let rupture = |sample: Option<u64>| sample.map_or_else(
                                || "no swarm rupture".to_string(), |n| format!("swarm rupture at sample {n}"));
                            ui.strong(format!("Trace models · {} / {} samples", r.samples, snap.uploaded_samples));
                            if snap.halted {
                                ui.label(format!("Legacy replay stopped at sample {}", snap.samples));
                            }
                            ui.label(format!("Timing (50 µs) · {} repeats · {}",
                                r.timing_matches, rupture(report.timing_rupture)));
                            let threshold = r.first_heat_crossing.map_or_else(
                                || "below 128".to_string(), |n| format!("crossed 128 at sample {n}"));
                            ui.label(format!("Heat · peak {:.2} · {threshold} · {}",
                                r.peak_heat, rupture(report.heat_rupture)));
                        }
                        Err(error) => { ui.colored_label(theme::WARN, format!("Trace models unavailable · {error}")); }
                    }
                    ui.add_space(4.0);
                }
                egui::ScrollArea::vertical()
                    .id_salt("observation_sections")
                    .max_height(if narrow { 140.0 } else { 300.0 })
                    .show(ui, |ui| {
                ui.collapsing("Experimental row phase trace (4096 events)", |ui| {
                    ui.label("Upload mathematical time_ns,address observations. This phase-alignment field also responds to many two-/four-fingerprint rotations and legitimate hot-cache loops. It is not a verified one-row or physical Rowhammer detector.");
                    ui.text_edit_singleline(&mut self.row_phase_path);
                    if ui.button("Evaluate row phase").clicked() {
                        self.evaluate_row_phase_path();
                    }
                    ui.label(&self.row_phase_status);
                    if let Some(receipt) = &self.row_phase_receipt {
                        ui.label(format!(
                            "Peak material {:.6} · final {:.6} · peak phase power {:.6}",
                            receipt.peak_energy, receipt.final_energy, receipt.peak_power
                        ));
                        ui.label("Selected ticks below; the CLI prints all 4096 numerical ticks.");
                        let mut indices: Vec<usize> = (0..8).collect();
                        if let Some(n) = receipt.first_crossing {
                            indices.extend(n.saturating_sub(3)..(n + 2).min(receipt.ticks.len()));
                        }
                        indices.extend(receipt.ticks.len().saturating_sub(8)..receipt.ticks.len());
                        indices.sort_unstable();
                        indices.dedup();
                        for index in indices {
                            if let Some(tick) = receipt.ticks.get(index) {
                                ui.monospace(format!(
                                    "{:04} E {:.6} power {:.6} bond {:.6}",
                                    tick.index, tick.energy, tick.power, tick.binding
                                ));
                            }
                        }
                    }
                });

                ui.collapsing("Timing trace (64 events)", |ui| {
                    ui.label("One uploaded time_ns,bytes_out stream scoped to a connection pair. Numerical rhythm only; scheduled traffic may also respond.");
                    ui.text_edit_singleline(&mut self.rhythm_path);
                    if ui.button("Evaluate rhythm").clicked() {
                        self.evaluate_rhythm_path();
                    }
                    ui.label(&self.rhythm_status);
                    if let Some(receipt) = &self.rhythm_receipt {
                        ui.label(format!(
                            "Scale {:.3} ns · peak {:.6} · final {:.6}",
                            receipt.tau_ns, receipt.peak_energy, receipt.final_energy
                        ));
                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            for tick in &receipt.ticks {
                                ui.monospace(format!(
                                    "{:02} E {:.5} fundamental {:.5} overtone {:.5}",
                                    tick.index, tick.energy, tick.fundamental, tick.overtone
                                ));
                            }
                        });
                    }
                });

                ui.collapsing("Memory pressure trace (96 intervals)", |ui| {
                    ui.label("Upload 97 one-second time_ns,some_total_us records from one Linux memory PSI source. Numerical pressure only; sustained legitimate workloads may also respond.");
                    ui.text_edit_singleline(&mut self.pressure_path);
                    if ui.button("Evaluate pressure").clicked() {
                        self.evaluate_pressure_path();
                    }
                    ui.label(&self.pressure_status);
                    if let Some(receipt) = &self.pressure_receipt {
                        ui.label(format!(
                            "Peak stall fraction {:.4} · peak material {:.6} · final {:.6}",
                            receipt.peak_fraction, receipt.peak_energy, receipt.final_energy
                        ));
                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            for tick in &receipt.ticks {
                                ui.monospace(format!(
                                    "{:02} stall {:.4} E {:.6} pop {:.4} bond {:.4}",
                                    tick.index, tick.fraction, tick.energy,
                                    tick.population, tick.binding
                                ));
                            }
                        });
                    }
                });

                ui.collapsing("Memory pressure short episodes (96 intervals)", |ui| {
                    ui.label("Upload the same 97-record Linux memory PSI format for the independent two-hop material. Numerical response only; ordinary scheduled work can also cross.");
                    ui.text_edit_singleline(&mut self.pressure_hop_path);
                    if ui.button("Evaluate short episodes").clicked() {
                        self.evaluate_pressure_hop_path();
                    }
                    ui.label(&self.pressure_hop_status);
                    if let Some(receipt) = &self.pressure_hop_receipt {
                        ui.label(format!(
                            "Peak stall fraction {:.4} · peak two-hop material {:.6} · final {:.6}",
                            receipt.peak_fraction, receipt.peak_energy, receipt.final_energy
                        ));
                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            for tick in &receipt.ticks {
                                ui.monospace(format!(
                                    "{:02} stall {:.4} E {:.6} site2 {:.4} bond {:.4}",
                                    tick.index, tick.fraction, tick.energy,
                                    tick.occupation[2], tick.binding
                                ));
                            }
                        });
                    }
                });

                ui.collapsing("Memory + I/O pressure (96 intervals)", |ui| {
                    ui.label("Upload 97 time_ns,memory_some_total_us,io_some_total_us records sampled together. Numerical joint material only; legitimate simultaneous pressure can also respond.");
                    ui.text_edit_singleline(&mut self.psi_path);
                    if ui.button("Evaluate joint pressure").clicked() {
                        self.evaluate_psi_path();
                    }
                    ui.label(&self.psi_status);
                    if let Some(receipt) = &self.psi_receipt {
                        ui.label(format!(
                            "Peak memory {:.4} · I/O {:.4} · joint material {:.6} · final {:.6}",
                            receipt.peak_memory_fraction, receipt.peak_io_fraction,
                            receipt.peak_energy, receipt.final_energy
                        ));
                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            for tick in &receipt.ticks {
                                ui.monospace(format!(
                                    "{:02} mem {:.4} io {:.4} E {:.6} joint {:.4}",
                                    tick.index, tick.memory_fraction, tick.io_fraction,
                                    tick.energy, tick.occupation[3]
                                ));
                            }
                        });
                    }
                });
                    });

                // counter cards
                let trips_color = if snap.trips > 0 { theme::DANGER } else { theme::OK };
                let halted_color = if snap.halted { theme::DANGER } else { theme::OK };
                let power_color = if snap.enabled { theme::OK } else { theme::TEXT_DIM };
                let power = if snap.enabled { "ON" } else { "OFF" };
                if narrow {
                    ui.columns(2, |cols| {
                        App::card(&mut cols[0], "SAMPLES", snap.samples, theme::TEXT);
                        App::card(&mut cols[1], "OBSERVED", snap.frames, theme::TEAL);
                    });
                    ui.columns(2, |cols| {
                        App::card(&mut cols[0], "TRIPS", snap.trips, trips_color);
                        App::card(&mut cols[1], "HALTED", snap.halted, halted_color);
                    });
                    App::card(ui, "POWER", power, power_color);
                } else {
                    ui.columns(3, |cols| {
                        App::card(&mut cols[0], "SAMPLES", snap.samples, theme::TEXT);
                        App::card(&mut cols[1], "OBSERVED", snap.frames, theme::TEAL);
                        App::card(&mut cols[2], "TRIPS", snap.trips, trips_color);
                    });
                    ui.columns(2, |cols| {
                        App::card(&mut cols[0], "HALTED", snap.halted, halted_color);
                        App::card(&mut cols[1], "POWER", power, power_color);
                    });
                }
                ui.add_space(12.0);

                // sparkline of emission rate
                ui.label(
                    egui::RichText::new("emission rate (observed frames / 200 ms)")
                        .size(11.0)
                        .color(theme::TEXT_DIM),
                );
                let desired = egui::vec2(ui.available_width(), 70.0);
                let (rect, _resp) = ui.allocate_exact_size(desired, egui::Sense::hover());
                let painter = ui.painter();
                painter.rect_filled(rect, egui::CornerRadius::same(8), theme::CARD);
                painter.rect_stroke(
                    rect,
                    egui::CornerRadius::same(8),
                    egui::Stroke::new(1.0, theme::BORDER),
                    egui::StrokeKind::Inside,
                );
                let max = self.history.iter().copied().fold(0.0_f64, f64::max);
                if max > 0.0 {
                    let n = self.history.len();
                    let points: Vec<egui::Pos2> = self
                        .history
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let x = rect.left() + (i as f32 / n.max(1) as f32) * rect.width();
                            let y = rect.bottom() - (*v as f32 / max as f32) * rect.height();
                            egui::pos2(x, y)
                        })
                        .collect();
                    if points.len() > 1 {
                        painter.add(egui::Shape::line(
                            points,
                            egui::Stroke::new(2.0, theme::ACCENT),
                        ));
                    }
                }
                ui.add_space(12.0);

                if let Some(r) = &snap.last_receipt {
                    ui.colored_label(theme::DANGER, format!("last receipt: {r}"));
                    ui.add_space(6.0);
                }
                ui.label(
                    egui::RichText::new("trip log (numerical receipts, newest last)")
                        .size(11.0)
                        .color(theme::TEXT_DIM),
                );
                egui::ScrollArea::vertical()
                    .max_height(if narrow { 150.0 } else { 220.0 })
                    .show(ui, |ui| {
                        for line in &snap.log {
                            ui.label(
                                egui::RichText::new(line)
                                    .size(11.0)
                                    .color(theme::TEXT)
                                    .monospace(),
                            );
                        }
                    });
                    });
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(format!("{APP_TITLE} — {APP_BYLINE}"))
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([880.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "atom-rowhammer-defender",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Ok(Box::new(App::new()))
        }),
    )
}
