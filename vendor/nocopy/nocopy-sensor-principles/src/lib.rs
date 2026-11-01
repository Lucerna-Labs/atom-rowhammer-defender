//! Content-independent first-principle dynamics for an observation field.
//! This crate does not hold secret bytes or produce a copy/clone verdict.

use std::f64::consts::PI;
use std::fmt;
use std::ptr;
use std::sync::atomic::{compiler_fence, Ordering};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Donor,
    Recipient,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChaosLevel {
    Low,
    Edge,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Observation {
    pub tick: u64,
    pub direction: Direction,
    pub site: u8,
    pub fingerprint: u64,
    pub quantity: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    Information,
    Conservation,
    AffinityConservation,
    CooperativeBinding,
    Mechanics,
    Physics,
    Chemistry,
    Cognition,
    LogisticChaos,
    TentChaos,
    Resonance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidObservation,
    InvalidKey,
    InvalidEnergy,
    NonFiniteState,
    QuantumNorm,
    DiffusionConservation,
    OutOfBounds,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}

impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug)]
pub struct KeyMeasurement {
    pub heat: f64,
    pub ruptured: bool,
    pub key_len: usize,
    pub nonzero_bytes: usize,
}

/// A finite-capacity material cell that owns one mutable byte allocation.
/// Fracture clears that allocation. Other copies are outside this cell.
pub struct KeyCell {
    bytes: Vec<u8>,
    heat: f64,
    ruptured: bool,
}

impl KeyCell {
    pub fn new(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.is_empty() { return Err(Error::InvalidKey); }
        Ok(Self { bytes, heat: 0.0, ruptured: false })
    }

    /// First-order relaxation and finite-capacity fracture. Reaction energy
    /// has no copy label; any sufficiently energetic cause can rupture a cell.
    pub fn absorb(&mut self, reaction: f64) -> Result<(), Error> {
        if !reaction.is_finite() || reaction < 0.0 {
            self.fracture();
            return Err(Error::InvalidEnergy);
        }
        self.heat = 0.25 * self.heat + reaction;
        if !self.heat.is_finite() {
            self.fracture();
            return Err(Error::InvalidEnergy);
        }
        if self.heat >= 0.25 {
            self.fracture();
        }
        Ok(())
    }

    pub fn dissipate_heat(&mut self) { self.heat = 0.0; }

    pub fn measurement(&self) -> KeyMeasurement {
        KeyMeasurement {
            heat: self.heat,
            ruptured: self.ruptured,
            key_len: self.bytes.len(),
            nonzero_bytes: self.bytes.iter().filter(|byte| **byte != 0).count(),
        }
    }

    pub fn usable(&self) -> bool { !self.ruptured }

    fn fracture(&mut self) {
        for byte in &mut self.bytes {
            // SAFETY: each pointer comes from an exclusive mutable reference
            // to a valid byte in this allocation.
            unsafe { ptr::write_volatile(byte, 0) };
        }
        compiler_fence(Ordering::SeqCst);
        self.ruptured = true;
    }
}

impl Drop for KeyCell {
    fn drop(&mut self) { self.fracture(); }
}

