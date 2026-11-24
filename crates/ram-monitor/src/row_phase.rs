//! Uploaded mathematical 64-byte-block trace into independent phase material.
//! The existing Rowhammer monitor and its swarms remain separate and unchanged.
use atom_row_phase_engine::{Chaos, Engine, Event, Mask, Order, RowPhaseOrchestrator};
use std::fmt;
use std::path::Path;

use crate::trace::TraceError;
use crate::{AccessSample, AddressKind, RowGeometry};

pub const EVENTS: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowPhaseError {
    Trace(TraceError),
    WrongCount(usize),
    InvalidSample(usize),
    TimeOrder(usize),
    Engine(String),
    NonFinite,
}
impl fmt::Display for RowPhaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Trace(error) => write!(f, "{error}"),
            Self::WrongCount(n) => {
                write!(f, "expected exactly {EVENTS} address observations; got {n}")
            }
            Self::InvalidSample(n) => {
                write!(f, "observation {n} is not an uploaded mathematical address")
            }
            Self::TimeOrder(n) => write!(f, "observation {n} timestamp must strictly increase"),
            Self::Engine(error) => write!(f, "row phase engine: {error}"),
            Self::NonFinite => f.write_str("row phase engine returned non-finite state"),
        }
    }
}
impl std::error::Error for RowPhaseError {}

pub fn read_upload(path: &Path) -> Result<Vec<AccessSample>, RowPhaseError> {
    let samples = crate::trace::read_trace(path).map_err(RowPhaseError::Trace)?;
    validate_samples(&samples)?;
    Ok(samples)
}

fn validate_samples(samples: &[AccessSample]) -> Result<(), RowPhaseError> {
    if samples.len() != EVENTS {
        return Err(RowPhaseError::WrongCount(samples.len()));
    }
    let mut last = None;
    for (index, sample) in samples.iter().enumerate() {
        if sample.phys != 0
            || sample.address_kind != AddressKind::Synthetic
            || sample.time_ns.is_none()
        {
            return Err(RowPhaseError::InvalidSample(index + 1));
        }
        let Some(time) = sample.time_ns else {
            return Err(RowPhaseError::InvalidSample(index + 1));
        };
        if last.is_some_and(|previous| time <= previous) {
            return Err(RowPhaseError::TimeOrder(index + 1));
        }
        last = Some(time);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowPhaseTick {
    pub index: usize,
    pub time_ns: u64,
    pub fingerprint: u64,
    pub energy: f64,
    pub crossed: bool,
    pub power: f64,
    pub real: f64,
    pub imag: f64,
    pub entropy: f64,
    pub binding: f64,
    pub habituation: f64,
    pub position: f64,
    pub heat: f64,
    pub alpha: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RowPhaseReceipt {
    pub first_crossing: Option<usize>,
    pub peak_energy: f64,
    pub final_energy: f64,
    pub peak_power: f64,
    pub final_power: f64,
    pub ticks: Vec<RowPhaseTick>,
}

pub fn evaluate(samples: &[AccessSample]) -> Result<RowPhaseReceipt, RowPhaseError> {
    validate_samples(samples)?;
    let geometry = RowGeometry::default();
    let mut engine = Engine::new(Order::PhaseInformationFirst, Mask::FULL, Chaos::Plain, 0)
        .map_err(|error| RowPhaseError::Engine(error.to_string()))?;
    let mut ticks = Vec::with_capacity(EVENTS);
    let mut first_crossing = None;
    let mut peak_energy = 0.0_f64;
    let mut peak_power = 0.0_f64;
    for (index, sample) in samples.iter().enumerate() {
        let Some(time_ns) = sample.time_ns else {
            return Err(RowPhaseError::InvalidSample(index + 1));
        };
        let fingerprint = geometry.fingerprint(sample.addr);
        let output = Engine::observe(
            &mut engine,
            Event {
                time_ns,
                fingerprint,
            },
        )
        .map_err(|error| RowPhaseError::Engine(error.to_string()))?;
        let values = [
            output.energy,
            output.power,
            output.real,
            output.imag,
            output.entropy,
            output.binding,
            output.habituation,
            output.position,
            output.heat,
            output.alpha,
        ];
        if values.iter().any(|value| !value.is_finite()) {
            return Err(RowPhaseError::NonFinite);
        }
        if output.crossed && first_crossing.is_none() {
            first_crossing = Some(index + 1);
        }
        peak_energy = peak_energy.max(output.energy);
        peak_power = peak_power.max(output.power);
        ticks.push(RowPhaseTick {
            index,
            time_ns,
            fingerprint,
            energy: output.energy,
            crossed: output.crossed,
            power: output.power,
            real: output.real,
            imag: output.imag,
            entropy: output.entropy,
            binding: output.binding,
            habituation: output.habituation,
            position: output.position,
            heat: output.heat,
            alpha: output.alpha,
        });
    }
    let final_energy = ticks.last().map_or(0.0, |tick| tick.energy);
    let final_power = ticks.last().map_or(0.0, |tick| tick.power);
    Ok(RowPhaseReceipt {
        first_crossing,
        peak_energy,
        final_energy,
        peak_power,
        final_power,
        ticks,
    })
}
