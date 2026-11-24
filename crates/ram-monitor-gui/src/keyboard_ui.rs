use crate::activity::{now, Activity};
use eframe::egui;
use ram_monitor::keyboard::{Event, Observer, Opening};
use ram_monitor_bus::{BusEvent, GuardBus};
use std::collections::BTreeSet;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct KeyboardUi {
    runtime: Option<GuardBus<Event>>,
    ready: bool,
    stopping: bool,
    fault: Option<String>,
    expected: BTreeSet<(u32, u32, u64)>,
    last_unexpected: Option<Opening>,
    count: u64,
    losses: u64,
    last_notification: Option<Instant>,
    pending: Arc<AtomicUsize>,
}
impl KeyboardUi {
    pub fn status(&self) -> &'static str {
        if self.fault.is_some() {
            "Unavailable"
        } else if self.stopping {
            "Stopping"
        } else if self.ready && self.losses > 0 {
            "Monitoring · events lost"
        } else if self.ready {
            "Monitoring new opens"
        } else if self.runtime.is_some() {
            "Starting"
        } else {
            "Off"
        }
    }
    fn fail(&mut self, e: String, log: &mut Activity) {
        self.ready = false;
        if self.fault.as_ref() != Some(&e) {
            log.record(now(), "Keyboard Guard", &e);
        }
        self.fault = Some(e);
    }
    fn handle(&mut self, event: BusEvent<Event>, log: &mut Activity) {
        let data = match event {
            BusEvent::Crashed(e) => Err(e),
            BusEvent::Delivery(d) => d.outcome,
        };
        if data.as_ref().is_ok_and(|e| *e != Event::Idle) {
            self.pending.fetch_sub(1, Ordering::Relaxed);
        }
        match data {
            Err(e) => self.fail(e, log),
            Ok(Event::Ready) => self.ready = true,
            Ok(Event::Idle) => {}
            Ok(Event::Lost(n)) => {
                self.losses = self.losses.saturating_add(n);
                log.record(
                    now(),
                    "Keyboard Guard",
                    &format!("Capture incomplete · {n} kernel events lost"),
                );
            }
            Ok(Event::Open(o)) => {
                if !o.expected_system_reader() && !self.expected.contains(&o.identity()) {
                    let message = format!(
                        "{} (PID {}, UID {}) opened {} for reading",
                        o.executable, o.pid, o.uid, o.device
                    );
                    log.record(o.at, "Keyboard Guard", &message);
                    if self
                        .last_notification
                        .is_none_or(|last| last.elapsed() >= Duration::from_secs(5))
                    {
                        self.last_notification = Some(Instant::now());
                        std::thread::spawn(move || {
                            let _ = std::process::Command::new("/usr/bin/gdbus")
                                .args([
                                    "call",
                                    "--session",
                                    "--timeout",
                                    "2",
                                    "--dest",
                                    "org.freedesktop.Notifications",
                                    "--object-path",
                                    "/org/freedesktop/Notifications",
                                    "--method",
                                    "org.freedesktop.Notifications.Notify",
                                    "Atom Defender",
                                    "0",
                                    "atom-rowhammer-defender",
                                    "Unexpected keyboard access",
                                    &message,
                                    "[]",
                                    "{}",
                                    "8000",
                                ])
                                .output();
                        });
                    }
                    self.count += 1;
                    self.last_unexpected = Some(o);
                }
            }
        }
    }
    pub fn poll(&mut self, log: &mut Activity) {
        let Some(bus) = self.runtime.as_ref() else {
            return;
        };
        let finished = bus.is_finished();
        let events: Vec<_> = bus.events().try_iter().collect();
        for event in events {
            self.handle(event, log);
        }
        if finished {
            if let Some(mut bus) = self.runtime.take() {
                let result = bus.join();
                for event in bus.events().try_iter() {
                    self.handle(event, log);
                }
                if let Err(e) = result {
                    self.fail(e.to_string(), log);
                }
            }
            self.ready = false;
            self.stopping = false;
        }
    }
    fn start(&mut self, ctx: &egui::Context, log: &mut Activity) {
        self.fault = None;
        self.ready = false;
        self.stopping = false;
        self.losses = 0;
        self.pending = Arc::default();
        let pending = Arc::clone(&self.pending);
        let ctx = ctx.clone();
        let run = Observer::start().and_then(|mut observer| {
            GuardBus::start_named(
                "defender.keyboard",
                "defender.keyboard.open",
                move |delay| {
                    let event = observer.next(delay)?;
                    if event != Event::Idle && pending.fetch_add(1, Ordering::Relaxed) >= 512 {
                        return Err("Keyboard monitoring interrupted · dashboard event backlog exceeded 512".into());
                    }
                    Ok(event)
                },
                Duration::from_millis(200),
                None,
                move || ctx.request_repaint(),
            )
            .map_err(|e| e.to_string())
        });
        match run {
            Ok(bus) => self.runtime = Some(bus),
            Err(e) => self.fail(e, log),
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui, log: &mut Activity) {
        ui.heading("Keyboard Guard");
        ui.label("Alerts when an unexpected process opens a physical keyboard for reading. Coverage begins when monitoring starts.");
        ui.add_space(8.0);
        ui.strong(self.status());
        if let Some(e) = &self.fault {
            ui.colored_label(crate::theme::WARN, e);
        }
        if let Some(bus) = &self.runtime {
            if ui
                .add_enabled(!self.stopping, egui::Button::new("Stop monitoring"))
                .clicked()
            {
                self.stopping = true;
                bus.stop();
            }
        } else if ui.button("Start monitoring").clicked() {
            self.start(ui.ctx(), log);
        }
        ui.add_space(8.0);
        ui.label(format!("Unexpected opens this session · {}", self.count));
        if let Some(o) = &self.last_unexpected {
            ui.label(format!(
                "{} (PID {}, UID {}) · {}",
                o.executable, o.pid, o.uid, o.device
            ));
            if o.ino != 0
                && !self.expected.contains(&o.identity())
                && ui
                    .button("Expect this executable for this session")
                    .clicked()
            {
                self.expected.insert(o.identity());
                log.record(
                    now(),
                    "Keyboard Guard",
                    &format!(
                        "Marked {} expected for UID {} for this session",
                        o.executable, o.uid
                    ),
                );
            }
        }
        if !self.expected.is_empty() && ui.button("Clear session exceptions").clicked() {
            self.expected.clear();
        }
        ui.small("Records new direct opens while Atom Defender is running. Earlier opens and capture inside an application or Xwayland are outside this view. The guard never reads keystrokes.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ram_monitor_bus::GuardDelivery;
    #[test]
    fn unexpected_open_is_visible_and_session_exception_is_exact() {
        let mut k = KeyboardUi {
            last_notification: Some(Instant::now()),
            ..Default::default()
        };
        let mut log = Activity::default();
        let event = Opening {
            at: 123,
            pid: 19,
            uid: 1000,
            exe_dev: 1,
            ino: 88,
            device: "/dev/input/event9".into(),
            executable: "reader".into(),
        };
        fn feed(k: &mut KeyboardUi, a: &mut Activity, e: Event) {
            k.pending.fetch_add(1, Ordering::Relaxed);
            k.handle(
                BusEvent::Delivery(GuardDelivery {
                    check: 1,
                    outcome: Ok(e),
                }),
                a,
            );
        }
        feed(&mut k, &mut log, Event::Ready);
        feed(&mut k, &mut log, Event::Open(event.clone()));
        assert_eq!(k.count, 1);
        assert!(log.rows[0].contains("reader (PID 19, UID 1000) opened /dev/input/event9"));
        k.expected.insert(event.identity());
        feed(&mut k, &mut log, Event::Open(event.clone()));
        feed(&mut k, &mut log, Event::Open(Opening { ino: 89, ..event }));
        assert_eq!(k.count, 2);
        feed(&mut k, &mut log, Event::Lost(3));
        assert_eq!(k.status(), "Monitoring · events lost");
        assert!(log.rows[0].contains("3 kernel events lost"));
        k.handle(BusEvent::Crashed("observer exited".into()), &mut log);
        assert_eq!(k.status(), "Unavailable");
    }

    #[test]
    #[ignore = "run in the keyboard acceptance VM with an external open/close witness"]
    fn live_keyboard_open_reaches_dashboard_history() {
        let mut ui = KeyboardUi::default();
        let ctx = egui::Context::default();
        let mut alerts = Activity::open();
        assert!(alerts.error.is_none());
        ui.start(&ctx, &mut alerts);
        let deadline = Instant::now() + Duration::from_secs(12);
        let mut announced = false;
        while ui.last_unexpected.is_none() && Instant::now() < deadline {
            ui.poll(&mut alerts);
            assert!(ui.fault.is_none(), "{:?}", ui.fault);
            if ui.ready && !announced {
                println!("GUEST_UI_READY");
                announced = true;
            }
            std::thread::sleep(Duration::from_millis(15));
        }
        let opening = ui
            .last_unexpected
            .as_ref()
            .expect("external keyboard open in dashboard");
        assert_eq!(ui.count, 1);
        let persisted = Activity::open();
        assert!(persisted.rows[0].contains(&format!(
            "(PID {}, UID {}) opened {}",
            opening.pid, opening.uid, opening.device
        )));
        println!(
            "GUEST_UI_OPEN {} {} {} {}",
            opening.pid, opening.uid, opening.device, opening.executable
        );
        ui.runtime.as_ref().unwrap().stop();
        while ui.runtime.is_some() && Instant::now() < deadline {
            ui.poll(&mut alerts);
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(ui.status(), "Off");
    }
}