#[derive(Clone, Copy, Debug)]
struct Memory {
    fingerprint: u64,
    strength: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Measurements {
    pub reaction: f64,
    pub quantum_norm_error: f64,
    pub diffusion_mass_error: f64,
    pub information_drive: f64,
    pub chemical_flux: f64,
    pub field_energy: f64,
    pub amplitude_weight: f64,
    pub inventory_gap: f64,
}

/// The state fields are coupled by shared drive and spatial activity. No field
/// identifies whether an observation was a copy, clone, or benign transfer.
pub struct Field {
    amplitude: [f64; 2],
    born_weight: f64,
    memory: [Option<Memory>; 4],
    memory_cursor: usize,
    information_drive: f64,
    last_similarity: f64,
    last_pulse: f64,
    cooperative_factor: f64,
    donor_inventory: f64,
    recipient_inventory: f64,
    position: f64,
    momentum: f64,
    potential: [f64; 4],
    diffusion_error: f64,
    reactants: [f64; 2],
    catalyst: f64,
    chemical_flux: f64,
    activation: [f64; 4],
    logistic: f64,
    logistic_rate: f64,
    thermal: f64,
    hill: bool,
    tent: f64,
    drive: f64,
}

impl Field {
    pub fn new(seed: u64) -> Self {
        let unit = |bits: u64| ((bits >> 11) as f64 + 0.5) / ((1_u64 << 53) as f64);
        Self {
            amplitude: [0.8, 0.6],
            born_weight: 0.64,
            memory: [None; 4],
            memory_cursor: 0,
            information_drive: 0.0,
            last_similarity: 0.0,
            last_pulse: 0.0,
            cooperative_factor: 1.0,
            donor_inventory: 0.0,
            recipient_inventory: 0.0,
            position: 0.0,
            momentum: 0.0,
            potential: [0.0; 4],
            diffusion_error: 0.0,
            reactants: [0.0; 2],
            catalyst: 0.7,
            chemical_flux: 0.0,
            activation: [0.0; 4],
            logistic: 0.05 + 0.9 * unit(seed ^ 0x9e37_79b9_7f4a_7c15),
            logistic_rate: 4.0,
            thermal: 0.0,
            hill: false,
            tent: 0.05 + 0.9 * unit(seed.rotate_left(23)),
            drive: 0.0,
        }
    }

    pub fn new_with_level(seed: u64, level: ChaosLevel) -> Self {
        let mut field = Self::new(seed);
        field.logistic_rate = match level {
            ChaosLevel::Low => 3.55,
            ChaosLevel::Edge => 3.569_945_672,
            ChaosLevel::High => 4.0,
        };
        for _ in 0..256 {
            field.logistic = field.logistic_rate * field.logistic * (1.0 - field.logistic);
        }
        field
    }

    pub fn validate_observation(o: Observation) -> Result<(), Error> {
        if o.quantity.is_finite() && (0.0..=1.0).contains(&o.quantity) { Ok(()) } else { Err(Error::InvalidObservation) }
    }

    /// A two-mode orthogonal rotation: the Born weight feeds later classical
    /// dynamics. This is calculated on a classical CPU, not physical quantum
    /// measurement or randomness.
    pub fn quantum_base(&mut self, observation: Observation) -> Result<(), Error> {
        Self::validate_observation(observation)?;
        let phase = (observation.fingerprint as u16 as f64 / 65_535.0) * PI / 4.0;
        let angle = 0.13 + phase + (observation.tick % 7) as f64 * 0.01;
        let (sin, cos) = angle.sin_cos();
        let [a, b] = self.amplitude;
        self.amplitude = [cos * a - sin * b, sin * a + cos * b];
        self.born_weight = self.amplitude[0] * self.amplitude[0];
        if self.thermal > 0.0 {
            let p0 = 1.0 / (1.0 + (-1.0 / self.thermal).exp());
            let mixed = 0.9 * self.born_weight + 0.1 * p0;
            self.amplitude = [mixed.sqrt(), (1.0 - mixed).sqrt()];
            self.born_weight = mixed;
        }
        self.check()
    }

    /// Classical replacement control for the quantum-derived base.
    pub fn neutral_base(&mut self, o: Observation) -> Result<(), Error> {
        Self::validate_observation(o)?; self.born_weight = 0.5; self.check()
    }

    /// Dimensionless software temperature on the two-mode base; 0.0 disables.
    pub fn set_thermal(&mut self, thermal: f64) {
        self.thermal = thermal;
    }

    /// Fixed experiment profile: squared-similarity information weighting and
    /// unit resonance coupling. Off by default; never content-selected.
    pub fn set_hill(&mut self, hill: bool) {
        self.hill = hill;
    }

    pub fn apply(&mut self, stage: Stage, observation: Observation) -> Result<(), Error> {
        Self::validate_observation(observation)?;
        match stage {
            Stage::Information => self.information(observation),
            Stage::Conservation => self.conservation(observation),
            Stage::AffinityConservation => self.affinity_conservation(observation),
            Stage::CooperativeBinding => self.cooperative_binding(),
            Stage::Mechanics => self.mechanics(),
            Stage::Physics => self.physics(observation),
            Stage::Chemistry => self.chemistry(observation),
            Stage::Cognition => self.cognition(observation),
            Stage::LogisticChaos => self.logistic(),
            Stage::TentChaos => self.tent(),
            Stage::Resonance => self.resonance(observation),
        }
        self.check()
    }

