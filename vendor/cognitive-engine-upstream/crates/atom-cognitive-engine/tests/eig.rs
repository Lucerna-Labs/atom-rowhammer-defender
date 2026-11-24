//! Expected information gain (candidate #2, D39) — the admission
//! discipline as gates: unknown gets a plan, following the plan changes
//! the outcome, recommending never mutates, and nothing-informative is
//! an honest None.

use atom_cognitive_engine::stacks::{kpr_stack, query_stack};
use atom_cognitive_engine::{
    CognitiveEngine, EngineConfig, MemoryTrace, ObservationKind, QueryTuning, TraceTier,
};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

fn commit(engine: &mut CognitiveEngine, cue: &[f64], value: usize) {
    commit_at(engine, cue, value, 1.0);
}

/// Commit at an arbitrary salience: low salience commits a WEAK trace —
/// the reliability-blocked setup the EIG gates need (a fully reinforced
/// trace sits at reliability ≈ 0.97 and clears any sane floor).
fn commit_at(engine: &mut CognitiveEngine, cue: &[f64], value: usize, salience: f64) {
    for _ in 0..4 {
        engine
            .observe(&kpr_stack(), cue, value, salience)
            .expect("observe");
    }
}

fn normalize(cue: &[f64]) -> Vec<f64> {
    let norm: f64 = cue.iter().map(|v| v * v).sum::<f64>().sqrt();
    cue.iter().map(|v| v / norm).collect()
}

#[test]
fn gate_unknown_gets_a_reinforce_plan() {
    // Setup: a committed trace whose similarity clears the query gate but
    // whose reliability sits below the floor (min_reliability raised so a
    // fresh commit is blocked). The query abstains — and the engine
    // prescribes exactly that trace's cue.
    let config = EngineConfig {
        min_reliability: 0.75, // above a fresh commit's reliability
        ..EngineConfig::default()
    };
    let query = query_stack();
    let mut engine = CognitiveEngine::new(config).expect("config");
    commit_at(&mut engine, &cue12(0), 1, 0.5);
    let outcome = engine.query(&query, &cue12(0)).expect("query");
    assert_eq!(outcome.claim, None, "reliability-blocked: must abstain");

    let plans = engine.recommend_observation(&cue12(0)).expect("recommend");
    assert!(!plans.is_empty(), "unknown must come with a plan");
    let plan = &plans[0];
    assert_eq!(plan.kind, ObservationKind::Reinforce);
    let target = plan.target_trace.expect("names the blocked trace");
    let trace = engine.trace_by_id(target).expect("trace");
    assert!(
        cosine(&cue12(0), &trace.cue) >= 0.60,
        "the plan targets a gate-clearing trace"
    );
    assert!(
        !plan.reason.is_empty(),
        "plans carry human-readable reasons"
    );
}

#[test]
fn gate_following_the_plan_changes_the_outcome() {
    // THE money gate: information gain realized. The abstaining query
    // becomes an answering query after the recommended observations are
    // actually taken — the plan pays off through normal KPR mechanics.
    let config = EngineConfig {
        min_reliability: 0.75,
        ..EngineConfig::default()
    };
    let query = query_stack();
    let mut engine = CognitiveEngine::new(config).expect("config");
    commit_at(&mut engine, &cue12(0), 1, 0.5);
    assert_eq!(
        engine.query(&query, &cue12(0)).expect("query").claim,
        None,
        "starts unknown"
    );
    let plan = engine
        .recommend_observation(&cue12(0))
        .expect("recommend")
        .into_iter()
        .next()
        .expect("a plan exists");
    // Follow the plan: observe at the recommended probe cue.
    for _ in 0..4 {
        engine
            .observe(&kpr_stack(), &plan.probe_cue, 1, 1.0)
            .expect("observe");
    }
    let outcome = engine.query(&query, &cue12(0)).expect("query");
    assert_eq!(
        outcome.claim.map(|(v, _)| v),
        Some(1),
        "following the plan converts unknown into an answer"
    );
}

