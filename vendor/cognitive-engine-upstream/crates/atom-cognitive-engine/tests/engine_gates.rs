//! Engine gates, root ablations, and ordering permutations.
//!
//! Every test names the root or ordering law it exercises. A root whose
//! ablation breaks no gate here is demoted at the next ROOTS.md revision
//! (decision D9).

use atom_cognitive_core::stages::Stack;
use atom_cognitive_engine::stacks::{
    self, misordered_decay_after_write, observe_stack, omit, query_stack, ungated_query_stack,
};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig};

/// Deterministic cue: one-hot symbol (dims 0..8) + one-hot context
/// (dims 9..12). cue_dim defaults to 12.
fn cue(symbol: usize, context: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 8] = 1.0;
    cue[8 + context % 4] = 1.0;
    cue
}

fn engine() -> CognitiveEngine {
    CognitiveEngine::new(EngineConfig::default()).expect("default config valid")
}

// ---- canonical gates ----------------------------------------------------

#[test]
fn gate_learns_associations() {
    let observe = observe_stack();
    let query = query_stack();
    let mut engine = engine();
    for value in 0..3 {
        for _ in 0..3 {
            engine
                .observe(&observe, &cue(value, 0), value, 1.0)
                .expect("observe");
        }
    }
    for value in 0..3 {
        let outcome = engine.query(&query, &cue(value, 0)).expect("query");
        assert_eq!(
            outcome.claim.map(|(v, _)| v),
            Some(value),
            "association {value} must be learned"
        );
    }
}

#[test]
fn gate_revises_contradictions_via_surprise_boost() {
    let observe = observe_stack();
    let query = query_stack();
    let mut engine = engine();
    for _ in 0..5 {
        engine
            .observe(&observe, &cue(0, 0), 0, 1.0)
            .expect("observe");
    }
    // Contradiction arrives at LOW salience; only the PREDICT root's
    // surprise boost lets three low-salience revisions overturn five
    // strong observations.
    for _ in 0..3 {
        engine
            .observe(&observe, &cue(0, 0), 1, 0.25)
            .expect("observe");
    }
    let outcome = engine.query(&query, &cue(0, 0)).expect("query");
    assert_eq!(outcome.claim.map(|(v, _)| v), Some(1), "revision must win");
    assert_eq!(engine.action_count("revise"), 3);
}

#[test]
fn gate_forgets_unsupported_noise() {
    let observe = observe_stack();
    let idle = stacks::idle_stack();
    let mut engine = engine();
    engine
        .observe(&observe, &cue(4, 1), 2, 0.25)
        .expect("observe");
    assert_eq!(engine.active_traces().len(), 1);
    engine.idle(&idle, 9).expect("idle");
    assert_eq!(
        engine.active_traces().len(),
        0,
        "a single low-salience trace must be forgotten by ttl expiry"
    );
}

#[test]
fn gate_conservation_holds() {
    let config = EngineConfig {
        capacity: 4,
        conservation_budget: 4.0,
        ..EngineConfig::default()
    };
    let observe = observe_stack();
    let mut engine = CognitiveEngine::new(config).expect("config valid");
    for symbol in 0..6 {
        engine
            .observe(&observe, &cue(symbol, 0), symbol % 4, 2.0)
            .expect("observe");
    }
    assert!(
        engine.total_mass() <= 4.0 + 1e-9,
        "conservation budget must hold after writes, got {}",
        engine.total_mass()
    );
}

#[test]
fn gate_abstains_on_unknown_cue() {
    let observe = observe_stack();
    let query = query_stack();
    let mut engine = engine();
    engine
        .observe(&observe, &cue(0, 0), 0, 1.0)
        .expect("observe");
    // Orthogonal cue: similarity 0 against everything.
    let outcome = engine.query(&query, &cue(7, 3)).expect("query");
    assert_eq!(outcome.claim, None, "unknown cue must abstain");
    assert_eq!(outcome.eligible_count, 0);
}

#[test]
fn gate_determinism_two_engines_same_hashes() {
    let observe = observe_stack();
    let query = query_stack();
    let run = |engine: &mut CognitiveEngine| {
        for round in 0..4 {
            for _ in 0..3 {
                engine
                    .observe(&observe, &cue(round % 4, round % 2), round % 4, 1.0)
                    .expect("observe");
            }
        }
        engine
            .observe(&observe, &cue(0, 0), 1, 0.25)
            .expect("observe");
        let outcome = engine.query(&query, &cue(0, 0)).expect("query");
        (
            engine
                .transitions()
                .last()
                .expect("transition")
                .state_hash
                .clone(),
            outcome.response_hash,
        )
    };
    let mut a = engine();
    let mut b = engine();
    assert_eq!(
        run(&mut a),
        run(&mut b),
        "identical programs must replay identically"
    );
}

