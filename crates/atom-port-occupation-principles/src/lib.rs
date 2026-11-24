//! Intrinsic order-insensitive field mechanics over opaque observations.
//! The density operator is computed on a classical CPU; it is not a QPU state.

use std::collections::VecDeque;
use std::f64::consts::{PI, TAU};

pub const SITES: usize = 16;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    NonFinite,
    InvalidInput,
}
impl std::fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid occupation field: {self:?}")
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

/// Unitary two-amplitude preparation, observed by Born squared overlap.
#[derive(Clone, Debug, Default)]
pub struct DestinationWave {
    previous: Option<[f64; 2]>,
}
impl DestinationWave {
    pub fn observe(&mut self, opaque: u64) -> Result<f64, PrincipleError> {
        let angle = TAU * (mix(opaque) as f64 / u64::MAX as f64);
        let state = [angle.cos(), angle.sin()];
        let overlap = self.previous.map_or(1.0, |before| {
            let inner = before[0] * state[0] + before[1] * state[1];
            inner * inner
        });
        self.previous = Some(state);
        finite(overlap.clamp(0.0, 1.0))
    }
    pub fn norm(&self) -> Option<f64> {
        self.previous
            .map(|state| state[0] * state[0] + state[1] * state[1])
    }
}

/// A real symmetric PSD density operator with unit trace. Each observation
/// mixes in a rank-one projector on the current site and its right neighbor.
#[derive(Clone, Debug)]
pub struct Density {
    rho: [[f64; SITES]; SITES],
}
impl Default for Density {
    fn default() -> Self {
        let mut rho = [[0.0; SITES]; SITES];
        for (i, row) in rho.iter_mut().enumerate() {
            row[i] = 1.0 / SITES as f64;
        }
        Self { rho }
    }
}
impl Density {
    pub fn observe(&mut self, port: u16, angle_shift: f64) -> Result<(), PrincipleError> {
        let angle = PI / 6.0 + finite(angle_shift)?;
        let left = site(port);
        let right = (left + 1) % SITES;
        for row in &mut self.rho {
            for value in row {
                *value *= 0.92;
            }
        }
        let a = angle.cos();
        let b = angle.sin();
        self.rho[left][left] += 0.08 * a * a;
        self.rho[right][right] += 0.08 * b * b;
        let off = 0.08 * a * b;
        self.rho[left][right] += off;
        self.rho[right][left] += off;
        if self.trace().is_finite() {
            Ok(())
        } else {
            Err(PrincipleError::NonFinite)
        }
    }
    pub fn correlation(&self, i: usize, j: usize) -> f64 {
        self.rho[i][j].abs()
    }
    pub fn trace(&self) -> f64 {
        (0..SITES).map(|i| self.rho[i][i]).sum()
    }
    pub fn free_energy(&self) -> f64 {
        -(0..SITES)
            .flat_map(|i| (i + 1..SITES).map(move |j| self.rho[i][j].abs()))
            .sum::<f64>()
    }
    pub fn valid(&self) -> bool {
        if (self.trace() - 1.0).abs() > 1e-10 {
            return false;
        }
        for i in 0..SITES {
            for j in 0..SITES {
                if !self.rho[i][j].is_finite() || (self.rho[i][j] - self.rho[j][i]).abs() > 1e-10 {
                    return false;
                }
            }
        }
        // A Cholesky-style PSD diagnostic tolerates semidefinite pivots.
        let mut lower = [[0.0; SITES]; SITES];
        for i in 0..SITES {
            for j in 0..=i {
                let mut residual = self.rho[i][j];
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

/// Shannon information over the latest 256 port-site observations.
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

/// Intrinsic repeated-stimulus adaptation with time recovery.
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
/// Reversible occupancy bounded by finite reaction sites.
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
                let rate = 0.7
                    * finite(overlap)?.clamp(0.0, 1.0)
                    * finite(response)?.clamp(0.0, 1.0)
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

/// Globally coupled membrane: pair potential exerts viscoelastic stress.
#[derive(Clone, Debug, Default)]
pub struct Membrane {
    pub stress: f64,
    pub strain: f64,
}
impl Membrane {
    pub fn observe(
        &mut self,
        density: &Density,
        occupied: [f64; SITES],
        dt_ns: u64,
    ) -> Result<f64, PrincipleError> {
        let mut potential = 0.0;
        for i in 0..SITES {
            for j in i + 1..SITES {
                potential += SITES as f64 * density.correlation(i, j) * occupied[i] * occupied[j];
            }
        }
        self.stress = finite(
            self.stress * (-(dt_ns as f64) / 250_000_000.0).exp() + 0.1 * finite(potential)?,
        )?;
        self.strain =
            finite(self.strain * (-(dt_ns as f64) / 500_000_000.0).exp() + 0.1 * self.stress)?;
        finite(self.stress * self.stress + self.strain * self.strain)
    }
    pub fn valid(&self) -> bool {
        self.stress.is_finite() && self.strain.is_finite()
    }
}

/// Dissipative thermal reservoir receiving viscoelastic work.
#[derive(Clone, Debug, Default)]
pub struct Heat {
    pub value: f64,
}
impl Heat {
    pub fn observe(&mut self, work: f64, entropy: f64, dt_ns: u64) -> Result<f64, PrincipleError> {
        let cooling = (-(dt_ns as f64)
            / (500_000_000.0 + 300_000_000.0 * finite(entropy)?.clamp(0.0, 1.0)))
        .exp();
        self.value = finite(self.value * cooling + 0.05 * finite(work)?.max(0.0))?;
        Ok(self.value)
    }
    pub fn valid(&self) -> bool {
        self.value.is_finite() && self.value >= 0.0
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
    fn psd_and_bounded_states_survive_many_observations() {
        let mut q = DestinationWave::default();
        let mut density = Density::default();
        let mut info = SiteInformation::default();
        let mut mind = Adaptation::default();
        let mut chemistry = Chemistry::default();
        let mut membrane = Membrane::default();
        let mut heat = Heat::default();
        for index in 0..1000u64 {
            let port = 4000 + (index % 23) as u16;
            let overlap = q.observe(index % 7).unwrap();
            density.observe(port, 0.0).unwrap();
            let entropy = info.observe(port).unwrap();
            let response = mind.observe(port, 10_000_000).unwrap();
            let occupied = chemistry
                .observe(port, Reaction::Adsorb, overlap, response, 1.0, 10_000_000)
                .unwrap();
            let work = membrane.observe(&density, occupied, 10_000_000).unwrap();
            heat.observe(work, entropy, 10_000_000).unwrap();
            assert!(density.valid());
            assert!((q.norm().unwrap() - 1.0).abs() < 1e-12);
            assert!(info.conserved() && info.retained() <= 256);
            assert!(mind.valid() && chemistry.valid() && membrane.valid() && heat.valid());
        }
    }
}
