//! Intrinsic numerical dynamics over ordered timing and size observations.
//! No endpoint identity, threat label, or action belongs to these laws.

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
        write!(f, "invalid recoil state: {self:?}")
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

/// A fixed-scale gap and its causal predecessor, without labeling a pattern.
#[derive(Clone, Debug)]
pub struct Gaps {
    tau: f64,
    variance: f64,
    last: Option<u64>,
    previous: Option<f64>,
}
impl Gaps {
    pub fn new(tau: f64, variance: f64) -> Result<Self, PrincipleError> {
        if !tau.is_finite() || tau <= 0.0 || !variance.is_finite() || variance < 0.0 {
            return Err(PrincipleError::BadScale);
        }
        Ok(Self {
            tau,
            variance,
            last: None,
            previous: None,
        })
    }
    pub fn measure(&self, time: u64) -> Result<(f64, f64, f64), PrincipleError> {
        let Some(last) = self.last else {
            return Ok((0.0, 0.0, 0.0));
        };
        if time <= last {
            return Err(PrincipleError::BackwardTime);
        }
        let deviation = finite(((time - last) as f64 - self.tau) / self.tau)?;
        let previous = self.previous.unwrap_or(0.0);
        let recoil = finite((-previous * deviation / self.variance.max(1e-6)).tanh())?;
        Ok((previous, deviation, recoil))
    }
    pub fn commit(&mut self, time: u64, deviation: f64) {
        if self.last.is_some() {
            self.previous = Some(deviation);
        }
        self.last = Some(time);
    }
    pub fn valid(&self) -> bool {
        self.tau.is_finite()
            && self.tau > 0.0
            && self.variance.is_finite()
            && self.variance >= 0.0
            && self.previous.is_none_or(f64::is_finite)
    }
}

/// Tensor product of two normalized smooth sign states, four amplitudes.
pub fn prepare(previous: f64, current: f64) -> Result<[f64; N], PrincipleError> {
    let before = (1.0 + (4.0 * finite(previous)?).tanh()) / 2.0;
    let now = (1.0 + (4.0 * finite(current)?).tanh()) / 2.0;
    let vector = [
        (before * now).sqrt(),
        (before * (1.0 - now)).sqrt(),
        ((1.0 - before) * now).sqrt(),
        ((1.0 - before) * (1.0 - now)).sqrt(),
    ];
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
    pub fn free_energy(&self) -> f64 {
        -self.purity()
    }
    pub fn valid(&self) -> bool {
        if !self.trace().is_finite()
            || (self.trace() - 1.0).abs() > 1e-10
            || !(0.25 - 1e-10..=1.0 + 1e-10).contains(&self.purity())
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

/// Shannon entropy of four occupied sign-pair modes.
pub fn entropy(diagonal: [f64; N]) -> Result<f64, PrincipleError> {
    let total: f64 = diagonal.iter().sum();
    if total <= 0.0 {
        return Ok(0.0);
    }
    finite(
        diagonal
            .iter()
            .filter(|&&v| v > 0.0)
            .map(|&v| {
                let p = v / total;
                -p * p.ln()
            })
            .sum::<f64>()
            / (N as f64).ln(),
    )
}

/// Bounded four-species reaction and mass-action quorum.
#[derive(Clone, Debug, Default)]
pub struct Binding {
    pub sites: [f64; N],
    previous_size: Option<u32>,
}
impl Binding {
    pub fn observe(
        &mut self,
        occupation: &Occupation,
        bytes: u32,
        rate_factor: f64,
    ) -> Result<[f64; N], PrincipleError> {
        let similarity = self.previous_size.map_or(1.0, |prior| {
            let delta = (i64::from(bytes) - i64::from(prior)).unsigned_abs() as f64;
            (-delta / (f64::from(prior) + 1.0)).exp()
        });
        let mut drive = [0.0; N];
        for (i, value) in drive.iter_mut().enumerate() {
            *value = occupation.matrix[i][i]
                + 0.25
                    * (0..N)
                        .filter(|&j| j != i)
                        .map(|j| occupation.matrix[i][j].abs())
                        .sum::<f64>();
        }
        let total: f64 = drive.iter().sum();
        if !total.is_finite() || total <= 0.0 {
            return Err(PrincipleError::Capacity);
        }
        let rate = 0.1 * finite(rate_factor)?;
        if !(0.0..=1.0).contains(&rate) {
            return Err(PrincipleError::Capacity);
        }
        for i in 0..N {
            self.sites[i] =
                finite((1.0 - rate) * self.sites[i] + rate * similarity * drive[i] / total)?;
        }
        self.previous_size = Some(bytes);
        if self.valid() {
            Ok(self.sites)
        } else {
            Err(PrincipleError::Capacity)
        }
    }
    pub fn quorum(&self, information: f64) -> Result<f64, PrincipleError> {
        let information = finite(information)?;
        let product = self.sites.iter().product::<f64>();
        let value = finite(256.0 * product * information.powi(8))?;
        if value < 0.0 || value > 1.0 + 1e-10 {
            return Err(PrincipleError::Capacity);
        }
        Ok(value)
    }
    pub fn free_energy(&self) -> f64 {
        -self.sites.iter().product::<f64>()
    }
    pub fn valid(&self) -> bool {
        self.sites
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
            && self.sites.iter().sum::<f64>() <= 1.0 + 1e-10
    }
}

/// Adaptation to a dominant sign-pair mode.
#[derive(Clone, Debug, Default)]
pub struct Adaptation {
    pub value: f64,
}
impl Adaptation {
    pub fn observe(&mut self, diagonal: [f64; N]) -> Result<f64, PrincipleError> {
        let dominant = diagonal.iter().copied().fold(0.0, f64::max);
        self.value = finite(0.95 * self.value + 0.05 * dominant)?;
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

/// Signed spring recoil with viscous relaxation.
#[derive(Clone, Debug, Default)]
pub struct Spring {
    pub strain: f64,
}
impl Spring {
    pub fn observe(
        &mut self,
        recoil: f64,
        quorum: f64,
        adaptation: f64,
    ) -> Result<f64, PrincipleError> {
        self.strain = finite(
            0.97 * self.strain
                + 0.03 * finite(recoil)? * finite(quorum)? / (1.0 + finite(adaptation)?),
        )?;
        if self.valid() {
            Ok(self.strain)
        } else {
            Err(PrincipleError::Capacity)
        }
    }
    pub fn valid(&self) -> bool {
        self.strain.is_finite() && (-1.0..=1.0).contains(&self.strain)
    }
}

/// Dissipation of recoil work.
#[derive(Clone, Debug, Default)]
pub struct Heat {
    pub value: f64,
}
impl Heat {
    pub fn observe(&mut self, strain: f64) -> Result<f64, PrincipleError> {
        self.value = finite(0.9 * self.value + 0.1 * finite(strain)?.max(0.0).powi(2))?;
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
    fn pair_preparation_and_psd_remain_bounded() {
        let mut gaps = Gaps::new(100_000_000.0, 0.01).unwrap();
        let mut density = Occupation::default();
        for i in 0..100u64 {
            let time = i * 100_000_000;
            let (previous, current, _) = gaps.measure(time).unwrap();
            let vector = prepare(previous, current).unwrap();
            assert!((norm(vector) - 1.0).abs() < 1e-12);
            density.observe(vector, 0.02, Base::Quantum, true).unwrap();
            assert!(density.valid());
            gaps.commit(time, current);
        }
        assert_eq!(gaps.measure(1), Err(PrincipleError::BackwardTime));
    }
}
