//! Four independently salted coherent modes and intrinsic material response.
use std::fmt;

pub const N: usize = 4;
const PUMP: f64 = 0.01;
const SALTS: [u64; N] = [
    0x6c20_94f1_ab73_d8e5,
    0x9a57_0cc2_d38f_614b,
    0x42ea_b6f0_19d7_83c5,
    0xdec3_791a_5b40_f268,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    BadTime,
    InvalidState,
}
impl fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadTime => f.write_str("timestamps must increase"),
            Self::InvalidState => f.write_str("phase quorum material invariant failed"),
        }
    }
}
impl std::error::Error for PrincipleError {}

pub fn mix64(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

fn phasor(fingerprint: u64, mode: usize, factor: f64) -> Result<(f64, f64), PrincipleError> {
    if mode >= N || !factor.is_finite() || !(0.95..=1.05).contains(&factor) {
        return Err(PrincipleError::InvalidState);
    }
    let u = (mix64(fingerprint ^ SALTS[mode]) >> 11) as f64 / ((1u64 << 53) as f64);
    let angle = std::f64::consts::TAU * u * factor;
    let (s, c) = angle.sin_cos();
    if s.is_finite() && c.is_finite() {
        Ok((c, s))
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
pub struct Complex {
    pub real: f64,
    pub imag: f64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Field {
    pub modes: [Complex; N],
    pub powers: [f64; N],
}
impl Field {
    pub fn coherent(
        &mut self,
        alpha: f64,
        fingerprint: u64,
        factor: f64,
    ) -> Result<(), PrincipleError> {
        for index in 0..N {
            let pair = phasor(fingerprint, index, factor)?;
            self.modes[index].real = alpha * self.modes[index].real + PUMP * pair.0;
            self.modes[index].imag = alpha * self.modes[index].imag + PUMP * pair.1;
            self.powers[index] = self.modes[index].real.powi(2) + self.modes[index].imag.powi(2);
        }
        self.validate()
    }

    pub fn population(&mut self, alpha: f64, diffusion: bool) -> Result<(), PrincipleError> {
        for index in 0..N {
            self.powers[index] = if diffusion {
                alpha * self.powers[index] + PUMP * PUMP
            } else {
                alpha * alpha * self.powers[index] + PUMP * PUMP
            };
            self.modes[index] = Complex::default();
        }
        self.validate()
    }

    pub fn collective(&self) -> Result<f64, PrincipleError> {
        self.validate()?;
        let value = self.powers.iter().copied().fold(f64::INFINITY, f64::min);
        unit(value)?;
        Ok(value)
    }

    pub fn information(&self) -> Result<f64, PrincipleError> {
        self.validate()?;
        let total = self.powers.iter().sum::<f64>();
        if total == 0.0 {
            return Ok(0.0);
        }
        let mut value = 0.0;
        for power in self.powers {
            let p = power / total;
            if p > 0.0 {
                value -= p * p.ln() / (N as f64).ln();
            }
        }
        unit(value)?;
        Ok(value.clamp(0.0, 1.0))
    }

    pub fn validate(&self) -> Result<(), PrincipleError> {
        if self
            .modes
            .iter()
            .all(|mode| mode.real.is_finite() && mode.imag.is_finite())
            && self
                .powers
                .iter()
                .all(|power| power.is_finite() && (-1e-10..=1.0 + 1e-10).contains(power))
        {
            Ok(())
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Binding(pub f64);
impl Binding {
    pub fn step(
        &mut self,
        collective: f64,
        info: f64,
        habituation: f64,
        factor: f64,
    ) -> Result<(), PrincipleError> {
        for value in [collective, info, habituation] {
            unit(value)?;
        }
        if !factor.is_finite() || !(0.95..=1.05).contains(&factor) {
            return Err(PrincipleError::InvalidState);
        }
        self.0 += 0.1 * (1.0 + 0.1 * info) * factor * collective * (1.0 - self.0)
            / (1.0 + habituation)
            - 0.02 * self.0;
        unit(self.0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Habituation(pub f64);
impl Habituation {
    pub fn step(&mut self, collective: f64) -> Result<(), PrincipleError> {
        unit(collective)?;
        self.0 = 0.95 * self.0 + 0.05 * collective;
        unit(self.0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Spring(pub f64);
impl Spring {
    pub fn step(
        &mut self,
        binding: f64,
        collective: f64,
        habituation: f64,
    ) -> Result<f64, PrincipleError> {
        let before = self.0;
        self.0 = 0.9 * self.0 + 0.1 * binding * collective / (1.0 + habituation);
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
        let u = (mix64(seed ^ 0x73a1_04d9_b82e_5c6f) >> 11) as f64 / ((1u64 << 53) as f64);
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
        let u = (mix64(seed ^ 0xb420_e679_1fca_8d35) >> 11) as f64 / ((1u64 << 53) as f64);
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
    fn four_coherent_modes_stay_bounded() -> Result<(), PrincipleError> {
        let mut field = Field::default();
        for _ in 0..100_000 {
            field.coherent(relaxation(4_000, 0.0)?, 123, 1.0)?;
        }
        assert!(field.powers.iter().all(|p| *p <= 1.0));
        assert!(field.collective()? > 0.0);
        Ok(())
    }
}
