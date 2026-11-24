//! BUILD-3 — perception: emergent identification. Precommitted in
//! `docs/BUILD3-PERCEPTION.md` BEFORE this code, under vault Rules 1/2/3
//! and project rules 0/0a/0b.
//!
//! Recognition is exact and authored nowhere: a presentation that is an
//! instance of known structure adds NO new difference modes (support
//! delta = 0); a novel presentation adds its new modes (delta > 0). The
//! decision is an integer comparison — no threshold exists anywhere.
//!
//! AUDIT (structural constants only): N, the interface convention
//! k = cycles/site, the epoch phase from stream position, plain
//! summation, and the declared numerical-zero epsilon used to COUNT
//! exact zeros (integer-aligned grid ~1e-30) — never consulted by any
//! decision; the decision is `after == before` on integer counts.

use crate::interference_field::{born_intensity, window_field, N};
use crate::perception::Reading;
use crate::reasoning_closure::{ClosureEngine, CycleOrder};
use std::collections::BTreeSet;

/// Constructible order variants (Rule 2 sweep, enumerated in the doc).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PresentOrder {
    /// Canonical: interference → destructive readout → accumulate.
    Canonical,
    /// Canonical without the structural epoch phase.
    NoPhase,
    /// BUILD-25 (BUILD-3's deferred variant c): the BUILD-2 feedback
    /// closure hooked into each presentation — the support is the union of
    /// per-presentation CLOSURE bins rather than of difference bins.
    FeedbackClosure,
}

/// The perceiving field: scenes accumulate; recognition = support delta.
pub struct PerceivingField {
    pattern: Vec<f64>,
    /// BUILD-25: the variant arm's accumulated closure bins (integer sets).
    closure_bins: BTreeSet<(usize, usize)>,
    presentations: usize,
    order: PresentOrder,
}

impl PerceivingField {
    pub fn new(order: PresentOrder) -> Self {
        PerceivingField {
            pattern: vec![0.0; N * N],
            closure_bins: BTreeSet::new(),
            presentations: 0,
            order,
        }
    }

    /// Present one scene. Returns the support delta: 0 = recognized
    /// (an instance of known structure), > 0 = novel (new difference
    /// modes added). Integer throughout; no threshold.
    pub fn present(&mut self, readings: &[Reading]) -> i64 {
        if self.order == PresentOrder::FeedbackClosure {
            return self.present_closure(readings);
        }
        let before = self.support() as i64;
        let phase = match self.order {
            PresentOrder::Canonical => {
                2.0 * core::f64::consts::PI
                    * (self.presentations as f64 * 0.618_033_988_749_894_9).fract()
            }
            // NoPhase — and the closure arm returned above.
            PresentOrder::NoPhase | PresentOrder::FeedbackClosure => 0.0,
        };
        self.presentations += 1;
        let field = window_field(readings, phase);
        let intensity = born_intensity(&field);
        for (site, value) in self.pattern.iter_mut().zip(intensity) {
            *site += value;
        }
        self.support() as i64 - before
    }

    /// BUILD-25: the variant arm — the presentation's own closure (BUILD-2's
    /// engine, its canonical 40-step protocol: matched controls by
    /// construction), unioned as bins. Integer sets; no epsilon of its own.
    fn present_closure(&mut self, readings: &[Reading]) -> i64 {
        let before = self.closure_bins.len() as i64;
        let mut closure = ClosureEngine::from_readings(readings, CycleOrder::MeasureThenInject);
        for _ in 0..40 {
            closure.step();
        }
        for bin in closure.support_bins() {
            self.closure_bins.insert(bin);
        }
        self.presentations += 1;
        self.closure_bins.len() as i64 - before
    }

    /// Frequency support of the accumulated pattern: count of modes with
    /// non-numerical-zero mass (declared zero-classifier; integers). For the
    /// variant arm this is the closure-bin union's size — exact integers, no
    /// epsilon of its own.
    pub fn support(&self) -> usize {
        if self.order == PresentOrder::FeedbackClosure {
            return self.closure_bins.len();
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
                        acc += self.pattern[y * N + x] * phase.cos();
                    }
                }
                spectrum[vy * N + vx] = acc * acc;
            }
        }
        let max = spectrum.iter().copied().fold(0.0f64, f64::max);
        let epsilon = max * 1e-18;
        spectrum.iter().filter(|v| **v > epsilon).count()
    }

    pub fn presentations(&self) -> usize {
        self.presentations
    }
}

/// Harness-side closed form: the number of accumulated spectrum BINS a
/// set of difference generators produces — each distinct difference
/// (and its conjugate) contributes; no coincidences assumed and verified
/// by the harness before use.
pub fn expected_bins(differences: &[(isize, isize)]) -> usize {
    let modulus = N as isize;
    let mut bins: Vec<(isize, isize)> = Vec::new();
    for (dx, dy) in differences {
        for (sx, sy) in [(dx.rem_euclid(modulus), dy.rem_euclid(modulus)),
                         ((-dx).rem_euclid(modulus), (-dy).rem_euclid(modulus))] {
            let bin = (sx, sy);
            if bin != (0, 0) && !bins.contains(&bin) {
                bins.push(bin);
            }
        }
    }
    bins.len()
}
