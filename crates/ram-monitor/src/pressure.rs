//! Strict uploaded PSI-counter boundary for the bounded pressure material stack.
//! This module performs no live sampling, attack attribution or remediation.

use atom_pressure_engine::{Chaos, Engine, Mask, Order, PressureOrchestrator};
use std::fmt;
use std::path::Path;

use crate::bounded_file::{read_regular_text, ReadError};

pub const RECORDS: usize = 97;
pub const TICKS: usize = RECORDS - 1;
pub const MAX_FILE_BYTES: u64 = 32_768;
const MIN_INTERVAL_NS: u64 = 900_000_000;
const MAX_INTERVAL_NS: u64 = 1_100_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PressureRecord {
    pub time_ns: u64,
    pub some_total_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PressureError {
    Io(String),
    Line { line: usize, reason: &'static str },
    WrongCount(usize),
    FileTooLarge(u64),
    Engine(String),
    NonFinite,
}
impl fmt::Display for PressureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => f.write_str(message),
            Self::Line { line, reason } => write!(f, "line {line}: {reason}"),
            Self::WrongCount(n) => write!(f, "expected exactly {RECORDS} records; got {n}"),
            Self::FileTooLarge(n) => write!(f, "pressure trace is too large: {n} bytes"),
            Self::Engine(message) => write!(f, "pressure engine: {message}"),
            Self::NonFinite => f.write_str("pressure engine returned a non-finite observation"),
        }
    }
}
impl std::error::Error for PressureError {}

pub(crate) fn decimal(raw: &str, line: usize) -> Result<u64, PressureError> {
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(PressureError::Line {
            line,
            reason: "expected an unsigned decimal integer",
        });
    }
    raw.parse::<u64>().map_err(|_| PressureError::Line {
        line,
        reason: "unsigned integer exceeds u64",
    })
}

pub fn parse_trace(text: &str) -> Result<[PressureRecord; RECORDS], PressureError> {
    let mut rows = Vec::with_capacity(RECORDS);
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let mut fields = raw.split(',');
        let first = fields.next().ok_or(PressureError::Line {
            line,
            reason: "expected time_ns,some_total_us",
        })?;
        let second = fields.next().ok_or(PressureError::Line {
            line,
            reason: "expected time_ns,some_total_us",
        })?;
        if fields.next().is_some() {
            return Err(PressureError::Line {
                line,
                reason: "expected time_ns,some_total_us",
            });
        }
        let current = PressureRecord {
            time_ns: decimal(first, line)?,
            some_total_us: decimal(second, line)?,
        };
        if let Some(previous) = rows.last() {
            validate_interval(previous, &current, line)?;
        }
        rows.push(current);
        if rows.len() > RECORDS {
            return Err(PressureError::WrongCount(rows.len()));
        }
    }
    rows.try_into()
        .map_err(|rows: Vec<PressureRecord>| PressureError::WrongCount(rows.len()))
}

pub(crate) fn validate_interval(
    previous: &PressureRecord,
    current: &PressureRecord,
    line: usize,
) -> Result<f64, PressureError> {
    let dt = current
        .time_ns
        .checked_sub(previous.time_ns)
        .ok_or(PressureError::Line {
            line,
            reason: "timestamp moved backward",
        })?;
    if !(MIN_INTERVAL_NS..=MAX_INTERVAL_NS).contains(&dt) {
        return Err(PressureError::Line {
            line,
            reason: "expected a 0.9 to 1.1 second interval",
        });
    }
    let delta = current
        .some_total_us
        .checked_sub(previous.some_total_us)
        .ok_or(PressureError::Line {
            line,
            reason: "some total moved backward",
        })?;
    if u128::from(delta) * 1_000 > u128::from(dt) {
        return Err(PressureError::Line {
            line,
            reason: "stall time exceeds elapsed time",
        });
    }
    let fraction = (delta as f64 * 1_000.0) / dt as f64;
    if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
        return Err(PressureError::NonFinite);
    }
    Ok(fraction)
}

pub fn read_trace(path: &Path) -> Result<[PressureRecord; RECORDS], PressureError> {
    let text = read_regular_text(path, MAX_FILE_BYTES as usize).map_err(|error| match error {
        ReadError::TooLarge(size) => PressureError::FileTooLarge(size),
        other => PressureError::Io(format!("{}: {other}", path.display())),
    })?;
    parse_trace(&text)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PressureTick {
    pub index: usize,
    pub fraction: f64,
    pub energy: f64,
    pub crossed: bool,
    pub population: f64,
    pub coherence: f64,
    pub entropy: f64,
    pub binding: f64,
    pub habituation: f64,
    pub position: f64,
    pub heat: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PressureReceipt {
    pub first_crossing: Option<usize>,
    pub peak_energy: f64,
    pub final_energy: f64,
    pub peak_fraction: f64,
    pub ticks: Vec<PressureTick>,
}

pub fn evaluate(records: &[PressureRecord; RECORDS]) -> Result<PressureReceipt, PressureError> {
    let mut engine = Engine::new(Order::TransportThenSource, Mask::FULL, Chaos::Plain, 0)
        .map_err(|error| PressureError::Engine(error.to_string()))?;
    let mut ticks = Vec::with_capacity(TICKS);
    let mut first_crossing = None;
    let mut peak_energy = 0.0_f64;
    let mut peak_fraction = 0.0_f64;
    for (index, pair) in records.windows(2).enumerate() {
        let fraction = validate_interval(&pair[0], &pair[1], index + 2)?;
        let state = Engine::observe(&mut engine, fraction)
            .map_err(|error| PressureError::Engine(error.to_string()))?;
        let values = [
            state.energy,
            state.population,
            state.coherence,
            state.trace,
            state.determinant,
            state.entropy,
            state.binding,
            state.habituation,
            state.position,
            state.heat,
        ];
        if values.iter().any(|value| !value.is_finite()) {
            return Err(PressureError::NonFinite);
        }
        if state.crossed && first_crossing.is_none() {
            first_crossing = Some(index + 1);
        }
        peak_energy = peak_energy.max(state.energy);
        peak_fraction = peak_fraction.max(fraction);
        ticks.push(PressureTick {
            index,
            fraction,
            energy: state.energy,
            crossed: state.crossed,
            population: state.population,
            coherence: state.coherence,
            entropy: state.entropy,
            binding: state.binding,
            habituation: state.habituation,
            position: state.position,
            heat: state.heat,
        });
    }
    let final_energy = ticks.last().map_or(0.0, |tick| tick.energy);
    Ok(PressureReceipt {
        first_crossing,
        peak_energy,
        final_energy,
        peak_fraction,
        ticks,
    })
}
