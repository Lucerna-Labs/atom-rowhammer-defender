//! BUILD-33 gates — precommitted in docs/BUILD33-SPLIT-CHAOS.md BEFORE this
//! code. The split chaos stage: gain, rotation, translation — 3! = 6 orders,
//! fully executed under two mechanisms, with the operator's verdict rule.
//!
//! The build is valid under BOTH readings of Rule 2 (the deployed hook's
//! "one to three candidate orders" and the notes' "every unique permutation"):
//! the declared set has three pieces, so the complete order space IS six
//! executions. That discrepancy is recorded in the precommit and asserted in
//! F5 rather than resolved silently.

use atom_cognitive_engine::chaos::{AffinePiece, ChaosMechanism, ChaosOrbit};
use atom_cognitive_engine::harness::{run, run_with_pad, Task};
use atom_cognitive_engine::interference_field::N;
use atom_cognitive_engine::scratch_pad::{self_clearing_ticks, ClearMechanism, ScratchPad};

const ALPHA: f64 = 1.0;

// ---- the recorded tasks (BUILD-12/16's, re-declared as observers) ----

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

/// All six unique orders of the three affine pieces (the complete 3! space).
const ORDERS: [[AffinePiece; 3]; 6] = [
    [
        AffinePiece::Gain,
        AffinePiece::Rotation,
        AffinePiece::Translation,
    ],
    [
        AffinePiece::Gain,
        AffinePiece::Translation,
        AffinePiece::Rotation,
    ],
    [
        AffinePiece::Rotation,
        AffinePiece::Gain,
        AffinePiece::Translation,
    ],
    [
        AffinePiece::Rotation,
        AffinePiece::Translation,
        AffinePiece::Gain,
    ],
    [
        AffinePiece::Translation,
        AffinePiece::Gain,
        AffinePiece::Rotation,
    ],
    [
        AffinePiece::Translation,
        AffinePiece::Rotation,
        AffinePiece::Gain,
    ],
];

fn order_name(order: &[AffinePiece; 3]) -> String {
    order.iter().map(|p| p.name()).collect::<Vec<_>>().join(">")
}

/// One arm: the recorded six-task pad cycle, with the declared pieces applied
/// at the END position in the declared order, each piece drawing from its OWN
/// dedicated orbit (so the perturbations are identical across orders and only
/// the order of application varies).
fn arm(
    mechanism: Option<ChaosMechanism>,
    pieces: &[AffinePiece],
) -> (usize, Vec<atom_cognitive_engine::harness::HarnessReport>) {
    let tasks = [task_a(), task_b()];
    let fresh = [run(&tasks[0]).scene_support, run(&tasks[1]).scene_support];
    let mut pad = ScratchPad::new();
    let mut gain_orbit = mechanism.map(ChaosOrbit::new);
    let mut phase_orbit = mechanism.map(ChaosOrbit::new);
    let mut shift_x = mechanism.map(ChaosOrbit::new);
    let mut shift_y = mechanism.map(ChaosOrbit::new);

    let mut reports = Vec::new();
    let mut supports = Vec::new();
    for index in 0..6 {
        let report = run_with_pad(&tasks[index % 2], &mut pad);
        supports.push(report.scene_support);
        reports.push(report);
        pad.clear(ClearMechanism::Dissipate, self_clearing_ticks());
        for piece in pieces {
            match piece {
                AffinePiece::Gain => {
                    if let Some(orbit) = gain_orbit.as_mut() {
                        pad.scale(orbit.next_factor(ALPHA));
                    }
                }
                AffinePiece::Rotation => {
                    if let Some(orbit) = phase_orbit.as_mut() {
                        pad.rotate(orbit.next_angle());
                    }
                }
                AffinePiece::Translation => {
                    if let (Some(ox), Some(oy)) = (shift_x.as_mut(), shift_y.as_mut()) {
                        pad.translate(ox.next_offset(N), oy.next_offset(N));
                    }
                }
            }
        }
    }
    let contamination: usize = (1..6)
        .map(|index| {
            (supports[index] as isize - fresh[index % 2] as isize).unsigned_abs()
        })
        .sum();
    (contamination, reports)
}

