//! Fixed four-state composition of two intrinsic unit perturbations.
//! It owns no resource names, scenarios, policy, collection or remediation.
use atom_psi_coupling_principles::{
    entropy, unit, Binding, Density, Habituation, Heat, Henon, Logistic, PrincipleError, Spring,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {
    pub first: f64,
    pub second: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    TransportThenSource,
    SourceThenTransport,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chaos {
    Plain,
    LogisticCorrelated,
    HenonBinding,
    Both,
    LogisticBinding,
    HenonCorrelated,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mask {
    pub quantum: bool,
    pub diffusion: bool,
    pub information: bool,
    pub chemistry: bool,
    pub cognition: bool,
    pub mechanics: bool,
    pub physics: bool,
}
impl Mask {
    pub const FULL: Self = Self {
        quantum: true,
        diffusion: false,
        information: true,
        chemistry: true,
        cognition: true,
        mechanics: true,
        physics: true,
    };
}

#[derive(Clone, Copy, Debug)]
pub struct Observation {
    pub energy: f64,
    pub crossed: bool,
    pub occupation: [f64; 4],
    pub coherence_03: f64,
    pub trace: f64,
    pub minimum_minor: f64,
    pub entropy: f64,
    pub binding: f64,
    pub habituation: f64,
    pub position: f64,
    pub heat: f64,
    pub selected_chaos: bool,
}

#[derive(Clone)]
pub struct Engine {
    density: Density,
    binding: Binding,
    habituation: Habituation,
    spring: Spring,
    heat: Heat,
    logistic: Logistic,
    henon: Henon,
    order: Order,
    mask: Mask,
    chaos: Chaos,
    last: Observation,
}

pub trait CouplingOrchestrator {
    fn observe(engine: &mut Engine, event: Event) -> Result<Observation, PrincipleError>;
}
impl CouplingOrchestrator for Engine {
    fn observe(engine: &mut Engine, event: Event) -> Result<Observation, PrincipleError> {
        engine.step(event)
    }
}

impl Engine {
    pub fn new(order: Order, mask: Mask, chaos: Chaos, seed: u64) -> Result<Self, PrincipleError> {
        let density = Density::default();
        density.validate()?;
        let last = Observation {
            energy: 0.0,
            crossed: false,
            occupation: [1.0, 0.0, 0.0, 0.0],
            coherence_03: 0.0,
            trace: 1.0,
            minimum_minor: 0.0,
            entropy: 0.0,
            binding: 0.0,
            habituation: 0.0,
            position: 0.0,
            heat: 0.0,
            selected_chaos: false,
        };
        Ok(Self {
            density,
            binding: Binding::default(),
            habituation: Habituation::default(),
            spring: Spring::default(),
            heat: Heat::default(),
            logistic: Logistic::new(seed),
            henon: Henon::new(seed),
            order,
            mask,
            chaos,
            last,
        })
    }

    pub fn state(&self) -> Observation {
        self.last
    }

    pub fn step(&mut self, event: Event) -> Result<Observation, PrincipleError> {
        if !event.first.is_finite()
            || !event.second.is_finite()
            || !(0.0..=1.0).contains(&event.first)
            || !(0.0..=1.0).contains(&event.second)
        {
            return Err(PrincipleError::InvalidInput);
        }
        if self.chaos == Chaos::Plain {
            let mut next = self.clone();
            let out = next.advance(event, 1.0, 1.0, false)?;
            *self = next;
            return Ok(out);
        }
        let logistic = self.logistic.next()?;
        let henon = self.henon.next()?;
        let (correlated, association) = match self.chaos {
            Chaos::LogisticCorrelated => (1.0 + 0.05 * logistic, 1.0),
            Chaos::HenonBinding => (1.0, 1.0 + 0.05 * henon),
            Chaos::Both => (1.0 + 0.05 * logistic, 1.0 + 0.05 * henon),
            Chaos::LogisticBinding => (1.0, 1.0 + 0.05 * logistic),
            Chaos::HenonCorrelated => (1.0 + 0.05 * henon, 1.0),
            Chaos::Plain => (1.0, 1.0),
        };
        let mut plain = self.clone();
        let plain_out = plain.advance(event, 1.0, 1.0, false)?;
        let mut perturbed = self.clone();
        let perturbed_out = perturbed.advance(event, correlated, association, true)?;
        let free = |out: Observation| {
            (out.occupation[3] - event.first * event.second).powi(2)
                + 0.1 * out.position.powi(2)
                + 0.1 * out.heat
        };
        if free(perturbed_out) < free(plain_out) {
            *self = perturbed;
            Ok(perturbed_out)
        } else {
            *self = plain;
            Ok(plain_out)
        }
    }

    fn advance(
        &mut self,
        event: Event,
        correlated_factor: f64,
        association_factor: f64,
        selected_chaos: bool,
    ) -> Result<Observation, PrincipleError> {
        let first = std::f64::consts::PI * event.first / 128.0;
        let second = std::f64::consts::PI * event.second / 128.0;
        let correlated = std::f64::consts::PI * event.first * event.second / 64.0
            * (1.0 - 0.1 * self.spring.0)
            * correlated_factor;
        let association = 0.1 * association_factor;
        if !correlated.is_finite() || !(0.095..=0.105 + 1e-12).contains(&association) {
            return Err(PrincipleError::InvalidState);
        }
        match self.order {
            Order::TransportThenSource => {
                self.local(first, second)?;
                self.correlate(correlated)?;
                self.source(event)?;
            }
            Order::SourceThenTransport => {
                self.source(event)?;
                self.correlate(correlated)?;
                self.local(first, second)?;
            }
        }
        if !self.mask.quantum {
            self.density.discard_coherence()?;
        }
        let m = self.density.matrix;
        let joint = m[3][3];
        let information = entropy(self.density)?;
        if self.mask.chemistry {
            self.binding.step(joint, association)?;
        }
        if self.mask.cognition {
            self.habituation.step(joint)?;
        }
        let displacement = if self.mask.mechanics {
            self.spring
                .step(self.binding.0, joint, self.habituation.0)?
        } else {
            0.0
        };
        if self.mask.physics {
            self.heat.step(displacement)?;
        }
        let energy = joint * self.binding.0 * self.spring.0 + self.heat.0;
        if !energy.is_finite() || energy < -1e-10 {
            return Err(PrincipleError::InvalidState);
        }
        for value in [
            self.binding.0,
            self.habituation.0,
            self.spring.0,
            self.heat.0,
        ] {
            unit(value)?;
        }
        let out = Observation {
            energy,
            crossed: energy >= 0.005,
            occupation: [m[0][0], m[1][1], m[2][2], m[3][3]],
            coherence_03: m[0][3],
            trace: self.density.trace(),
            minimum_minor: self.density.minimum_principal_minor(),
            entropy: information,
            binding: self.binding.0,
            habituation: self.habituation.0,
            position: self.spring.0,
            heat: self.heat.0,
            selected_chaos,
        };
        self.last = out;
        Ok(out)
    }

    fn local(&mut self, first: f64, second: f64) -> Result<(), PrincipleError> {
        self.rotate(0, 2, first)?;
        self.rotate(1, 3, first)?;
        self.rotate(0, 1, second)?;
        self.rotate(2, 3, second)
    }

    fn correlate(&mut self, angle: f64) -> Result<(), PrincipleError> {
        self.rotate(0, 3, angle)
    }

    fn rotate(&mut self, left: usize, right: usize, angle: f64) -> Result<(), PrincipleError> {
        if self.mask.diffusion {
            self.density.diffuse(left, right, angle)
        } else {
            self.density.rotate(left, right, angle)
        }
    }

    fn source(&mut self, event: Event) -> Result<(), PrincipleError> {
        let information = if self.mask.information {
            entropy(self.density)?
        } else {
            0.0
        };
        self.density
            .reset(0.12 * (1.0 - event.first.min(event.second)) * (1.0 - 0.1 * information))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replay_and_psd_survive_mixed_inputs() -> Result<(), PrincipleError> {
        let mut a = Engine::new(Order::TransportThenSource, Mask::FULL, Chaos::Both, 53)?;
        let mut b = a.clone();
        for i in 0..256 {
            let event = Event {
                first: ((i * 17) % 101) as f64 / 100.0,
                second: ((i * 31) % 101) as f64 / 100.0,
            };
            let left = a.step(event)?;
            let right = b.step(event)?;
            assert_eq!(left.energy.to_bits(), right.energy.to_bits());
            assert!((left.trace - 1.0).abs() <= 1e-9);
            assert!(left.minimum_minor >= -1e-9);
        }
        assert_eq!(
            a.step(Event {
                first: f64::NAN,
                second: 0.5
            })
            .err(),
            Some(PrincipleError::InvalidInput)
        );
        Ok(())
    }
}
