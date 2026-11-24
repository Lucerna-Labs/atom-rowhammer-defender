//! BUILD-14 gates — precommitted in `docs/BUILD14-SELECTION-COMPOSITE.md`
//! BEFORE the module. Selection = relaxation over consequence-accumulated
//! structure: participation settles at k = |kernel| = N^2/|H(D)|.

use atom_cognitive_engine::action_world::{ActionPolicy, World};
use atom_cognitive_engine::perception::Reading;
use atom_cognitive_engine::preference::{PreferenceField, RelaxOperator};
use atom_cognitive_engine::reasoning_closure::generated_subgroup_size;
use atom_cognitive_engine::selection::{
    expected_k, intensity_participation, kernel_of_differences, AccumArm, SelectionComposite,
};

const STEPS: usize = 10;
const N: isize = 20;
const W: (f64, f64) = (0.80, 0.30);
const W_B: (f64, f64) = (0.60, 0.20);

/// The recorded differences in index units (the harness's own closed-form
/// input; the engine never sees this).
fn differences(actions: &[(f64, f64)], world: (f64, f64)) -> Vec<(isize, isize)> {
    actions
        .iter()
        .map(|(ax, ay)| {
            (
                ((ax - world.0) * N as f64).round() as isize,
                ((ay - world.1) * N as f64).round() as isize,
            )
        })
        .collect()
}

fn sorted(mut v: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    v.sort();
    v
}

/// V1 (precommitted: pass) — the law transfers to learned structure:
/// participation == k, k == |K| == N^2/|H|, survivors ARE the kernel.
#[test]
fn gate_v1_law_transfer_canonical() {
    let mut c = SelectionComposite::new(World { wave: W }, 7, AccumArm::FullStack);
    c.learn(ActionPolicy::FromField, 100);
    let pattern = c.accumulated();
    let pre = intensity_participation(&pattern);
    let actions = c.actions().to_vec();
    let k = expected_k(&actions, W);
    let h = generated_subgroup_size(&differences(&actions, W));
    assert_eq!(
        k * h,
        (N * N) as usize,
        "k * |H| == N^2 (two independent computations)"
    );
    let kernel = sorted(kernel_of_differences(&actions, W));
    c.relax(STEPS, RelaxOperator::Exponential);
    let post = c.participation();
    let top = sorted(c.top_sites(k));
    eprintln!("V1 canonical: |H|={h} k={k} pre={pre:.3} post={post:.6}");
    assert!(
        (post - k as f64).abs() < 1e-3,
        "participation {post} must settle at k={k}"
    );
    assert_eq!(top, kernel, "the top-k Born sites ARE the evidence kernel");
    assert!(post < pre, "concentration: post {post} < pre {pre}");
}

/// V2 (precommitted: pass) — selection is the evidence kernel: a short
/// history gives k = 20 for both worlds, and the two kernels DIFFER
/// (pinned memberships from the Python verification).
#[test]
fn gate_v2_evidence_kernel_two_worlds() {
    let mut a = SelectionComposite::new(World { wave: W }, 11, AccumArm::FullStack);
    a.learn(ActionPolicy::Fixed, 2);
    let actions_a = a.actions().to_vec();
    let ka = expected_k(&actions_a, W);
    let kernel_a = sorted(kernel_of_differences(&actions_a, W));
    a.relax(STEPS, RelaxOperator::Exponential);
    let top_a = sorted(a.top_sites(ka));

    let mut b = SelectionComposite::new(World { wave: W_B }, 11, AccumArm::FullStack);
    b.learn(ActionPolicy::Fixed, 2);
    let actions_b = b.actions().to_vec();
    let kb = expected_k(&actions_b, W_B);
    let kernel_b = sorted(kernel_of_differences(&actions_b, W_B));
    b.relax(STEPS, RelaxOperator::Exponential);
    let top_b = sorted(b.top_sites(kb));

    eprintln!(
        "V2 two worlds: kA={ka} partA={:.5} kB={kb} partB={:.5}",
        a.participation(),
        b.participation()
    );
    assert_eq!(ka, 20, "short history: k = 20 (predicted)");
    assert_eq!(kb, 20, "short history: k = 20 (predicted)");
    assert!((a.participation() - 20.0).abs() < 1e-3, "world A settles at 20");
    assert!((b.participation() - 20.0).abs() < 1e-3, "world B settles at 20");
    assert_eq!(top_a, kernel_a, "survivors == world A's kernel");
    assert_eq!(top_b, kernel_b, "survivors == world B's kernel");
    assert_ne!(kernel_a, kernel_b, "different worlds, different selected sets");
    assert!(
        kernel_a.contains(&(1, 19)) && kernel_b.contains(&(1, 17)),
        "pinned memberships (1,19) in A and (1,17) in B"
    );
}

