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
fn run(fingerprints: impl Iterator<Item = u64>, threshold: usize) -> u64 {
    let mut monitor = RamMonitor::new(RowGeometry::default(), 64, threshold);
    let mut feed = feed();
    fingerprints
        .enumerate()
        .filter_map(|(index, fp)| monitor.observe_addr(&mut feed, (fp << 6) + index as u64 % 64))
        .count() as u64
}
#[test]
fn cycle8_ruptures() {
    assert!(run((0..512).map(|i| i % 8), 16) > 0);
}
#[test]
fn cycle16_ruptures() {
    assert!(run((0..512).map(|i| i % 16), 16) > 0);
}
#[test]
fn cycle32_ruptures() {
    assert!(run((0..1024).map(|i| i % 32), 16) > 0);
}
#[test]
fn cycle59_ruptures() {
    assert!(run((0..1024).map(|i| i % 59), 16) > 0);
}
#[test]
fn burst8_ruptures() {
    assert!(
        run(
            (0..512).map(|i| if i % 64 < 8 { 3 } else { 64 + i % 128 }),
            16
        ) > 0
    );
}
#[test]
fn uniform_and_sequential_stay_intact() {
    let uniform = (0..512).map(|i| mix(i) % 256);
    assert_eq!(run(uniform, 16), 0);
    assert_eq!(run((0..512).map(|i| i % 256), 16), 0);
}
#[test]
fn strict_operator_threshold_disables_secondary_observers() {
    assert_eq!(run((0..512).map(|i| i % 16), 64), 0);
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

#[test]
fn fast_timed_cycle512_ruptures() {
    assert!(run_timed((0..8192).map(|i| i % 512), 50).is_some());
}

#[test]
fn slow_timed_locality_stays_intact() {
    for period in [1, 2, 4, 8, 16, 32, 59, 64, 128, 256, 512] {
        assert_eq!(
            run_timed((0..1024).map(|i| i % period), 1_000_000),
            None,
            "period {period}"
        );
    }
}

#[test]
fn subcritical_fast_bursts_stay_intact() {
    assert_eq!(run_timed((0..128).map(|_| 3), 50), None);
    let mut monitor = RamMonitor::new(RowGeometry::default(), 64, 16);
    let mut feed = feed();
    for burst in 0..8_u64 {
        let start = burst * 5_000_000;
        for index in 0..128_u64 {
            assert!(monitor
                .observe_sample(
                    &mut feed,
                    AccessSample {
                        addr: ((burst * 4096) << 6) + index % 64,
                        time_ns: Some(start + index * 50),
                        address_kind: AddressKind::Synthetic,
                        ..AccessSample::default()
                    },
                )
                .is_none());
        }
    }
}

#[test]
fn sustained_fast_burst_ruptures_at_thermal_boundary() {
    assert_eq!(run_timed((0..128).map(|_| 3), 50), None);
    assert!(run_timed((0..130).map(|_| 3), 50).is_some());
}

#[test]
fn subcritical_thermal_plateau_stays_intact() {
    assert_eq!(run_timed((0..10_000).map(|_| 3), 4_000), None);
    assert_eq!(run_timed((0..10_000).map(|_| 3), 4_100), None);
}

#[test]
fn sketch_capacity_edge_ruptures_and_unique_control_stays_intact() {
    assert!(run_timed((0..131_300).map(|index| index % 131_072), 10).is_some());
    assert_eq!(run_timed(0..131_300, 10), None);
}
