//! Fixed composition of intrinsic scalar-pressure material states.
//! Labels, policy, and classification belong outside the engine.
use atom_pressure_principles::{
    entropy, unit, Binding, Density, Habituation, Heat, Henon, Logistic, PrincipleError, Spring,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    TransportThenSource,
    SourceThenTransport,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chaos {
    Plain,
    LogisticRotation,
    HenonBinding,
    Both,
    LogisticBinding,
    HenonRotation,
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
    pub population: f64,
    pub coherence: f64,
    pub trace: f64,
    pub determinant: f64,
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

pub trait PressureOrchestrator {
    fn observe(engine: &mut Engine, fraction: f64) -> Result<Observation, PrincipleError>;
}
impl PressureOrchestrator for Engine {
    fn observe(engine: &mut Engine, fraction: f64) -> Result<Observation, PrincipleError> {
        engine.step(fraction)
    }
}

impl Engine {
    pub fn new(order: Order, mask: Mask, chaos: Chaos, seed: u64) -> Result<Self, PrincipleError> {
        let density = Density::default();
        density.validate()?;
        let last = Observation {
            energy: 0.0,
            crossed: false,
            population: 0.0,
            coherence: 0.0,
            trace: 1.0,
            determinant: 0.0,
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

    pub fn step(&mut self, fraction: f64) -> Result<Observation, PrincipleError> {
        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
            return Err(PrincipleError::InvalidInput);
        }
        if self.chaos == Chaos::Plain {
            let mut next = self.clone();
            let out = next.advance(fraction, 1.0, 1.0, false)?;
            *self = next;
            return Ok(out);
        }
        let l = self.logistic.next()?;
        let h = self.henon.next()?;
        let (rotation, association) = match self.chaos {
            Chaos::LogisticRotation => (1.0 + 0.05 * l, 1.0),
            Chaos::HenonBinding => (1.0, 1.0 + 0.05 * h),
            Chaos::Both => (1.0 + 0.05 * l, 1.0 + 0.05 * h),
            Chaos::LogisticBinding => (1.0, 1.0 + 0.05 * l),
            Chaos::HenonRotation => (1.0 + 0.05 * h, 1.0),
            Chaos::Plain => (1.0, 1.0),
        };
        let mut plain = self.clone();
        let plain_out = plain.advance(fraction, 1.0, 1.0, false)?;
        let mut perturbed = self.clone();
        let perturbed_out = perturbed.advance(fraction, rotation, association, true)?;
        let free = |out: Observation| {
            (out.population - fraction).powi(2) + 0.1 * out.position.powi(2) + 0.1 * out.heat
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
        fraction: f64,
        rotation_factor: f64,
        association_factor: f64,
        selected_chaos: bool,
    ) -> Result<Observation, PrincipleError> {
        let theta =
            std::f64::consts::PI * fraction / 64.0 * (1.0 - 0.1 * self.spring.0) * rotation_factor;
        let association = 0.1 * association_factor;
        if !(0.095..=0.105).contains(&association) || !theta.is_finite() {
            return Err(PrincipleError::InvalidState);
        }
        match self.order {
            Order::TransportThenSource => {
                self.transport(theta)?;
                self.source(fraction)?;
            }
            Order::SourceThenTransport => {
                self.source(fraction)?;
                self.transport(theta)?;
            }
        }
        if !self.mask.quantum {
            self.density.discard_coherence()?;
        }
        let p1 = self.density.r11;
        let h = entropy(self.density)?;
        if self.mask.chemistry {
            self.binding.step(p1, association)?;
        }
        if self.mask.cognition {
            self.habituation.step(p1)?;
        }
        let displacement = if self.mask.mechanics {
            self.spring.step(self.binding.0, p1, self.habituation.0)?
        } else {
            0.0
        };
        if self.mask.physics {
            self.heat.step(displacement)?;
        }
        let energy = p1 * self.binding.0 * self.spring.0 + self.heat.0;
        if !energy.is_finite() || energy < -1e-12 {
            return Err(PrincipleError::InvalidState);
        }
        unit(self.binding.0)?;
        unit(self.habituation.0)?;
        unit(self.spring.0)?;
        unit(self.heat.0)?;
        let out = Observation {
            energy,
            crossed: energy >= 0.005,
            population: p1,
            coherence: self.density.r01,
            trace: self.density.r00 + self.density.r11,
            determinant: self.density.r00 * self.density.r11 - self.density.r01 * self.density.r01,
            entropy: h,
            binding: self.binding.0,
            habituation: self.habituation.0,
            position: self.spring.0,
            heat: self.heat.0,
            selected_chaos,
        };
        self.last = out;
        Ok(out)
    }

    fn transport(&mut self, theta: f64) -> Result<(), PrincipleError> {
        if self.mask.diffusion {
            self.density.diffuse(theta)
        } else {
            self.density.rotate(theta)
        }
    }

    fn source(&mut self, fraction: f64) -> Result<(), PrincipleError> {
        let info = if self.mask.information {
            entropy(self.density)?
        } else {
            0.0
        };
        let delta = 0.15 * (1.0 - fraction) * (1.0 - 0.1 * info);
        self.density.reset(delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replay_is_exact_and_invalid_input_fails_closed() -> Result<(), PrincipleError> {
        let mut a = Engine::new(Order::TransportThenSource, Mask::FULL, Chaos::Both, 42)?;
        let mut b = a.clone();
        for i in 0..256 {
            let fraction = ((i * 17) % 101) as f64 / 100.0;
            let left = a.step(fraction)?;
            let right = b.step(fraction)?;
            assert_eq!(left.energy.to_bits(), right.energy.to_bits());
            assert!((left.trace - 1.0).abs() < 1e-10);
            assert!(left.determinant >= -1e-12);
        }
        assert_eq!(a.step(f64::NAN).err(), Some(PrincipleError::InvalidInput));
        Ok(())
    }
}
