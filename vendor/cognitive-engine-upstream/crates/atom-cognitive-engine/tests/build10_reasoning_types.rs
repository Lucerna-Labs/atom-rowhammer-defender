//! BUILD-10 gates — precommitted in docs/BUILD10-REASONING-TYPES.md
//! BEFORE this code. Bottom-up, top-down, abductive, counterfactual,
//! analogical, defeasible, non-linear: each an exact measured task.

use atom_cognitive_engine::reasoning_types::{
    analogical_rotation, analogical_transfer, apply_rotation, bottom_up,
    counterfactual_differences, recover_world, top_down, top_down_complement, CoherentMemory,
};
use atom_cognitive_engine::spatial::rotate90;

const PART1: [(i64, i64); 2] = [(2, 2), (6, 2)];
const PART2: [(i64, i64); 2] = [(10, 10), (10, 6)];

/// RT1 (precommitted: pass) — BOTTOM-UP: parts 2+2 bins, whole 12, the
/// 8 new bins exactly the cross-part relations.
#[test]
fn gate_rt1_bottom_up() {
    let (s1, s2, sw, cross) = bottom_up(&PART1, &PART2);
    eprintln!("bottom-up: parts {s1}+{s2} -> whole {sw}, cross-part relations {cross}");
    assert_eq!((s1, s2, sw, cross), (2, 2, 12, 8));
}

/// RT2 (precommitted: pass) — TOP-DOWN: parts constrained by the whole;
/// the complement is exactly the whole's closure.
#[test]
fn gate_rt2_top_down() {
    let mut whole = PART1.to_vec();
    whole.extend_from_slice(&PART2);
    let (whole_closure, part_closure, inside) = top_down(&PART1, &whole);
    eprintln!("top-down: |H_whole| {whole_closure}, |H_part| {part_closure}, part-inside {inside}");
    assert_eq!(whole_closure, 25);
    assert!(part_closure >= 1);
    assert!(inside, "the part's differences lie inside the whole's closure");
    assert!(
        top_down_complement(&PART1, &PART2, &whole),
        "closure(parts) == closure(whole): the complement is exactly permitted"
    );
}

/// RT3 (precommitted: pass) — ABDUCTION + COUNTERFACTUAL: recover the
/// world exactly; predict an untaken action's consequences and verify.
#[test]
fn gate_rt3_abduction_counterfactual() {
    let world = (16i64, 6i64);
    let action = (5i64, 10i64);
    let observed = ((action.0 - world.0).rem_euclid(20), (action.1 - world.1).rem_euclid(20));
    let recovered = recover_world(action, observed);
    eprintln!("abduction: recovered world {recovered:?} (true {world:?})");
    assert_eq!(recovered, world, "the best explanation is exact");
    // Counterfactual: predict for an UNTAKEN action, verify when taken.
    let untaken = (9i64, 14i64);
    let predicted = counterfactual_differences(untaken, recovered);
    let actual = ((untaken.0 - world.0).rem_euclid(20), (untaken.1 - world.1).rem_euclid(20));
    let actual_min = if actual.0 > 10 { (actual.0 - 20, actual.1) } else { actual };
    let actual_min = if actual_min.1 > 10 { (actual_min.0, actual_min.1 - 20) } else { actual_min };
    eprintln!("counterfactual: predicted {predicted:?}, actual {actual_min:?}");
    assert_eq!(predicted, actual_min, "counterfactual prediction verified exactly");
}

/// RT4 (precommitted: pass) — ANALOGICAL: translation transfer exact;
/// rotational analogy recovered from structure and applied; a broken
/// analogue (reflection) is not mistaken for a rotation.
#[test]
fn gate_rt4_analogical() {
    let d = analogical_transfer((2, 2), (6, 2), (10, 10));
    eprintln!("translation analogy: D = {d:?}");
    assert_eq!(d, (14, 10));
    let base = [(2i64, 2i64), (6, 2), (2, 5)];
    let copy: Vec<(i64, i64)> = base.iter().map(|p| rotate90(*p)).collect();
    let rotation = analogical_rotation(&base, &copy);
    eprintln!("rotational analogy: recovered rotation index {rotation:?}");
    assert_eq!(rotation, Some(1), "90-degree rotation recovered");
    let prediction = apply_rotation(1, (10, 10));
    assert_eq!(prediction, rotate90((10, 10)));
}

