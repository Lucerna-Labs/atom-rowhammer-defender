//! Replay gates R1-R4 + control C1 of RAM-MONITOR-PRECOMMIT-2026-10-31.
//! Predictions from scripts/ram_predict.py (run before code):
//!   hammer-0..3: trip at samples=17, swarm_reads=2
//!   hammer-4..7: trip at samples 21-22, swarm_reads 6-7
//!   scatter/benign (256 samples): emitted=0, no trip
//!   mixed: trip at samples=18, swarm_reads=2

use imperium_nocopy_swarm_live::{Order, SwarmLiveConfig, SwarmLiveFeed};
use ram_monitor::{AccessSource, RamMonitor, ReplaySource, RowGeometry};

fn splitmix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

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
    assert!(feed.is_armed());
    feed
}

/// Run sample-by-sample until the first receipt; returns (samples consumed,
/// receipt). `None` if no trip within the source.
fn run_until_trip(
    monitor: &mut RamMonitor,
    feed: &mut SwarmLiveFeed,
    source: &mut impl AccessSource,
) -> Option<(u64, u64)> {
    let mut samples = 0u64;
    while let Some(s) = source.next_sample() {
        samples += 1;
        if let Some(receipt) = monitor.observe_addr(feed, s.addr) {
            return Some((samples, receipt.trip_reads));
        }
    }
    None
}

fn monitor() -> RamMonitor {
    RamMonitor::new(RowGeometry::default(), 64, 16)
}

#[test]
fn r1_hammer_home_classes_trip_within_two_swarm_reads() {
    for class in 0..=3u8 {
        let mut feed = armed_feed();
        let mut monitor = monitor();
        let mut source = ReplaySource::hammer(class, 256, 6);
        let (samples, trip_reads) = run_until_trip(&mut monitor, &mut feed, &mut source)
            .unwrap_or_else(|| panic!("hammer-{class} must trip"));
        assert!(trip_reads <= 2, "R1: class {class} trip_reads {trip_reads} <= 2");
        assert!(samples <= 20, "R1: class {class} samples {samples} <= 20");
    }
}

#[test]
fn r2_hammer_nonhome_classes_trip_within_seven_swarm_reads() {
    for class in 4..=7u8 {
        let mut feed = armed_feed();
        let mut monitor = monitor();
        let mut source = ReplaySource::hammer(class, 256, 6);
        let (samples, trip_reads) = run_until_trip(&mut monitor, &mut feed, &mut source)
            .unwrap_or_else(|| panic!("hammer-{class} must trip"));
        assert!(trip_reads <= 7, "R2: class {class} trip_reads {trip_reads} <= 7");
        assert!(samples <= 24, "R2: class {class} samples {samples} <= 24");
    }
}

#[test]
fn r3_scatter_and_benign_emit_nothing_and_never_trip() {
    for (name, source) in [
        ("scatter", ReplaySource::scatter(256, 6)),
        ("benign", ReplaySource::benign(256, 6)),
    ] {
        let mut feed = armed_feed();
        let mut monitor = monitor();
        let mut source = source;
        let stats = monitor.run(&mut feed, &mut source, 256);
        assert_eq!(stats.samples, 256, "{name}: all samples consumed");
        assert_eq!(stats.emitted, 0, "R3: {name} must emit 0 concentrated observations");
        assert_eq!(feed.stats().trips, 0, "R3: {name} must never trip");
    }
}

#[test]
fn r4_mixed_hammer_plus_scatter_still_trips_fast() {
    let mut feed = armed_feed();
    let mut monitor = monitor();
    let mut source = ReplaySource::mixed(256, 6);
    let (samples, trip_reads) = run_until_trip(&mut monitor, &mut feed, &mut source)
        .expect("R4: mixed must trip");
    assert!(samples <= 20, "R4: samples {samples} <= 20");
    assert!(trip_reads <= 2, "R4: trip_reads {trip_reads} <= 2");
}

#[test]
fn c1_unarmed_feed_never_trips_on_hammer() {
    // Control: copies are ingested but there is no arming config, so the feed
    // never arms and the same hammer stays silent.
    let mut feed = SwarmLiveFeed::new();
    for label in 0..4u64 {
        let copy: Vec<u8> = (0..32)
            .map(|p| (splitmix((label * 32 + p as u64) ^ 0xC0FFEE) & 0xFF) as u8)
            .collect();
        let _ = feed.ingest_key_copy(label, copy);
    }
    assert!(!feed.is_armed(), "no arming config -> never armed");
    let mut monitor = monitor();
    let mut source = ReplaySource::hammer(0, 256, 6);
    let stats = monitor.run(&mut feed, &mut source, 256);
    assert_eq!(stats.emitted, 241, "concentration gate still emits on hammer");
    assert_eq!(feed.stats().trips, 0, "C1: unarmed feed never trips");
}
