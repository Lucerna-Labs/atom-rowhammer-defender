//! Kinetic proofreading (candidate #1, D35) — the full admission
//! discipline as gates: transient dissipation, staged promotion, the
//! contradiction reset, the matched-baseline comparison, the ablation,
//! the adversarial test, and zero new false assertions.

use atom_cognitive_engine::stacks::{kpr_stack, observe_stack, query_stack, record_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig, TraceTier};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

fn engine() -> CognitiveEngine {
    CognitiveEngine::new(EngineConfig::default()).expect("config valid")
}

#[test]
fn gate_transient_noise_never_asserts() {
    // A single low-salience observation (one touch) stays an uncommitted
    // Candidate: query-invisible, fast-decaying. The KPR law.
    let kpr = kpr_stack();
    let query = query_stack();
    let mut engine = engine();
    engine.observe(&kpr, &cue12(3), 1, 0.4).expect("observe");
    let trace = engine.active_traces()[0];
    assert_eq!(trace.tier, TraceTier::Candidate);
    let outcome = engine.query(&query, &cue12(3)).expect("query");
    assert_eq!(
        outcome.claim, None,
        "uncommitted evidence must never serve an assertion"
    );
    assert!(outcome.activations.is_empty());
}

#[test]
fn gate_persistence_promotes_and_asserts() {
    // The same signal, observed on distinct ticks, survives the stages
    // and earns commitment — selective retention.
    let kpr = kpr_stack();
    let query = query_stack();
    let mut engine = engine();
    for _ in 0..4 {
        engine.observe(&kpr, &cue12(3), 1, 1.0).expect("observe");
    }
    assert!(
        engine
            .active_traces()
            .iter()
            .any(|t| t.tier != TraceTier::Candidate),
        "a persistent contradiction-free signal must promote"
    );
    let outcome = engine.query(&query, &cue12(3)).expect("query");
    assert_eq!(outcome.claim.map(|(v, _)| v), Some(1));
}

#[test]
fn gate_contradiction_resets_accumulation() {
    // The KPR reset: a candidate that has accumulated stages and then
    // receives a contradiction loses its progress toward commitment.
    let kpr = kpr_stack();
    let mut engine = engine();
    // Two clean stages.
    engine.observe(&kpr, &cue12(3), 1, 1.0).expect("observe");
    engine.observe(&kpr, &cue12(3), 1, 1.0).expect("observe");
    let id = engine.active_traces()[0].id;
    let stages_before = engine.trace_by_id(id).expect("trace").stages;
    assert!(stages_before >= 2);
    // A contradicting value arrives: the write revises (contradiction
    // counted), the promote stage RESETS the stages.
    engine.observe(&kpr, &cue12(3), 2, 1.0).expect("contradict");
    let trace = engine.trace_by_id(id).expect("trace");
    assert_eq!(
        trace.stages, 0,
        "contradiction must dissipate accumulated stages (the KPR reset)"
    );
    assert_eq!(trace.tier, TraceTier::Candidate);
}

#[test]
fn gate_beats_the_matched_baseline() {
    // The existing-composition baseline: single-stage ingestion (the
    // record stack, i.e. what the engine could already do). Under the
    // baseline, one-touch noise becomes an assertable trace; under KPR
    // the same noise abstains. This is the "contributes something new"
    // proof.
    let query = query_stack();
    let mut baseline = engine();
    baseline
        .observe(&record_stack(), &cue12(5), 2, 0.4)
        .expect("baseline observe");
    let baseline_outcome = baseline.query(&query, &cue12(5)).expect("query");
    assert!(
        baseline_outcome.claim.is_some(),
        "the single-stage baseline commits one-touch noise"
    );
    let mut kpr_engine = engine();
    kpr_engine
        .observe(&kpr_stack(), &cue12(5), 2, 0.4)
        .expect("kpr observe");
    let kpr_outcome = kpr_engine.query(&query, &cue12(5)).expect("query");
    assert_eq!(
        kpr_outcome.claim, None,
        "KPR refuses what the baseline commits — the new contribution"
    );
}

