//! Retrieval fluency as a metacognitive signal (W16, D44) — the
//! admission discipline as gates, including the honesty condition:
//! fluency is a fact about the RETRIEVAL, never about the TRUTH.

use atom_cognitive_engine::stacks::{kpr_stack, query_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig, FluencyState};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

fn normalize(cue: &[f64]) -> Vec<f64> {
    let norm: f64 = cue.iter().map(|v| v * v).sum::<f64>().sqrt();
    cue.iter().map(|v| v / norm).collect()
}

fn engine() -> CognitiveEngine {
    CognitiveEngine::new(EngineConfig::default()).expect("config valid")
}

/// Commit a trace through KPR (persistent signal).
fn commit(engine: &mut CognitiveEngine, cue: &[f64], value: usize) {
    for _ in 0..4 {
        engine
            .observe(&kpr_stack(), cue, value, 1.0)
            .expect("observe");
    }
}

#[test]
fn gate_fluent_read() {
    // A well-supported, unopposed knowledge region reads Fluent:
    // several traces agree, the margin is wide, real support.
    // Geometry: offsets at x=0.8 give cos-to-query ≈ 0.781 (eligible)
    // and pairwise cos ≈ 0.61 (below the 0.72 match threshold, so the
    // cluster nucleates as separate traces instead of binding into one).
    let query = query_stack();
    let mut engine = engine();
    for offset_symbol in [1usize, 2, 3] {
        let mut cue = vec![0.0; 12];
        cue[0] = 1.0;
        cue[offset_symbol] = 0.8;
        let cue = normalize(&cue);
        commit(&mut engine, &cue, 1);
    }
    let outcome = engine.query(&query, &cue12(0)).expect("query");
    assert_eq!(outcome.claim.map(|(v, _)| v), Some(1));
    assert_eq!(outcome.fluency.state, FluencyState::Fluent);
    assert!(outcome.fluency.margin >= 0.15, "wide margin");
    assert!(outcome.fluency.eligible_count > 2, "real support");
    assert!(!outcome.fluency.state.describe().is_empty());
}

#[test]
fn gate_contested_read() {
    // Two equal-reliability coalitions at symmetric cues, opposite
    // values: weights near 0.5/0.5, margin near zero — the engine
    // KNOWS the retrieval is contested even while returning a claim.
    let query = query_stack();
    let mut engine = engine();
    let mut left = vec![0.0; 12];
    left[0] = 1.0;
    left[2] = 0.8;
    let mut right = vec![0.0; 12];
    right[0] = 1.0;
    right[3] = 0.8;
    commit(&mut engine, &normalize(&left), 1);
    commit(&mut engine, &normalize(&right), 2);
    let outcome = engine.query(&query, &cue12(0)).expect("query");
    assert!(outcome.claim.is_some(), "a claim is still returned");
    assert_eq!(
        outcome.fluency.state,
        FluencyState::Contested,
        "a near-tie must read as contested — margin {:.4}",
        outcome.fluency.margin
    );
    assert!(outcome.fluency.margin < 0.15);
}

#[test]
fn gate_sparse_and_blank_reads() {
    let query = query_stack();
    let mut engine = engine();
    // Blank: nothing eligible.
    let outcome = engine.query(&query, &cue12(6)).expect("query");
    assert_eq!(outcome.fluency.state, FluencyState::Blank);
    assert_eq!(outcome.fluency.eligible_count, 0);
    assert_eq!(outcome.claim, None, "blank matches the abstention");
    // Sparse: a single committed trace — support exists but it is thin.
    commit(&mut engine, &cue12(0), 1);
    let outcome = engine.query(&query, &cue12(0)).expect("query");
    assert_eq!(outcome.fluency.state, FluencyState::Sparse);
    assert_eq!(outcome.fluency.eligible_count, 1);
    assert_eq!(outcome.claim.map(|(v, _)| v), Some(1));
}

#[test]
fn gate_fluency_never_asserts() {
    // Read-only law: fluency metadata cannot change the verdict. The
    // claim, eligible count, and full activation set are identical to a
    // world without the signal (they are computed before it, and the
    // response hash does not include fluency).
    let query = query_stack();
    let mut engine = engine();
    commit(&mut engine, &cue12(0), 1);
    commit(&mut engine, &cue12(6), 2);
    let outcome = engine.query(&query, &cue12(0)).expect("query");
    let again = engine.query(&query, &cue12(0)).expect("again");
    assert_eq!(outcome.claim, again.claim);
    assert_eq!(outcome.response_hash, again.response_hash);
    assert_eq!(outcome.activated_all, again.activated_all);
    // Fluency on the blank read is present but claim stays None.
    let blank = engine.query(&query, &cue12(9)).expect("blank");
    assert_eq!(blank.fluency.state, FluencyState::Blank);
    assert_eq!(blank.claim, None);
}

#[test]
fn gate_fluency_is_not_correctness() {
    // THE honesty condition: a wrong-but-unopposed CONSENSUS reads
    // Fluent. Three traces agree on value 3 with no opposing evidence —
    // the retrieval shape is unambiguous (Fluent, wide margin) while the
    // answer is whatever the cluster says. The engine can feel certain
    // and be wrong, exactly like a person. Fluency is a fact about the
    // retrieval, never about the truth — which is why W14's breakdown
    // bound exists as a SEPARATE mechanism.
    let query = query_stack();
    let mut engine = engine();
    for offset_symbol in [1usize, 2, 3] {
        let mut cue = vec![0.0; 12];
        cue[0] = 1.0;
        cue[offset_symbol] = 0.8;
        let cue = normalize(&cue);
        commit(&mut engine, &cue, 3);
    }
    let outcome = engine.query(&query, &cue12(0)).expect("query");
    assert_eq!(outcome.fluency.state, FluencyState::Fluent);
    assert!(outcome.fluency.margin > 0.5, "unambiguous shape");
    assert!(
        !outcome
            .fluency
            .state
            .describe()
            .to_lowercase()
            .contains("correct"),
        "fluency descriptions must never assert correctness"
    );
}

#[test]
fn gate_deterministic_and_structured() {
    let query = query_stack();
    let mut engine = engine();
    for symbol in 0..3 {
        commit(&mut engine, &cue12(symbol * 4), symbol % 4);
    }
    for symbol in 0..3 {
        let left = engine.query(&query, &cue12(symbol * 4)).expect("q1");
        let right = engine.query(&query, &cue12(symbol * 4)).expect("q2");
        assert_eq!(left.fluency, right.fluency, "fluency must replay exactly");
    }
}