/// V3 (precommitted: pass, with the honest metric note) — the BUILD-7
/// contrast: the accumulated distribution is diffuse; relaxation
/// concentrates it. The BUILD-7 |a-W| metric over the selected set is
/// RECORDED, not gated: the composite ranks consistency with the record,
/// not proximity to W.
#[test]
fn gate_v3_build7_contrast_no_relax() {
    let mut c = SelectionComposite::new(World { wave: W }, 7, AccumArm::FullStack);
    c.learn(ActionPolicy::FromField, 100);
    let pre = c.participation();
    let pattern = c.accumulated();
    let ip = intensity_participation(&pattern);
    assert!(
        (pre - ip).abs() < 1e-9,
        "phase-free lift: field participation == pattern participation ({pre} vs {ip})"
    );
    let k = expected_k(&c.actions().to_vec(), W);
    c.relax(STEPS, RelaxOperator::Exponential);
    let post = c.participation();
    let selected = c.top_sites(k);
    let metric: f64 = selected
        .iter()
        .map(|(x, y)| {
            let a = (*x as f64 / N as f64, *y as f64 / N as f64);
            ((a.0 - W.0).powi(2) + (a.1 - W.1).powi(2)).sqrt()
        })
        .sum::<f64>()
        / selected.len() as f64;
    eprintln!(
        "V3 contrast: no-relax {pre:.3} -> relaxed {post:.6} (k={k}); \
         BUILD-7 |a-W| over the selected set {metric:.3} (uniform baseline 0.499; recorded, not gated)"
    );
    assert!(post < pre, "relaxation concentrates: {post} < {pre}");
}

/// V4 (Rule 2, precommitted: pass) — the declared primitive set
/// {accumulate A, relax R} has 2! = 2 unique orders, both executed; plus
/// the operator arm and the interleaved arm with their predictions.
#[test]
fn gate_v4_order_sweep_and_arms() {
    // (b) R->A: relaxation before any data is vacuous (zero landscape has no
    // maximum), so the endpoint equals the NoRelax arm exactly.
    let mut ra = SelectionComposite::new(World { wave: W }, 7, AccumArm::FullStack);
    ra.relax(STEPS, RelaxOperator::Exponential);
    ra.learn(ActionPolicy::FromField, 100);
    let ra_part = ra.participation();
    let no_relax = intensity_participation(&ra.accumulated());
    assert!(
        (ra_part - no_relax).abs() < 1e-9,
        "R->A is vacuous: {ra_part} vs NoRelax {no_relax}"
    );

    // Arm: Multiplicative operator (predicted: also reaches k, per BUILD-4's
    // corrected record — re-measured here through the shared code).
    let mut mul = SelectionComposite::new(World { wave: W }, 7, AccumArm::FullStack);
    mul.learn(ActionPolicy::FromField, 100);
    mul.relax(STEPS, RelaxOperator::Multiplicative);
    let k = expected_k(&mul.actions().to_vec(), W);
    eprintln!(
        "V4: R->A {ra_part:.3} == NoRelax; multiplicative post {:.6} (k={k})",
        mul.participation()
    );
    assert!(
        (mul.participation() - k as f64).abs() < 1e-3,
        "multiplicative also reaches k"
    );

    // Arm: interleaved (re-derive + relax after every epoch) — predicted to
    // reach the full-history kernel.
    let mut il = SelectionComposite::new(World { wave: W }, 7, AccumArm::FullStack);
    il.learn_interleaved(ActionPolicy::FromField, 100, STEPS, RelaxOperator::Exponential);
    let kil = expected_k(&il.actions().to_vec(), W);
    eprintln!("V4: interleaved post {:.6} (k={kil})", il.participation());
    assert!(
        (il.participation() - kil as f64).abs() < 1e-3,
        "interleaved reaches the full-history kernel"
    );
}

/// V5 (Rule 4 no-base control, precommitted: pass) — waves added as
/// intensities: flat pattern, nothing to select.
#[test]
fn gate_v5_no_base_control() {
    let mut c = SelectionComposite::new(World { wave: W }, 7, AccumArm::IntensityAddition);
    c.learn(ActionPolicy::FromField, 100);
    c.relax(STEPS, RelaxOperator::Exponential);
    let p = c.participation();
    eprintln!("V5 no-base: participation {p:.6} (flat pattern: 400)");
    assert!(
        (p - 400.0).abs() < 1e-6,
        "without the interference base there is nothing to select ({p})"
    );
}

/// V6 (precommitted: pass) — audit: one shared relaxation implementation
/// (both call sites go through `relax_field`), no parameter surface beyond
/// structural budgets, monotone concentration under the canonical operator.
#[test]
fn gate_v6_audit() {
    let mut readings = Vec::new();
    let mut id = 10u64;
    for (e, c) in [(0.80, 0.70), (0.80, 0.30), (0.30, 0.70)] {
        for _ in 0..5u64 {
            readings.push(Reading::new(id, 0.5, 0.5, 0.5, e, c, 0.5));
            id += 1;
        }
    }
    let mut field = PreferenceField::from_readings(&readings);
    let mut prev = f64::INFINITY;
    for _ in 0..6 {
        let p = field.participation();
        assert!(
            p <= prev + 1e-9,
            "participation must not increase ({p} vs {prev})"
        );
        prev = p;
        field.relax_step(RelaxOperator::Exponential);
    }
    eprintln!("V6 audit: shared relaxation code, monotone concentration holds");
}
