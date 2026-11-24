//! Pareto preservation (#6, D50) — the admission gates: brute-force
//! verified non-domination, tension NAMED instead of collapsed (the
//! reliability champion and the fresh contender both stand on the front),
//! pure-metadata (arming the front may not change any verdict or hash),
//! boundedness, and determinism.

use atom_cognitive_engine::stacks::{observe_stack, query_stack, record_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig, QueryTuning, cosine_similarity};

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
        surprise_boost: 1.0, // fair fixtures: no prediction boost asymmetry
        capacity: 32,
        ..EngineConfig::default()
    }
}

fn pareto_on(engine: &CognitiveEngine) -> QueryTuning {
    QueryTuning {
        pareto_front: true,
        ..QueryTuning::from_config(engine.config())
    }
}

/// A dominates B: ≥ on every axis, strictly > on at least one.
fn dominates(a: (f64, f64, f64), b: (f64, f64, f64)) -> bool {
    let (ar, asup, an) = a;
    let (br, bsup, bn) = b;
    ar >= br && asup >= bsup && an >= bn && (ar > br || asup > bsup || an > bn)
}

#[test]
fn gate_non_domination_verified_brute_force() {
    // A crowded facet with varied salience: domination structure is
    // non-trivial, and the front is verified against an independent
    // brute-force recomputation.
    let mut engine = CognitiveEngine::new(durable()).expect("config");
    let record = record_stack();
    for (i, salience) in [0.6, 1.0, 1.4, 2.0, 0.8].iter().enumerate() {
        let value = if i % 2 == 0 { 1 } else { 2 };
        engine
            .observe(&record, &cue12(0), value, *salience)
            .expect("observe");
    }
    let outcome = engine
        .query_tuned(&query_stack(), &cue12(0), pareto_on(&engine))
        .expect("query");
    assert!(!outcome.front.is_empty(), "a non-empty facet has a front");

    // Independent eligible set: active, above the gate, above the floor.
    let config = engine.config();
    let eligible: Vec<(u64, (f64, f64, f64))> = engine
        .active_traces()
        .into_iter()
        .filter(|t| {
            cosine_similarity(&cue12(0), &t.cue) >= config.query_threshold
                && t.reliability() >= config.min_reliability
        })
        .map(|t| (t.id, (t.reliability(), t.support, t.novelty)))
        .collect();
    assert_eq!(eligible.len(), outcome.eligible_count);

    for member in &outcome.front {
        let axes = (member.reliability, member.support, member.novelty);
        let dominated = eligible
            .iter()
            .any(|(id, other)| *id != member.trace_id && dominates(*other, axes));
        assert!(!dominated, "front member {} is dominated", member.trace_id);
    }
    let front_ids: Vec<u64> = outcome.front.iter().map(|m| m.trace_id).collect();
    for (id, axes) in &eligible {
        if front_ids.contains(id) {
            continue;
        }
        let dominated = eligible
            .iter()
            .any(|(other_id, other)| other_id != id && dominates(*other, *axes));
        assert!(
            dominated,
            "eligible trace {id} off the front must be dominated"
        );
    }
}

#[test]
fn gate_tension_is_named_not_collapsed() {
    // The champion (12 reinforcements, habituated, value 1) vs the fresh
    // contender (record-nucleated, value 2, full novelty). The champion
    // wins reliability; the contender wins support (salience boost on
    // its surprising birth) and novelty. Neither dominates — the front
    // names BOTH, where a single score would have collapsed the tension
    // into the champion alone.
    let mut engine = CognitiveEngine::new(EngineConfig {
        habituation_rate: 0.10,
        ..durable()
    })
    .expect("config");
    let observe = observe_stack();
    for _ in 0..12 {
        engine
            .observe(&observe, &cue12(0), 1, 1.0)
            .expect("observe");
    }
    let record = record_stack();
    engine
        .observe(&record, &cue12(0), 2, 1.0)
        .expect("contender observe");

    let outcome = engine
        .query_tuned(&query_stack(), &cue12(0), pareto_on(&engine))
        .expect("query");
    assert_eq!(
        outcome.front.len(),
        2,
        "champion and contender must both stand on the front"
    );
    let values: Vec<usize> = outcome.front.iter().map(|m| m.value).collect();
    assert!(values.contains(&1) && values.contains(&2));
    // The champion leads by reliability; the contender holds novelty 1.0.
    let champion = outcome.front.first().expect("sorted by reliability");
    assert_eq!(champion.value, 1);
    let contender = outcome.front.last().expect("two members");
    assert!((contender.novelty - 1.0).abs() < 1e-12);
}

#[test]
fn gate_front_is_pure_metadata() {
    let mut engine = CognitiveEngine::new(durable()).expect("config");
    let record = record_stack();
    for (i, salience) in [0.6, 1.0, 1.4, 2.0, 0.8].iter().enumerate() {
        let value = if i % 2 == 0 { 1 } else { 2 };
        engine
            .observe(&record, &cue12(0), value, *salience)
            .expect("observe");
    }
    let off = engine.query(&query_stack(), &cue12(0)).expect("query off");
    let on = engine
        .query_tuned(&query_stack(), &cue12(0), pareto_on(&engine))
        .expect("query on");
    assert!(off.front.is_empty(), "front is off by default");
    assert!(!on.front.is_empty());
    // Arming the front may not change ANY recorded answer.
    assert_eq!(off.claim, on.claim);
    assert_eq!(off.activations, on.activations);
    assert_eq!(
        off.response_hash, on.response_hash,
        "the front is excluded from the response hash"
    );
}

#[test]
fn gate_bounded_and_deterministic() {
    let mut engine = CognitiveEngine::new(durable()).expect("config");
    let record = record_stack();
    for (i, salience) in [0.6, 1.0, 1.4, 2.0, 0.8, 1.2, 0.9].iter().enumerate() {
        engine
            .observe(&record, &cue12(0), i % 3, *salience)
            .expect("observe");
    }
    let first = engine
        .query_tuned(&query_stack(), &cue12(0), pareto_on(&engine))
        .expect("query");
    let second = engine
        .query_tuned(&query_stack(), &cue12(0), pareto_on(&engine))
        .expect("query");
    assert!(
        first.front.len() <= first.eligible_count,
        "front is bounded"
    );
    assert_eq!(first.front, second.front, "front is deterministic");
}
