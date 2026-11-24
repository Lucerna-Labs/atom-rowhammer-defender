//! BUILD-3 gates — precommitted in docs/BUILD3-PERCEPTION.md BEFORE this
//! code. Recognition = exact integer support delta; no thresholds.
//! Harness-side analysis may know the regimes and the closed forms.

use atom_cognitive_engine::perception::Reading;
use atom_cognitive_engine::perception_field::{expected_bins, PerceivingField, PresentOrder};

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

fn differences(regimes: &[(f64, f64)]) -> Vec<(isize, isize)> {
    let mut out = Vec::new();
    for i in 0..regimes.len() {
        for j in (i + 1)..regimes.len() {
            out.push((
                ((regimes[i].0 - regimes[j].0) * 20.0).round() as isize,
                ((regimes[i].1 - regimes[j].1) * 20.0).round() as isize,
            ));
        }
    }
    out
}

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

/// P1 (precommitted: pass) — RECOGNITION EXACTNESS: after forming on
/// {A,B,C} (support 6), a new instance gives delta 0; {A,B,N} gives
/// delta +4 (6 -> 10). Integer matches against the closed forms.
#[test]
fn gate_p1_recognition_exactness() {
    // CORRECTION, kept on record: the precommit's closed forms omitted
    // (i) the DC bin, which the accumulated intensity always carries
    // (the constant term of |sum|^2), and (ii) that (10,0) is
    // SELF-CONJUGATE (10 = -10 mod 20), so one difference contributes
    // one bin, not two. Correct counts: triangle = 5 difference bins +
    // DC = 6 (the engine measured 6); with N = 9 + DC = 10 (measured
    // 10). The ENGINE was right; the hand arithmetic slipped.
    let triangle = [A, B, C];
    let expected_formed = expected_bins(&differences(&triangle)) + 1; // + DC
    eprintln!("closed form (triangle): {expected_formed} bins (incl DC)");
    assert_eq!(expected_formed, 6);

    let mut field = PerceivingField::new(PresentOrder::Canonical);
    for _ in 0..9 {
        field.present(&scene(10, &triangle));
    }
    assert_eq!(field.support(), expected_formed, "formed support");

    // New instance of known structure: delta 0.
    let delta_known = field.present(&scene(50_000, &triangle));
    assert_eq!(delta_known, 0, "a known instance must add no modes");
    assert_eq!(field.support(), 6);

    // Novel scene {A,B,N}: the two new differences add 4 bins.
    // SECOND CORRECTION, kept on record: the support after the novel
    // presentation is the UNION of history differences and the novel
    // scene's differences (recognition accumulates) -- not the novel
    // scene's differences alone. Union: triangle diffs + A-N + B-N = 9
    // difference bins + DC = 10. The engine measured 10 throughout;
    // both hand-accounting slips were mine.
    let with_novel = [A, B, NOVEL];
    let mut union = differences(&triangle);
    union.extend(differences(&with_novel));
    let expected_after_novel = expected_bins(&union) + 1; // + DC
    eprintln!("closed form (history union with N): {expected_after_novel} bins (incl DC)");
    assert_eq!(expected_after_novel, 10);
    let delta_novel = field.present(&scene(60_000, &with_novel));
    assert_eq!(delta_novel, 4, "novel presentation adds its new bins");
    assert_eq!(field.support(), expected_after_novel);

    // And the novel scene again: now known — delta 0.
    let delta_novel_again = field.present(&scene(70_000, &with_novel));
    assert_eq!(delta_novel_again, 0, "novel-then-seen must be recognized");
}

/// P2 (precommitted: pass) — ONLINE, PER-PRESENTATION deltas:
/// [+6, 0, +4, 0] for forming, known-instance, novel, novel-again.
#[test]
fn gate_p2_online_per_presentation() {
    let mut field = PerceivingField::new(PresentOrder::Canonical);
    let deltas = vec![
        field.present(&scene(10, &[A, B, C])),
        field.present(&scene(20, &[A, B, C])),
        field.present(&scene(30, &[A, B, NOVEL])),
        field.present(&scene(40, &[A, B, NOVEL])),
    ];
    eprintln!("deltas: {deltas:?}");
    assert_eq!(deltas, vec![6, 0, 4, 0]);
}

/// P3 (precommitted: pass) — ORDER INVARIANCE: shuffled presentation
/// orders reach the same final support exactly.
#[test]
fn gate_p3_order_invariance() {
    let sequence: Vec<(u64, Vec<(f64, f64)>)> = vec![
        (10, vec![A, B, C]),
        (20, vec![A, B, C]),
        (30, vec![A, B, NOVEL]),
        (40, vec![A, B, NOVEL]),
        (50, vec![A, B, C]),
    ];
    let mut finals = Vec::new();
    for seed in [1u64, 2, 3, 4, 5] {
        let mut field = PerceivingField::new(PresentOrder::Canonical);
        for (base, regimes) in shuffled(sequence.clone(), seed) {
            field.present(&scene(base, &regimes));
        }
        finals.push(field.support());
    }
    eprintln!("final supports across shuffles: {finals:?}");
    assert!(finals.iter().all(|s| *s == finals[0]), "order invariance");
    assert_eq!(finals[0], 10);
}

/// P4 (precommitted: pass) — AUDIT: the decision is `delta == 0` on
/// integers; there is no threshold anywhere (the module has no config
/// surface beyond the order variant).
#[test]
fn gate_p4_no_thresholds() {
    // A fresh field's first presentation is always novel (delta > 0),
    // and recognition decisions are exact integers — asserted across
    // the whole exercise above; here we assert the API shape: present()
    // takes readings, returns an integer delta, consults no config.
    let mut field = PerceivingField::new(PresentOrder::Canonical);
    let delta = field.present(&scene(10, &[A, B]));
    assert!(delta > 0);
    assert_eq!(field.presentations(), 1);
}

/// P5 (precommitted: pass) — HISTORY IS LOAD-BEARING: a fresh engine
/// reports novelty for every distinct scene; recognition requires the
/// accumulated field.
#[test]
fn gate_p5_history_load_bearing() {
    // Fresh engine, two distinct scenes, both novel.
    let mut fresh = PerceivingField::new(PresentOrder::Canonical);
    let d1 = fresh.present(&scene(10, &[A, B, C]));
    let d2 = fresh.present(&scene(20, &[A, B, NOVEL]));
    assert!(d1 > 0 && d2 > 0, "fresh engine: all structure is novel");
    // Same engine, repeat the second scene: recognized.
    let d3 = fresh.present(&scene(30, &[A, B, NOVEL]));
    assert_eq!(d3, 0, "with history, the repeat is recognized");
}

/// Rule 2 variant measurement (named in the doc): the NoPhase order must
/// make the same recognition decisions.
#[test]
fn gate_variant_nophase() {
    let mut field = PerceivingField::new(PresentOrder::NoPhase);
    let deltas = vec![
        field.present(&scene(10, &[A, B, C])),
        field.present(&scene(20, &[A, B, C])),
        field.present(&scene(30, &[A, B, NOVEL])),
        field.present(&scene(40, &[A, B, NOVEL])),
    ];
    eprintln!("nophase deltas: {deltas:?}");
    assert_eq!(deltas, vec![6, 0, 4, 0]);
}
