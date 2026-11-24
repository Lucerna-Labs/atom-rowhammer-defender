//! E1R — quantum-first, EMERGENT perception (precommitted spec:
//! `docs/E1R-PERCEPTION-QUANTUM.md`, written before this code, under the
//! four discovery rules in `a-rag-atom/docs/DISCOVERY-RULES.md`).
//!
//! ZERO TYPED-IN CONTENT: no label vocabulary, no authored conjunctions,
//! no pattern-content constants. The only constants are named regime
//! knobs (`nucleation_gap`, `reinforce_rate`, `window`, `floor`, `seed`,
//! arrangement flags). Names are emergent bank indices, never authored
//! words. The interface encoding (readings → amplitudes by rotation) is
//! mechanism — the sensor surface — not an instruction: which
//! combination "means" what is nowhere typed; it forms.
//!
//! Honest v0 scope: interference is amplitude-level (inner products and
//! blending on the weave substrate). Gate-circuit interference
//! (CNOT-swap match measurement with Born sampling) is the next
//! increment.

use crate::weave::{Register, Rng};
use crate::perception::Reading;

/// Regime knobs ONLY (rule 2: no content constants live here).
#[derive(Debug, Clone, Copy)]
pub struct EmergentConfig {
    /// BIND's nucleation bar: a reading whose best fidelity to the bank
    /// is below this nucleates a NEW prototype (its own state).
    pub nucleation_gap: f64,
    /// REINFORCE weight α: prototype ← normalize(proto + α·ψ(reading)).
    pub reinforce_rate: f64,
    /// Readings per window.
    pub window: usize,
    /// Evidence count a prototype must reach within a window to name it.
    pub floor: usize,
    /// Whether reinforcement also DRIFTS the prototype (revision). When
    /// false, matching prototypes are selected but never updated.
    pub drift: bool,
    /// Arrangement axis under sweep: reinforce-then-assign (true) or
    /// assign-then-reinforce (false) within the ingest step.
    pub reinforce_then_assign: bool,
    /// Deterministic seed (structure must not depend on it beyond order).
    pub seed: u64,
}

impl Default for EmergentConfig {
    fn default() -> Self {
        EmergentConfig {
            nucleation_gap: 0.7,
            reinforce_rate: 0.3,
            window: 5,
            floor: 3,
            drift: true,
            reinforce_then_assign: false,
            seed: 2026_0925,
        }
    }
}

/// The resonance bank: prototype states nucleated from data (BIND),
/// reinforced by amplitude interference (REINFORCE), drifting when armed
/// (revision). Bank indices are the emergent names.
#[derive(Debug, Clone)]
pub struct EmergentPerceiver {
    config: EmergentConfig,
    prototypes: Vec<Register>,
    rng: Rng,
}

impl EmergentPerceiver {
    pub fn new(config: EmergentConfig) -> Self {
        let seed = config.seed;
        EmergentPerceiver {
            config,
            prototypes: Vec::new(),
            rng: Rng::new(seed),
        }
    }

    pub fn config(&self) -> &EmergentConfig {
        &self.config
    }

    /// The bank's emergent names (indices), in nucleation order.
    pub fn bank_size(&self) -> usize {
        self.prototypes.len()
    }

    /// Read-only access to a formed prototype (scoring/inspection).
    pub fn prototype(&self, index: usize) -> Option<&Register> {
        self.prototypes.get(index)
    }

    /// Interface encoding (mechanism, not instruction): the reading's two
    /// load-bearing dimensions prepare a 2-qubit product register by
    /// rotation. The other four dimensions are unused in v0 — an
    /// interface choice, recorded here, not a content rule.
    pub fn encode(reading: &Reading) -> Register {
        let mut state = Register::zeros(2);
        state.rotate_y(0, core::f64::consts::PI * reading.exploration);
        state.rotate_y(1, core::f64::consts::PI * reading.convergence);
        state
    }

    fn best_match(&self, state: &Register) -> Option<(usize, f64)> {
        let mut best: Option<(usize, f64)> = None;
        for (index, prototype) in self.prototypes.iter().enumerate() {
            let fidelity = prototype.fidelity(state);
            if best.is_none_or(|(_, best_f)| fidelity > best_f) {
                best = Some((index, fidelity));
            }
        }
        best
    }

    /// Ingest one reading: nucleate (BIND) below the gap, otherwise
    /// reinforce (interference blend) the best-matching prototype.
    /// Returns the reading's assignment — the emergent name it took.
    pub fn ingest(&mut self, reading: &Reading) -> usize {
        let state = Self::encode(reading);
        match self.best_match(&state) {
            None => {
                self.prototypes.push(state);
                self.prototypes.len() - 1
            }
            Some((index, fidelity)) => {
                if fidelity < self.config.nucleation_gap {
                    self.prototypes.push(state);
                    self.prototypes.len() - 1
                } else {
                    let assignment = if self.config.reinforce_then_assign {
                        if self.config.drift {
                            let blend = state;
                            let rate = self.config.reinforce_rate;
                            if let Some(prototype) = self.prototypes.get_mut(index) {
                                prototype.interfere(&blend, rate);
                            }
                        }
                        index
                    } else {
                        index
                    };
                    if !self.config.reinforce_then_assign && self.config.drift {
                        let blend = state;
                        let rate = self.config.reinforce_rate;
                        if let Some(prototype) = self.prototypes.get_mut(index) {
                            prototype.interfere(&blend, rate);
                        }
                    }
                    assignment
                }
            }
        }
    }

