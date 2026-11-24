//! Intrinsic mathematics for ordered opaque auth observations.
//! No source or principal identifier has a security meaning in this crate.

use std::collections::VecDeque;
use std::f64::consts::TAU;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrincipleError {
    NonFinite,
    NonMonotoneTime,
}

impl std::fmt::Display for PrincipleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid principle state: {self:?}")
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

fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

/// A unitary real subspace of a two-amplitude complex state. Norm is conserved
/// by the rotation; squared inner product is the Born-style overlap.
#[derive(Clone, Debug, Default)]
pub struct Amplitude {
    previous: Option<[f64; 2]>,
}
impl Amplitude {
    pub fn observe(&mut self, opaque: u64, phase_delta: f64) -> Result<f64, PrincipleError> {
        let phase = TAU * (mix(opaque) as f64 / u64::MAX as f64) + finite(phase_delta)?;
        let prepared = [phase.cos(), phase.sin()];
        let overlap = self.previous.map_or(1.0, |before| {
            let inner = before[0] * prepared[0] + before[1] * prepared[1];
            inner * inner
        });
        self.previous = Some(prepared);
        finite(overlap.clamp(0.0, 1.0))
    }
    pub fn norm(&self) -> Option<f64> {
        self.previous
            .map(|state| state[0] * state[0] + state[1] * state[1])
    }
}

/// Bounded empirical distribution, with Shannon entropy normalized to 32 bins.
#[derive(Clone, Debug, Default)]
pub struct Occupation {
    bins: [u16; 32],
    order: VecDeque<usize>,
}
impl Occupation {
    pub fn observe(&mut self, opaque: u64) -> Result<f64, PrincipleError> {
        let bin = (mix(opaque) as usize) % self.bins.len();
        if self.order.len() == 256 {
            if let Some(old) = self.order.pop_front() {
                self.bins[old] -= 1;
            }
        }
        self.order.push_back(bin);
        self.bins[bin] += 1;
        let total = self.order.len() as f64;
        let entropy = self
            .bins
            .iter()
            .filter(|&&n| n > 0)
            .map(|&n| {
                let p = n as f64 / total;
                -p * p.ln()
            })
            .sum::<f64>()
            / (32.0f64).ln();
        finite(entropy)
    }
    pub fn count(&self) -> usize {
        self.order.len()
    }
    pub fn bin_sum(&self) -> usize {
        self.bins.iter().map(|&n| n as usize).sum()
    }
}

#[derive(Clone, Copy, Debug)]
struct Site {
    opaque: u64,
    occupancy: f64,
}

/// Reversible adsorption and desorption over a bounded number of sites.
#[derive(Clone, Debug, Default)]
pub struct Binding {
    sites: VecDeque<Site>,
    last_time: Option<u64>,
}
impl Binding {
    pub fn observe(
        &mut self,
        opaque: u64,
        time_ns: u64,
        reactivity: f64,
    ) -> Result<f64, PrincipleError> {
        let dt = self
            .last_time
            .map_or(0, |last| time_ns.saturating_sub(last));
        if self.last_time.is_some_and(|last| time_ns < last) {
            return Err(PrincipleError::NonMonotoneTime);
        }
        self.last_time = Some(time_ns);
        let survival = (-(dt as f64) / 2_000_000_000.0).exp();
        for site in &mut self.sites {
            site.occupancy *= survival;
        }
        let rate = finite(reactivity)?.clamp(0.0, 1.0);
        let index = self.sites.iter().position(|site| site.opaque == opaque);
        let flux = match index {
            Some(index) => {
                let mut site = self.sites.remove(index).ok_or(PrincipleError::NonFinite)?;
                let flux = (1.0 - site.occupancy) * rate;
                site.occupancy += flux;
                self.sites.push_back(site);
                flux
            }
            None => {
                if self.sites.len() == 64 {
                    self.sites.pop_front();
                }
                self.sites.push_back(Site {
                    opaque,
                    occupancy: rate,
                });
                rate
            }
        };
        finite(flux)
    }
    pub fn site_count(&self) -> usize {
        self.sites.len()
    }
    pub fn valid(&self) -> bool {
        self.sites
            .iter()
            .all(|site| site.occupancy.is_finite() && (0.0..=1.0).contains(&site.occupancy))
    }
}

