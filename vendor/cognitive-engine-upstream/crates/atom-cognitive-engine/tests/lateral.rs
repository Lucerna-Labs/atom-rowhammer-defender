//! Lateral inhibition (#13, D58) — the admission gates: the redundant
//! chorus around the winner fades and distinct contributors GAIN share
//! (specialization), mutually-distinct members are untouched (no
//! artificial differences manufactured), a same-value chorus never
//! changes the WINNER (suppression reshapes echo, not elections), and
//! the defaults-off / determinism controls.

use atom_cognitive_engine::stacks::{observe_stack, query_stack, record_stack};
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

fn lateral(engine: &CognitiveEngine, gain: f64) -> QueryTuning {
    QueryTuning {
        lateral_inhibition: gain,
        ..QueryTuning::from_config(engine.config())
    }
}

/// A chorus (five identical-cue copies voting value 1, salience 2.0 so
/// they hold the top reliability) plus one DISTINCT contributor at a
/// nearby cue voting value 2. A blend query cue between them makes both
/// sides eligible.
fn chorus_world() -> CognitiveEngine {
    let mut engine = CognitiveEngine::new(durable()).expect("config");
    let record = record_stack();
    for _ in 0..5 {
        engine
            .observe(&record, &cue12(0), 1, 2.0)
            .expect("chorus observe");
    }
    let observe = observe_stack();
    for _ in 0..2 {
        engine
            .observe(&observe, &cue12(1), 2, 1.0)
            .expect("distinct observe");
    }
    engine
}

fn blend_query() -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[0] = 1.0;
    cue[1] = 1.0;
    // normalized by validate: cosine_similarity normalizes internally
    cue
}

fn share_of(engine: &CognitiveEngine, gain: f64, id: u64) -> f64 {
    engine
        .query_tuned(&query_stack(), &blend_query(), lateral(engine, gain))
        .expect("query")
        .activations
        .iter()
        .find(|(trace_id, _)| *trace_id == id)
        .map(|(_, weight)| *weight)
        .unwrap_or(0.0)
}

#[test]
fn gate_chorus_fades_distinct_rises() {
    let engine = chorus_world();
    let distinct_id = engine.traces().last().expect("distinct").id;
    let control = share_of(&engine, 0.0, distinct_id);
    let armed = share_of(&engine, 0.8, distinct_id);
    assert!(
        armed > control * 1.5,
        "the distinct contributor must gain share as the chorus fades ({armed} vs {control})"
    );
}

#[test]
fn gate_same_value_chorus_keeps_its_winner() {
    // Suppression reshapes the ECHO, not the ELECTION: a chorus voting
    // one value stays that value (no artificial difference manufactured
    // among agreeing members).
    let engine = chorus_world();
    let control = engine
        .query_tuned(&query_stack(), &blend_query(), lateral(&engine, 0.0))
        .expect("query");
    let armed = engine
        .query_tuned(&query_stack(), &blend_query(), lateral(&engine, 0.9))
        .expect("query");
    // The chorus outnumbers the distinct member heavily; the armed
    // answer must not flip to the distinct member's value.
    assert_eq!(
        control.claim.map(|(v, _)| v),
        armed.claim.map(|(v, _)| v),
        "suppression must not elect the suppressed side's rival"
    );
}

#[test]
fn gate_distinct_members_untouched() {
    // Mutually dissimilar members: quadratic suppression leaves their
    // weights at the control values (renormalized — nothing to
    // redistribute when nothing is suppressed).
    let mut engine = CognitiveEngine::new(durable()).expect("config");
    let record = record_stack();
    for symbol in [6usize, 7, 8, 9] {
        engine
            .observe(&record, &cue12(symbol), 1, 1.0)
            .expect("observe");
    }
    // A query cue near symbol 6 makes it the winner; others are at
    // similarity 0 to it — untouched by construction.
    let control = engine
        .query_tuned(&query_stack(), &cue12(6), lateral(&engine, 0.0))
        .expect("query");
    let armed = engine
        .query_tuned(&query_stack(), &cue12(6), lateral(&engine, 1.0))
        .expect("query");
    let weight_of = |outcome: &atom_cognitive_engine::QueryOutcome, id: u64| {
        outcome
            .activations
            .iter()
            .find(|(tid, _)| *tid == id)
            .map(|(_, w)| *w)
            .unwrap_or(0.0)
    };
    for trace in engine.traces() {
        let (a, b) = (weight_of(&control, trace.id), weight_of(&armed, trace.id));
        assert!(
            (a - b).abs() < 1e-9,
            "distinct members must be untouched ({a} vs {b})"
        );
    }
}

#[test]
fn gate_defaults_off_determinism() {
    // gain 0: identical to the plain query.
    let engine = chorus_world();
    let plain = engine.query(&query_stack(), &blend_query()).expect("query");
    let off = engine
        .query_tuned(&query_stack(), &blend_query(), lateral(&engine, 0.0))
        .expect("query");
    assert_eq!(plain.response_hash, off.response_hash);
    // Determinism: same armed query twice.
    let a = engine
        .query_tuned(&query_stack(), &blend_query(), lateral(&engine, 0.8))
        .expect("query");
    let b = engine
        .query_tuned(&query_stack(), &blend_query(), lateral(&engine, 0.8))
        .expect("query");
    assert_eq!(a.response_hash, b.response_hash);
}
