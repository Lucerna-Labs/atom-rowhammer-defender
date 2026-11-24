//! Exact opaque-ID site composition; no address geometry, label or action.
use std::collections::HashMap;
use std::fmt;

use atom_exact_row_site_principles::{
    Concentration, Logistic, Phase, PrincipleError, Reservoir, KICK, SITE_TAU_NS,
};

pub const MAX_EVENTS: usize = 4096;
pub const CAPACITY: f64 = 90.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub time_ns: u64,
    pub fingerprint: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    PhaseThenChemistry,
    ChemistryThenPhase,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chaos {
    Plain,
    LogisticKick,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mask {
    pub quantum: bool,
    pub chemistry: bool,
    pub physics: bool,
}
impl Mask {
    pub const FULL: Self = Self {
        quantum: true,
        chemistry: true,
        physics: true,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineError {
    TimeReversal,
    TooManyEvents,
    Principle(PrincipleError),
}
impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TimeReversal => f.write_str("event time moved backward"),
            Self::TooManyEvents => f.write_str("too many events or exact sites"),
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
struct Site {
    phase: Phase,
    concentration: Concentration,
    last_time_ns: u64,
}
impl Site {
    fn fresh(time_ns: u64) -> Self {
        Self {
            phase: Phase::default(),
            concentration: Concentration::default(),
            last_time_ns: time_ns,
        }
    }
    fn advance(
        mut self,
        time_ns: u64,
        order: Order,
        mask: Mask,
        factor: f64,
    ) -> Result<(Self, f64), EngineError> {
        self.concentration.decay(time_ns - self.last_time_ns)?;
        let previous_occupation = if mask.quantum {
            self.phase.occupation()
        } else {
            0.5
        };
        if mask.quantum {
            self.phase.rotate(KICK * factor)?;
        }
        let current_occupation = if mask.quantum {
            self.phase.occupation()
        } else {
            0.5
        };
        let weight = match order {
            Order::PhaseThenChemistry => current_occupation,
            Order::ChemistryThenPhase => previous_occupation,
        };
        if mask.chemistry {
            self.concentration.deposit(weight)?;
        }
        self.last_time_ns = time_ns;
        self.phase.validate()?;
        self.concentration.validate()?;
        let normalized_material = self.concentration.0 / (1.0 + self.concentration.0);
        let alignment = 1.0 - (self.phase.occupation() - normalized_material).abs();
        if !alignment.is_finite() {
            return Err(EngineError::Principle(PrincipleError::InvalidState));
        }
        Ok((self, alignment))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Observation {
    pub energy: f64,
    pub peak_energy: f64,
    pub first_crossing: Option<usize>,
    pub max_site_concentration: f64,
    pub max_phase_norm_error: f64,
    pub chosen_chaos_steps: usize,
    pub events: usize,
    pub sites: usize,
}

pub struct Engine {
    sites: HashMap<u64, Site>,
    logistic: Logistic,
    reservoir: Reservoir,
    order: Order,
    chaos: Chaos,
    mask: Mask,
    global_max: f64,
    last_time_ns: Option<u64>,
    last: Observation,
}
impl Engine {
    pub fn new(order: Order, chaos: Chaos, mask: Mask, seed: u64) -> Self {
        Self {
            sites: HashMap::new(),
            logistic: Logistic::new(seed),
            reservoir: Reservoir::default(),
            order,
            chaos,
            mask,
            global_max: 0.0,
            last_time_ns: None,
            last: Observation {
                energy: 0.0,
                peak_energy: 0.0,
                first_crossing: None,
                max_site_concentration: 0.0,
                max_phase_norm_error: 0.0,
                chosen_chaos_steps: 0,
                events: 0,
                sites: 0,
            },
        }
    }
    pub fn state(&self) -> Observation {
        self.last
    }
    pub fn observe(&mut self, event: Event) -> Result<Observation, EngineError> {
        if self.last.events >= MAX_EVENTS
            || (self.sites.len() >= MAX_EVENTS && !self.sites.contains_key(&event.fingerprint))
        {
            return Err(EngineError::TooManyEvents);
        }
        if self.last_time_ns.is_some_and(|last| event.time_ns < last) {
            return Err(EngineError::TimeReversal);
        }
        let dt_global = self.last_time_ns.map_or(0, |last| event.time_ns - last);
        let site = self
            .sites
            .get(&event.fingerprint)
            .copied()
            .unwrap_or_else(|| Site::fresh(event.time_ns));
        let (plain, plain_alignment) = site.advance(event.time_ns, self.order, self.mask, 1.0)?;
        let mut logistic = self.logistic;
        let mut chosen = plain;
        let mut selected = false;
        if self.chaos == Chaos::LogisticKick {
            let centered = logistic.next_centered()?;
            if let Ok((candidate, alignment)) =
                site.advance(event.time_ns, self.order, self.mask, 1.0 + 0.05 * centered)
            {
                if alignment > plain_alignment + 1e-12 {
                    chosen = candidate;
                    selected = true;
                }
            }
        }
        let decayed_max = self.global_max * (-(dt_global as f64) / SITE_TAU_NS).exp();
        let max_concentration = decayed_max.max(chosen.concentration.0);
        let mut reservoir = self.reservoir;
        let source = if self.mask.physics {
            max_concentration
        } else {
            0.0
        };
        let energy = reservoir.advance(source, dt_global)?;
        self.sites.insert(event.fingerprint, chosen);
        self.logistic = logistic;
        self.reservoir = reservoir;
        self.global_max = max_concentration;
        self.last_time_ns = Some(event.time_ns);
        self.last.events += 1;
        self.last.sites = self.sites.len();
        self.last.energy = energy;
        self.last.peak_energy = self.last.peak_energy.max(energy);
        self.last.max_site_concentration = max_concentration;
        self.last.max_phase_norm_error = self
            .last
            .max_phase_norm_error
            .max((chosen.phase.norm() - 1.0).abs());
        self.last.chosen_chaos_steps += usize::from(selected);
        if self.last.first_crossing.is_none() && energy >= CAPACITY {
            self.last.first_crossing = Some(self.last.events);
        }
        if self.last.max_phase_norm_error > 1e-10 || !self.last.peak_energy.is_finite() {
            return Err(EngineError::Principle(PrincipleError::InvalidState));
        }
        Ok(self.last)
    }
}

pub fn evaluate(
    events: &[Event],
    order: Order,
    chaos: Chaos,
    mask: Mask,
    seed: u64,
) -> Result<Observation, EngineError> {
    if events.len() > MAX_EVENTS {
        return Err(EngineError::TooManyEvents);
    }
    let mut engine = Engine::new(order, chaos, mask, seed);
    for &event in events {
        engine.observe(event)?;
    }
    Ok(engine.state())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_repeat_and_time_reversal() -> Result<(), EngineError> {
        let events = (0..4096)
            .map(|index| Event {
                time_ns: index * 4_000,
                fingerprint: 123,
            })
            .collect::<Vec<_>>();
        let a = evaluate(
            &events,
            Order::PhaseThenChemistry,
            Chaos::Plain,
            Mask::FULL,
            1,
        )?;
        let b = evaluate(
            &events,
            Order::PhaseThenChemistry,
            Chaos::Plain,
            Mask::FULL,
            1,
        )?;
        assert_eq!(a, b);
        assert!(a.max_phase_norm_error <= 1e-10);
        let mut engine = Engine::new(Order::PhaseThenChemistry, Chaos::Plain, Mask::FULL, 1);
        engine.observe(events[1])?;
        let before = engine.state();
        assert_eq!(engine.observe(events[0]), Err(EngineError::TimeReversal));
        assert_eq!(engine.state(), before);
        Ok(())
    }
}
