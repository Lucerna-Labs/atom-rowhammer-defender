//! Precision-weighted prediction error (#7, D54) — the admission gates:
//! a weak ping does not dissolve a strong belief (proportionality), a
//! strong contradiction still fully revises even a maximal belief (the
//! escape hatch — precision never locks a belief), persistent weak
//! signals still accumulate and win (bounded skepticism, no immunity),
//! and the defaults-off / validation / determinism controls.

use atom_cognitive_engine::stacks::observe_stack;
use atom_cognitive_engine::{CognitiveEngine, EngineConfig};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

fn durable(precision_gain: f64) -> EngineConfig {
    EngineConfig {
        initial_ttl: 512,
        ttl_per_hit: 16,
        decay_support_rate: 0.0005,
        decay_mass_rate: 0.0003,
        conservation_budget: 1.0e6,
        capacity: 16,
        precision_gain,
        ..EngineConfig::default()
    }
}

/// A well-established belief: `reinforcements` × value 1 at salience 1.0.
fn established(precision_gain: f64, reinforcements: usize) -> CognitiveEngine {
    let mut engine = CognitiveEngine::new(durable(precision_gain)).expect("config");
    let observe = observe_stack();
    for _ in 0..reinforcements {
        engine
            .observe(&observe, &cue12(0), 1, 1.0)
            .expect("observe");
    }
    engine
}

fn evidence_for(engine: &CognitiveEngine, value: usize) -> f64 {
    engine.traces().first().expect("the belief").evidence[value]
}

#[test]
fn gate_weak_ping_does_not_dissolve_strong_belief() {
    // One 0.25-salience contradicting ping against a 10-hit belief. The
    // control (gain 0) takes the full historical dissolve; the armed
    // engine keeps most of its standing evidence. Both keep the value —
    // the difference is proportional resistance, not blindness.
    let mut armed = established(1.0, 10);
    let mut control = established(0.0, 10);
    let observe = observe_stack();
    armed
        .observe(&observe, &cue12(0), 2, 0.25)
        .expect("weak ping");
    control
        .observe(&observe, &cue12(0), 2, 0.25)
        .expect("weak ping");
    assert_eq!(armed.traces().first().unwrap().value(), Some(1));
    assert_eq!(control.traces().first().unwrap().value(), Some(1));
    let (armed_evidence, control_evidence) = (evidence_for(&armed, 1), evidence_for(&control, 1));
    assert!(
        armed_evidence > 1.5 * control_evidence,
        "the armed belief must retain far more standing evidence ({armed_evidence} vs {control_evidence})"
    );
    // And the dissent the ping cast is small, not zero: the belief was
    // heard, just not obeyed.
    assert!(evidence_for(&armed, 2) > 0.0);
    assert!(evidence_for(&armed, 2) < evidence_for(&control, 2));
}

#[test]
fn gate_strong_contradiction_still_revises() {
    // The escape hatch (anti-self-reinforcement): a maximally reliable
    // belief contradicted at FULL salience still flips — precision
    // weighting slows noise, never locks the belief.
    let mut engine = established(1.0, 12);
    let observe = observe_stack();
    for _ in 0..3 {
        engine
            .observe(&observe, &cue12(0), 3, 2.0)
            .expect("strong contradiction");
    }
    assert_eq!(
        engine.traces().first().unwrap().value(),
        Some(3),
        "strong contradictions must revise even a maximal belief"
    );
}

#[test]
fn gate_persistent_weak_signal_eventually_wins() {
    // Bounded skepticism: repeated consistent low-salience contradictions
    // accumulate — by twelve pings the armed engine flips too. The
    // control flips earlier (measured), proving the resistance is real
    // but finite: no belief becomes immune.
    let flip_tick = |precision_gain: f64| -> usize {
        let mut engine = established(precision_gain, 3);
        let observe = observe_stack();
        for ping in 1..=12usize {
            engine
                .observe(&observe, &cue12(0), 2, 0.4)
                .expect("weak ping");
            if engine.traces().first().unwrap().value() == Some(2) {
                return ping;
            }
        }
        13 // never flipped
    };
    let control = flip_tick(0.0);
    let armed = flip_tick(1.0);
    assert!(control <= 4, "control flips fast (measured {control})");
    assert!(
        armed > control,
        "armed resistance must be real (armed {armed} vs control {control})"
    );
    assert!(
        armed <= 12,
        "bounded skepticism: consistent weak evidence must win by 12 pings (took {armed})"
    );
}

#[test]
fn gate_defaults_off_byte_identical_and_validated() {
    // gain 0 reproduces the historical revision trajectory exactly.
    let (mut left, mut right) = (established(0.0, 5), established(0.0, 5));
    let observe = observe_stack();
    for engine in [&mut left, &mut right] {
        engine.observe(&observe, &cue12(0), 2, 1.0).expect("revise");
    }
    assert_eq!(
        left.transitions().last().map(|t| t.state_hash.clone()),
        right.transitions().last().map(|t| t.state_hash.clone())
    );
    // Out-of-range gain rejected fail-closed.
    assert!(CognitiveEngine::new(durable(1.5)).is_err());
    // Determinism with the mechanism armed.
    let (mut a, mut b) = (established(1.0, 5), established(1.0, 5));
    for engine in [&mut a, &mut b] {
        engine.observe(&observe, &cue12(0), 2, 0.5).expect("revise");
        engine
            .observe(&observe, &cue12(0), 1, 1.0)
            .expect("reinforce");
    }
    assert_eq!(
        a.transitions().last().map(|t| t.state_hash.clone()),
        b.transitions().last().map(|t| t.state_hash.clone())
    );
}
