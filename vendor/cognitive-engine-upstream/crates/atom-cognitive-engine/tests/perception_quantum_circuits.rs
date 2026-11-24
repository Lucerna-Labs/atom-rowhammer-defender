//! E1R-2 gates — the gate-circuit increment (SWAP-test Born-sampled
//! matching) and the E1 class-level ablations re-run on the EMERGENT
//! mechanism. Precommitted in `docs/E1R-PERCEPTION-QUANTUM.md` §E1R-2
//! BEFORE this code, per Jesse's "Do it". Labels remain harness-side.

use atom_cognitive_engine::perception::Reading;
use atom_cognitive_engine::perception_quantum::{EmergentConfig, EmergentPerceiver};
use atom_cognitive_engine::weave::{Register, Rng};

fn reading(id: u64, exploration: f64, convergence: f64) -> Reading {
    Reading::new(id, 0.5, 0.5, 0.5, exploration, convergence, 0.5)
}

fn window(base_id: u64, class: usize) -> Vec<Reading> {
    let (exploration, convergence) = match class {
        0 => (0.80, 0.70),
        1 => (0.80, 0.30),
        _ => (0.30, 0.70),
    };
    (0..5u64)
        .map(|k| {
            reading(
                base_id + k,
                exploration + 0.01 * k as f64,
                convergence - 0.01 * k as f64,
            )
        })
        .collect()
}

/// A formed three-prototype bank over the E1 corpus.
fn formed() -> EmergentPerceiver {
    let mut perceiver = EmergentPerceiver::new(EmergentConfig::default());
    for class in 0..3 {
        for reading in window(10 + class as u64 * 300, class) {
            perceiver.ingest(&reading);
        }
    }
    assert_eq!(perceiver.bank_size(), 3, "three regimes formed");
    perceiver
}

/// Q1 (precommitted: pass) — the Born-sampled SWAP test recovers exact
/// fidelity: identical → ≥ 0.95; orthogonal → ≤ 0.05; intermediate within
/// ±0.06 at 400 shots (seeded).
#[test]
fn gate_swap_test_recovers_fidelity() {
    let mut rng = Rng::new(99);
    let mut a = Register::zeros(2);
    a.rotate_y(0, 0.9);
    a.rotate_y(1, 0.4);
    let identical = a.swap_test(&a, &mut rng, 400);
    assert!(identical >= 0.95, "identical states: {identical:.3}");

    let mut orthogonal = Register::zeros(2);
    orthogonal.rotate_y(0, 0.9 + core::f64::consts::PI);
    orthogonal.rotate_y(1, 0.4);
    let sampled_orthogonal = orthogonal.swap_test(&a, &mut rng, 400);
    assert!(sampled_orthogonal <= 0.05, "orthogonal states: {sampled_orthogonal:.3}");

    let mut near = Register::zeros(2);
    near.rotate_y(0, 0.9 + 0.2);
    near.rotate_y(1, 0.4);
    let exact = near.fidelity(&a);
    let sampled_near = near.swap_test(&a, &mut rng, 400);
    assert!(
        (sampled_near - exact).abs() <= 0.06,
        "intermediate: exact {exact:.3}, sampled {sampled_near:.3}"
    );
}

/// Q2 (precommitted: pass) — Born-sampled naming (SWAP-test assignments,
/// 400 shots) produces the SAME window→prototype mapping as exact
/// naming, purity preserved.
#[test]
fn gate_born_sampled_naming_matches_exact() {
    let perceiver = formed();
    for class in 0..3 {
        let readings = window(10 + class as u64 * 300, class);
        let exact = perceiver
            .name_window(&readings)
            .expect("exact naming works");
        let sampled = perceiver
            .name_window_sampled(&readings, 400, (7 + class) as u64)
            .unwrap_or_else(|| panic!("sampled naming abstained on class {class}"));
        assert_eq!(
            exact, sampled,
            "Born-sampled naming must agree with exact naming (class {class})"
        );
    }
    // Purity through sampled naming (harness-side labels).
    let mut correct = 0usize;
    for class in 0..3 {
        for repeat in 0..3 {
            let readings = window(10 + class as u64 * 300 + repeat * 10, class);
            if let Some(name) = perceiver.name_window_sampled(&readings, 400, 31 + repeat) {
                let same_class = perceiver
                    .name_window(&window(10 + class as u64 * 300, class))
                    .expect("exact");
                if name == same_class {
                    correct += 1;
                }
            }
        }
    }
    assert!(correct >= 8, "sampled-naming purity {correct}/9");
}

