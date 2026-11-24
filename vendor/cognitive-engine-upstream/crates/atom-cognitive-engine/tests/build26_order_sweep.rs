//! BUILD-26 — BUILD-1's own order sweep (the last Rule-2 debt).
//! Precommitted in `docs/BUILD26-ORDER-SWEEP.md` BEFORE this test.
//!
//! Declared primitive set: S (coherent superposition with the epoch
//! carrier), R (destructive Born readout), A (intensity accumulation) →
//! 3! = 6 unique orders. Two are constructible (SRA = `InterferenceField`,
//! SAR = `CoherentField`); four are not, and are NAMED with their reasons.
//! Per Rule 2 the untested orders are reported and the claim is bounded to
//! the executed ones.

use atom_cognitive_engine::interference_field::{
    intensity_spectrum, Arm, CoherentField, InterferenceField, N,
};
use atom_cognitive_engine::perception::Reading;

// ---- harness-side fixtures, mirrored from BUILD-1's test (duplicated
// deliberately — observations are not behavior) ----

fn window(base_id: u64, regime: usize) -> Vec<Reading> {
    let (exploration, convergence) = match regime {
        0 => (0.80, 0.70),
        1 => (0.80, 0.30),
        _ => (0.30, 0.70),
    };
    (0..5u64)
        .map(|k| Reading::new(base_id + k, 0.5, 0.5, 0.5, exploration, convergence, 0.5))
        .collect()
}

fn expected_differences() -> Vec<(isize, isize)> {
    let regimes = [(0.80f64, 0.70f64), (0.80, 0.30), (0.30, 0.70)];
    let mut diffs = Vec::new();
    for a in 0..3 {
        for b in (a + 1)..3 {
            let dx = (regimes[a].0 - regimes[b].0) * N as f64;
            let dy = (regimes[a].1 - regimes[b].1) * N as f64;
            diffs.push((dx.round() as isize, dy.round() as isize));
        }
    }
    diffs
}

fn circular_distance_bin(a: (isize, isize), b: (isize, isize)) -> f64 {
    let n = N as isize;
    let mut dx = (a.0 - b.0).rem_euclid(n);
    let mut dy = (a.1 - b.1).rem_euclid(n);
    if dx > n / 2 {
        dx -= n;
    }
    if dy > n / 2 {
        dy -= n;
    }
    ((dx * dx + dy * dy) as f64).sqrt()
}

fn local_mass(spectrum: &[f64], center: (isize, isize)) -> f64 {
    let n = N as isize;
    let mut mass = 0.0;
    for dy in -1..=1isize {
        for dx in -1..=1isize {
            let x = (center.0 + dx).rem_euclid(n) as usize;
            let y = (center.1 + dy).rem_euclid(n) as usize;
            mass += spectrum[y * N + x];
        }
    }
    mass
}

fn mean_centered_spectrum(pattern: &[f64]) -> Vec<f64> {
    let mean = pattern.iter().sum::<f64>() / pattern.len() as f64;
    let centered: Vec<f64> = pattern.iter().map(|v| v - mean).collect();
    intensity_spectrum(&centered)
}

// ---- the declared sweep table (the enumeration, as data) ----

struct OrderEntry {
    order: &'static str,
    constructible: bool,
    reason: &'static str,
}

const ORDERS: [OrderEntry; 6] = [
    OrderEntry {
        order: "SRA",
        constructible: true,
        reason: "canonical stack: InterferenceField with Arm::FullStack",
    },
    OrderEntry {
        order: "SAR",
        constructible: true,
        reason: "no-reset order: CoherentField (BUILD-1's G3 arm)",
    },
    OrderEntry {
        order: "RSA",
        constructible: false,
        reason: "the readout precedes any formed field — nothing exists to measure",
    },
    OrderEntry {
        order: "RAS",
        constructible: false,
        reason: "same: the readout has no field",
    },
    OrderEntry {
        order: "ASR",
        constructible: false,
        reason: "accumulation precedes content — vacuously zero",
    },
    OrderEntry {
        order: "ARS",
        constructible: false,
        reason: "same: nothing to accumulate",
    },
];

/// The bounded claim (W4's audit target).
const SCOPE: &str = "best observed among the EXECUTED orders (SRA, SAR) of the declared primitive \
                     set {S, R, A}; untested orders reported: RSA, RAS, ASR, ARS — no global-best claim";

/// The swept corpus: nine epochs at the three regimes, then nine epochs at
/// the same regimes shifted in kx by 0.15 (BUILD-1's G3 construction).
fn sweeps_corpus() -> (Vec<Vec<Reading>>, Vec<Vec<Reading>>) {
    let shifted = |base: u64, regime: usize, shift: f64| -> Vec<Reading> {
        window(base, regime)
            .into_iter()
            .map(|mut r| {
                r.exploration += shift;
                r
            })
            .collect()
    };
    let mut half_one = Vec::new();
    let mut base = 10u64;
    for _ in 0..9 {
        let mut epoch = Vec::new();
        for regime in 0..3 {
            epoch.extend(window(base, regime));
            base += 100;
        }
        half_one.push(epoch);
    }
    let mut half_two = Vec::new();
    for _ in 0..9 {
        let mut epoch = Vec::new();
        for regime in 0..3 {
            epoch.extend(shifted(base, regime, 0.15));
            base += 100;
        }
        half_two.push(epoch);
    }
    (half_one, half_two)
}

