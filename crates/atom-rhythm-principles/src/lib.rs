//! Intrinsic timing and state mechanics. Inputs are numeric observations;
//! no endpoint identity, attack label, or response policy enters this crate.

use std::collections::VecDeque;
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    BackwardTime,
    NonFinite,
    Capacity,
}
impl std::fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid rhythm state: {self:?}")
    }
}
impl std::error::Error for PrincipleError {}
pub fn finite(value: f64) -> Result<f64, PrincipleError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(PrincipleError::NonFinite)
    }
}

/// Period memory from a damped oscillator; phase uses the prior estimate.
#[derive(Clone, Debug, Default)]
pub struct Period {
    origin: Option<u64>,
    last: Option<u64>,
    pub tau: Option<f64>,
}
impl Period {
    pub fn prepare(&self, time_ns: u64) -> Result<(f64, Option<u64>), PrincipleError> {
        let Some(last) = self.last else {
            return Ok((0.0, None));
        };
        if time_ns <= last {
            return Err(PrincipleError::BackwardTime);
        }
        let gap = time_ns - last;
        let tau = self.tau.unwrap_or(gap as f64);
        if tau <= 0.0 || !tau.is_finite() {
            return Err(PrincipleError::NonFinite);
        }
        let origin = self.origin.ok_or(PrincipleError::NonFinite)?;
        let phase = finite(PI * ((time_ns - origin) as f64) / tau)?;
        Ok((phase, Some(gap)))
    }
    pub fn commit(&mut self, time_ns: u64, gap: Option<u64>) -> Result<(), PrincipleError> {
        if self.origin.is_none() {
            self.origin = Some(time_ns);
        }
        if let Some(gap) = gap {
            self.tau = Some(finite(
                self.tau
                    .map_or(gap as f64, |tau| 0.95 * tau + 0.05 * gap as f64),
            )?);
        }
        self.last = Some(time_ns);
        Ok(())
    }
    pub fn valid(&self) -> bool {
        self.tau.is_none_or(|tau| tau.is_finite() && tau > 0.0)
    }
}

/// A two-amplitude projector mixture. Trace one and positive semidefinite.
#[derive(Clone, Debug)]
pub struct Density {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}
impl Default for Density {
    fn default() -> Self {
        Self {
            a: 0.5,
            b: 0.0,
            c: 0.5,
        }
    }
}
impl Density {
    pub fn observe(&mut self, phase: f64) -> Result<f64, PrincipleError> {
        let phase = finite(phase)?;
        let (sine, cosine) = phase.sin_cos();
        self.a = finite(0.95 * self.a + 0.05 * cosine * cosine)?;
        self.b = finite(0.95 * self.b + 0.05 * cosine * sine)?;
        self.c = finite(0.95 * self.c + 0.05 * sine * sine)?;
        if !self.valid() {
            return Err(PrincipleError::Capacity);
        }
        Ok(self.purity())
    }
    pub fn purity(&self) -> f64 {
        self.a * self.a + 2.0 * self.b * self.b + self.c * self.c
    }
    pub fn free_energy(&self) -> f64 {
        -self.purity()
    }
    pub fn valid(&self) -> bool {
        self.a.is_finite()
            && self.b.is_finite()
            && self.c.is_finite()
            && (self.a + self.c - 1.0).abs() < 1e-10
            && self.a >= -1e-12
            && self.c >= -1e-12
            && self.a * self.c - self.b * self.b >= -1e-10
            && (0.5 - 1e-10..=1.0 + 1e-10).contains(&self.purity())
    }
}