#[test]
fn gate_explore_cools_monotonically_to_floor() {
    let observe = observe_stack();
    let mut engine = engine();
    let mut previous = engine.explore_temperature();
    for symbol in 0..60 {
        engine
            .observe(&observe, &cue(symbol % 8, symbol % 4), symbol % 4, 1.0)
            .expect("observe");
        let current = engine.explore_temperature();
        assert!(
            current <= previous + 1e-12,
            "temperature must cool monotonically"
        );
        previous = current;
    }
    assert!(previous <= engine.config().explore_floor + 1e-9);
}

// ---- root ablations (D9: a root with no failing ablation is demoted) ----

#[test]
fn ablate_predict_revision_fails() {
    // The canonical stack makes OBSERVE_WRITE require OBSERVE_PREDICT (the
    // surprise linkage), so omitting predict would remove learning outright.
    // To isolate the boost, this ablation rebuilds the observe stack with
    // write consuming only gate+attend — legal, executable, and free of the
    // prediction surprise boost.
    use atom_cognitive_core::roots::Root;
    use atom_cognitive_core::stages::{Stack, StageSpec};
    use atom_cognitive_engine::stacks::{
        OBSERVE_ATTEND, OBSERVE_CONSERVE, OBSERVE_DECAY, OBSERVE_GATE, OBSERVE_VALIDATE,
        OBSERVE_VERIFY, OBSERVE_WRITE,
    };
    let ablated = Stack::assemble(vec![
        StageSpec::new(OBSERVE_VALIDATE, None, &[]),
        StageSpec::new(OBSERVE_DECAY, Some(Root::Decay), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_ATTEND, Some(Root::Attend), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_GATE, Some(Root::Gate), &[OBSERVE_ATTEND]),
        StageSpec::new(
            OBSERVE_WRITE,
            Some(Root::Bind),
            &[OBSERVE_GATE, OBSERVE_ATTEND],
        ),
        StageSpec::new(OBSERVE_CONSERVE, Some(Root::Conserve), &[OBSERVE_WRITE]),
        StageSpec::new(OBSERVE_VERIFY, None, &[OBSERVE_CONSERVE]),
    ])
    .expect("boost-free ablation stack assembles");
    let full = observe_stack();
    let query = query_stack();
    let run = |stack: &Stack| {
        let mut engine = engine();
        for _ in 0..5 {
            engine.observe(stack, &cue(0, 0), 0, 1.0).expect("observe");
        }
        for _ in 0..3 {
            engine.observe(stack, &cue(0, 0), 1, 0.25).expect("observe");
        }
        engine
            .query(&query, &cue(0, 0))
            .expect("query")
            .claim
            .map(|(v, _)| v)
    };
    assert_eq!(run(&full), Some(1), "canonical revises");
    assert_eq!(
        run(&ablated),
        Some(0),
        "without PREDICT the surprise boost is gone and the low-salience \
         contradiction fails to overturn the original learning"
    );
}

#[test]
fn ablate_decay_noise_persists() {
    let full = stacks::idle_stack();
    let ablated = omit(&full, "idle.decay");
    let observe = observe_stack();
    let check = |idle: &Stack| {
        let mut engine = engine();
        engine
            .observe(&observe, &cue(4, 1), 2, 0.25)
            .expect("observe");
        engine.idle(idle, 9).expect("idle");
        engine.active_traces().len()
    };
    assert_eq!(check(&full), 0, "canonical forgets the noise trace");
    assert_eq!(
        check(&ablated),
        1,
        "without DECAY the unsupported trace persists — the root is load-bearing"
    );
}

#[test]
fn ablate_conserve_budget_violated() {
    let config = EngineConfig {
        capacity: 4,
        conservation_budget: 4.0,
        ..EngineConfig::default()
    };
    let full = observe_stack();
    let ablated = omit(&full, stacks::OBSERVE_CONSERVE);
    let run = |stack: &Stack| {
        let mut engine = CognitiveEngine::new(config.clone()).expect("config valid");
        for symbol in 0..6 {
            engine
                .observe(stack, &cue(symbol, 0), symbol % 4, 2.0)
                .expect("observe");
        }
        engine.total_mass()
    };
    assert!(run(&full) <= 4.0 + 1e-9);
    assert!(
        run(&ablated) > 4.0,
        "without CONSERVE the budget is violated — the root is load-bearing"
    );
}

