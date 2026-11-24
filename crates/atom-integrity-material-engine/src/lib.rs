//! Fixed observation-only composition over statistical occupation and
//! metastable material. Runtime has no file names or expected labels.
use atom_integrity_material_principles::{
    entropy, source, Binding, Habituation, Heat, Henon, Logistic, Material, Occupation,
    PrincipleError, N,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    BindBeforeHabituate,
    HabituateBeforeBind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chaos {
    Plain,
    LogisticAtTransport,
    HenonAtBinding,
    Both,
    LogisticAtBinding,
    HenonAtTransport,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Logistic,
    Henon,
}
impl Chaos {
    fn transport(self) -> Option<Source> {
        match self {
            Self::LogisticAtTransport | Self::Both => Some(Source::Logistic),
            Self::HenonAtTransport => Some(Source::Henon),
            _ => None,
        }
    }
    fn binding(self) -> Option<Source> {
        match self {
            Self::HenonAtBinding | Self::Both => Some(Source::Henon),
            Self::LogisticAtBinding => Some(Source::Logistic),
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
    pub rupture: bool,
    pub base_trace: f64,
    pub base_coherence: f64,
    pub information: f64,
    pub occupation: [f64; N],
    pub binding: [f64; N],
    pub habituation: [f64; N],
    pub material: [f64; N],
    pub heat: [f64; N],
}
#[derive(Clone)]
pub struct Engine {
    occupation: Occupation,
    binding: Binding,
    habituation: Habituation,
    material: Material,
    heat: Heat,
    logistic: Logistic,
    henon: Henon,
    mask: Mask,
    chaos: Chaos,
    route: fn(&mut Engine, [f64; 8]) -> Result<Observation, PrincipleError>,
}
impl Engine {
    pub fn new(order: Order, chaos: Chaos, mask: Mask, seed: u64) -> Self {
        let route = match order {
            Order::BindBeforeHabituate => Self::bind_before_habit,
            Order::HabituateBeforeBind => Self::habit_before_bind,
        };
        Self {
            occupation: Occupation::default(),
            binding: Binding::default(),
            habituation: Habituation::default(),
            material: Material::default(),
            heat: Heat::default(),
            logistic: Logistic::new(seed),
            henon: Henon::new(seed ^ 0x51f7_0b17),
            mask,
            chaos,
            route,
        }
    }
    pub fn observe(&mut self, input: [f64; 8]) -> Result<Observation, PrincipleError> {
        let mut candidate = self.clone();
        let result = (candidate.route)(&mut candidate, input)?;
        if !candidate.invariants_hold() {
            return Err(PrincipleError::Capacity);
        }
        *self = candidate;
        Ok(result)
    }
    pub fn invariants_hold(&self) -> bool {
        self.occupation.valid()
            && self.binding.valid()
            && self.habituation.valid()
            && self.material.valid()
            && self.heat.valid()
    }
    fn quantum(
        &mut self,
        vector: [f64; N],
        logistic: f64,
        henon: f64,
    ) -> Result<(), PrincipleError> {
        if !self.mask.quantum {
            return self.occupation.observe(vector, 0.08, true);
        }
        let mut plain = self.occupation.clone();
        plain.observe(vector, 0.08, false)?;
        match self.chaos.transport() {
            Some(source) => {
                let dither = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.occupation.clone();
                candidate.observe(vector, 0.08 * (1.0 + 0.05 * dither), false)?;
                if candidate.free_energy() < plain.free_energy() {
                    self.occupation = candidate;
                } else {
                    self.occupation = plain;
                }
            }
            None => {
                self.occupation = plain;
            }
        }
        Ok(())
    }
    fn bind(
        &mut self,
        diagonal: [f64; N],
        information: f64,
        activity: [f64; N],
        logistic: f64,
        henon: f64,
    ) -> Result<[f64; N], PrincipleError> {
        if !self.mask.chemistry {
            return Ok([0.0; N]);
        }
        let mut plain = self.binding.clone();
        let plain_bound = plain.observe(diagonal, information, activity, 1.0)?;
        match self.chaos.binding() {
            Some(source) => {
                let dither = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.binding.clone();
                let candidate_bound =
                    candidate.observe(diagonal, information, activity, 1.0 + 0.05 * dither)?;
                if candidate.free_energy() < plain.free_energy() {
                    self.binding = candidate;
                    Ok(candidate_bound)
                } else {
                    self.binding = plain;
                    Ok(plain_bound)
                }
            }
            None => {
                self.binding = plain;
                Ok(plain_bound)
            }
        }
    }
    fn bind_before_habit(&mut self, input: [f64; 8]) -> Result<Observation, PrincipleError> {
        self.compose(input, false)
    }
    fn habit_before_bind(&mut self, input: [f64; 8]) -> Result<Observation, PrincipleError> {
        self.compose(input, true)
    }
    fn compose(
        &mut self,
        input: [f64; 8],
        cognition_first: bool,
    ) -> Result<Observation, PrincipleError> {
        let logistic = self.logistic.step();
        let henon = self.henon.step();
        let vector = source(input)?;
        self.quantum(vector, logistic, henon)?;
        let diagonal = self.occupation.diagonal();
        let information = if self.mask.information {
            entropy(diagonal)?
        } else {
            0.0
        };
        let (binding, activity) = if cognition_first {
            let activity = if self.mask.cognition {
                self.habituation.observe(diagonal)?
            } else {
                [0.0; N]
            };
            let bound = self.bind(diagonal, information, activity, logistic, henon)?;
            (bound, activity)
        } else {
            let prior = if self.mask.cognition {
                self.habituation.activity
            } else {
                [0.0; N]
            };
            let bound = self.bind(diagonal, information, prior, logistic, henon)?;
            let activity = if self.mask.cognition {
                self.habituation.observe(diagonal)?
            } else {
                [0.0; N]
            };
            (bound, activity)
        };
        let salience = if self.mask.cognition {
            self.habituation.salience(diagonal)?
        } else {
            diagonal
        };
        let material = if self.mask.mechanics {
            self.material.observe(salience, binding)?
        } else {
            [0.0; N]
        };
        let heat = if self.mask.physics {
            self.heat.observe(material)?
        } else {
            [0.0; N]
        };
        let energy = (0..N)
            .map(|i| material[i] * material[i] + heat[i])
            .fold(0.0f64, f64::max);
        if !energy.is_finite() {
            return Err(PrincipleError::NonFinite);
        }
        Ok(Observation {
            energy,
            rupture: energy >= 0.5,
            base_trace: self.occupation.trace(),
            base_coherence: self.occupation.coherence(),
            information,
            occupation: diagonal,
            binding,
            habituation: activity,
            material,
            heat,
        })
    }
}

/// Input, fixed route, native engine execution, observed result.
pub struct MaterialOrchestrator;
impl MaterialOrchestrator {
    pub fn observe(engine: &mut Engine, input: [f64; 8]) -> Result<Observation, PrincipleError> {
        engine.observe(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clean_state_stays_zero_and_bad_input_does_not_mutate() {
        let mut engine = Engine::new(Order::BindBeforeHabituate, Chaos::Plain, Mask::FULL, 0);
        for _ in 0..40 {
            let result = MaterialOrchestrator::observe(&mut engine, [0.0; 8]).unwrap();
            assert_eq!(result.energy, 0.0);
            assert_eq!(result.base_trace, 0.0);
        }
        let before = engine.occupation.trace();
        assert_eq!(
            MaterialOrchestrator::observe(&mut engine, [0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            Err(PrincipleError::InvalidBit)
        );
        assert_eq!(engine.occupation.trace(), before);
    }
}