/// RT5 (precommitted: pass) — DEFEASIBLE: exact coherent retraction
/// (support without the epoch, restored on re-presentation). The three
/// rejected retractors (condensation, thinning, dissipation) are
/// recorded in the doc as measured findings.
#[test]
fn gate_rt5_defeasible() {
    let scene = [(4i64, 4i64), (5, 4), (4, 5), (5, 5)];
    let other = [(12i64, 12i64), (16, 12)];
    let mut memory = CoherentMemory::new();
    memory.present(&scene, 0.0);
    let after_first = memory.support();
    memory.present(&other, 0.0);
    let after_second = memory.support();

    // Reference: memory with ONLY the second epoch.
    let mut only_second = CoherentMemory::new();
    only_second.present(&other, 0.0);
    let reference = only_second.support();

    memory.retract(0);
    let after_retraction = memory.support();
    eprintln!(
        "defeasible: after_first {after_first}, after_second {after_second}, after_retraction {after_retraction}, reference {reference}"
    );
    assert_eq!(
        after_retraction, reference,
        "retracting an epoch recovers EXACTLY the without-that-epoch memory"
    );
    memory.restore(0);
    assert_eq!(memory.support(), after_second, "restoring returns the full memory");
}

/// RT6 (precommitted: pass) — NON-LINEAR: the three regime signatures.
/// FIXED POINT: closure saturates (reasoning_closure); CYCLE: rational
/// shear revisits few positions; GROWTH: irrational shear never repeats.
#[test]
fn gate_rt6_nonlinear_regimes() {
    use atom_cognitive_engine::generation::{GeneratingField, ShearAxis, GOLDEN_CONJUGATE};
    use atom_cognitive_engine::perception::Reading;
    use atom_cognitive_engine::reasoning_closure::ClosureEngine;

    // Fixed point.
    let readings: Vec<Reading> = [(0.80, 0.70), (0.80, 0.30), (0.30, 0.70)]
        .iter()
        .flat_map(|(e, c)| (0..5u64).map(move |k| Reading::new(k, 0.5, 0.5, 0.5, *e, *c, 0.5)))
        .collect();
    let mut closure = ClosureEngine::from_readings(&readings, atom_cognitive_engine::reasoning_closure::CycleOrder::MeasureThenInject);
    for _ in 0..40 {
        closure.step();
    }
    let fixed_support = closure.support();

    // Cycle: rational shear.
    let mut rational = GeneratingField::from_seed_wave((0.35, 0.15), ShearAxis::Both, 0.6, 0.6);
    let mut rational_positions = Vec::new();
    rational_positions.push(rational.peak_x_frequency());
    for _ in 0..63 {
        rational.step();
        rational_positions.push(rational.peak_x_frequency());
    }
    let rational_distinct = {
        let mut v: Vec<i64> = rational_positions.iter().map(|p| (p * 1e6).round() as i64).collect();
        v.sort_unstable();
        v.dedup();
        v.len()
    };

    // Growth: irrational shear.
    let mut irrational = GeneratingField::from_seed_wave(
        (0.35, 0.15),
        ShearAxis::Both,
        GOLDEN_CONJUGATE,
        GOLDEN_CONJUGATE,
    );
    let mut irrational_positions = Vec::new();
    irrational_positions.push(irrational.peak_x_frequency());
    for _ in 0..63 {
        irrational.step();
        irrational_positions.push(irrational.peak_x_frequency());
    }
    let irrational_distinct = {
        let mut v: Vec<i64> = irrational_positions.iter().map(|p| (p * 1e6).round() as i64).collect();
        v.sort_unstable();
        v.dedup();
        v.len()
    };

    eprintln!(
        "non-linear regimes: fixed-point support {fixed_support}, rational distinct {rational_distinct} (cycle), irrational distinct {irrational_distinct} (growth)"
    );
    assert!(fixed_support > 0, "fixed point exists");
    assert!(rational_distinct <= 10, "rational regime cycles (bounded)");
    assert!(irrational_distinct >= 60, "irrational regime grows (unbounded)");
}
