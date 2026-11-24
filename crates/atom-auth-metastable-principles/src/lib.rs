//! Intrinsic persistent states over opaque observations. No attack label,
//! response policy, credential, or physical quantum claim enters this crate.

use std::collections::VecDeque;
use std::f64::consts::TAU;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    NonFinite,
    BackwardTime,
}
impl std::fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid metastable state: {self:?}")
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

/// Normalized unitary preparation and squared-overlap observation.
#[derive(Clone, Debug, Default)]
pub struct Wave {
    previous: Option<[f64; 2]>,
}
impl Wave {
    pub fn observe(&mut self, opaque: u64, phase_shift: f64) -> Result<f64, PrincipleError> {
        let theta = TAU * (mix(opaque) as f64 / u64::MAX as f64) + finite(phase_shift)?;
        let prepared = [theta.cos(), theta.sin()];
        let fidelity = self.previous.map_or(1.0, |last| {
            let inner = last[0] * prepared[0] + last[1] * prepared[1];
            inner * inner
        });
        self.previous = Some(prepared);
        finite(fidelity.clamp(0.0, 1.0))
    }
    pub fn norm(&self) -> Option<f64> {
        self.previous.map(|s| s[0] * s[0] + s[1] * s[1])
    }
}

/// Bounded empirical origin distribution with Shannon entropy.
#[derive(Clone, Debug, Default)]
pub struct OriginEntropy {
    bins: [u16; 32],
    order: VecDeque<usize>,
}
impl OriginEntropy {
    pub fn observe(&mut self, origin: u64) -> Result<f64, PrincipleError> {
        let bin = (mix(origin) as usize) % self.bins.len();
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

#[derive(Clone, Copy, Debug)]
struct HabitSite {
    opaque: u64,
    fatigue: f64,
    time_ns: u64,
}

/// Stimulus adaptation with per-identity recovery and bounded site eviction.
#[derive(Clone, Debug, Default)]
pub struct Habit {
    sites: VecDeque<HabitSite>,
}
impl Habit {
    pub fn observe(&mut self, opaque: u64, time_ns: u64) -> Result<f64, PrincipleError> {
        let index = self.sites.iter().position(|site| site.opaque == opaque);
        if index
            .and_then(|position| self.sites.get(position))
            .is_some_and(|site| time_ns < site.time_ns)
        {
            return Err(PrincipleError::BackwardTime);
        }
        let response = match index {
            Some(index) => {
                let mut site = self.sites.remove(index).ok_or(PrincipleError::NonFinite)?;
                if time_ns < site.time_ns {
                    return Err(PrincipleError::BackwardTime);
                }
                site.fatigue *= (-((time_ns - site.time_ns) as f64) / 2_000_000_000.0).exp();
                let response = 1.0 / (1.0 + site.fatigue);
                site.fatigue = (site.fatigue + 1.0).min(32.0);
                site.time_ns = time_ns;
                self.sites.push_back(site);
                response
            }
            None => {
                if self.sites.len() == 64 {
                    self.sites.pop_front();
                }
                self.sites.push_back(HabitSite {
                    opaque,
                    fatigue: 1.0,
                    time_ns,
                });
                1.0
            }
        };
        finite(response)
    }
    pub fn retained(&self) -> usize {
        self.sites.len()
    }
    pub fn valid(&self) -> bool {
        self.sites
            .iter()
            .all(|site| site.fatigue.is_finite() && (0.0..=32.0).contains(&site.fatigue))
    }
}

#[derive(Clone, Copy, Debug)]
struct BondSite {
    opaque: u64,
    occupied: f64,
    crosslink: f64,
    time_ns: u64,
}

/// Reversible adsorption plus autocatalytic crosslink, bounded by free sites.
#[derive(Clone, Debug, Default)]
pub struct Bonds {
    sites: VecDeque<BondSite>,
}
impl Bonds {
    pub fn observe(
        &mut self,
        opaque: u64,
        time_ns: u64,
        load: f64,
        overlap: f64,
        entropy: f64,
        response: f64,
        rate_factor: f64,
    ) -> Result<f64, PrincipleError> {
        let load = finite(load)?.clamp(0.0, 1.0);
        let overlap = finite(overlap)?.clamp(0.0, 1.0);
        let entropy = finite(entropy)?.clamp(0.0, 1.0);
        let response = finite(response)?.clamp(0.0, 1.0);
        let rate_factor = finite(rate_factor)?.clamp(0.95, 1.05);
        let index = self.sites.iter().position(|site| site.opaque == opaque);
        if index
            .and_then(|position| self.sites.get(position))
            .is_some_and(|site| time_ns < site.time_ns)
        {
            return Err(PrincipleError::BackwardTime);
        }
        let mut site = match index {
            Some(index) => self.sites.remove(index).ok_or(PrincipleError::NonFinite)?,
            None => BondSite {
                opaque,
                occupied: 0.0,
                crosslink: 0.0,
                time_ns,
            },
        };
        if time_ns < site.time_ns {
            return Err(PrincipleError::BackwardTime);
        }
        let dt = time_ns - site.time_ns;
        site.occupied *= (-(dt as f64) / 2_000_000_000.0).exp();
        site.crosslink *= (-(dt as f64) / 8_000_000_000.0).exp();
        let adsorption = 0.2 * load * overlap * (0.5 + 0.5 * entropy) * response * rate_factor;
        site.occupied += adsorption * (1.0 - site.occupied);
        site.crosslink += 0.3 * site.occupied * site.occupied * (1.0 - site.crosslink);
        site.time_ns = time_ns;
        let flux = finite(site.occupied * site.crosslink)?;
        if self.sites.len() == 64 {
            self.sites.pop_front();
        }
        self.sites.push_back(site);
        if self.valid() {
            Ok(flux)
        } else {
            Err(PrincipleError::NonFinite)
        }
    }
    pub fn retained(&self) -> usize {
        self.sites.len()
    }
    pub fn valid(&self) -> bool {
        self.sites.iter().all(|site| {
            site.occupied.is_finite()
                && site.crosslink.is_finite()
                && (0.0..=1.0).contains(&site.occupied)
                && (0.0..=1.0).contains(&site.crosslink)
        })
    }
    pub fn free_energy(&self) -> f64 {
        self.sites
            .iter()
            .map(|site| -(site.occupied + site.crosslink))
            .sum()
    }
}

/// Viscous/elastic strain, retaining a different state than thermal heat.
#[derive(Clone, Debug, Default)]
pub struct Maxwell {
    pub stress: f64,
    pub strain: f64,
}
impl Maxwell {
    pub fn observe(&mut self, flux: f64, dt_ns: u64) -> Result<f64, PrincipleError> {
        self.stress =
            finite(self.stress * (-(dt_ns as f64) / 1_000_000_000.0).exp() + 0.8 * finite(flux)?)?;
        self.strain =
            finite(self.strain * (-(dt_ns as f64) / 3_000_000_000.0).exp() + 0.25 * self.stress)?;
        finite(self.stress * self.stress + self.strain * self.strain)
    }
}

/// Dissipative thermal work storage.
#[derive(Clone, Debug, Default)]
pub struct Heat {
    pub value: f64,
}
impl Heat {
    pub fn observe(&mut self, work: f64, dt_ns: u64) -> Result<f64, PrincipleError> {
        self.value = finite(
            self.value * (-(dt_ns as f64) / 2_000_000_000.0).exp() + 0.08 * finite(work)?.max(0.0),
        )?;
        Ok(self.value)
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
    fn intrinsic_bounds_hold() {
        let mut wave = Wave::default();
        let mut info = OriginEntropy::default();
        let mut habit = Habit::default();
        let mut bonds = Bonds::default();
        let mut maxwell = Maxwell::default();
        let mut heat = Heat::default();
        for index in 0..1000u64 {
            let time = index * 500_000_000;
            let q = wave.observe(index % 7, 0.0).unwrap();
            let s = info.observe(index).unwrap();
            let h = habit.observe(index % 19, time).unwrap();
            let flux = bonds.observe(index % 5, time, 1.0, q, s, h, 1.0).unwrap();
            let work = maxwell.observe(flux, 500_000_000).unwrap();
            heat.observe(work, 500_000_000).unwrap();
            assert!((wave.norm().unwrap() - 1.0).abs() < 1e-12);
            assert!(info.conserved() && info.retained() <= 256);
            assert!(habit.valid() && habit.retained() <= 64);
            assert!(bonds.valid() && bonds.retained() <= 64);
            assert!(maxwell.stress.is_finite() && maxwell.strain.is_finite());
            assert!(heat.value.is_finite() && heat.value >= 0.0);
        }
    }

    #[test]
    fn backward_time_preserves_retained_sites() {
        let mut bonds = Bonds::default();
        let mut habit = Habit::default();
        bonds.observe(7, 100, 1.0, 1.0, 0.5, 1.0, 1.0).unwrap();
        habit.observe(9, 100).unwrap();
        let before = bonds.free_energy();
        assert_eq!(
            bonds.observe(7, 99, 1.0, 1.0, 0.5, 1.0, 1.0),
            Err(PrincipleError::BackwardTime)
        );
        assert_eq!(habit.observe(9, 99), Err(PrincipleError::BackwardTime));
        assert_eq!(bonds.retained(), 1);
        assert_eq!(habit.retained(), 1);
        assert_eq!(bonds.free_energy(), before);
    }
}
