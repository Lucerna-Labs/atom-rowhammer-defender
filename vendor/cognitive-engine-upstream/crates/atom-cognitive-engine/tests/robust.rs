//! Robust aggregation (W14, D41) — the admission discipline as gates:
//! the confident-wrong adversary, the no-harm control, the
//! majority-respected bound, the cap-vs-trim ablation, determinism, and
//! the claim benchmark re-run under robust tuning.

use atom_cognitive_engine::stacks::{kpr_stack, observe_stack, query_stack, record_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig, QueryTuning};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

fn normalize(cue: &[f64]) -> Vec<f64> {
    let norm: f64 = cue.iter().map(|v| v * v).sum::<f64>().sqrt();
    cue.iter().map(|v| v / norm).collect()
}

fn base_tuning() -> QueryTuning {
    QueryTuning::from_config(&EngineConfig::default())
}

fn robust(cap: f64) -> QueryTuning {
    QueryTuning {
        robust_weight_cap: cap,
        ..QueryTuning::from_config(&EngineConfig::default())
    }
}

/// The adversarial scenario: a 2-trace coalition voting value 1 at ~0.707
/// similarity (just above the 0.68 query gate, safely BELOW the 0.72
/// match threshold so nothing binds across), and ONE confident adversary
/// voting value 2 at similarity 1.0 with heavy reinforcement.
///
/// Geometry lesson from the first attempt (caught by the diagnostic):
/// if the coalition sits above the match threshold, the adversary's own
/// observations BIND and revise the coalition — manufacturing a genuine
/// majority the cap then rightly refuses to flip. The window between
/// the query gate (0.68) and the match threshold (0.72) is exactly
/// where a single-adversary scenario can be constructed.
fn adversarial_engine() -> CognitiveEngine {
    let mut engine = CognitiveEngine::new(EngineConfig::default()).expect("config");
    // Coalition: two traces at cos = 1/sqrt(2) ≈ 0.7071 to the query cue.
    for offset_symbol in [2usize, 3] {
        let mut cue = vec![0.0; 12];
        cue[0] = 1.0;
        cue[offset_symbol] = 1.0;
        let cue = normalize(&cue);
        for _ in 0..3 {
            engine.observe(&kpr_stack(), &cue, 1, 1.0).expect("observe");
        }
    }
    // Adversary: exactly on the query cue (sim 1.0 to itself, 0.7071 to
    // the coalition — below the match threshold, so it nucleates alone),
    // then reinforced heavily for dominant reliability.
    let observe = observe_stack();
    for _ in 0..15 {
        engine
            .observe(&observe, &cue12(0), 2, 1.0)
            .expect("observe");
    }
    engine
}

#[test]
fn gate_confident_wrong_adversary_defeated() {
    // The money gate: under plain aggregation the single confident
    // adversary outvotes the coalition (wrong answer); under the
    // breakdown bound the coalition wins.
    let query = query_stack();
    let engine = adversarial_engine();
    let plain = engine
        .query_tuned(&query, &cue12(0), base_tuning())
        .expect("plain");
    assert_eq!(
        plain.claim.map(|(v, _)| v),
        Some(2),
        "setup check: the adversary must dominate the plain mixture"
    );
    let robust_outcome = engine
        .query_tuned(&query, &cue12(0), robust(0.35))
        .expect("robust");
    assert_eq!(
        robust_outcome.claim.map(|(v, _)| v),
        Some(1),
        "the coalition must outvote the capped adversary"
    );
}

#[test]
fn gate_no_harm_without_an_adversary() {
    // No adversary: a single consistent, reinforced trace answers the
    // same under plain and robust aggregation — the bound changes
    // nothing when there is nothing to defend against.
    let query = query_stack();
    let mut engine = CognitiveEngine::new(EngineConfig::default()).expect("config");
    for _ in 0..6 {
        engine
            .observe(&kpr_stack(), &cue12(4), 1, 1.0)
            .expect("observe");
    }
    let plain = engine
        .query_tuned(&query, &cue12(4), base_tuning())
        .expect("plain");
    let robust_outcome = engine
        .query_tuned(&query, &cue12(4), robust(0.35))
        .expect("robust");
    assert_eq!(plain.claim.map(|(v, _)| v), Some(1));
    assert_eq!(
        robust_outcome.claim.map(|(v, _)| v),
        Some(1),
        "robust aggregation must not change the honest answer"
    );
}

