//! BUILD-4 — preference: thermodynamic relaxation on the interference
//! landscape. Precommitted in `docs/BUILD4-PREFERENCE.md` BEFORE this
//! code, under vault Rules 1/2/3 and project rules 0/0a/0b.
//!
//! AUDIT (the only numbers): N (lattice), the unit tick (one step per
//! iterate — structural), and the landscape's own maximum (the
//! exponent self-scales by it — no temperature constant). No threshold
//! anywhere.

use crate::interference_field::{born_intensity, window_field, N};
use crate::perception::Reading;
use crate::weave::Cx;

/// The relaxation operator under sweep (Rule 2).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RelaxOperator {
    /// Canonical: psi <- normalize(psi * exp(I / max(I))) — imaginary-
    /// time relaxation, self-scaling exponent.
    Exponential,
    /// Variant: psi <- normalize(psi * |psi|) — multiplicative
    /// condensation (measured in Python to equilibrate multi-site; the
    /// recorded operator-sensitivity finding).
    Multiplicative,
}

/// A relaxing field: the interference landscape plus one amplitude
/// distribution flowing down it.
pub struct PreferenceField {
    field: Vec<Cx>,
}

impl PreferenceField {
    pub fn from_readings(readings: &[Reading]) -> Self {
        PreferenceField {
            field: window_field(readings, 0.0),
        }
    }

    /// One unit tick of relaxation; no other parameter exists. The step is
    /// the shared `relax_field` below — BUILD-14 composes this same code.
    pub fn relax_step(&mut self, operator: RelaxOperator) {
        relax_field(&mut self.field, operator);
    }

    /// Participation: 1 / sum p^2 over the Born distribution.
    pub fn participation(&self) -> f64 {
        let total: f64 = self
            .field
            .iter()
            .map(|v| v.re * v.re + v.im * v.im)
            .sum();
        let sumsq: f64 = self
            .field
            .iter()
            .map(|v| {
                let p = (v.re * v.re + v.im * v.im) / total;
                p * p
            })
            .sum();
        1.0 / sumsq
    }

    /// The top-n sites by Born probability (harness readout).
    pub fn top_sites(&self, n: usize) -> Vec<(usize, usize)> {
        let mut sites: Vec<(f64, usize, usize)> = self
            .field
            .iter()
            .enumerate()
            .map(|(index, v)| {
                let p = v.re * v.re + v.im * v.im;
                (p, index % N, index / N)
            })
            .collect();
        sites.sort_by(|a, b| b.0.partial_cmp(&a.0).expect("finite"));
        sites.into_iter().take(n).map(|(_, x, y)| (x, y)).collect()
    }
}

/// BUILD-4's relaxation step as shared code (extracted unchanged so BUILD-14
/// composes the same operator rather than a copy): one unit tick of
/// `psi <- normalize(psi * exp(I / max I))`, or the multiplicative map.
/// Degenerate guard (structural): a zero landscape has no maximum, so the
/// tick is a no-op — exp(0/0) is undefined, not relaxable.
pub fn relax_field(field: &mut [Cx], operator: RelaxOperator) {
    let intensity = born_intensity(field);
    let max = intensity.iter().copied().fold(0.0f64, f64::max);
    if max <= 0.0 {
        return;
    }
    match operator {
        RelaxOperator::Exponential => {
            for (site, i) in field.iter_mut().zip(&intensity) {
                let factor = (i / max).exp();
                site.re *= factor;
                site.im *= factor;
            }
        }
        RelaxOperator::Multiplicative => {
            for site in field.iter_mut() {
                let magnitude = (site.re * site.re + site.im * site.im).sqrt();
                site.re *= magnitude;
                site.im *= magnitude;
            }
        }
    }
    normalize_field(field);
}

fn normalize_field(field: &mut [Cx]) {
    let norm = field
        .iter()
        .map(|v| v.re * v.re + v.im * v.im)
        .sum::<f64>()
        .sqrt();
    if norm > 0.0 {
        let scale = 1.0 / norm;
        for site in field.iter_mut() {
            site.re *= scale;
            site.im *= scale;
        }
    }
}

/// Harness closed form: the argmax set of the landscape (sites within
/// 1e-6 relative of the maximum intensity) — size and membership.
pub fn argmax_set(readings: &[Reading]) -> Vec<(usize, usize)> {
    let field = window_field(readings, 0.0);
    let intensity = born_intensity(&field);
    let max = intensity.iter().copied().fold(0.0f64, f64::max);
    intensity
        .iter()
        .enumerate()
        .filter(|(_, v)| (**v - max).abs() < 1e-6 * max)
        .map(|(index, _)| (index % N, index / N))
        .collect()
}
