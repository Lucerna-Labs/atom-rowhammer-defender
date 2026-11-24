//! Fixed phase/chemistry/mechanics composition over one scoped event stream.
//! The engine reports numerical state and never names a traffic purpose.
use atom_rhythm_principles::{
    Adaptation, Binding, Circular, Density, Heat, Henon, Logistic, Period, PhaseInformation,
    PrincipleError, Strain,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub time_ns: u64,
    pub bytes_out: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    BindBeforeAdapt,
    AdaptBeforeBind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chaos {
    Plain,
    LogisticAtPhase,
    HenonAtBinding,
    Both,
    LogisticAtBinding,
    HenonAtPhase,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Logistic,
    Henon,
}
impl Chaos {
    fn phase(self) -> Option<Source> {
        match self {
            Self::LogisticAtPhase | Self::Both => Some(Source::Logistic),
            Self::HenonAtPhase => Some(Source::Henon),
            _ => None,
        }
    }
    fn binding(self) -> Option<Source> {
        match self {
            Self::HenonAtBinding | Self::Both => Some(Source::Henon),
            Self::LogisticAtBinding => Some(Source::Logistic),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mask {
    pub quantum: bool,
    pub circular: bool,
    pub information: bool,
    pub chemistry: bool,
    pub cognition: bool,
    pub mechanics: bool,
    pub physics: bool,
}
impl Mask {
    pub const FULL: Self = Self {
        quantum: true,
        circular: false,
        information: true,
        chemistry: true,
        cognition: true,
        mechanics: true,
        physics: true,
    };
    pub const BASE_ONLY: Self = Self {
        quantum: true,
        circular: false,
        information: false,
        chemistry: false,
        cognition: false,
        mechanics: false,
        physics: false,
    };
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Observation {
    pub energy: f64,
    pub rupture: bool,
    pub purity: f64,
    pub entropy: f64,
    pub binding: f64,
    pub adaptation: f64,
    pub strain: f64,
    pub heat: f64,
    pub tau: Option<f64>,
    pub phase: f64,
    pub phase_norm: f64,
}
#[derive(Clone)]
pub struct Engine {
    period: Period,
    density: Density,
    circular: Circular,
    information: PhaseInformation,
    binding: Binding,
    adaptation: Adaptation,
    strain: Strain,
    heat: Heat,
    logistic: Logistic,
    henon: Henon,
    mask: Mask,
    chaos: Chaos,
    route: fn(&mut Engine, Event) -> Result<Observation, PrincipleError>,
}
impl Engine {
    pub fn new(order: Order, chaos: Chaos, mask: Mask, seed: u64) -> Self {
        let route = match order {
            Order::BindBeforeAdapt => Self::bind_before_adapt,
            Order::AdaptBeforeBind => Self::adapt_before_bind,
        };
        Self {
            period: Period::default(),
            density: Density::default(),
            circular: Circular::default(),
            information: PhaseInformation::default(),
            binding: Binding::default(),
            adaptation: Adaptation::default(),
            strain: Strain::default(),
            heat: Heat::default(),
            logistic: Logistic::new(seed),
            henon: Henon::new(seed ^ 0xBEAC_0A55),
            mask,
            chaos,
            route,
        }
    }
    pub fn observe(&mut self, input: Event) -> Result<Observation, PrincipleError> {
        let mut candidate = self.clone();
        let output = (candidate.route)(&mut candidate, input)?;
        if !candidate.invariants_hold() {
            return Err(PrincipleError::Capacity);
        }
        *self = candidate;
        Ok(output)
    }
    pub fn invariants_hold(&self) -> bool {
        self.period.valid()
            && self.density.valid()
            && self.circular.valid()
            && self.information.conserved()
            && self.information.retained() <= 64
            && self.binding.valid()
            && self.adaptation.valid()
            && self.strain.valid()
            && self.heat.valid()
    }
    fn quantum(&mut self, phase: f64, logistic: f64, henon: f64) -> Result<f64, PrincipleError> {
        if self.mask.circular {
            return self.circular.observe(phase);
        }
        if !self.mask.quantum {
            return Ok(0.5);
        }
        let mut plain = self.density.clone();
        let plain_value = plain.observe(phase)?;
        match self.chaos.phase() {
            Some(source) => {
                let dither = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.density.clone();
                let candidate_value =
                    candidate.observe(phase + 0.05 * std::f64::consts::PI * dither)?;
                if candidate.free_energy() < plain.free_energy() {
                    self.density = candidate;
                    Ok(candidate_value)
                } else {
                    self.density = plain;
                    Ok(plain_value)
                }
            }
            None => {
                self.density = plain;
                Ok(plain_value)
            }
        }
    }
    fn bind(
        &mut self,
        bytes: u32,
        entropy: f64,
        adaptation: f64,
        logistic: f64,
        henon: f64,
    ) -> Result<f64, PrincipleError> {
        if !self.mask.chemistry {
            return Ok(0.0);
        }
        let mut plain = self.binding.clone();
        let plain_value = plain.observe(bytes, entropy, adaptation, 1.0)?;
        match self.chaos.binding() {
            Some(source) => {
                let dither = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.binding.clone();
                let candidate_value =
                    candidate.observe(bytes, entropy, adaptation, 1.0 + 0.05 * dither)?;
                if candidate.free_energy() < plain.free_energy() {
                    self.binding = candidate;
                    Ok(candidate_value)
                } else {
                    self.binding = plain;
                    Ok(plain_value)
                }
            }
            None => {
                self.binding = plain;
                Ok(plain_value)
            }
        }
    }
    fn bind_before_adapt(&mut self, event: Event) -> Result<Observation, PrincipleError> {
        self.compose(event, false)
    }
    fn adapt_before_bind(&mut self, event: Event) -> Result<Observation, PrincipleError> {
        self.compose(event, true)
    }
    fn compose(
        &mut self,
        event: Event,
        cognition_first: bool,
    ) -> Result<Observation, PrincipleError> {
        let (phase, gap) = self.period.prepare(event.time_ns)?;
        let logistic = self.logistic.step();
        let henon = self.henon.step();
        let purity = self.quantum(phase, logistic, henon)?;
        let entropy = if self.mask.information {
            self.information.observe(phase)?
        } else {
            0.0
        };
        let (binding, adaptation) = if cognition_first {
            let adaptation = if self.mask.cognition {
                self.adaptation.observe(purity)?
            } else {
                0.0
            };
            let binding = self.bind(event.bytes_out, entropy, adaptation, logistic, henon)?;
            (binding, adaptation)
        } else {
            let prior = if self.mask.cognition {
                self.adaptation.value
            } else {
                0.0
            };
            let binding = self.bind(event.bytes_out, entropy, prior, logistic, henon)?;
            let adaptation = if self.mask.cognition {
                self.adaptation.observe(purity)?
            } else {
                0.0
            };
            (binding, adaptation)
        };
        let strain = if self.mask.mechanics {
            self.strain.observe(purity, binding, adaptation)?
        } else {
            0.0
        };
        let heat = if self.mask.physics {
            self.heat.observe(strain)?
        } else {
            0.0
        };
        self.period.commit(event.time_ns, gap)?;
        let energy = strain * strain + heat;
        if !energy.is_finite() {
            return Err(PrincipleError::NonFinite);
        }
        let phase_norm = phase.cos().powi(2) + phase.sin().powi(2);
        Ok(Observation {
            energy,
            rupture: energy >= 0.03,
            purity,
            entropy,
            binding,
            adaptation,
            strain,
            heat,
            tau: self.period.tau,
            phase,
            phase_norm,
        })
    }
}

pub struct RhythmOrchestrator;
impl RhythmOrchestrator {
    pub fn observe(engine: &mut Engine, input: Event) -> Result<Observation, PrincipleError> {
        engine.observe(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn time_order_and_clean_state_are_bounded() {
        let mut engine = Engine::new(Order::BindBeforeAdapt, Chaos::Plain, Mask::FULL, 0);
        for index in 0..64 {
            let out = RhythmOrchestrator::observe(
                &mut engine,
                Event {
                    time_ns: index * 100_000_000,
                    bytes_out: 64,
                },
            )
            .unwrap();
            assert!((out.phase_norm - 1.0).abs() < 1e-12 && engine.invariants_hold());
        }
        assert_eq!(
            RhythmOrchestrator::observe(
                &mut engine,
                Event {
                    time_ns: 100,
                    bytes_out: 64
                }
            ),
            Err(PrincipleError::BackwardTime)
        );
    }
}
