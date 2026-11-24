//! Second independent pressure material readout over the same strict upload.
//! No OR policy, attack inference, live sampling or remediation is performed.

use atom_pressure_hop_engine::{Chaos, Engine, Mask, Order, PressureHopOrchestrator};

use crate::pressure::{validate_interval, PressureError, PressureRecord, RECORDS, TICKS};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PressureHopTick {
    pub index: usize,
    pub fraction: f64,
    pub energy: f64,
    pub crossed: bool,
    pub occupation: [f64; 3],
    pub coherence: [f64; 3],
    pub entropy: f64,
    pub binding: f64,
    pub habituation: f64,
    pub position: f64,
    pub heat: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PressureHopReceipt {
    pub first_crossing: Option<usize>,
    pub peak_energy: f64,
    pub final_energy: f64,
    pub peak_fraction: f64,
    pub ticks: Vec<PressureHopTick>,
}

pub fn evaluate(records: &[PressureRecord; RECORDS]) -> Result<PressureHopReceipt, PressureError> {
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
            state.occupation[0],
            state.occupation[1],
            state.occupation[2],
            state.coherence[0],
            state.coherence[1],
            state.coherence[2],
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
        ticks.push(PressureHopTick {
            index,
            fraction,
            energy: state.energy,
            crossed: state.crossed,
            occupation: state.occupation,
            coherence: state.coherence,
            entropy: state.entropy,
            binding: state.binding,
            habituation: state.habituation,
            position: state.position,
            heat: state.heat,
        });
    }
    let final_energy = ticks.last().map_or(0.0, |tick| tick.energy);
    Ok(PressureHopReceipt {
        first_crossing,
        peak_energy,
        final_energy,
        peak_fraction,
        ticks,
    })
}
