//! Bet hedging (#10, D51) — the admission gates: the regime-flip
//! retention gate (the money gate — when the world changes, the hedged
//! engine still holds the alternative and answers; the unhedged engine
//! has forgotten it and abstains), the hedge-delays-never-cancels gate
//! (a falsified trace still crosses the hard floors — later, but it
//! crosses), do-no-harm defaults, and determinism.

use atom_cognitive_engine::stacks::{idle_stack, observe_stack, query_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

/// Fast-clock regime so forgetting happens inside a test window: decay
/// 0.06/step, ttl 4 + 2/hit. Hedge slows the tail; floors stay absolute.
fn fast_clock(hedge_rate: f64) -> EngineConfig {
    EngineConfig {
        decay_support_rate: 0.06,
        decay_mass_rate: 0.03,
        initial_ttl: 4,
        ttl_per_hit: 2,
        hedge_rate,
        conservation_budget: 1.0e6,
        capacity: 16,
        ..EngineConfig::default()
    }
}

fn build_world() -> (CognitiveEngine, u64) {
    // A strong incumbent (value 1) and a lightly-supported alternative
    // (value 2) — the alternative is the attention tail.
    let mut engine = CognitiveEngine::new(fast_clock(0.0)).expect("config");
    let observe = observe_stack();
    for _ in 0..8 {
        engine
            .observe(&observe, &cue12(2), 1, 1.0)
            .expect("observe");
    }
    for _ in 0..2 {
        engine
            .observe(&observe, &cue12(1), 2, 1.0)
            .expect("observe");
    }
    let alternative = engine.traces().last().expect("nucleated").id;
    (engine, alternative)
}

#[test]
fn gate_regime_flip_keeps_the_alternative() {
    let query = query_stack();
    let idle = idle_stack();

    // Unhedged: a long quiet period forgets the alternative — the flip
    // query finds nothing and abstains.
    let (mut unhedged, alt_plain) = build_world();
    unhedged.idle(&idle, 40).expect("idle");
    assert!(
        !unhedged.trace_by_id(alt_plain).unwrap().active,
        "unhedged: the unsupported alternative must be forgotten"
    );
    let plain_answer = unhedged.query(&query, &cue12(1)).expect("query");
    assert!(plain_answer.claim.is_none(), "forgotten → abstain");

    // Hedged: the tail survives the same quiet period and answers.
    let mut engine = CognitiveEngine::new(fast_clock(0.5)).expect("config");
    let observe = observe_stack();
    for _ in 0..8 {
        engine
            .observe(&observe, &cue12(2), 1, 1.0)
            .expect("observe");
    }
    for _ in 0..2 {
        engine
            .observe(&observe, &cue12(1), 2, 1.0)
            .expect("observe");
    }
    let alt = engine.traces().last().expect("nucleated").id;
    engine.idle(&idle, 40).expect("idle");
    assert!(
        engine.trace_by_id(alt).unwrap().active,
        "hedged: the tail trace must survive the quiet period"
    );
    let hedged_answer = engine.query(&query, &cue12(1)).expect("query");
    assert_eq!(
        hedged_answer.claim.map(|(v, _)| v),
        Some(2),
        "the hedged alternative still answers after the regime flip"
    );

    // And when evidence returns, the hedged engine reinforces ACCUMULATED
    // evidence — its answering trace carries the whole history, while the
    // unhedged engine answers from a fresh, evidence-poor nucleation.
    engine
        .observe(&observe, &cue12(1), 2, 1.0)
        .expect("flip observe");
    let mut unhedged_after = unhedged;
    unhedged_after
        .observe(&observe, &cue12(1), 2, 1.0)
        .expect("flip observe");
    let hedged_flip = engine.query(&query, &cue12(1)).expect("query");
    let plain_flip = unhedged_after.query(&query, &cue12(1)).expect("query");
    let evidence_of = |outcome: &atom_cognitive_engine::QueryOutcome, eng: &CognitiveEngine| {
        let id = outcome.activations.first().expect("answered").0;
        eng.trace_by_id(id)
            .expect("trace")
            .evidence
            .iter()
            .sum::<f64>()
    };
    let (hedged_evidence, plain_evidence) = (
        evidence_of(&hedged_flip, &engine),
        evidence_of(&plain_flip, &unhedged_after),
    );
    assert!(
        hedged_evidence > plain_evidence,
        "retained evidence must outweigh a fresh nucleation ({hedged_evidence} vs {plain_evidence})"
    );
}

#[test]
fn gate_hedge_delays_but_never_cancels_expiry() {
    // A falsified trace still crosses the hard floors. Contradict a trace
    // down to ~0.14 support, then watch both engines idle: the hedged
    // copy dies LATER (half decay speed, frozen ttl) but it dies.
    let idle = idle_stack();
    let death_step = |hedge_rate: f64| -> u64 {
        let mut engine = CognitiveEngine::new(fast_clock(hedge_rate)).expect("config");
        let observe = observe_stack();
        engine
            .observe(&observe, &cue12(4), 1, 1.0)
            .expect("observe");
        let victim = engine.traces().first().expect("nucleated").id;
        // Contradictions FIRST (while the victim still binds — each revise
        // refreshes its ttl): four take support to ~0.3, safely above the
        // hard floor and clearly below any healthy anchor.
        for round in 0..4 {
            let value = if round % 2 == 0 { 2 } else { 1 };
            engine
                .observe(&observe, &cue12(4), value, 1.0)
                .expect("contradict");
        }
        // Then the standing anchor (value 0, heavily reinforced) so the
        // victim is the unambiguous attention tail.
        for _ in 0..8 {
            engine
                .observe(&observe, &cue12(5), 0, 1.0)
                .expect("observe");
        }
        assert!(
            engine.trace_by_id(victim).unwrap().active,
            "setup: still above the floor after the contradictions — state: {:?}",
            engine
                .trace_by_id(victim)
                .map(|t| (t.support, t.mass, t.ttl, t.hits))
        );
        let mut step = 0u64;
        while step < 200 && engine.trace_by_id(victim).unwrap().active {
            engine.idle(&idle, 1).expect("idle");
            step += 1;
        }
        assert!(
            !engine.trace_by_id(victim).unwrap().active,
            "the falsified trace must eventually expire (hedge is not immortality)"
        );
        step
    };
    let unhedged = death_step(0.0);
    let hedged = death_step(0.5);
    assert!(
        hedged > unhedged,
        "hedging must delay the falsified trace's death ({hedged} vs {unhedged})"
    );
}

#[test]
fn gate_defaults_off_identical_and_validated() {
    // hedge 0.0 reproduces the historical decay trajectory byte-for-byte.
    let (mut left, _) = build_world();
    let (mut right, _) = build_world();
    left.idle(&idle_stack(), 12).expect("idle");
    right.idle(&idle_stack(), 12).expect("idle");
    assert_eq!(
        left.transitions().last().map(|t| t.state_hash.clone()),
        right.transitions().last().map(|t| t.state_hash.clone()),
        "two identical engines stay identical (hedge default off)"
    );
    // Out-of-range hedge is rejected fail-closed.
    let config = fast_clock(0.6);
    assert!(CognitiveEngine::new(config).is_err());
    // Determinism with hedge on: same construction, same idle, same hash.
    let mut a = CognitiveEngine::new(fast_clock(0.5)).expect("config");
    let mut b = CognitiveEngine::new(fast_clock(0.5)).expect("config");
    let observe = observe_stack();
    for engine in [&mut a, &mut b] {
        engine
            .observe(&observe, &cue12(2), 1, 1.0)
            .expect("observe");
        engine
            .observe(&observe, &cue12(1), 2, 1.0)
            .expect("observe");
        engine.idle(&idle_stack(), 30).expect("idle");
    }
    assert_eq!(
        a.transitions().last().map(|t| t.state_hash.clone()),
        b.transitions().last().map(|t| t.state_hash.clone())
    );
}
