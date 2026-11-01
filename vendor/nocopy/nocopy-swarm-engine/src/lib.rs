//! Swarm orchestrator: one sensor engine per owned cell, observations routed
//! by their declared site, quorum coupling applied to the addressed cell.
//! The orchestrator carries no policy and no key bytes; it returns numerical
//! receipts only.

use nocopy_sensor_engine::{Arm, Direction, Engine, Error as EngineError, KeyCell, Observation, Order};
use nocopy_sensor_principles::Error as PrincipleError;
use nocopy_swarm_principles::{
    coupling_term, fired, receipt, Error as SwarmError, SwarmConfig, SwarmReceipt, SWARM_CELLS,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Swarm(SwarmError),
    Engine(EngineError),
    OutOfOrderObservation,
}

impl From<SwarmError> for Error {
    fn from(error: SwarmError) -> Self { Error::Swarm(error) }
}

impl From<EngineError> for Error {
    fn from(error: EngineError) -> Self { Error::Engine(error) }
}

impl From<PrincipleError> for Error {
    fn from(error: PrincipleError) -> Self { Error::Engine(EngineError::from(error)) }
}

/// N engines + N owned key cells + fixed swarm config. Order, arm, thermal
/// and hill profile are fixed experiment structure.
pub struct SwarmEngine {
    engines: [Engine; SWARM_CELLS],
    keys: [KeyCell; SWARM_CELLS],
    config: SwarmConfig,
    reads: i64,
    trip_reads: i64,
}

impl SwarmEngine {
    pub fn new(
        seeds: [u64; SWARM_CELLS],
        keys: [KeyCell; SWARM_CELLS],
        order: Order,
        config: SwarmConfig,
        thermal: f64,
        hill: bool,
    ) -> Self {
        let engines = seeds.map(|seed| {
            let mut engine = Engine::new(seed, order, Arm::Full);
            engine.set_profile(thermal, hill);
            engine
        });
        Self { engines, keys, config, reads: 0, trip_reads: -1 }
    }

    /// Register one cell's donor observation (its own fingerprint) and absorb
    /// the resulting reaction into that cell.
    pub fn register_donor(&mut self, site: usize, fingerprint: u64) -> Result<SwarmReceipt, Error> {
        if site >= SWARM_CELLS {
            return Err(Error::Swarm(SwarmError::InvalidSite));
        }
        let observation = Observation {
            tick: 0,
            direction: Direction::Donor,
            site: site as u8,
            fingerprint,
            quantity: 1.0,
        };
        self.engines[site].observe(observation)?;
        let reaction = self.engines[site].measurements()?.reaction;
        self.keys[site].absorb(reaction)?;
        Ok(self.current_receipt())
    }

    /// One recipient read addressed to `site`. The addressed cell's engine
    /// processes the observation; the cell absorbs its reaction plus the
    /// quorum coupling from the other cells' heat.
    pub fn read(&mut self, site: usize, fingerprint: u64, quantity: f64, tick: u64) -> Result<SwarmReceipt, Error> {
        if site >= SWARM_CELLS {
            return Err(Error::Swarm(SwarmError::InvalidSite));
        }
        let observation = Observation {
            tick,
            direction: Direction::Recipient,
            site: site as u8,
            fingerprint,
            quantity,
        };
        self.engines[site].observe(observation)?;
        let reaction = self.engines[site].measurements()?.reaction;
        let heats = self.heats();
        let coupling = coupling_term(&heats, site, self.config)?;
        self.keys[site].absorb(reaction + coupling)?;
        self.reads += 1;
        if self.trip_reads < 0 && fired(&self.heats(), self.config) {
            self.trip_reads = self.reads;
        }
        Ok(self.current_receipt())
    }

    pub fn heats(&self) -> [f64; SWARM_CELLS] {
        let mut heats = [0.0_f64; SWARM_CELLS];
        for (i, key) in self.keys.iter().enumerate() {
            heats[i] = key.measurement().heat;
        }
        heats
    }

    pub fn ruptured_flags(&self) -> [bool; SWARM_CELLS] {
        let mut flags = [false; SWARM_CELLS];
        for (i, key) in self.keys.iter().enumerate() {
            flags[i] = key.measurement().ruptured;
        }
        flags
    }

    pub fn nonzero_bytes(&self) -> [usize; SWARM_CELLS] {
        let mut nonzero = [0_usize; SWARM_CELLS];
        for (i, key) in self.keys.iter().enumerate() {
            nonzero[i] = key.measurement().nonzero_bytes;
        }
        nonzero
    }

    pub fn reads(&self) -> i64 { self.reads }

    /// Current field measurements of one cell's engine (for control tracking).
    pub fn measurements(&self, site: usize) -> Result<nocopy_sensor_engine::Measurements, Error> {
        if site >= SWARM_CELLS {
            return Err(Error::Swarm(SwarmError::InvalidSite));
        }
        Ok(self.engines[site].measurements()?)
    }

    /// Current numerical receipt (trip_reads = first-fire read count, -1 if none).
    pub fn current_receipt(&self) -> SwarmReceipt {
        receipt(&self.heats(), &self.ruptured_flags(), self.trip_reads, self.config)
    }
}
