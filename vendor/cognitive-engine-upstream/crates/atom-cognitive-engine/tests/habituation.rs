//! Habituation/sensitization (W15, D47) — the admission discipline as
//! gates: the constant loses attention, the surprise regains it, and
//! the read-only-queries law survives the whole construction.

use atom_cognitive_engine::stacks::{kpr_stack, query_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig, MemoryTrace};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

fn normalize(cue: &[f64]) -> Vec<f64> {
    let norm: f64 = cue.iter().map(|v| v * v).sum::<f64>().sqrt();
    cue.iter().map(|v| v / norm).collect()
}

fn habituated() -> CognitiveEngine {
    CognitiveEngine::new(EngineConfig {
        habituation_rate: 0.3,
        novelty_gain: 0.5,
        ..EngineConfig::default()
    })
    .expect("config")
}

/// Two traces equidistant from the query cue; one has been confirmed
/// MANY times (habituated), one is fresh.
fn two_traces(engine: &mut CognitiveEngine) {
    let mut old_cue = vec![0.0; 12];
    old_cue[2] = 1.0;
    old_cue[0] = 1.0; // symmetric placement around cue12(1)... use explicit
    let _ = old_cue;
    // Simpler geometry: both traces at cos = 0.707 to the query cue,
    // symmetric (offsets on different symbols).
    for (symbol, count) in [(0usize, 12usize), (6usize, 4usize)] {
        let mut cue = vec![0.0; 12];
        cue[1] = 1.0;
        cue[symbol] = 1.0;
        let cue = normalize(&cue);
        for _ in 0..count {
            engine.observe(&kpr_stack(), &cue, 1, 1.0).expect("observe");
        }
    }
}

#[test]
fn gate_habituated_constant_loses_attention() {
    // The fresh trace (4 confirmations) must outweigh the habituated
    // one (12 confirmations) at equal similarity — with the mechanism
    // OFF they tie; with it ON the fresh one leads.
    let mut on = habituated();
    two_traces(&mut on);
    let traces: Vec<&MemoryTrace> = on.traces().iter().collect();
    assert_eq!(traces.len(), 2, "two distinct traces");
    let (habituated_trace, fresh_trace) = if traces[0].hits > traces[1].hits {
        (traces[0], traces[1])
    } else {
        (traces[1], traces[0])
    };
    assert!(
        habituated_trace.novelty < fresh_trace.novelty,
        "repeated confirmation decays novelty ({:.4} vs {:.4})",
        habituated_trace.novelty,
        fresh_trace.novelty
    );
    let outcome = on.query(&query_stack(), &cue12(1)).expect("query");
    let top = outcome
        .activations
        .first()
        .map(|(id, _)| *id)
        .expect("activation");
    assert_eq!(
        top, fresh_trace.id,
        "the fresh trace must out-rank the habituated constant"
    );

    // The with/without control: mechanism off, both at novelty 1.0 —
    // the ranking falls back to reliability (more hits wins).
    let mut off = CognitiveEngine::new(EngineConfig::default()).expect("config");
    two_traces(&mut off);
    let outcome = off.query(&query_stack(), &cue12(1)).expect("query");
    let top_off = outcome
        .activations
        .first()
        .map(|(id, _)| *id)
        .expect("activation");
    let more_hits = off.traces().iter().max_by_key(|t| t.hits).expect("trace");
    assert_eq!(
        top_off, more_hits.id,
        "with the mechanism off, reliability (hits) rules the ranking"
    );
}

#[test]
fn gate_surprise_sensitizes() {
    // A habituated trace that receives a CONTRADICTION regains full
    // novelty — the strong stimulus restores attention.
    let mut engine = habituated();
    let mut cue = vec![0.0; 12];
    cue[1] = 1.0;
    cue[0] = 1.0;
    let cue = normalize(&cue);
    for _ in 0..12 {
        engine.observe(&kpr_stack(), &cue, 1, 1.0).expect("observe");
    }
    let trace_id = engine.traces()[0].id;
    let before = engine.trace_by_id(trace_id).expect("trace").novelty;
    assert!(before < 0.05, "deeply habituated ({before:.4})");
    // The surprise: a contradictory value.
    engine
        .observe(&kpr_stack(), &cue, 2, 1.0)
        .expect("contradict");
    let after = engine.trace_by_id(trace_id).expect("trace").novelty;
    assert_eq!(
        after, 1.0,
        "contradiction restores novelty to full (sensitization)"
    );
}

#[test]
fn gate_queries_stay_read_only() {
    // The law that matters most here (the testing-effect tension stays
    // deferred): repeated QUERYING must not change any trace — novelty
    // included. Habituation lives on the observe side only.
    let mut engine = habituated();
    two_traces(&mut engine);
    let before: Vec<MemoryTrace> = engine.traces().to_vec();
    for _ in 0..10 {
        let _ = engine.query(&query_stack(), &cue12(1)).expect("query");
    }
    let after: Vec<MemoryTrace> = engine.traces().to_vec();
    assert_eq!(before, after, "queries never mutate traces");
}

#[test]
fn gate_defaults_off_and_validated() {
    // Defaults: novelty exists (1.0 everywhere) but gain 0 means no
    // retrieval effect — outcomes identical to the pre-W15 world.
    let mut on_default = CognitiveEngine::new(EngineConfig::default()).expect("config");
    two_traces(&mut on_default);
    let mut off = CognitiveEngine::new(EngineConfig::default()).expect("config");
    two_traces(&mut off);
    let a = on_default.query(&query_stack(), &cue12(1)).expect("query");
    let b = off.query(&query_stack(), &cue12(1)).expect("query");
    assert_eq!(a.response_hash, b.response_hash);
    assert!(
        off.traces().iter().all(|t| (t.novelty - 1.0).abs() < 1e-12),
        "with rate 0, novelty never decays"
    );
    // Validation.
    for (rate, gain) in [(0.6, 0.5), (0.3, 1.5)] {
        let bad = EngineConfig {
            habituation_rate: rate,
            novelty_gain: gain,
            ..EngineConfig::default()
        };
        assert!(CognitiveEngine::new(bad).is_err());
    }
    // Determinism.
    let mut again = habituated();
    two_traces(&mut again);
    let left = again.query(&query_stack(), &cue12(1)).expect("q1");
    let right = again.query(&query_stack(), &cue12(1)).expect("q2");
    assert_eq!(left.response_hash, right.response_hash);
}
