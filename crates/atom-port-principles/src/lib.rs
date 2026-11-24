//! Intrinsic mathematical states over opaque observations. These are classical
//! computations inspired by first principles, not a network or quantum device.

use std::collections::VecDeque;
use std::f64::consts::TAU;

pub const SITES: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    NonFinite,
    InvalidInput,
}
impl std::fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid port-field state: {self:?}")
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
pub fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
pub fn site(port: u16) -> usize {
    usize::from(port) % SITES
}

/// Unitary preparation of a two-amplitude state; squared overlap is measured.
#[derive(Clone, Debug, Default)]
pub struct TwoAmplitude {
    previous: Option<[f64; 2]>,
}
impl TwoAmplitude {
    pub fn observe(&mut self, opaque: u64) -> Result<f64, PrincipleError> {
        let theta = TAU * (mix(opaque) as f64 / u64::MAX as f64);
        let state = [theta.cos(), theta.sin()];
        let overlap = self.previous.map_or(1.0, |last| {
            let dot = last[0] * state[0] + last[1] * state[1];
            dot * dot
        });
        self.previous = Some(state);
        finite(overlap.clamp(0.0, 1.0))
    }
    pub fn norm(&self) -> Option<f64> {
        self.previous.map(|s| s[0] * s[0] + s[1] * s[1])
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Complex {
    re: f64,
    im: f64,
}
impl Complex {
    fn scale(self, n: f64) -> Self {
        Self {
            re: self.re * n,
            im: self.im * n,
        }
    }
    fn add(self, b: Self) -> Self {
        Self {
            re: self.re + b.re,
            im: self.im + b.im,
        }
    }
    fn sub(self, b: Self) -> Self {
        Self {
            re: self.re - b.re,
            im: self.im - b.im,
        }
    }
    fn norm(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
    fn phase(self, angle: f64) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            re: self.re * c - self.im * s,
            im: self.re * s + self.im * c,
        }
    }
}

/// A phase kick followed by norm-preserving neighbor mixing on a ring.
#[derive(Clone, Debug)]
pub struct PortWave {
    amplitudes: [Complex; SITES],
}
impl Default for PortWave {
    fn default() -> Self {
        Self {
            amplitudes: [Complex {
                re: 1.0 / (SITES as f64).sqrt(),
                im: 0.0,
            }; SITES],
        }
    }
}
impl PortWave {
    pub fn observe(&mut self, port: u16, phase_shift: f64) -> Result<[f64; SITES], PrincipleError> {
        let site = site(port);
        let angle = TAU * (mix(u64::from(port)) as f64 / u64::MAX as f64) + finite(phase_shift)?;
        self.amplitudes[site] = self.amplitudes[site].phase(angle);
        let next = (site + 1) % SITES;
        let (s, c) = 0.12f64.sin_cos();
        let a = self.amplitudes[site];
        let b = self.amplitudes[next];
        self.amplitudes[site] = a.scale(c).sub(b.scale(s));
        self.amplitudes[next] = a.scale(s).add(b.scale(c));
        let density = self.densities();
        if density.iter().all(|n| n.is_finite() && *n >= 0.0) {
            Ok(density)
        } else {
            Err(PrincipleError::NonFinite)
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

/// Shannon entropy of the latest 256 site observations.
#[derive(Clone, Debug, Default)]
pub struct SiteInformation {
    bins: [u16; SITES],
    order: VecDeque<usize>,
}
impl SiteInformation {
    pub fn observe(&mut self, port: u16) -> Result<f64, PrincipleError> {
        let site = site(port);
        if self.order.len() == 256 {
            if let Some(old) = self.order.pop_front() {
                self.bins[old] -= 1;
            }
        }
        self.order.push_back(site);
        self.bins[site] += 1;
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
                / (SITES as f64).ln(),
        )
    }
    pub fn retained(&self) -> usize {
        self.order.len()
    }
    pub fn conserved(&self) -> bool {
        self.bins.iter().map(|&n| n as usize).sum::<usize>() == self.order.len()
    }
}

/// Recovery and habituation for repeated opaque stimuli at each site.
#[derive(Clone, Debug, Default)]
pub struct Adaptation {
    prior: [Option<u16>; SITES],
    fatigue: [f64; SITES],
}
impl Adaptation {
    pub fn observe(&mut self, port: u16, dt_ns: u64) -> Result<f64, PrincipleError> {
        let survival = (-(dt_ns as f64) / 1_000_000_000.0).exp();
        for value in &mut self.fatigue {
            *value *= survival;
        }
        let site = site(port);
        let response = if self.prior[site] == Some(port) {
            1.0 / (1.0 + self.fatigue[site])
        } else {
            1.0
        };
        self.fatigue[site] = (self.fatigue[site] + 1.0).min(32.0);
        self.prior[site] = Some(port);
        finite(response)
    }
    pub fn valid(&self) -> bool {
        self.fatigue
            .iter()
            .all(|n| n.is_finite() && (0.0..=32.0).contains(n))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reaction {
    Adsorb,
    Release,
}

/// Reversible occupation with adsorption limited by unoccupied capacity.
#[derive(Clone, Debug, Default)]
pub struct Chemistry {
    pub occupied: [f64; SITES],
}
impl Chemistry {
    pub fn observe(
        &mut self,
        port: u16,
        reaction: Reaction,
        overlap: f64,
        wave_density: f64,
        response: f64,
        rate_factor: f64,
        dt_ns: u64,
    ) -> Result<[f64; SITES], PrincipleError> {
        let survival = (-(dt_ns as f64) / 1_000_000_000.0).exp();
        for value in &mut self.occupied {
            *value *= survival;
        }
        let site = site(port);
        match reaction {
            Reaction::Adsorb => {
                let rate = 0.5
                    * finite(overlap)?.clamp(0.0, 1.0)
                    * finite(response)?.clamp(0.0, 1.0)
                    * (SITES as f64 * finite(wave_density)?).clamp(0.0, 1.0)
                    * finite(rate_factor)?.clamp(0.95, 1.05);
                self.occupied[site] += rate * (1.0 - self.occupied[site]);
            }
            Reaction::Release => {
                self.occupied[site] *= 0.1;
            }
        }
        if self.valid() {
            Ok(self.occupied)
        } else {
            Err(PrincipleError::NonFinite)
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

/// Elastic tension from adjacent occupied reaction sites.
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
    ) -> Result<[f64; SITES], PrincipleError> {
        let decay_v = (-(dt_ns as f64) / 300_000_000.0).exp();
        let decay_x = (-(dt_ns as f64) / 700_000_000.0).exp();
        let previous = self.displacement;
        for i in 0..SITES {
            let left = (i + SITES - 1) % SITES;
            let right = (i + 1) % SITES;
            let tension = occupied[i] * (occupied[left] + occupied[right]) * 0.5;
            let spring = 0.08 * (previous[left] + previous[right] - 2.0 * previous[i]);
            self.velocity[i] = finite(decay_v * self.velocity[i] + 0.5 * tension + spring)?;
            self.displacement[i] = finite(decay_x * previous[i] + 0.12 * self.velocity[i])?;
        }
        Ok(std::array::from_fn(|i| {
            self.velocity[i] * self.velocity[i] + self.displacement[i] * self.displacement[i]
        }))
    }
    pub fn valid(&self) -> bool {
        self.velocity
            .iter()
            .chain(self.displacement.iter())
            .all(|n| n.is_finite())
    }
}

/// Diffusive cooling of mechanical work across the ring.
#[derive(Clone, Debug, Default)]
pub struct Thermal {
    pub heat: [f64; SITES],
}
impl Thermal {
    pub fn observe(
        &mut self,
        work: [f64; SITES],
        entropy: f64,
        dt_ns: u64,
    ) -> Result<[f64; SITES], PrincipleError> {
        let previous = self.heat;
        let survival = (-(dt_ns as f64) / 500_000_000.0).exp();
        let diffusion = 0.03 + 0.05 * finite(entropy)?.clamp(0.0, 1.0);
        for i in 0..SITES {
            let left = previous[(i + SITES - 1) % SITES];
            let right = previous[(i + 1) % SITES];
            self.heat[i] = finite(
                survival * previous[i]
                    + diffusion * (left + right - 2.0 * previous[i])
                    + 0.05 * finite(work[i])?.max(0.0),
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
    fn intrinsic_states_preserve_bounds() {
        let mut q = TwoAmplitude::default();
        let mut wave = PortWave::default();
        let mut info = SiteInformation::default();
        let mut mind = Adaptation::default();
        let mut chem = Chemistry::default();
        let mut elastic = Elastic::default();
        let mut thermal = Thermal::default();
        for index in 0..1000u64 {
            let port = 4000 + (index % 127) as u16;
            let overlap = q.observe(index % 4).unwrap();
            let density = wave.observe(port, 0.0).unwrap();
            let entropy = info.observe(port).unwrap();
            let response = mind.observe(port, 10_000_000).unwrap();
            let occupied = chem
                .observe(
                    port,
                    Reaction::Adsorb,
                    overlap,
                    density[site(port)],
                    response,
                    1.0,
                    10_000_000,
                )
                .unwrap();
            let work = elastic.observe(occupied, 10_000_000).unwrap();
            thermal.observe(work, entropy, 10_000_000).unwrap();
            assert!((q.norm().unwrap() - 1.0).abs() < 1e-12);
            assert!((wave.norm() - 1.0).abs() < 1e-12);
            assert!(info.conserved() && info.retained() <= 256);
            assert!(mind.valid() && chem.valid() && elastic.valid() && thermal.valid());
        }
    }
}
