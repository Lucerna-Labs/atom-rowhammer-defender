//! Intrinsic mathematical states for paired binary observations. This crate
//! has no file, attacker, policy, or expected-result vocabulary.

use std::f64::consts::FRAC_PI_2;

pub const N: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    InvalidBit,
    NonFinite,
    Capacity,
    Matrix,
}
impl std::fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid material state: {self:?}")
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

/// Squared projector distance between two normalized binary preparations.
pub fn source(input: [f64; 8]) -> Result<[f64; N], PrincipleError> {
    let mut vector = [0.0; N];
    for (index, target) in vector.iter_mut().enumerate() {
        let a = input[2 * index];
        let b = input[2 * index + 1];
        if !matches!(a, 0.0 | 1.0) || !matches!(b, 0.0 | 1.0) {
            return Err(PrincipleError::InvalidBit);
        }
        let a_state = [(FRAC_PI_2 * a).cos(), (FRAC_PI_2 * a).sin()];
        let b_state = [(FRAC_PI_2 * b).cos(), (FRAC_PI_2 * b).sin()];
        let inner = a_state[0] * b_state[0] + a_state[1] * b_state[1];
        let distance = finite((1.0 - inner * inner).clamp(0.0, 1.0))?;
        *target = distance.sqrt();
    }
    Ok(vector)
}

