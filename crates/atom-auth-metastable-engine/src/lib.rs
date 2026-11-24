//! Fixed execution over opaque events. No attack names, intent labels, or
//! account action enters this two-crate candidate.
use atom_auth_metastable_principles::{
    finite, Bonds, Habit, Heat, Henon, Logistic, Maxwell, OriginEntropy, PrincipleError, Wave,
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
    BondBeforeHabituation,
    HabituationBeforeBond,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chaos {
    Plain,
    LogisticAtWave,
    HenonAtBond,
    Both,
    LogisticAtBond,
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
    fn bond(self) -> Option<Source> {
        match self {
            Self::HenonAtBond | Self::Both => Some(Source::Henon),
            Self::LogisticAtBond => Some(Source::Logistic),
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
    pub origin_entropy: f64,
    pub bond_flux: f64,
    pub habituation: f64,
    pub quantum_norm: Option<f64>,
    pub retained_bonds: usize,
    pub retained_habits: usize,
    pub retained_observations: usize,
}
pub struct Engine {
    wave: Wave,
    info: OriginEntropy,
    bonds: Bonds,
    habit: Habit,
    maxwell: Maxwell,
    heat: Heat,
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
            Order::BondBeforeHabituation => Self::bond_before_habit,
            Order::HabituationBeforeBond => Self::habit_before_bond,
        };
        Self {
            wave: Wave::default(),
            info: OriginEntropy::default(),
            bonds: Bonds::default(),
            habit: Habit::default(),
            maxwell: Maxwell::default(),
            heat: Heat::default(),
            logistic: Logistic::new(seed),
            henon: Henon::new(seed ^ 0xb0ad_5eed),
            mask,
            chaos,
            replacement_overlap: None,
            step,
            previous_response: 1.0,
            last_time: None,
            peak_energy: 0.0,
        }
    }
    pub fn with_constant_overlap(mut self, value: f64) -> Result<Self, PrincipleError> {
        let value = finite(value)?;
        if !(0.0..=1.0).contains(&value) {
            return Err(PrincipleError::NonFinite);
        }
        self.replacement_overlap = Some(value);
        Ok(self)
    }
    pub fn observe(&mut self, event: Event) -> Result<Observation, PrincipleError> {
        if self.last_time.is_some_and(|last| event.time_ns < last) {
            return Err(PrincipleError::BackwardTime);
        }
        let dt = self.last_time.map_or(0, |last| event.time_ns - last);
        let output = (self.step)(self, event, dt)?;
        self.last_time = Some(event.time_ns);
        Ok(output)
    }
    pub fn invariants_hold(&self) -> bool {
        self.wave
            .norm()
            .is_none_or(|norm| (norm - 1.0).abs() < 1e-12)
            && self.info.conserved()
            && self.info.retained() <= 256
            && self.bonds.valid()
            && self.bonds.retained() <= 64
            && self.habit.valid()
            && self.habit.retained() <= 64
            && self.maxwell.stress.is_finite()
            && self.maxwell.strain.is_finite()
            && self.heat.value.is_finite()
            && self.heat.value >= 0.0
            && self.peak_energy.is_finite()
    }
    fn overlap(
        &mut self,
        principal: u64,
        logistic: f64,
        henon: f64,
    ) -> Result<f64, PrincipleError> {
        if let Some(value) = self.replacement_overlap {
            return Ok(value);
        }
        if !self.mask.quantum {
            return Ok(0.0);
        }
        let mut plain = self.wave.clone();
        let plain_value = plain.observe(principal, 0.0)?;
        match self.chaos.wave() {
            Some(source) => {
                let shift = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.wave.clone();
                let candidate_value = candidate.observe(principal, 0.05 * shift)?;
                if candidate_value < plain_value {
                    self.wave = candidate;
                    Ok(candidate_value)
                } else {
                    self.wave = plain;
                    Ok(plain_value)
                }
            }
            None => {
                self.wave = plain;
                Ok(plain_value)
            }
        }
    }
    fn bond_before_habit(&mut self, event: Event, dt: u64) -> Result<Observation, PrincipleError> {
        self.compose(event, dt, false)
    }
    fn habit_before_bond(&mut self, event: Event, dt: u64) -> Result<Observation, PrincipleError> {
        self.compose(event, dt, true)
    }
    fn bond_step(
        &mut self,
        event: Event,
        overlap: f64,
        entropy: f64,
        response: f64,
        logistic: f64,
        henon: f64,
    ) -> Result<f64, PrincipleError> {
        if !self.mask.chemistry {
            return Ok(0.0);
        }
        let load = f64::from(event.failure);
        let mut plain = self.bonds.clone();
        let plain_flux = plain.observe(
            event.principal,
            event.time_ns,
            load,
            overlap,
            entropy,
            response,
            1.0,
        )?;
        match self.chaos.bond() {
            Some(source) => {
                let shift = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.bonds.clone();
                let candidate_flux = candidate.observe(
                    event.principal,
                    event.time_ns,
                    load,
                    overlap,
                    entropy,
                    response,
                    1.0 + 0.05 * shift,
                )?;
                // Chemical free energy is the local retention law.
                if candidate.free_energy() < plain.free_energy() {
                    self.bonds = candidate;
                    Ok(candidate_flux)
                } else {
                    self.bonds = plain;
                    Ok(plain_flux)
                }
            }
            None => {
                self.bonds = plain;
                Ok(plain_flux)
            }
        }
    }
    fn compose(
        &mut self,
        event: Event,
        dt: u64,
        cognition_first: bool,
    ) -> Result<Observation, PrincipleError> {
        let logistic = self.logistic.step();
        let henon = self.henon.step();
        let overlap = self.overlap(event.principal, logistic, henon)?;
        let entropy = if self.mask.information {
            self.info.observe(event.origin)?
        } else {
            0.0
        };
        let (bond_flux, response) = if cognition_first {
            let response = if self.mask.cognition {
                self.habit.observe(event.origin, event.time_ns)?
            } else {
                1.0
            };
            let flux = self.bond_step(event, overlap, entropy, response, logistic, henon)?;
            (flux, response)
        } else {
            let flux = self.bond_step(
                event,
                overlap,
                entropy,
                self.previous_response,
                logistic,
                henon,
            )?;
            let response = if self.mask.cognition {
                self.habit.observe(event.origin, event.time_ns)?
            } else {
                1.0
            };
            (flux, response)
        };
        self.previous_response = response;
        let work = if self.mask.mechanics {
            self.maxwell.observe(bond_flux, dt)?
        } else {
            0.0
        };
        let heat = if self.mask.physics {
            self.heat.observe(work, dt)?
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
            origin_entropy: entropy,
            bond_flux,
            habituation: response,
            quantum_norm: self.wave.norm(),
            retained_bonds: self.bonds.retained(),
            retained_habits: self.habit.retained(),
            retained_observations: self.info.retained(),
        })
    }
}

pub struct MetastableOrchestrator;
impl MetastableOrchestrator {
    pub fn observe(engine: &mut Engine, input: Event) -> Result<Observation, PrincipleError> {
        engine.observe(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_and_error() {
        let mut engine = Engine::new(Order::BondBeforeHabituation, Chaos::Plain, Mask::FULL, 0);
        let event = Event {
            time_ns: 100,
            origin: 1,
            principal: 2,
            failure: true,
        };
        let result = MetastableOrchestrator::observe(&mut engine, event).unwrap();
        assert!(result.energy.is_finite() && engine.invariants_hold());
        assert_eq!(
            MetastableOrchestrator::observe(
                &mut engine,
                Event {
                    time_ns: 99,
                    ..event
                }
            ),
            Err(PrincipleError::BackwardTime)
        );
    }
}