#[test]
fn gate_ablation_promote_is_load_bearing() {
    // Ablation: the KPR stack minus the promote stage never commits
    // anything — every trace stays a Candidate forever.
    use atom_cognitive_core::roots::Root;
    use atom_cognitive_core::stages::{Stack, StageSpec};
    use atom_cognitive_engine::stacks::{
        OBSERVE_ATTEND, OBSERVE_CONSERVE, OBSERVE_DECAY, OBSERVE_GATE_CANDIDATE, OBSERVE_PREDICT,
        OBSERVE_VALIDATE, OBSERVE_VERIFY, OBSERVE_WRITE,
    };
    let ablated = Stack::assemble(vec![
        StageSpec::new(OBSERVE_VALIDATE, None, &[]),
        StageSpec::new(OBSERVE_PREDICT, Some(Root::Predict), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_DECAY, Some(Root::Decay), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_ATTEND, Some(Root::Attend), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_GATE_CANDIDATE, Some(Root::Gate), &[OBSERVE_ATTEND]),
        StageSpec::new(
            OBSERVE_WRITE,
            Some(Root::Bind),
            &[OBSERVE_GATE_CANDIDATE, OBSERVE_ATTEND, OBSERVE_PREDICT],
        ),
        StageSpec::new(OBSERVE_CONSERVE, Some(Root::Conserve), &[OBSERVE_WRITE]),
        StageSpec::new(OBSERVE_VERIFY, None, &[OBSERVE_CONSERVE]),
    ])
    .expect("ablated kpr stack assembles");
    let mut engine = engine();
    for _ in 0..6 {
        engine
            .observe(&ablated, &cue12(3), 1, 1.0)
            .expect("observe");
    }
    assert!(
        engine
            .active_traces()
            .iter()
            .all(|t| t.tier == TraceTier::Candidate),
        "without the promote stage nothing ever commits"
    );
    let outcome = engine.query(&query_stack(), &cue12(3)).expect("query");
    assert_eq!(outcome.claim, None);
}

#[test]
fn gate_adversarial_noise_storm_stays_silent() {
    // Adversarial: a storm of distinct one-touch noise cues — the classic
    // false-assertion generator — must produce zero assertions under KPR.
    let kpr = kpr_stack();
    let query = query_stack();
    let mut engine = engine();
    for symbol in 0..10 {
        engine
            .observe(&kpr, &cue12(symbol), symbol % 4, 0.3)
            .expect("noise");
    }
    for symbol in 0..10 {
        let outcome = engine.query(&query, &cue12(symbol)).expect("query");
        assert_eq!(
            outcome.claim, None,
            "noise storm symbol {symbol}: zero assertions"
        );
    }
    // And the canonical (non-KPR) observe cycle is unchanged: committed
    // evidence still flows normally alongside the candidate machinery.
    let mut plain = CognitiveEngine::new(EngineConfig::default()).expect("config");
    plain
        .observe(&observe_stack(), &cue12(3), 1, 1.0)
        .expect("observe");
    plain
        .observe(&observe_stack(), &cue12(3), 1, 1.0)
        .expect("observe");
    let outcome = plain.query(&query, &cue12(3)).expect("query");
    assert!(
        outcome.claim.is_some(),
        "no regression in the canonical path"
    );
}

#[test]
fn gate_same_tick_restagings_are_not_independent() {
    // The distinct-tick rule within one observe call is structural (one
    // observation = one tick); the enforceable version: kpr_min_stages
    // bounds how many DISTINCT observations commitment requires, and the
    // config validation rejects nonsense values.
    let config = EngineConfig {
        kpr_min_stages: 0,
        ..EngineConfig::default()
    };
    assert!(CognitiveEngine::new(config).is_err());
    let config = EngineConfig {
        kpr_min_stages: 99,
        ..EngineConfig::default()
    };
    assert!(CognitiveEngine::new(config).is_err());
    // A high bar is honest: 5 stages need 5 separate observations.
    let config = EngineConfig {
        kpr_min_stages: 5,
        ..EngineConfig::default()
    };
    let kpr = kpr_stack();
    let mut engine = CognitiveEngine::new(config).expect("config");
    for _ in 0..4 {
        engine.observe(&kpr, &cue12(3), 1, 1.0).expect("observe");
    }
    assert!(
        engine
            .active_traces()
            .iter()
            .all(|t| t.tier == TraceTier::Candidate),
        "four touches must not clear a five-stage bar"
    );
    engine.observe(&kpr, &cue12(3), 1, 1.0).expect("fifth");
    assert!(
        engine
            .active_traces()
            .iter()
            .any(|t| t.tier != TraceTier::Candidate),
        "the fifth distinct touch clears the bar"
    );
}