    fn information(&mut self, observation: Observation) {
        for memory in self.memory.iter_mut().flatten() {
            memory.strength *= 0.9;
        }
        match observation.direction {
            Direction::Donor => {
                self.last_similarity = 0.0;
                self.memory[self.memory_cursor] = Some(Memory {
                    fingerprint: observation.fingerprint,
                    strength: observation.quantity,
                });
                self.memory_cursor = (self.memory_cursor + 1) % self.memory.len();
                self.drive += 0.02 * observation.quantity;
            }
            Direction::Recipient => {
                let similarity = self
                    .memory
                    .iter()
                    .flatten()
                    .map(|memory| {
                        let distance = (memory.fingerprint ^ observation.fingerprint).count_ones();
                        (1.0 - distance as f64 / 64.0) * memory.strength
                    })
                    .fold(0.0_f64, f64::max);
                let pulse = similarity * observation.quantity;
                self.last_similarity = similarity;
                self.information_drive += pulse;
                let weight = if self.hill { similarity * similarity } else { 1.0 };
                self.drive += pulse * (0.25 + self.born_weight) * weight;
            }
        }
    }

    fn conservation(&mut self, observation: Observation) {
        match observation.direction {
            Direction::Donor => self.donor_inventory += observation.quantity,
            Direction::Recipient => self.recipient_inventory += observation.quantity,
        }
        let gap = self.recipient_inventory - self.donor_inventory;
        // Resource imbalance supplies quadratic potential energy to the
        // shared field. It has no copy label, verdict, or rejection path.
        self.drive += 0.25 * gap * gap;
    }

    fn affinity_conservation(&mut self, observation: Observation) {
        match observation.direction {
            Direction::Donor => self.donor_inventory += observation.quantity,
            Direction::Recipient => self.recipient_inventory += observation.quantity,
        }
        let gap = self.recipient_inventory - self.donor_inventory;
        let excess_affinity = (2.0 * self.last_similarity - 1.0).max(0.0);
        self.drive += 0.25 * gap * gap * excess_affinity;
    }

    /// Similarity-gated coincidence with the previous recipient pulse.
    fn resonance(&mut self, observation: Observation) {
        let excess = (2.0 * self.last_similarity - 1.0).max(0.0);
        let coefficient = if self.hill { 1.0 } else { 0.25 };
        self.drive += coefficient * excess * self.last_pulse;
        self.last_pulse = if observation.direction == Direction::Recipient { self.last_similarity * observation.quantity } else { 0.0 };
    }

    fn cooperative_binding(&mut self) {
        let squared = self.last_similarity * self.last_similarity;
        self.cooperative_factor = 1.0 + squared / (0.25 + squared);
    }

    fn mechanics(&mut self) {
        // Symplectic Euler map for a spring with a field-driven impulse.
        self.momentum += -0.125 * self.position + 0.25 * self.drive;
        self.position += 0.25 * self.momentum;
        self.drive += 0.05 * self.momentum.abs();
    }

    fn physics(&mut self, observation: Observation) {
        let site = observation.site as usize % self.potential.len();
        self.potential[site] += 0.1 * observation.quantity * (0.1 + self.drive.abs());
        let before = self.potential.iter().sum::<f64>();
        let old = self.potential;
        for index in 0..4 {
            let left = old[(index + 3) % 4];
            let right = old[(index + 1) % 4];
            self.potential[index] = old[index] + 0.1 * (left + right - 2.0 * old[index]);
        }
        let after = self.potential.iter().sum::<f64>();
        self.diffusion_error = self.diffusion_error.max((after - before).abs());
        let max = self
            .potential
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let min = self.potential.iter().copied().fold(f64::INFINITY, f64::min);
        self.drive += 0.1 * (max - min);
    }

