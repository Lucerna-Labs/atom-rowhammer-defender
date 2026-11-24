//! BUILD-6 — generation and open-endedness via incommensurate shear.
//! Precommitted in `docs/BUILD6-GENERATION.md` BEFORE this code.
//!
//! AUDIT: N; the rotation number alpha = 1/phi (the golden conjugate —
//! the mechanism's declared incommensurability parameter, the most
//! irrational rotation number); the unit tick. No threshold.

use crate::interference_field::born_intensity;
use crate::interference_field::N;
use crate::weave::Cx;

/// Shear axis under sweep (Rule 2).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ShearAxis {
    X,
    Y,
    Both,
}

/// The generating field: one wave marched by incommensurate shear.
pub struct GeneratingField {
    field: Vec<Cx>,
    alpha: f64,
    beta: f64,
    axis: ShearAxis,
    epoch: usize,
}

/// The golden conjugate — the mechanism's rotation number.
pub const GOLDEN_CONJUGATE: f64 = 0.618_033_988_749_894_9;

impl GeneratingField {
    pub fn from_seed_wave(seed_wave: (f64, f64), axis: ShearAxis, alpha: f64, beta: f64) -> Self {
        let mut field = vec![Cx::new(0.0, 0.0); N * N];
        for y in 0..N {
            for x in 0..N {
                let phase =
                    2.0 * core::f64::consts::PI * (seed_wave.0 * x as f64 + seed_wave.1 * y as f64);
                field[y * N + x] = Cx::new(phase.cos(), phase.sin());
            }
        }
        GeneratingField {
            field,
            alpha,
            beta,
            axis,
            epoch: 0,
        }
    }

    /// One epoch: PHASE-PRESERVING reweight (psi * |psi|, BUILD-4's
    /// operator) then the incommensurate shear, then normalization.
    ///
    /// FIRST IMPLEMENTATION WAS A FIXED POINT, kept on record: the
    /// sqrt-of-intensity reweight is phase-blind, so combined with a
    /// position-fixed shear it recreated the identical wave every epoch
    /// — the field never marched (measured: 2 distinct frequencies over
    /// 64 epochs). The phase-preserving reweight keeps the accumulated
    /// phase; the shear then accumulates and the frequency marches by
    /// alpha per epoch (verified in Python before this port).
    pub fn step(&mut self) {
        let intensity = born_intensity(&self.field);
        let max = intensity.iter().copied().fold(0.0f64, f64::max);
        let phase_step = match self.axis {
            ShearAxis::X => (self.alpha, 0.0),
            ShearAxis::Y => (0.0, self.beta),
            ShearAxis::Both => (self.alpha, self.beta),
        };
        for y in 0..N {
            for x in 0..N {
                let site = self.field[y * N + x];
                let magnitude = (site.re * site.re + site.im * site.im).sqrt();
                let weight = if max > 0.0 { magnitude / max } else { 0.0 };
                let shear_phase = 2.0
                    * core::f64::consts::PI
                    * (phase_step.0 * x as f64 + phase_step.1 * y as f64);
                let (s, c) = (shear_phase.sin(), shear_phase.cos());
                // (psi * weight) * e^{i shear}
                let weighted = Cx::new(site.re * weight, site.im * weight);
                self.field[y * N + x] = Cx::new(
                    weighted.re * c - weighted.im * s,
                    weighted.re * s + weighted.im * c,
                );
            }
        }
        // Conservation: renormalize (the norm is the budget).
        let norm = self
            .field
            .iter()
            .map(|v| v.re * v.re + v.im * v.im)
            .sum::<f64>()
            .sqrt();
        if norm > 0.0 {
            let scale = 1.0 / norm;
            for site in self.field.iter_mut() {
                site.re *= scale;
                site.im *= scale;
            }
        }
        self.epoch += 1;
    }

    /// PHYSICAL readout (corrected twice, kept on record): the first
    /// version averaged the wrapped phase increment INCLUDING the
    /// lattice-boundary wrap site (contaminated: integer-looking bins);
    /// the second used quadratic DFT interpolation (readout noise
    /// swamped the gap structure). This one is the multi-lag phase
    /// unwrap: consistent phases at lags {1,2,3,5,7} select the unique
    /// frequency in (-0.5, 0.5]. Exact for single-mode fields.
    pub fn peak_x_frequency(&self) -> f64 {
        let lags = [1usize, 2, 3, 5, 7];
        let mut thetas = [0.0f64; 5];
        for (slot, lag) in lags.iter().enumerate() {
            let mut acc = crate::weave::Cx::new(0.0, 0.0);
            for x in 0..N - lag {
                let a = self.field[x + lag]; // row 0
                let b = self.field[x];
                acc.re += a.re * b.re + a.im * b.im;
                acc.im += a.im * b.re - a.re * b.im;
            }
            thetas[slot] = acc.im.atan2(acc.re) / (2.0 * core::f64::consts::PI);
        }
        let mut best_error = f64::INFINITY;
        let mut best = 0.0f64;
        for m in -3..=3 {
            let candidate = thetas[0] + m as f64;
            if candidate.abs() > 0.5 {
                continue;
            }
            let mut error = 0.0f64;
            for (slot, lag) in lags.iter().enumerate() {
                let mut d = candidate * *lag as f64 - thetas[slot];
                d -= d.round();
                error += d.abs();
            }
            if error < best_error {
                best_error = error;
                best = candidate;
            }
        }
        best * N as f64
    }

    pub fn epoch(&self) -> usize {
        self.epoch
    }
}

/// Distinct-gap count of a visited-position set (harness statistic for
/// the three-gap check).
pub fn distinct_gaps(positions: &[f64]) -> usize {
    let mut sorted: Vec<f64> = positions.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    let mut gaps: Vec<i64> = Vec::new();
    for pair in sorted.windows(2) {
        let gap = ((pair[1] - pair[0]) * 1e3).round() as i64; // 3-decimal bins: merges fold-boundary near-duplicates
        if gap > 0 && !gaps.contains(&gap) {
            gaps.push(gap);
        }
    }
    gaps.len()
}
