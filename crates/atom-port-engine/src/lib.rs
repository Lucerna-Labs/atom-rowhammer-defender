//! Fixed numeric composition for ordered network observations. It cannot
//! inspect attack labels, scan the host, or change network policy.

use std::collections::HashMap;
use std::path::Path;

use atom_port_principles::{
    finite, site, Adaptation, Chemistry, Elastic, Henon, Logistic, PortWave, PrincipleError,
    Reaction, SiteInformation, Thermal, TwoAmplitude, SITES,
};

pub const MAX_TRACE: usize = 1_000_000;
pub const MAX_PENDING: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Attempt,
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub time_ns: u64,
    pub origin: u64,
    pub destination: u64,
    pub port: u16,
    pub phase: Phase,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortError {
    InvalidPort,
    BackwardTime,
    UnmatchedCompletion,
    PendingCapacity,
    InvalidTrace(String),
    Principle(PrincipleError),
}
impl From<PrincipleError> for PortError {
    fn from(value: PrincipleError) -> Self {
        Self::Principle(value)
    }
}
impl std::fmt::Display for PortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid network observation: {self:?}")
    }
}
impl std::error::Error for PortError {}

pub fn parse_trace(text: &str) -> Result<Vec<Event>, PortError> {
    let mut events = Vec::new();
    let mut last = None;
    let mut pending: HashMap<(u64, u64, u16), u32> = HashMap::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        if fields.len() != 5 {
            return Err(PortError::InvalidTrace(format!(
                "line {}: expected five fields",
                index + 1
            )));
        }
        let number = |field: &str| -> Result<u64, PortError> {
            if let Some(hex) = field.strip_prefix("0x") {
                u64::from_str_radix(hex, 16)
            } else {
                field.parse::<u64>()
            }
            .map_err(|_| PortError::InvalidTrace(format!("line {}: invalid integer", index + 1)))
        };
        let time_ns = number(fields[0])?;
        let origin = number(fields[1])?;
        let destination = number(fields[2])?;
        let raw_port = number(fields[3])?;
        let port = u16::try_from(raw_port).map_err(|_| PortError::InvalidPort)?;
        if port == 0 {
            return Err(PortError::InvalidPort);
        }
        if last.is_some_and(|before| time_ns < before) {
            return Err(PortError::BackwardTime);
        }
        let phase = match fields[4] {
            "attempt" => Phase::Attempt,
            "complete" => Phase::Complete,
            _ => {
                return Err(PortError::InvalidTrace(format!(
                    "line {}: invalid phase",
                    index + 1
                )))
            }
        };
        let key = (origin, destination, port);
        match phase {
            Phase::Attempt => {
                if pending.len() >= MAX_PENDING && !pending.contains_key(&key) {
                    return Err(PortError::PendingCapacity);
                }
                let count = pending.entry(key).or_default();
                *count = count.checked_add(1).ok_or(PortError::PendingCapacity)?;
            }
            Phase::Complete => {
                let count = pending
                    .get_mut(&key)
                    .ok_or(PortError::UnmatchedCompletion)?;
                *count -= 1;
                if *count == 0 {
                    pending.remove(&key);
                }
            }
        }
        events.push(Event {
            time_ns,
            origin,
            destination,
            port,
            phase,
        });
        if events.len() > MAX_TRACE {
            return Err(PortError::PendingCapacity);
        }
        last = Some(time_ns);
    }
    if events.is_empty() {
        return Err(PortError::InvalidTrace("empty trace".into()));
    }
    Ok(events)
}
pub fn read_trace(path: &Path) -> Result<Vec<Event>, PortError> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| PortError::InvalidTrace(format!("{}: {error}", path.display())))?;
    parse_trace(&text)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    ReactBeforeHabituate,
    HabituateBeforeReact,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chaos {
    Plain,
    LogisticAtPhase,
    HenonAtReaction,
    Both,
    LogisticAtReaction,
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
    fn reaction(self) -> Option<Source> {
        match self {
            Self::HenonAtReaction | Self::Both => Some(Source::Henon),
            Self::LogisticAtReaction => Some(Source::Logistic),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mask {
    pub quantum: bool,
    pub information: bool,
    pub chemistry: bool,
    pub cognition: bool,
    pub mechanics: bool,
    pub physics: bool,
}
impl Mask {
    pub const FULL: Self = Self {
        quantum: true,
        information: true,
        chemistry: true,
        cognition: true,
        mechanics: true,
        physics: true,
    };
    pub const BASE_ONLY: Self = Self {
        quantum: true,
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
    pub peak_energy: f64,
    pub rupture: bool,
    pub destination_overlap: f64,
    pub port_entropy: f64,
    pub destination_norm: Option<f64>,
    pub port_norm: Option<f64>,
    pub retained_observations: usize,
    pub pending: usize,
}

pub struct Engine {
    destination: TwoAmplitude,
    wave: PortWave,
    information: SiteInformation,
    chemistry: Chemistry,
    mind: Adaptation,
    elastic: Elastic,
    thermal: Thermal,
    logistic: Logistic,
    henon: Henon,
    mask: Mask,
    chaos: Chaos,
    replacement: bool,
    step: fn(&mut Engine, Event, u64) -> Result<Observation, PortError>,
    previous_response: f64,
    last_time: Option<u64>,
    pending: HashMap<(u64, u64, u16), u32>,
    peak_energy: f64,
}
impl Engine {
    pub fn new(order: Order, chaos: Chaos, mask: Mask, seed: u64) -> Self {
        let step = match order {
            Order::ReactBeforeHabituate => Self::react_before_habit,
            Order::HabituateBeforeReact => Self::habit_before_react,
        };
        Self {
            destination: TwoAmplitude::default(),
            wave: PortWave::default(),
            information: SiteInformation::default(),
            chemistry: Chemistry::default(),
            mind: Adaptation::default(),
            elastic: Elastic::default(),
            thermal: Thermal::default(),
            logistic: Logistic::new(seed),
            henon: Henon::new(seed ^ 0x5c4a_71ee),
            mask,
            chaos,
            replacement: false,
            step,
            previous_response: 1.0,
            last_time: None,
            pending: HashMap::new(),
            peak_energy: 0.0,
        }
    }
    pub fn with_classical_replacement(mut self) -> Self {
        self.replacement = true;
        self
    }
    pub fn observe(&mut self, event: Event) -> Result<Observation, PortError> {
        if event.port == 0 {
            return Err(PortError::InvalidPort);
        }
        if self.last_time.is_some_and(|last| event.time_ns < last) {
            return Err(PortError::BackwardTime);
        }
        let key = (event.origin, event.destination, event.port);
        match event.phase {
            Phase::Attempt => {
                if self.pending.len() >= MAX_PENDING && !self.pending.contains_key(&key) {
                    return Err(PortError::PendingCapacity);
                }
                if self.pending.get(&key) == Some(&u32::MAX) {
                    return Err(PortError::PendingCapacity);
                }
            }
            Phase::Complete => {
                if !self.pending.contains_key(&key) {
                    return Err(PortError::UnmatchedCompletion);
                }
            }
        }
        let dt = self.last_time.map_or(0, |last| event.time_ns - last);
        let result = (self.step)(self, event, dt)?;
        match event.phase {
            Phase::Attempt => {
                *self.pending.entry(key).or_default() += 1;
            }
            Phase::Complete => {
                if let Some(count) = self.pending.get_mut(&key) {
                    *count -= 1;
                    if *count == 0 {
                        self.pending.remove(&key);
                    }
                }
            }
        }
        self.last_time = Some(event.time_ns);
        Ok(Observation {
            pending: self.pending.len(),
            ..result
        })
    }
    pub fn invariants_hold(&self) -> bool {
        self.destination
            .norm()
            .is_none_or(|n| (n - 1.0).abs() < 1e-12)
            && (self.wave.norm() - 1.0).abs() < 1e-12
            && self.information.conserved()
            && self.information.retained() <= 256
            && self.chemistry.valid()
            && self.mind.valid()
            && self.elastic.valid()
            && self.thermal.valid()
            && self.pending.len() <= MAX_PENDING
            && self.peak_energy.is_finite()
    }
    fn quantum(
        &mut self,
        event: Event,
        logistic: f64,
        henon: f64,
    ) -> Result<(f64, [f64; SITES]), PortError> {
        if self.replacement {
            return Ok((0.5, [1.0 / SITES as f64; SITES]));
        }
        if !self.mask.quantum {
            return Ok((0.0, [0.0; SITES]));
        }
        let overlap = self.destination.observe(event.destination)?;
        let mut plain = self.wave.clone();
        let plain_density = plain.observe(event.port, 0.0)?;
        let densities = match self.chaos.phase() {
            Some(source) => {
                let dither = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.wave.clone();
                let candidate_density = candidate.observe(event.port, 0.05 * dither)?;
                if candidate.entropy() > plain.entropy() {
                    self.wave = candidate;
                    candidate_density
                } else {
                    self.wave = plain;
                    plain_density
                }
            }
            None => {
                self.wave = plain;
                plain_density
            }
        };
        Ok((overlap, densities))
    }
    fn chemistry(
        &mut self,
        event: Event,
        overlap: f64,
        density: f64,
        response: f64,
        dt: u64,
        logistic: f64,
        henon: f64,
    ) -> Result<[f64; SITES], PortError> {
        if !self.mask.chemistry {
            return Ok([0.0; SITES]);
        }
        let reaction = match event.phase {
            Phase::Attempt => Reaction::Adsorb,
            Phase::Complete => Reaction::Release,
        };
        let mut plain = self.chemistry.clone();
        let plain_occupied =
            plain.observe(event.port, reaction, overlap, density, response, 1.0, dt)?;
        let occupied = match self.chaos.reaction() {
            Some(source) => {
                let dither = match source {
                    Source::Logistic => logistic,
                    Source::Henon => henon,
                };
                let mut candidate = self.chemistry.clone();
                let candidate_occupied = candidate.observe(
                    event.port,
                    reaction,
                    overlap,
                    density,
                    response,
                    1.0 + 0.05 * dither,
                    dt,
                )?;
                if candidate.free_energy() < plain.free_energy() {
                    self.chemistry = candidate;
                    candidate_occupied
                } else {
                    self.chemistry = plain;
                    plain_occupied
                }
            }
            None => {
                self.chemistry = plain;
                plain_occupied
            }
        };
        Ok(occupied)
    }
    fn react_before_habit(&mut self, event: Event, dt: u64) -> Result<Observation, PortError> {
        self.compose(event, dt, false)
    }
    fn habit_before_react(&mut self, event: Event, dt: u64) -> Result<Observation, PortError> {
        self.compose(event, dt, true)
    }
    fn compose(
        &mut self,
        event: Event,
        dt: u64,
        cognition_first: bool,
    ) -> Result<Observation, PortError> {
        let logistic = self.logistic.step();
        let henon = self.henon.step();
        let (overlap, densities) = self.quantum(event, logistic, henon)?;
        let entropy = if self.mask.information {
            self.information.observe(event.port)?
        } else {
            0.0
        };
        let density = densities[site(event.port)];
        let (occupied, response) = if cognition_first {
            let response = if self.mask.cognition {
                self.mind.observe(event.port, dt)?
            } else {
                1.0
            };
            let occupied =
                self.chemistry(event, overlap, density, response, dt, logistic, henon)?;
            (occupied, response)
        } else {
            let occupied = self.chemistry(
                event,
                overlap,
                density,
                self.previous_response,
                dt,
                logistic,
                henon,
            )?;
            let response = if self.mask.cognition {
                self.mind.observe(event.port, dt)?
            } else {
                1.0
            };
            (occupied, response)
        };
        self.previous_response = response;
        let work = if self.mask.mechanics {
            self.elastic.observe(occupied, dt)?
        } else {
            [0.0; SITES]
        };
        let heat = if self.mask.physics {
            self.thermal.observe(work, entropy, dt)?
        } else {
            [0.0; SITES]
        };
        let energy = finite(
            (0..SITES)
                .map(|i| work[i] + heat[i])
                .fold(0.0_f64, f64::max),
        )?;
        self.peak_energy = self.peak_energy.max(energy);
        Ok(Observation {
            energy,
            peak_energy: self.peak_energy,
            rupture: energy >= 1.0,
            destination_overlap: overlap,
            port_entropy: entropy,
            destination_norm: self.destination.norm(),
            port_norm: self.mask.quantum.then(|| self.wave.norm()),
            retained_observations: self.information.retained(),
            pending: self.pending.len(),
        })
    }
}

/// A fixed route to the selected engine, returning its observed result.
pub struct PortOrchestrator;
impl PortOrchestrator {
    pub fn observe(engine: &mut Engine, input: Event) -> Result<Observation, PortError> {
        engine.observe(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parser_accepts_paired_trace_and_rejects_bad_input() {
        let valid =
            parse_trace("# t,o,d,p,phase\n0,7,11,4000,attempt\n100,7,11,4000,complete\n").unwrap();
        assert_eq!(valid.len(), 2);
        assert_eq!(
            parse_trace("0,7,11,4000,complete"),
            Err(PortError::UnmatchedCompletion)
        );
        assert_eq!(
            parse_trace("0,7,11,4000,attempt\n1,7,11,4000,complete\n0,7,11,4001,attempt"),
            Err(PortError::BackwardTime)
        );
        assert!(parse_trace("0,7,11,0,attempt").is_err());
        assert!(parse_trace("0,7,11,4000,maybe").is_err());
    }
    #[test]
    fn fixed_route_and_error_preserve_state() {
        let mut engine = Engine::new(Order::ReactBeforeHabituate, Chaos::Plain, Mask::FULL, 0);
        let input = Event {
            time_ns: 100,
            origin: 7,
            destination: 11,
            port: 4000,
            phase: Phase::Attempt,
        };
        let first = PortOrchestrator::observe(&mut engine, input).unwrap();
        assert!(first.energy.is_finite() && engine.invariants_hold());
        assert_eq!(
            PortOrchestrator::observe(
                &mut engine,
                Event {
                    time_ns: 101,
                    phase: Phase::Complete,
                    port: 4001,
                    ..input
                }
            ),
            Err(PortError::UnmatchedCompletion)
        );
        assert_eq!(
            PortOrchestrator::observe(
                &mut engine,
                Event {
                    time_ns: 99,
                    ..input
                }
            ),
            Err(PortError::BackwardTime)
        );
        assert_eq!(engine.pending.len(), 1);
    }
}