/// An unnormalized PSD statistical occupation matrix. The source adds a
/// rank-one outer product and an orthogonal transport redistributes it.
#[derive(Clone, Debug, Default)]
pub struct Occupation {
    pub matrix: [[f64; N]; N],
}
impl Occupation {
    pub fn observe(
        &mut self,
        source: [f64; N],
        angle: f64,
        population: bool,
    ) -> Result<(), PrincipleError> {
        let angle = finite(angle)?;
        if population {
            for (index, row) in self.matrix.iter_mut().enumerate() {
                row[index] = 0.95 * row[index] + 0.12 * source[index] * source[index];
            }
        } else {
            for left in 0..N {
                let right = (left + 1) % N;
                let (sine, cosine) = angle.sin_cos();
                for column in 0..N {
                    let a = self.matrix[left][column];
                    let b = self.matrix[right][column];
                    self.matrix[left][column] = cosine * a - sine * b;
                    self.matrix[right][column] = sine * a + cosine * b;
                }
                for row in 0..N {
                    let a = self.matrix[row][left];
                    let b = self.matrix[row][right];
                    self.matrix[row][left] = cosine * a - sine * b;
                    self.matrix[row][right] = sine * a + cosine * b;
                }
            }
            for row in &mut self.matrix {
                for value in row {
                    *value *= 0.95;
                }
            }
            for i in 0..N {
                for j in 0..N {
                    self.matrix[i][j] += 0.12 * source[i] * source[j];
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
        std::array::from_fn(|index| self.matrix[index][index])
    }
    pub fn trace(&self) -> f64 {
        self.diagonal().iter().sum()
    }
    pub fn coherence(&self) -> f64 {
        (0..N)
            .flat_map(|i| (i + 1..N).map(move |j| self.matrix[i][j].powi(2)))
            .sum()
    }
    pub fn free_energy(&self) -> f64 {
        -self.coherence()
    }
    pub fn valid(&self) -> bool {
        if !self.trace().is_finite() || self.trace() < -1e-10 || self.trace() > 9.6 + 1e-9 {
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

/// Shannon entropy of the current diagonal occupation distribution.
pub fn entropy(diagonal: [f64; N]) -> Result<f64, PrincipleError> {
    let total: f64 = diagonal.iter().sum();
    if total <= 1e-12 {
        return Ok(0.0);
    }
    finite(
        diagonal
            .iter()
            .filter(|&&x| x > 0.0)
            .map(|&x| {
                let p = x / total;
                -p * p.ln()
            })
            .sum::<f64>()
            / (N as f64).ln(),
    )
}

/// Free-site association and slow desorption.
#[derive(Clone, Debug, Default)]
pub struct Binding {
    pub occupied: [f64; N],
}
impl Binding {
    pub fn observe(
        &mut self,
        diagonal: [f64; N],
        information: f64,
        habituation: [f64; N],
        rate_factor: f64,
    ) -> Result<[f64; N], PrincipleError> {
        let information = finite(information)?;
        let rate_factor = finite(rate_factor)?;
        for i in 0..N {
            let rate = 0.2 * (1.0 + 0.2 * information) * finite(diagonal[i])? * rate_factor
                / (1.0 + finite(habituation[i])?);
            self.occupied[i] = finite(0.98 * self.occupied[i] + rate * (1.0 - self.occupied[i]))?;
        }
        if self.valid() {
            Ok(self.occupied)
        } else {
            Err(PrincipleError::Capacity)
        }
    }
    pub fn free_energy(&self) -> f64 {
        -self.occupied.iter().sum::<f64>()
    }
    pub fn valid(&self) -> bool {
        self.occupied
            .iter()
            .all(|n| n.is_finite() && (0.0..=1.0).contains(n))
    }
}

/// Activity-driven habituation and recovery.
#[derive(Clone, Debug, Default)]
pub struct Habituation {
    pub activity: [f64; N],
}
impl Habituation {
    pub fn observe(&mut self, diagonal: [f64; N]) -> Result<[f64; N], PrincipleError> {
        for i in 0..N {
            self.activity[i] = finite(0.95 * self.activity[i] + 0.05 * finite(diagonal[i])?)?;
        }
        Ok(self.activity)
    }
    pub fn salience(&self, diagonal: [f64; N]) -> Result<[f64; N], PrincipleError> {
        let mut values = [0.0; N];
        for i in 0..N {
            values[i] = finite(diagonal[i] / (1.0 + self.activity[i]))?;
        }
        Ok(values)
    }
    pub fn valid(&self) -> bool {
        self.activity.iter().all(|n| n.is_finite() && *n >= 0.0)
    }
}

/// Overdamped bistable material recurrence, zero fixed in the absence of drive.
#[derive(Clone, Debug, Default)]
pub struct Material {
    pub state: [f64; N],
}
impl Material {
    pub fn observe(
        &mut self,
        salience: [f64; N],
        binding: [f64; N],
    ) -> Result<[f64; N], PrincipleError> {
        for i in 0..N {
            let argument = finite(
                1.5 * self.state[i] + 2.0 * (finite(salience[i])? + 0.1 * finite(binding[i])?),
            )?;
            self.state[i] = finite(argument.tanh())?;
        }
        Ok(self.state)
    }
    pub fn valid(&self) -> bool {
        self.state
            .iter()
            .all(|n| n.is_finite() && (-1.0..=1.0).contains(n))
    }
}

/// Dissipation of material work.
#[derive(Clone, Debug, Default)]
pub struct Heat {
    pub stored: [f64; N],
}
impl Heat {
    pub fn observe(&mut self, material: [f64; N]) -> Result<[f64; N], PrincipleError> {
        for i in 0..N {
            self.stored[i] = finite(0.9 * self.stored[i] + 0.1 * finite(material[i])?.powi(2))?;
        }
        Ok(self.stored)
    }
    pub fn valid(&self) -> bool {
        self.stored.iter().all(|n| n.is_finite() && *n >= 0.0)
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
    fn clean_zero_and_single_perturbation_are_finite() {
        let mut matrix = Occupation::default();
        let clean = source([0.0; 8]).unwrap();
        for _ in 0..50 {
            matrix.observe(clean, 0.08, false).unwrap();
        }
        assert_eq!(matrix.trace(), 0.0);
        let pulse = source([0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]).unwrap();
        matrix.observe(pulse, 0.08, false).unwrap();
        assert!((matrix.trace() - 0.12).abs() < 1e-10);
        assert!(matrix.valid());
        assert_eq!(
            source([0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            Err(PrincipleError::InvalidBit)
        );
    }
}
