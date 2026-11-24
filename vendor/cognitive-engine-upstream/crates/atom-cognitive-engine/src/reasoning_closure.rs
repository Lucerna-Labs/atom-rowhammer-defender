//! BUILD-2 — the reasoning engine: derivation as interference closure.
//! Precommitted in `docs/BUILD2-REASONING.md` BEFORE this code, under
//! the vault Rules 1/2/3 and project rules 0/0a/0b.
//!
//! The inference step IS the physics: interference extracts the
//! difference (relation) between premises; re-injection makes relations
//! into content; the loop chains them. The derivation stops when the
//! frequency support saturates. There is NO inference rule, rule table,
//! depth limit, or stopping threshold anywhere in this module.
//!
//! AUDIT (structural constants only): N (lattice size), the interface
//! convention k = cycles/site (BUILD-1 declaration), the L2 norm
//! (conservation — no constant), and the epoch phase from stream
//! position. The numerical-zero epsilon in `support()` classifies EXACT
//! zeros of an integer-aligned grid (~1e-30) for COUNTING — it is a
//! harness-side classification, never a behavioral threshold: no
//! decision in the engine consults it.

use crate::interference_field::{born_intensity, window_field, N};
use crate::perception::Reading;
use crate::weave::Cx;

/// Constructible orderings of the physical cycle (Rule 2 sweep).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CycleOrder {
    /// Canonical: measure (destructive readout) then re-inject.
    MeasureThenInject,
    /// Variant: re-inject from the unmeasured field, THEN measure
    /// (the destructive readout still happens, but the feedback samples
    /// the coherent field rather than the intensity).
    InjectThenMeasure,
    /// Canonical without the structural epoch phase.
    MeasureThenInjectNoPhase,
}

/// The closure engine: premises in, derivation epoch by epoch, support
/// saturation as the emergent stopping.
pub struct ClosureEngine {
    field: Vec<Cx>,
    epoch: usize,
    order: CycleOrder,
    support_history: Vec<usize>,
}

impl ClosureEngine {
    pub fn from_readings(readings: &[Reading], order: CycleOrder) -> Self {
        let phase = Self::epoch_phase(0, order);
        ClosureEngine {
            field: window_field(readings, phase),
            epoch: 0,
            order,
            support_history: Vec::new(),
        }
    }

    /// Structural epoch phase from the stream position (BUILD-1
    /// convention; absent in the NoPhase order).
    fn epoch_phase(epoch: usize, order: CycleOrder) -> f64 {
        match order {
            CycleOrder::MeasureThenInjectNoPhase => 0.0,
            _ => {
                2.0 * core::f64::consts::PI
                    * (epoch as f64 * 0.618_033_988_749_894_9).fract()
            }
        }
    }

    /// One derivation epoch: the Born readout, then re-injection of the
    /// measured intensity as the next wave source (feedback — the
    /// systems primitive), L2-normalized (conservation).
    pub fn step(&mut self) {
        let carrier = Self::epoch_phase(self.epoch + 1, self.order);
        let carrier = Cx::new(carrier.cos(), carrier.sin());
        let next = match self.order {
            CycleOrder::MeasureThenInject | CycleOrder::MeasureThenInjectNoPhase => {
                let intensity = born_intensity(&self.field);
                Self::re_inject(&intensity, carrier)
            }
            CycleOrder::InjectThenMeasure => {
                // Feedback samples the COHERENT field: amplitude is
                // re-used directly (no destructive readout before the
                // feedback path), then the readout still fires.
                let coherent: Vec<f64> = self
                    .field
                    .iter()
                    .map(|site| (site.re * site.re + site.im * site.im).sqrt())
                    .collect();
                let _ = born_intensity(&self.field); // the readout fires
                Self::re_inject(&coherent, carrier)
            }
        };
        self.field = next;
        self.epoch += 1;
        self.support_history.push(self.support());
    }

    /// Re-injection: amplitude = sqrt(L2-normalized intensity), carrying
    /// the epoch phase. Conservation: the L2 norm is the budget.
    fn re_inject(intensity: &[f64], carrier: Cx) -> Vec<Cx> {
        let norm = intensity
            .iter()
            .map(|v| v * v)
            .sum::<f64>()
            .sqrt();
        let scale = if norm > 0.0 { 1.0 / norm } else { 1.0 };
        intensity
            .iter()
            .map(|v| {
                let amplitude = (v * scale).sqrt();
                Cx::new(amplitude * carrier.re, amplitude * carrier.im)
            })
            .collect()
    }

    /// BUILD-25: the support as BINS — the same classification `support()`
    /// uses (non-numerical-zero Fourier modes), returned as a set so a
    /// composing build can union it.
    pub fn support_bins(&self) -> Vec<(usize, usize)> {
        let mut spectrum = vec![0.0f64; N * N];
        for vy in 0..N {
            for vx in 0..N {
                let mut acc = Cx::new(0.0, 0.0);
                for y in 0..N {
                    for x in 0..N {
                        let phase = -2.0
                            * core::f64::consts::PI
                            * (vx as f64 * x as f64 + vy as f64 * y as f64)
                            / N as f64;
                        let site = self.field[y * N + x];
                        acc.re += site.re * phase.cos() - site.im * phase.sin();
                        acc.im += site.re * phase.sin() + site.im * phase.cos();
                    }
                }
                spectrum[vy * N + vx] = acc.re * acc.re + acc.im * acc.im;
            }
        }
        let max = spectrum.iter().copied().fold(0.0f64, f64::max);
        let epsilon = max * 1e-18; // exact zeros are ~1e-30 relative
        let mut bins = Vec::new();
        for vy in 0..N {
            for vx in 0..N {
                if spectrum[vy * N + vx] > epsilon {
                    bins.push((vx, vy));
                }
            }
        }
        bins
    }

    /// The frequency support of the current content: the count of
    /// Fourier modes with non-numerical-zero mass.
    pub fn support(&self) -> usize {
        self.support_bins().len()
    }

    pub fn support_history(&self) -> &[usize] {
        &self.support_history
    }

    pub fn epoch(&self) -> usize {
        self.epoch
    }
}

/// Measurement API: the frequency support of a SINGLE-SHOT readout of
/// the given readings (no feedback) — the R3 baseline.
pub fn single_shot_support(readings: &[Reading]) -> usize {
    let field = window_field(readings, 0.0);
    let intensity = born_intensity(&field);
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
                    acc += intensity[y * N + x] * phase.cos();
                }
            }
            spectrum[vy * N + vx] = acc * acc;
        }
    }
    let max = spectrum.iter().copied().fold(0.0f64, f64::max);
    spectrum.iter().filter(|v| **v > max * 1e-18).count()
}

/// Harness-side: enumerate the subgroup H of Z_N^2 generated by the
/// given difference vectors (integer arithmetic; the closed form the
/// emergent support must match).
pub fn generated_subgroup_size(generators: &[(isize, isize)]) -> usize {
    let modulus = N as isize;
    let mut h: Vec<(isize, isize)> = vec![(0, 0)];
    let mut frontier = vec![(0isize, 0isize)];
    while let Some((x, y)) = frontier.pop() {
        for (gx, gy) in generators {
            let next = ((x + gx).rem_euclid(modulus), (y + gy).rem_euclid(modulus));
            if !h.contains(&next) {
                h.push(next);
                frontier.push(next);
            }
        }
    }
    h.len()
}
