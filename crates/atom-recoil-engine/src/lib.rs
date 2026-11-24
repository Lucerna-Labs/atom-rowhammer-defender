//! Fixed observation-only stack over sign-pair density, chemical quorum and
//! mechanical recoil. The runtime has no endpoint or attack vocabulary.
use atom_recoil_principles::{
    entropy, norm, prepare, Adaptation, Base, Binding, Gaps, Heat, Henon, Logistic, Occupation,
    PrincipleError, Spring, N,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub time_ns: u64,
    pub bytes_out: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    TransportThenSource,
    SourceThenTransport,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chaos {
    Plain,
    LogisticAtSource,
    HenonAtChemistry,
    Both,
    LogisticAtChemistry,
    HenonAtSource,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Logistic,
    Henon,
}
impl Chaos {
    fn source(self) -> Option<Source> {
        match self {
            Self::LogisticAtSource | Self::Both => Some(Source::Logistic),
            Self::HenonAtSource => Some(Source::Henon),
            _ => None,
        }
    }
    fn chemistry(self) -> Option<Source> {
        match self {
            Self::HenonAtChemistry | Self::Both => Some(Source::Henon),
            Self::LogisticAtChemistry => Some(Source::Logistic),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mask {
    pub quantum: bool,
    pub diffusion: bool,
    pub information: bool,
    pub chemistry: bool,
    pub cognition: bool,
    pub mechanics: bool,
    pub physics: bool,
}
impl Mask {
    pub const FULL: Self = Self {
        quantum: true,
        diffusion: false,
        information: true,
        chemistry: true,
        cognition: true,
        mechanics: true,
        physics: true,
    };
    pub const BASE_ONLY: Self = Self {
        quantum: true,
        diffusion: false,
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
    pub trace: f64,
    pub purity: f64,
    pub entropy: f64,
    pub quorum: f64,
    pub recoil: f64,
    pub adaptation: f64,
    pub strain: f64,
    pub heat: f64,
    pub source_norm: f64,
    pub modes: [f64; N],
    pub binding: [f64; N],
}
#[derive(Clone)]
pub struct Engine {
    gaps: Gaps,
    occupation: Occupation,
    binding: Binding,
    adaptation: Adaptation,
    spring: Spring,
    heat: Heat,
    logistic: Logistic,
    henon: Henon,
    mask: Mask,
    chaos: Chaos,
    route: fn(&mut Engine, Event) -> Result<Observation, PrincipleError>,
}
impl Engine {
    pub fn new(
        order: Order,
        chaos: Chaos,
        mask: Mask,
        seed: u64,
        tau: f64,
        variance: f64,
    ) -> Result<Self, PrincipleError> {
        let route = match order {
            Order::TransportThenSource => Self::transport_then_source,
            Order::SourceThenTransport => Self::source_then_transport,
        };
        Ok(Self {
            gaps: Gaps::new(tau, variance)?,
            occupation: Occupation::default(),
            binding: Binding::default(),
            adaptation: Adaptation::default(),
            spring: Spring::default(),
            heat: Heat::default(),
            logistic: Logistic::new(seed),
            henon: Henon::new(seed ^ 0x0BE7_1C01),
            mask,
            chaos,
            route,
        })
    }
    pub fn observe(&mut self, event: Event) -> Result<Observation, PrincipleError> {
        let mut candidate = self.clone();
        let output = (candidate.route)(&mut candidate, event)?;
        if !candidate.invariants_hold() {
            return Err(PrincipleError::Capacity);
        }
        *self = candidate;
        Ok(output)
    }
    pub fn invariants_hold(&self) -> bool {
        self.gaps.valid()
            && self.occupation.valid()
            && self.binding.valid()
            && self.adaptation.valid()
            && self.spring.valid()
            && self.heat.valid()
    }
    fn quantum(
        &mut self,
        previous: f64,
        current: f64,
        logistic: f64,
        henon: f64,
        transport_first: bool,
    ) -> Result<f64, PrincipleError> {
        let base = if self.mask.quantum {
            Base::Quantum
        } else if self.mask.diffusion {
            Base::Diffusion
        } else {
            Base::Population
        };
        let vector = prepare(previous, current)?;
        let source_norm = norm(vector);
        let mut plain = self.occupation.clone();
        plain.observe(vector, 0.02, base, transport_first)?;
        if base != Base::Quantum {
            self.occupation = plain;
            return Ok(source_norm);
        }
        match self.chaos.source() {
            Some(source) => {
                let dither = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.occupation.clone();
                candidate.observe(
                    prepare(previous, current * (1.0 + 0.05 * dither))?,
                    0.02,
                    base,
                    transport_first,
                )?;
                if candidate.free_energy() < plain.free_energy() {
                    self.occupation = candidate;
                } else {
                    self.occupation = plain;
                }
            }
            None => {
                self.occupation = plain;
            }
        }
        Ok(source_norm)
    }
    fn bind(&mut self, bytes: u32, logistic: f64, henon: f64) -> Result<[f64; N], PrincipleError> {
        if !self.mask.chemistry {
            return Ok([0.0; N]);
        }
        let mut plain = self.binding.clone();
        let plain_values = plain.observe(&self.occupation, bytes, 1.0)?;
        match self.chaos.chemistry() {
            Some(source) => {
                let dither = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.binding.clone();
                let candidate_values =
                    candidate.observe(&self.occupation, bytes, 1.0 + 0.05 * dither)?;
                if candidate.free_energy() < plain.free_energy() {
                    self.binding = candidate;
                    Ok(candidate_values)
                } else {
                    self.binding = plain;
                    Ok(plain_values)
                }
            }
            None => {
                self.binding = plain;
                Ok(plain_values)
            }
        }
    }
    fn transport_then_source(&mut self, event: Event) -> Result<Observation, PrincipleError> {
        self.compose(event, true)
    }
    fn source_then_transport(&mut self, event: Event) -> Result<Observation, PrincipleError> {
        self.compose(event, false)
    }
    fn compose(
        &mut self,
        event: Event,
        transport_first: bool,
    ) -> Result<Observation, PrincipleError> {
        let (previous, current, recoil) = self.gaps.measure(event.time_ns)?;
        let logistic = self.logistic.step();
        let henon = self.henon.step();
        let source_norm = self.quantum(previous, current, logistic, henon, transport_first)?;
        let modes = self.occupation.diagonal();
        let information = if self.mask.information {
            entropy(modes)?
        } else {
            0.0
        };
        let binding = self.bind(event.bytes_out, logistic, henon)?;
        let quorum = if self.mask.chemistry {
            self.binding.quorum(information)?
        } else {
            0.0
        };
        let adaptation = if self.mask.cognition {
            self.adaptation.observe(modes)?
        } else {
            0.0
        };
        let strain = if self.mask.mechanics {
            self.spring.observe(recoil, quorum, adaptation)?
        } else {
            0.0
        };
        let heat = if self.mask.physics {
            self.heat.observe(strain)?
        } else {
            0.0
        };
        self.gaps.commit(event.time_ns, current);
        let energy = strain.max(0.0).powi(2) + heat;
        if !energy.is_finite() {
            return Err(PrincipleError::NonFinite);
        }
        Ok(Observation {
            energy,
            rupture: energy >= 0.00005,
            trace: self.occupation.trace(),
            purity: self.occupation.purity(),
            entropy: information,
            quorum,
            recoil,
            adaptation,
            strain,
            heat,
            source_norm,
            modes,
            binding,
        })
    }
}

pub struct RecoilOrchestrator;
impl RecoilOrchestrator {
    pub fn observe(engine: &mut Engine, input: Event) -> Result<Observation, PrincipleError> {
        engine.observe(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_gaps_have_zero_recoil_and_bad_time_is_rejected() {
        let mut engine = Engine::new(
            Order::TransportThenSource,
            Chaos::Plain,
            Mask::FULL,
            0,
            100_000_000.0,
            0.01,
        )
        .unwrap();
        for i in 0..64 {
            let output = RecoilOrchestrator::observe(
                &mut engine,
                Event {
                    time_ns: i * 100_000_000,
                    bytes_out: 64,
                },
            )
            .unwrap();
            assert_eq!(output.recoil, 0.0);
            assert!((output.source_norm - 1.0).abs() < 1e-12);
        }
        assert_eq!(
            RecoilOrchestrator::observe(
                &mut engine,
                Event {
                    time_ns: 1,
                    bytes_out: 64
                }
            ),
            Err(PrincipleError::BackwardTime)
        );
    }
}
