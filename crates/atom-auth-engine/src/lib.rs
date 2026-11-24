//! Fixed classical compositions over intrinsic states. The input identifiers
//! are opaque; this engine has no attack names, classifier, or response policy.

use atom_auth_principles::{
    finite, Amplitude, Binding, Habituation, Henon, Logistic, Momentum, Occupation, PrincipleError,
    Reservoir,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub time_ns: u64,
    pub origin: u64,
    pub principal: u64,
    pub failure: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    AdsorbBeforeHabituate,
    HabituateBeforeAdsorb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chaos {
    Plain,
    LogisticAtOverlap,
    HenonAtForce,
    Both,
    LogisticAtForce,
    HenonAtOverlap,
}

impl Chaos {
    fn phase(self) -> Option<Source> {
        match self {
            Self::LogisticAtOverlap | Self::Both => Some(Source::Logistic),
            Self::HenonAtOverlap => Some(Source::Henon),
            _ => None,
        }
    }
    fn force(self) -> Option<Source> {
        match self {
            Self::HenonAtForce | Self::Both => Some(Source::Henon),
            Self::LogisticAtForce => Some(Source::Logistic),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Logistic,
    Henon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mask {
    pub quantum: bool,
    pub information: bool,
    pub chemistry: bool,
    pub cognition: bool,
    pub mechanics: bool,
    pub physics: bool,
}
impl Mask {
    pub const FULL: Self = Self {
        quantum: true,
        information: true,
        chemistry: true,
        cognition: true,
        mechanics: true,
        physics: true,
    };
    pub const BASE_ONLY: Self = Self {
        quantum: true,
        information: false,
        chemistry: false,
        cognition: false,
        mechanics: false,
        physics: false,
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Observation {
    pub energy: f64,
    pub peak_energy: f64,
    pub rupture: bool,
    pub overlap: f64,
    pub entropy: f64,
    pub binding_flux: f64,
    pub habituation: f64,
    pub quantum_norm: Option<f64>,
    pub retained_sites: usize,
    pub retained_observations: usize,
}

pub struct Engine {
    amplitude: Amplitude,
    occupation: Occupation,
    binding: Binding,
    habituation: Habituation,
    momentum: Momentum,
    reservoir: Reservoir,
    logistic: Logistic,
    henon: Henon,
    mask: Mask,
    chaos: Chaos,
    replacement_overlap: Option<f64>,
    step: fn(&mut Engine, Event, u64) -> Result<Observation, PrincipleError>,
    previous_response: f64,
    last_time: Option<u64>,
    peak_energy: f64,
}

impl Engine {
    pub fn new(order: Order, chaos: Chaos, mask: Mask, seed: u64) -> Self {
        let step = match order {
            Order::AdsorbBeforeHabituate => Self::adsorb_before_habituate,
            Order::HabituateBeforeAdsorb => Self::habituate_before_adsorb,
        };
        Self {
            amplitude: Amplitude::default(),
            occupation: Occupation::default(),
            binding: Binding::default(),
            habituation: Habituation::default(),
            momentum: Momentum::default(),
            reservoir: Reservoir::default(),
            logistic: Logistic::new(seed),
            henon: Henon::new(seed ^ 0xa11c_e5a1),
            mask,
            chaos,
            step,
            replacement_overlap: None,
            previous_response: 1.0,
            last_time: None,
            peak_energy: 0.0,
        }
    }

    /// Explicit classical replacement used only by matched experiments.
    pub fn with_replacement_overlap(mut self, overlap: f64) -> Result<Self, PrincipleError> {
        let overlap = finite(overlap)?;
        if !(0.0..=1.0).contains(&overlap) {
            return Err(PrincipleError::NonFinite);
        }
        self.replacement_overlap = Some(overlap);
        Ok(self)
    }

    pub fn invariants_hold(&self) -> bool {
        self.amplitude
            .norm()
            .is_none_or(|norm| (norm - 1.0).abs() < 1e-12)
            && self.occupation.count() == self.occupation.bin_sum()
            && self.occupation.count() <= 256
            && self.binding.site_count() <= 64
            && self.binding.valid()
            && self.habituation.valid()
            && self.momentum.velocity.is_finite()
            && self.momentum.displacement.is_finite()
            && self.reservoir.heat.is_finite()
            && self.reservoir.heat >= 0.0
            && self.peak_energy.is_finite()
    }

    /// Fixed route into the selected composition; no input-content dispatch.
    pub fn observe(&mut self, event: Event) -> Result<Observation, PrincipleError> {
        if self.last_time.is_some_and(|last| event.time_ns < last) {
            return Err(PrincipleError::NonMonotoneTime);
        }
        let dt = self.last_time.map_or(0, |last| event.time_ns - last);
        let result = (self.step)(self, event, dt)?;
        self.last_time = Some(event.time_ns);
        Ok(result)
    }

    fn next_chaos(&mut self) -> (f64, f64) {
        (self.logistic.step(), self.henon.step())
    }

    fn overlap(&mut self, origin: u64, logistic: f64, henon: f64) -> Result<f64, PrincipleError> {
        if let Some(replacement) = self.replacement_overlap {
            return Ok(replacement);
        }
        if !self.mask.quantum {
            return Ok(1.0);
        }
        let plain = {
            let mut candidate = self.amplitude.clone();
            let value = candidate.observe(origin, 0.0)?;
            (candidate, value)
        };
        let selected = match self.chaos.phase() {
            Some(source) => {
                let perturbation = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.amplitude.clone();
                let value = candidate.observe(origin, 0.05 * perturbation)?;
                // Orthogonalization's local law retains the more distinguishable
                // valid state. The selected state persists into the next step.
                if value < plain.1 {
                    (candidate, value)
                } else {
                    plain
                }
            }
            None => plain,
        };
        self.amplitude = selected.0;
        Ok(selected.1)
    }

    fn adsorb_before_habituate(
        &mut self,
        event: Event,
        dt: u64,
    ) -> Result<Observation, PrincipleError> {
        self.compose(event, dt, false)
    }

    fn habituate_before_adsorb(
        &mut self,
        event: Event,
        dt: u64,
    ) -> Result<Observation, PrincipleError> {
        self.compose(event, dt, true)
    }

    fn compose(
        &mut self,
        event: Event,
        dt: u64,
        cognition_first: bool,
    ) -> Result<Observation, PrincipleError> {
        let (logistic, henon) = self.next_chaos();
        let overlap = self.overlap(event.origin, logistic, henon)?;
        let entropy = if self.mask.information {
            self.occupation.observe(event.principal)?
        } else {
            0.0
        };
        let (binding_flux, response) = if cognition_first {
            let response = if self.mask.cognition {
                self.habituation.observe(event.principal, event.time_ns)?
            } else {
                1.0
            };
            let flux = if self.mask.chemistry {
                self.binding
                    .observe(event.principal, event.time_ns, response)?
            } else {
                0.0
            };
            (flux, response)
        } else {
            let flux = if self.mask.chemistry {
                self.binding
                    .observe(event.principal, event.time_ns, self.previous_response)?
            } else {
                0.0
            };
            let response = if self.mask.cognition {
                self.habituation.observe(event.principal, event.time_ns)?
            } else {
                1.0
            };
            (flux, response)
        };
        self.previous_response = response;

        // The component terms represent interacting conserved or dissipative
        // states. No term inspects a behavior label or chooses a target output.
        let load = f64::from(event.failure);
        let impulse = finite(
            load * overlap
                * overlap
                * (0.04 + 0.12 * entropy + 0.10 * binding_flux + 0.08 * response),
        )?;
        let work = if self.mask.mechanics {
            let mut plain = self.momentum.clone();
            let plain_work = plain.observe(impulse, dt)?;
            match self.chaos.force() {
                Some(source) => {
                    let perturbation = match source {
                        Source::Logistic => logistic,
                        Source::Henon => henon,
                    };
                    let mut candidate = self.momentum.clone();
                    let candidate_work =
                        candidate.observe(impulse * (1.0 + 0.05 * perturbation), dt)?;
                    // Minimum elastic work is a local least-action selector.
                    // The retained momentum participates in later observations.
                    if candidate_work < plain_work {
                        self.momentum = candidate;
                        candidate_work
                    } else {
                        self.momentum = plain;
                        plain_work
                    }
                }
                None => {
                    self.momentum = plain;
                    plain_work
                }
            }
        } else {
            0.0
        };
        let heat = if self.mask.physics {
            self.reservoir.observe(work, dt)?
        } else {
            0.0
        };
        let energy = finite(work + heat)?;
        self.peak_energy = self.peak_energy.max(energy);
        Ok(Observation {
            energy,
            peak_energy: self.peak_energy,
            rupture: energy >= 1.0,
            overlap,
            entropy,
            binding_flux,
            habituation: response,
            quantum_norm: if self.mask.quantum {
                self.amplitude.norm()
            } else {
                None
            },
            retained_sites: self.binding.site_count(),
            retained_observations: self.occupation.count(),
        })
    }
}

/// Four-step boundary: input, fixed stack, engine invocation, observed result.
pub struct AuthOrchestrator;
impl AuthOrchestrator {
    pub fn observe(engine: &mut Engine, input: Event) -> Result<Observation, PrincipleError> {
        engine.observe(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_reaches_engine_and_rejects_backward_time() {
        let mut engine = Engine::new(Order::AdsorbBeforeHabituate, Chaos::Plain, Mask::FULL, 0);
        let event = Event {
            time_ns: 100,
            origin: 7,
            principal: 1,
            failure: true,
        };
        let first = AuthOrchestrator::observe(&mut engine, event).unwrap();
        assert!(first.energy > 0.0);
        assert!((first.quantum_norm.unwrap() - 1.0).abs() < 1e-12);
        assert_eq!(
            AuthOrchestrator::observe(
                &mut engine,
                Event {
                    time_ns: 99,
                    ..event
                }
            ),
            Err(PrincipleError::NonMonotoneTime)
        );
    }
}