/// Response fatigue on repeated stimuli, with recovery over elapsed time.
#[derive(Clone, Debug, Default)]
pub struct Habituation {
    previous: Option<u64>,
    adaptation: f64,
    last_time: Option<u64>,
}
impl Habituation {
    pub fn observe(&mut self, opaque: u64, time_ns: u64) -> Result<f64, PrincipleError> {
        if self.last_time.is_some_and(|last| time_ns < last) {
            return Err(PrincipleError::NonMonotoneTime);
        }
        let dt = self.last_time.map_or(0, |last| time_ns - last);
        self.adaptation *= (-(dt as f64) / 1_000_000_000.0).exp();
        let response = if self.previous == Some(opaque) {
            1.0 / (1.0 + self.adaptation)
        } else {
            1.0
        };
        self.adaptation = (self.adaptation + 1.0).min(32.0);
        self.previous = Some(opaque);
        self.last_time = Some(time_ns);
        finite(response)
    }
    pub fn valid(&self) -> bool {
        self.adaptation.is_finite() && (0.0..=32.0).contains(&self.adaptation)
    }
}

/// Damped momentum coupled to a spring displacement.
#[derive(Clone, Debug, Default)]
pub struct Momentum {
    pub velocity: f64,
    pub displacement: f64,
}
impl Momentum {
    pub fn observe(&mut self, impulse: f64, dt_ns: u64) -> Result<f64, PrincipleError> {
        self.velocity =
            finite(self.velocity * (-(dt_ns as f64) / 500_000_000.0).exp() + finite(impulse)?)?;
        self.displacement = finite(
            self.displacement * (-(dt_ns as f64) / 1_000_000_000.0).exp() + 0.08 * self.velocity,
        )?;
        finite(self.velocity * self.velocity + self.displacement * self.displacement)
    }
}

/// A dissipative reservoir receiving mechanical work.
#[derive(Clone, Debug, Default)]
pub struct Reservoir {
    pub heat: f64,
}
impl Reservoir {
    pub fn observe(&mut self, work: f64, dt_ns: u64) -> Result<f64, PrincipleError> {
        self.heat = finite(
            self.heat * (-(dt_ns as f64) / 700_000_000.0).exp() + 0.04 * finite(work)?.max(0.0),
        )?;
        Ok(self.heat)
    }
}

/// Deterministic logistic recurrence, centered before applying a small phase shift.
#[derive(Clone, Debug)]
pub struct Logistic {
    x: f64,
}
impl Logistic {
    pub fn new(seed: u64) -> Self {
        let x = 0.1 + 0.8 * (mix(seed) as f64 / u64::MAX as f64);
        Self { x }
    }
    pub fn step(&mut self) -> f64 {
        self.x = 4.0 * self.x * (1.0 - self.x);
        2.0 * self.x - 1.0
    }
}

/// Folded Hénon recurrence with the measured folded center kept explicit.
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
    fn intrinsic_bounds_and_conservation() {
        let mut q = Amplitude::default();
        let mut info = Occupation::default();
        let mut bind = Binding::default();
        let mut mind = Habituation::default();
        let mut mechanical = Momentum::default();
        let mut heat = Reservoir::default();
        for index in 0..1000 {
            let time = index * 100_000_000;
            let subject = (index % 71) as u64;
            q.observe(index as u64 % 3, 0.0).unwrap();
            let entropy = info.observe(subject).unwrap();
            let flux = bind.observe(subject, time, 1.0).unwrap();
            let response = mind.observe(subject, time).unwrap();
            let work = mechanical
                .observe(entropy + flux + response, 100_000_000)
                .unwrap();
            heat.observe(work, 100_000_000).unwrap();
            assert!((q.norm().unwrap() - 1.0).abs() < 1e-12);
            assert_eq!(info.count(), info.bin_sum());
            assert!(info.count() <= 256 && bind.site_count() <= 64);
            assert!(bind.valid() && mind.valid() && heat.heat.is_finite());
        }
    }
}
