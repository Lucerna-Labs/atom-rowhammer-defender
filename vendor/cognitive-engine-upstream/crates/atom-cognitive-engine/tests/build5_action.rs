//! BUILD-5 gates — precommitted in docs/BUILD5-ACTION.md BEFORE this
//! code. Consequence, not observation, teaches the world's structure.

use atom_cognitive_engine::action_world::{
    expected_support, passive_support, ActionLoopEngine, ActionPolicy, World,
};

const W: (f64, f64) = (0.80, 0.30);
const EPOCHS: usize = 50;

/// S1 (precommitted: pass) — EXACT LEARNING: support == closed form
/// from the recorded actions, exactly.
#[test]
fn gate_s1_exact_learning() {
    let mut engine = ActionLoopEngine::new(World { wave: W }, 7);
    for _ in 0..EPOCHS {
        engine.epoch(ActionPolicy::FromField, true);
    }
    let measured = engine.support();
    let expected = expected_support(engine.actions(), W);
    eprintln!(
        "epochs {EPOCHS}, distinct actions {}, support {measured}, closed form {expected}",
        engine.actions().iter().map(|a| ((a.0 * 100.0) as i64, (a.1 * 100.0) as i64)).collect::<std::collections::HashSet<_>>().len()
    );
    assert_eq!(measured, expected, "support must equal the recorded-action closed form exactly");
}

/// S2 (precommitted: pass) — CONSEQUENCE VS OBSERVATION: 50 passive
/// presentations of the world's wave alone = structureless (DC only);
/// the action loop learns the world's relational structure.
#[test]
fn gate_s2_consequence_vs_observation() {
    // Passive arm (in-module ablation): present the world's wave alone.
    let passive = passive_support(W, EPOCHS);
    let mut engine = ActionLoopEngine::new(World { wave: W }, 7);
    for _ in 0..EPOCHS {
        engine.epoch(ActionPolicy::FromField, true);
    }
    eprintln!("passive support {passive} (structureless); action-loop support {}", engine.support());
    assert_eq!(passive, 1, "observing the world alone teaches nothing (DC only)");
    assert!(engine.support() > 50, "the action loop learns the world's structure");
}

/// S3 (precommitted: pass) — CONSEQUENCE IS LOAD-BEARING: actions
/// sampled and recorded, responses discarded → no learning.
#[test]
fn gate_s3_consequence_load_bearing() {
    let mut engine = ActionLoopEngine::new(World { wave: W }, 11);
    for _ in 0..EPOCHS {
        engine.epoch(ActionPolicy::FromField, false);
    }
    assert_eq!(engine.support(), 0, "no consequence, no learning");
    assert_eq!(engine.actions().len(), EPOCHS, "actions still recorded");
}

/// S5 (precommitted: measured) — the policy sweep: each policy's support
/// matches its OWN recorded-action closed form.
#[test]
fn gate_s5_policy_sweep() {
    for policy in [ActionPolicy::FromField, ActionPolicy::Uniform, ActionPolicy::Fixed] {
        let mut engine = ActionLoopEngine::new(World { wave: W }, 23);
        for _ in 0..EPOCHS {
            engine.epoch(policy, true);
        }
        let measured = engine.support();
        let expected = expected_support(engine.actions(), W);
        eprintln!("policy {policy:?}: support {measured}, closed form {expected}");
        assert_eq!(measured, expected, "policy {policy:?} must match its own closed form");
    }
}
