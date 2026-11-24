//! BUILD-1 — synthetic emergence from quantum functions (precommitted:
//! `docs/BUILD1-SYNTHETIC-EMERGENCE.md`, written before this code, under
//! rules 0a/0b: atom stacks are never hardcoded; quantum functions
//! operate, axioms are not evaluated; anything short of synthetic
//! emergence is failure).
//!
//! AUDIT LIST (G4 — complete; any constant not on this list in the
//! formation pathway is a rule-0b failure):
//! - `N` (lattice size) — STRUCTURAL (substrate definition).
//! - the reading→k map — INTERFACE (sensor surface; mechanism, not
//!   instruction: it maps dimensions to wavevectors and contains no
//!   answer-content).
//! - per-window phase from stream position — STRUCTURAL (each
//!   measurement epoch's illumination phase).
//! - plain summation of intensities — no constant.
//! There are NO thresholds, rates, or bars anywhere in this module.

use crate::weave::Cx;
use crate::perception::Reading;

/// The diffraction lattice: N×N sites; k-vectors live in [0,1)².
pub const N: usize = 20;

/// INTERFACE: a reading's two load-bearing dimensions become its
/// wavevector. No other content enters.
pub(crate) fn wavevector(reading: &Reading) -> (f64, f64) {
    (reading.exploration, reading.convergence)
}

/// One measurement epoch's field: the window's waves added as
/// amplitudes (superposition + interference), carrying the epoch's
/// structural phase.
pub(crate) fn window_field(readings: &[Reading], epoch_phase: f64) -> Vec<Cx> {
    let mut field = vec![Cx::new(0.0, 0.0); N * N];
    let carrier = Cx::new(epoch_phase.cos(), epoch_phase.sin());
    for reading in readings {
        let (kx, ky) = wavevector(reading);
        for y in 0..N {
            for x in 0..N {
                // INTERFACE DECLARATION (structural): k is CYCLES PER
                // SITE. (The earlier /N made k cycles-per-lattice, so a
                // regime at k=0.8 wrapped discontinuously across the
                // lattice boundary — every wave leaked across the whole
                // spectrum. Verified analytically: with cycles/site the
                // difference vectors land on exact integer bins and
                // leakage is numerically zero.)
                let phase = 2.0 * core::f64::consts::PI * (kx * x as f64 + ky * y as f64);
                let wave = Cx::new(phase.cos(), phase.sin());
                // amplitude addition (superposition) with the epoch carrier
                let contribution = wave * carrier;
                let site = &mut field[y * N + x];
                site.re += contribution.re;
                site.im += contribution.im;
            }
        }
    }
    field
}

/// The Born readout: intensity |psi|^2 of one window's field.
pub(crate) fn born_intensity(field: &[Cx]) -> Vec<f64> {
    field
        .iter()
        .map(|site| site.re * site.re + site.im * site.im)
        .collect()
}

/// The stack: ingest windows, measure each destructively (field resets
/// every window — collapse), accumulate the pattern.
pub struct InterferenceField {
    pattern: Vec<f64>,
    windows: usize,
}

/// Ablation selector for the load-bearing gates. `FullStack` is the
/// atom stack; the others exist ONLY as ablation arms (G2/G3) — they
/// are measurements, not builds.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Arm {
    FullStack,
    /// G2: waves added as INTENSITIES — no interference.
    NoInterference,
}

/// G3 arm (no per-window reset) is the separate `CoherentField` below —
/// destructive measurement removed means the field never collapses.

impl InterferenceField {
    pub fn new() -> Self {
        InterferenceField {
            pattern: vec![0.0; N * N],
            windows: 0,
        }
    }

    /// The accumulated pattern (harness readout).
    pub fn pattern_snapshot(&self) -> Vec<f64> {
        self.pattern.clone()
    }

    pub fn ingest_window(&mut self, readings: &[Reading], arm: Arm) {
        // Structural epoch phase: each measurement epoch re-fires with
        // its own illumination phase (derived from stream position).
        let epoch_phase = 2.0
            * core::f64::consts::PI
            * (self.windows as f64 * 0.618_033_988_749_894_9).fract();
        self.windows += 1;
        match arm {
            Arm::FullStack => {
                let field = window_field(readings, epoch_phase);
                let intensity = born_intensity(&field);
                for (site, i) in self.pattern.iter_mut().zip(intensity) {
                    *site += i;
                }
            }
            Arm::NoInterference => {
                // G2: each wave's intensity added directly — a flat field
                // of count `readings.len()` at every site.
                for site in self.pattern.iter_mut() {
                    *site += readings.len() as f64;
                }
            }
        }
    }
}

/// G3 arm: coherent accumulation without destructive reset.
pub struct CoherentField {
    field: Vec<Cx>,
    windows: usize,
}

impl CoherentField {
    pub fn new() -> Self {
        CoherentField {
            field: vec![Cx::new(0.0, 0.0); N * N],
            windows: 0,
        }
    }

    pub fn ingest_window(&mut self, readings: &[Reading]) {
        let epoch_phase = 2.0
            * core::f64::consts::PI
            * (self.windows as f64 * 0.618_033_988_749_894_9).fract();
        self.windows += 1;
        let field = window_field(readings, epoch_phase);
        for (site, contribution) in self.field.iter_mut().zip(field) {
            site.re += contribution.re;
            site.im += contribution.im;
        }
    }

    /// The measured pattern of the coherent (never-reset) field.
    pub fn pattern(&self) -> Vec<f64> {
        born_intensity(&self.field)
    }
}

/// The pattern's own spectrum (harness-side analysis; used by tests to
/// read the emergent fringe structure): D(v) = sum_x I(x) e^{-i2πv·x}.
/// Off-origin peaks of |D| sit at the pairwise difference vectors of
/// the latent regimes — the diffraction signature.
pub fn intensity_spectrum(pattern: &[f64]) -> Vec<f64> {
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
                    let value = pattern[y * N + x];
                    acc.re += value * phase.cos();
                    acc.im += value * phase.sin();
                }
            }
            spectrum[vy * N + vx] = acc.re * acc.re + acc.im * acc.im;
        }
    }
    spectrum
}

/// Strongest off-origin spectral peak position (harness-side; tests
/// only). Returns (vx, vy) and its magnitude.
pub fn strongest_off_origin(spectrum: &[f64]) -> ((usize, usize), f64) {
    let mut best = ((0usize, 0usize), 0.0f64);
    for vy in 0..N {
        for vx in 0..N {
            if vx == 0 && vy == 0 {
                continue;
            }
            let value = spectrum[vy * N + vx];
            if value > best.1 {
                best = ((vx, vy), value);
            }
        }
    }
    best
}

impl Default for InterferenceField {
    fn default() -> Self {
        Self::new()
    }
}
