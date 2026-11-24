//! Fixed serial two-hop material composition over scalar perturbations.
//! Input purpose, labels, and any policy remain outside these owning crates.
use atom_pressure_hop_principles::{
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
    LogisticFirst,
    HenonBinding,
    Both,
    LogisticBinding,
    HenonSecond,
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
    pub occupation: [f64; 3],
    pub coherence: [f64; 3],
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

pub trait PressureHopOrchestrator {
    fn observe(engine: &mut Engine, fraction: f64) -> Result<Observation, PrincipleError>;
}
impl PressureHopOrchestrator for Engine {
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
            occupation: [1.0, 0.0, 0.0],
            coherence: [0.0; 3],
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
            let out = next.advance(fraction, 1.0, 1.0, 1.0, false)?;
            *self = next;
            return Ok(out);
        }
        let l = self.logistic.next()?;
        let h = self.henon.next()?;
        let (first, second, association) = match self.chaos {
            Chaos::LogisticFirst => (1.0 + 0.05 * l, 1.0, 1.0),
            Chaos::HenonBinding => (1.0, 1.0, 1.0 + 0.05 * h),
            Chaos::Both => (1.0 + 0.05 * l, 1.0, 1.0 + 0.05 * h),
            Chaos::LogisticBinding => (1.0, 1.0, 1.0 + 0.05 * l),
            Chaos::HenonSecond => (1.0, 1.0 + 0.05 * h, 1.0),
            Chaos::Plain => (1.0, 1.0, 1.0),
        };
        let mut plain = self.clone();
        let plain_out = plain.advance(fraction, 1.0, 1.0, 1.0, false)?;
        let mut perturbed = self.clone();
        let perturbed_out = perturbed.advance(fraction, first, second, association, true)?;
        let free = |out: Observation| {
            (out.occupation[2] - fraction * fraction).powi(2)
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
        fraction: f64,
        first_factor: f64,
        second_factor: f64,
        association_factor: f64,
        selected_chaos: bool,
    ) -> Result<Observation, PrincipleError> {
        let theta = std::f64::consts::PI * fraction / 32.0 * (1.0 - 0.1 * self.spring.0);
        let first = theta * first_factor;
        let second = theta * second_factor;
        let association = 0.2 * association_factor;
        if !first.is_finite() || !second.is_finite() || !(0.19..=0.21).contains(&association) {
            return Err(PrincipleError::InvalidState);
        }
        match self.order {
            Order::TransportThenSource => {
                self.transport(first, second)?;
                self.source(fraction)?;
            }
            Order::SourceThenTransport => {
                self.source(fraction)?;
                self.transport(first, second)?;
            }
        }
        if !self.mask.quantum {
            self.density.discard_coherence()?;
        }
        let p2 = self.density.occupation(2)?;
        let information = entropy(self.density)?;
        if self.mask.chemistry {
            self.binding.step(p2, fraction, association)?;
        }
        if self.mask.cognition {
            self.habituation.step(p2)?;
        }
        let displacement = if self.mask.mechanics {
            self.spring.step(self.binding.0, p2, self.habituation.0)?
        } else {
            0.0
        };
        if self.mask.physics {
            self.heat.step(displacement)?;
        }
        let energy = p2 * self.binding.0 * self.spring.0 + self.heat.0;
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
        let m = self.density.matrix;
        let out = Observation {
            energy,
            crossed: energy >= 0.0001,
            occupation: [m[0][0], m[1][1], m[2][2]],
            coherence: [m[0][1], m[1][2], m[0][2]],
            trace: m[0][0] + m[1][1] + m[2][2],
            determinant: self.density.determinant(),
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

    fn transport(&mut self, first: f64, second: f64) -> Result<(), PrincipleError> {
        if self.mask.diffusion {
            self.density.diffuse(0, 1, first)?;
            self.density.diffuse(1, 2, second)
        } else {
            self.density.rotate(0, 1, first)?;
            self.density.rotate(1, 2, second)
        }
    }

    fn source(&mut self, fraction: f64) -> Result<(), PrincipleError> {
        let information = if self.mask.information {
            entropy(self.density)?
        } else {
            0.0
        };
        self.density
            .reset(0.30 * (1.0 - fraction) * (1.0 - 0.1 * information))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_repeat_and_density_invariants() -> Result<(), PrincipleError> {
        let mut a = Engine::new(Order::TransportThenSource, Mask::FULL, Chaos::Both, 17)?;
        let mut b = a.clone();
        for i in 0..256 {
            let fraction = ((i * 19) % 101) as f64 / 100.0;
            let left = a.step(fraction)?;
            let right = b.step(fraction)?;
            assert_eq!(left.energy.to_bits(), right.energy.to_bits());
            assert!((left.trace - 1.0).abs() < 1e-9);
            assert!(left.determinant >= -1e-10);
        }
        assert_eq!(a.step(f64::NAN).err(), Some(PrincipleError::InvalidInput));
        Ok(())
    }
}
