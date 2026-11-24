//! Strict uploaded paired-PSI counter input and numerical joint-state receipt.
//! No live collector, resource attribution, attack label or action.
use atom_psi_coupling_engine::{Chaos, CouplingOrchestrator, Engine, Event, Mask, Order};
use std::path::Path;

use crate::bounded_file::{read_regular_text, ReadError};
use crate::pressure::{decimal, validate_interval, PressureError, PressureRecord};

pub const RECORDS: usize = 97;
pub const TICKS: usize = RECORDS - 1;
pub const MAX_FILE_BYTES: u64 = 48_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PsiRecord {
    pub time_ns: u64,
    pub memory_some_total_us: u64,
    pub io_some_total_us: u64,
}

fn fractions(a: &PsiRecord, b: &PsiRecord, line: usize) -> Result<(f64, f64), PressureError> {
    let memory_a = PressureRecord {
        time_ns: a.time_ns,
        some_total_us: a.memory_some_total_us,
    };
    let memory_b = PressureRecord {
        time_ns: b.time_ns,
        some_total_us: b.memory_some_total_us,
    };
    let io_a = PressureRecord {
        time_ns: a.time_ns,
        some_total_us: a.io_some_total_us,
    };
    let io_b = PressureRecord {
        time_ns: b.time_ns,
        some_total_us: b.io_some_total_us,
    };
    Ok((
        validate_interval(&memory_a, &memory_b, line)?,
        validate_interval(&io_a, &io_b, line)?,
    ))
}

pub fn parse_trace(text: &str) -> Result<[PsiRecord; RECORDS], PressureError> {
    let mut rows = Vec::with_capacity(RECORDS);
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let mut fields = raw.split(',');
        let (Some(time), Some(memory), Some(io)) = (fields.next(), fields.next(), fields.next())
        else {
            return Err(PressureError::Line {
                line,
                reason: "expected time_ns,memory_some_total_us,io_some_total_us",
            });
        };
        if fields.next().is_some() {
            return Err(PressureError::Line {
                line,
                reason: "expected time_ns,memory_some_total_us,io_some_total_us",
            });
        }
        let current = PsiRecord {
            time_ns: decimal(time, line)?,
            memory_some_total_us: decimal(memory, line)?,
            io_some_total_us: decimal(io, line)?,
        };
        if let Some(previous) = rows.last() {
            fractions(previous, &current, line)?;
        }
        rows.push(current);
        if rows.len() > RECORDS {
            return Err(PressureError::WrongCount(rows.len()));
        }
    }
    rows.try_into()
        .map_err(|rows: Vec<PsiRecord>| PressureError::WrongCount(rows.len()))
}

pub fn read_trace(path: &Path) -> Result<[PsiRecord; RECORDS], PressureError> {
    let text = read_regular_text(path, MAX_FILE_BYTES as usize).map_err(|error| match error {
        ReadError::TooLarge(size) => PressureError::FileTooLarge(size),
        other => PressureError::Io(format!("{}: {other}", path.display())),
    })?;
    parse_trace(&text)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PsiTick {
    pub index: usize,
    pub memory_fraction: f64,
    pub io_fraction: f64,
    pub energy: f64,
    pub crossed: bool,
    pub occupation: [f64; 4],
    pub joint_coherence: f64,
    pub entropy: f64,
    pub binding: f64,
    pub habituation: f64,
    pub position: f64,
    pub heat: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PsiReceipt {
    pub first_crossing: Option<usize>,
    pub peak_energy: f64,
    pub final_energy: f64,
    pub peak_memory_fraction: f64,
    pub peak_io_fraction: f64,
    pub ticks: Vec<PsiTick>,
}

pub fn evaluate(records: &[PsiRecord; RECORDS]) -> Result<PsiReceipt, PressureError> {
    let mut engine = Engine::new(Order::TransportThenSource, Mask::FULL, Chaos::Plain, 0)
        .map_err(|error| PressureError::Engine(error.to_string()))?;
    let mut ticks = Vec::with_capacity(TICKS);
    let mut first_crossing = None;
    let mut peak_energy = 0.0_f64;
    let mut peak_memory_fraction = 0.0_f64;
    let mut peak_io_fraction = 0.0_f64;
    for (index, pair) in records.windows(2).enumerate() {
        let (memory_fraction, io_fraction) = fractions(&pair[0], &pair[1], index + 2)?;
        let state = Engine::observe(
            &mut engine,
            Event {
                first: memory_fraction,
                second: io_fraction,
            },
        )
        .map_err(|error| PressureError::Engine(error.to_string()))?;
        let values = [
            state.energy,
            state.occupation[0],
            state.occupation[1],
            state.occupation[2],
            state.occupation[3],
            state.coherence_03,
            state.trace,
            state.minimum_minor,
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
        peak_memory_fraction = peak_memory_fraction.max(memory_fraction);
        peak_io_fraction = peak_io_fraction.max(io_fraction);
        ticks.push(PsiTick {
            index,
            memory_fraction,
            io_fraction,
            energy: state.energy,
            crossed: state.crossed,
            occupation: state.occupation,
            joint_coherence: state.coherence_03,
            entropy: state.entropy,
            binding: state.binding,
            habituation: state.habituation,
            position: state.position,
            heat: state.heat,
        });
    }
    let final_energy = ticks.last().map_or(0.0, |tick| tick.energy);
    Ok(PsiReceipt {
        first_crossing,
        peak_energy,
        final_energy,
        peak_memory_fraction,
        peak_io_fraction,
        ticks,
    })
}
