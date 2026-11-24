//! Continuing read-only comparison against one captured trusted reference.
//! Exact byte differences drive alerts; E012/E013 remains a numeric receipt.

use std::path::{Path, PathBuf};

use crate::file_object::{reading_from_metadata, ObjectIdentity};
use crate::integrity::{
    compare_records, read_record, read_record_with_metadata, IntegrityError, IntegrityReceipt,
    RECORD_BYTES,
};

#[derive(Clone, Debug, PartialEq)]
pub enum WatchCheck {
    Compared(IntegrityReceipt),
    ComparedWithObject(IntegrityReceipt, bool),
    ReadError(String),
    EngineError(String),
}
impl WatchCheck {
    pub fn alert(&self) -> bool {
        match self {
            Self::Compared(receipt) => receipt.different_bytes > 0,
            Self::ComparedWithObject(receipt, changed) => receipt.different_bytes > 0 || *changed,
            Self::ReadError(_) | Self::EngineError(_) => true,
        }
    }
}

pub struct IntegrityWatch {
    reference: [u8; RECORD_BYTES],
    observed: PathBuf,
    object_baseline: Option<ObjectIdentity>,
    checks: u64,
    ever_alerted: bool,
    current: WatchCheck,
}
impl IntegrityWatch {
    pub fn trusted_reference(&self) -> &[u8; RECORD_BYTES] {
        &self.reference
    }
    pub fn baseline_object_identity(&self) -> Option<ObjectIdentity> {
        self.object_baseline
    }

    /// Accept a new object baseline only after the selected path now holds
    /// the already captured trusted bytes. Never reload the reference path.
    pub fn rearm_verified_object(&mut self) -> Result<(), IntegrityError> {
        let (record, metadata) = read_record_with_metadata(&self.observed, "observed")?;
        if record != self.reference {
            return Err(IntegrityError::Io(
                "restored bytes differ from captured reference".into(),
            ));
        }
        let identity = reading_from_metadata(&metadata)
            .map_err(|error| IntegrityError::Io(error.to_string()))?
            .identity;
        let receipt = compare_records(&self.reference, &record)?;
        self.object_baseline = Some(identity);
        self.current = WatchCheck::ComparedWithObject(receipt, false);
        Ok(())
    }

    /// Capture the trusted bytes once and validate the observed path before
    /// returning any streaming output.
    pub fn start(reference: &Path, observed: &Path) -> Result<Self, IntegrityError> {
        Self::start_inner(reference, observed, false)
    }

    pub fn start_with_object(reference: &Path, observed: &Path) -> Result<Self, IntegrityError> {
        Self::start_inner(reference, observed, true)
    }

    fn start_inner(
        reference: &Path,
        observed: &Path,
        with_object: bool,
    ) -> Result<Self, IntegrityError> {
        let reference = read_record(reference, "reference")?;
        let (observed_record, object_baseline) = if with_object {
            let (record, metadata) = read_record_with_metadata(observed, "observed")?;
            let identity = reading_from_metadata(&metadata)
                .map_err(|error| IntegrityError::Io(format!("{}: {error}", observed.display())))?
                .identity;
            (record, Some(identity))
        } else {
            (read_record(observed, "observed")?, None)
        };
        let receipt = compare_records(&reference, &observed_record)?;
        let current = if with_object {
            WatchCheck::ComparedWithObject(receipt, false)
        } else {
            WatchCheck::Compared(receipt)
        };
        Ok(Self {
            reference,
            observed: observed.to_path_buf(),
            object_baseline,
            checks: 1,
            ever_alerted: current.alert(),
            current,
        })
    }

    pub fn poll(&mut self) -> &WatchCheck {
        let observed = if let Some(baseline) = self.object_baseline {
            read_record_with_metadata(&self.observed, "observed").and_then(|(record, metadata)| {
                let changed = reading_from_metadata(&metadata)
                    .map_err(|error| {
                        IntegrityError::Io(format!("{}: {error}", self.observed.display()))
                    })?
                    .identity
                    != baseline;
                Ok((record, Some(changed)))
            })
        } else {
            read_record(&self.observed, "observed").map(|record| (record, None))
        };
        self.current = match observed {
            Ok((record, changed)) => match compare_records(&self.reference, &record) {
                Ok(receipt) => match changed {
                    Some(changed) => WatchCheck::ComparedWithObject(receipt, changed),
                    None => WatchCheck::Compared(receipt),
                },
                Err(error) => WatchCheck::EngineError(error.to_string()),
            },
            Err(error) => WatchCheck::ReadError(error.to_string()),
        };
        self.checks = self.checks.saturating_add(1);
        self.ever_alerted |= self.current.alert();
        &self.current
    }

    pub fn current(&self) -> &WatchCheck {
        &self.current
    }
    pub fn checks(&self) -> u64 {
        self.checks
    }
    pub fn ever_alerted(&self) -> bool {
        self.ever_alerted
    }
}
