//! Controller: runs the monitor on a background thread and exposes a shared
//! snapshot plus commands (apply settings / reset detector). This is the
//! headless-testable core the GUI (and any other front end) drives.
//!
//! Settings are only ever changed by an explicit operator command — never
//! inferred from observations. Applying settings rebuilds the simulation and
//! re-arms the stack (a fresh feed with four owned copies); Reset re-arms
//! with the current settings. All outputs remain numerical receipts only.

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use imperium_nocopy_swarm_live::{Order, SwarmLiveConfig, SwarmLiveFeed};

use crate::{AccessSample, AccessSource, RamMonitor, RowGeometry, TraceSource};

/// Operator-adjustable settings. Applied only via an explicit command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub window: usize,
    pub thresh: usize,
    pub row_shift: u32,
    pub row_bits: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            window: 64,
            thresh: 16,
            row_shift: 6,
            row_bits: 8,
        }
    }
}

impl Settings {
    fn geometry(&self) -> RowGeometry {
        RowGeometry {
            row_shift: self.row_shift,
            row_bits: self.row_bits,
        }
    }
}

/// Bounded trip log line (numerical receipt rendered as text).
pub const LOG_CAP: usize = 64;

/// Point-in-time view of the running monitor.
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub samples: u64,
    pub frames: u64,
    pub emitted: u64,
    pub trips: u64,
    pub armed: bool,
    pub halted: bool,
    pub source_name: String,
    pub evidence: String,
    pub last_receipt: Option<String>,
    pub log: Vec<String>,
}

/// Commands to the controller thread.
#[derive(Clone, Debug)]
pub enum Command {
    /// Rebuild counters + detector with these settings (also re-arms).
    Apply(Settings),
    /// Re-arm the detector with the current settings (clears the latched trip).
    Reset,
    /// Replace the current source with one uploaded portable simulation trace.
    LoadTrace {
        samples: Vec<AccessSample>,
        name: String,
    },
}

fn splitmix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

/// A freshly armed feed with four locally generated OWNED copies (standalone
/// has no mesh; the Imperium integration sources copies from the handshake
/// key-copy sink — live keys are never read).
fn armed_feed() -> SwarmLiveFeed {
    let mut feed = SwarmLiveFeed::new();
    let seeds: [u64; 4] = core::array::from_fn(|j| splitmix(j as u64 ^ 0x5EED));
    feed.set_arming_config(SwarmLiveConfig::s3([0, 1, 2, 3], seeds, Order::L));
    for label in 0..4u64 {
        let copy: Vec<u8> = (0..32)
            .map(|p| (splitmix((label * 32 + p as u64) ^ 0xC0FFEE) & 0xFF) as u8)
            .collect();
        let _ = feed.ingest_key_copy(label, copy);
    }
    feed
}

/// Handle to the running monitor.
pub struct Controller {
    shared: Arc<Mutex<Snapshot>>,
    cmd: Sender<Command>,
}