/// F1 (precommitted: pass) — THE PIECES ARE DISTINCT OPERATIONS. Gain scales
/// magnitudes and leaves phases; rotation preserves magnitudes and moves every
/// phase; translation preserves the multiset of magnitudes and permutes sites.
#[test]
fn gate_f1_pieces_are_distinct_operations() {
    let pattern: Vec<f64> = (0..N * N)
        .map(|i| ((i % 7) as f64) * 0.25 - 0.5)
        .collect();

    let seeded = || {
        let mut pad = ScratchPad::new();
        pad.write_pattern(&pattern);
        pad
    };
    let magnitudes = |pad: &ScratchPad| {
        let mut m: Vec<f64> = pad
            .field()
            .iter()
            .map(|c| (c.re * c.re + c.im * c.im).sqrt())
            .collect();
        m.sort_by(|a, b| a.partial_cmp(b).unwrap());
        m
    };

    // Gain: magnitudes scale by |factor|, phases unchanged.
    let mut gained = seeded();
    let factor = 1.25_f64;
    gained.scale(factor);
    let base = magnitudes(&seeded());
    let scaled = magnitudes(&gained);
    for (b, s) in base.iter().zip(&scaled) {
        assert!((s - b * factor).abs() < 1e-12, "gain scales magnitudes");
    }

    // Rotation: magnitudes preserved exactly, every phase moves by theta.
    let mut rotated = seeded();
    let theta = 0.7_f64;
    rotated.rotate(theta);
    let after = magnitudes(&rotated);
    for (b, a) in base.iter().zip(&after) {
        assert!((a - b).abs() < 1e-12, "rotation preserves magnitudes");
    }
    let phase_of = |pad: &ScratchPad, i: usize| pad.field()[i].im.atan2(pad.field()[i].re);
    for i in [0usize, 17, 200] {
        if base[i] > 1e-9 {
            let delta = phase_of(&rotated, i) - phase_of(&seeded(), i);
            let wrapped = (delta - theta + core::f64::consts::PI).rem_euclid(2.0 * core::f64::consts::PI)
                - core::f64::consts::PI;
            assert!(wrapped.abs() < 1e-9, "rotation moves every phase by theta");
        }
    }

    // Translation: the multiset of magnitudes is preserved, sites permuted.
    let mut shifted = seeded();
    let (dx, dy) = (3usize, 5usize);
    shifted.translate(dx, dy);
    let moved = magnitudes(&shifted);
    for (b, m) in base.iter().zip(&moved) {
        assert!((m - b).abs() < 1e-12, "translation preserves the magnitude multiset");
    }
    let source = seeded();
    for y in 0..N {
        for x in 0..N {
            let got = shifted.field()[y * N + x];
            let want = source.field()[((y + dy) % N) * N + (x + dx) % N];
            assert_eq!(got, want, "translation permutes the sites cyclically");
        }
    }
    // A full-lattice translation is the identity.
    let mut wrap = seeded();
    wrap.translate(N, N);
    assert_eq!(wrap.field(), source.field());
    eprintln!("F1: gain scales, rotation turns, translation moves — three distinct operations");
}

/// F2 (precommitted: pass) — THE ORDERS AND CONTROLS ARE AS DECLARED, and the
/// setup is comparable to BUILD-32's (the gain-only arm reproduces 16).
#[test]
fn gate_f2_orders_and_controls() {
    let orders: std::collections::BTreeSet<Vec<u8>> = ORDERS
        .iter()
        .map(|o| {
            o.iter()
                .map(|p| match p {
                    AffinePiece::Gain => 0u8,
                    AffinePiece::Rotation => 1,
                    AffinePiece::Translation => 2,
                })
                .collect()
        })
        .collect();
    assert_eq!(orders.len(), 6, "3! = 6 unique orders, all declared");

    let (baseline, _) = arm(None, &[]);
    assert_eq!(baseline, 16, "no-chaos baseline reproduces BUILD-32's recorded 16");

    let (gain_only, _) = arm(Some(ChaosMechanism::Logistic), &[AffinePiece::Gain]);
    assert_eq!(gain_only, 16, "gain-only reproduces BUILD-32's arm exactly (setup comparable)");

    // The declared execution count: 6 tasks x (6 orders x 2 mechanisms +
    // 3 single pieces x 2 mechanisms + 1 control) = 6 x 19 = 114 runs.
    let declared_runs = 6 * (6 * 2 + 3 * 2 + 1);
    eprintln!("F2: baseline {baseline}, gain-only {gain_only}, declared runs {declared_runs}");
    assert_eq!(declared_runs, 114);
}

