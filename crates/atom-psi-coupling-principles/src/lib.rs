//! Four-state density and material primitives for two unit perturbations.
use std::fmt;

pub const N: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    InvalidInput,
    InvalidState,
}
impl fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => f.write_str("inputs must be finite unit fractions"),
            Self::InvalidState => f.write_str("four-state material invariant failed"),
        }
    }
}
impl std::error::Error for PrincipleError {}

#[derive(Clone, Copy, Debug)]
pub struct Density {
    pub matrix: [[f64; N]; N],
}
impl Default for Density {
    fn default() -> Self {
        let mut matrix = [[0.0; N]; N];
        matrix[0][0] = 1.0;
        Self { matrix }
    }
}
impl Density {
    pub fn rotate(&mut self, left: usize, right: usize, angle: f64) -> Result<(), PrincipleError> {
        if left >= N || right >= N || left == right || !angle.is_finite() {
            return Err(PrincipleError::InvalidInput);
        }
        let (s, c) = angle.sin_cos();
        let mut u = [[0.0; N]; N];
        for (i, row) in u.iter_mut().enumerate() {
            row[i] = 1.0;
        }
        u[left][left] = c;
        u[right][right] = c;
        u[left][right] = -s;
        u[right][left] = s;
        let mut tmp = [[0.0; N]; N];
        let mut next = [[0.0; N]; N];
        for i in 0..N {
            for j in 0..N {
                for k in 0..N {
                    tmp[i][j] += u[i][k] * self.matrix[k][j];
                }
            }
        }
        for i in 0..N {
            for j in 0..N {
                for k in 0..N {
                    next[i][j] += tmp[i][k] * u[j][k];
                }
            }
        }
        self.matrix = next;
        self.validate()
    }

    pub fn reset(&mut self, delta: f64) -> Result<(), PrincipleError> {
        if !delta.is_finite() || !(0.0..=1.0).contains(&delta) {
            return Err(PrincipleError::InvalidInput);
        }
        for i in 0..N {
            for j in 0..N {
                self.matrix[i][j] *= 1.0 - delta;
            }
        }
        self.matrix[0][0] += delta;
        self.validate()
    }

    pub fn discard_coherence(&mut self) -> Result<(), PrincipleError> {
        for i in 0..N {
            for j in 0..N {
                if i != j {
                    self.matrix[i][j] = 0.0;
                }
            }
        }
        self.validate()
    }

    pub fn diffuse(&mut self, left: usize, right: usize, angle: f64) -> Result<(), PrincipleError> {
        if left >= N || right >= N || left == right || !angle.is_finite() {
            return Err(PrincipleError::InvalidInput);
        }
        let p = angle.sin().powi(2);
        let a = self.matrix[left][left];
        let b = self.matrix[right][right];
        self.matrix[left][left] = (1.0 - p) * a + p * b;
        self.matrix[right][right] = p * a + (1.0 - p) * b;
        self.validate()
    }

    pub fn trace(&self) -> f64 {
        (0..N).map(|i| self.matrix[i][i]).sum()
    }

    /// The minimum principal minor checks PSD of this real symmetric 4x4 state.
    pub fn minimum_principal_minor(&self) -> f64 {
        let mut minimum = f64::INFINITY;
        for mask in 1..(1 << N) {
            let mut indices = [0usize; N];
            let mut count = 0;
            for i in 0..N {
                if (mask >> i) & 1 == 1 {
                    indices[count] = i;
                    count += 1;
                }
            }
            minimum = minimum.min(determinant(
                &self.matrix,
                &indices[..count],
                &indices[..count],
            ));
        }
        minimum
    }

    pub fn validate(&self) -> Result<(), PrincipleError> {
        let m = &self.matrix;
        if m.iter().flatten().all(|x| x.is_finite())
            && (self.trace() - 1.0).abs() <= 1e-9
            && (0..N).all(|i| (0..N).all(|j| (m[i][j] - m[j][i]).abs() <= 1e-10))
            && self.minimum_principal_minor() >= -1e-9
        {
            Ok(())
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

fn determinant(matrix: &[[f64; N]; N], rows: &[usize], cols: &[usize]) -> f64 {
    if rows.len() == 1 {
        return matrix[rows[0]][cols[0]];
    }
    let mut sum = 0.0;
    for (index, &col) in cols.iter().enumerate() {
        let mut remaining = [0usize; N];
        let mut count = 0;
        for (position, &candidate) in cols.iter().enumerate() {
            if position != index {
                remaining[count] = candidate;
                count += 1;
            }
        }
        let sign = if index % 2 == 0 { 1.0 } else { -1.0 };
        sum += sign * matrix[rows[0]][col] * determinant(matrix, &rows[1..], &remaining[..count]);
    }
    sum
}

pub fn entropy(density: Density) -> Result<f64, PrincipleError> {
    density.validate()?;
    let value = (0..N)
        .map(|i| density.matrix[i][i])
        .filter(|p| *p > 0.0)
        .map(|p| -p * p.ln() / (N as f64).ln())
        .sum::<f64>();
    unit(value)?;
    Ok(value.clamp(0.0, 1.0))
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Binding(pub f64);
impl Binding {
    pub fn step(&mut self, joint: f64, association: f64) -> Result<(), PrincipleError> {
        unit(joint)?;
        if !association.is_finite() || !(0.095..=0.105 + 1e-12).contains(&association) {
            return Err(PrincipleError::InvalidInput);
        }
        self.0 += association * joint * (1.0 - self.0) - 0.02 * self.0;
        unit(self.0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Habituation(pub f64);
impl Habituation {
    pub fn step(&mut self, joint: f64) -> Result<(), PrincipleError> {
        unit(joint)?;
        self.0 = 0.95 * self.0 + 0.05 * joint;
        unit(self.0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Spring(pub f64);
impl Spring {
    pub fn step(
        &mut self,
        binding: f64,
        joint: f64,
        adaptation: f64,
    ) -> Result<f64, PrincipleError> {
        let before = self.0;
        self.0 = 0.85 * self.0 + 0.15 * binding * joint / (1.0 + adaptation);
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
        let u = mix64(seed ^ 0x4d9b_2073_2c7a_581f) >> 11;
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
        let u = (mix64(seed ^ 0x84e9_5a13_90b2_46d7) >> 11) as f64 / ((1u64 << 53) as f64);
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

fn mix64(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn correlated_and_local_channels_keep_psd_trace() -> Result<(), PrincipleError> {
        let mut d = Density::default();
        for i in 0..1000 {
            let a = std::f64::consts::PI * (i % 97) as f64 / 12_416.0;
            d.rotate(0, 2, a)?;
            d.rotate(0, 1, a)?;
            d.rotate(0, 3, a)?;
            d.reset(0.12 * (i % 41) as f64 / 41.0)?;
            d.validate()?;
        }
        Ok(())
    }
}
