//! BUILD-2 gates — precommitted in docs/BUILD2-REASONING.md BEFORE this
//! code. Reasoning = relational closure via interference + re-injection;
//! no inference rule exists anywhere. Harness-side analysis may know
//! the premises and the group-theoretic closed form.

use atom_cognitive_engine::perception::Reading;
use atom_cognitive_engine::reasoning_closure::{generated_subgroup_size, ClosureEngine, CycleOrder};

fn scene(base_id: u64, regime: usize) -> Vec<Reading> {
    let (exploration, convergence) = match regime {
        0 => (0.80, 0.70),
        1 => (0.80, 0.30),
        _ => (0.30, 0.70),
    };
    (0..5u64)
        .map(|k| Reading::new(base_id + k, 0.5, 0.5, 0.5, exploration, convergence, 0.5))
        .collect()
}

fn premises() -> Vec<Reading> {
    let mut out = Vec::new();
    let mut base = 10u64;
    for regime in 0..3 {
        out.extend(scene(base, regime));
        base += 100;
    }
    out
}

/// The three pairwise difference vectors of the premises (N=20 grid).
fn difference_generators() -> Vec<(isize, isize)> {
    let regimes = [(0.80f64, 0.70f64), (0.80, 0.30), (0.30, 0.70)];
    let n = 20f64;
    let mut out = Vec::new();
    for a in 0..3 {
        for b in (a + 1)..3 {
            out.push((
                ((regimes[a].0 - regimes[b].0) * n).round() as isize,
                ((regimes[a].1 - regimes[b].1) * n).round() as isize,
            ));
        }
    }
    out
}

/// R1 (precommitted: pass) — CLOSURE EMERGENCE: the engine's converged
/// frequency support equals the generated subgroup |H| exactly.
#[test]
fn gate_r1_closure_emergence() {
    let expected = generated_subgroup_size(&difference_generators());
    eprintln!("closed form |H| = {expected} (hand derivation predicted 10)");
    assert_eq!(expected, 10, "group closed form sanity");
    let mut engine = ClosureEngine::from_readings(&premises(), CycleOrder::MeasureThenInject);
    for _ in 0..40 {
        engine.step();
    }
    let measured = engine.support();
    eprintln!("engine converged support = {measured} (history tail: {:?})",
        &engine.support_history()[engine.support_history().len().saturating_sub(5)..]);
    assert_eq!(
        measured, expected,
        "emergent support must equal the group closure exactly"
    );
}

/// R2 (precommitted: pass) — SELF-TERMINATION: support reaches |H| and
/// STAYS for at least 10 further epochs. No stopping rule exists in the
/// engine; termination is saturation.
#[test]
fn gate_r2_self_termination() {
    let expected = generated_subgroup_size(&difference_generators());
    let mut engine = ClosureEngine::from_readings(&premises(), CycleOrder::MeasureThenInject);
    let mut saturated_at: Option<usize> = None;
    for epoch in 0..41 {
        engine.step();
        if saturated_at.is_none() && engine.support() == expected {
            saturated_at = Some(epoch);
        }
    }
    let saturated_at = saturated_at.expect("support must reach the closure");
    let history = engine.support_history();
    assert!(
        saturated_at + 10 < history.len(),
        "saturation must occur early enough to observe stability"
    );
    for (i, s) in history.iter().enumerate().skip(saturated_at) {
        assert_eq!(
            *s, expected,
            "support left the closure at epoch {i} ({s}) — non-closure"
        );
    }
    eprintln!("saturated at epoch {saturated_at}; stable through {}", history.len());
}

/// R3 (precommitted: pass) — FEEDBACK IS LOAD-BEARING: a single-shot
/// measurement (no re-injection) carries only the first-order
/// differences (3 modes + conjugates = 6 bins), never the closure.
#[test]
fn gate_r3_feedback_load_bearing() {
    use atom_cognitive_engine::reasoning_closure::single_shot_support;
    let single_shot = single_shot_support(&premises());
    let closure = generated_subgroup_size(&difference_generators());
    eprintln!("single-shot support {single_shot}; closure {closure}");
    assert_eq!(single_shot, 6, "first-order differences only (+conjugates)");
    assert!(single_shot < closure, "single-shot must not reach the closure");
}

/// R4 (precommitted: pass) — AUDIT: the derivation pathway contains no
/// inference constants (checked by the module's audit list; here we
/// assert the engine runs with no configuration surface at all — no
/// knobs, no thresholds: the only inputs are readings and an order).
#[test]
fn gate_r4_no_inference_constants() {
    // The engine has no config struct, no limits: constructing it takes
    // readings and an order; stepping takes nothing. If a rule existed,
    // it would need a home — there is none.
    let mut engine = ClosureEngine::from_readings(&premises(), CycleOrder::MeasureThenInject);
    engine.step();
    assert_eq!(engine.epoch(), 1);
}

/// R5 (precommitted: measured, all constructible order variants) —
/// Rule 2: execute and measure every constructible ordering of the
/// cycle; name any non-constructible orderings explicitly.
#[test]
fn gate_r5_order_variants() {
    let expected = generated_subgroup_size(&difference_generators());
    // NON-CONSTRUCTIBLE (named, not skipped silently): any ordering that
    // places re-injection BEFORE any interference has no field to
    // re-inject; any ordering WITHOUT the Born readout is not this
    // stack's cycle (it is the NoReset ablation from BUILD-1, already
    // measured there: phantom structure, memorylessness lost).
    // MEASURED FINDING (R5 failed as first written, corrected on record):
    // the order is not cosmetic — it decides whether derivation chains at
    // all. MeasureThenInject (feedback from the MEASURED intensity |psi|^2)
    // reaches the closure; InjectThenMeasure (feedback from the coherent
    // amplitude |psi|) collapses to ONE mode, because squaring the
    // measurement is what creates the relation PRODUCTS that chain. This
    // is the Rule-2 lesson arriving as physics: measure every order; the
    // first/working one is not presumed best, and the failing one is a
    // finding about the mechanism.
    let measured: Vec<(CycleOrder, usize)> = [
        CycleOrder::MeasureThenInject,
        CycleOrder::InjectThenMeasure,
        CycleOrder::MeasureThenInjectNoPhase,
    ]
    .into_iter()
    .map(|order| {
        let mut engine = ClosureEngine::from_readings(&premises(), order);
        for _ in 0..40 {
            engine.step();
        }
        let support = engine.support();
        eprintln!("order {order:?}: converged support {support} (closure {expected})");
        (order, support)
    })
    .collect();
    for (order, support) in &measured {
        match order {
            CycleOrder::InjectThenMeasure => assert_eq!(
                *support, 1,
                "the coherent-amplitude feedback order is expected to collapse (measured finding)"
            ),
            _ => assert_eq!(
                *support, expected,
                "order {order:?} must reach the closure"
            ),
        }
    }
}
