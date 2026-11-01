//! Atom Rowhammer Defender — operator console (by Lucerna Labs, a division of
//! Lucerna Media). Package/binary id stays `ram-monitor-gui`; the display name
//! lives here and in the desktop entries.
//!
//! Left panel: operator-adjustable simulation settings and portable trace
//! upload. Main panel: status, counter
//! cards, an emission-rate sparkline, and the numerical trip log.
//! Simulation-only: the GUI processes only generated or uploaded observations.

use std::collections::VecDeque;

use eframe::egui;
use ram_monitor::{read_trace, Controller, Settings};

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

const APP_TITLE: &str = "Atom Rowhammer Defender";
const APP_BYLINE: &str = "by Lucerna Labs, a division of Lucerna Media";

struct App {
    controller: Controller,
    draft: Settings,
    applied: Settings,
    /// emission-rate history (observed frames per 200 ms tick), for the sparkline
    history: VecDeque<f64>,
    last_frames: u64,
    trace_path: String,
    upload_status: String,
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
            trace_path: String::new(),
            upload_status: "No trace loaded".to_string(),
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
                self.controller.load_trace(samples, name.clone());
                self.upload_status = format!("Loaded {name}: {count} samples");
                self.trace_path = path.display().to_string();
            }
            Err(error) => self.upload_status = format!("Trace rejected: {error}"),
        }
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
        let dropped: Vec<std::path::PathBuf> = ui.ctx().input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_path_buf())
                .collect()
        });
        for path in dropped {
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
                ui.label(egui::RichText::new(&self.upload_status).size(10.5).color(theme::ACCENT));
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
                // header
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(APP_TITLE)
                            .size(26.0)
                            .strong()
                            .color(theme::TEXT),
                    );
                    let (text, color) = if snap.trips > 0 {
                        ("TRIPPED", theme::DANGER)
                    } else if snap.armed {
                        ("ARMED", theme::OK)
                    } else {
                        ("DISARMED", theme::TEXT_DIM)
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
                        "Uploaded mathematical observations only; no CPU or hardware-ring input.",
                    )
                    .color(theme::WARN),
                );
                ui.label(
                    egui::RichText::new(&snap.evidence)
                        .size(10.5)
                        .color(theme::TEXT_DIM),
                );

                // counter cards
                ui.horizontal(|ui| {
                    App::card(ui, "SAMPLES", snap.samples, theme::TEXT);
                    App::card(ui, "OBSERVED", snap.frames, theme::TEAL);
                    App::card(
                        ui,
                        "TRIPS",
                        snap.trips,
                        if snap.trips > 0 {
                            theme::DANGER
                        } else {
                            theme::OK
                        },
                    );
                });
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
                    .max_height(ui.available_height() - 20.0)
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
