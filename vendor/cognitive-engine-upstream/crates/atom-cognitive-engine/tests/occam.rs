//! Occam/dedup selection (W17, D45) — the admission discipline as
//! gates: the correlated-copies adversary (W14's open hole), distinct
//! evidence untouched, the dedup-then-cap stack, benchmark integrity,
//! determinism, and validation.

use atom_cognitive_engine::stacks::{kpr_stack, query_stack, record_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig, FluencyState, QueryTuning};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

fn normalize(cue: &[f64]) -> Vec<f64> {
    let norm: f64 = cue.iter().map(|v| v * v).sum::<f64>().sqrt();
    cue.iter().map(|v| v / norm).collect()
}

fn base() -> QueryTuning {
    QueryTuning::from_config(&EngineConfig::default())
}

fn with(cap: f64, dedup: f64) -> QueryTuning {
    QueryTuning {
        robust_weight_cap: cap,
        dedup_threshold: dedup,
        ..QueryTuning::from_config(&EngineConfig::default())
    }
}

/// The correlated-copies adversary: a coalition of two DISTINCT traces
/// voting value 1, versus THREE near-identical copies (mutual cue sim
/// ≈ 0.98) voting value 2 near the query cue.
///
/// Geometry lesson from the first attempt (kept — it is itself a
/// finding): KPR-committed near-copies MERGE at observe time (mutual
/// sim above the match threshold → later copies bind and revise the
/// first). The observe path already practices a form of Occam. The
/// correlated-copies attack therefore requires RECORD-mode copies —
/// near-duplicates installed as documents, which is exactly how the
/// duplicate-vote hazard arrives in real corpora.
fn copies_engine() -> CognitiveEngine {
    let mut engine = CognitiveEngine::new(EngineConfig::default()).expect("config");
    // Coalition: two distinct traces, value 1.
    for offset_symbol in [2usize, 3] {
        let mut cue = vec![0.0; 12];
        cue[0] = 1.0;
        cue[offset_symbol] = 1.0;
        let cue = normalize(&cue);
        for _ in 0..3 {
            engine.observe(&kpr_stack(), &cue, 1, 1.0).expect("observe");
        }
    }
    // The adversary's near-copies: value 2, cues almost identical to the
    // query cue, installed as RECORDS (documents never merge) at high
    // salience so each is a strong committed trace.
    for jitter_symbol in [1usize, 4, 5] {
        let mut cue = vec![0.0; 12];
        cue[0] = 1.0;
        cue[jitter_symbol] = 0.15;
        let cue = normalize(&cue);
        engine
            .observe(&record_stack(), &cue, 2, 1.8)
            .expect("record");
    }
    engine
}

#[test]
fn gate_correlated_copies_defeated() {
    // The hole W14 left open: three correlated copies each sit UNDER the
    // 0.35 cap, so cap-only still lets the wrong value win. Dedup
    // collapses the copies to one voter first — then the coalition wins.
    let query = query_stack();
    let engine = copies_engine();
    // Cap alone: the wrong value still wins (the open hole, demonstrated).
    let cap_only = engine
        .query_tuned(&query, &cue12(0), with(0.35, 1.01))
        .expect("cap only");
    assert_eq!(
        cap_only.claim.map(|(v, _)| v),
        Some(2),
        "setup check: correlated copies defeat the cap alone"
    );
    // Dedup + cap, stacked in order: the copies become one voter.
    let stacked = engine
        .query_tuned(&query, &cue12(0), with(0.35, 0.95))
        .expect("dedup + cap");
    assert_eq!(
        stacked.claim.map(|(v, _)| v),
        Some(1),
        "dedup collapses correlated copies; the coalition of distinct \
         evidence wins"
    );
}

#[test]
fn gate_distinct_evidence_not_collapsed() {
    // The W16 fluent cluster (pairwise sim ≈ 0.61) must keep its
    // independent voters under dedup: Fluent read survives, claim
    // unchanged. Occam collapses REDUNDANCY, not diversity.
    let query = query_stack();
    let mut engine = CognitiveEngine::new(EngineConfig::default()).expect("config");
    for offset_symbol in [1usize, 2, 3] {
        let mut cue = vec![0.0; 12];
        cue[0] = 1.0;
        cue[offset_symbol] = 0.8;
        let cue = normalize(&cue);
        for _ in 0..4 {
            engine.observe(&kpr_stack(), &cue, 1, 1.0).expect("observe");
        }
    }
    let plain = engine.query(&query, &cue12(0)).expect("plain");
    let deduped = engine
        .query_tuned(&query, &cue12(0), with(1.0, 0.95))
        .expect("dedup on");
    assert_eq!(plain.claim, deduped.claim);
    assert_eq!(
        deduped.fluency.state,
        FluencyState::Fluent,
        "distinct traces stay distinct voters"
    );
    assert!(
        deduped.fluency.eligible_count > 2,
        "no collapse at pairwise sim ≈ 0.61"
    );
}

#[test]
fn gate_dedup_before_cap_is_the_order() {
    // Stacking-order proof: the composition (dedup THEN cap) is what
    // wins; neither alone does. Also the clamp keeps nonsense out.
    let query = query_stack();
    let engine = copies_engine();
    let dedup_only = engine
        .query_tuned(&query, &cue12(0), with(1.0, 0.95))
        .expect("dedup only");
    // Dedup alone: one copy survives with a dominant renormalized
    // weight — still wrong (the single strongest voter is the copy).
    assert_eq!(dedup_only.claim.map(|(v, _)| v), Some(2));
    // Config validation + clamping.
    let bad = EngineConfig {
        dedup_threshold: 0.2,
        ..EngineConfig::default()
    };
    assert!(CognitiveEngine::new(bad).is_err());
    let clamped = with(0.35, 0.10).clamped();
    assert!((clamped.dedup_threshold - 0.5).abs() < 1e-12);
}

#[test]
fn gate_deterministic_replay() {
    let query = query_stack();
    let engine = copies_engine();
    let left = engine
        .query_tuned(&query, &cue12(0), with(0.35, 0.95))
        .expect("first");
    let right = engine
        .query_tuned(&query, &cue12(0), with(0.35, 0.95))
        .expect("second");
    assert_eq!(left.response_hash, right.response_hash);
    assert_eq!(left.fluency, right.fluency);
}

#[test]
fn gate_defaults_change_nothing() {
    // D42's with/without law: default tuning (dedup 1.01) is
    // byte-identical to the pre-W17 world — asserted against the
    // copies engine where any accidental dedup would show.
    let query = query_stack();
    let engine = copies_engine();
    let plain = engine.query(&query, &cue12(0)).expect("plain query");
    let defaulted = engine
        .query_tuned(&query, &cue12(0), base())
        .expect("defaulted");
    assert_eq!(plain.claim, defaulted.claim);
    assert_eq!(plain.response_hash, defaulted.response_hash);
}