    /// Name a window by the majority prototype of its readings (count ≥
    /// floor, else abstain). Assignments are made against the CURRENT
    /// bank — the post-formation state.
    pub fn name_window(&self, readings: &[Reading]) -> Option<usize> {
        if readings.len() < self.config.floor {
            return None;
        }
        let mut counts: Vec<usize> = vec![0; self.prototypes.len()];
        for reading in readings {
            let state = Self::encode(reading);
            if let Some((index, fidelity)) = self.best_match(&state) {
                if fidelity >= self.config.nucleation_gap {
                    counts[index] += 1;
                }
            }
        }
        let best = counts.iter().max().copied().unwrap_or(0);
        if best < self.config.floor {
            return None;
        }
        let winners = counts.iter().filter(|c| **c == best).count();
        if winners > 1 {
            return None; // tied evidence abstains
        }
        counts.iter().position(|c| *c == best)
    }

    /// E1R-2 — BORN-SAMPLED naming: assignments decided by SWAP-test
    /// circuits (H–cSWAP–H, measured) rather than exact inner products.
    /// `shots` per (reading, prototype) comparison; deterministic per
    /// `seed`. Same majority/floor/tie discipline as `name_window`.
    pub fn name_window_sampled(
        &self,
        readings: &[Reading],
        shots: usize,
        seed: u64,
    ) -> Option<usize> {
        if readings.len() < self.config.floor {
            return None;
        }
        let mut rng = crate::weave::Rng::new(seed);
        let mut counts: Vec<usize> = vec![0; self.prototypes.len()];
        for reading in readings {
            let state = Self::encode(reading);
            let mut best: Option<(usize, f64)> = None;
            for (index, prototype) in self.prototypes.iter().enumerate() {
                let sampled = prototype.swap_test(&state, &mut rng, shots);
                if best.is_none_or(|(_, f)| sampled > f) {
                    best = Some((index, sampled));
                }
            }
            if let Some((index, fidelity)) = best {
                if fidelity >= self.config.nucleation_gap {
                    counts[index] += 1;
                }
            }
        }
        let best_count = counts.iter().max().copied().unwrap_or(0);
        if best_count < self.config.floor {
            return None;
        }
        let winners = counts.iter().filter(|c| **c == best_count).count();
        if winners > 1 {
            return None;
        }
        counts.iter().position(|c| *c == best_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(id: u64, exploration: f64, convergence: f64) -> Reading {
        Reading::new(id, 0.5, 0.5, 0.5, exploration, convergence, 0.5)
    }

    #[test]
    fn nucleation_then_reinforcement() {
        let mut perceiver = EmergentPerceiver::new(EmergentConfig::default());
        let first = perceiver.ingest(&reading(1, 0.8, 0.7));
        assert_eq!(perceiver.bank_size(), 1, "first reading nucleates");
        let near = perceiver.ingest(&reading(2, 0.81, 0.69));
        assert_eq!(near, first, "nearby reading reinforces, not nucleates");
        assert_eq!(perceiver.bank_size(), 1);
        let far = perceiver.ingest(&reading(3, 0.3, 0.7));
        assert_eq!(perceiver.bank_size(), 2, "distant reading nucleates");
        assert_ne!(far, first);
    }

    #[test]
    fn drifted_prototype_tracks_its_readings() {
        let config = EmergentConfig {
            reinforce_rate: 0.5,
            ..EmergentConfig::default()
        };
        let mut perceiver = EmergentPerceiver::new(config);
        for k in 0..10u64 {
            perceiver.ingest(&reading(k, 0.8, 0.7));
        }
        assert_eq!(perceiver.bank_size(), 1);
        // A window of the same regime names through the drifted bank.
        let window: Vec<Reading> = (0..5u64)
            .map(|k| reading(100 + k, 0.8, 0.7))
            .collect();
        assert_eq!(perceiver.name_window(&window), Some(0));
    }

    #[test]
    fn the_rng_seed_exists_but_structure_does_not_depend_on_it() {
        // The bank forms from geometry; the seed only orders nucleation.
        for seed in [1u64, 7, 99, 2026] {
            let config = EmergentConfig {
                seed,
                ..EmergentConfig::default()
            };
            let mut perceiver = EmergentPerceiver::new(config);
            for k in 0..5u64 {
                perceiver.ingest(&reading(k, 0.8, 0.7));
                perceiver.ingest(&reading(50 + k, 0.8, 0.3));
                perceiver.ingest(&reading(100 + k, 0.3, 0.7));
            }
            assert_eq!(perceiver.bank_size(), 3, "three regimes, seed {seed}");
        }
    }
}
