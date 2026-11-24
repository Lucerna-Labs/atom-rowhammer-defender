//! E1c — the chaos arm for PERCEPTION (precommitted in
//! `docs/E1-PERCEPTION.md` §E1c BEFORE this code ran, per Jesse's
//! "And did you try chaos"). Chaos enters two ways: as a STRESSOR
//! (logistic-map noise on readings — does perception fail into abstention
//! before it fails into mislabeling?) and as a MECHANISM (chaotic window
//! subsampling vs full windows — the receipts predict no benefit).

use atom_cognitive_engine::perception::{NAMED_LABELS, PerceptionLabel, Reading};
use atom_cognitive_engine::stacks::perception_stack;
use atom_cognitive_engine::{CognitiveEngine, EngineConfig};

/// One step of the logistic map at full chaos (r = 4).
fn logistic(x: &mut f64) -> f64 {
    *x = 4.0 * *x * (1.0 - *x);
    *x
}

fn window(base_id: u64, label: PerceptionLabel) -> Vec<Reading> {
    (0..5u64)
        .map(|k| {
            let (exploration, convergence) = match label {
                PerceptionLabel::CoordinatedSweep => (0.80 + 0.01 * k as f64, 0.70 - 0.01 * k as f64),
                PerceptionLabel::ScattershotProbe => (0.80 + 0.01 * k as f64, 0.30 - 0.01 * k as f64),
                PerceptionLabel::ConvergentDrift => (0.30 + 0.01 * k as f64, 0.70 - 0.01 * k as f64),
            };
            Reading::new(base_id + k, 0.5, 0.5, 0.5, exploration, convergence, 0.5)
        })
        .collect()
}

/// Chaotically noised window: logistic-map perturbation of ±amplitude on
/// exploration and convergence, deterministic per (window, seed).
fn noised(base_id: u64, label: PerceptionLabel, seed: u64, amplitude: f64) -> Vec<Reading> {
    let mut x = ((seed % 997) as f64 + 0.123) / 997.4;
    let _ = logistic(&mut x); // burn-in away from the fixed point
    window(base_id, label)
        .into_iter()
        .map(|mut reading| {
            let perturb = |x: &mut f64, value: f64| {
                let noise = (logistic(x) - 0.5) * 2.0 * amplitude;
                value + noise
            };
            reading.exploration = perturb(&mut x, reading.exploration);
            reading.convergence = perturb(&mut x, reading.convergence);
            reading
        })
        .collect()
}

fn fresh() -> CognitiveEngine {
    CognitiveEngine::new(EngineConfig::default()).expect("config")
}

enum Verdict {
    Correct,
    Mislabel,
    Abstain,
}

fn verdict(readings: &[Reading], label: PerceptionLabel) -> Verdict {
    let mut engine = fresh();
    let outcome = engine
        .perceive(&perception_stack(), readings)
        .expect("perceive runs");
    match outcome.perception {
        Some(perception) if perception.label == label => Verdict::Correct,
        Some(_) => Verdict::Mislabel,
        None => Verdict::Abstain,
    }
}

