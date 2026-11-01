//! Quorum-swarm primitives: collective coupling and receipts over a fixed
//! set of owned cells. This crate holds no secret bytes and produces no
//! copy/clone verdict; receipts are numerical only.

use std::fmt;

/// Fixed swarm size: four owned cells, matching the four field sites.
pub const SWARM_CELLS: usize = 4;

/// Fracture threshold of the underlying key cell (heat at/above ruptures).
pub const RUPTURE_HEAT: f64 = 0.25;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidConfig,
    InvalidSite,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}

impl std::error::Error for Error {}

/// Fixed experiment structure: coupling strength Q, warm threshold theta,
/// quorum K. Never selected from observation content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwarmConfig {
    pub coupling: f64,
    pub warm_threshold: f64,
    pub quorum: usize,
}

impl SwarmConfig {
    pub fn new(coupling: f64, warm_threshold: f64, quorum: usize) -> Result<Self, Error> {
        if !coupling.is_finite() || coupling < 0.0 {
            return Err(Error::InvalidConfig);
        }
        if !warm_threshold.is_finite() || warm_threshold <= 0.0 || warm_threshold >= RUPTURE_HEAT {
            return Err(Error::InvalidConfig);
        }
        if quorum == 0 || quorum > SWARM_CELLS {
            return Err(Error::InvalidConfig);
        }
        Ok(Self { coupling, warm_threshold, quorum })
    }
}

/// Numerical-only swarm receipt. No key bytes, no verdict.
#[derive(Clone, Copy, Debug)]
pub struct SwarmReceipt {
    pub trip_reads: i64,
    pub warm: usize,
    pub ruptured: usize,
    pub heats: [f64; SWARM_CELLS],
    pub ruptured_flags: [bool; SWARM_CELLS],
}

/// Quorum-sensing coupling: the addressed cell feels a fraction Q of the
/// other cells' current heat (collective activation / thermal diffusion).
pub fn coupling_term(heats: &[f64; SWARM_CELLS], site: usize, config: SwarmConfig) -> Result<f64, Error> {
    if site >= SWARM_CELLS {
        return Err(Error::InvalidSite);
    }
    let mut sum = 0.0_f64;
    for (j, heat) in heats.iter().enumerate() {
        if j != site {
            sum += heat;
        }
    }
    Ok(config.coupling * sum)
}

/// Number of cells at or above the warm threshold.
pub fn warm_count(heats: &[f64; SWARM_CELLS], config: SwarmConfig) -> usize {
    heats.iter().filter(|heat| **heat >= config.warm_threshold).count()
}

/// Number of ruptured cells (heat at/above the fracture threshold).
pub fn ruptured_count(heats: &[f64; SWARM_CELLS]) -> usize {
    heats.iter().filter(|heat| **heat >= RUPTURE_HEAT).count()
}

/// Evaluate the swarm receipt. `trip_reads` is the global recipient-read
/// count at which the receipt first fired, or -1 if it has not fired within
/// the observed window.
pub fn receipt(
    heats: &[f64; SWARM_CELLS],
    ruptured_flags: &[bool; SWARM_CELLS],
    trip_reads: i64,
    config: SwarmConfig,
) -> SwarmReceipt {
    let warm = warm_count(heats, config);
    let ruptured = ruptured_count(heats);
    SwarmReceipt {
        trip_reads,
        warm,
        ruptured,
        heats: *heats,
        ruptured_flags: *ruptured_flags,
    }
}

/// The receipt fires when any cell has ruptured or the warm quorum is met.
pub fn fired(heats: &[f64; SWARM_CELLS], config: SwarmConfig) -> bool {
    ruptured_count(heats) > 0 || warm_count(heats, config) >= config.quorum
}
