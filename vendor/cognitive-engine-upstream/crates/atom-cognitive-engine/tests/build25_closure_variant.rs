//! BUILD-25 gates — the closure variant (BUILD-3's deferred Rule-2 arm).
//! Predictions were verified in Python before the precommit: the triangle's
//! closure is 10 (== BUILD-2's measured |H| for the same scene), the novel
//! scene's closure is 50, and the P2 deltas are [10, 0, 40, 0].

use atom_cognitive_engine::perception::Reading;
use atom_cognitive_engine::perception_field::{PerceivingField, PresentOrder};

fn scene(base_id: u64, regimes: &[(f64, f64)]) -> Vec<Reading> {
    let mut out = Vec::new();
    let mut id = base_id;
    for (exploration, convergence) in regimes {
        for _ in 0..5u64 {
            out.push(Reading::new(id, 0.5, 0.5, 0.5, *exploration, *convergence, 0.5));
            id += 1;
        }
    }
    out
}

const A: (f64, f64) = (0.80, 0.70);
const B: (f64, f64) = (0.80, 0.30);
const C: (f64, f64) = (0.30, 0.70);
const NOVEL: (f64, f64) = (0.50, 0.50);

fn shuffled<T>(items: Vec<T>, seed: u64) -> Vec<T> {
    let mut state = seed | 1;
    let mut items = items;
    for i in (1..items.len()).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let j = (state >> 33) as usize % (i + 1);
        items.swap(i, j);
    }
    items
}

/// V1 (precommitted: pass) — THE P2 SEQUENCE: deltas [10, 0, 40, 0];
/// supports after each [10, 10, 50, 50].
#[test]
fn gate_v1_p2_deltas_exact() {
    let mut field = PerceivingField::new(PresentOrder::FeedbackClosure);
    let d1 = field.present(&scene(10, &[A, B, C]));
    let s1 = field.support();
    let d2 = field.present(&scene(20, &[A, B, C]));
    let s2 = field.support();
    let d3 = field.present(&scene(30, &[A, B, NOVEL]));
    let s3 = field.support();
    let d4 = field.present(&scene(40, &[A, B, NOVEL]));
    let s4 = field.support();
    eprintln!("V1: deltas {:?} supports {:?}", [d1, d2, d3, d4], [s1, s2, s3, s4]);
    assert_eq!([d1, d2, d3, d4], [10, 0, 40, 0], "the predicted deltas");
    assert_eq!([s1, s2, s3, s4], [10, 10, 50, 50], "the predicted supports");
}

/// V2 (precommitted: pass) — THE P1 SEQUENCE on the variant arm.
#[test]
fn gate_v2_p1_sequence() {
    let mut field = PerceivingField::new(PresentOrder::FeedbackClosure);
    for _ in 0..9 {
        field.present(&scene(10, &[A, B, C]));
    }
    assert_eq!(field.support(), 10, "formed support = the triangle's closure");
    assert_eq!(
        field.present(&scene(50_000, &[A, B, C])),
        0,
        "a known instance adds nothing to the closure union"
    );
    assert_eq!(
        field.present(&scene(60_000, &[A, B, NOVEL])),
        40,
        "the novel scene's closure adds its complement"
    );
    assert_eq!(field.present(&scene(70_000, &[A, B, NOVEL])), 0, "novel-again");
    assert_eq!(field.support(), 50);
    eprintln!("V2: formed 10; known 0; novel +40; novel-again 0; final 50");
}

/// V3 (precommitted: pass) — THE COMPARISON, recorded: same decisions,
/// coarser magnitudes (the measured trade of composing BUILD-2).
#[test]
fn gate_v3_comparison_recorded() {
    let mut canonical = PerceivingField::new(PresentOrder::Canonical);
    let c = [
        canonical.present(&scene(10, &[A, B, C])),
        canonical.present(&scene(20, &[A, B, C])),
        canonical.present(&scene(30, &[A, B, NOVEL])),
        canonical.present(&scene(40, &[A, B, NOVEL])),
    ];
    let mut closure = PerceivingField::new(PresentOrder::FeedbackClosure);
    let v = [
        closure.present(&scene(10, &[A, B, C])),
        closure.present(&scene(20, &[A, B, C])),
        closure.present(&scene(30, &[A, B, NOVEL])),
        closure.present(&scene(40, &[A, B, NOVEL])),
    ];
    eprintln!("V3: canonical {c:?} vs closure-variant {v:?}");
    assert_eq!(c, [6, 0, 4, 0], "the canonical arm is untouched");
    assert_eq!(v, [10, 0, 40, 0]);
    for (a, b) in c.iter().zip(v.iter()) {
        assert_eq!(*a == 0, *b == 0, "identical recognition decisions");
    }
}

/// V4 (precommitted: pass) — ORDER INVARIANCE: every shuffle reaches the
/// same final support (the closure union is a set union).
#[test]
fn gate_v4_order_invariance() {
    let sequence: Vec<(u64, Vec<(f64, f64)>)> = vec![
        (10, vec![A, B, C]),
        (20, vec![A, B, C]),
        (30, vec![A, B, NOVEL]),
        (40, vec![A, B, NOVEL]),
        (50, vec![A, B, C]),
    ];
    let mut finals = Vec::new();
    for seed in [1u64, 2, 3, 4, 5] {
        let mut field = PerceivingField::new(PresentOrder::FeedbackClosure);
        for (base, regimes) in shuffled(sequence.clone(), seed) {
            field.present(&scene(base, &regimes));
        }
        finals.push(field.support());
    }
    eprintln!("V4: finals {finals:?}");
    assert!(finals.iter().all(|s| *s == finals[0]), "order invariance");
    assert_eq!(finals[0], 50);
}

/// V5 (precommitted: pass) — HISTORY IS LOAD-BEARING on the variant arm.
#[test]
fn gate_v5_history_load_bearing() {
    let mut fresh = PerceivingField::new(PresentOrder::FeedbackClosure);
    let d1 = fresh.present(&scene(10, &[A, B, C]));
    let d2 = fresh.present(&scene(20, &[A, B, NOVEL]));
    assert!(d1 > 0 && d2 > 0, "fresh engine: all structure is novel");
    assert_eq!(
        fresh.present(&scene(30, &[A, B, NOVEL])),
        0,
        "with history, the repeat is recognized"
    );
}

/// V6 (precommitted: pass) — AUDIT: the variant's own accumulation is an
/// integer bin set (no epsilon); decisions are `delta == 0`.
#[test]
fn gate_v6_audit() {
    let mut field = PerceivingField::new(PresentOrder::FeedbackClosure);
    assert_eq!(field.support(), 0, "an empty closure union");
    field.present(&scene(10, &[A, B]));
    assert!(field.support() > 0);
    eprintln!(
        "V6: integer bin sets; decisions are delta == 0; the epsilon is BUILD-2's shared classification"
    );
}