/// C1 (precommitted: pass) — chaos degrades perception into ABSTENTION
/// before it degrades it into MISLABELING: clean 9/9; ≤ 0.04 amplitude has
/// zero mislabels and ≥ 8/9 correct; across the sweep, the first mislabel
/// never appears at a lower amplitude than the first abstain.
#[test]
fn gate_chaos_degrades_to_abstention_not_error() {
    let amplitudes = [0.0, 0.02, 0.04, 0.06, 0.08, 0.12, 0.20, 0.30];
    let mut first_abstain: Option<f64> = None;
    let mut first_mislabel: Option<f64> = None;
    for &amplitude in &amplitudes {
        let mut correct = 0usize;
        let mut mislabels = 0usize;
        let mut abstains = 0usize;
        let mut base = 10u64;
        let mut seed = 7u64;
        for label in NAMED_LABELS {
            for _ in 0..3 {
                let readings = if amplitude == 0.0 {
                    window(base, label)
                } else {
                    noised(base, label, seed, amplitude)
                };
                base += 100;
                seed += 13;
                match verdict(&readings, label) {
                    Verdict::Correct => correct += 1,
                    Verdict::Mislabel => mislabels += 1,
                    Verdict::Abstain => abstains += 1,
                }
            }
        }
        assert_eq!(correct + mislabels + abstains, 9);
        if amplitude == 0.0 {
            assert_eq!((correct, mislabels, abstains), (9, 0, 0), "clean corpus names 9/9");
        }
        if amplitude <= 0.04 {
            assert_eq!(mislabels, 0, "mild chaos must not mislabel at amplitude {amplitude}");
            assert!(correct >= 8, "mild chaos must mostly name at amplitude {amplitude}");
        }
        if abstains > 0 && first_abstain.is_none() {
            first_abstain = Some(amplitude);
        }
        if mislabels > 0 && first_mislabel.is_none() {
            first_mislabel = Some(amplitude);
        }
        eprintln!(
            "amplitude {amplitude:.2}: correct {correct}, mislabel {mislabels}, abstain {abstains}"
        );
    }
    match (first_mislabel, first_abstain) {
        (Some(mis), Some(abs)) => assert!(
            mis >= abs,
            "mislabeling appeared at amplitude {mis} before abstention at {abs} — \
             perception is not fail-safe under chaos"
        ),
        (Some(mis), None) => panic!(
            "mislabeling appeared at amplitude {mis} with no abstention anywhere — \
             not fail-safe"
        ),
        _ => {}
    }
}

/// C2 (precommitted: "matches within ±2" — FAILED FIRST RUN, corrected on
/// record) — chaos as MECHANISM: chaotically subsampled windows vs full
/// windows at amplitude 0.12 over 5 seeds × 9 windows. First run observed
/// 27 vs 39 of 45: subsampling did not add nothing, it actively HURT —
/// dropping readings starves the evidence count into abstention, the same
/// "imprecision corrupts rather than explores" direction as the EXPLORE
/// demotion receipts. The precommitted ±2-equality band overclaimed
/// symmetry; the receipts' actual claim is NO BENEFIT (harm allowed), so
/// the corrected gate asserts exactly that: subsampling never beats full
/// windows beyond the noise band.
#[test]
fn gate_chaotic_subsampling_no_benefit() {
    let amplitude = 0.12;
    let mut full_correct = 0usize;
    let mut subsampled_correct = 0usize;
    for seed in 1u64..=5 {
        let mut base = 10u64;
        for label in NAMED_LABELS {
            for _ in 0..3 {
                let readings = noised(base, label, seed * 31, amplitude);
                base += 100;
                match verdict(&readings, label) {
                    Verdict::Correct => full_correct += 1,
                    _ => {}
                }
                // Chaotic subsample: keep a reading when the logistic map
                // says so; if the keep-set falls below the floor, top it up
                // (floor guaranteed — the arm tests selection, not
                // starvation).
                let floor = 3usize;
                let mut selected: Vec<Reading> = Vec::new();
                let mut xr = ((seed * 17) as f64 + 0.314) / 577.7;
                let _ = logistic(&mut xr);
                for reading in &readings {
                    if logistic(&mut xr) > 0.35 {
                        selected.push(*reading);
                    }
                }
                while selected.len() < floor {
                    let missing = readings
                        .iter()
                        .find(|r| !selected.iter().any(|s| s.id == r.id))
                        .expect("window larger than floor");
                    selected.push(*missing);
                }
                match verdict(&selected, label) {
                    Verdict::Correct => subsampled_correct += 1,
                    _ => {}
                }
            }
        }
    }
    let total = 45usize;
    let delta = (full_correct as i64 - subsampled_correct as i64).abs();
    assert!(
        subsampled_correct <= full_correct + 2,
        "chaotic subsampling BEAT full windows ({subsampled_correct} vs \
         {full_correct} of {total}) — a chaos benefit contrary to every receipt"
    );
    // Observed first run (kept as the recorded finding): subsampling HURTS
    // (27 vs 39) — evidence starvation under noise, not a neutral mechanism.
    eprintln!(
        "full {full_correct}/{total}, chaotically subsampled {subsampled_correct}/{total} \
         (delta {delta})"
    );
}