/// A1 (precommitted: pass) — load-bearing: with no bank formed, nothing
/// names (0/9).
#[test]
fn gate_emergent_naming_is_load_bearing() {
    let perceiver = EmergentPerceiver::new(EmergentConfig::default());
    let mut named = 0;
    for class in 0..3 {
        for repeat in 0..3 {
            let readings = window(10 + class as u64 * 300 + repeat * 10, class);
            if perceiver.name_window(&readings).is_some() {
                named += 1;
            }
        }
    }
    assert_eq!(named, 0, "without formation there is no naming");
}

/// Logistic-map noise (deterministic per (window, seed)).
fn noised(base_id: u64, class: usize, seed: u64, amplitude: f64) -> Vec<Reading> {
    let mut x = ((seed % 991) as f64 + 0.117) / 991.3;
    x = 4.0 * x * (1.0 - x);
    window(base_id, class)
        .into_iter()
        .map(|mut r| {
            x = 4.0 * x * (1.0 - x);
            r.exploration += (x - 0.5) * 2.0 * amplitude;
            x = 4.0 * x * (1.0 - x);
            r.convergence += (x - 0.5) * 2.0 * amplitude;
            r
        })
        .collect()
}

/// A2 (precommitted: pass, FLAGGED UNCERTAIN at high amplitude) — the
/// emergent matcher degrades into abstention before misassignment:
/// across the noise sweep, misassignments ≤ 2 and never exceed
/// abstentions at any amplitude.
#[test]
fn gate_emergent_fail_safe_under_chaos() {
    let perceiver = formed();
    // Reference names from clean windows.
    let reference: Vec<usize> = (0..3)
        .map(|class| perceiver.name_window(&window(10 + class as u64 * 300, class)).expect("clean"))
        .collect();
    let mut total_misassigned = 0usize;
    for amplitude in [0.0f64, 0.04, 0.08, 0.12, 0.20, 0.30] {
        let mut misassigned = 0usize;
        let mut abstained = 0usize;
        let mut seed = 5u64;
        for class in 0..3 {
            for _ in 0..3 {
                let readings = if amplitude == 0.0 {
                    window(10 + class as u64 * 300, class)
                } else {
                    noised(10 + class as u64 * 300, class, seed, amplitude)
                };
                seed += 17;
                match perceiver.name_window(&readings) {
                    Some(name) if name == reference[class as usize] => {}
                    Some(_) => misassigned += 1,
                    None => abstained += 1,
                }
            }
        }
        eprintln!(
            "amplitude {amplitude:.2}: misassigned {misassigned}, abstained {abstained}"
        );
        total_misassigned += misassigned;
        assert!(
            misassigned <= abstained,
            "misassignment ({misassigned}) exceeded abstention ({abstained}) at amplitude {amplitude}"
        );
    }
    assert!(total_misassigned <= 2, "total misassignments {total_misassigned}");
}

/// A3 (precommitted: pass) — revision via drift: a formed prototype fed a
/// gradual regime shift tracks it (drift ON ends closer to the shifted
/// regime than drift OFF).
#[test]
fn gate_emergent_revision_via_drift() {
    let drift_config = EmergentConfig {
        reinforce_rate: 0.4,
        ..EmergentConfig::default()
    };
    let fixed_config = EmergentConfig {
        reinforce_rate: 0.4,
        drift: false,
        ..EmergentConfig::default()
    };
    let mut drifting = EmergentPerceiver::new(drift_config);
    let mut fixed = EmergentPerceiver::new(fixed_config);
    // Form on regime-0 readings.
    for r in window(10, 0) {
        drifting.ingest(&r);
        fixed.ingest(&r);
    }
    // Gradual shift toward regime-1 geometry (convergence 0.7 -> 0.3).
    for step in 0..12u64 {
        let convergence = 0.70 - 0.0333 * step as f64;
        let r = reading(100 + step, 0.80, convergence);
        drifting.ingest(&r);
        fixed.ingest(&r);
    }
    let target = EmergentPerceiver::encode(&reading(999, 0.80, 0.30));
    let drifted = drifting.prototype(0).expect("prototype 0").fidelity(&target);
    let stayed = fixed.prototype(0).expect("prototype 0").fidelity(&target);
    assert!(
        drifted > stayed,
        "drift must track the shift (drifted {drifted:.3} vs fixed {stayed:.3})"
    );
}
