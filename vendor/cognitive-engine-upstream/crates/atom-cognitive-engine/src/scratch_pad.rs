//! BUILD-15 — the thermodynamic scratch pad: hold, dissipate, reuse.
//! Precommitted in `docs/BUILD15-SCRATCH-PAD.md` BEFORE this code, under
//! vault Rules 1/2/3/4 and project rules 0/0a/0b.
//!
//! Working memory for the harness: a task writes its intermediates into the
//! pad, reads them back from the pad, and the pad dissipates them at task
//! end so the capacity is reusable.
//!
//! AUDIT (the only numbers): N (substrate) and the tick count (a declared
//! experiment budget). The readout's relative resolution is the observer's
//! convention, not the pad's mechanism. No rate, no threshold, no scoring;
//! structures are experiment input and supports are measurements.

use crate::interference_field::{born_intensity, intensity_spectrum, N};
use crate::weave::Cx;

/// Observer resolution for the support readout (harness convention, not a
/// pad mechanism).
pub const SUPPORT_RESOLUTION: f64 = 1e-9;

/// Clearing mechanism under measurement (Rule 2 discipline applied to the
/// clearing role — both are executed and recorded):
/// `Dissipate` is the canonical harmonic self-scaling decay; `Reset` is the
/// destructive readout (BUILD-1's measured memorylessness) as the competing
/// arm.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ClearMechanism {
    Dissipate,
    Reset,
}

/// The pad: one field over the substrate lattice (capacity = the lattice,
/// declared).
pub struct ScratchPad {
    field: Vec<Cx>,
}

impl ScratchPad {
    pub fn new() -> Self {
        ScratchPad {
            field: vec![Cx::new(0.0, 0.0); N * N],
        }
    }

    /// Hold: accumulate a structure into the pad (superposition — the
    /// substrate's own addition, no new rule).
    pub fn write(&mut self, structure: &[Cx]) {
        for (site, value) in self.field.iter_mut().zip(structure) {
            site.re += value.re;
            site.im += value.im;
        }
    }

    /// BUILD-16 wiring interface: carry a real pattern as real amplitudes.
    /// The write→read round-trip is the identity, exact in f64.
    pub fn write_pattern(&mut self, pattern: &[f64]) {
        for (site, value) in self.field.iter_mut().zip(pattern) {
            site.re += *value;
        }
    }

    /// The carried pattern (real parts).
    /// BUILD-32: the declared END-OF-STACK state seam. Scale the carried
    /// field in place (both components, so the phase is preserved) — the
    /// one operation an end-position mechanism needs, and nothing more.
    pub fn scale(&mut self, factor: f64) {
        for site in self.field.iter_mut() {
            site.re *= factor;
            site.im *= factor;
        }
    }

    /// BUILD-33: rotate the carried field by `theta` (both components) — the
    /// ROTATION piece's declared interaction point. Magnitudes are preserved
    /// exactly; every phase moves by theta.
    pub fn rotate(&mut self, theta: f64) {
        let (sin, cos) = theta.sin_cos();
        for site in self.field.iter_mut() {
            let (re, im) = (site.re, site.im);
            site.re = re * cos - im * sin;
            site.im = re * sin + im * cos;
        }
    }

    /// BUILD-33: roll the carried field cyclically by `(dx, dy)` — the
    /// TRANSLATION piece's declared interaction point, on the lattice's own
    /// torus convention. The multiset of magnitudes is preserved; the sites
    /// are permuted.
    pub fn translate(&mut self, dx: usize, dy: usize) {
        let n = crate::interference_field::N;
        let (dx, dy) = (dx % n, dy % n);
        if dx == 0 && dy == 0 {
            return;
        }
        let mut out = vec![Cx::new(0.0, 0.0); self.field.len()];
        for y in 0..n {
            for x in 0..n {
                out[y * n + x] = self.field[((y + dy) % n) * n + (x + dx) % n];
            }
        }
        self.field = out;
    }

    pub fn read_pattern(&self) -> Vec<f64> {
        self.field.iter().map(|v| v.re).collect()
    }

