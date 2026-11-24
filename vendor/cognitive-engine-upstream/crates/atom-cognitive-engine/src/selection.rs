//! BUILD-14 — selection: relaxation over consequence-accumulated structure.
//! Precommitted in `docs/BUILD14-SELECTION-COMPOSITE.md` BEFORE this code,
//! under vault Rules 1/2/3/4 and project rules 0/0a/0b.
//!
//! The composite BUILD-7 named and did not build: BUILD-5's consequence
//! accumulation, lifted phase-free into amplitudes, relaxed by BUILD-4's
//! shared operator. Selection = the set consistent with EVERY recorded
//! consequence — the kernel of the recorded differences.
//!
//! AUDIT (the only numbers): N (substrate), the unit tick (the shared
//! relaxation step), the landscape's own maximum (self-scaling exponent),
//! and structural budgets (epochs, steps, seeds). No threshold, no scoring
//! function, no target; K, k and participation are measurements.

use crate::action_world::{ActionLoopEngine, ActionPolicy, World};
use crate::interference_field::N;
use crate::preference::{relax_field, RelaxOperator};
use crate::weave::Cx;

/// Accumulation arm under measurement (Rule 4 controls). `FullStack` is the
/// atom stack: the interference of action + world accumulates. The
/// `IntensityAddition` arm is the no-base control — BUILD-1's NoInterference
/// arm mirrored: each epoch's two waves are added as intensities (|psi|^2 =
/// 1 per wave per site, no interference), so the pattern is flat and there
/// is nothing to select. An ablation arm, not a build.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AccumArm {
    FullStack,
    IntensityAddition,
}

/// The composite: BUILD-5's action loop; its accumulated pattern lifted
/// phase-free to amplitudes (DECLARATION, structural: psi_s = sqrt(I_s) —
/// the accumulated data carries no phase of its own); then BUILD-4's shared
/// relaxation operator ticked at the unit step.
pub struct SelectionComposite {
    engine: ActionLoopEngine,
    field: Vec<Cx>,
    arm: AccumArm,
    epochs: usize,
}

impl SelectionComposite {
    pub fn new(world: World, seed: u64, arm: AccumArm) -> Self {
        SelectionComposite {
            engine: ActionLoopEngine::new(world, seed),
            field: vec![Cx::new(0.0, 0.0); N * N],
            arm,
            epochs: 0,
        }
    }

    /// E epochs of the action loop (BUILD-5, unchanged); the field is
    /// re-derived from the accumulated pattern afterwards (the lift).
    pub fn learn(&mut self, policy: ActionPolicy, epochs: usize) {
        for _ in 0..epochs {
            self.engine.epoch(policy, true);
        }
        self.epochs += epochs;
        self.rebuild();
    }

    /// The interleaved arm (recorded with its prediction in the precommit):
    /// after every epoch the field is re-derived from the pattern accumulated
    /// so far and relaxed `steps` unit ticks.
    pub fn learn_interleaved(
        &mut self,
        policy: ActionPolicy,
        epochs: usize,
        steps: usize,
        operator: RelaxOperator,
    ) {
        for _ in 0..epochs {
            self.engine.epoch(policy, true);
            self.epochs += 1;
            self.rebuild();
            self.relax(steps, operator);
        }
    }

    fn rebuild(&mut self) {
        match self.arm {
            AccumArm::FullStack => {
                self.field = self
                    .engine
                    .pattern_snapshot()
                    .into_iter()
                    .map(|i| Cx::new(i.max(0.0).sqrt(), 0.0))
                    .collect();
            }
            AccumArm::IntensityAddition => {
                // Two waves per epoch, |psi|^2 = 1 each, added directly ->
                // a flat pattern (no interference -> no structure).
                let flat = 2.0 * self.epochs as f64;
                self.field = vec![Cx::new(flat.sqrt(), 0.0); N * N];
            }
        }
    }

    /// R unit ticks of the shared relaxation operator.
    pub fn relax(&mut self, steps: usize, operator: RelaxOperator) {
        for _ in 0..steps {
            relax_field(&mut self.field, operator);
        }
    }

    /// Born participation 1/sum p^2 of the current field.
    pub fn participation(&self) -> f64 {
        field_participation(&self.field)
    }

    /// The top-n sites by Born probability (harness readout).
    pub fn top_sites(&self, n: usize) -> Vec<(usize, usize)> {
        top_sites(&self.field, n)
    }

    pub fn actions(&self) -> &[(f64, f64)] {
        self.engine.actions()
    }

    /// The accumulated pattern (harness readout).
    pub fn accumulated(&self) -> Vec<f64> {
        self.engine.pattern_snapshot()
    }
}

/// Born participation of a complex field.
pub fn field_participation(field: &[Cx]) -> f64 {
    let total: f64 = field.iter().map(|v| v.re * v.re + v.im * v.im).sum();
    if total <= 0.0 {
        return 0.0;
    }
    let sumsq: f64 = field
        .iter()
        .map(|v| {
            let p = (v.re * v.re + v.im * v.im) / total;
            p * p
        })
        .sum();
    1.0 / sumsq
}

/// Born participation of an intensity pattern — equal to the lifted field's
/// participation before relaxation (the lift is phase-free: |psi|^2 = I).
pub fn intensity_participation(pattern: &[f64]) -> f64 {
    let total: f64 = pattern.iter().sum();
    if total <= 0.0 {
        return 0.0;
    }
    let sumsq: f64 = pattern.iter().map(|v| v * v).sum();
    total * total / sumsq
}

/// Top-n sites by Born probability of a field.
pub fn top_sites(field: &[Cx], n: usize) -> Vec<(usize, usize)> {
    let mut sites: Vec<(f64, usize, usize)> = field
        .iter()
        .enumerate()
        .map(|(index, v)| (v.re * v.re + v.im * v.im, index % N, index / N))
        .collect();
    sites.sort_by(|a, b| b.0.partial_cmp(&a.0).expect("finite"));
    sites.into_iter().take(n).map(|(_, x, y)| (x, y)).collect()
}

/// Harness closed form (integer, exact): the evidence kernel — every site s
/// where EVERY recorded difference has phase 0, i.e. `d·s ≡ 0 (mod N)`.
/// This is the argmax set of the accumulated pattern: the sum of cosines
/// attains its maximum iff every term is maximal.
pub fn kernel_of_differences(actions: &[(f64, f64)], world: (f64, f64)) -> Vec<(usize, usize)> {
    let modulus = N as isize;
    let mut out = Vec::new();
    for y in 0..N {
        for x in 0..N {
            let consistent = actions.iter().all(|(ax, ay)| {
                let dx = ((ax - world.0) * N as f64).round() as isize;
                let dy = ((ay - world.1) * N as f64).round() as isize;
                (dx * x as isize + dy * y as isize) % modulus == 0
            });
            if consistent {
                out.push((x, y));
            }
        }
    }
    out
}

/// Harness closed form: k = |K| — the selection's size. Equal to N^2 / |H|,
/// the index of the subgroup generated by the recorded differences (BUILD-2's
/// `generated_subgroup_size`); the tests assert the two computations agree.
pub fn expected_k(actions: &[(f64, f64)], world: (f64, f64)) -> usize {
    kernel_of_differences(actions, world).len()
}
