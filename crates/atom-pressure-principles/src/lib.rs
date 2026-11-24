//! Intrinsic pressure-history primitives. Inputs are unit fractions, not behavior labels.
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    InvalidInput,
    InvalidState,
}
impl fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => f.write_str("fraction must be finite and in [0,1]"),
            Self::InvalidState => f.write_str("material state violated its invariant"),
        }
    }
}
impl std::error::Error for PrincipleError {}

#[derive(Clone, Copy, Debug)]
pub struct Density {
    pub r00: f64,
    pub r01: f64,
    pub r11: f64,
}
impl Default for Density {
    fn default() -> Self {
        Self {
            r00: 1.0,
            r01: 0.0,
            r11: 0.0,
        }
    }
}
impl Density {
    /// Orthogonal two-plane rotation preserves trace and positive semidefiniteness.
    pub fn rotate(&mut self, theta: f64) -> Result<(), PrincipleError> {
        if !theta.is_finite() {
            return Err(PrincipleError::InvalidState);
        }
        let (s, c) = theta.sin_cos();
        let (a, b, d) = (self.r00, self.r01, self.r11);
        self.r00 = c * c * a - 2.0 * c * s * b + s * s * d;
        self.r01 = c * s * (a - d) + (c * c - s * s) * b;
        self.r11 = s * s * a + 2.0 * c * s * b + c * c * d;
        self.validate()
    }

    /// Convex reset toward the ground state. This is a positive trace-preserving channel.
    pub fn reset(&mut self, delta: f64) -> Result<(), PrincipleError> {
        if !delta.is_finite() || !(0.0..=1.0).contains(&delta) {
            return Err(PrincipleError::InvalidInput);
        }
        let keep = 1.0 - delta;
        self.r00 = keep * self.r00 + delta;
        self.r01 *= keep;
        self.r11 *= keep;
        self.validate()
    }

    /// Power-matched diagonal population arm; no cross-level coherence survives.
    pub fn discard_coherence(&mut self) -> Result<(), PrincipleError> {
        self.r01 = 0.0;
        self.validate()
    }

    /// Classical diffusion of populations using the same rotation transition probability.
    pub fn diffuse(&mut self, theta: f64) -> Result<(), PrincipleError> {
        if !theta.is_finite() {
            return Err(PrincipleError::InvalidState);
        }
        let p = theta.sin().powi(2);
        let old = self.r11;
        self.r11 = (1.0 - p) * old + p * (1.0 - old);
        self.r00 = 1.0 - self.r11;
        self.r01 = 0.0;
        self.validate()
    }

    pub fn validate(&self) -> Result<(), PrincipleError> {
        let tr = self.r00 + self.r11;
        let det = self.r00 * self.r11 - self.r01 * self.r01;
        if [self.r00, self.r01, self.r11, tr, det]
            .iter()
            .all(|x| x.is_finite())
            && (tr - 1.0).abs() <= 1e-10
            && self.r00 >= -1e-12
            && self.r11 >= -1e-12
            && det >= -1e-12
            && self.r00 <= 1.0 + 1e-12
            && self.r11 <= 1.0 + 1e-12
        {
            Ok(())
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

/// Normalized diagonal Shannon entropy, including the boundary convention 0 log 0 = 0.
pub fn entropy(state: Density) -> Result<f64, PrincipleError> {
    state.validate()?;
    let h = [state.r00, state.r11]
        .into_iter()
        .filter(|p| *p > 0.0)
        .map(|p| -p * p.log2())
        .sum::<f64>();
    if h.is_finite() && (-1e-12..=1.0 + 1e-12).contains(&h) {
        Ok(h.clamp(0.0, 1.0))
    } else {
        Err(PrincipleError::InvalidState)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Binding(pub f64);
impl Binding {
    pub fn step(&mut self, p1: f64, association: f64) -> Result<(), PrincipleError> {
        if !p1.is_finite()
            || !(0.0..=1.0 + 1e-12).contains(&p1)
            || !association.is_finite()
            || !(0.0..=0.2).contains(&association)
        {
            return Err(PrincipleError::InvalidInput);
        }
        self.0 += association * p1 * (1.0 - self.0) - 0.02 * self.0;
        unit(self.0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Habituation(pub f64);
impl Habituation {
    pub fn step(&mut self, p1: f64) -> Result<(), PrincipleError> {
        self.0 = 0.95 * self.0 + 0.05 * p1;
        unit(self.0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Spring(pub f64);
impl Spring {
    pub fn step(&mut self, bond: f64, p1: f64, habituation: f64) -> Result<f64, PrincipleError> {
        let before = self.0;
        self.0 = 0.85 * self.0 + 0.15 * bond * p1 / (1.0 + habituation);
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
    if value.is_finite() && (-1e-12..=1.0 + 1e-12).contains(&value) {
        Ok(())
    } else {
        Err(PrincipleError::InvalidState)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Logistic(pub f64);
impl Logistic {
    pub fn new(seed: u64) -> Self {
        let u = mix64(seed ^ 0xa53d_1709_76a3_02d1) >> 11;
        Self(0.1 + 0.8 * (u as f64 / ((1u64 << 53) as f64)))
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
        let u = (mix64(seed ^ 0x917d_2235_96b4_10ce) >> 11) as f64 / ((1u64 << 53) as f64);
        Self {
            x: 0.1 + 0.2 * u,
            y: 0.0,
        }
    }
    pub fn next(&mut self) -> Result<f64, PrincipleError> {
        let new_x = 1.0 - 1.4 * self.x * self.x + self.y;
        self.y = 0.3 * self.x;
        self.x = new_x;
        if self.x.is_finite() && self.y.is_finite() && self.x.abs() < 10.0 {
            Ok(self.x.tanh())
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

pub fn mix64(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn density_stays_psd_under_rotation_and_reset() -> Result<(), PrincipleError> {
        let mut d = Density::default();
        for i in 0..2000 {
            d.rotate(std::f64::consts::PI * (i % 101) as f64 / 6400.0)?;
            d.reset(0.15 * (i % 97) as f64 / 97.0)?;
            d.validate()?;
        }
        Ok(())
    }
}