    /// Dissipate: `ticks` harmonic self-scaling decay steps. At tick τ the
    /// amplitude multiplies by τ/(τ+1), so after T ticks the amplitude
    /// factor is exactly 1/(T+1) and the mass factor exactly 1/(T+1)^2.
    /// No authored rate: the tick count IS the scale. Asymptotic — the pad
    /// approaches zero and never reaches it (the thermodynamic signature).
    pub fn dissipate(&mut self, ticks: usize) {
        for tau in 1..=ticks {
            let factor = tau as f64 / (tau as f64 + 1.0);
            for site in self.field.iter_mut() {
                site.re *= factor;
                site.im *= factor;
            }
        }
    }

    /// Reset arm: clear by destructive measurement — exactly zero.
    pub fn reset(&mut self) {
        for site in self.field.iter_mut() {
            site.re = 0.0;
            site.im = 0.0;
        }
    }

    /// Clear by the selected mechanism.
    pub fn clear(&mut self, mechanism: ClearMechanism, ticks: usize) {
        match mechanism {
            ClearMechanism::Dissipate => self.dissipate(ticks),
            ClearMechanism::Reset => self.reset(),
        }
    }

    /// Total mass (sum |psi|^2).
    pub fn mass(&self) -> f64 {
        self.field
            .iter()
            .map(|v| v.re * v.re + v.im * v.im)
            .sum()
    }

    pub fn field(&self) -> &[Cx] {
        &self.field
    }

    /// Harness readout: the difference-bin support of the pad's content at
    /// the observer's relative resolution.
    pub fn support(&self) -> usize {
        support_of(&self.field)
    }
}

impl Default for ScratchPad {
    fn default() -> Self {
        Self::new()
    }
}

/// Harness closed form: the dissipation law — mass fraction after `ticks`.
pub fn expected_mass_fraction(ticks: usize) -> f64 {
    let t = ticks as f64 + 1.0;
    1.0 / (t * t)
}

/// The derived, content-free clearing budget (BUILD-16): the tick count at
/// which a residue's own mass fraction falls to the observer's resolution —
/// ceil(1/sqrt(RESOLUTION) − 1) = 31622. Derived from the resolution alone;
/// no content, no authored rate (BUILD-15's law).
pub fn self_clearing_ticks() -> usize {
    (1.0 / SUPPORT_RESOLUTION.sqrt() - 1.0).ceil() as usize
}

/// Harness readout: difference-bin support of a field — the spectrum of the
/// Born intensity at the observer's relative resolution.
pub fn support_of(field: &[Cx]) -> usize {
    let pattern = born_intensity(field);
    let spectrum = intensity_spectrum(&pattern);
    let max = spectrum.iter().copied().fold(0.0f64, f64::max);
    if max <= 0.0 {
        return 0;
    }
    spectrum
        .iter()
        .filter(|v| **v > SUPPORT_RESOLUTION * max)
        .count()
}

/// Harness closed form: the tick count at which a residue's largest bin
/// falls below the next structure's resolution — derived from the two
/// fields' own bin powers (no authored number). Verified in Python before
/// this code: the knife-edge is predictable (T_min 31622 for the design
/// pair; T-1 contaminated, T clean).
pub fn ticks_for_visibility(residue: &[Cx], next: &[Cx]) -> usize {
    let pad_max = max_dft_bin(residue);
    let next_max = max_dft_bin(next);
    if pad_max <= 0.0 || next_max <= 0.0 {
        return 0;
    }
    let ratio = SUPPORT_RESOLUTION * next_max / pad_max;
    if ratio >= 1.0 {
        return 0;
    }
    (1.0 / ratio.sqrt() - 1.0).ceil() as usize
}

/// The largest bin of the field's own DFT (the quantity the tick formula
/// was derived against in Python).
fn max_dft_bin(field: &[Cx]) -> f64 {
    let pi = core::f64::consts::PI;
    let mut best = 0.0f64;
    for vy in 0..N {
        for vx in 0..N {
            let mut acc = Cx::new(0.0, 0.0);
            for y in 0..N {
                for x in 0..N {
                    let phase =
                        -2.0 * pi * ((vx * x + vy * y) as f64) / N as f64;
                    let site = field[y * N + x];
                    acc.re += site.re * phase.cos() - site.im * phase.sin();
                    acc.im += site.re * phase.sin() + site.im * phase.cos();
                }
            }
            let power = acc.re * acc.re + acc.im * acc.im;
            if power > best {
                best = power;
            }
        }
    }
    best
}
