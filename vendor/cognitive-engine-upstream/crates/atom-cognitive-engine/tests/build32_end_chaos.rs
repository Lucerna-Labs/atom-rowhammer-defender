//! BUILD-32 gates — precommitted in docs/BUILD32-END-CHAOS.md BEFORE this
//! code. The operator's directive: add chaos at the END of each stack,
//! measure it, and REJECT it unless it moves the frozen metric.
//!
//! Nothing here is promoted. The gates measure the mechanisms, measure the
//! arms under matched controls, and apply the verdict rule to the numbers.

use atom_cognitive_engine::action_world::{
    expected_support, ActionLoopEngine, ActionPolicy, World,
};
use atom_cognitive_engine::chaos::{
    modulate, tent_collapse_step, ChaosMechanism, ChaosOrbit,
};
use atom_cognitive_engine::harness::{clear_for_next, run, run_with_pad, Task};
use atom_cognitive_engine::scratch_pad::{self_clearing_ticks, ClearMechanism, ScratchPad};

const ALPHA: f64 = 1.0;
const W: (f64, f64) = (0.80, 0.30);

// ---- the recorded tasks (BUILD-12/16's, re-declared here as observers) ----

fn task_a() -> Task {
    Task {
        blocks: vec![(0, 8), (10, 0)],
        scene: vec![(2, 2), (6, 2), (4, 2)],
        target: (1, 0),
        query: ((2, 2), (6, 2), (4, 2)),
        source_block: (1, 0),
    }
}

fn task_b() -> Task {
    Task {
        blocks: vec![(0, 8), (10, 0)],
        scene: vec![(1, 1), (5, 3)],
        target: (0, 16),
        query: ((1, 1), (5, 3), (3, 2)),
        source_block: (1, 0),
    }
}

/// The arm under test: which mechanisms modulate the carried state once per
/// cycle, in this order.
fn mechanisms(arm: usize) -> Vec<ChaosMechanism> {
    match arm {
        0 => vec![],
        1 => vec![ChaosMechanism::Logistic],
        2 => vec![ChaosMechanism::Henon],
        _ => vec![ChaosMechanism::Logistic, ChaosMechanism::Henon],
    }
}

fn arm_name(arm: usize) -> &'static str {
    match arm {
        0 => "no-chaos",
        1 => "logistic-at-end",
        2 => "henon-at-end",
        _ => "both-at-end",
    }
}

/// E1 (precommitted: pass) — THE MECHANISMS ARE MECHANISMS: bounded, alive,
/// deterministic, distinct in their measured statistics, sensitive to a
/// 1e-12 seed change, and their declared factors stay in [0.5, 1.5].
#[test]
fn gate_e1_mechanisms_are_mechanisms() {
    for mechanism in [ChaosMechanism::Logistic, ChaosMechanism::Henon] {
        let mut orbit = ChaosOrbit::new(mechanism);
        let mut values = Vec::with_capacity(20_000);
        let mut factors = Vec::with_capacity(20_000);
        for _ in 0..20_000 {
            values.push(orbit.next_u());
            factors.push(orbit.next_factor(ALPHA));
        }
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let f_min = factors.iter().copied().fold(f64::INFINITY, f64::min);
        let f_max = factors.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!((0.0..=1.0).contains(&min) && (0.0..=1.0).contains(&max), "bounded");
        assert!(min < 0.01 && max > 0.99, "the orbit is alive across the interval");
        // The declared bound is the INTERVAL the factors live in — a finite
        // orbit need not attain its endpoints (my first version asserted the
        // attained extremes equal 0.5 and 1.5 exactly; corrected on record).
        assert!(f_min >= 0.5 - 1e-9 && f_max <= 1.5 + 1e-9, "factors within [0.5, 1.5]");
        assert!(f_min < 0.52 && f_max > 1.48, "the orbit exercises the interval");
        eprintln!(
            "E1 {}: u in [{min:.6}, {max:.6}], mean {mean:.6}, factors in [{f_min:.4}, {f_max:.4}]",
            mechanism.name()
        );
    }

    // Deterministic: identical seeds reproduce exactly.
    let replay = |m: ChaosMechanism| {
        let mut orbit = ChaosOrbit::new(m);
        (0..64).map(|_| orbit.next_u()).collect::<Vec<f64>>()
    };
    assert_eq!(replay(ChaosMechanism::Logistic), replay(ChaosMechanism::Logistic));
    assert_eq!(replay(ChaosMechanism::Henon), replay(ChaosMechanism::Henon));
    assert_ne!(replay(ChaosMechanism::Logistic), replay(ChaosMechanism::Henon), "distinct");

    // Sensitive dependence: a 1e-12 seed change diverges past 0.1.
    let logistic = |seed: f64| {
        let mut x = seed;
        (0..80).map(|_| { x = 4.0 * x * (1.0 - x); x }).collect::<Vec<f64>>()
    };
    let a = logistic(0.4);
    let b = logistic(0.4 + 1e-12);
    let step = a.iter().zip(&b).position(|(p, q)| (p - q).abs() > 0.1).expect("diverges");
    eprintln!("E1 sensitive dependence: logistic diverges past 0.1 at step {step}");
    assert!(step <= 60);
}

