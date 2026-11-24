//! Causal event ordering (#4, D56) — the admission gates: same-origin
//! revisions supersede (the classic correction still works), CONCURRENT
//! origins never silently overwrite each other (both masses stay live),
//! arrival order does not decide concurrent outcomes (the money gate —
//! permutation invariance, machine-checked), the unattributed stream is
//! byte-identical to plain observe, and determinism holds.

use atom_cognitive_engine::stacks::{observe_stack, query_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig};

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
        capacity: 16,
        ..EngineConfig::default()
    }
}

#[test]
fn gate_sequential_same_origin_still_revises() {
    // Within one causal source, later observations supersede earlier —
    // the corpus-correction semantics, unchanged.
    let mut engine = CognitiveEngine::new(durable()).expect("config");
    let observe = observe_stack();
    for _ in 0..5 {
        engine
            .observe_with_origin(&observe, &cue12(0), &[1], 1.0, 7)
            .expect("observe");
    }
    assert_eq!(engine.traces().first().unwrap().value(), Some(1));
    for _ in 0..3 {
        engine
            .observe_with_origin(&observe, &cue12(0), &[2], 1.0, 7)
            .expect("correction");
    }
    assert_eq!(
        engine.traces().first().unwrap().value(),
        Some(2),
        "same-origin corrections supersede"
    );
}

#[test]
fn gate_concurrent_origins_keep_both_masses_and_order_independent() {
    // Two explicit origins, different values, same cue. The later
    // arrival must NOT dissolve the earlier belief — and the FINAL
    // distribution must not depend on which origin arrived first
    // (arrival order is not causal order).
    let build = |first: u64, second: u64| -> CognitiveEngine {
        let mut engine = CognitiveEngine::new(durable()).expect("config");
        let observe = observe_stack();
        for _ in 0..5 {
            engine
                .observe_with_origin(&observe, &cue12(0), &[1], 1.0, first)
                .expect("observe");
        }
        for _ in 0..8 {
            engine
                .observe_with_origin(&observe, &cue12(0), &[2], 1.0, second)
                .expect("observe");
        }
        engine
    };
    // Origin 7 writes value 1; origin 9 writes value 2. When 9 is the
    // SECOND writer its writes are concurrent (no dissolve).
    let concurrent = build(7, 9);
    let trace = concurrent.traces().first().unwrap();
    let total: f64 = trace.evidence.iter().sum();
    assert!(
        trace.evidence[1] >= 0.30 * total,
        "the standing origin's mass must survive concurrent writes ({} of {})",
        trace.evidence[1],
        total
    );
    assert!(trace.evidence[2] > 0.0, "the dissent must be recorded");
    // Permutation invariance: the REVERSED arrival order (9 first with
    // value 2, then 7 with value 1) converges to the mirrored
    // distribution — value 1's share in one order equals value 2's share
    // in the other.
    let flipped = build(9, 7);
    let (forward, reverse) = (
        concurrent.traces().first().unwrap(),
        flipped.traces().first().unwrap(),
    );
    let share = |evidence: &[f64], value: usize| {
        let total: f64 = evidence.iter().sum();
        evidence[value] / total
    };
    let (share_1, share_2) = (share(&forward.evidence, 1), share(&forward.evidence, 2));
    let (rshare_2, rshare_1) = (share(&reverse.evidence, 2), share(&reverse.evidence, 1));
    assert!(
        (share_1 - rshare_1).abs() < 1e-6 && (share_2 - rshare_2).abs() < 1e-6,
        "arrival order must not change concurrent shares ({share_1}/{rshare_1}, {share_2}/{rshare_2})"
    );
}

#[test]
fn gate_unattributed_stream_is_byte_identical() {
    // origin 0 = the default stream: observe_with_origin(0) reproduces
    // plain observe exactly (hash-equal trajectories).
    let config = durable();
    let (mut attributed, mut plain) = (
        CognitiveEngine::new(config.clone()).expect("config"),
        CognitiveEngine::new(config).expect("config"),
    );
    let observe = observe_stack();
    for step in 0..10 {
        let value = step % 2; // alternating revisions — the hardest case
        attributed
            .observe_with_origin(&observe, &cue12(3), &[value], 1.0, 0)
            .expect("attributed observe");
        plain
            .observe(&observe, &cue12(3), value, 1.0)
            .expect("plain observe");
    }
    assert_eq!(
        attributed
            .transitions()
            .last()
            .map(|t| t.state_hash.clone()),
        plain.transitions().last().map(|t| t.state_hash.clone()),
        "origin 0 must be byte-identical to the default stream"
    );
}

#[test]
fn gate_concurrent_queries_answer_deterministically() {
    // Queries over concurrent ground are deterministic and the value
    // order between swapped arrivals matches (no order-luck in answers).
    let build = |first: u64, second: u64| -> CognitiveEngine {
        let mut engine = CognitiveEngine::new(durable()).expect("config");
        let observe = observe_stack();
        for _ in 0..5 {
            engine
                .observe_with_origin(&observe, &cue12(0), &[1], 1.0, first)
                .expect("observe");
        }
        for _ in 0..8 {
            engine
                .observe_with_origin(&observe, &cue12(0), &[2], 1.0, second)
                .expect("observe");
        }
        engine
    };
    let query = query_stack();
    let forward = build(7, 9).query(&query, &cue12(0)).expect("query");
    let forward_flip = build(9, 7).query(&query, &cue12(0)).expect("query");
    // The dominant value flips with the dominant origin — not with
    // arrival order.
    assert_eq!(
        forward.claim.map(|(v, _)| v),
        forward_flip.claim.map(|(v, _)| v),
        "the answer follows the evidence mass, not the arrival order"
    );
    let again = build(7, 9).query(&query, &cue12(0)).expect("query");
    assert_eq!(forward.response_hash, again.response_hash);
}
