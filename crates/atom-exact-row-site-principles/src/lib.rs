//! Intrinsic site and reservoir laws. IDs and attack labels live outside.
use std::fmt;

pub const KICK: f64 = 0.04;
pub const SITE_TAU_NS: f64 = 1_000_000.0;
pub const RESERVOIR_TAU_NS: f64 = 100_000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    InvalidInput,
    InvalidState,
}
impl fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => f.write_str("invalid finite material input"),
            Self::InvalidState => f.write_str("site material invariant failed"),
        }
    }
}
impl std::error::Error for PrincipleError {}

#[derive(Clone, Copy, Debug)]
pub struct Phase {
    pub ground: f64,
    pub excited: f64,
}
impl Default for Phase {
    fn default() -> Self {
        Self {
            ground: 1.0,
            excited: 0.0,
        }
    }
}
impl Phase {
    pub fn rotate(&mut self, angle: f64) -> Result<(), PrincipleError> {
        if !angle.is_finite() || angle.abs() > 0.1 {
            return Err(PrincipleError::InvalidInput);
        }
        let (sin, cos) = angle.sin_cos();
        let ground = self.ground * cos - self.excited * sin;
        let excited = self.ground * sin + self.excited * cos;
        self.ground = ground;
        self.excited = excited;
        self.validate()
    }
    pub fn occupation(self) -> f64 {
        self.excited * self.excited
    }
    pub fn norm(self) -> f64 {
        self.ground.mul_add(self.ground, self.occupation())
    }
    pub fn validate(self) -> Result<(), PrincipleError> {
        if self.ground.is_finite()
            && self.excited.is_finite()
            && (self.norm() - 1.0).abs() <= 1e-10
            && self.occupation() >= -1e-10
            && self.occupation() <= 1.0 + 1e-10
        {
            Ok(())
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Concentration(pub f64);
impl Concentration {
    pub fn decay(&mut self, dt_ns: u64) -> Result<(), PrincipleError> {
        self.0 *= (-(dt_ns as f64) / SITE_TAU_NS).exp();
        self.validate()
    }
    pub fn deposit(&mut self, weight: f64) -> Result<(), PrincipleError> {
        if !weight.is_finite() || !(0.0..=1.0 + 1e-10).contains(&weight) {
            return Err(PrincipleError::InvalidInput);
        }
        self.0 += weight.clamp(0.0, 1.0);
        self.validate()
    }
    pub fn validate(self) -> Result<(), PrincipleError> {
        if self.0.is_finite() && (0.0..=4096.0).contains(&self.0) {
            Ok(())
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Reservoir(pub f64);
impl Reservoir {
    pub fn advance(&mut self, source: f64, dt_ns: u64) -> Result<f64, PrincipleError> {
        if !source.is_finite() || !(0.0..=4096.0).contains(&source) {
            return Err(PrincipleError::InvalidInput);
        }
        let coupling = 1.0 - (-(dt_ns as f64) / RESERVOIR_TAU_NS).exp();
        self.0 += coupling * (source - self.0);
        if self.0.is_finite() && (0.0..=4096.0).contains(&self.0) {
            Ok(self.0)
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Logistic(f64);
impl Logistic {
    pub fn new(seed: u64) -> Self {
        let bits = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15).rotate_left(23);
        let x = 0.1 + 0.8 * ((bits >> 11) as f64 / ((1_u64 << 53) as f64));
        Self(x)
    }
    pub fn next_centered(&mut self) -> Result<f64, PrincipleError> {
        self.0 = 4.0 * self.0 * (1.0 - self.0);
        if self.0.is_finite() && (0.0..=1.0).contains(&self.0) {
            Ok(2.0 * self.0 - 1.0)
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_phase_norm_and_bounded_material() -> Result<(), PrincipleError> {
        let mut phase = Phase::default();
        let mut concentration = Concentration::default();
        let mut reservoir = Reservoir::default();
        for _ in 0..4096 {
            phase.rotate(KICK)?;
            concentration.decay(4_000)?;
            concentration.deposit(phase.occupation())?;
            reservoir.advance(concentration.0, 4_000)?;
        }
        phase.validate()?;
        assert!(reservoir.0 >= 0.0 && reservoir.0 <= 4096.0);
        Ok(())
    }
}
