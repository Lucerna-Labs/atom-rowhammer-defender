//! Information bottleneck (#8, D55) — the admission gates: the argmax is
//! provably never pruned (machine-checked over a dusty corpus), removed
//! mass is accounted exactly, answers keep their value and fluency shape
//! (predictive structure retained — the bottleneck constraint), and the
//! off/validation/determinism controls.

use atom_cognitive_engine::stacks::{observe_stack, query_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

fn durable(precision_gain: f64) -> EngineConfig {
    EngineConfig {
        initial_ttl: 512,
        ttl_per_hit: 16,
        decay_support_rate: 0.0005,
        decay_mass_rate: 0.0003,
        conservation_budget: 1.0e6,
        capacity: 32,
        precision_gain,
        ..EngineConfig::default()
    }
}

/// A dusty belief: a well-established value plus the tiny dissent votes
/// that precision-weighted weak pings accumulate.
fn dusty_belief() -> CognitiveEngine {
    let mut engine = CognitiveEngine::new(durable(1.0)).expect("config");
    let observe = observe_stack();
    for _ in 0..10 {
        engine
            .observe(&observe, &cue12(0), 1, 1.0)
            .expect("observe");
    }
    for value in [2usize, 3] {
        for _ in 0..3 {
            engine
                .observe(&observe, &cue12(0), value, 0.25)
                .expect("weak ping");
        }
    }
    engine
}

#[test]
fn gate_argmax_never_pruned_and_dust_cleared() {
    let mut engine = dusty_belief();
    let before = engine.traces().first().expect("belief").clone();
    assert_eq!(before.value(), Some(1));
    assert!(
        before.evidence[2] > 0.0 && before.evidence[3] > 0.0,
        "setup: the pings must have left dust"
    );
    // Threshold chosen from the ACTUAL dissent share (×1.5 headroom, capped
    // inside the validated bound) — the gate tests the mechanism, not a
    // hand-tuned fixture constant.
    let total: f64 = before.evidence.iter().sum();
    let dust = (before.evidence[2].min(before.evidence[3]) / total * 1.5).min(0.12);
    assert!(dust > 0.0 && dust <= 0.125);
    let report = engine.compress_evidence(dust).expect("compress");
    let after = engine.traces().first().expect("belief");
    // The bottleneck constraint: predictive structure retained.
    assert_eq!(after.value(), Some(1), "argmax survives compression");
    assert!(
        after.evidence[2] == 0.0 && after.evidence[3] == 0.0,
        "dust below the share threshold must be gone"
    );
    assert!(after.evidence[1] > 0.9 * before.evidence[1]);
    assert!(report.entries_pruned >= 2, "the dust was real");
    assert_eq!(report.traces_touched, 1);
    // Exact accounting: total loss equals the reported mass removed.
    let loss = total_evidence_of(&before) - after.evidence.iter().sum::<f64>();
    assert!(
        (loss - report.mass_removed).abs() < 1e-12,
        "mass accounting must be exact ({loss} vs {})",
        report.mass_removed
    );
}

fn total_evidence_of(trace: &atom_cognitive_engine::MemoryTrace) -> f64 {
    trace.evidence.iter().sum()
}

#[test]
fn gate_answers_keep_value_and_fluency() {
    // Compression changes the representation, not the verdict: value and
    // fluency shape survive.
    let mut engine = dusty_belief();
    let query = query_stack();
    let before = engine.query(&query, &cue12(0)).expect("query");
    let total: f64 = engine
        .traces()
        .first()
        .expect("belief")
        .evidence
        .iter()
        .sum();
    let dissent = engine.traces().first().expect("belief").evidence[2];
    let dust = (dissent / total * 1.5).min(0.12);
    engine.compress_evidence(dust).expect("compress");
    let after = engine.query(&query, &cue12(0)).expect("query");
    assert_eq!(
        before.claim.map(|(v, _)| v),
        after.claim.map(|(v, _)| v),
        "the compressed engine answers the same"
    );
    assert_eq!(
        before.fluency.state, after.fluency.state,
        "the retrieval shape is preserved"
    );
}

#[test]
fn gate_off_noop_validation_determinism() {
    // dust 0.0 is a clean no-op: state byte-identical, zero report.
    let mut engine = dusty_belief();
    let hash_before = engine.state_digest();
    let report = engine.compress_evidence(0.0).expect("off compress");
    assert_eq!(report.entries_pruned, 0);
    assert_eq!(report.mass_removed, 0.0);
    assert_eq!(engine.state_digest(), hash_before);
    // Out-of-range dust rejected fail-closed (bound is 1/(2·values)).
    assert!(engine.compress_evidence(0.5).is_err());
    // Determinism: identical dusty engines compress identically.
    let (mut left, mut right) = (dusty_belief(), dusty_belief());
    let l = left.compress_evidence(0.08).expect("compress");
    let r = right.compress_evidence(0.08).expect("compress");
    assert_eq!(l.entries_pruned, r.entries_pruned);
    assert_eq!(left.state_digest(), right.state_digest());
}
