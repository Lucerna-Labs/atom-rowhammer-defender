//! Stigmergy (#14, D52) — the admission gates: bounded, hash-visible
//! deposition; the routing-not-evidence law (machine-checked: marking
//! queries leave every TRACE byte-identical, only bonds move); plain
//! queries never deposit (Stage-9); the coordination payoff (a marked
//! route strengthens its far reaches — measured); evaporation (no
//! permanent ruts); defaults-off and determinism.

use atom_cognitive_engine::stacks::{idle_stack, mother_query_stack, observe_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig, QueryTuning};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

/// A three-trace chain A—B—C in the durable regime: A eligible from
/// cue12(0), B and C reachable only through bond expansion.
fn chain() -> CognitiveEngine {
    let mut engine = CognitiveEngine::new(EngineConfig {
        initial_ttl: 512,
        ttl_per_hit: 16,
        decay_support_rate: 0.0005,
        decay_mass_rate: 0.0003,
        conservation_budget: 1.0e6,
        capacity: 16,
        ..EngineConfig::default()
    })
    .expect("config");
    let observe = observe_stack();
    for symbol in 0..3usize {
        for _ in 0..3 {
            engine
                .observe(&observe, &cue12(symbol), 1, 1.0)
                .expect("observe");
        }
    }
    let ids: Vec<u64> = engine.traces().iter().map(|t| t.id).collect();
    engine.add_bond(ids[0], ids[1]).expect("bond");
    engine.add_bond(ids[1], ids[2]).expect("bond");
    engine
}

fn marking(tuning_rate: f64, engine: &CognitiveEngine) -> QueryTuning {
    QueryTuning {
        stigmergy_rate: tuning_rate,
        ..QueryTuning::from_config(engine.config())
    }
}

fn gain_on(engine: &CognitiveEngine) -> QueryTuning {
    QueryTuning {
        stigmergy_gain: 1.0,
        ..QueryTuning::from_config(engine.config())
    }
}

#[test]
fn gate_deposition_bounded_and_hash_visible() {
    let mut engine = chain();
    let query = mother_query_stack();
    let traces_before = engine.traces().to_vec();
    let hash_before = engine.state_digest();
    let mut marked_total = 0usize;
    for _ in 0..12 {
        let result = engine
            .query_marking(&query, &cue12(0), marking(0.5, &engine))
            .expect("marking query");
        assert_eq!(result.edges_marked, 2, "both walked edges receive trail");
        marked_total += result.edges_marked;
    }
    assert_eq!(marked_total, 24);
    // The routing bias is VISIBLE: the state digest moved while every
    // trace stayed byte-identical — the movement is the pheromone.
    assert_eq!(engine.traces(), traces_before.as_slice());
    let hash_after = engine.state_digest();
    assert_ne!(
        hash_before, hash_after,
        "pheromone must appear in the state digest"
    );
}

#[test]
fn gate_routing_never_touches_evidence() {
    // The law, machine-checked: after 50 marking queries every TRACE is
    // byte-identical to a control engine that ran plain queries — only
    // the bonds (pheromone) may differ.
    let mut marking_engine = chain();
    let plain_engine = chain();
    let query = mother_query_stack();
    for _ in 0..50 {
        marking_engine
            .query_marking(&query, &cue12(0), marking(0.4, &marking_engine))
            .expect("marking query");
        plain_engine
            .query_tuned(&query, &cue12(0), marking(0.0, &plain_engine))
            .expect("plain query");
    }
    assert_eq!(
        marking_engine.traces(),
        plain_engine.traces(),
        "the routing-not-evidence law: marking queries may not change any trace"
    );
    assert_ne!(
        marking_engine.bond_count(),
        0,
        "setup: bonds exist (pheromone lives on them)"
    );
    // And the routing payoff is real: the far member (hop 2) weighs MORE
    // through the marked route than through the unmarked one.
    let marked_probe = marking_engine
        .query_tuned(&query, &cue12(0), gain_on(&marking_engine))
        .expect("probe");
    let plain_probe = plain_engine
        .query_tuned(&query, &cue12(0), gain_on(&plain_engine))
        .expect("probe");
    let far_id = plain_engine.traces().last().expect("chain end").id;
    let weight_of = |outcome: &atom_cognitive_engine::QueryOutcome| {
        outcome
            .activations
            .iter()
            .find(|(id, _)| *id == far_id)
            .map(|(_, weight)| *weight)
            .expect("far member reached")
    };
    let (marked_weight, plain_weight) = (weight_of(&marked_probe), weight_of(&plain_probe));
    assert!(
        marked_weight > plain_weight,
        "the marked route must strengthen its far reaches ({marked_weight} vs {plain_weight})"
    );
}

