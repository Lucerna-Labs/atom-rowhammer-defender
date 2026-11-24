//! Conventional file-pair input boundary for the unchanged E012/E013 Atom
//! component. The byte comparison is external evidence, never an Atom input
//! label or a claim of reference authenticity.

use std::fmt;
use std::path::Path;

use crate::bounded_file::{read_regular_bytes_with_metadata, ReadError};
use axiom_engine::Engine;

pub const RECORD_BYTES: usize = 160;
pub const TICKS: usize = 320;
const SEED: u64 = 2202;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IntegrityError {
    Io(String),
    Length { role: &'static str, size: usize },
    Engine(String),
    NonFinite,
}
impl fmt::Display for IntegrityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => formatter.write_str(message),
            Self::Length { role, size } => write!(
                formatter,
                "{role} must contain exactly {RECORD_BYTES} bytes; read {size}"
            ),
            Self::Engine(message) => write!(formatter, "integrity engine: {message}"),
            Self::NonFinite => formatter.write_str("integrity engine produced a non-finite value"),
        }
    }
}
impl std::error::Error for IntegrityError {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntegrityTick {
    pub index: usize,
    pub materials: [f64; 4],
    pub mean: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IntegrityReceipt {
    pub ticks: Vec<IntegrityTick>,
    pub long_mean: f64,
    pub original_mean: f64,
    pub peak_mean: f64,
    /// Exact conventional byte comparison; it never enters the Atom engine.
    pub different_bytes: usize,
}

pub fn read_record(path: &Path, role: &'static str) -> Result<[u8; RECORD_BYTES], IntegrityError> {
    read_record_with_metadata(path, role).map(|(record, _)| record)
}

pub fn read_record_with_metadata(
    path: &Path,
    role: &'static str,
) -> Result<([u8; RECORD_BYTES], std::fs::Metadata), IntegrityError> {
    let (bytes, metadata) =
        read_regular_bytes_with_metadata(path, RECORD_BYTES + 1).map_err(|error| match error {
            ReadError::TooLarge(_) => IntegrityError::Length {
                role,
                size: RECORD_BYTES + 1,
            },
            other => IntegrityError::Io(format!("{}: {other}", path.display())),
        })?;
    if bytes.len() != RECORD_BYTES {
        return Err(IntegrityError::Length {
            role,
            size: bytes.len(),
        });
    }
    let record = bytes
        .try_into()
        .map_err(|bytes: Vec<u8>| IntegrityError::Length {
            role,
            size: bytes.len(),
        })?;
    Ok((record, metadata))
}

pub fn compare_files(
    reference: &Path,
    observed: &Path,
) -> Result<IntegrityReceipt, IntegrityError> {
    let reference = read_record(reference, "reference")?;
    let observed = read_record(observed, "observed")?;
    compare_records(&reference, &observed)
}

pub fn compare_records(
    reference: &[u8; RECORD_BYTES],
    observed: &[u8; RECORD_BYTES],
) -> Result<IntegrityReceipt, IntegrityError> {
    let mut engine =
        Engine::new("B", "full", "logistic", SEED, "off").map_err(IntegrityError::Engine)?;
    let mut ticks = Vec::with_capacity(TICKS);
    let mut long_sum = 0.0;
    let mut original_sum = 0.0;
    let mut peak_mean = 0.0_f64;
    for (&before, &after) in reference.iter().zip(observed.iter()) {
        for offset in [0, 4] {
            let input = std::array::from_fn(|index| {
                let bit = offset + index / 2;
                let byte = if index % 2 == 0 { before } else { after };
                f64::from((byte >> bit) & 1)
            });
            engine = engine.step(input).map_err(IntegrityError::Engine)?;
            let index = ticks.len();
            let materials = engine.sites.each_ref().map(|site| site.output);
            let mean = materials.iter().sum::<f64>() / 4.0;
            if !mean.is_finite() || materials.iter().any(|value| !value.is_finite()) {
                return Err(IntegrityError::NonFinite);
            }
            if (240..304).contains(&index) {
                long_sum += mean;
            }
            if (160..224).contains(&index) {
                original_sum += mean;
            }
            peak_mean = peak_mean.max(mean);
            ticks.push(IntegrityTick {
                index,
                materials,
                mean,
            });
        }
    }
    if ticks.len() != TICKS {
        return Err(IntegrityError::Engine("wrong tick count".into()));
    }
    Ok(IntegrityReceipt {
        ticks,
        long_mean: long_sum / 64.0,
        original_mean: original_sum / 64.0,
        peak_mean,
        different_bytes: reference
            .iter()
            .zip(observed.iter())
            .filter(|(a, b)| a != b)
            .count(),
    })
}
