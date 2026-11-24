//! BUILD-7 — selection in processing: the honest composite measurement
//! (fourth of the four gaps). Measured FIRST in Python; this test
//! asserts the measured NEGATIVE explicitly so a future positive
//! (concentration actually emerging) trips it.
//!
//! VERDICT: NOT EMERGENT at this scale. The consequence loop learns the
//! world's structure (BUILD-5, exact) but the action distribution does
//! not concentrate: mean |a - W| drifts 0.453 -> 0.429 against a uniform
//! baseline of 0.499, distinct actions unchanged. Consequence-learning
//! accumulates structure but does not RANK it; ranking at the action
//! level would need the preference dynamics (BUILD-4) applied to the
//! accumulated structure — named, not built.

use atom_cognitive_engine::action_world::{ActionLoopEngine, ActionPolicy, World};

const W: (f64, f64) = (0.80, 0.30);

#[test]
fn gate_selection_measured_negative() {
    let mut engine = ActionLoopEngine::new(World { wave: W }, 7);
    for _ in 0..100 {
        engine.epoch(ActionPolicy::FromField, true);
    }
    let actions = engine.actions();
    let distance = |a: &(f64, f64)| ((a.0 - W.0).powi(2) + (a.1 - W.1).powi(2)).sqrt();
    let first: f64 = actions[..33].iter().map(distance).sum::<f64>() / 33.0;
    let last: f64 = actions[67..].iter().map(distance).sum::<f64>() / 33.0;
    let baseline = 0.499; // mean distance to W over the uniform 20x20 grid
    eprintln!("mean |a-W|: first third {first:.3}, last third {last:.3}, uniform baseline {baseline:.3}");
    // The measured negative: no halving (the positive-detection bar).
    assert!(
        last > first * 0.5,
        "selection concentrated ({first:.3} -> {last:.3}) — the negative changed; revisit BUILD-7"
    );
    // And the drift stays modest relative to the baseline.
    assert!(
        (last - baseline).abs() < (first - baseline).abs() + 0.1,
        "no meaningful approach to the world's neighbourhood"
    );
}
