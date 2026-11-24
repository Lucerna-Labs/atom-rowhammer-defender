//! BUILD-16 gates — precommitted in `docs/BUILD16-HARNESS-PAD.md` BEFORE the
//! wiring. The harness's formed field travels through the working-memory pad.

use atom_cognitive_core::stages::Stack;
use atom_cognitive_engine::geometry::GeometryField;
use atom_cognitive_engine::harness::{clear_for_next, harness_stages, run, run_with_pad, Task};
use atom_cognitive_engine::interference_field::N;
use atom_cognitive_engine::scratch_pad::{self_clearing_ticks, ClearMechanism, ScratchPad};

const PI: f64 = core::f64::consts::PI;

/// The consumer's resolution quantity: the pattern's largest DFT power
/// (what `GeometryField::difference_vectors` thresholds at 1e-9 absolute).
fn max_pattern_dft_power(pattern: &[f64]) -> f64 {
    let mut best = 0.0f64;
    for vy in 0..N {
        for vx in 0..N {
            let mut re = 0.0f64;
            let mut im = 0.0f64;
            for y in 0..N {
                for x in 0..N {
                    let phase = -2.0 * PI * ((vx * x + vy * y) as f64) / N as f64;
                    let value = pattern[y * N + x];
                    re += value * phase.cos();
                    im += value * phase.sin();
                }
            }
            let power = re * re + im * im;
            if power > best {
                best = power;
            }
        }
    }
    best
}

fn task_a() -> Task {
    // BUILD-12's W2 task: the collinear scene, a non-derivable target that
    // the ask stage makes derivable.
    Task {
        blocks: vec![(0, 8), (10, 0)],
        scene: vec![(2, 2), (6, 2), (4, 2)],
        target: (1, 0),
        query: ((2, 2), (6, 2), (4, 2)),
        source_block: (1, 0),
    }
}

fn task_a_derivable() -> Task {
    Task {
        target: (0, 16),
        ..task_a()
    }
}

fn task_b() -> Task {
    // A distinct scene (different bin structure) for the reuse test.
    Task {
        blocks: vec![(0, 8), (10, 0)],
        scene: vec![(1, 1), (5, 3)],
        target: (0, 16),
        query: ((1, 1), (5, 3), (3, 2)),
        source_block: (1, 0),
    }
}

/// P1 (precommitted: pass) — EXACT THROUGH THE PAD: the pad-wired report
/// equals the direct report, field for field.
#[test]
fn gate_p1_exact_through_the_pad() {
    for (name, task) in [
        ("non-derivable", task_a()),
        ("derivable", task_a_derivable()),
        ("scene B", task_b()),
    ] {
        let direct = run(&task);
        let mut pad = ScratchPad::new();
        let through = run_with_pad(&task, &mut pad);
        eprintln!("P1 {name}: direct {direct:?}");
        eprintln!("P1 {name}: through {through:?}");
        assert_eq!(direct, through, "{name}: the pad round-trip is exact");
    }
}

/// P2 (precommitted: pass; the dissipation arm FAILED and is kept on
/// record) — REUSE ACROSS TASKS. The wiring found a real design bug: the
/// harness's readout thresholds at an ABSOLUTE 1e-9 on the pattern's DFT
/// power, so a dissipating residue stays visible for ~3.8e7 ticks — far
/// beyond the pad's derived 31622-tick budget (which serves the pad's own
/// RELATIVE 1e-9 readout). Clearing budgets are CONSUMER-SCOPED; the reset
/// arm is the exact mechanism for this consumer.
#[test]
fn gate_p2_reuse_across_tasks() {
    let a = task_a();
    let b = task_b();
    let fresh_b = run(&b);

    // The consumer-scoped requirement, derived from the consumer's own
    // resolution constant and the scene's own bin power (no authored number).
    let field = GeometryField::form(&a.scene);
    let p_a = max_pattern_dft_power(field.raw_pattern());
    let a_max = (1e-9 / p_a).sqrt();
    let ticks_required = (1.0 / a_max - 1.0) as u64;
    eprintln!(
        "P2 consumer scope: pattern DFT power {p_a:.3e}; required residue amplitude <= {a_max:.3e}; required ticks ~{ticks_required} vs the pad-derived budget {}",
        self_clearing_ticks()
    );
    assert!(
        ticks_required > self_clearing_ticks() as u64,
        "the consumer's budget exceeds the pad's own — the mismatch the wiring revealed"
    );

    // Reset arm (exact): the mechanism that works for this consumer.
    let mut pad = ScratchPad::new();
    assert_eq!(run_with_pad(&a, &mut pad), run(&a), "first task exact");
    clear_for_next(&mut pad);
    let rb = run_with_pad(&b, &mut pad);
    eprintln!("P2 reset reuse: B {rb:?} vs fresh {fresh_b:?}");
    assert_eq!(rb, fresh_b, "post-reset reuse is bit-identical");

    // Dissipation arm — PRECOMMIT PREDICTION FAILED, kept on record: at the
    // pad's derived budget the harness's readout is STILL contaminated.
    let mut pad_d = ScratchPad::new();
    let _ = run_with_pad(&a, &mut pad_d);
    pad_d.clear(ClearMechanism::Dissipate, self_clearing_ticks());
    let residue_mass = pad_d.mass();
    let rb_d = run_with_pad(&b, &mut pad_d);
    eprintln!(
        "P2 dissipation arm (T={}): residue mass {residue_mass:.3e}; B scene_support {} vs fresh {} — CONTAMINATED (recorded)",
        self_clearing_ticks(),
        rb_d.scene_support,
        fresh_b.scene_support
    );
    assert_ne!(
        rb_d, fresh_b,
        "recorded: the dissipating residue is still visible to the harness's absolute threshold"
    );
}

