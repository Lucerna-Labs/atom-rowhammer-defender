//! Controller gates (headless, replay sources): G1 hammer trips and logs a
//! receipt; G2 applying a stricter threshold suppresses a mixed (90% hammer)
//! trace that trips at the default threshold — the false-flag adjustment path;
//! G3 reset re-arms and clears the log.

use std::time::{Duration, Instant};

use ram_monitor::{Controller, LoopSource, Settings};

fn hammer_factory() -> impl Fn(&Settings) -> Vec<Box<dyn ram_monitor::AccessSource>> + Send + 'static
{
    |_| {
        // One region, class 0: every sample the same fingerprint.
        let addrs: Vec<u64> = (0..256u64).map(|k| k % 64).collect();
        vec![Box::new(LoopSource::new(addrs)) as Box<dyn ram_monitor::AccessSource>]
    }
}

fn mixed_factory() -> impl Fn(&Settings) -> Vec<Box<dyn ram_monitor::AccessSource>> + Send + 'static
{
    |_| {
        // 90% hammer class 0 + 10% scatter over classes 0..7 (splitmix choice).
        let mut addrs = Vec::new();
        for k in 0..4096u64 {
            let mut v = k.wrapping_add(0x9e37_79b9_7f4a_7c15);
            v = (v ^ (v >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            v = (v ^ (v >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            let r = v ^ (v >> 31);
            if r % 10 == 0 {
                addrs.push((k % 8) << 6);
            } else {
                addrs.push(k % 64);
            }
        }
        vec![Box::new(LoopSource::new(addrs)) as Box<dyn ram_monitor::AccessSource>]
    }
}

fn wait_for<F: Fn(&ram_monitor::Snapshot) -> bool>(
    c: &Controller,
    pred: F,
    secs: u64,
) -> ram_monitor::Snapshot {
    let deadline = Instant::now() + Duration::from_secs(secs);
    loop {
        let s = c.snapshot();
        if pred(&s) || Instant::now() > deadline {
            return s;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn g1_hammer_trips_and_logs_receipt() {
    let c = Controller::spawn(hammer_factory());
    let s = wait_for(&c, |s| s.trips >= 1, 10);
    assert!(s.armed, "feed armed");
    assert!(s.trips >= 1, "G1: hammer must trip");
    assert!(s.last_receipt.is_some(), "G1: receipt logged");
    assert!(!s.log.is_empty());
    let r = s.last_receipt.unwrap();
    assert!(r.contains("trip_reads="), "receipt is numerical: {r}");
}

#[test]
fn g2_stricter_threshold_suppresses_mixed_trace() {
    let c = Controller::spawn(mixed_factory());
    // Default (window 64, thresh 16): mixed 90% hammer trips.
    let s = wait_for(&c, |s| s.trips >= 1, 10);
    assert!(s.trips >= 1, "G2: mixed trips at default thresh");
    // Apply the strictest threshold: top fingerprint must hold all 64 of 64
    // samples; the 10% scatter guarantees it never does -> no trip after
    // re-arm. Apply clears the log, so wait for that as proof it landed.
    let stricter = Settings {
        thresh: 64,
        ..Settings::default()
    };
    c.apply(stricter);
    let off = wait_for(&c, |s| !s.enabled && s.log.is_empty() && s.trips == 0, 10);
    assert!(!off.enabled, "G2: Apply returns the simulation to OFF");
    c.set_enabled(true);
    let _ = wait_for(&c, |s| s.enabled && s.samples > 0, 10);
    std::thread::sleep(Duration::from_millis(500));
    let s = c.snapshot();
    assert_eq!(s.trips, 0, "G2: stricter thresh suppresses the mixed trace");
    assert!(
        s.samples > 0,
        "monitor consumes samples only after explicit ON"
    );
}

/// Hammer for a while, then go quiet (scatter): trips once, never again.
struct HammerThenQuiet {
    head: Vec<u64>,
    tail: Vec<u64>,
    pos: usize,
}

impl ram_monitor::AccessSource for HammerThenQuiet {
    fn next_sample(&mut self) -> Option<ram_monitor::AccessSample> {
        let addr = if self.pos < self.head.len() {
            self.head[self.pos]
        } else {
            self.tail[(self.pos - self.head.len()) % self.tail.len()]
        };
        self.pos += 1;
        Some(ram_monitor::AccessSample {
            addr,
            address_kind: ram_monitor::AddressKind::Synthetic,
            ..ram_monitor::AccessSample::default()
        })
    }
}

#[test]
fn g3_reset_rearms_and_clears_log() {
    // Short head: the controller drains up to 512 samples per tick, so after
    // the trip tick the source is already in the quiet tail.
    let head: Vec<u64> = (0..100u64).map(|k| k % 64).collect(); // hammer, trips fast
    let tail: Vec<u64> = (0..4096u64).map(|k| (k % 256) << 6).collect(); // sequential, never trips
    let c = Controller::spawn(move |_| {
        vec![Box::new(HammerThenQuiet {
            head: head.clone(),
            tail: tail.clone(),
            pos: 0,
        }) as Box<dyn ram_monitor::AccessSource>]
    });
    let s = wait_for(&c, |s| s.trips >= 1 && !s.log.is_empty(), 10);
    assert!(s.trips >= 1, "G3: hammer head trips");
    c.reset();
    // After reset the source is already in the quiet tail, so the trip must
    // stay cleared.
    let s = wait_for(&c, |s| s.armed && s.trips == 0 && s.log.is_empty(), 10);
    assert!(s.armed, "G3: re-armed after reset");
    assert_eq!(s.trips, 0, "G3: trip cleared by reset and stays cleared");
    assert!(s.log.is_empty(), "G3: log cleared by reset");
}

#[test]
fn g4_uploaded_trace_is_the_only_source_and_trips() {
    let controller = Controller::spawn(|_| Vec::new());
    let before = controller.snapshot();
    assert_eq!(before.samples, 0);
    let samples = (0..130_u64)
        .map(|index| ram_monitor::AccessSample {
            addr: (3 << 6) + index % 64,
            time_ns: Some(index * 50),
            address_kind: ram_monitor::AddressKind::Synthetic,
            ..ram_monitor::AccessSample::default()
        })
        .collect();
    controller.load_trace(samples, "uploaded.trace".to_string());
    let staged = wait_for(
        &controller,
        |snapshot| snapshot.source_name == "uploaded.trace" && !snapshot.enabled,
        10,
    );
    assert_eq!(staged.samples, 0, "upload must stay staged while OFF");
    controller.set_enabled(true);
    let after = wait_for(
        &controller,
        |snapshot| snapshot.source_name == "uploaded.trace" && snapshot.trips >= 1,
        10,
    );
    assert_eq!(after.samples, 130);
    assert!(after.trips >= 1);
    assert!(after.halted);
    assert!(after.samples <= 130);
    assert!(after.last_receipt.is_some());
}

#[test]
fn g5_off_stops_consumption_and_on_restarts_from_the_beginning() {
    let controller = Controller::spawn(|_| Vec::new());
    let samples = (0..100_000_u64)
        .map(|index| ram_monitor::AccessSample {
            addr: index << 6,
            time_ns: Some(index * 50),
            address_kind: ram_monitor::AddressKind::Synthetic,
            ..ram_monitor::AccessSample::default()
        })
        .collect();
    controller.load_trace(samples, "long-safe.trace".to_string());
    let staged = wait_for(
        &controller,
        |snapshot| snapshot.source_name == "long-safe.trace",
        10,
    );
    assert!(!staged.enabled);
    assert_eq!(staged.samples, 0);

    controller.set_enabled(true);
    let running = wait_for(
        &controller,
        |snapshot| snapshot.enabled && snapshot.samples > 0,
        10,
    );
    assert!(running.samples > 0);
    controller.set_enabled(false);
    let stopped = wait_for(&controller, |snapshot| !snapshot.enabled, 10);
    let stopped_samples = stopped.samples;
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(controller.snapshot().samples, stopped_samples);

    controller.set_enabled(true);
    let restarted = wait_for(
        &controller,
        |snapshot| snapshot.enabled && snapshot.samples > 0,
        10,
    );
    assert!(restarted.samples <= 512, "ON restarts from the beginning");
}

#[test]
fn g6_on_stays_on_without_a_trace_and_load_while_on_starts() {
    let controller = Controller::spawn(|_| Vec::new());
    controller.set_enabled(true);
    let waiting = wait_for(&controller, |snapshot| snapshot.enabled, 10);
    assert!(waiting.enabled);
    assert_eq!(waiting.samples, 0);

    let samples = (0..130_u64)
        .map(|index| ram_monitor::AccessSample {
            addr: (3 << 6) + index % 64,
            time_ns: Some(index * 50),
            address_kind: ram_monitor::AddressKind::Synthetic,
            ..ram_monitor::AccessSample::default()
        })
        .collect();
    controller.load_trace(samples, "load-while-on.trace".to_string());
    let running = wait_for(
        &controller,
        |snapshot| {
            snapshot.enabled
                && snapshot.source_name == "load-while-on.trace"
                && snapshot.samples > 0
        },
        10,
    );
    assert!(running.enabled);
    assert!(running.samples > 0);
}
