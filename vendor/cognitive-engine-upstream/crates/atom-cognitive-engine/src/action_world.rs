//! BUILD-5 — action and consequence. Precommitted in
//! `docs/BUILD5-ACTION.md` BEFORE this code, under vault Rules 1/2/3
//! and project rules 0/0a/0b.
//!
//! The engine acts (Born-sampled wavevector from its own field), the
//! world answers with the superposition of the action and the world's
//! hidden wave, and the response's intensity accumulates. No reward is
//! authored anywhere: interference is the only coupling.
//!
//! AUDIT: the world's wave is ENVIRONMENT INPUT (not authored behavior);
//! the Born sampler, the superposition and the accumulation are
//! mechanism. No threshold, no reward term.

use crate::causal_path::{CausalGraph, Node};
use crate::interference_field::{born_intensity, N};
use crate::weave::{Cx, Rng};

/// The difference of an action from the world's wave, in index units —
/// the loop's own law, shared by the closed form, the recorder, and the
/// gates (BUILD-28: one implementation).
pub fn difference_of(action: (f64, f64), world: (f64, f64)) -> (i64, i64) {
    (
        ((action.0 - world.0) * N as f64).round() as i64,
        ((action.1 - world.1) * N as f64).round() as i64,
    )
}

/// The bins a difference writes: the difference and its conjugate on the
/// lattice, skipping `(0,0)` (DC only) and collapsing the self-conjugate
/// case (`2d = 0` mod N) to one bin.
pub fn bins_of_difference(d: (i64, i64)) -> Vec<(i64, i64)> {
    let modulus = N as i64;
    let mut bins: Vec<(i64, i64)> = Vec::new();
    for (sx, sy) in [
        (d.0.rem_euclid(modulus), d.1.rem_euclid(modulus)),
        ((-d.0).rem_euclid(modulus), (-d.1).rem_euclid(modulus)),
    ] {
        if (sx, sy) != (0, 0) && !bins.contains(&(sx, sy)) {
            bins.push((sx, sy));
        }
    }
    bins
}

/// The frequency bins of an accumulated pattern — the classification
/// `support()` counts, returned as the set so the recorder can name the
/// substrate's structure.
fn pattern_bins(pattern: &[f64]) -> Vec<(i64, i64)> {
    let mut spectrum = vec![0.0f64; N * N];
    for vy in 0..N {
        for vx in 0..N {
            let mut acc = 0.0f64;
            for y in 0..N {
                for x in 0..N {
                    let phase = -2.0
                        * core::f64::consts::PI
                        * (vx as f64 * x as f64 + vy as f64 * y as f64)
                        / N as f64;
                    acc += pattern[y * N + x] * phase.cos();
                }
            }
            spectrum[vy * N + vx] = acc * acc;
        }
    }
    let max = spectrum.iter().copied().fold(0.0f64, f64::max);
    let mut bins = Vec::new();
    if max <= 0.0 {
        return bins;
    }
    let epsilon = max * 1e-18;
    for vy in 0..N {
        for vx in 0..N {
            if spectrum[vy * N + vx] > epsilon {
                bins.push((vx as i64, vy as i64));
            }
        }
    }
    bins
}

/// The environment: one hidden wave. Input, not authored behavior.
#[derive(Clone, Copy, Debug)]
pub struct World {
    pub wave: (f64, f64),
}

/// Action policy under sweep (Rule 2).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActionPolicy {
    /// Canonical: Born-sampled from the engine's own accumulated field
    /// (uniform when the field is empty — the initial condition).
    FromField,
    /// Variant: uniform sampling regardless of the field.
    Uniform,
    /// Variant: a fixed action every epoch (the degenerate control).
    Fixed,
}

/// The action loop engine.
pub struct ActionLoopEngine {
    pattern: Vec<f64>,
    actions: Vec<(f64, f64)>,
    rng: Rng,
    world: World,
    /// BUILD-28: the loop records its own arrows — `Action → Content` for
    /// the bins an epoch writes, `Content → Action` for the substrate an
    /// action was sampled from. Nothing here is authored; the graph is a
    /// record of what the run did.
    causal: CausalGraph,
}

impl ActionLoopEngine {
    pub fn new(world: World, seed: u64) -> Self {
        ActionLoopEngine {
            pattern: vec![0.0; N * N],
            actions: Vec::new(),
            rng: Rng::new(seed),
            world,
            causal: CausalGraph::new(),
        }
    }

