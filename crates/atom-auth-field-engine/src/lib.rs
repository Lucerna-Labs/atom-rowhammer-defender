//! Fixed compositions of an intrinsic eight-site field. Attack labels and
//! actions belong outside this engine.

use atom_auth_field_principles::{
    finite, site, Adaptation, Adsorption, Elastic, FieldError, Henon, Logistic, OriginInformation,
    Thermal, Wave, SITES,
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
    LogisticAtWave,
    HenonAtSpring,
    Both,
    LogisticAtSpring,
    HenonAtWave,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Logistic,
    Henon,
}
impl Chaos {
    fn wave(self) -> Option<Source> {
        match self {
            Self::LogisticAtWave | Self::Both => Some(Source::Logistic),
            Self::HenonAtWave => Some(Source::Henon),
            _ => None,
        }
    }
    fn spring(self) -> Option<Source> {
        match self {
            Self::HenonAtSpring | Self::Both => Some(Source::Henon),
            Self::LogisticAtSpring => Some(Source::Logistic),
            _ => None,
        }
    }
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
    pub wave_norm: Option<f64>,
    pub wave_entropy: Option<f64>,
    pub origin_entropy: f64,
    pub habituation: f64,
    pub retained_observations: usize,
}

pub struct Engine {
    wave: Wave,
    info: OriginInformation,
    chemistry: Adsorption,
    mind: Adaptation,
    elastic: Elastic,
    thermal: Thermal,
    logistic: Logistic,
    henon: Henon,
    mask: Mask,
    chaos: Chaos,
    replacement_density: Option<f64>,
    step: fn(&mut Engine, Event, u64) -> Result<Observation, FieldError>,
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
            wave: Wave::default(),
            info: OriginInformation::default(),
            chemistry: Adsorption::default(),
            mind: Adaptation::default(),
            elastic: Elastic::default(),
            thermal: Thermal::default(),
            logistic: Logistic::new(seed),
            henon: Henon::new(seed ^ 0xd15a_b17e),
            mask,
            chaos,
            replacement_density: None,
            step,
            previous_response: 1.0,
            last_time: None,
            peak_energy: 0.0,
        }
    }

    /// Explicit classical replacement for matched experiments only.
    pub fn with_uniform_replacement(mut self) -> Self {
        self.replacement_density = Some(1.0 / SITES as f64);
        self
    }

    pub fn observe(&mut self, event: Event) -> Result<Observation, FieldError> {
        if self.last_time.is_some_and(|last| event.time_ns < last) {
            return Err(FieldError::BackwardTime);
        }
        let dt = self.last_time.map_or(0, |last| event.time_ns - last);
        let result = (self.step)(self, event, dt)?;
        self.last_time = Some(event.time_ns);
        Ok(result)
    }

    pub fn invariants_hold(&self) -> bool {
        (self.wave.norm() - 1.0).abs() <= 1e-12
            && self.info.conserved()
            && self.info.retained() <= 256
            && self.chemistry.valid()
            && self.mind.valid()
            && self.elastic.valid()
            && self.thermal.valid()
            && self.peak_energy.is_finite()
    }

    fn select_wave(
        &mut self,
        event: Event,
        logistic: f64,
        henon: f64,
    ) -> Result<[f64; SITES], FieldError> {
        if let Some(density) = self.replacement_density {
            return Ok([density; SITES]);
        }
        if !self.mask.quantum {
            return Ok([0.0; SITES]);
        }
        let mut plain = self.wave.clone();
        let plain_density = plain.observe(event.origin, event.principal, 0.0)?;
        match self.chaos.wave() {
            Some(source) => {
                let value = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.wave.clone();
                let candidate_density =
                    candidate.observe(event.origin, event.principal, 0.05 * value)?;
                // Intrinsic mixing law keeps the valid state with higher
                // amplitude entropy; the chosen state persists.
                if candidate.entropy() > plain.entropy() {
                    self.wave = candidate;
                    Ok(candidate_density)
                } else {
                    self.wave = plain;
                    Ok(plain_density)
                }
            }
            None => {
                self.wave = plain;
                Ok(plain_density)
            }
        }
    }

    fn adsorb_before_habituate(
        &mut self,
        event: Event,
        dt: u64,
    ) -> Result<Observation, FieldError> {
        self.compose(event, dt, false)
    }
    fn habituate_before_adsorb(
        &mut self,
        event: Event,
        dt: u64,
    ) -> Result<Observation, FieldError> {
        self.compose(event, dt, true)
    }

    fn compose(
        &mut self,
        event: Event,
        dt: u64,
        cognition_first: bool,
    ) -> Result<Observation, FieldError> {
        let logistic = self.logistic.step();
        let henon = self.henon.step();
        let densities = self.select_wave(event, logistic, henon)?;
        let entropy = if self.mask.information {
            self.info.observe(event.origin)?
        } else {
            0.0
        };
        let destination = site(event.principal);
        let load = f64::from(event.failure);
        let (occupied, response) = if cognition_first {
            let response = if self.mask.cognition {
                self.mind.observe(event.origin, dt)?
            } else {
                1.0
            };
            let occupied = if self.mask.chemistry {
                self.chemistry
                    .observe(destination, densities[destination], load, response, dt)?
            } else {
                [0.0; SITES]
            };
            (occupied, response)
        } else {
            let occupied = if self.mask.chemistry {
                self.chemistry.observe(
                    destination,
                    densities[destination],
                    load,
                    self.previous_response,
                    dt,
                )?
            } else {
                [0.0; SITES]
            };
            let response = if self.mask.cognition {
                self.mind.observe(event.origin, dt)?
            } else {
                1.0
            };
            (occupied, response)
        };
        self.previous_response = response;

        let kinetic = if self.mask.mechanics {
            let mut plain = self.elastic.clone();
            let plain_energy = plain.observe(occupied, dt, 0.15)?;
            match self.chaos.spring() {
                Some(source) => {
                    let value = match source {
                        Source::Logistic => logistic,
                        Source::Henon => henon,
                    };
                    let mut candidate = self.elastic.clone();
                    let candidate_energy =
                        candidate.observe(occupied, dt, 0.15 * (1.0 + 0.05 * value))?;
                    // Least elastic work is a local selector. Retain its state.
                    if candidate_energy.iter().sum::<f64>() < plain_energy.iter().sum::<f64>() {
                        self.elastic = candidate;
                        candidate_energy
                    } else {
                        self.elastic = plain;
                        plain_energy
                    }
                }
                None => {
                    self.elastic = plain;
                    plain_energy
                }
            }
        } else {
            [0.0; SITES]
        };
        let heat = if self.mask.physics {
            self.thermal.observe(kinetic, dt, entropy)?
        } else {
            [0.0; SITES]
        };
        let energy = finite(
            (0..SITES)
                .map(|i| kinetic[i] + heat[i])
                .fold(0.0_f64, f64::max),
        )?;
        self.peak_energy = self.peak_energy.max(energy);
        Ok(Observation {
            energy,
            peak_energy: self.peak_energy,
            rupture: energy >= 1.0,
            wave_norm: self.mask.quantum.then(|| self.wave.norm()),
            wave_entropy: self.mask.quantum.then(|| self.wave.entropy()),
            origin_entropy: entropy,
            habituation: response,
            retained_observations: self.info.retained(),
        })
    }
}

/// The public route accepts input and returns only the selected engine result.
pub struct FieldOrchestrator;
impl FieldOrchestrator {
    pub fn observe(engine: &mut Engine, event: Event) -> Result<Observation, FieldError> {
        engine.observe(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_and_rejection() {
        let mut engine = Engine::new(Order::AdsorbBeforeHabituate, Chaos::Plain, Mask::FULL, 0);
        let event = Event {
            time_ns: 100,
            origin: 1,
            principal: 2,
            failure: true,
        };
        let first = FieldOrchestrator::observe(&mut engine, event).unwrap();
        assert!(first.energy.is_finite() && engine.invariants_hold());
        assert_eq!(
            FieldOrchestrator::observe(
                &mut engine,
                Event {
                    time_ns: 99,
                    ..event
                }
            ),
            Err(FieldError::BackwardTime)
        );
    }
}
