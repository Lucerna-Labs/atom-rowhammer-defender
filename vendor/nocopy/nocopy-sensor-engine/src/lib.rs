//! Fixed-order observation-only orchestrator for the sensor component.
//! The public result is a continuous field measurement, never a copy verdict.

pub use nocopy_sensor_principles::{
    ChaosLevel, Direction, KeyCell, KeyMeasurement, Measurements, Observation,
};
use nocopy_sensor_principles::{Error as PrincipleError, Field, Stage};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Order {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    L,
    M,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChaosMode {
    NoChaos,
    Logistic,
    Tent,
    Both,
}

impl ChaosMode {
    fn includes(self, stage: Stage) -> bool {
        match stage { Stage::LogisticChaos => matches!(self, Self::Logistic | Self::Both), Stage::TentChaos => matches!(self, Self::Tent | Self::Both), _ => true }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arm {
    Full,
    NoBase,
    BaseOnly,
    NoChaos,
    LogisticOnly,
    TentOnly,
    NoInformation,
    NoMechanics,
    NoPhysics,
    NoChemistry,
    NoCognition,
    NoConservation,
    NoCooperativeBinding,
}

impl Arm {
    pub const ALL: [Self; 11] = [
        Self::Full,
        Self::NoBase,
        Self::BaseOnly,
        Self::NoChaos,
        Self::LogisticOnly,
        Self::TentOnly,
        Self::NoInformation,
        Self::NoMechanics,
        Self::NoPhysics,
        Self::NoChemistry,
        Self::NoCognition,
    ];

    pub const INVENTORY_ARMS: [Self; 12] = [
        Self::Full,
        Self::NoBase,
        Self::BaseOnly,
        Self::NoChaos,
        Self::LogisticOnly,
        Self::TentOnly,
        Self::NoInformation,
        Self::NoMechanics,
        Self::NoPhysics,
        Self::NoChemistry,
        Self::NoCognition,
        Self::NoConservation,
    ];

    pub const COOPERATIVE_ARMS: [Self; 13] = [
        Self::Full,
        Self::NoBase,
        Self::BaseOnly,
        Self::NoChaos,
        Self::LogisticOnly,
        Self::TentOnly,
        Self::NoInformation,
        Self::NoMechanics,
        Self::NoPhysics,
        Self::NoChemistry,
        Self::NoCognition,
        Self::NoConservation,
        Self::NoCooperativeBinding,
    ];

    fn includes(self, stage: Stage) -> bool {
        match self {
            Self::Full | Self::NoBase => true,
            Self::BaseOnly => false,
            Self::NoChaos => !matches!(stage, Stage::LogisticChaos | Stage::TentChaos),
            Self::LogisticOnly => stage != Stage::TentChaos,
            Self::TentOnly => stage != Stage::LogisticChaos,
            Self::NoInformation => !matches!(stage, Stage::Information | Stage::Resonance),
            Self::NoMechanics => stage != Stage::Mechanics,
            Self::NoPhysics => stage != Stage::Physics,
            Self::NoChemistry => stage != Stage::Chemistry,
            Self::NoCognition => stage != Stage::Cognition,
            Self::NoConservation => {
                !matches!(stage, Stage::Conservation | Stage::AffinityConservation)
            }
            Self::NoCooperativeBinding => stage != Stage::CooperativeBinding,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    OutOfOrderObservation,
    Principle(PrincipleError),
}

impl From<PrincipleError> for Error {
    fn from(value: PrincipleError) -> Self {
        Self::Principle(value)
    }
}

const ORDER_A: [Stage; 7] = [Stage::Information, Stage::Mechanics, Stage::Physics, Stage::Chemistry, Stage::Cognition, Stage::LogisticChaos, Stage::TentChaos];

const ORDER_B: [Stage; 7] = [Stage::Information, Stage::Chemistry, Stage::TentChaos, Stage::Physics, Stage::Mechanics, Stage::LogisticChaos, Stage::Cognition];

const ORDER_C: [Stage; 8] = [Stage::Information, Stage::Conservation, Stage::Mechanics, Stage::Physics, Stage::Chemistry, Stage::Cognition, Stage::LogisticChaos, Stage::TentChaos];

const ORDER_D: [Stage; 8] = [Stage::Information, Stage::Chemistry, Stage::TentChaos, Stage::Conservation, Stage::Physics, Stage::Mechanics, Stage::LogisticChaos, Stage::Cognition];

const ORDER_E: [Stage; 8] = [Stage::Information, Stage::AffinityConservation, Stage::Mechanics, Stage::Physics, Stage::Chemistry, Stage::Cognition, Stage::LogisticChaos, Stage::TentChaos];

const ORDER_F: [Stage; 8] = [Stage::Information, Stage::Chemistry, Stage::TentChaos, Stage::AffinityConservation, Stage::Physics, Stage::Mechanics, Stage::LogisticChaos, Stage::Cognition];

const ORDER_G: [Stage; 9] = [Stage::Information, Stage::CooperativeBinding, Stage::AffinityConservation, Stage::Mechanics, Stage::Physics, Stage::Chemistry, Stage::Cognition, Stage::LogisticChaos, Stage::TentChaos];

const ORDER_H: [Stage; 9] = [Stage::Information, Stage::CooperativeBinding, Stage::Chemistry, Stage::TentChaos, Stage::AffinityConservation, Stage::Physics, Stage::Mechanics, Stage::LogisticChaos, Stage::Cognition];

const ORDER_I: [Stage; 9] = [Stage::Information, Stage::CooperativeBinding, Stage::AffinityConservation, Stage::Mechanics, Stage::Physics, Stage::Chemistry, Stage::Cognition, Stage::TentChaos, Stage::LogisticChaos];

const ORDER_J: [Stage; 9] = [Stage::Information, Stage::CooperativeBinding, Stage::Chemistry, Stage::AffinityConservation, Stage::Physics, Stage::Mechanics, Stage::Cognition, Stage::TentChaos, Stage::LogisticChaos];

const ORDER_L: [Stage; 9] = [Stage::Information, Stage::Resonance, Stage::AffinityConservation, Stage::Mechanics, Stage::Physics, Stage::Chemistry, Stage::Cognition, Stage::LogisticChaos, Stage::TentChaos];

const ORDER_M: [Stage; 9] = [Stage::Information, Stage::Resonance, Stage::Chemistry, Stage::TentChaos, Stage::AffinityConservation, Stage::Physics, Stage::Mechanics, Stage::LogisticChaos, Stage::Cognition];

/// The engine carries mechanical state. Order and ablation choices are fixed
/// experiment structure and never selected from observation content.
pub struct Engine {
    field: Field,
    order: Order,
    arm: Arm,
    chaos_mode: Option<ChaosMode>,
    last_tick: Option<u64>,
}

#[derive(Clone, Copy, Debug)]
pub struct DestructionReport {
    pub sensor: Measurements,
    pub key: KeyMeasurement,
    pub peak_reaction: f64,
    pub post_drain_reaction: f64,
    pub max_post_drain_reaction: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct LevelRun {
    pub order: Order,
    pub arm: Arm,
    pub level: ChaosLevel,
    pub mode: ChaosMode,
    pub destruction_present: bool,
}

struct CoupledConfig {
    order: Order,
    arm: Arm,
    destruction_present: bool,
    dissipate_each_reading: bool,
    profile: Option<(ChaosLevel, ChaosMode)>,
    thermal: f64,
    hill: bool,
}

impl Engine {
    pub fn new(seed: u64, order: Order, arm: Arm) -> Self {
        Self {
            field: Field::new(seed),
            order,
            arm,
            chaos_mode: None,
            last_tick: None,
        }
    }

    pub fn new_with_level(
        seed: u64,
        order: Order,
        arm: Arm,
        level: ChaosLevel,
        mode: ChaosMode,
    ) -> Self {
        Self {
            field: Field::new_with_level(seed, level),
            order,
            arm,
            chaos_mode: Some(mode),
            last_tick: None,
        }
    }

    pub fn observe(&mut self, observation: Observation) -> Result<(), Error> {
        if self.last_tick.is_some_and(|last| observation.tick <= last) {
            return Err(Error::OutOfOrderObservation);
        }
        if self.arm == Arm::NoBase {
            self.field.neutral_base(observation)?;
        } else {
            self.field.quantum_base(observation)?;
        }
        let stages: &[Stage] = match self.order {
            Order::A => &ORDER_A,
            Order::B => &ORDER_B,
            Order::C => &ORDER_C,
            Order::D => &ORDER_D,
            Order::E => &ORDER_E,
            Order::F => &ORDER_F,
            Order::G => &ORDER_G,
            Order::H => &ORDER_H,
            Order::I => &ORDER_I,
            Order::J => &ORDER_J,
            Order::L => &ORDER_L,
            Order::M => &ORDER_M,
        };
        for stage in stages {
            if self.arm.includes(*stage) && self.chaos_mode.is_none_or(|mode| mode.includes(*stage))
            {
                self.field.apply(*stage, observation)?;
            }
        }
        self.last_tick = Some(observation.tick);
        Ok(())
    }

    pub fn measurements(&self) -> Result<Measurements, Error> {
        self.field.measurements().map_err(Error::from)
    }

    /// Fixed experiment profile on the base: software temperature and Hill-2
    /// weighting. Never selected from observation content.
    pub fn set_profile(&mut self, thermal: f64, hill: bool) {
        self.field.set_thermal(thermal);
        self.field.set_hill(hill);
    }
}

/// The normal component route: accept observations, run the fixed full stack,
/// and return what emerged. This function has no content-dependent routing.
pub fn sense(observations: &[Observation], seed: u64) -> Result<Measurements, Error> {
    run_experiment(observations, seed, Order::A, Arm::Full)
}

/// A content-independent experiment route for declared orders and ablations.
pub fn run_experiment(
    observations: &[Observation],
    seed: u64,
    order: Order,
    arm: Arm,
) -> Result<Measurements, Error> {
    let mut engine = Engine::new(seed, order, arm);
    for observation in observations {
        engine.observe(*observation)?;
    }
    engine.measurements()
}

/// Experimentally couples the sensor field to a finite-capacity key cell.
/// `destruction_present` is a fixed ablation setting, never selected from
/// observation content. The result does not contain key bytes or a verdict.
pub fn run_destruction_experiment(
    observations: &[Observation],
    seed: u64,
    order: Order,
    arm: Arm,
    key: &mut KeyCell,
    destruction_present: bool,
) -> Result<DestructionReport, Error> {
    run_coupled(
        observations,
        seed,
        key,
        CoupledConfig {
            order,
            arm,
            destruction_present,
            dissipate_each_reading: false,
            profile: None,
            thermal: 0.0,
            hill: false,
        },
    )
}

/// Fixed software-temperature variant of the coupled destruction experiment.
/// `thermal` is the dimensionless T on the two-mode base; 0.0 disables it.
pub fn run_thermal_experiment(
    observations: &[Observation], seed: u64, order: Order, arm: Arm,
    key: &mut KeyCell, destruction_present: bool, thermal: f64,
) -> Result<DestructionReport, Error> {
    run_coupled(observations, seed, key, CoupledConfig { order, arm, destruction_present, dissipate_each_reading: false, profile: None, thermal, hill: false })
}

/// Fixed Hill-2 profile variant of the coupled destruction experiment:
/// squared-similarity information weighting with unit resonance coupling.
/// `thermal` is the dimensionless T on the two-mode base; 0.0 disables it.
pub fn run_hill_experiment(
    observations: &[Observation], seed: u64, order: Order, arm: Arm,
    key: &mut KeyCell, destruction_present: bool, thermal: f64,
) -> Result<DestructionReport, Error> {
    run_coupled(observations, seed, key, CoupledConfig { order, arm, destruction_present, dissipate_each_reading: false, profile: None, thermal, hill: true })
}

/// The same fixed composition with a dissipative sink after every reading.
pub fn run_dissipating_experiment(
    observations: &[Observation],
    seed: u64,
    order: Order,
    arm: Arm,
    key: &mut KeyCell,
    destruction_present: bool,
) -> Result<DestructionReport, Error> {
    run_coupled(
        observations,
        seed,
        key,
        CoupledConfig {
            order,
            arm,
            destruction_present,
            dissipate_each_reading: true,
            profile: None,
            thermal: 0.0,
            hill: false,
        },
    )
}

/// Fixed-level continuation study; candidate modes execute one terminal chaos.
pub fn run_dissipating_with_level(
    observations: &[Observation],
    seed: u64,
    key: &mut KeyCell,
    config: LevelRun,
) -> Result<DestructionReport, Error> {
    run_coupled(
        observations,
        seed,
        key,
        CoupledConfig {
            order: config.order,
            arm: config.arm,
            destruction_present: config.destruction_present,
            dissipate_each_reading: true,
            profile: Some((config.level, config.mode)),
            thermal: 0.0,
            hill: false,
        },
    )
}

fn run_coupled(
    observations: &[Observation],
    seed: u64,
    key: &mut KeyCell,
    config: CoupledConfig,
) -> Result<DestructionReport, Error> {
    let CoupledConfig {
        order,
        arm,
        destruction_present,
        dissipate_each_reading,
        profile,
        thermal,
        hill,
    } = config;
    let mut engine = match profile {
        Some((level, mode)) => Engine::new_with_level(seed, order, arm, level, mode),
        None => {
            let mut engine = Engine::new(seed, order, arm);
            engine.set_profile(thermal, hill);
            engine
        }
    };
    let mut sensor = engine.measurements()?;
    let mut peak_reaction = 0.0_f64;
    let mut max_post_drain_reaction = 0.0_f64;
    for observation in observations {
        engine.observe(*observation)?;
        sensor = engine.measurements()?;
        peak_reaction = peak_reaction.max(sensor.reaction);
        if destruction_present {
            key.absorb(sensor.reaction)?;
        }
        if dissipate_each_reading {
            key.dissipate_heat();
            engine.field.dissipate_energy()?;
            max_post_drain_reaction = max_post_drain_reaction.max(engine.measurements()?.reaction);
        }
    }
    Ok(DestructionReport {
        sensor,
        key: key.measurement(),
        peak_reaction,
        post_drain_reaction: engine.measurements()?.reaction,
        max_post_drain_reaction,
    })
}
