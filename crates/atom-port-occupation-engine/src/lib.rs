//! Fixed composition over an intrinsic density and membrane field. This
//! engine observes numbers and cannot alter a network or label an attacker.

use atom_port_occupation_principles::{
    finite, Adaptation, Chemistry, Density, DestinationWave, Heat, Henon, Logistic, Membrane,
    PrincipleError, Reaction, SiteInformation,
};
use std::collections::HashMap;

pub const MAX_PENDING: usize = 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Attempt,
    Complete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub time_ns: u64,
    pub origin: u64,
    pub destination: u64,
    pub port: u16,
    pub phase: Phase,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldError {
    InvalidPort,
    BackwardTime,
    UnmatchedCompletion,
    PendingCapacity,
    Principle(PrincipleError),
}
impl From<PrincipleError> for FieldError {
    fn from(error: PrincipleError) -> Self {
        Self::Principle(error)
    }
}
impl std::fmt::Display for FieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid occupation input: {self:?}")
    }
}
impl std::error::Error for FieldError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    ReactBeforeHabituate,
    HabituateBeforeReact,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chaos {
    Plain,
    LogisticAtProjector,
    HenonAtChemistry,
    Both,
    LogisticAtChemistry,
    HenonAtProjector,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Logistic,
    Henon,
}
impl Chaos {
    fn projector(self) -> Option<Source> {
        match self {
            Self::LogisticAtProjector | Self::Both => Some(Source::Logistic),
            Self::HenonAtProjector => Some(Source::Henon),
            _ => None,
        }
    }
    fn chemistry(self) -> Option<Source> {
        match self {
            Self::HenonAtChemistry | Self::Both => Some(Source::Henon),
            Self::LogisticAtChemistry => Some(Source::Logistic),
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
    pub overlap: f64,
    pub port_entropy: f64,
    pub destination_norm: Option<f64>,
    pub density_trace: f64,
    pub retained_observations: usize,
    pub pending: usize,
}
pub struct Engine {
    destination: DestinationWave,
    density: Density,
    info: SiteInformation,
    chemistry: Chemistry,
    mind: Adaptation,
    membrane: Membrane,
    heat: Heat,
    logistic: Logistic,
    henon: Henon,
    mask: Mask,
    chaos: Chaos,
    replacement: bool,
    step: fn(&mut Engine, Event, u64) -> Result<Observation, FieldError>,
    previous_response: f64,
    last_time: Option<u64>,
    peak_energy: f64,
    pending: HashMap<(u64, u64, u16), u32>,
}
impl Engine {
    pub fn new(order: Order, chaos: Chaos, mask: Mask, seed: u64) -> Self {
        let step = match order {
            Order::ReactBeforeHabituate => Self::react_before_habit,
            Order::HabituateBeforeReact => Self::habit_before_react,
        };
        Self {
            destination: DestinationWave::default(),
            density: Density::default(),
            info: SiteInformation::default(),
            chemistry: Chemistry::default(),
            mind: Adaptation::default(),
            membrane: Membrane::default(),
            heat: Heat::default(),
            logistic: Logistic::new(seed),
            henon: Henon::new(seed ^ 0xa85e_f1e1),
            mask,
            chaos,
            replacement: false,
            step,
            previous_response: 1.0,
            last_time: None,
            peak_energy: 0.0,
            pending: HashMap::new(),
        }
    }
    pub fn with_classical_replacement(mut self) -> Self {
        self.replacement = true;
        self
    }
    pub fn observe(&mut self, event: Event) -> Result<Observation, FieldError> {
        if event.port == 0 {
            return Err(FieldError::InvalidPort);
        }
        if self.last_time.is_some_and(|last| event.time_ns < last) {
            return Err(FieldError::BackwardTime);
        }
        let key = (event.origin, event.destination, event.port);
        match event.phase {
            Phase::Attempt => {
                if self.pending.len() >= MAX_PENDING && !self.pending.contains_key(&key) {
                    return Err(FieldError::PendingCapacity);
                }
                if self.pending.get(&key) == Some(&u32::MAX) {
                    return Err(FieldError::PendingCapacity);
                }
            }
            Phase::Complete => {
                if !self.pending.contains_key(&key) {
                    return Err(FieldError::UnmatchedCompletion);
                }
            }
        }
        let dt = self.last_time.map_or(0, |last| event.time_ns - last);
        let result = (self.step)(self, event, dt)?;
        match event.phase {
            Phase::Attempt => {
                *self.pending.entry(key).or_default() += 1;
            }
            Phase::Complete => {
                if let Some(count) = self.pending.get_mut(&key) {
                    *count -= 1;
                    if *count == 0 {
                        self.pending.remove(&key);
                    }
                }
            }
        }
        self.last_time = Some(event.time_ns);
        Ok(Observation {
            pending: self.pending.len(),
            ..result
        })
    }
    pub fn invariants_hold(&self) -> bool {
        self.destination
            .norm()
            .is_none_or(|norm| (norm - 1.0).abs() < 1e-12)
            && self.density.valid()
            && self.info.conserved()
            && self.info.retained() <= 256
            && self.chemistry.valid()
            && self.mind.valid()
            && self.membrane.valid()
            && self.heat.valid()
            && self.pending.len() <= MAX_PENDING
            && self.peak_energy.is_finite()
    }
    fn quantum(&mut self, event: Event, logistic: f64, henon: f64) -> Result<f64, FieldError> {
        if self.replacement {
            return Ok(0.5);
        }
        if !self.mask.quantum {
            return Ok(0.0);
        }
        let overlap = self.destination.observe(event.destination)?;
        let mut plain = self.density.clone();
        plain.observe(event.port, 0.0)?;
        match self.chaos.projector() {
            Some(source) => {
                let dither = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.density.clone();
                candidate.observe(event.port, 0.05 * dither)?;
                if candidate.free_energy() < plain.free_energy() {
                    self.density = candidate;
                } else {
                    self.density = plain;
                }
            }
            None => {
                self.density = plain;
            }
        }
        Ok(overlap)
    }
    fn chemistry(
        &mut self,
        event: Event,
        overlap: f64,
        response: f64,
        dt: u64,
        logistic: f64,
        henon: f64,
    ) -> Result<[f64; 16], FieldError> {
        if !self.mask.chemistry {
            return Ok([0.0; 16]);
        }
        let reaction = match event.phase {
            Phase::Attempt => Reaction::Adsorb,
            Phase::Complete => Reaction::Release,
        };
        let mut plain = self.chemistry.clone();
        let plain_occupied = plain.observe(event.port, reaction, overlap, response, 1.0, dt)?;
        let occupied = match self.chaos.chemistry() {
            Some(source) => {
                let dither = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.chemistry.clone();
                let candidate_occupied = candidate.observe(
                    event.port,
                    reaction,
                    overlap,
                    response,
                    1.0 + 0.05 * dither,
                    dt,
                )?;
                if candidate.free_energy() < plain.free_energy() {
                    self.chemistry = candidate;
                    candidate_occupied
                } else {
                    self.chemistry = plain;
                    plain_occupied
                }
            }
            None => {
                self.chemistry = plain;
                plain_occupied
            }
        };
        Ok(occupied)
    }
    fn react_before_habit(&mut self, event: Event, dt: u64) -> Result<Observation, FieldError> {
        self.compose(event, dt, false)
    }
    fn habit_before_react(&mut self, event: Event, dt: u64) -> Result<Observation, FieldError> {
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
        let overlap = self.quantum(event, logistic, henon)?;
        let entropy = if self.mask.information {
            self.info.observe(event.port)?
        } else {
            0.0
        };
        let (occupied, response) = if cognition_first {
            let response = if self.mask.cognition {
                self.mind.observe(event.port, dt)?
            } else {
                1.0
            };
            let occupied = self.chemistry(event, overlap, response, dt, logistic, henon)?;
            (occupied, response)
        } else {
            let occupied =
                self.chemistry(event, overlap, self.previous_response, dt, logistic, henon)?;
            let response = if self.mask.cognition {
                self.mind.observe(event.port, dt)?
            } else {
                1.0
            };
            (occupied, response)
        };
        self.previous_response = response;
        let work = if self.mask.mechanics {
            self.membrane.observe(&self.density, occupied, dt)?
        } else {
            0.0
        };
        let heat = if self.mask.physics {
            self.heat.observe(work, entropy, dt)?
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
            port_entropy: entropy,
            destination_norm: self.destination.norm(),
            density_trace: self.density.trace(),
            retained_observations: self.info.retained(),
            pending: self.pending.len(),
        })
    }
}

pub struct OccupationOrchestrator;
impl OccupationOrchestrator {
    pub fn observe(engine: &mut Engine, input: Event) -> Result<Observation, FieldError> {
        engine.observe(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_keeps_psd_and_fails_closed_on_unmatched_completion() {
        let mut engine = Engine::new(Order::ReactBeforeHabituate, Chaos::Plain, Mask::FULL, 0);
        let event = Event {
            time_ns: 100,
            origin: 7,
            destination: 11,
            port: 4000,
            phase: Phase::Attempt,
        };
        let first = OccupationOrchestrator::observe(&mut engine, event).unwrap();
        assert!(first.energy.is_finite() && engine.invariants_hold());
        assert_eq!(
            OccupationOrchestrator::observe(
                &mut engine,
                Event {
                    time_ns: 101,
                    port: 4001,
                    phase: Phase::Complete,
                    ..event
                }
            ),
            Err(FieldError::UnmatchedCompletion)
        );
        assert_eq!(
            OccupationOrchestrator::observe(
                &mut engine,
                Event {
                    time_ns: 99,
                    ..event
                }
            ),
            Err(FieldError::BackwardTime)
        );
        assert_eq!(engine.pending.len(), 1);
    }
}
