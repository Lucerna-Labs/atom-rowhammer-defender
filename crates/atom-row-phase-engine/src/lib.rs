//! Fixed composition of opaque timed fingerprints into phase/material state.
//! No attack names, physical-row claims, routes or actions enter this engine.
use atom_row_phase_principles::{
    entropy, phasor, relaxation, unit, Binding, Field, Habituation, Heat, Henon, Logistic,
    PrincipleError, Spring,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub time_ns: u64,
    pub fingerprint: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    PhaseInformationFirst,
    PhaseHabituationFirst,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chaos {
    Plain,
    LogisticPhase,
    HenonChemistry,
    Both,
    LogisticChemistry,
    HenonPhase,
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
    pub power: f64,
    pub real: f64,
    pub imag: f64,
    pub entropy: f64,
    pub binding: f64,
    pub habituation: f64,
    pub position: f64,
    pub heat: f64,
    pub alpha: f64,
    pub selected_chaos: bool,
}

#[derive(Clone)]
pub struct Engine {
    field: Field,
    binding: Binding,
    habituation: Habituation,
    spring: Spring,
    heat: Heat,
    logistic: Logistic,
    henon: Henon,
    last_time: Option<u64>,
    order: Order,
    mask: Mask,
    chaos: Chaos,
    last: Observation,
}

pub trait RowPhaseOrchestrator {
    fn observe(engine: &mut Engine, event: Event) -> Result<Observation, PrincipleError>;
}
impl RowPhaseOrchestrator for Engine {
    fn observe(engine: &mut Engine, event: Event) -> Result<Observation, PrincipleError> {
        engine.step(event)
    }
}

impl Engine {
    pub fn new(order: Order, mask: Mask, chaos: Chaos, seed: u64) -> Result<Self, PrincipleError> {
        let field = Field::default();
        field.validate()?;
        let last = Observation {
            energy: 0.0,
            crossed: false,
            power: 0.0,
            real: 0.0,
            imag: 0.0,
            entropy: 0.0,
            binding: 0.0,
            habituation: 0.0,
            position: 0.0,
            heat: 0.0,
            alpha: 0.0,
            selected_chaos: false,
        };
        Ok(Self {
            field,
            binding: Binding::default(),
            habituation: Habituation::default(),
            spring: Spring::default(),
            heat: Heat::default(),
            logistic: Logistic::new(seed),
            henon: Henon::new(seed),
            last_time: None,
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
        let elapsed = match self.last_time {
            Some(last) if event.time_ns > last => event.time_ns - last,
            Some(_) => return Err(PrincipleError::BadTime),
            None => 0,
        };
        if self.chaos == Chaos::Plain {
            let mut next = self.clone();
            let out = next.advance(event, elapsed, 1.0, 1.0, false)?;
            *self = next;
            return Ok(out);
        }
        let l = self.logistic.next()?;
        let h = self.henon.next()?;
        let (phase, chemistry) = match self.chaos {
            Chaos::LogisticPhase => (1.0 + 0.05 * l, 1.0),
            Chaos::HenonChemistry => (1.0, 1.0 + 0.05 * h),
            Chaos::Both => (1.0 + 0.05 * l, 1.0 + 0.05 * h),
            Chaos::LogisticChemistry => (1.0, 1.0 + 0.05 * l),
            Chaos::HenonPhase => (1.0 + 0.05 * h, 1.0),
            Chaos::Plain => (1.0, 1.0),
        };
        let mut plain = self.clone();
        let plain_out = plain.advance(event, elapsed, 1.0, 1.0, false)?;
        let mut perturbed = self.clone();
        let perturbed_out = perturbed.advance(event, elapsed, phase, chemistry, true)?;
        let free = |out: Observation| {
            (1.0 - out.power).powi(2) + 0.1 * out.position.powi(2) + 0.1 * out.heat
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
        elapsed: u64,
        phase_factor: f64,
        chemistry_factor: f64,
        selected_chaos: bool,
    ) -> Result<Observation, PrincipleError> {
        let alpha = relaxation(elapsed, self.spring.0)?;
        if self.mask.quantum {
            self.field
                .coherent(alpha, phasor(event.fingerprint, phase_factor)?)?;
        } else {
            self.field.population(alpha, self.mask.diffusion)?;
        }
        let power = self.field.power;
        let information = if self.mask.information {
            entropy(power)?
        } else {
            0.0
        };
        match self.order {
            Order::PhaseInformationFirst => {
                self.chemistry(power, information, chemistry_factor)?;
                self.cognition(power)?;
            }
            Order::PhaseHabituationFirst => {
                self.cognition(power)?;
                self.chemistry(power, information, chemistry_factor)?;
            }
        }
        let displacement = if self.mask.mechanics {
            self.spring
                .step(self.binding.0, power, self.habituation.0)?
        } else {
            0.0
        };
        if self.mask.physics {
            self.heat.step(displacement)?;
        }
        let energy = self.binding.0 * self.spring.0 + self.heat.0;
        if !energy.is_finite() || energy < -1e-10 {
            return Err(PrincipleError::InvalidState);
        }
        for value in [
            power,
            self.binding.0,
            self.habituation.0,
            self.spring.0,
            self.heat.0,
        ] {
            unit(value)?;
        }
        let out = Observation {
            energy,
            crossed: energy >= 0.01,
            power,
            real: self.field.real,
            imag: self.field.imag,
            entropy: information,
            binding: self.binding.0,
            habituation: self.habituation.0,
            position: self.spring.0,
            heat: self.heat.0,
            alpha,
            selected_chaos,
        };
        self.last_time = Some(event.time_ns);
        self.last = out;
        Ok(out)
    }

    fn chemistry(
        &mut self,
        power: f64,
        information: f64,
        factor: f64,
    ) -> Result<(), PrincipleError> {
        if self.mask.chemistry {
            self.binding
                .step(power, information, self.habituation.0, factor)?;
        }
        Ok(())
    }

    fn cognition(&mut self, power: f64) -> Result<(), PrincipleError> {
        if self.mask.cognition {
            self.habituation.step(power)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_repeat_and_time_order() -> Result<(), PrincipleError> {
        let mut a = Engine::new(Order::PhaseInformationFirst, Mask::FULL, Chaos::Both, 9)?;
        let mut b = a.clone();
        for n in 0..256u64 {
            let event = Event {
                time_ns: n * 4_000,
                fingerprint: n % 3,
            };
            let left = a.step(event)?;
            let right = b.step(event)?;
            assert_eq!(left.energy.to_bits(), right.energy.to_bits());
            assert!(left.power <= 1.0 + 1e-10);
        }
        assert_eq!(
            a.step(Event {
                time_ns: 255 * 4_000,
                fingerprint: 0
            })
            .err(),
            Some(PrincipleError::BadTime)
        );
        Ok(())
    }
}