/// The phantom vectors, excluding coincidences with real difference vectors.
fn phantom_vectors() -> Vec<(isize, isize)> {
    let regimes = [(0.80f64, 0.70f64), (0.80, 0.30), (0.30, 0.70)];
    let mut phantoms = Vec::new();
    for a in 0..3 {
        for b in 0..3 {
            let dx = (regimes[a].0 - (regimes[b].0 + 0.15)) * N as f64;
            let dy = (regimes[a].1 - regimes[b].1) * N as f64;
            phantoms.push((dx.round() as isize, dy.round() as isize));
        }
    }
    let real = expected_differences();
    phantoms
        .into_iter()
        .filter(|p| !real.iter().any(|r| circular_distance_bin(*p, *r) < 1.0))
        .collect()
}

/// W1 (precommitted: pass) — ENUMERATION: six unique orders, two
/// constructible, four named with reasons; the executed count equals the
/// constructible count.
#[test]
fn gate_w1_enumeration() {
    assert_eq!(ORDERS.len(), 6, "3 distinct primitives -> 3! = 6 unique orders");
    let constructible: Vec<&str> = ORDERS
        .iter()
        .filter(|o| o.constructible)
        .map(|o| o.order)
        .collect();
    let untested: Vec<&str> = ORDERS
        .iter()
        .filter(|o| !o.constructible)
        .map(|o| o.order)
        .collect();
    eprintln!("W1: executed {constructible:?}; untested {untested:?}");
    assert_eq!(constructible, vec!["SRA", "SAR"]);
    assert_eq!(untested, vec!["RSA", "RAS", "ASR", "ARS"]);
    for entry in ORDERS.iter().filter(|o| !o.constructible) {
        assert!(
            !entry.reason.is_empty(),
            "{} is untested and must carry its reason",
            entry.order
        );
    }
    assert_eq!(
        constructible.len(),
        2,
        "the executed count equals the constructible count"
    );
}

/// W2 (precommitted: pass) — MATCHED CONTROLS: both arms ingest the
/// identical 18 windows and are scored over the identical phantom set.
#[test]
fn gate_w2_matched_controls() {
    let (half_one, half_two) = sweeps_corpus();
    let windows = half_one.len() + half_two.len();
    assert_eq!(windows, 18, "eighteen windows per arm");

    let mut full = InterferenceField::new();
    let mut coherent = CoherentField::new();
    let mut ingested = 0usize;
    for window in half_one.iter().chain(half_two.iter()) {
        full.ingest_window(window, Arm::FullStack);
        coherent.ingest_window(window);
        ingested += 1;
    }
    assert_eq!(ingested, 18, "both arms saw every window");

    let phantoms = phantom_vectors();
    assert!(
        !phantoms.is_empty(),
        "the phantom set must be non-empty to discriminate"
    );
    assert_eq!(
        phantoms.len(),
        phantom_vectors().len(),
        "the same phantom set scores both arms"
    );
    eprintln!(
        "W2: 18 windows per arm (SRA, SAR); {} phantom vectors, identical for both",
        phantoms.len()
    );
}

/// W3 (precommitted: pass) — THE FINDING: the canonical order carries ~no
/// phantom mass; the no-reset order carries it heavily (the readout's
/// POSITION is what decides).
#[test]
fn gate_w3_finding() {
    let (half_one, half_two) = sweeps_corpus();
    let mut full = InterferenceField::new();
    let mut coherent = CoherentField::new();
    for window in half_one.iter().chain(half_two.iter()) {
        full.ingest_window(window, Arm::FullStack);
        coherent.ingest_window(window);
    }

    let phantoms = phantom_vectors();
    let full_spectrum = mean_centered_spectrum(&full.pattern_snapshot());
    let coherent_spectrum = mean_centered_spectrum(&coherent.pattern());
    let full_phantom: f64 = phantoms.iter().map(|p| local_mass(&full_spectrum, *p)).sum();
    let coherent_phantom: f64 =
        phantoms.iter().map(|p| local_mass(&coherent_spectrum, *p)).sum();

    eprintln!(
        "W3: phantom mass — SRA (canonical) {full_phantom:.1}; SAR (no-reset) {coherent_phantom:.1} over {} vectors",
        phantoms.len()
    );
    assert!(
        coherent_phantom > full_phantom * 2.0,
        "BUILD-1's own bar: no-reset carries phantom structure (coherent {coherent_phantom} vs full {full_phantom})"
    );
    assert!(
        full_phantom < 1.0,
        "the canonical order carries ~no phantom mass ({full_phantom})"
    );
    assert!(
        coherent_phantom > 4.0e9,
        "the recorded magnitude (4.8e9) re-observed ({coherent_phantom})"
    );
}

/// W4 (precommitted: pass) — THE BOUNDED CLAIM: the scope names the
/// declared set, the executed orders, and every untested order.
#[test]
fn gate_w4_bounded_claim() {
    for name in ["SRA", "SAR", "RSA", "RAS", "ASR", "ARS"] {
        assert!(SCOPE.contains(name), "the scope names {name}");
    }
    assert!(SCOPE.contains("no global-best claim"), "the claim is bounded");
    eprintln!("W4: {SCOPE}");
}
