//! Conventional portable 64-event input boundary for the measured four-mode
//! rhythm component. No endpoint, attack name or policy enters the engine.

use std::fmt;
use std::path::Path;

use crate::bounded_file::{read_regular_text, ReadError};
use atom_rhythm_four_engine::{Chaos, Engine, Event, Mask, Order, RhythmFourOrchestrator};

pub const EVENTS: usize = 64;
pub const MAX_FILE_BYTES: u64 = 65_536;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RhythmError {
    Io(String),
    Line { line: usize, reason: &'static str },
    WrongCount(usize),
    FileTooLarge(u64),
    Engine(String),
    NonFinite,
}
impl fmt::Display for RhythmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => f.write_str(message),
            Self::Line { line, reason } => write!(f, "line {line}: {reason}"),
            Self::WrongCount(n) => write!(f, "expected exactly {EVENTS} observations; got {n}"),
            Self::FileTooLarge(n) => write!(f, "timing trace is too large: {n} bytes"),
            Self::Engine(message) => write!(f, "rhythm engine: {message}"),
            Self::NonFinite => f.write_str("rhythm engine returned a non-finite observation"),
        }
    }
}
impl std::error::Error for RhythmError {}

fn number(value: &str, line: usize) -> Result<u64, RhythmError> {
    let value = value.trim();
    let parsed = if let Some(hex) = value.strip_prefix("0x") {
        u64::from_str_radix(hex, 16)
    } else {
        value.parse::<u64>()
    };
    parsed.map_err(|_| RhythmError::Line {
        line,
        reason: "invalid unsigned integer",
    })
}

pub fn parse_trace(text: &str) -> Result<[Event; EVENTS], RhythmError> {
    let mut events = Vec::with_capacity(EVENTS);
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split(',');
        let time = fields.next().ok_or(RhythmError::Line {
            line: index + 1,
            reason: "missing time_ns",
        })?;
        let size = fields.next().ok_or(RhythmError::Line {
            line: index + 1,
            reason: "expected time_ns,bytes_out",
        })?;
        if fields.next().is_some() {
            return Err(RhythmError::Line {
                line: index + 1,
                reason: "expected time_ns,bytes_out",
            });
        }
        let time_ns = number(time, index + 1)?;
        let raw_bytes = number(size, index + 1)?;
        let bytes_out = u32::try_from(raw_bytes).map_err(|_| RhythmError::Line {
            line: index + 1,
            reason: "bytes_out exceeds u32",
        })?;
        if events
            .last()
            .is_some_and(|previous: &Event| time_ns <= previous.time_ns)
        {
            return Err(RhythmError::Line {
                line: index + 1,
                reason: "timestamp must increase",
            });
        }
        events.push(Event { time_ns, bytes_out });
        if events.len() > EVENTS {
            return Err(RhythmError::WrongCount(events.len()));
        }
    }
    events
        .try_into()
        .map_err(|events: Vec<Event>| RhythmError::WrongCount(events.len()))
}

pub fn read_trace(path: &Path) -> Result<[Event; EVENTS], RhythmError> {
    let text = read_regular_text(path, MAX_FILE_BYTES as usize).map_err(|error| match error {
        ReadError::TooLarge(size) => RhythmError::FileTooLarge(size),
        other => RhythmError::Io(format!("{}: {other}", path.display())),
    })?;
    parse_trace(&text)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmTick {
    pub index: usize,
    pub energy: f64,
    pub purity: f64,
    pub fundamental: f64,
    pub overtone: f64,
    pub binding: f64,
    pub adaptation: f64,
    pub strain: [f64; 2],
    pub heat: [f64; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct RhythmReceipt {
    pub tau_ns: f64,
    pub first_crossing: Option<usize>,
    pub peak_energy: f64,
    pub final_energy: f64,
    pub ticks: Vec<RhythmTick>,
}

pub fn evaluate(events: &[Event; EVENTS]) -> Result<RhythmReceipt, RhythmError> {
    let elapsed = events[EVENTS - 1]
        .time_ns
        .checked_sub(events[0].time_ns)
        .ok_or(RhythmError::Engine("time moved backward".into()))?;
    let tau_ns = elapsed as f64 / (EVENTS - 1) as f64;
    if !tau_ns.is_finite() || tau_ns <= 0.0 {
        return Err(RhythmError::NonFinite);
    }
    let mut engine = Engine::new(
        Order::TransportThenSource,
        Chaos::Plain,
        Mask::FULL,
        0,
        tau_ns,
    )
    .map_err(|error| RhythmError::Engine(error.to_string()))?;
    let mut ticks = Vec::with_capacity(EVENTS);
    let mut first_crossing = None;
    let mut peak_energy = 0.0_f64;
    for (index, event) in events.iter().enumerate() {
        let output = RhythmFourOrchestrator::observe(&mut engine, *event)
            .map_err(|error| RhythmError::Engine(error.to_string()))?;
        if output.rupture && first_crossing.is_none() {
            first_crossing = Some(index + 1);
        }
        peak_energy = peak_energy.max(output.energy);
        let values = [
            output.energy,
            output.purity,
            output.harmonics[0],
            output.harmonics[1],
            output.binding,
            output.adaptation,
            output.strain[0],
            output.strain[1],
            output.heat[0],
            output.heat[1],
        ];
        if values.iter().any(|value| !value.is_finite()) {
            return Err(RhythmError::NonFinite);
        }
        ticks.push(RhythmTick {
            index,
            energy: output.energy,
            purity: output.purity,
            fundamental: output.harmonics[0],
            overtone: output.harmonics[1],
            binding: output.binding,
            adaptation: output.adaptation,
            strain: output.strain,
            heat: output.heat,
        });
    }
    let final_energy = ticks.last().map_or(0.0, |tick| tick.energy);
    Ok(RhythmReceipt {
        tau_ns,
        first_crossing,
        peak_energy,
        final_energy,
        ticks,
    })
}
