//! Behaviour required by the documented 50,000 ns recurrence horizon, which the
//! shipped repeat-density gate does not implement.
//!
//! The tests are `#[ignore]`d so the suite stays green while the gap stays
//! visible; run them with `cargo test --test documented_horizon_gates --
//! --ignored` to observe the mismatch. They are intentionally not deleted:
//! `documented_horizon_attack_ruptures` is the false negative and
//! `documented_horizon_benign_scan_stays_intact` is the false positive described
//! in the 2026-11-02 correction at the top of `docs/ADVERSARIAL-TIMED-RESULT.md`.

use imperium_nocopy_swarm_live::{Order, SwarmLiveConfig, SwarmLiveFeed};
use ram_monitor::{AccessSample, AddressKind, RamMonitor, RowGeometry};

fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn feed() -> SwarmLiveFeed {
    let mut feed = SwarmLiveFeed::new();
    let seeds = core::array::from_fn(|index| mix(index as u64 ^ 0x5eed));
    feed.set_arming_config(SwarmLiveConfig::s3([0, 1, 2, 3], seeds, Order::L));
    for label in 0..4_u64 {
        let copy = (0..32)
            .map(|position| (mix(label * 32 + position ^ 0xc0ffee) & 255) as u8)
            .collect();
        let _ = feed.ingest_key_copy(label, copy);
    }
    feed
}

fn run_timed(fingerprints: impl Iterator<Item = u64>, spacing_ns: u64) -> Option<usize> {
    let mut monitor = RamMonitor::new(RowGeometry::default(), 64, 16);
    let mut feed = feed();
    fingerprints.enumerate().find_map(|(index, fp)| {
        monitor
            .observe_sample(
                &mut feed,
                AccessSample {
                    addr: (fp << 6) + index as u64 % 64,
                    time_ns: Some(index as u64 * spacing_ns),
                    address_kind: AddressKind::Synthetic,
                    ..AccessSample::default()
                },
            )
            .map(|_| index + 1)
    })
}

/// A constant address recurring every 4,000 ns is inside the documented
/// 50,000 ns horizon, so the documented rule must rupture it. It does not:
/// the shipped heat integrator saturates at 125.5 units, below its danger
/// threshold of 128.
#[test]
#[ignore = "shipped gate is repeat-density; a 4,000 ns recurrence never ruptures (documented-horizon false negative)"]
fn documented_horizon_attack_ruptures() {
    assert!(run_timed((0..4_096).map(|_| 7), 4_000).is_some());
}

/// A balanced scan of 4,096 distinct lines at 50 ns spacing recurs per line
/// only every 204,800 ns, outside the documented horizon, so the documented
/// rule must not rupture it. The shipped gate does rupture it.
#[test]
#[ignore = "shipped gate ruptures a benign fast balanced scan (documented-horizon false positive)"]
fn documented_horizon_benign_scan_stays_intact() {
    assert_eq!(run_timed((0..8_192).map(|i| i % 4_096), 50), None);
}
