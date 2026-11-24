//! Local material laws; no process names, behavior labels or response actions.
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    InvalidInput,
    InvalidState,
}
impl fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => f.write_str("invalid finite unit input"),
            Self::InvalidState => f.write_str("material state invariant failed"),
        }
    }
}
impl std::error::Error for PrincipleError {}

pub fn unit(value: f64) -> Result<f64, PrincipleError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(value)
    } else {
        Err(PrincipleError::InvalidInput)
    }
}

/// Real two-amplitude state; rotations conserve squared norm.
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
    pub fn norm(self) -> f64 {
        self.ground
            .mul_add(self.ground, self.excited * self.excited)
    }
    pub fn occupation(self) -> f64 {
        self.excited * self.excited
    }
    pub fn rotate(&mut self, angle: f64) -> Result<(), PrincipleError> {
        if !angle.is_finite() || angle.abs() > 1.0 {
            return Err(PrincipleError::InvalidInput);
        }
        let (sin, cos) = angle.sin_cos();
        let ground = self.ground * cos - self.excited * sin;
        let excited = self.ground * sin + self.excited * cos;
        self.ground = ground;
        self.excited = excited;
        self.validate()
    }
    pub fn validate(self) -> Result<(), PrincipleError> {
        if self.ground.is_finite()
            && self.excited.is_finite()
            && (self.norm() - 1.0).abs() <= 1e-10
            && (-1e-10..=1.0 + 1e-10).contains(&self.occupation())
        {
            Ok(())
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
    pub fn alignment(self, material: f64) -> Result<f64, PrincipleError> {
        unit(material)?;
        Ok(1.0 - (self.occupation() - material).abs())
    }
}

/// A site carries bounded material and an unpaired offspring ligand.
#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub material: f64,
    pub unpaired: f64,
    pub children: u8,
}
impl Site {
    pub fn seed() -> Self {
        Self {
            material: 0.5,
            unpaired: 0.0,
            children: 0,
        }
    }
    pub fn offspring(&mut self, occupation: f64) -> Result<(Self, f64), PrincipleError> {
        unit(occupation)?;
        let child_material = (0.35 + 0.45 * self.material + 0.2 * occupation).min(1.0);
        let ligand = child_material * occupation;
        let overlap = self.unpaired * ligand;
        self.unpaired = ligand;
        self.children = self.children.saturating_add(1);
        self.validate()?;
        let child = Self {
            material: child_material,
            unpaired: 0.0,
            children: 0,
        };
        child.validate()?;
        Ok((child, overlap))
    }
    pub fn validate(self) -> Result<(), PrincipleError> {
        if unit(self.material).is_ok() && unit(self.unpaired).is_ok() {
            Ok(())
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

/// A bond reservoir responds to overlap, with bounded chemical relaxation.
#[derive(Clone, Copy, Debug, Default)]
pub struct Bond(pub f64);
impl Bond {
    pub fn advance(&mut self, overlap: f64, dt_ns: u64) -> Result<f64, PrincipleError> {
        unit(overlap)?;
        let decay = (-(dt_ns as f64) / 10_000_000.0).exp();
        self.0 = (self.0 * decay + overlap * (1.0 - self.0)).clamp(0.0, 1.0);
        unit(self.0)?;
        Ok(overlap * (0.7 + 0.3 * self.0))
    }
}

/// Heat stores bounded work and dissipates independently of site identity.
#[derive(Clone, Copy, Debug, Default)]
pub struct Heat(pub f64);
impl Heat {
    pub fn advance(&mut self, work: f64, dt_ns: u64) -> Result<f64, PrincipleError> {
        if !work.is_finite() || !(0.0..=1.0).contains(&work) {
            return Err(PrincipleError::InvalidInput);
        }
        self.0 = self.0 * (-(dt_ns as f64) / 10_000_000.0).exp() + work;
        if self.0.is_finite() && (0.0..=64.0).contains(&self.0) {
            Ok(self.0)
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

/// Centered logistic source; one small deterministic variation per birth.
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
        unit(self.0)?;
        Ok(2.0 * self.0 - 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn phase_norm_and_material_bounds() -> Result<(), PrincipleError> {
        let mut phase = Phase::default();
        let mut site = Site::seed();
        let mut bond = Bond::default();
        let mut heat = Heat::default();
        for _ in 0..64 {
            phase.rotate(0.4)?;
            let (_, overlap) = site.offspring(phase.occupation())?;
            heat.advance(bond.advance(overlap, 100_000)?, 100_000)?;
        }
        phase.validate()?;
        assert!(site.unpaired <= 1.0 && heat.0 <= 64.0);
        Ok(())
    }
}
