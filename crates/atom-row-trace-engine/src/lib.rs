use atom_row_trace_principles::{retained_fraction, Residence};
pub use atom_row_trace_principles::{TimeReversal, HEAT_DECAY_NS};
use std::hash::Hash;

pub const HORIZON_NS: u64 = 50_000;
pub const HEAT_MEMORY_NS: u64 = 4_000_000;
pub const HEAT_CAPACITY: f64 = 128.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reading {
    pub samples: u64,
    pub timing_matches: u64,
    pub first_match: Option<u64>,
    pub first_heat_crossing: Option<u64>,
    pub peak_heat: f64,
    pub heat: f64,
}

pub struct Engine<K> {
    timing_sites: Residence<K>,
    heat_sites: Residence<K>,
    previous_ns: Option<u64>,
    reading: Reading,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    pub timing_repeat: bool,
    pub cooled: bool,
    pub heat: f64,
}

impl<K: Copy + Eq + Hash> Default for Engine<K> {
    fn default() -> Self {
        Self {
            timing_sites: Residence::new(HORIZON_NS),
            heat_sites: Residence::new(HEAT_MEMORY_NS),
            previous_ns: None,
            reading: Reading::default(),
        }
    }
}

impl<K: Copy + Eq + Hash> Engine<K> {
    pub fn reading(&self) -> Reading {
        self.reading
    }

    pub fn last_time_ns(&self) -> Option<u64> {
        self.previous_ns
    }

    pub fn observe(&mut self, time_ns: u64, identity: K) -> Result<Step, TimeReversal> {
        let dt = time_ns
            .checked_sub(self.previous_ns.unwrap_or(time_ns))
            .ok_or(TimeReversal)?;
        let timing = self.timing_sites.visit(time_ns, identity)?;
        let deposit = self.heat_sites.visit(time_ns, identity)?;
        let retained = self.reading.heat * retained_fraction(dt);
        let cooled = self.reading.heat >= 1.0 && retained < 1.0;
        let heat = retained + f64::from(deposit);
        self.previous_ns = Some(time_ns);
        let reading = &mut self.reading;
        reading.samples += 1;
        reading.heat = heat;
        reading.peak_heat = reading.peak_heat.max(heat);
        if timing {
            reading.timing_matches += 1;
            reading.first_match.get_or_insert(reading.samples);
        }
        if heat >= HEAT_CAPACITY {
            reading.first_heat_crossing.get_or_insert(reading.samples);
        }
        Ok(Step {
            timing_repeat: timing,
            cooled,
            heat,
        })
    }
}
