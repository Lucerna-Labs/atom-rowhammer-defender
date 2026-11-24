//! BUILD-32 — the end-of-stack chaos mechanisms: candidate first-principle
//! mechanisms, added to be MEASURED and NOT promoted. See
//! `docs/BUILD32-END-CHAOS.md` (precommitted before this code).
//!
//! Two mechanisms, selected by INTRINSIC DYNAMICS rather than by name:
//! the logistic map at `r = 4` (1-D unimodal sensitive dependence) and the
//! Hénon map at `(a, b) = (1.4, 0.3)` (a 2-D dissipative strange attractor,
//! folded into the unit interval by its own modular convention).
//!
//! A third canonical candidate is **rejected here by measurement, not by
//! argument**: the tent map. In f64 its binary-shift dynamics consume one
//! mantissa bit per iteration, so the orbit reaches **exactly 0.0** — a
//! fixed point — within about fifty steps and perturbs nothing thereafter.
//! `tent_collapse_step` returns the measured step.
//!
//! AUDIT: the parameters (r, a, b, seeds) and the amplitude are DECLARED
//! experiment parameters, identical across matched arms; the fold and the
//! factor form are declared conventions. There is no target, threshold,
//! routing, corrective branch, or reward anywhere: the module produces a
//! bounded modulation and nothing else. Nothing in the engine consumes it.

/// The selected mechanisms.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChaosMechanism {
    /// Logistic map, `r = 4`: 1-D unimodal sensitive dependence
    /// (arcsine invariant density over [0, 1]).
    Logistic,
    /// Hénon map, `(a, b) = (1.4, 0.3)`: bounded chaotic mixing on a
    /// dissipative strange attractor, folded into [0, 1) by `x mod 1`.
    Henon,
}

impl ChaosMechanism {
    pub fn name(&self) -> &'static str {
        match self {
            ChaosMechanism::Logistic => "logistic(r=4)",
            ChaosMechanism::Henon => "henon(a=1.4,b=0.3)",
        }
    }

    /// The declared seed (an experiment parameter, not a tuned value).
    fn seed(&self) -> (f64, f64) {
        match self {
            ChaosMechanism::Logistic => (0.4, 0.0),
            ChaosMechanism::Henon => (0.1, 0.1),
        }
    }
}

/// A deterministic chaotic orbit, read as a sequence in [0, 1).
pub struct ChaosOrbit {
    mechanism: ChaosMechanism,
    x: f64,
    y: f64,
}

impl ChaosOrbit {
    pub fn new(mechanism: ChaosMechanism) -> Self {
        let (x, y) = mechanism.seed();
        ChaosOrbit { mechanism, x, y }
    }

    pub fn mechanism(&self) -> ChaosMechanism {
        self.mechanism
    }

    /// The next orbit value in [0, 1) — the mechanism's own dynamics
    /// (the Hénon coordinate is folded by its declared modular convention).
    pub fn next_u(&mut self) -> f64 {
        match self.mechanism {
            ChaosMechanism::Logistic => {
                self.x = 4.0 * self.x * (1.0 - self.x);
                self.x
            }
            ChaosMechanism::Henon => {
                let (x, y) = (self.x, self.y);
                self.x = 1.0 - 1.4 * x * x + y;
                self.y = 0.3 * x;
                (self.x % 1.0 + 1.0) % 1.0
            }
        }
    }

    /// The bounded, matched end-of-cycle perturbation factor:
    /// `1 + alpha * (u - 0.5)`, with `alpha` declared and identical across
    /// arms, so every factor lies in `[1 - alpha/2, 1 + alpha/2]`.
    /// This is the GAIN piece of BUILD-33's affine split.
    pub fn next_factor(&mut self, alpha: f64) -> f64 {
        1.0 + alpha * (self.next_u() - 0.5)
    }

    /// BUILD-33, the ROTATION piece: an angle in `(-pi, pi)` from the orbit,
    /// `theta = pi * (2u - 1)` (a declared convention, identical across arms).
    pub fn next_angle(&mut self) -> f64 {
        core::f64::consts::PI * (2.0 * self.next_u() - 1.0)
    }

    /// BUILD-33, the TRANSLATION piece: a lattice offset in `0..n` from the
    /// orbit, `floor(n * u) mod n` (a declared convention).
    pub fn next_offset(&mut self, n: usize) -> usize {
        ((self.next_u() * n as f64) as usize) % n
    }
}

/// BUILD-33: the three affine pieces of a perturbation, by intrinsic effect
/// on a complex state — gain (magnitude), rotation (phase), translation
/// (place). Each interacts with a consumer readout differently; see
/// `docs/BUILD33-SPLIT-CHAOS.md`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AffinePiece {
    Gain,
    Rotation,
    Translation,
}

impl AffinePiece {
    pub fn name(&self) -> &'static str {
        match self {
            AffinePiece::Gain => "gain",
            AffinePiece::Rotation => "rotation",
            AffinePiece::Translation => "translation",
        }
    }
}

/// The declared mechanism's interaction point: modulate a carried state in
/// place by one bounded factor. Nothing else is done to the state.
pub fn modulate(state: &mut [f64], factor: f64) {
    for value in state.iter_mut() {
        *value *= factor;
    }
}

/// The tent map `x -> 2x` (x < 1/2) / `2(1 - x)`: the canonical
/// bounded-chaotic-mixing example, measured here as a REJECTED mechanism.
/// Returns the step at which the f64 orbit reaches exactly `0.0` (the fixed
/// point it can never leave), or `None` within `limit` steps.
pub fn tent_collapse_step(seed: f64, limit: usize) -> Option<usize> {
    let mut x = seed;
    for step in 1..=limit {
        x = if x < 0.5 { 2.0 * x } else { 2.0 * (1.0 - x) };
        if x == 0.0 {
            return Some(step);
        }
    }
    None
}
