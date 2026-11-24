//! Full-range observations independent of the simulation's stop receipt.
use crate::{immutable_feed, AccessSample, HeatMilestones, RegionIdentity, RowGeometry};
use atom_row_trace_engine::Engine;
use imperium_nocopy_swarm_live::SwarmLiveFeed;

pub use atom_row_trace_engine::{Reading, HEAT_CAPACITY, HORIZON_NS};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ModelReport {
    pub reading: Reading,
    pub timing_rupture: Option<u64>,
    pub heat_rupture: Option<u64>,
}

pub struct TraceModels {
    geometry: RowGeometry,
    engine: Engine<RegionIdentity>,
    timing_swarm: SwarmLiveFeed,
    heat_swarm: SwarmLiveFeed,
    heat_milestones: HeatMilestones,
    timing_rupture: Option<u64>,
    heat_rupture: Option<u64>,
}

impl TraceModels {
    pub fn new(geometry: RowGeometry) -> Result<Self, &'static str> {
        if geometry.row_shift >= 64
            || geometry.row_bits == 0
            || geometry.row_bits > 64 - geometry.row_shift
        {
            return Err("invalid row geometry");
        }
        let timing_swarm = immutable_feed();
        let heat_swarm = immutable_feed();
        if !timing_swarm.is_armed() || !heat_swarm.is_armed() {
            return Err("trace model swarms could not arm");
        }
        Ok(Self {
            geometry,
            engine: Engine::default(),
            timing_swarm,
            heat_swarm,
            heat_milestones: HeatMilestones::default(),
            timing_rupture: None,
            heat_rupture: None,
        })
    }

    pub fn extend(&mut self, samples: &[AccessSample]) -> Result<(), &'static str> {
        // Reject a malformed batch before it can partially advance either model.
        samples
            .iter()
            .try_fold(self.engine.last_time_ns(), |last, sample| {
                let time = sample
                    .time_ns
                    .ok_or("timing and heat models require timestamps")?;
                if last.is_some_and(|previous| previous > time) {
                    return Err("trace timestamp moved backward");
                }
                Ok(Some(time))
            })?;
        for &sample in samples {
            let time = sample
                .time_ns
                .ok_or("timing and heat models require timestamps")?;
            let site = self.geometry.identity(sample);
            let step = self
                .engine
                .observe(time, site)
                .map_err(|_| "trace timestamp moved backward")?;
            if step.timing_repeat && self.timing_rupture.is_none() {
                let class = self.geometry.class_for_fingerprint(site.block);
                if self.timing_swarm.observe_class(class).is_some() {
                    self.timing_rupture = Some(self.engine.reading().samples);
                }
            }
            if self.heat_rupture.is_none() {
                if step.cooled && self.heat_swarm.frames() > 0 {
                    self.heat_swarm = immutable_feed();
                }
                if let Some(class) = self.heat_milestones.observe(step.heat, step.cooled) {
                    if self.heat_swarm.observe_class(class).is_some() {
                        self.heat_rupture = Some(self.engine.reading().samples);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn report(&self) -> ModelReport {
        ModelReport {
            reading: self.engine.reading(),
            timing_rupture: self.timing_rupture,
            heat_rupture: self.heat_rupture,
        }
    }
}
