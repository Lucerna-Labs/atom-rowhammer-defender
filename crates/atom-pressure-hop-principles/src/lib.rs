//! Three-site intrinsic density and material primitives; no behavioral labels.
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    InvalidInput,
    InvalidState,
}
impl fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => f.write_str("input must be finite and in [0,1]"),
            Self::InvalidState => f.write_str("three-site material invariant failed"),
        }
    }
}
impl std::error::Error for PrincipleError {}

#[derive(Clone, Copy, Debug)]
pub struct Density {
    pub matrix: [[f64; 3]; 3],
}
impl Default for Density {
    fn default() -> Self {
        Self {
            matrix: [[1.0, 0.0, 0.0], [0.0; 3], [0.0; 3]],
        }
    }
}
impl Density {
    pub fn rotate(&mut self, left: usize, right: usize, theta: f64) -> Result<(), PrincipleError> {
        if left >= 3 || right >= 3 || left == right || !theta.is_finite() {
            return Err(PrincipleError::InvalidInput);
        }
        let (s, c) = theta.sin_cos();
        let mut u = [[0.0; 3]; 3];
        for (i, row) in u.iter_mut().enumerate() {
            row[i] = 1.0;
        }
        u[left][left] = c;
        u[right][right] = c;
        u[left][right] = -s;
        u[right][left] = s;
        let mut tmp = [[0.0; 3]; 3];
        let mut next = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    tmp[i][j] += u[i][k] * self.matrix[k][j];
                }
            }
        }
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
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
        let keep = 1.0 - delta;
        for i in 0..3 {
            for j in 0..3 {
                self.matrix[i][j] *= keep;
            }
        }
        self.matrix[0][0] += delta;
        self.validate()
    }

    pub fn discard_coherence(&mut self) -> Result<(), PrincipleError> {
        for i in 0..3 {
            for j in 0..3 {
                if i != j {
                    self.matrix[i][j] = 0.0;
                }
            }
        }
        self.validate()
    }

    pub fn diffuse(&mut self, left: usize, right: usize, theta: f64) -> Result<(), PrincipleError> {
        if left >= 3 || right >= 3 || left == right || !theta.is_finite() {
            return Err(PrincipleError::InvalidInput);
        }
        let p = theta.sin().powi(2);
        let a = self.matrix[left][left];
        let b = self.matrix[right][right];
        self.matrix[left][left] = (1.0 - p) * a + p * b;
        self.matrix[right][right] = p * a + (1.0 - p) * b;
        self.validate()
    }

    pub fn occupation(&self, site: usize) -> Result<f64, PrincipleError> {
        self.validate()?;
        self.matrix
            .get(site)
            .map(|row| row[site])
            .ok_or(PrincipleError::InvalidInput)
    }

    pub fn determinant(&self) -> f64 {
        let m = &self.matrix;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    pub fn validate(&self) -> Result<(), PrincipleError> {
        let m = &self.matrix;
        let tr = m[0][0] + m[1][1] + m[2][2];
        let minors = [
            m[0][0] * m[1][1] - m[0][1] * m[1][0],
            m[0][0] * m[2][2] - m[0][2] * m[2][0],
            m[1][1] * m[2][2] - m[1][2] * m[2][1],
        ];
        if m.iter().flatten().all(|x| x.is_finite())
            && (tr - 1.0).abs() <= 1e-9
            && (0..3).all(|i| m[i][i] >= -1e-10 && m[i][i] <= 1.0 + 1e-10)
            && (0..3).all(|i| (0..3).all(|j| (m[i][j] - m[j][i]).abs() <= 1e-10))
            && minors.iter().all(|x| *x >= -1e-10)
            && self.determinant() >= -1e-10
        {
            Ok(())
        } else {
            Err(PrincipleError::InvalidState)
        }
    }
}

pub fn entropy(density: Density) -> Result<f64, PrincipleError> {
    density.validate()?;
    let h = (0..3)
        .map(|i| density.matrix[i][i])
        .filter(|p| *p > 0.0)
        .map(|p| -p * p.ln() / 3.0_f64.ln())
        .sum::<f64>();
    unit(h)?;
    Ok(h.clamp(0.0, 1.0))
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Binding(pub f64);
impl Binding {
    pub fn step(&mut self, p: f64, fraction: f64, association: f64) -> Result<(), PrincipleError> {
        unit(p)?;
        unit(fraction)?;
        if !association.is_finite() || !(0.19..=0.21).contains(&association) {
            return Err(PrincipleError::InvalidInput);
        }
        self.0 += association * fraction * p * (1.0 - self.0) - 0.08 * self.0;
        unit(self.0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Habituation(pub f64);
impl Habituation {
    pub fn step(&mut self, p: f64) -> Result<(), PrincipleError> {
        unit(p)?;
        self.0 = 0.9 * self.0 + 0.1 * p;
        unit(self.0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Spring(pub f64);
impl Spring {
    pub fn step(&mut self, binding: f64, p: f64, habituation: f64) -> Result<f64, PrincipleError> {
        let before = self.0;
        self.0 = 0.7 * self.0 + 0.3 * binding * p / (1.0 + habituation);
        unit(self.0)?;
        Ok(self.0 - before)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Heat(pub f64);
impl Heat {
    pub fn step(&mut self, displacement: f64) -> Result<(), PrincipleError> {
        self.0 = 0.9 * self.0 + 0.1 * displacement * displacement;
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
        let u = mix64(seed ^ 0x43e1_791a_278b_50dc) >> 11;
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
        let u = (mix64(seed ^ 0x603d_b278_a251_1e94) >> 11) as f64 / ((1u64 << 53) as f64);
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

fn mix64(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn two_hop_channel_preserves_density_invariants() -> Result<(), PrincipleError> {
        let mut d = Density::default();
        for i in 0..2000 {
            let a = std::f64::consts::PI * (i % 97) as f64 / 3104.0;
            d.rotate(0, 1, a)?;
            d.rotate(1, 2, a)?;
            d.reset(0.16 * (i % 41) as f64 / 41.0)?;
            d.validate()?;
        }
        Ok(())
    }
}