    fn chemistry(&mut self, observation: Observation) {
        let index = usize::from(observation.direction == Direction::Recipient);
        self.reactants[index] = (self.reactants[index] + 0.4 * observation.quantity).min(1.0);
        let extent = (0.3 + 0.2 * self.drive.abs().min(1.0))
            * self.catalyst
            * self.cooperative_factor
            * self.reactants[0]
            * self.reactants[1];
        let extent = extent.min(self.reactants[0]).min(self.reactants[1]);
        self.reactants[0] -= extent;
        self.reactants[1] -= extent;
        self.chemical_flux += extent;
        self.drive += 0.2 * extent;
    }

    fn cognition(&mut self, observation: Observation) {
        let site = observation.site as usize % self.activation.len();
        self.activation[site] =
            (self.activation[site] + 0.1 * observation.quantity * self.drive.abs()).min(1.0);
        let old = self.activation;
        for index in 0..4 {
            let competing = old.iter().sum::<f64>() - old[index];
            self.activation[index] = (old[index] - 0.08 * competing).max(0.0);
        }
        let max = self.activation.iter().copied().fold(0.0_f64, f64::max);
        let min = self.activation.iter().copied().fold(1.0_f64, f64::min);
        self.drive += 0.1 * (max - min);
    }

    fn logistic(&mut self) {
        self.logistic = self.logistic_rate * self.logistic * (1.0 - self.logistic);
        self.drive *= 0.75 + 0.5 * self.logistic;
    }

    fn tent(&mut self) {
        self.tent = if self.tent <= 0.5 { 2.0 * self.tent } else { 2.0 * (1.0 - self.tent) };
        self.drive *= 0.75 + 0.5 * self.tent;
    }

    pub fn measurements(&self) -> Result<Measurements, Error> {
        self.check()?;
        let field_energy = self.activation.iter().map(|value| value * value).sum::<f64>();
        let reaction = self.chemical_flux * (1.0 + self.drive.abs()) + 0.01 * field_energy;
        if !reaction.is_finite() {
            return Err(Error::NonFiniteState);
        }
        Ok(Measurements {
            reaction,
            quantum_norm_error: self.quantum_norm_error(),
            diffusion_mass_error: self.diffusion_error,
            information_drive: self.information_drive,
            chemical_flux: self.chemical_flux,
            field_energy,
            amplitude_weight: self.born_weight,
            inventory_gap: self.recipient_inventory - self.donor_inventory,
        })
    }

    /// Transfer transient field energy to an external sink after one reading.
    /// Observation memory, inventory, reactants, and base state remain.
    pub fn dissipate_energy(&mut self) -> Result<(), Error> {
        self.drive = 0.0;
        self.chemical_flux = 0.0;
        self.position = 0.0;
        self.momentum = 0.0;
        self.potential = [0.0; 4];
        self.activation = [0.0; 4];
        self.cooperative_factor = 1.0;
        self.check()
    }

    fn quantum_norm_error(&self) -> f64 { (self.amplitude[0] * self.amplitude[0] + self.amplitude[1] * self.amplitude[1] - 1.0).abs() }

    fn check(&self) -> Result<(), Error> {
        let scalars = [
            self.born_weight,
            self.information_drive,
            self.last_similarity,
            self.last_pulse,
            self.cooperative_factor,
            self.donor_inventory,
            self.recipient_inventory,
            self.position,
            self.momentum,
            self.catalyst,
            self.chemical_flux,
            self.logistic,
            self.logistic_rate,
            self.tent,
            self.drive,
        ];
        if !scalars
            .iter()
            .chain(self.potential.iter())
            .chain(self.reactants.iter())
            .chain(self.activation.iter())
            .all(|value| value.is_finite())
        {
            return Err(Error::NonFiniteState);
        }
        if self.quantum_norm_error() > 1e-10 {
            return Err(Error::QuantumNorm);
        }
        if self.diffusion_error > 1e-10 {
            return Err(Error::DiffusionConservation);
        }
        if self
            .reactants
            .iter()
            .chain(self.activation.iter())
            .any(|value| !(0.0..=1.0).contains(value))
            || !(0.0..=1.0).contains(&self.logistic)
            || !(0.0..=1.0).contains(&self.tent)
            || !(0.0..=1.0).contains(&self.last_similarity)
            || !(1.0..=2.0).contains(&self.cooperative_factor)
            || self.donor_inventory < 0.0
            || self.recipient_inventory < 0.0
        {
            return Err(Error::OutOfBounds);
        }
        Ok(())
    }
}