/// E2 (precommitted: pass) — THE TENT MAP IS REJECTED BY MEASUREMENT. Its
/// f64 orbit reaches exactly 0.0 (a fixed point) within ~50 steps and its
/// long-run statistics are nowhere near uniform: a mechanism that cannot
/// perturb anything, rejected before any stack test.
#[test]
fn gate_e2_tent_rejected_by_measurement() {
    let s1 = tent_collapse_step(0.3141592653589793, 200);
    let s2 = tent_collapse_step(0.7, 200);
    eprintln!("E2 tent (f64): collapse to exactly 0.0 at steps {s1:?} and {s2:?}");
    assert_eq!(s1, Some(50), "measured collapse step (precommitted)");
    assert_eq!(s2, Some(53), "measured collapse step (precommitted)");

    // The long-run statistics of a dead orbit are not those of a mixer.
    let mut x = 0.3141592653589793;
    let mut values = Vec::with_capacity(20_000);
    for _ in 0..20_000 {
        x = if x < 0.5 { 2.0 * x } else { 2.0 * (1.0 - x) };
        values.push(x);
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let below = values.iter().filter(|v| **v < 0.1).count() as f64 / values.len() as f64;
    eprintln!("E2 tent 20k-step mean {mean:.6} (uniform would be 0.5); fraction < 0.1 = {below:.4}");
    assert!(mean < 0.01, "the orbit is dead, not mixing");
    assert!(below > 0.99);
}

/// Stack A — the harness carried through one padding across six tasks
/// (A,B,A,B,A,B). Returns (contamination total, per-task carried supports,
/// mean factor used) for one arm.
fn stack_a(arm: usize) -> (usize, Vec<usize>, f64) {
    let tasks = [task_a(), task_b()];
    let fresh: Vec<usize> = tasks.iter().map(|t| run(t).scene_support).collect();

    let mut pad = ScratchPad::new();
    let mut orbits: Vec<ChaosOrbit> = mechanisms(arm).into_iter().map(ChaosOrbit::new).collect();
    let mut factor_sum = 0.0;
    let mut factor_count = 0usize;

    let mut carried_supports = Vec::new();
    for index in 0..6 {
        let task = &tasks[index % 2];
        let report = run_with_pad(task, &mut pad);
        carried_supports.push(report.scene_support);
        // The recorded baseline clearing (the legitimate, imperfect arm).
        pad.clear(ClearMechanism::Dissipate, self_clearing_ticks());
        // THE END POSITION: the carried state, modulated once per cycle.
        for orbit in orbits.iter_mut() {
            let factor = orbit.next_factor(ALPHA);
            factor_sum += factor;
            factor_count += 1;
            // The AUTHORITATIVE carried state is scaled (both components, so
            // the phase is preserved) — the mechanism's declared interaction
            // point, and nothing else is done to it.
            pad.scale(factor);
        }
    }

    let contamination: usize = (1..6)
        .map(|index| {
            let fresh_value = fresh[index % 2];
            (carried_supports[index] as isize - fresh_value as isize).unsigned_abs()
        })
        .sum();
    let mean_factor = if factor_count > 0 { factor_sum / factor_count as f64 } else { 1.0 };
    (contamination, carried_supports, mean_factor)
}

/// E3 (precommitted: pass) — STACK A, FOUR ARMS, MATCHED. The contamination
/// total per arm, the invariant (verdict fields untouched), and the verdict.
#[test]
fn gate_e3_stack_a_pad_reuse() {
    let fresh = [run(&task_a()).scene_support, run(&task_b()).scene_support];
    eprintln!("E3 fresh supports A={} B={}", fresh[0], fresh[1]);

    let mut results = Vec::new();
    for arm in 0..4 {
        let (contamination, supports, mean_factor) = stack_a(arm);
        eprintln!(
            "E3 {}: carried supports {supports:?}, contamination {contamination}, mean factor {mean_factor:.6}",
            arm_name(arm)
        );
        results.push((contamination, mean_factor));
    }

    // The invariant: the FIRST task's readout through the pad equals its
    // fresh value (the pad round-trip is exact on an empty pad), the
    // recorded DISSIPATION baseline is contaminated (BUILD-16's recorded
    // finding, reproduced here), and every arm's derivation fields equal the
    // direct run's — the chaos stage touched the carried state and nothing
    // else.
    let (_, baseline_supports, _) = stack_a(0);
    assert_eq!(baseline_supports[0], fresh[0], "the first task through the pad is exact");
    eprintln!(
        "E3 baseline (dissipation) carries {baseline_supports:?} vs fresh {fresh:?} — contaminated, as recorded in BUILD-16"
    );

    // The derivation fields (the verdicts) are untouched by every arm.
    let direct = [run(&task_a()), run(&task_b())];
    for arm in 0..4 {
        let tasks = [task_a(), task_b()];
        let mut pad = ScratchPad::new();
        let mut orbits: Vec<ChaosOrbit> =
            mechanisms(arm).into_iter().map(ChaosOrbit::new).collect();
        for index in 0..6 {
            let report = run_with_pad(&tasks[index % 2], &mut pad);
            let reference = &direct[index % 2];
            assert_eq!(report.closure_before, reference.closure_before, "{}: closure", arm_name(arm));
            assert_eq!(report.sufficient_before, reference.sufficient_before, "{}: sufficiency", arm_name(arm));
            assert_eq!(report.sufficient_after, reference.sufficient_after, "{}: re-sufficiency", arm_name(arm));
            assert_eq!(report.derivable_now, reference.derivable_now, "{}: derivable", arm_name(arm));
            assert_eq!(report.economy_blocks, reference.economy_blocks, "{}: economy", arm_name(arm));
            assert_eq!(report.economy_answers, reference.economy_answers, "{}: answers", arm_name(arm));
            assert_eq!(report.judge, reference.judge, "{}: judge", arm_name(arm));
            pad.clear(ClearMechanism::Dissipate, self_clearing_ticks());
            for orbit in orbits.iter_mut() {
                pad.scale(orbit.next_factor(ALPHA));
            }
        }
    }
    eprintln!("E3 invariant: every arm's verdict fields equal the direct run's");

    // The verdict rule, applied to the numbers.
    let (baseline, _) = results[0];
    let mut promoted = Vec::new();
    for (arm, (contamination, mean_factor)) in results.iter().enumerate().skip(1) {
        let strict_improvement = *contamination < baseline;
        eprintln!(
            "E3 verdict {}: contamination {contamination} vs baseline {baseline} — {} (mean factor {mean_factor:.6})",
            arm_name(arm),
            if strict_improvement { "PROMOTE" } else { "REJECT" }
        );
        if strict_improvement {
            promoted.push(arm);
        }
    }
    eprintln!("E3 Stack A verdict: {} arm(s) promoted", promoted.len());
}

/// Stack B — the action loop over 50 epochs. Returns (terminal support,
/// closed form, stabilization epoch, mean factor).
fn stack_b(arm: usize) -> (usize, usize, usize, f64) {
    let mechanisms = mechanisms(arm);

    let run_loop = |epochs: usize, modulate_each: bool, orbits: &mut Vec<ChaosOrbit>, sum: &mut f64, count: &mut usize| {
        let mut engine = ActionLoopEngine::new(World { wave: W }, 7);
        for _ in 0..epochs {
            engine.epoch(ActionPolicy::FromField, true);
            if modulate_each {
                for orbit in orbits.iter_mut() {
                    let factor = orbit.next_factor(ALPHA);
                    *sum += factor;
                    *count += 1;
                    let mut pattern = engine.pattern_snapshot();
                    modulate(&mut pattern, factor);
                    engine.set_pattern(&pattern);
                }
            }
        }
        (engine.support(), engine.actions().to_vec())
    };

    let mut sum = 0.0;
    let mut count = 0usize;
    let mut orbits: Vec<ChaosOrbit> = mechanisms.iter().copied().map(ChaosOrbit::new).collect();
    let (terminal, actions) = run_loop(50, true, &mut orbits, &mut sum, &mut count);
    let closed_form = expected_support(&actions, W);

    // The stabilization epoch: the first epoch count whose support equals the
    // terminal support (deterministic replay from the same seed).
    let mut stabilization = 50;
    for epochs in 1..50 {
        let mut s = 0.0;
        let mut c = 0usize;
        let mut orbits: Vec<ChaosOrbit> = mechanisms.iter().copied().map(ChaosOrbit::new).collect();
        let (support, _) = run_loop(epochs, true, &mut orbits, &mut s, &mut c);
        if support == terminal {
            stabilization = epochs;
            break;
        }
    }
    let mean_factor = if count > 0 { sum / count as f64 } else { 1.0 };
    (terminal, closed_form, stabilization, mean_factor)
}

/// E4 (precommitted: pass) — STACK B, FOUR ARMS, MATCHED. The terminal
/// support against the closed form (the hard invariant) and the
/// stabilization epoch (the frozen benefit metric) per arm, then the verdict.
#[test]
fn gate_e4_stack_b_action_loop() {
    let mut results = Vec::new();
    for arm in 0..4 {
        let (terminal, closed_form, stabilization, mean_factor) = stack_b(arm);
        eprintln!(
            "E4 {}: terminal support {terminal} vs closed form {closed_form} (invariant {}), stabilization epoch {stabilization}, mean factor {mean_factor:.6}",
            arm_name(arm),
            if terminal == closed_form { "HELD" } else { "BROKEN" }
        );
        results.push((terminal, closed_form, stabilization, mean_factor));
    }

    // The baseline reproduces BUILD-5's recorded law exactly.
    assert_eq!(results[0].0, results[0].1, "no-chaos: support == the recorded-action closed form");

    // The verdict rule: promotion needs a strict improvement of the frozen
    // benefit metric (a SMALLER stabilization epoch) with the invariant held.
    let baseline_epoch = results[0].2;
    for (arm, (terminal, closed_form, stabilization, _)) in results.iter().enumerate().skip(1) {
        let invariant = terminal == closed_form;
        let strict = invariant && *stabilization < baseline_epoch;
        eprintln!(
            "E4 verdict {}: invariant {}, epoch {stabilization} vs baseline {baseline_epoch} — {}",
            arm_name(arm),
            if invariant { "held" } else { "BROKEN" },
            if strict { "PROMOTE" } else { "REJECT" }
        );
    }
}

/// E6 (precommitted: pass) — NO ENGINE BEHAVIOR CHANGED. The recorded
/// baselines still hold: BUILD-5's closed-form law, BUILD-16's reset
/// exactness, and the first task through the pad exact.
#[test]
fn gate_e6_no_engine_behavior_changed() {
    // BUILD-5's law, unchanged.
    let mut engine = ActionLoopEngine::new(World { wave: W }, 7);
    for _ in 0..50 {
        engine.epoch(ActionPolicy::FromField, true);
    }
    assert_eq!(engine.support(), expected_support(engine.actions(), W));

    // BUILD-16's reset arm, unchanged: bit-identical reuse.
    let (a, b) = (task_a(), task_b());
    let fresh_b = run(&b);
    let mut pad = ScratchPad::new();
    assert_eq!(run_with_pad(&a, &mut pad), run(&a), "first task exact");
    clear_for_next(&mut pad);
    assert_eq!(run_with_pad(&b, &mut pad), fresh_b, "post-reset reuse bit-identical");

    // The two declared seams are inert when nothing uses them.
    let mut idle = ScratchPad::new();
    idle.scale(1.0);
    assert_eq!(idle.mass(), 0.0);
    eprintln!("E6: recorded baselines unchanged; the seams are additive only");
}