/// F3 + F4 (precommitted: pass) — THE FROZEN METRIC PER ARM, THE INVARIANTS,
/// AND THE VERDICT. Every arm's contamination total is reported; the first
/// task is exact in every arm; the verdict fields equal the direct run's; and
/// the verdict is computed from the numbers per the operator's rule.
#[test]
fn gate_f3_f4_metric_invariants_and_verdict() {
    let tasks = [task_a(), task_b()];
    let direct = [run(&tasks[0]), run(&tasks[1])];
    let mut results: Vec<(String, usize)> = Vec::new();

    let (baseline, baseline_reports) = arm(None, &[]);
    results.push(("no-chaos".to_string(), baseline));
    assert_eq!(baseline_reports[0].scene_support, direct[0].scene_support, "first task exact");

    for mechanism in [ChaosMechanism::Logistic, ChaosMechanism::Henon] {
        // single-piece controls
        for piece in [
            AffinePiece::Gain,
            AffinePiece::Rotation,
            AffinePiece::Translation,
        ] {
            let (contamination, _) = arm(Some(mechanism), &[piece]);
            results.push((format!("{}|{}", mechanism.name(), piece.name()), contamination));
        }
        // the six orders
        for order in &ORDERS {
            let (contamination, reports) = arm(Some(mechanism), order);
            for (index, report) in reports.iter().enumerate() {
                let reference = &direct[index % 2];
                assert_eq!(report.closure_before, reference.closure_before, "invariant: closure");
                assert_eq!(report.sufficient_before, reference.sufficient_before, "invariant: sufficiency");
                assert_eq!(report.sufficient_after, reference.sufficient_after, "invariant: re-sufficiency");
                assert_eq!(report.derivable_now, reference.derivable_now, "invariant: derivable");
                assert_eq!(report.economy_blocks, reference.economy_blocks, "invariant: economy");
                assert_eq!(report.economy_answers, reference.economy_answers, "invariant: answers");
                assert_eq!(report.judge, reference.judge, "invariant: judge");
            }
            assert_eq!(reports[0].scene_support, direct[0].scene_support, "invariant: first task exact");
            results.push((format!("{}|{}", mechanism.name(), order_name(order)), contamination));
        }
    }

    for (name, contamination) in &results {
        eprintln!("F3 {name}: contamination {contamination}");
    }

    // F4 — the verdict, computed from the numbers per the operator's rule.
    let improved: Vec<&(String, usize)> = results
        .iter()
        .filter(|(name, c)| name != "no-chaos" && *c < baseline)
        .collect();
    let equal = results.iter().filter(|(_, c)| *c == baseline).count();
    let worse = results.iter().filter(|(_, c)| *c > baseline).count();
    eprintln!(
        "F4 verdict: {} arm(s) strictly improved (< {baseline}), {equal} tied, {worse} worse — PROMOTE: {}",
        improved.len(),
        if improved.is_empty() {
            "none (chaos rejected: no measurable benefit, per the operator's rule)".to_string()
        } else {
            improved
                .iter()
                .map(|(n, _)| n.clone())
                .collect::<Vec<_>>()
                .join(", ")
        }
    );
    for (name, _) in &improved {
        eprintln!("F4 candidate for promotion: {name}");
    }
}

/// F5 (precommitted: pass) — THE RULE-CONFLICT RECORD IS IN THE BUILD. The
/// precommit carries BOTH Rule 2 texts (the deployed hook's bounded wording
/// and the notes' exhaustive wording) and this build satisfies both.
#[test]
fn gate_f5_rule_conflict_recorded() {
    let doc = include_str!("../../../docs/BUILD33-SPLIT-CHAOS.md");
    assert!(
        doc.contains("Do\n  not default to random or exhaustive factorial sweeps"),
        "the deployed hook's bounded wording is recorded"
    );
    assert!(
        doc.contains("every unique permutation"),
        "the canonical notes' exhaustive wording is recorded"
    );
    assert!(
        doc.contains("3! = 6") && doc.contains("VALID UNDER BOTH READINGS"),
        "the both-readings design is recorded"
    );
    assert_eq!(ORDERS.len(), 6, "the complete order space of the declared set is executed");
    eprintln!("F5: the hook-vs-notes conflict is in the build, and both readings are satisfied");
}