impl Controller {
    /// Spawn the controller thread. `factory` builds the sources for given
    /// settings. The portable GUI supplies an empty factory and loads traces.
    pub fn spawn<F>(factory: F) -> Self
    where
        F: Fn(&Settings) -> Vec<Box<dyn AccessSource>> + Send + 'static,
    {
        let (tx, rx) = channel::<Command>();
        let shared = Arc::new(Mutex::new(Snapshot::default()));
        let shared2 = Arc::clone(&shared);
        std::thread::spawn(move || {
            let mut settings = Settings::default();
            let mut sources = factory(&settings);
            let mut loaded_trace: Option<Vec<AccessSample>> = None;
            let mut source_name = if sources.is_empty() {
                "none".to_string()
            } else {
                "generated replay".to_string()
            };
            let mut monitor =
                RamMonitor::new(settings.geometry(), settings.window, settings.thresh);
            let mut feed = armed_feed();
            let mut log: Vec<String> = Vec::new();
            let mut last_trips = 0u64;
            let mut samples = 0u64;
            let mut halted = false;
            loop {
                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        Command::Apply(s) => {
                            settings = s;
                            sources = match loaded_trace.as_ref() {
                                Some(samples) => TraceSource::new(samples.clone())
                                    .map(|source| vec![Box::new(source) as Box<dyn AccessSource>])
                                    .unwrap_or_default(),
                                None => factory(&settings),
                            };
                            monitor = RamMonitor::new(
                                settings.geometry(),
                                settings.window,
                                settings.thresh,
                            );
                            feed = armed_feed();
                            last_trips = 0;
                            halted = false;
                            log.clear();
                        }
                        Command::Reset => {
                            monitor = RamMonitor::new(
                                settings.geometry(),
                                settings.window,
                                settings.thresh,
                            );
                            feed = armed_feed();
                            sources.clear();
                            source_name = "none (reset)".to_string();
                            last_trips = 0;
                            halted = false;
                            log.clear();
                        }
                        Command::LoadTrace {
                            samples: trace_samples,
                            name,
                        } => {
                            loaded_trace = Some(trace_samples.clone());
                            source_name = name;
                            sources = TraceSource::new(trace_samples)
                                .map(|source| vec![Box::new(source) as Box<dyn AccessSource>])
                                .unwrap_or_default();
                            monitor = RamMonitor::new(
                                settings.geometry(),
                                settings.window,
                                settings.thresh,
                            );
                            feed = armed_feed();
                            last_trips = 0;
                            halted = false;
                            samples = 0;
                            log.clear();
                        }
                    }
                }
                if !halted {
                    'sources: for source in sources.iter_mut() {
                        for _ in 0..512 {
                            let Some(sample) = source.next_sample() else {
                                break;
                            };
                            samples += 1;
                            if monitor.observe_sample(&mut feed, sample).is_some() {
                                halted = true;
                                break 'sources;
                            }
                        }
                    }
                }
                let stats = feed.stats();
                let trips = stats.trips + monitor.secondary_trips();
                if trips > last_trips {
                    if let Some(r) = feed
                        .receipts()
                        .last()
                        .copied()
                        .or_else(|| monitor.last_secondary_receipt())
                    {
                        let line = format!(
                            "trip_reads={} warm={} ruptured={} heats={:?}",
                            r.trip_reads, r.warm, r.ruptured, r.heats
                        );
                        log.push(line.clone());
                        if log.len() > LOG_CAP {
                            log.remove(0);
                        }
                    }
                    last_trips = trips;
                }
                let snap = Snapshot {
                    samples,
                    frames: feed.frames() + monitor.secondary_frames(),
                    emitted: feed.frames() + monitor.secondary_frames(),
                    trips,
                    armed: feed.is_armed() && monitor.secondary_armed(),
                    halted,
                    source_name: source_name.clone(),
                    evidence: monitor.evidence.summary(),
                    last_receipt: log.last().cloned(),
                    log: log.clone(),
                };
                if let Ok(mut guard) = shared2.lock() {
                    *guard = snap;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });
        Self { shared, cmd: tx }
    }

    /// Point-in-time snapshot of the monitor.
    pub fn snapshot(&self) -> Snapshot {
        self.shared.lock().map(|g| g.clone()).unwrap_or_default()
    }

    /// Apply new settings (rebuilds counters and re-arms the detector).
    pub fn apply(&self, settings: Settings) {
        let _ = self.cmd.send(Command::Apply(settings));
    }

    /// Re-arm the detector with the current settings.
    pub fn reset(&self) {
        let _ = self.cmd.send(Command::Reset);
    }

    pub fn load_trace(&self, samples: Vec<AccessSample>, name: String) {
        let _ = self.cmd.send(Command::LoadTrace { samples, name });
    }
}

/// A looping replay source for tests/benchmarks (cycles forever).
pub struct LoopSource {
    addrs: Vec<u64>,
    pos: usize,
}

impl LoopSource {
    #[must_use]
    pub fn new(addrs: Vec<u64>) -> Self {
        Self { addrs, pos: 0 }
    }
}

impl AccessSource for LoopSource {
    fn next_sample(&mut self) -> Option<AccessSample> {
        if self.addrs.is_empty() {
            return None;
        }
        let addr = self.addrs[self.pos % self.addrs.len()];
        self.pos += 1;
        Some(AccessSample {
            addr,
            address_kind: crate::AddressKind::Synthetic,
            ..AccessSample::default()
        })
    }
}