#[test]
fn gate_genuine_majority_still_wins() {
    // The bound is not a veto: three adversaries voting value 2 against
    // one coalition trace voting value 1 keep value 2 — a real majority
    // deserves its win. Robust ≠ minority rule.
    let query = query_stack();
    let mut engine = CognitiveEngine::new(EngineConfig::default()).expect("config");
    // One value-1 trace off-axis.
    let mut cue = vec![0.0; 12];
    cue[0] = 1.0;
    cue[6] = 0.9;
    let cue = normalize(&cue);
    for _ in 0..4 {
        engine
            .observe(&record_stack(), &cue, 1, 1.0)
            .expect("record");
    }
    // Three value-2 traces near the query, reinforced.
    for offset_symbol in [1usize, 2, 3] {
        let mut cue = vec![0.0; 12];
        cue[0] = 1.0;
        cue[offset_symbol] = 0.5;
        let cue = normalize(&cue);
        for _ in 0..4 {
            engine
                .observe(&record_stack(), &cue, 2, 1.0)
                .expect("record");
        }
    }
    let plain = engine
        .query_tuned(&query, &cue12(0), base_tuning())
        .expect("plain");
    let robust_outcome = engine
        .query_tuned(&query, &cue12(0), robust(0.35))
        .expect("robust");
    assert_eq!(plain.claim.map(|(v, _)| v), Some(2), "majority wins plain");
    assert_eq!(
        robust_outcome.claim.map(|(v, _)| v),
        Some(2),
        "majority must still win under the cap — the bound is not a veto"
    );
}

#[test]
fn gate_ablation_cap_is_load_bearing() {
    // Ablation: trim alone does NOT stop a heavy adversary (trim removes
    // the LOWEST weights — the coalition, not the adversary). Only the
    // cap carries the defense.
    let query = query_stack();
    let engine = adversarial_engine();
    let trim_only = QueryTuning {
        robust_weight_cap: 1.0,
        robust_trim: 1,
        ..QueryTuning::from_config(&EngineConfig::default())
    };
    let outcome = engine
        .query_tuned(&query, &cue12(0), trim_only)
        .expect("trim only");
    assert_eq!(
        outcome.claim.map(|(v, _)| v),
        Some(2),
        "trim alone must not defeat the adversary — the cap is load-bearing"
    );
    let cap_only = QueryTuning {
        robust_weight_cap: 0.35,
        robust_trim: 0,
        ..QueryTuning::from_config(&EngineConfig::default())
    };
    let outcome = engine
        .query_tuned(&query, &cue12(0), cap_only)
        .expect("cap only");
    assert_eq!(outcome.claim.map(|(v, _)| v), Some(1));
}

#[test]
fn gate_deterministic_and_validated() {
    // Identical robust queries replay identically; nonsense knobs fail
    // closed at both config and tuning levels.
    let query = query_stack();
    let engine = adversarial_engine();
    let left = engine
        .query_tuned(&query, &cue12(0), robust(0.35))
        .expect("first");
    let right = engine
        .query_tuned(&query, &cue12(0), robust(0.35))
        .expect("second");
    assert_eq!(left.response_hash, right.response_hash);
    let bad = EngineConfig {
        robust_weight_cap: 0.05,
        ..EngineConfig::default()
    };
    assert!(CognitiveEngine::new(bad).is_err());
    let bad = EngineConfig {
        robust_trim: 99,
        ..EngineConfig::default()
    };
    assert!(CognitiveEngine::new(bad).is_err());
    // Clamping: an out-of-range cap in tuning is clamped, not applied.
    let clamped = robust(0.01).clamped();
    assert!((clamped.robust_weight_cap - 0.15).abs() < 1e-12);
}

#[test]
fn gate_existing_behavior_unchanged_by_default() {
    // The operator's with/without law: defaults off means every already
    // validated surface runs byte-identically — same query outcomes with
    // the default tuning as before the mechanism existed.
    let query = query_stack();
    let mut engine = CognitiveEngine::new(EngineConfig::default()).expect("config");
    for symbol in 0..4 {
        for _ in 0..4 {
            engine
                .observe(&kpr_stack(), &cue12(symbol), symbol % 4, 1.0)
                .expect("observe");
        }
    }
    for symbol in 0..4 {
        let plain = engine.query(&query, &cue12(symbol)).expect("plain");
        let defaulted = engine
            .query_tuned(&query, &cue12(symbol), base_tuning())
            .expect("defaulted");
        assert_eq!(plain.claim, defaulted.claim);
        assert_eq!(plain.response_hash, defaulted.response_hash);
    }
}