/// Matched classical circular memory with the same source phase and loss.
#[derive(Clone, Debug, Default)]
pub struct Circular {
    pub x: f64,
    pub y: f64,
}
impl Circular {
    pub fn observe(&mut self, phase: f64) -> Result<f64, PrincipleError> {
        let phase = 2.0 * finite(phase)?;
        self.x = finite(0.95 * self.x + 0.05 * phase.cos())?;
        self.y = finite(0.95 * self.y + 0.05 * phase.sin())?;
        let purity = finite((1.0 + self.x * self.x + self.y * self.y) / 2.0)?;
        if !(0.5 - 1e-10..=1.0 + 1e-10).contains(&purity) {
            return Err(PrincipleError::Capacity);
        }
        Ok(purity)
    }
    pub fn valid(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

/// Bounded Shannon information of the latest phase bins.
#[derive(Clone, Debug, Default)]
pub struct PhaseInformation {
    bins: [u8; 8],
    order: VecDeque<usize>,
}
impl PhaseInformation {
    pub fn observe(&mut self, phase: f64) -> Result<f64, PrincipleError> {
        let position = finite(phase)?.rem_euclid(PI) / PI;
        let bin = ((position * 8.0).floor() as usize).min(7);
        if self.order.len() == 64 {
            if let Some(old) = self.order.pop_front() {
                self.bins[old] -= 1;
            }
        }
        self.order.push_back(bin);
        self.bins[bin] += 1;
        let total = self.order.len() as f64;
        finite(
            self.bins
                .iter()
                .filter(|&&n| n > 0)
                .map(|&n| {
                    let p = n as f64 / total;
                    -p * p.ln()
                })
                .sum::<f64>()
                / (8.0f64).ln(),
        )
    }
    pub fn retained(&self) -> usize {
        self.order.len()
    }
    pub fn conserved(&self) -> bool {
        self.bins.iter().map(|&n| n as usize).sum::<usize>() == self.order.len()
    }
}

/// Reversible size binding with entropy and adaptation coupling.
#[derive(Clone, Debug, Default)]
pub struct Binding {
    pub occupied: f64,
    previous: Option<u32>,
}
impl Binding {
    pub fn observe(
        &mut self,
        bytes: u32,
        entropy: f64,
        adaptation: f64,
        rate_factor: f64,
    ) -> Result<f64, PrincipleError> {
        let similarity = self.previous.map_or(1.0, |prior| {
            let delta = (i64::from(bytes) - i64::from(prior)).unsigned_abs() as f64;
            (-delta / (f64::from(prior) + 1.0)).exp()
        });
        let association = 0.1
            * similarity
            * (1.0 - self.occupied)
            * (1.0 + 0.2 * (1.0 - finite(entropy)?))
            * finite(rate_factor)?
            / (1.0 + finite(adaptation)?);
        self.occupied = finite(0.95 * self.occupied + association)?;
        self.previous = Some(bytes);
        if self.valid() {
            Ok(self.occupied)
        } else {
            Err(PrincipleError::Capacity)
        }
    }
    pub fn free_energy(&self) -> f64 {
        -self.occupied
    }
    pub fn valid(&self) -> bool {
        self.occupied.is_finite() && (0.0..=1.0).contains(&self.occupied)
    }
}

/// Habituation of sustained coherent input.
#[derive(Clone, Debug, Default)]
pub struct Adaptation {
    pub value: f64,
}
impl Adaptation {
    pub fn observe(&mut self, purity: f64) -> Result<f64, PrincipleError> {
        let coherence = 2.0 * (finite(purity)? - 0.5);
        self.value = finite(0.95 * self.value + 0.05 * coherence)?;
        if self.valid() {
            Ok(self.value)
        } else {
            Err(PrincipleError::Capacity)
        }
    }
    pub fn valid(&self) -> bool {
        self.value.is_finite() && (0.0..=1.0 + 1e-10).contains(&self.value)
    }
}

/// Damped strain driven by coupled phase, binding and adaptation states.
#[derive(Clone, Debug, Default)]
pub struct Strain {
    pub value: f64,
}
impl Strain {
    pub fn observe(
        &mut self,
        purity: f64,
        binding: f64,
        adaptation: f64,
    ) -> Result<f64, PrincipleError> {
        let force = 0.2 * (finite(purity)? - 0.5) * finite(binding)? / (1.0 + finite(adaptation)?);
        self.value = finite(0.85 * self.value + force)?;
        if self.valid() {
            Ok(self.value)
        } else {
            Err(PrincipleError::Capacity)
        }
    }
    pub fn valid(&self) -> bool {
        self.value.is_finite() && (0.0..=1.0).contains(&self.value)
    }
}

/// Dissipation of mechanical work.
#[derive(Clone, Debug, Default)]
pub struct Heat {
    pub value: f64,
}
impl Heat {
    pub fn observe(&mut self, strain: f64) -> Result<f64, PrincipleError> {
        self.value = finite(0.9 * self.value + 0.1 * finite(strain)?.powi(2))?;
        if self.valid() {
            Ok(self.value)
        } else {
            Err(PrincipleError::Capacity)
        }
    }
    pub fn valid(&self) -> bool {
        self.value.is_finite() && (0.0..=1.0).contains(&self.value)
    }
}

pub fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
#[derive(Clone, Debug)]
pub struct Logistic {
    x: f64,
}
impl Logistic {
    pub fn new(seed: u64) -> Self {
        Self {
            x: 0.1 + 0.8 * (mix(seed) as f64 / u64::MAX as f64),
        }
    }
    pub fn step(&mut self) -> f64 {
        self.x = 4.0 * self.x * (1.0 - self.x);
        2.0 * self.x - 1.0
    }
}
#[derive(Clone, Debug)]
pub struct Henon {
    x: f64,
    y: f64,
}
impl Henon {
    pub fn new(seed: u64) -> Self {
        Self {
            x: 0.05 + 0.1 * (mix(seed) as f64 / u64::MAX as f64),
            y: 0.1,
        }
    }
    pub fn step(&mut self) -> f64 {
        let next = 1.0 - 1.4 * self.x * self.x + self.y;
        self.y = 0.3 * self.x;
        self.x = next;
        self.x.rem_euclid(1.0) - 0.486
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn density_and_classical_resultant_have_matching_purity() {
        let mut density = Density::default();
        let mut classical = Circular::default();
        for index in 0..1000 {
            let phase = index as f64 * 0.123;
            let a = density.observe(phase).unwrap();
            let b = classical.observe(phase).unwrap();
            assert!((a - b).abs() < 1e-12);
            assert!(density.valid() && classical.valid());
        }
    }
}