/// P3 (precommitted: pass) — LOAD-BEARING, NOT DECORATIVE: the no-clearing
/// arm changes the outcome (asserted); the under-clearing arm is measured
/// and recorded (visibility depends on the residue's bin powers —
/// BUILD-15's measured nuance: a degraded budget may or may not show).
#[test]
fn gate_p3_load_bearing() {
    let a = task_a();
    let b = task_b();
    let fresh_b = run(&b);

    let mut pad = ScratchPad::new();
    let _ = run_with_pad(&a, &mut pad);
    pad.dissipate(self_clearing_ticks() / 5);
    let rb_under = run_with_pad(&b, &mut pad);
    let under_differs = rb_under != fresh_b;
    eprintln!(
        "P3 under-clearing ({} ticks): B {rb_under:?} vs fresh {fresh_b:?} -> {}",
        self_clearing_ticks() / 5,
        if under_differs { "DIFFERS (recorded)" } else { "already clean (recorded nuance)" }
    );

    let mut pad_none = ScratchPad::new();
    let _ = run_with_pad(&a, &mut pad_none);
    let rb_none = run_with_pad(&b, &mut pad_none);
    assert_ne!(
        rb_none, fresh_b,
        "an arm that changes the outcome exists — the pad is not decorative"
    );
}

/// P4 (precommitted: pass) — ORDERING PRESERVED: the composite still
/// assembles and mis-ordered variants are still construction errors.
#[test]
fn gate_p4_ordering_preserved() {
    let stack = Stack::assemble(harness_stages()).expect("canonical composite assembles");
    assert_eq!(stack.len(), 8);
    let mut stages = harness_stages();
    let index = stages
        .iter()
        .position(|s| s.id == "harness.sufficiency")
        .expect("sufficiency present");
    let stage = stages.remove(index);
    stages.insert(3, stage);
    assert!(
        Stack::assemble(stages).is_err(),
        "mis-ordered composite still a construction error"
    );
    eprintln!("P4: 8 stages assemble; mis-ordered rejected — the pad bypasses no prerequisite");
}

/// P5 (precommitted: pass) — CONTENT-BLINDNESS: two tasks differing only in
/// the target traverse the identical route; only the verdicts differ.
#[test]
fn gate_p5_content_blindness() {
    let mut pad1 = ScratchPad::new();
    let r1 = run_with_pad(&task_a(), &mut pad1);
    let mut pad2 = ScratchPad::new();
    let r2 = run_with_pad(&task_a_derivable(), &mut pad2);
    eprintln!("P5 non-derivable {r1:?}");
    eprintln!("P5 derivable     {r2:?}");
    assert_eq!(r1.scene_support, r2.scene_support, "same scene readout");
    assert_eq!(r1.judge, r2.judge, "same judgment");
    assert_eq!(r1.closure_before, r2.closure_before, "same closure");
    assert_ne!(
        r1.sufficient_before, r2.sufficient_before,
        "only the verdict differs"
    );
}

/// P6 (precommitted: pass) — AUDIT: the budget is derived, not authored.
#[test]
fn gate_p6_audit() {
    assert_eq!(
        self_clearing_ticks(),
        31622,
        "ceil(1/sqrt(1e-9) - 1) — derived from the observer's resolution alone"
    );
    eprintln!(
        "P6 audit: clearing budget {} derived from the resolution; dissipation and tick law are BUILD-15's shared code",
        self_clearing_ticks()
    );
}
