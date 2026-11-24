//! Fixed process-lineage material composition, without threat labels or action.
use std::collections::HashMap;
use std::fmt;

use atom_process_lineage_principles::{Bond, Heat, Logistic, Phase, PrincipleError, Site};

pub const MAX_BIRTHS: usize = 64;
pub const CAPACITY: f64 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Birth {
    pub time_ns: u64,
    pub pid: u64,
    pub ppid: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    PhaseThenSite,
    SiteThenPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chaos {
    Plain,
    LogisticPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mask {
    pub quantum: bool,
    pub biology: bool,
    pub chemistry: bool,
    pub physics: bool,
}
impl Mask {
    pub const FULL: Self = Self {
        quantum: true,
        biology: true,
        chemistry: true,
        physics: true,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineError {
    InvalidBirth,
    TimeReversal,
    TooManyBirths,
    Principle(PrincipleError),
}
impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBirth => f.write_str("invalid or repeated process identity"),
            Self::TimeReversal => f.write_str("birth time moved backward"),
            Self::TooManyBirths => f.write_str("too many process births"),
            Self::Principle(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for EngineError {}
impl From<PrincipleError> for EngineError {
    fn from(value: PrincipleError) -> Self {
        Self::Principle(value)
    }
}

#[derive(Clone, Copy, Debug)]
struct Node {
    phase: Phase,
    site: Site,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Observation {
    pub energy: f64,
    pub peak_energy: f64,
    pub first_crossing: Option<usize>,
    pub phase_norm_error: f64,
    pub chemical_bond: f64,
    pub chosen_chaos_steps: usize,
    pub births: usize,
}

#[derive(Clone)]
pub struct Engine {
    nodes: HashMap<u64, Node>,
    bond: Bond,
    heat: Heat,
    logistic: Logistic,
    order: Order,
    chaos: Chaos,
    mask: Mask,
    last_time_ns: Option<u64>,
    last: Observation,
}

impl Engine {
    pub fn new(order: Order, chaos: Chaos, mask: Mask, seed: u64) -> Self {
        Self {
            nodes: HashMap::new(),
            bond: Bond::default(),
            heat: Heat::default(),
            logistic: Logistic::new(seed),
            order,
            chaos,
            mask,
            last_time_ns: None,
            last: Observation {
                energy: 0.0,
                peak_energy: 0.0,
                first_crossing: None,
                phase_norm_error: 0.0,
                chemical_bond: 0.0,
                chosen_chaos_steps: 0,
                births: 0,
            },
        }
    }

    pub fn state(&self) -> Observation {
        self.last
    }

    pub fn observe(&mut self, birth: Birth) -> Result<Observation, EngineError> {
        if birth.pid == 0 || birth.pid == birth.ppid || self.nodes.contains_key(&birth.pid) {
            return Err(EngineError::InvalidBirth);
        }
        if self.nodes.len() >= MAX_BIRTHS {
            return Err(EngineError::TooManyBirths);
        }
        if self.last_time_ns.is_some_and(|last| birth.time_ns < last) {
            return Err(EngineError::TimeReversal);
        }
        let mut base = self.clone();
        let centered = if self.chaos == Chaos::LogisticPhase {
            base.logistic.next_centered()?
        } else {
            0.0
        };
        let mut plain = base.clone();
        let plain_alignment = plain.advance(birth, 1.0)?;
        let mut chosen = plain;
        if self.chaos == Chaos::LogisticPhase {
            let mut variant = base;
            let angle_factor = 1.0 + 0.05 * centered;
            if let Ok(variant_alignment) = variant.advance(birth, angle_factor) {
                if variant_alignment > plain_alignment + 1e-12 {
                    variant.last.chosen_chaos_steps += 1;
                    chosen = variant;
                }
            }
        }
        *self = chosen;
        Ok(self.last)
    }

    fn advance(&mut self, birth: Birth, angle_factor: f64) -> Result<f64, EngineError> {
        let dt = self.last_time_ns.map_or(0, |last| birth.time_ns - last);
        let (child, overlap, alignment) =
            if let Some(mut parent) = self.nodes.get(&birth.ppid).copied() {
                let prior_occ = if self.mask.quantum {
                    parent.phase.occupation()
                } else {
                    0.5
                };
                if self.mask.quantum {
                    let angle = (0.35 + 0.10 * parent.site.material) * angle_factor;
                    parent.phase.rotate(angle)?;
                }
                let current_occ = if self.mask.quantum {
                    parent.phase.occupation()
                } else {
                    0.5
                };
                let site_occ = match self.order {
                    Order::PhaseThenSite => current_occ,
                    Order::SiteThenPhase => prior_occ,
                };
                let (site, raw_overlap) = if self.mask.biology {
                    parent.site.offspring(site_occ)?
                } else {
                    (Site::seed(), 0.0)
                };
                let alignment = parent.phase.alignment(parent.site.material)?;
                let child = Node {
                    phase: parent.phase,
                    site,
                };
                self.nodes.insert(birth.ppid, parent);
                (child, raw_overlap, alignment)
            } else {
                (
                    Node {
                        phase: Phase::default(),
                        site: Site::seed(),
                    },
                    0.0,
                    0.0,
                )
            };
        child.phase.validate()?;
        child.site.validate()?;
        self.nodes.insert(birth.pid, child);
        let work = self
            .bond
            .advance(if self.mask.chemistry { overlap } else { 0.0 }, dt)?;
        let energy = self
            .heat
            .advance(if self.mask.physics { work } else { 0.0 }, dt)?;
        self.last.births += 1;
        self.last_time_ns = Some(birth.time_ns);
        self.last.energy = energy;
        self.last.peak_energy = self.last.peak_energy.max(energy);
        if self.last.first_crossing.is_none() && energy >= CAPACITY {
            self.last.first_crossing = Some(self.last.births);
        }
        self.last.chemical_bond = self.bond.0;
        self.last.phase_norm_error = self
            .nodes
            .values()
            .map(|node| (node.phase.norm() - 1.0).abs())
            .fold(0.0, f64::max);
        if !self.last.peak_energy.is_finite() || self.last.phase_norm_error > 1e-10 {
            return Err(EngineError::Principle(PrincipleError::InvalidState));
        }
        Ok(alignment)
    }
}

pub fn evaluate(
    stream: &[Birth],
    order: Order,
    chaos: Chaos,
    mask: Mask,
    seed: u64,
) -> Result<Observation, EngineError> {
    if stream.len() > MAX_BIRTHS {
        return Err(EngineError::TooManyBirths);
    }
    let mut engine = Engine::new(order, chaos, mask, seed);
    for &birth in stream {
        engine.observe(birth)?;
    }
    Ok(engine.state())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_repeat_and_invalid_input() -> Result<(), EngineError> {
        let stream = (2..=32)
            .map(|pid| Birth {
                time_ns: (pid - 2) * 100_000,
                pid,
                ppid: pid / 2,
            })
            .collect::<Vec<_>>();
        let a = evaluate(
            &stream,
            Order::PhaseThenSite,
            Chaos::LogisticPhase,
            Mask::FULL,
            7,
        )?;
        let b = evaluate(
            &stream,
            Order::PhaseThenSite,
            Chaos::LogisticPhase,
            Mask::FULL,
            7,
        )?;
        assert_eq!(a, b);
        assert!(a.phase_norm_error <= 1e-10);
        let mut engine = Engine::new(Order::PhaseThenSite, Chaos::Plain, Mask::FULL, 0);
        engine.observe(stream[0])?;
        let before = engine.state();
        assert_eq!(engine.observe(stream[0]), Err(EngineError::InvalidBirth));
        assert_eq!(engine.state(), before);
        Ok(())
    }
}