#[test]
fn ablate_query_gate_unreliable_trace_asserted() {
    let config = EngineConfig {
        min_reliability: 0.30, // a fresh low-salience trace is unreliable
        ..EngineConfig::default()
    };
    let observe = observe_stack();
    let gated = query_stack();
    let ungated = ungated_query_stack();
    let mut engine = CognitiveEngine::new(config).expect("config valid");
    // One low-salience trace whose cue exactly matches the query.
    engine
        .observe(&observe, &cue(2, 2), 3, 0.25)
        .expect("observe");
    let gated_outcome = engine.query(&gated, &cue(2, 2)).expect("query");
    assert_eq!(
        gated_outcome.claim, None,
        "the gate must exclude the unreliable trace and the engine must abstain"
    );
    let ungated_outcome = engine.query(&ungated, &cue(2, 2)).expect("query");
    assert_eq!(
        ungated_outcome.claim.map(|(v, _)| v),
        Some(3),
        "without the gate (O7 failure mode) attention asserts from the \
         unreliable trace — the gate is load-bearing"
    );
}

#[test]
fn ablate_write_nothing_persists() {
    let full = observe_stack();
    let ablated = omit(&full, stacks::OBSERVE_WRITE);
    let query = query_stack();
    let run = |stack: &Stack| {
        let mut engine = engine();
        for _ in 0..4 {
            engine.observe(stack, &cue(0, 0), 0, 1.0).expect("observe");
        }
        (
            engine.active_traces().len(),
            engine.query(&query, &cue(0, 0)).expect("query").claim,
        )
    };
    let (count, claim) = run(&full);
    assert_eq!((count, claim.map(|(v, _)| v)), (1, Some(0)));
    let (count, claim) = run(&ablated);
    assert_eq!(
        (count, claim),
        (0, None),
        "without BIND nothing persists — the root is load-bearing"
    );
}

// ---- ordering permutations (behavioral layer, ORDERING.md) ---------------

#[test]
fn permutation_decay_after_write_measurably_changes_state() {
    // Law O1: time must pass before learning. The mis-ordered variant is
    // legal to assemble; the reinforced trace is decayed on every write
    // tick, so its ttl and support end strictly below the canonical ones.
    let canonical = observe_stack();
    let misordered = misordered_decay_after_write();
    let run = |stack: &Stack| {
        let mut engine = engine();
        for _ in 0..15 {
            engine.observe(stack, &cue(0, 0), 0, 1.0).expect("observe");
        }
        let trace = engine.active_traces()[0];
        (trace.ttl, trace.support, trace.mass)
    };
    let (canonical_ttl, canonical_support, canonical_mass) = run(&canonical);
    let (mis_ttl, mis_support, mis_mass) = run(&misordered);
    assert!(
        canonical_ttl > mis_ttl,
        "canonical ttl {canonical_ttl} must exceed mis-ordered {mis_ttl}"
    );
    assert!(
        canonical_support > mis_support,
        "canonical support {canonical_support} must exceed mis-ordered {mis_support}"
    );
    assert!(canonical_mass > mis_mass);
}

#[test]
fn permutation_misordered_stack_is_executed_in_its_declared_order() {
    // The engine is genuinely stage-order-driven: the transition log of the
    // mis-ordered run must differ from the canonical run on the same program.
    let canonical = observe_stack();
    let misordered = misordered_decay_after_write();
    let run = |stack: &Stack| {
        let mut engine = engine();
        for round in 0..6 {
            engine
                .observe(stack, &cue(round % 4, round % 2), round % 4, 1.0)
                .expect("observe");
        }
        engine
            .transitions()
            .iter()
            .map(|t| t.state_hash.clone())
            .collect::<Vec<_>>()
    };
    assert_ne!(run(&canonical), run(&misordered));
}

#[test]
fn permutation_cannot_assemble_write_before_its_prerequisites() {
    use atom_cognitive_core::roots::Root;
    use atom_cognitive_core::stages::{Stack as S, StackError, StageSpec};
    // Layer-1 enforcement: write before gate is a construction error.
    let stages = vec![
        StageSpec::new("observe.validate", None, &[]),
        StageSpec::new("observe.attend", Some(Root::Attend), &["observe.validate"]),
        StageSpec::new(
            "observe.write",
            Some(Root::Bind),
            &["observe.attend", "observe.gate"],
        ),
        StageSpec::new("observe.gate", Some(Root::Gate), &["observe.attend"]),
    ];
    assert!(matches!(
        S::assemble(stages),
        Err(StackError::UnsatisfiedPrerequisite { .. })
    ));
}
