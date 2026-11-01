use std::fmt;
use std::path::Path;

use crate::{AccessSample, AccessSource, AddressKind};

pub const TRACE_MAX_SAMPLES: usize = 1_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceError {
    message: String,
}

impl TraceError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for TraceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TraceError {}

fn number(value: &str, line: usize, field: &str) -> Result<u64, TraceError> {
    let value = value.trim();
    let parsed = if let Some(hex) = value.strip_prefix("0x") {
        u64::from_str_radix(hex, 16)
    } else {
        value.parse::<u64>()
    };
    parsed.map_err(|_| TraceError::new(format!("line {line}: invalid {field}")))
}

pub fn parse_trace(text: &str) -> Result<Vec<AccessSample>, TraceError> {
    let mut samples = Vec::new();
    let mut last_time = None;
    for (index, raw) in text.lines().enumerate() {
        let line_number = index + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split(',');
        let time = fields
            .next()
            .ok_or_else(|| TraceError::new(format!("line {line_number}: missing time_ns")))?;
        let address = fields
            .next()
            .ok_or_else(|| TraceError::new(format!("line {line_number}: missing address")))?;
        if fields.next().is_some() {
            return Err(TraceError::new(format!(
                "line {line_number}: expected time_ns,address"
            )));
        }
        let time_ns = number(time, line_number, "time_ns")?;
        let addr = number(address, line_number, "address")?;
        if last_time.is_some_and(|last| time_ns < last) {
            return Err(TraceError::new(format!(
                "line {line_number}: timestamp moved backward"
            )));
        }
        last_time = Some(time_ns);
        samples.push(AccessSample {
            addr,
            time_ns: Some(time_ns),
            address_kind: AddressKind::Synthetic,
            ..AccessSample::default()
        });
        if samples.len() > TRACE_MAX_SAMPLES {
            return Err(TraceError::new(format!(
                "line {line_number}: trace exceeds {TRACE_MAX_SAMPLES} samples"
            )));
        }
    }
    if samples.is_empty() {
        return Err(TraceError::new("trace contains no samples"));
    }
    Ok(samples)
}

pub fn read_trace(path: &Path) -> Result<Vec<AccessSample>, TraceError> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| TraceError::new(format!("{}: {error}", path.display())))?;
    parse_trace(&text)
}

pub struct TraceSource {
    samples: Vec<AccessSample>,
    position: usize,
}

impl TraceSource {
    pub fn new(samples: Vec<AccessSample>) -> Result<Self, TraceError> {
        if samples.is_empty() {
            return Err(TraceError::new("trace contains no samples"));
        }
        Ok(Self {
            samples,
            position: 0,
        })
    }
}

impl AccessSource for TraceSource {
    fn next_sample(&mut self) -> Option<AccessSample> {
        let sample = self.samples.get(self.position).copied()?;
        self.position += 1;
        Some(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_portable_trace_and_hex_addresses() {
        let samples = parse_trace("# time,address\n0,0x1000\n50,4160\n").unwrap();
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].addr, 0x1000);
        assert_eq!(samples[1].time_ns, Some(50));
    }

    #[test]
    fn rejects_empty_malformed_and_backward_time() {
        assert!(parse_trace("# only a comment\n").is_err());
        assert!(parse_trace("0\n").is_err());
        assert!(parse_trace("10,0x1000\n9,0x1040\n").is_err());
        assert!(parse_trace("0,1,2\n").is_err());
    }
}
