//! Intrinsic driven phase and material laws over opaque numeric fingerprints.
use std::fmt;

pub const PUMP: f64 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    BadTime,
    InvalidState,
}
impl fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadTime => f.write_str("timestamps must increase"),
            Self::InvalidState => f.write_str("phase material invariant failed"),
        }
    }
}
impl std::error::Error for PrincipleError {}

pub fn mix64(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// One opaque fingerprint gives one unit phasor; no behavior label enters.
pub fn phasor(fingerprint: u64, factor: f64) -> Result<(f64, f64), PrincipleError> {
    if !factor.is_finite() || !(0.95..=1.05).contains(&factor) {
        return Err(PrincipleError::InvalidState);
    }
    let u = (mix64(fingerprint ^ 0x6c20_94f1_ab73_d8e5) >> 11) as f64 / ((1u64 << 53) as f64);
    let angle = std::f64::consts::TAU * u * factor;
    let pair = angle.sin_cos();
    if pair.0.is_finite() && pair.1.is_finite() {
        Ok((pair.1, pair.0))
    } else {
        Err(PrincipleError::InvalidState)
    }
}

pub fn relaxation(elapsed_ns: u64, position: f64) -> Result<f64, PrincipleError> {
    unit(position)?;
    let alpha = 0.99 * (-(elapsed_ns as f64) / 500_000.0).exp() * (1.0 - 0.1 * position);
    if alpha.is_finite() && (0.0..=0.99).contains(&alpha) {
        Ok(alpha)
    } else {
        Err(PrincipleError::InvalidState)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Field {
    pub real: f64,
    pub imag: f64,
    pub power: f64,
}
impl Field {
    pub fn coherent(&mut self, alpha: f64, phasor: (f64, f64)) -> Result<(), PrincipleError> {
        self.real = alpha * self.real + PUMP * phasor.0;
        self.imag = alpha * self.imag + PUMP * phasor.1;
        self.power = self.real * self.real + self.imag * self.imag;
        self.validate()
    }
    pub fn population(&mut self, alpha: f64, diffusion: bool) -> Result<(), PrincipleError> {
        self.power = if diffusion {
            alpha * self.power + PUMP * PUMP
        } else {
            alpha * alpha * self.power + PUMP * PUMP
        };
        self.real = 0.0;
        self.imag = 0.0;
        self.validate()
    }
    pub fn validate(&self) -> Result<(), PrincipleError> {
        if self.real.is_finite()
            && self.imag.is_finite()
            && self.power.is_finite()
            && (-1e-10..=1.0 + 1e-10).contains(&self.power)
            && self.real.abs() <= 1.0 + 1e-10
            && self.imag.abs() <= 1.0 + 1e-10
        {
            Ok(())
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

pub fn entropy(power: f64) -> Result<f64, PrincipleError> {
    unit(power)?;
    let mut value = 0.0;
    for probability in [power, 1.0 - power] {
        if probability > 0.0 {
            value -= probability * probability.log2();
        }
    }
    unit(value)?;
    Ok(value.clamp(0.0, 1.0))
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Binding(pub f64);
impl Binding {
    pub fn step(
        &mut self,
        power: f64,
        information: f64,
        habituation: f64,
        factor: f64,
    ) -> Result<(), PrincipleError> {
        unit(power)?;
        unit(information)?;
        unit(habituation)?;
        if !factor.is_finite() || !(0.95..=1.05).contains(&factor) {
            return Err(PrincipleError::InvalidState);
        }
        let rate = 0.1 * (1.0 + 0.1 * information) * factor;
        self.0 += rate * power * (1.0 - self.0) / (1.0 + habituation) - 0.02 * self.0;
        unit(self.0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Habituation(pub f64);
impl Habituation {
    pub fn step(&mut self, power: f64) -> Result<(), PrincipleError> {
        unit(power)?;
        self.0 = 0.95 * self.0 + 0.05 * power;
        unit(self.0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Spring(pub f64);
impl Spring {
    pub fn step(
        &mut self,
        binding: f64,
        power: f64,
        habituation: f64,
    ) -> Result<f64, PrincipleError> {
        let before = self.0;
        self.0 = 0.9 * self.0 + 0.1 * binding * power / (1.0 + habituation);
        unit(self.0)?;
        Ok(self.0 - before)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Heat(pub f64);
impl Heat {
    pub fn step(&mut self, displacement: f64) -> Result<(), PrincipleError> {
        self.0 = 0.95 * self.0 + 0.05 * displacement * displacement;
        unit(self.0)
    }
}

pub fn unit(value: f64) -> Result<(), PrincipleError> {
    if value.is_finite() && (-1e-10..=1.0 + 1e-10).contains(&value) {
        Ok(())
    } else {
        Err(PrincipleError::InvalidState)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Logistic(pub f64);
impl Logistic {
    pub fn new(seed: u64) -> Self {
        let u = (mix64(seed ^ 0x9b13_7af0_54ce_9821) >> 11) as f64 / ((1u64 << 53) as f64);
        Self(0.1 + 0.8 * u)
    }
    pub fn next(&mut self) -> Result<f64, PrincipleError> {
        self.0 = 3.91 * self.0 * (1.0 - self.0);
        if self.0.is_finite() && (0.0..=1.0).contains(&self.0) {
            Ok(2.0 * self.0 - 1.0)
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Henon {
    x: f64,
    y: f64,
}
impl Henon {
    pub fn new(seed: u64) -> Self {
        let u = (mix64(seed ^ 0x2bd7_ea41_60f9_35cc) >> 11) as f64 / ((1u64 << 53) as f64);
        Self {
            x: 0.1 + 0.2 * u,
            y: 0.0,
        }
    }
    pub fn next(&mut self) -> Result<f64, PrincipleError> {
        let next = 1.0 - 1.4 * self.x * self.x + self.y;
        self.y = 0.3 * self.x;
        self.x = next;
        if self.x.is_finite() && self.y.is_finite() && self.x.abs() < 10.0 {
            Ok(self.x.tanh())
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coherent_bound_and_decay_hold() -> Result<(), PrincipleError> {
        let mut field = Field::default();
        let pair = phasor(123, 1.0)?;
        for _ in 0..100_000 {
            field.coherent(relaxation(4_000, 0.0)?, pair)?;
        }
        assert!(field.power <= 1.0);
        assert!(field.power > 0.0);
        Ok(())
    }
}
