//! Intrinsic eight-site dynamics over opaque events. Mathematical amplitudes
//! run on a classical CPU and make no physical quantum or security claim.

use std::collections::VecDeque;
use std::f64::consts::TAU;

pub const SITES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldError {
    NonFinite,
    BackwardTime,
}
impl std::fmt::Display for FieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid field state: {self:?}")
    }
}
impl std::error::Error for FieldError {}

pub fn finite(value: f64) -> Result<f64, FieldError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(FieldError::NonFinite)
    }
}

pub fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

pub fn site(opaque: u64) -> usize {
    (mix(opaque) as usize) % SITES
}

#[derive(Clone, Copy, Debug, Default)]
struct Complex {
    re: f64,
    im: f64,
}
impl Complex {
    fn scale(self, value: f64) -> Self {
        Self {
            re: self.re * value,
            im: self.im * value,
        }
    }
    fn add(self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
    fn minus(self, other: Self) -> Self {
        Self {
            re: self.re - other.re,
            im: self.im - other.im,
        }
    }
    fn norm(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
    fn phase(self, angle: f64) -> Self {
        let (sine, cosine) = angle.sin_cos();
        Self {
            re: self.re * cosine - self.im * sine,
            im: self.re * sine + self.im * cosine,
        }
    }
}

/// Unitary phase kick and Givens rotation on a complex amplitude ring.
#[derive(Clone, Debug)]
pub struct Wave {
    amplitudes: [Complex; SITES],
}
impl Default for Wave {
    fn default() -> Self {
        Self {
            amplitudes: [Complex {
                re: 1.0 / (SITES as f64).sqrt(),
                im: 0.0,
            }; SITES],
        }
    }
}
impl Wave {
    pub fn observe(
        &mut self,
        origin: u64,
        principal: u64,
        phase_shift: f64,
    ) -> Result<[f64; SITES], FieldError> {
        let site = site(principal);
        let phase = TAU * (mix(origin ^ principal.rotate_left(17)) as f64 / u64::MAX as f64)
            + finite(phase_shift)?;
        self.amplitudes[site] = self.amplitudes[site].phase(phase);
        let next = (site + 1) % SITES;
        let cosine = 0.15f64.cos();
        let sine = 0.15f64.sin();
        let a = self.amplitudes[site];
        let b = self.amplitudes[next];
        self.amplitudes[site] = a.scale(cosine).minus(b.scale(sine));
        self.amplitudes[next] = a.scale(sine).add(b.scale(cosine));
        let densities = self.densities();
        if densities.iter().all(|n| n.is_finite() && *n >= 0.0) {
            Ok(densities)
        } else {
            Err(FieldError::NonFinite)
        }
    }
    pub fn densities(&self) -> [f64; SITES] {
        self.amplitudes.map(Complex::norm)
    }
    pub fn norm(&self) -> f64 {
        self.densities().iter().sum()
    }
    pub fn entropy(&self) -> f64 {
        self.densities()
            .iter()
            .filter(|&&p| p > 0.0)
            .map(|p| -p * p.ln())
            .sum::<f64>()
            / (SITES as f64).ln()
    }
}

/// Shannon information over a bounded rolling distribution of opaque sources.
#[derive(Clone, Debug, Default)]
pub struct OriginInformation {
    bins: [u16; 32],
    order: VecDeque<usize>,
}
impl OriginInformation {
    pub fn observe(&mut self, opaque: u64) -> Result<f64, FieldError> {
        let bin = (mix(opaque) as usize) % self.bins.len();
        if self.order.len() == 256 {
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
                / (32.0f64).ln(),
        )
    }
    pub fn retained(&self) -> usize {
        self.order.len()
    }
    pub fn conserved(&self) -> bool {
        self.bins.iter().map(|&n| n as usize).sum::<usize>() == self.order.len()
    }
}

/// Bounded reaction sites, with adsorption limited by unoccupied capacity.
#[derive(Clone, Debug, Default)]
pub struct Adsorption {
    pub occupancy: [f64; SITES],
}
impl Adsorption {
    pub fn observe(
        &mut self,
        site: usize,
        density: f64,
        load: f64,
        reactivity: f64,
        dt_ns: u64,
    ) -> Result<[f64; SITES], FieldError> {
        let survival = (-(dt_ns as f64) / 2_000_000_000.0).exp();
        for occupied in &mut self.occupancy {
            *occupied *= survival;
        }
        let rate = 0.4
            * finite(load)?.clamp(0.0, 1.0)
            * finite(reactivity)?.clamp(0.0, 1.0)
            * (SITES as f64 * finite(density)?).clamp(0.0, 1.0);
        self.occupancy[site] += rate * (1.0 - self.occupancy[site]);
        if self.valid() {
            Ok(self.occupancy)
        } else {
            Err(FieldError::NonFinite)
        }
    }
    pub fn valid(&self) -> bool {
        self.occupancy
            .iter()
            .all(|n| n.is_finite() && (0.0..=1.0).contains(n))
    }
}

/// Per-site habituation and time recovery for repeated opaque stimuli.
#[derive(Clone, Debug, Default)]
pub struct Adaptation {
    previous: [Option<u64>; SITES],
    fatigue: [f64; SITES],
}
impl Adaptation {
    pub fn observe(&mut self, opaque: u64, dt_ns: u64) -> Result<f64, FieldError> {
        let survival = (-(dt_ns as f64) / 1_000_000_000.0).exp();
        for n in &mut self.fatigue {
            *n *= survival;
        }
        let site = site(opaque);
        let response = if self.previous[site] == Some(opaque) {
            1.0 / (1.0 + self.fatigue[site])
        } else {
            1.0
        };
        self.fatigue[site] = (self.fatigue[site] + 1.0).min(32.0);
        self.previous[site] = Some(opaque);
        finite(response)
    }
    pub fn valid(&self) -> bool {
        self.fatigue
            .iter()
            .all(|n| n.is_finite() && (0.0..=32.0).contains(n))
    }
}

/// Elastic ring responding to spatial gradients of occupied sites.
#[derive(Clone, Debug, Default)]
pub struct Elastic {
    pub velocity: [f64; SITES],
    pub displacement: [f64; SITES],
}
impl Elastic {
    pub fn observe(
        &mut self,
        occupied: [f64; SITES],
        dt_ns: u64,
        spring: f64,
    ) -> Result<[f64; SITES], FieldError> {
        let spring = finite(spring)?;
        let v_survival = (-(dt_ns as f64) / 500_000_000.0).exp();
        let x_survival = (-(dt_ns as f64) / 1_000_000_000.0).exp();
        let mean = occupied.iter().sum::<f64>() / SITES as f64;
        let previous = self.displacement;
        for index in 0..SITES {
            let left = previous[(index + SITES - 1) % SITES];
            let right = previous[(index + 1) % SITES];
            let gradient = occupied[index] - mean;
            let spring_force = spring * (left + right - 2.0 * previous[index]);
            self.velocity[index] =
                finite(v_survival * self.velocity[index] + 0.35 * gradient + spring_force)?;
            self.displacement[index] =
                finite(x_survival * previous[index] + 0.2 * self.velocity[index])?;
        }
        Ok(std::array::from_fn(|index| {
            self.velocity[index] * self.velocity[index]
                + self.displacement[index] * self.displacement[index]
        }))
    }
    pub fn valid(&self) -> bool {
        self.velocity
            .iter()
            .chain(self.displacement.iter())
            .all(|n| n.is_finite())
    }
}

/// Dissipative and diffusive thermal reservoirs over the same ring.
#[derive(Clone, Debug, Default)]
pub struct Thermal {
    pub heat: [f64; SITES],
}
impl Thermal {
    pub fn observe(
        &mut self,
        kinetic: [f64; SITES],
        dt_ns: u64,
        entropy: f64,
    ) -> Result<[f64; SITES], FieldError> {
        let previous = self.heat;
        let survival = (-(dt_ns as f64) / 700_000_000.0).exp();
        let diffusion = 0.04 + 0.04 * finite(entropy)?.clamp(0.0, 1.0);
        for index in 0..SITES {
            let left = previous[(index + SITES - 1) % SITES];
            let right = previous[(index + 1) % SITES];
            self.heat[index] = finite(
                previous[index] * survival
                    + diffusion * (left + right - 2.0 * previous[index])
                    + 0.05 * finite(kinetic[index])?.max(0.0),
            )?
            .max(0.0);
        }
        Ok(self.heat)
    }
    pub fn valid(&self) -> bool {
        self.heat.iter().all(|n| n.is_finite() && *n >= 0.0)
    }
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
    fn field_invariants_hold_after_many_updates() {
        let mut wave = Wave::default();
        let mut info = OriginInformation::default();
        let mut chem = Adsorption::default();
        let mut mind = Adaptation::default();
        let mut elastic = Elastic::default();
        let mut thermal = Thermal::default();
        for index in 0..1000u64 {
            let wave_density = wave.observe(index, index / 5, 0.0).unwrap();
            let entropy = info.observe(index).unwrap();
            let response = mind.observe(index, 100_000_000).unwrap();
            let site = site(index / 5);
            let occupied = chem
                .observe(site, wave_density[site], 1.0, response, 100_000_000)
                .unwrap();
            let kinetic = elastic.observe(occupied, 100_000_000, 0.15).unwrap();
            thermal.observe(kinetic, 100_000_000, entropy).unwrap();
            assert!((wave.norm() - 1.0).abs() < 1e-12);
            assert!(info.conserved() && info.retained() <= 256);
            assert!(chem.valid() && mind.valid() && elastic.valid() && thermal.valid());
        }
    }
}