#[test]
fn gate_plain_queries_never_deposit() {
    // Stage-9 holds: plain queries never write, even with a deposit rate
    // armed in the tuning — query_tuned is read-only by construction
    // (&self), and the routing effect proves the trails stayed empty.
    let engine = chain();
    let query = mother_query_stack();
    for _ in 0..100 {
        engine
            .query_tuned(&query, &cue12(0), marking(0.9, &engine))
            .expect("plain query");
    }
    // No deposits: reading with gain must equal reading without gain.
    let with_gain = engine
        .query_tuned(&query, &cue12(0), gain_on(&engine))
        .expect("probe");
    let without_gain = engine.query(&query, &cue12(0)).expect("probe");
    assert_eq!(
        with_gain.response_hash, without_gain.response_hash,
        "unmarked trails must not change routing"
    );
}

#[test]
fn gate_evaporation_fades_trails() {
    // Trails decay with time on the same clock as memory: no permanent
    // ruts unless traffic renews them.
    let mut engine = chain();
    let query = mother_query_stack();
    for _ in 0..10 {
        engine
            .query_marking(&query, &cue12(0), marking(0.5, &engine))
            .expect("marking query");
    }
    let fresh_probe = engine
        .query_tuned(&query, &cue12(0), gain_on(&engine))
        .expect("probe");
    engine.idle(&idle_stack(), 60).expect("idle");
    let faded_probe = engine
        .query_tuned(&query, &cue12(0), gain_on(&engine))
        .expect("probe");
    let far_id = engine.traces().last().expect("chain end").id;
    let weight_of = |outcome: &atom_cognitive_engine::QueryOutcome| {
        outcome
            .activations
            .iter()
            .find(|(id, _)| *id == far_id)
            .map(|(_, weight)| *weight)
            .expect("far member reached")
    };
    let (fresh, faded) = (weight_of(&fresh_probe), weight_of(&faded_probe));
    assert!(
        faded < fresh,
        "evaporation must fade the trail's routing effect ({faded} vs {fresh})"
    );
}

#[test]
fn gate_defaults_off_bounded_and_deterministic() {
    // Rate 0 deposits nothing.
    let mut engine = chain();
    let query = mother_query_stack();
    let result = engine
        .query_marking(&query, &cue12(0), marking(0.0, &engine))
        .expect("marking query");
    assert_eq!(result.edges_marked, 0, "rate 0 = no work-trace");
    // Validation rejects out-of-range configs.
    let bad = EngineConfig {
        pheromone_decay: 1.0,
        ..EngineConfig::default()
    };
    assert!(CognitiveEngine::new(bad).is_err());
    // Determinism: identical marking programs leave identical states.
    let mut left = chain();
    let mut right = chain();
    for _ in 0..8 {
        left.query_marking(&query, &cue12(0), marking(0.5, &left))
            .expect("marking");
        right
            .query_marking(&query, &cue12(0), marking(0.5, &right))
            .expect("marking");
    }
    assert_eq!(
        left.transitions().last().map(|t| t.state_hash.clone()),
        right.transitions().last().map(|t| t.state_hash.clone())
    );
    // Boundedness through routing: after enough deposits, more marking
    // does not keep raising the far member's weight (cap 4.0): compare
    // the weight growth between 20 and 40 deposits.
    let weight_of = |engine: &CognitiveEngine| {
        engine
            .query_tuned(&query, &cue12(0), gain_on(engine))
            .expect("probe")
            .activations
            .iter()
            .find(|(id, _)| *id == engine.traces().last().unwrap().id)
            .map(|(_, w)| *w)
            .expect("far member")
    };
    let w20 = weight_of(&left);
    for _ in 0..20 {
        left.query_marking(&query, &cue12(0), marking(0.5, &left))
            .expect("marking");
    }
    let w40 = weight_of(&left);
    assert!(
        (w40 - w20).abs() < 1e-9,
        "pheromone is capped — routing saturates ({w20} vs {w40})"
    );
}
