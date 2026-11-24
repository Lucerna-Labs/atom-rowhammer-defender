//! Intrinsic phase and material laws over numeric observations. No endpoint
//! identity, expected behavior, or response policy enters this crate.

use std::collections::VecDeque;
use std::f64::consts::{FRAC_1_SQRT_2, PI};

pub const N: usize = 4;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    BadScale,
    BackwardTime,
    NonFinite,
    Capacity,
    Matrix,
}
impl std::fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid four-mode state: {self:?}")
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

/// Fixed time scale supplied with a complete observation stream.
#[derive(Clone, Debug)]
pub struct PhaseClock {
    tau: f64,
    origin: Option<u64>,
    last: Option<u64>,
}
impl PhaseClock {
    pub fn new(tau: f64) -> Result<Self, PrincipleError> {
        if !tau.is_finite() || tau <= 0.0 {
            return Err(PrincipleError::BadScale);
        }
        Ok(Self {
            tau,
            origin: None,
            last: None,
        })
    }
    pub fn phase_for(&self, time_ns: u64) -> Result<f64, PrincipleError> {
        if self.last.is_some_and(|last| time_ns <= last) {
            return Err(PrincipleError::BackwardTime);
        }
        let origin = self.origin.unwrap_or(time_ns);
        finite(PI * (time_ns - origin) as f64 / self.tau)
    }
    pub fn commit(&mut self, time_ns: u64) {
        if self.origin.is_none() {
            self.origin = Some(time_ns);
        }
        self.last = Some(time_ns);
    }
    pub fn valid(&self) -> bool {
        self.tau.is_finite() && self.tau > 0.0
    }
}