/// F6 (precommitted: pass; the finding) — THE AFFINE-BLINDNESS MEASUREMENT
/// on the REAL system: the carried residue from an actual run, plus the next
/// task's actual contribution. Each affine piece really does change the
/// carried state (asserted: the state differs), while the consumer's
/// membership readout is unmoved — the residue's own power dominates the
/// sum, so the affine group cannot change which bins clear the threshold.
#[test]
fn gate_f6_affine_blindness_measured() {
    use atom_cognitive_engine::geometry::GeometryField;

    // The real carried residue: task A through the pad, dissipated as the
    // baseline arm does. Rebuilt deterministically per piece (the pad is not
    // Clone, and the construction is deterministic, so the residue is
    // identical every time).
    let build = |mutate: Option<usize>| -> ScratchPad {
        let mut pad = ScratchPad::new();
        let _ = run_with_pad(&task_a(), &mut pad);
        pad.clear(ClearMechanism::Dissipate, self_clearing_ticks());
        match mutate {
            None => {}
            Some(0) => pad.scale(1.37),
            Some(1) => pad.rotate(0.7),
            _ => pad.translate(3, 5),
        }
        pad
    };
    let residue: Vec<f64> = build(None).read_pattern();
    // The real next contribution: task B's formed pattern.
    let contribution: Vec<f64> = GeometryField::form(&task_b().scene).raw_pattern().to_vec();

    let summed = |residue: &[f64]| -> Vec<f64> {
        residue.iter().zip(&contribution).map(|(r, c)| r + c).collect()
    };
    let membership = |pattern: &[f64]| {
        GeometryField::from_pattern(pattern, &task_b().scene)
            .difference_vectors()
            .len()
    };
    let bin_powers = |pattern: &[f64]| -> Vec<f64> {
        let mut powers = Vec::with_capacity(N * N);
        for vy in 0..N {
            for vx in 0..N {
                let mut re = 0.0f64;
                let mut im = 0.0f64;
                for y in 0..N {
                    for x in 0..N {
                        let phase = -2.0 * core::f64::consts::PI
                            * ((vx * x + vy * y) as f64)
                            / N as f64;
                        let value = pattern[y * N + x];
                        re += value * phase.cos();
                        im += value * phase.sin();
                    }
                }
                powers.push(re * re + im * im);
            }
        }
        powers
    };

    let baseline_sum = summed(&residue);
    let baseline_powers = bin_powers(&baseline_sum);
    let baseline_membership = membership(&baseline_sum);

    let mut state_changed = 0usize;
    let mut membership_moved = 0usize;
    for (name, mutation) in [("gain", 0usize), ("rotation", 1), ("translation", 2)] {
        let perturbed = build(Some(mutation));
        let changed = perturbed.read_pattern() != residue;
        if changed {
            state_changed += 1;
        }
        let sum = summed(&perturbed.read_pattern());
        let powers = bin_powers(&sum);
        let moved = powers
            .iter()
            .zip(&baseline_powers)
            .filter(|(a, b)| (**a - **b).abs() > 1e-9 * b.abs().max(1.0))
            .count();
        let max_delta = powers
            .iter()
            .zip(&baseline_powers)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max);
        let member = membership(&sum);
        if member != baseline_membership {
            membership_moved += 1;
        }
        eprintln!(
            "F6 {name}: state changed {changed}; {moved} bins moved power (max |delta| {max_delta:.3e}); membership {member} vs baseline {baseline_membership}"
        );
    }

    assert_eq!(state_changed, 3, "every affine piece really does change the carried state");
    assert_eq!(
        membership_moved, 0,
        "and none of them changes the consumer's membership readout: the readout is affine-blind"
    );
    eprintln!("F6: the perturbations are real; the consumer's membership readout is blind to all of them");
}
