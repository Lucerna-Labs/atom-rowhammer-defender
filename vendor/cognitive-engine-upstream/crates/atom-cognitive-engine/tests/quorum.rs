//! Quorum sensing (#15, D59) — the admission gates: correlated copies
//! count ONCE (correlated worlds cannot fake a quorum — the risk column,
//! gated by construction), distributed support earns deliberation (the
//! bridge fires only when the crowd is real), the singleton fast path
//! keeps the anchor's own answer while skipping the walk, and the
//! defaults-off / determinism controls.

use atom_cognitive_engine::stacks::{mother_query_stack, observe_stack, record_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig, QueryTuning};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

fn durable() -> EngineConfig {
    EngineConfig {
        initial_ttl: 512,
        ttl_per_hit: 16,
        decay_support_rate: 0.0005,
        decay_mass_rate: 0.0003,
        conservation_budget: 1.0e6,
        capacity: 32,
        surprise_boost: 1.0,
        ..EngineConfig::default()
    }
}

fn quorum(engine: &CognitiveEngine, min: usize) -> QueryTuning {
    QueryTuning {
        quorum_min: min,
        ..QueryTuning::from_config(engine.config())
    }
}

/// A bonded world: `anchors` record-mode memories at `anchor_cue`
/// (identical cues = a correlated chorus), each bonded to a far memory
/// at cue12(5) voting value 3.
fn bonded_world(anchor_cue: usize, anchors: usize) -> CognitiveEngine {
    let mut engine = CognitiveEngine::new(durable()).expect("config");
    let record = record_stack();
    let observe = observe_stack();
    for _ in 0..anchors {
        engine
            .observe(&record, &cue12(anchor_cue), 1, 1.0)
            .expect("anchor");
    }
    for _ in 0..3 {
        engine.observe(&observe, &cue12(5), 3, 1.0).expect("far");
    }
    let ids: Vec<u64> = engine.traces().iter().map(|t| t.id).collect();
    for anchor_id in &ids[..anchors] {
        engine.add_bond(*anchor_id, ids[anchors]).expect("bond");
    }
    engine
}

fn far_activated(engine: &CognitiveEngine, tuning: QueryTuning, query_cue: usize) -> bool {
    let outcome = engine
        .query_tuned(&mother_query_stack(), &cue12(query_cue), tuning)
        .expect("query");
    let far_id = engine.traces()[engine.traces().len() - 1].id;
    outcome.activated_all.contains(&far_id)
}

#[test]
fn gate_correlated_copies_cannot_fake_a_quorum() {
    // Four identical-cue copies are ONE distinct supporter. With
    // quorum_min 3 the deliberation is not earned: the far bonded
    // memory stays unreached. The same world with quorum off reaches it.
    let engine = bonded_world(0, 4);
    assert!(
        !far_activated(&engine, quorum(&engine, 3), 0),
        "copies must not fake a quorum"
    );
    assert!(
        far_activated(&engine, quorum(&engine, 0), 0),
        "quorum off reaches the bonded memory"
    );
    // The report says exactly why: one distinct supporter, quorum unmet.
    let outcome = engine
        .query_tuned(&mother_query_stack(), &cue12(0), quorum(&engine, 3))
        .expect("query");
    assert_eq!(outcome.quorum.distinct_supporters, 1);
    assert!(!outcome.quorum.quorum_met);
}

#[test]
fn gate_distributed_support_earns_deliberation() {
    // Three DISTINCT anchors: the shared blend (dims 0-2) plus one
    // private perturbation dim each (dims 3-5, weight 1.0). Each sits at
    // similarity 0.866 to the blend query (eligible), pairwise 0.75 —
    // BELOW the 0.90 clustering line: three distinct supporters. Each
    // bonds to the far memory; quorum 3 earns the bridge.
    let mut engine = CognitiveEngine::new(durable()).expect("config");
    let record = record_stack();
    let observe = observe_stack();
    for extra in [3usize, 4, 5] {
        let mut cue = vec![0.0; 12];
        cue[0] = 1.0;
        cue[1] = 1.0;
        cue[2] = 1.0;
        cue[extra] = 1.0;
        engine.observe(&record, &cue, 1, 1.0).expect("anchor");
    }
    for _ in 0..3 {
        engine.observe(&observe, &cue12(5), 3, 1.0).expect("far");
    }
    let ids: Vec<u64> = engine.traces().iter().map(|t| t.id).collect();
    for anchor_id in &ids[..3] {
        engine.add_bond(*anchor_id, ids[3]).expect("bond");
    }
    let mut cue = vec![0.0; 12];
    cue[0] = 1.0;
    cue[1] = 1.0;
    cue[2] = 1.0;
    let outcome = engine
        .query_tuned(&mother_query_stack(), &cue, quorum(&engine, 3))
        .expect("query");
    assert_eq!(outcome.quorum.distinct_supporters, 3);
    assert!(outcome.quorum.quorum_met);
    assert!(
        outcome.activated_all.contains(&ids[3]),
        "the bridge fires on distributed support"
    );
    // And below the bar it does not.
    let outcome = engine
        .query_tuned(&mother_query_stack(), &cue, quorum(&engine, 4))
        .expect("query");
    assert!(!outcome.quorum.quorum_met);
    assert!(!outcome.activated_all.contains(&ids[3]));
}

#[test]
fn gate_singleton_fast_path_keeps_the_answer() {
    // One anchor, quorum 2: the walk is skipped, the anchor still
    // answers its own value — savings with no answer cost at the anchor.
    let engine = bonded_world(0, 1);
    let outcome = engine
        .query_tuned(&mother_query_stack(), &cue12(0), quorum(&engine, 2))
        .expect("query");
    assert_eq!(outcome.quorum.distinct_supporters, 1);
    assert!(!outcome.quorum.quorum_met);
    assert_eq!(
        outcome.claim.map(|(v, _)| v),
        Some(1),
        "the anchor answers its own value without the walk"
    );
    assert!(
        !far_activated(&engine, quorum(&engine, 2), 0),
        "un-earned deliberation skips the bonded walk"
    );
}

#[test]
fn gate_defaults_off_determinism() {
    // quorum 0: byte-identical to the plain query.
    let engine = bonded_world(0, 2);
    let plain = engine
        .query(&mother_query_stack(), &cue12(0))
        .expect("plain");
    let off = engine
        .query_tuned(&mother_query_stack(), &cue12(0), quorum(&engine, 0))
        .expect("off");
    assert_eq!(plain.response_hash, off.response_hash);
    assert_eq!(
        off.quorum.distinct_supporters, 0,
        "off reads zero supporters"
    );
    // Determinism with the gate armed.
    let a = engine
        .query_tuned(&mother_query_stack(), &cue12(0), quorum(&engine, 5))
        .expect("query");
    let b = engine
        .query_tuned(&mother_query_stack(), &cue12(0), quorum(&engine, 5))
        .expect("query");
    assert_eq!(a.response_hash, b.response_hash);
    assert_eq!(a.quorum, b.quorum);
}
