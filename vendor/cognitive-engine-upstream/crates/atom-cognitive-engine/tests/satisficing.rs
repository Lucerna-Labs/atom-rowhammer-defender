//! Satisficing (W18, D46) — the admission discipline as gates: the
//! fast path earns its skip (identical claim, less effort), contested
//! ground still gets full deliberation, and the bridge survives when
//! the margin bar demands effort.

use atom_cognitive_engine::stacks::{kpr_stack, mother_query_stack, record_stack};
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

fn base() -> QueryTuning {
    QueryTuning::from_config(&EngineConfig::default())
}

fn satisficing(margin: f64) -> QueryTuning {
    QueryTuning {
        satisficing_margin: margin,
        ..QueryTuning::from_config(&EngineConfig::default())
    }
}

fn engine() -> CognitiveEngine {
    CognitiveEngine::new(EngineConfig::default()).expect("config")
}

/// A fluent cluster of three distinct same-value traces, PLUS a bonded
/// distant relative (echo bond to an orthogonal trace) that only
/// expansion can reach.
fn fluent_cluster_with_bridge() -> CognitiveEngine {
    let mut engine = engine();
    for offset_symbol in [1usize, 2, 3] {
        let mut cue = vec![0.0; 12];
        cue[0] = 1.0;
        cue[offset_symbol] = 0.8;
        let cue = normalize(&cue);
        for _ in 0..4 {
            engine.observe(&kpr_stack(), &cue, 1, 1.0).expect("observe");
        }
    }
    // The distant relative: orthogonal cue, committed, bonded to the
    // strongest cluster trace.
    engine
        .observe(&record_stack(), &cue12(8), 1, 1.0)
        .expect("record");
    let cluster_first = engine.traces()[0].id;
    let distant = engine.traces().last().expect("trace").id;
    engine.add_bond(cluster_first, distant).expect("bond");
    engine
}

#[test]
fn gate_fast_path_earns_its_skip() {
    // On a fluent local read, satisficing returns the IDENTICAL claim
    // with strictly less effort: the bonded distant trace is not
    // activated because the BFS never runs.
    let query = mother_query_stack();
    let full = {
        let engine = fluent_cluster_with_bridge();
        engine.query_tuned(&query, &cue12(0), base()).expect("full")
    };
    let fast = {
        let engine = fluent_cluster_with_bridge();
        engine
            .query_tuned(&query, &cue12(0), satisficing(0.5))
            .expect("fast")
    };
    assert_eq!(
        full.claim.map(|(v, _)| v),
        fast.claim.map(|(v, _)| v),
        "identical answer (float-weight epsilon aside)"
    );
    assert_eq!(
        full.fluency.state,
        atom_cognitive_engine::FluencyState::Fluent,
        "setup check: the local read is fluent"
    );
    assert!(
        fast.activated_all.len() < full.activated_all.len(),
        "the fast path activates fewer members (effort saved): {} vs {}",
        fast.activated_all.len(),
        full.activated_all.len()
    );
    // The distant relative is reached only under full deliberation.
    assert!(full.activated_all.len() > 3, "full path reaches the bond");
}

#[test]
fn gate_contested_ground_still_gets_full_deliberation() {
    // The near-tie scenario from W16: the local margin is BELOW the
    // bar, so satisficing must NOT skip — expansion runs and the
    // outcome matches full deliberation exactly.
    let query = mother_query_stack();
    let mut engine = engine();
    let mut left = vec![0.0; 12];
    left[0] = 1.0;
    left[2] = 0.8;
    let mut right = vec![0.0; 12];
    right[0] = 1.0;
    right[3] = 0.8;
    for (cue, value) in [(normalize(&left), 1usize), (normalize(&right), 2usize)] {
        for _ in 0..4 {
            engine
                .observe(&kpr_stack(), &cue, value, 1.0)
                .expect("observe");
        }
    }
    // Bond both to a distant trace so expansion would change membership.
    engine
        .observe(&record_stack(), &cue12(9), 1, 1.0)
        .expect("record");
    let left_id = engine.traces()[0].id;
    let distant = engine.traces().last().expect("trace").id;
    engine.add_bond(left_id, distant).expect("bond");
    let full = engine.query_tuned(&query, &cue12(0), base()).expect("full");
    let contested = engine
        .query_tuned(&query, &cue12(0), satisficing(0.5))
        .expect("fast");
    assert_eq!(
        full.fluency.state,
        atom_cognitive_engine::FluencyState::Contested,
        "setup check: contested local read"
    );
    assert_eq!(
        full.activated_all, contested.activated_all,
        "below the bar, satisficing must deliberate in full"
    );
    assert_eq!(full.claim, contested.claim);
}

#[test]
fn gate_defaults_off_and_deterministic() {
    let query = mother_query_stack();
    let engine = fluent_cluster_with_bridge();
    let plain = engine.query(&query, &cue12(0)).expect("plain");
    let defaulted = engine
        .query_tuned(&query, &cue12(0), base())
        .expect("defaulted");
    assert_eq!(plain.activated_all, defaulted.activated_all);
    let first = engine
        .query_tuned(&query, &cue12(0), satisficing(0.5))
        .expect("first");
    let second = engine
        .query_tuned(&query, &cue12(0), satisficing(0.5))
        .expect("second");
    assert_eq!(first.response_hash, second.response_hash);
    // Config validation and clamping.
    let bad = EngineConfig {
        satisficing_margin: 1.5,
        ..EngineConfig::default()
    };
    assert!(CognitiveEngine::new(bad).is_err());
    let clamped = satisficing(2.0).clamped();
    assert!((clamped.satisficing_margin - 1.0).abs() < 1e-12);
}