    /// One perception–action–consequence epoch; returns the action taken
    /// (recorded — an observable the engine emits).
    pub fn epoch(&mut self, policy: ActionPolicy, learn: bool) -> (f64, f64) {
        let epoch = self.actions.len();
        // The sampling substrate as it stands BEFORE this epoch's
        // accumulation — recorded only when the policy consults the field
        // and the field carries structure (BUILD-28).
        let substrate: Option<Vec<(i64, i64)>> = match policy {
            ActionPolicy::FromField => {
                let total: f64 = self.pattern.iter().sum();
                if total > 0.0 {
                    Some(pattern_bins(&self.pattern))
                } else {
                    None
                }
            }
            _ => None,
        };
        // Action in INDEX space (on the lattice grid); the wavevector is
        // index / N (cycles/site, BUILD-1 interface) — on-grid actions
        // keep the closed form integral, verified in Python.
        let (ix, iy): (usize, usize) = match policy {
            ActionPolicy::FromField => {
                let total: f64 = self.pattern.iter().sum();
                if total <= 0.0 {
                    (
                        (self.rng.unit() * N as f64) as usize % N,
                        (self.rng.unit() * N as f64) as usize % N,
                    )
                } else {
                    let mut pick = self.rng.unit() * total;
                    let mut index = N * N - 1;
                    for (i, value) in self.pattern.iter().enumerate() {
                        pick -= value;
                        if pick <= 0.0 {
                            index = i;
                            break;
                        }
                    }
                    (index % N, index / N)
                }
            }
            ActionPolicy::Uniform => (
                (self.rng.unit() * N as f64) as usize % N,
                (self.rng.unit() * N as f64) as usize % N,
            ),
            ActionPolicy::Fixed => (5, 15),
        };
        let action = (ix as f64 / N as f64, iy as f64 / N as f64);
        self.actions.push(action);
        // THE ARROWS: recorded by the epoch that produced them.
        let action_node = Node::action((ix as i64, iy as i64));
        if let Some(bins) = substrate {
            for bin in bins {
                self.causal.record(Node::content(bin), action_node, epoch);
            }
        }
        if learn {
            for bin in bins_of_difference(difference_of(action, self.world.wave)) {
                self.causal.record(action_node, Node::content(bin), epoch);
            }
            // The response: action wave + world wave, superposed; the
            // Born readout of the response accumulates as consequence.
            let response = self.response_field(action);
            let intensity = born_intensity(&response);
            for (site, value) in self.pattern.iter_mut().zip(intensity) {
                *site += value;
            }
        }
        action
    }

    /// BUILD-28: the recorded arrows of this run (`&self` — reading the
    /// graph cannot change the loop, and nothing in the loop consults it).
    pub fn causal_graph(&self) -> &CausalGraph {
        &self.causal
    }

    /// The world's answer: action wave + hidden wave (cycles/site
    /// interface, BUILD-1 declaration).
    fn response_field(&self, action: (f64, f64)) -> Vec<Cx> {
        let mut field = vec![Cx::new(0.0, 0.0); N * N];
        for y in 0..N {
            for x in 0..N {
                let mut sum = Cx::new(0.0, 0.0);
                for (kx, ky) in [action, self.world.wave] {
                    let phase = 2.0 * core::f64::consts::PI * (kx * x as f64 + ky * y as f64);
                    sum.re += phase.cos();
                    sum.im += phase.sin();
                }
                field[y * N + x] = sum;
            }
        }
        field
    }

    pub fn actions(&self) -> &[(f64, f64)] {
        &self.actions
    }

    /// The accumulated pattern (harness readout) — the structure the loop
    /// learned through consequence (BUILD-14 lifts this into amplitudes).
    pub fn pattern_snapshot(&self) -> Vec<f64> {
        self.pattern.clone()
    }

    /// BUILD-32: the declared END-OF-EPOCH state seam — replace the
    /// accumulated pattern. The one write an end-position mechanism needs;
    /// no existing behavior consults it.
    pub fn set_pattern(&mut self, pattern: &[f64]) {
        for (site, value) in self.pattern.iter_mut().zip(pattern) {
            *site = *value;
        }
    }

    /// RF-free support readout of the accumulated pattern (integer) —
    /// the count of the same classification `pattern_bins` returns.
    pub fn support(&self) -> usize {
        pattern_bins(&self.pattern).len()
    }
}

/// The S2 contrast arm: present the world's wave passively N times
/// (observation, not consequence) and measure the accumulated support.
/// A single wave is structureless — expected: 1 (DC only).
pub fn passive_support(world: (f64, f64), epochs: usize) -> usize {
    let mut pattern = vec![0.0f64; N * N];
    for _ in 0..epochs {
        for y in 0..N {
            for x in 0..N {
                let phase =
                    2.0 * core::f64::consts::PI * (world.0 * x as f64 + world.1 * y as f64);
                // one wave: |psi|^2 = 1 at every site
                let _ = phase;
                pattern[y * N + x] += 1.0;
            }
        }
    }
    let mut spectrum = vec![0.0f64; N * N];
    for vy in 0..N {
        for vx in 0..N {
            let mut acc = 0.0f64;
            for y in 0..N {
                for x in 0..N {
                    let phase = -2.0
                        * core::f64::consts::PI
                        * (vx as f64 * x as f64 + vy as f64 * y as f64)
                        / N as f64;
                    acc += pattern[y * N + x] * phase.cos();
                }
            }
            spectrum[vy * N + vx] = acc * acc;
        }
    }
    let max = spectrum.iter().copied().fold(0.0f64, f64::max);
    if max <= 0.0 {
        return 0;
    }
    spectrum.iter().filter(|v| **v > max * 1e-18).count()
}

/// Harness closed form: the bins of all recorded differences ±(a − W)
/// (conjugate-inclusive) plus DC — through the SAME law the recorder uses
/// (`difference_of` + `bins_of_difference`), so the closed form and the
/// arrows cannot drift apart.
pub fn expected_support(actions: &[(f64, f64)], world: (f64, f64)) -> usize {
    let mut bins: Vec<(i64, i64)> = Vec::new();
    for action in actions {
        for bin in bins_of_difference(difference_of(*action, world)) {
            if !bins.contains(&bin) {
                bins.push(bin);
            }
        }
    }
    bins.len() + 1 // + DC
}
