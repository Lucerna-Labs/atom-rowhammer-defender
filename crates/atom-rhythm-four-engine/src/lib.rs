//! Fixed four-mode composition over one externally scoped timing stream.
//! It returns numeric state and has no traffic-purpose labels or policy.
use atom_rhythm_four_principles::{
    norm, prepare, Adaptation, Base, Binding, Heat, Henon, Logistic, Occupation, PhaseClock,
    PhaseInformation, PrincipleError, Strain, N,
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
    pub purity: f64,
    pub trace: f64,
    pub occupation: [f64; N],
    pub harmonics: [f64; 2],
    pub entropy: f64,
    pub binding: f64,
    pub adaptation: f64,
    pub strain: [f64; 2],
    pub heat: [f64; 2],
    pub phase: f64,
    pub phase_norm: f64,
}
#[derive(Clone)]
pub struct Engine {
    clock: PhaseClock,
    occupation: Occupation,
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
    pub fn new(
        order: Order,
        chaos: Chaos,
        mask: Mask,
        seed: u64,
        tau_ns: f64,
    ) -> Result<Self, PrincipleError> {
        let route = match order {
            Order::TransportThenSource => Self::transport_then_source,
            Order::SourceThenTransport => Self::source_then_transport,
        };
        Ok(Self {
            clock: PhaseClock::new(tau_ns)?,
            occupation: Occupation::default(),
            information: PhaseInformation::default(),
            binding: Binding::default(),
            adaptation: Adaptation::default(),
            strain: Strain::default(),
            heat: Heat::default(),
            logistic: Logistic::new(seed),
            henon: Henon::new(seed ^ 0xF04D_5CA1),
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
        self.clock.valid()
            && self.occupation.valid()
            && self.information.conserved()
            && self.information.retained() <= 64
            && self.binding.valid()
            && self.adaptation.valid()
            && self.strain.valid()
            && self.heat.valid()
    }
    fn quantum(
        &mut self,
        phase: f64,
        logistic: f64,
        henon: f64,
        transport_first: bool,
    ) -> Result<(), PrincipleError> {
        let base = if self.mask.quantum {
            Base::Quantum
        } else if self.mask.diffusion {
            Base::Diffusion
        } else {
            Base::Population
        };
        let vector = prepare(phase)?;
        let mut plain = self.occupation.clone();
        plain.observe(vector, 0.02, base, transport_first)?;
        if base != Base::Quantum {
            self.occupation = plain;
            return Ok(());
        }
        match self.chaos.phase() {
            Some(source) => {
                let dither = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.occupation.clone();
                candidate.observe(
                    prepare(phase + 0.05 * std::f64::consts::PI * dither)?,
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
        Ok(())
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
        let phase = self.clock.phase_for(event.time_ns)?;
        let logistic = self.logistic.step();
        let henon = self.henon.step();
        self.quantum(phase, logistic, henon, transport_first)?;
        let purity = self.occupation.purity();
        let diagonal = self.occupation.diagonal();
        let harmonics = self.occupation.harmonic_energies();
        let entropy = if self.mask.information {
            self.information.observe(phase)?
        } else {
            0.0
        };
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
        let strain = if self.mask.mechanics {
            self.strain.observe(harmonics, binding, adaptation)?
        } else {
            [0.0; 2]
        };
        let heat = if self.mask.physics {
            self.heat.observe(strain)?
        } else {
            [0.0; 2]
        };
        self.clock.commit(event.time_ns);
        let energy = strain[0] * strain[0] + heat[0];
        if !energy.is_finite() {
            return Err(PrincipleError::NonFinite);
        }
        Ok(Observation {
            energy,
            rupture: energy >= 0.04,
            purity,
            trace: self.occupation.trace(),
            occupation: diagonal,
            harmonics,
            entropy,
            binding,
            adaptation,
            strain,
            heat,
            phase,
            phase_norm: norm(prepare(phase)?),
        })
    }
}

pub struct RhythmFourOrchestrator;
impl RhythmFourOrchestrator {
    pub fn observe(engine: &mut Engine, input: Event) -> Result<Observation, PrincipleError> {
        engine.observe(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_scale_and_bad_time_fail_closed() {
        assert!(Engine::new(Order::TransportThenSource, Chaos::Plain, Mask::FULL, 0, 0.0).is_err());
        let mut e = Engine::new(
            Order::TransportThenSource,
            Chaos::Plain,
            Mask::FULL,
            0,
            100_000_000.0,
        )
        .unwrap();
        for i in 0..64 {
            let output = RhythmFourOrchestrator::observe(
                &mut e,
                Event {
                    time_ns: i * 100_000_000,
                    bytes_out: 64,
                },
            )
            .unwrap();
            assert!((output.phase_norm - 1.0).abs() < 1e-12 && e.invariants_hold());
        }
        assert_eq!(
            RhythmFourOrchestrator::observe(
                &mut e,
                Event {
                    time_ns: 1,
                    bytes_out: 64
                }
            ),
            Err(PrincipleError::BackwardTime)
        );
    }
}
