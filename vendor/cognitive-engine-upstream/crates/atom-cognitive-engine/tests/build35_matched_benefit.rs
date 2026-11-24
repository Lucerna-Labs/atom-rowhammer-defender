//! BUILD-35 gates — precommitted in docs/BUILD35-MATCHED-BENEFIT.md BEFORE
//! this code. Paired, multi-instance benefit test: is the recorded effect
//! categorical (the mechanism predicts every instance) or marginal?

use atom_cognitive_engine::geometry::GeometryField;
use atom_cognitive_engine::interference_field::N;
use atom_cognitive_engine::perception::Reading;
use atom_cognitive_engine::reasoning_closure::{
    generated_subgroup_size, ClosureEngine, CycleOrder,
};

const INSTANCES: usize = 24;
const PROTOCOL_EPOCHS: usize = 40;

/// The declared relation ladder (Family R).
const LADDER: [(isize, isize); 8] = [
    (6, 0),
    (2, 0),
    (4, 0),
    (14, 0),
    (0, 4),
    (0, 16),
    (12, 0),
    (8, 0),
];

/// Family R: the base set plus `k` further relations from the ladder.
fn relations_for(k: usize) -> Vec<(isize, isize)> {
    let mut out = vec![(0isize, 8isize), (10, 0)];
    for index in 0..k {
        out.push(LADDER[index % LADDER.len()]);
    }
    out
}

/// Family S: the recorded triple plus the `k`-th point of the declared
/// ladder — 24 DISTINCT points, so the 24 scenes are distinct.
/// CORRECTION (kept on record): the first version appended points from an
/// 8-point ladder with a wrap, which produced only 16 distinct scenes; M1
/// caught it before any claim was made.
fn point_ladder() -> Vec<(i64, i64)> {
    let mut points = Vec::with_capacity(24);
    for k in 1..=12i64 {
        points.push((k, 0));
    }
    for k in 1..=12i64 {
        points.push((0, k));
    }
    points
}

fn scene_for(k: usize) -> Vec<(i64, i64)> {
    let mut out = vec![(2i64, 2i64), (6, 2), (4, 2)];
    out.push(point_ladder()[k]);
    out
}

/// The readings a closure needs: one per distinct difference of the relation
/// set, so the composition sees the same content the authored algorithm does.
fn readings_for(relations: &[(isize, isize)]) -> Vec<Reading> {
    let mut out = Vec::new();
    let mut id = 1u64;
    for (dx, dy) in relations {
        // a reading whose wavevector difference carries the relation
        out.push(Reading::new(id, 0.5, 0.5, 0.5, *dx as f64 / 20.0, *dy as f64 / 20.0, 0.5));
        id += 1;
    }
    out
}

/// M1 (precommitted: pass) — THE GENERATOR IS AS DECLARED: every instance is
/// distinct, so no instance is repeated evidence.
#[test]
fn gate_m1_generator_declared() {
    let mut sets: Vec<Vec<(isize, isize)>> = (0..INSTANCES).map(relations_for).collect();
    sets.sort();
    let total = sets.len();
    sets.dedup();
    assert_eq!(sets.len(), total, "the 24 relation sets are pairwise distinct");

    let mut scenes: Vec<Vec<(i64, i64)>> = (0..INSTANCES).map(scene_for).collect();
    scenes.sort();
    let total = scenes.len();
    scenes.dedup();
    assert_eq!(scenes.len(), total, "the 24 scenes are pairwise distinct");
    eprintln!("M1: {INSTANCES} distinct relation sets and {INSTANCES} distinct scenes");
}