/// Normalized four-mode preparation from two coupled temporal harmonics.
pub fn prepare(phase: f64) -> Result<[f64; N], PrincipleError> {
    let phase = finite(phase)?;
    let vector = [
        phase.cos(),
        phase.sin(),
        (2.0 * phase).cos(),
        (2.0 * phase).sin(),
    ]
    .map(|v| v * FRAC_1_SQRT_2);
    if vector.iter().all(|v| v.is_finite()) {
        Ok(vector)
    } else {
        Err(PrincipleError::NonFinite)
    }
}
pub fn norm(vector: [f64; N]) -> f64 {
    vector.iter().map(|v| v * v).sum()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Base {
    Quantum,
    Population,
    Diffusion,
}
#[derive(Clone, Debug)]
pub struct Occupation {
    pub matrix: [[f64; N]; N],
}
impl Default for Occupation {
    fn default() -> Self {
        let mut matrix = [[0.0; N]; N];
        for (i, row) in matrix.iter_mut().enumerate() {
            row[i] = 1.0 / N as f64;
        }
        Self { matrix }
    }
}
impl Occupation {
    fn transport(&mut self, angle: f64) -> Result<(), PrincipleError> {
        let (s, c) = finite(angle)?.sin_cos();
        for (left, right) in [(0, 1), (2, 3)] {
            for column in 0..N {
                let a = self.matrix[left][column];
                let b = self.matrix[right][column];
                self.matrix[left][column] = c * a - s * b;
                self.matrix[right][column] = s * a + c * b;
            }
            for row in 0..N {
                let a = self.matrix[row][left];
                let b = self.matrix[row][right];
                self.matrix[row][left] = c * a - s * b;
                self.matrix[row][right] = s * a + c * b;
            }
        }
        Ok(())
    }
    fn source(&mut self, vector: [f64; N]) {
        for i in 0..N {
            for j in 0..N {
                self.matrix[i][j] = 0.95 * self.matrix[i][j] + 0.05 * vector[i] * vector[j];
            }
        }
    }
    pub fn observe(
        &mut self,
        vector: [f64; N],
        angle: f64,
        base: Base,
        transport_first: bool,
    ) -> Result<(), PrincipleError> {
        match base {
            Base::Quantum => {
                if transport_first {
                    self.transport(angle)?;
                    self.source(vector);
                } else {
                    self.source(vector);
                    self.transport(angle)?;
                }
            }
            Base::Population => {
                for i in 0..N {
                    self.matrix[i][i] = 0.95 * self.matrix[i][i] + 0.05 * vector[i] * vector[i];
                }
            }
            Base::Diffusion => {
                let old = self.diagonal();
                for i in 0..N {
                    self.matrix[i][i] = 0.95
                        * (0.96 * old[i] + 0.02 * (old[(i + N - 1) % N] + old[(i + 1) % N]))
                        + 0.05 * vector[i] * vector[i];
                }
            }
        }
        if self.valid() {
            Ok(())
        } else {
            Err(PrincipleError::Matrix)
        }
    }
    pub fn diagonal(&self) -> [f64; N] {
        std::array::from_fn(|i| self.matrix[i][i])
    }
    pub fn trace(&self) -> f64 {
        self.diagonal().iter().sum()
    }
    pub fn purity(&self) -> f64 {
        self.matrix.iter().flatten().map(|v| v * v).sum()
    }
    pub fn harmonic_energies(&self) -> [f64; 2] {
        [(0, 1), (2, 3)].map(|(a, b)| {
            let difference = self.matrix[a][a] - self.matrix[b][b];
            let coherence = 2.0 * self.matrix[a][b];
            4.0 * (difference * difference + coherence * coherence)
        })
    }
    pub fn free_energy(&self) -> f64 {
        -self.purity()
    }
    pub fn valid(&self) -> bool {
        if !self.trace().is_finite()
            || (self.trace() - 1.0).abs() > 1e-10
            || !(0.25 - 1e-10..=1.0 + 1e-10).contains(&self.purity())
            || self
                .harmonic_energies()
                .iter()
                .any(|value| !value.is_finite() || !(-1e-10..=1.0 + 1e-10).contains(value))
        {
            return false;
        }
        for i in 0..N {
            for j in 0..N {
                if !self.matrix[i][j].is_finite()
                    || (self.matrix[i][j] - self.matrix[j][i]).abs() > 1e-10
                {
                    return false;
                }
            }
        }
        let mut lower = [[0.0; N]; N];
        for i in 0..N {
            for j in 0..=i {
                let mut residual = self.matrix[i][j];
                for k in 0..j {
                    residual -= lower[i][k] * lower[j][k];
                }
                if i == j {
                    if residual < -1e-10 {
                        return false;
                    }
                    lower[i][j] = residual.max(0.0).sqrt();
                } else if lower[j][j] > 1e-12 {
                    lower[i][j] = residual / lower[j][j];
                } else if residual.abs() > 1e-10 {
                    return false;
                }
            }
        }
        true
    }
}

/// Shannon entropy of the latest64 phase positions.
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

/// Reversible binding under size similarity and information allocation.
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

/// Habituation to retained coherent occupation.
#[derive(Clone, Debug, Default)]
pub struct Adaptation {
    pub value: f64,
}
impl Adaptation {
    pub fn observe(&mut self, purity: f64) -> Result<f64, PrincipleError> {
        let coherence = (finite(purity)? - 0.25) / 0.75;
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

/// Two damped strains with reciprocal inhibition between harmonic modes.
#[derive(Clone, Debug, Default)]
pub struct Strain {
    pub values: [f64; 2],
}
impl Strain {
    pub fn observe(
        &mut self,
        harmonics: [f64; 2],
        binding: f64,
        adaptation: f64,
    ) -> Result<[f64; 2], PrincipleError> {
        for i in 0..2 {
            let other = harmonics[1 - i];
            self.values[i] = finite(
                0.85 * self.values[i]
                    + 0.2 * finite(binding)? * finite(harmonics[i])?
                        / ((1.0 + 4.0 * finite(other)?) * (1.0 + finite(adaptation)?)),
            )?;
        }
        if self.valid() {
            Ok(self.values)
        } else {
            Err(PrincipleError::Capacity)
        }
    }
    pub fn valid(&self) -> bool {
        self.values
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
    }
}

/// Per-harmonic dissipation of strain work.
#[derive(Clone, Debug, Default)]
pub struct Heat {
    pub values: [f64; 2],
}
impl Heat {
    pub fn observe(&mut self, strain: [f64; 2]) -> Result<[f64; 2], PrincipleError> {
        for i in 0..2 {
            self.values[i] = finite(0.9 * self.values[i] + 0.1 * finite(strain[i])?.powi(2))?;
        }
        if self.valid() {
            Ok(self.values)
        } else {
            Err(PrincipleError::Capacity)
        }
    }
    pub fn valid(&self) -> bool {
        self.values
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
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
    fn unit_four_mode_density_and_time_rejection() {
        let mut clock = PhaseClock::new(100_000_000.0).unwrap();
        let mut density = Occupation::default();
        for index in 0..100u64 {
            let time = index * 100_000_000;
            let phase = clock.phase_for(time).unwrap();
            let vector = prepare(phase).unwrap();
            assert!((norm(vector) - 1.0).abs() < 1e-12);
            density.observe(vector, 0.02, Base::Quantum, true).unwrap();
            assert!(density.valid());
            clock.commit(time);
        }
        assert_eq!(clock.phase_for(99), Err(PrincipleError::BackwardTime));
    }
}
