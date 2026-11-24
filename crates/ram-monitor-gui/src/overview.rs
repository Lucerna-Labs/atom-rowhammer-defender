use super::{theme, App};
use eframe::egui;
use ram_monitor::Snapshot;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    Overview,
    Rowhammer,
    File,
    Keyboard,
    Network,
    Alerts,
}
impl App {
    pub(super) fn navigation(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("defender_navigation").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("Atom Defender");
                ui.separator();
                for (view, label) in [
                    (View::Overview, "Overview"),
                    (View::Rowhammer, "Rowhammer"),
                    (View::File, "File Guard"),
                    (View::Keyboard, "Keyboard Guard"),
                    (View::Network, "Network Lightcone"),
                    (View::Alerts, "Alert history"),
                ] {
                    let text = egui::RichText::new(label).color(if self.view == view {
                        theme::BG
                    } else {
                        theme::TEXT
                    });
                    ui.selectable_value(&mut self.view, view, text);
                }
            });
        });
    }
    fn file_state(&self) -> &'static str {
        if self.guard.failed {
            "Needs attention"
        } else if let Some(status) = self.credential.summary() {
            status
        } else if self.guard.active() {
            "Automatic restore armed"
        } else if self.integrity_watch.is_some() {
            "Watching selected file"
        } else {
            "Off"
        }
    }
    fn history_panel(&self, ui: &mut egui::Ui, limit: usize) {
        if let Some(e) = &self.activity.error {
            ui.colored_label(
                theme::WARN,
                format!("Alert history storage unavailable · {e}"),
            );
        }
        if self.activity.rows.is_empty() {
            ui.label("No alerts recorded.");
        }
        for row in self.activity.rows.iter().take(limit) {
            ui.label(row);
            ui.separator();
        }
    }
    pub(super) fn show_module(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(20.0))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| match self.view {
                    View::Overview => {
                        ui.heading("Protection overview");
                        ui.add_space(12.0);
                        for (view, title, description, status) in [
                            (
                                View::Rowhammer,
                                "Rowhammer",
                                "Analyze an uploaded memory trace.",
                                if snap.halted {
                                    "Replay stopped"
                                } else if snap.enabled {
                                    "Analyzing trace"
                                } else {
                                    "Off"
                                },
                            ),
                            (
                                View::File,
                                "File Guard",
                                "Control credential access, watch integrity, or restore a file.",
                                self.file_state(),
                            ),
                            (
                                View::Keyboard,
                                "Keyboard Guard",
                                "Monitor new opens of physical keyboards.",
                                self.keyboard.status(),
                            ),
                            (
                                View::Network,
                                "Network Lightcone",
                                "Compare live or captured traffic to exact reference paths.",
                                self.network.status(),
                            ),
                        ] {
                            egui::Frame::new()
                                .fill(theme::CARD)
                                .inner_margin(14.0)
                                .corner_radius(6.0)
                                .show(ui, |ui| {
                                    ui.set_min_width((ui.available_width() - 2.0).max(0.0));
                                    ui.horizontal_wrapped(|ui| {
                                        if ui.link(egui::RichText::new(title).size(19.0)).clicked()
                                        {
                                            self.view = view;
                                        }
                                        ui.label(status);
                                    });
                                    ui.label(description);
                                });
                            ui.add_space(7.0);
                        }
                        ui.add_space(8.0);
                        ui.heading("Recent alerts");
                        self.history_panel(ui, 5);
                    }
                    View::File => {
                        ui.heading("File Guard");
                        ui.label(self.file_state());
                        self.file_controls(ui);
                    }
                    View::Keyboard => self.keyboard.show(ui, &mut self.activity),
                    View::Network => self.network.show(ui, &mut self.activity),
                    View::Alerts => {
                        ui.heading("Alert history");
                        ui.add_space(10.0);
                        self.history_panel(ui, 200);
                    }
                    View::Rowhammer => {}
                });
            });
    }
}
