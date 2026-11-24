use atom_row_trace_engine::{Engine, TimeReversal};
use std::collections::HashMap;

#[test]
fn lifetime_endpoints_and_equal_timestamps() {
    for (gap, timing, heat) in [
        (0, true, 1.0),
        (50_000, true, 1.0),
        (50_001, false, 1.0),
        (4_000_000, false, 1.0),
        (4_000_001, false, 0.0),
    ] {
        let mut engine = Engine::default();
        assert!(!engine.observe(u64::MAX - gap, 1).unwrap().timing_repeat);
        assert_eq!(engine.observe(u64::MAX, 1).unwrap().timing_repeat, timing);
        assert_eq!(engine.reading().heat, heat);
    }
}

#[test]
fn backward_event_leaves_both_lanes_intact() {
    let mut engine = Engine::default();
    engine.observe(80, 7).unwrap();
    let before = engine.reading();
    assert_eq!(engine.observe(79, 9), Err(TimeReversal));
    assert_eq!(engine.reading(), before);
    assert!(!engine.observe(80, 9).unwrap().timing_repeat);
    assert!(engine.observe(80, 7).unwrap().timing_repeat);
    assert_eq!(engine.reading().heat, 1.0);
}

#[test]
fn capacity_boundary_and_simultaneous_duplicates() {
    let mut engine = Engine::default();
    for site in 0..65_536 {
        assert!(!engine.observe(0, site).unwrap().timing_repeat);
    }
    assert!(engine.observe(0, 0).unwrap().timing_repeat);
    assert!(engine.observe(0, 0).unwrap().timing_repeat);
    assert!(!engine.observe(0, 1).unwrap().timing_repeat);
    assert!(engine.observe(0, 0).unwrap().timing_repeat);
    assert_eq!(engine.reading().heat, 3.0);
}

#[test]
fn fresh_traces_match_independent_impulse_superposition_at_every_sample() {
    for seed in [73_u64, 89, 101, 137] {
        let mut rng = seed;
        let mut clock = 0_u64;
        let mut engine = Engine::default();
        let mut times = HashMap::new();
        let mut impulses = Vec::new();
        let mut peak = 0.0_f64;
        let mut first_heat = None;
        let mut first_repeat = None;
        let mut n_repeats = 0;
        for n in 1..=768 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            clock += match n {
                400 | 720 => 4_000_001,
                401..=719 => (rng >> 32) % 80_000,
                _ => (rng >> 32) % 101,
            };
            let site = ((rng >> 16) % 7) << 45;
            let dt = times.insert(site, clock).map(|last| clock - last);
            let repeat = dt.is_some_and(|d| d <= 50_000);
            if repeat {
                n_repeats += 1;
                first_repeat.get_or_insert(n);
            }
            if dt.is_some_and(|d| d <= 4_000_000) {
                impulses.push(clock);
            }
            let heat: f64 = impulses
                .iter()
                .map(|&t| (-((clock - t) as f64) / 500_000.0).exp())
                .sum();
            peak = peak.max(heat);
            if heat >= 128.0 {
                first_heat.get_or_insert(n);
            }
            assert_eq!(engine.observe(clock, site).unwrap().timing_repeat, repeat);
            let reading = engine.reading();
            assert_eq!(
                (
                    reading.samples,
                    reading.timing_matches,
                    reading.first_match,
                    reading.first_heat_crossing
                ),
                (n, n_repeats, first_repeat, first_heat)
            );
            assert!((reading.heat - heat).abs() < 1e-9 && (reading.peak_heat - peak).abs() < 1e-9);
        }
    }
}