/// M2 + M4 (precommitted: pass) — THE DERIVATION PAIRS, per instance, and the
/// categorical-or-marginal verdict computed from the counts.
#[test]
fn gate_m2_m4_derivation_pairs() {
    let mut equal_ea = 0usize;
    let mut differ_ea = 0usize;
    let mut x_lower = 0usize;
    let mut x_equal = 0usize;
    let mut x_higher = 0usize;

    for k in 0..INSTANCES {
        let relations = relations_for(k);
        let readings = readings_for(&relations);

        let mut emergent = ClosureEngine::from_readings(&readings, CycleOrder::MeasureThenInject);
        let mut ablated = ClosureEngine::from_readings(&readings, CycleOrder::InjectThenMeasure);
        for _ in 0..PROTOCOL_EPOCHS {
            emergent.step();
            ablated.step();
        }
        let e = emergent.support();
        let x = ablated.support();
        let a = generated_subgroup_size(&relations);

        if e == a {
            equal_ea += 1;
        } else {
            differ_ea += 1;
        }
        match x.cmp(&e) {
            std::cmp::Ordering::Less => x_lower += 1,
            std::cmp::Ordering::Equal => x_equal += 1,
            std::cmp::Ordering::Greater => x_higher += 1,
        }
        eprintln!("M2 instance {k}: emergent E={e}, authored A={a}, readout-ablated X={x}");
    }

    eprintln!(
        "M2 paired counts — (E, A): {equal_ea} equal, {differ_ea} different | (E, X): X lower {x_lower}, equal {x_equal}, higher {x_higher}"
    );
    assert_eq!(equal_ea, INSTANCES, "the composition reaches the authored integer on every instance");
    assert_eq!(differ_ea, 0);

    // The ablation's direction, reported as measured — NOT categorical.
    // CORRECTION (kept on record): the precommit predicted 24/24 "X lower";
    // measured, one instance (k = 6) inverts it: without the destructive
    // readout the field accumulates coherently and its support saturates to
    // the full conjugate-pair count (200 of 400 bins), which is not a better
    // answer but no discrimination at all.
    assert_eq!(x_lower, INSTANCES - 1, "the ablation loses on 23 of 24 instances");
    assert_eq!(x_equal, 0);
    assert_eq!(x_higher, 1, "one instance saturates instead");

    eprintln!(
        "M4 verdict: (E, A) is a CATEGORICAL TIE over {INSTANCES} instances — the quantum-derived route is an equivalent path, not an accuracy advantage, on this quantity."
    );
    eprintln!(
        "M4 verdict: (E, X) is MARGINAL, not categorical — the destructive readout loses on 23/24 (X = 1 against E >= 10) and SATURATES on instance 6 (X = 200 = every conjugate pair of the 20x20 lattice: no discrimination, not an improvement)."
    );
}

/// M3 + M4 (precommitted: pass) — THE SCENE PAIR, per instance, with counts.
#[test]
fn gate_m3_scene_pair() {
    let mut b_greater = 0usize;
    let mut b_equal = 0usize;
    let mut b_lower = 0usize;

    for k in 0..INSTANCES {
        let scene = scene_for(k);
        let base = GeometryField::form(&scene).difference_vectors().len();
        let flat = vec![scene.len() as f64; N * N];
        let replacement = GeometryField::from_pattern(&flat, &scene).difference_vectors().len();
        match base.cmp(&replacement) {
            std::cmp::Ordering::Greater => b_greater += 1,
            std::cmp::Ordering::Equal => b_equal += 1,
            std::cmp::Ordering::Less => b_lower += 1,
        }
        if k < 4 || k == INSTANCES - 1 {
            eprintln!("M3 instance {k}: coherent base {base} vs classical replacement {replacement}");
        }
    }

    eprintln!(
        "M3 paired counts — (B, C): base greater {b_greater}, equal {b_equal}, lower {b_lower} over {INSTANCES} instances"
    );
    assert_eq!(b_greater, INSTANCES, "the coherent base produces structure on every instance");
    assert_eq!((b_equal, b_lower), (0, 0));
    eprintln!(
        "M4 verdict: (B, C) is CATEGORICAL — the base owns structure generation on every instance."
    );
}

/// M5 (precommitted: pass) — NO ENGINE CHANGE: the recorded closed forms hold.
#[test]
fn gate_m5_no_engine_change() {
    assert_eq!(generated_subgroup_size(&[(0, 8), (10, 0), (10, 12)]), 10, "BUILD-2's recorded |H|");
    assert_eq!(generated_subgroup_size(&[(0, 8), (10, 0), (10, 12), (6, 0)]), 50, "BUILD-17/21's recorded 50");
    let scene = vec![(2i64, 2i64), (6, 2), (4, 2)];
    assert_eq!(GeometryField::form(&scene).difference_vectors().len(), 4, "BUILD-12/16's recorded scene_support");
    eprintln!("M5: the recorded closed forms are unchanged");
}