#[test]
fn gate_explore_plan_for_near_misses() {
    // A near miss (sim just below the gate) yields an Explore plan at the
    // query's own cue — and following it commits a pattern the query can
    // then reach.
    let query = query_stack();
    let mut engine = CognitiveEngine::new(EngineConfig::default()).expect("config");
    commit(&mut engine, &cue12(0), 1);
    // Query cue at ~0.64 similarity to the committed trace: inside the
    // 0.08 margin below the 0.68 default gate, far from eligible.
    let mut far = vec![0.0; 12];
    far[0] = 1.0;
    far[5] = 1.2; // cos = 1/sqrt(1+1.44) ≈ 0.640
    let far = normalize(&far);
    assert_eq!(
        engine.query(&query, &far).expect("query").claim,
        None,
        "near miss must abstain"
    );
    let plans = engine.recommend_observation(&far).expect("recommend");
    assert!(!plans.is_empty(), "a near miss earns an Explore plan");
    let plan = &plans[0];
    assert_eq!(plan.kind, ObservationKind::Explore);
    assert_eq!(plan.target_trace, None, "no committed trace is targeted");
    // Following it: the environment confirms a pattern at the probe cue.
    for _ in 0..4 {
        engine
            .observe(&kpr_stack(), &plan.probe_cue, 2, 1.0)
            .expect("observe");
    }
    let outcome = engine.query(&query, &far).expect("query");
    assert_eq!(
        outcome.claim.map(|(v, _)| v),
        Some(2),
        "the explored pattern becomes an answer"
    );
}

#[test]
fn gate_recommending_never_mutates() {
    // The read-only law: recommending changes no state whatsoever.
    let config = EngineConfig {
        min_reliability: 0.75,
        ..EngineConfig::default()
    };
    let mut engine = CognitiveEngine::new(config).expect("config");
    commit_at(&mut engine, &cue12(0), 1, 0.5);
    let before: Vec<MemoryTrace> = engine.traces().to_vec();
    let _ = engine.recommend_observation(&cue12(0)).expect("recommend");
    let _ = engine.recommend_observation(&cue12(0)).expect("again");
    assert_eq!(engine.traces(), before.as_slice(), "zero mutation");
}

#[test]
fn gate_nothing_informative_is_an_honest_none() {
    // Adversarial: a query in a region with NO committed structure near
    // enough (no near miss inside the margin) returns NO plan — the
    // engine never fabricates a useful-looking probe.
    let mut engine = CognitiveEngine::new(EngineConfig::default()).expect("config");
    commit_at(&mut engine, &cue12(0), 1, 0.5);
    // Orthogonal cue: sim 0 to everything — far outside the margin.
    let plans = engine.recommend_observation(&cue12(8)).expect("recommend");
    assert!(
        plans.is_empty(),
        "no nearby structure → no informative observation, honestly"
    );
}

#[test]
fn gate_plans_are_bounded_and_ranked() {
    // Bounded: at most 3 plans; ranked by descending score.
    let config = EngineConfig {
        min_reliability: 0.95, // block nearly everything
        ..EngineConfig::default()
    };
    let mut engine = CognitiveEngine::new(config).expect("config");
    for symbol in 0..5 {
        commit(&mut engine, &cue12(symbol), symbol % 4);
    }
    let plans = engine.recommend_observation(&cue12(0)).expect("recommend");
    assert!(plans.len() <= 3, "bounded plan list, got {}", plans.len());
    for pair in plans.windows(2) {
        assert!(
            pair[0].score >= pair[1].score,
            "plans must be ranked by score"
        );
    }
    // Config validation: nonsense knobs fail closed.
    let bad = EngineConfig {
        eig_margin: 2.0,
        ..EngineConfig::default()
    };
    assert!(CognitiveEngine::new(bad).is_err());
    let bad = EngineConfig {
        eig_min_score: 1.5,
        ..EngineConfig::default()
    };
    assert!(CognitiveEngine::new(bad).is_err());
}

#[test]
fn gate_plans_never_assert_directly() {
    // Zero-new-false-assertions: even with plans available, the query
    // outcome remains an abstention until observations are taken.
    let config = EngineConfig {
        min_reliability: 0.75,
        ..EngineConfig::default()
    };
    let query = query_stack();
    let mut engine = CognitiveEngine::new(config).expect("config");
    commit_at(&mut engine, &cue12(0), 1, 0.5);
    let _ = engine.recommend_observation(&cue12(0)).expect("recommend");
    assert_eq!(
        engine.query(&query, &cue12(0)).expect("query").claim,
        None,
        "a recommendation is not an assertion"
    );
    let _ = QueryTuning::from_config(engine.config());
    let _ = TraceTier::Candidate; // tier vocabulary still coherent
}

fn cosine(a: &[f64], b: &[f64]) -> f64 {
    atom_cognitive_engine::cosine_similarity(a, b)
}
