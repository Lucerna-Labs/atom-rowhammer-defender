//! Standalone RAM monitor (see `RAM-MONITOR-PRECOMMIT-2026-10-31.md`).
//!
//! Pipeline: uploaded or generated mathematical observations -> row-region
//! fingerprint -> thermal recurrence composition -> immutable swarms.
//!
//! Boundary: simulation-only. The app opens no CPU, PMU, or hardware ring. It
//! never touches live keys, and every swarm holds owned copies only. The stack
//! does not identify or classify named attacks. Numerical rupture receipts are
//! simulated evidence, not physical prevention.

use std::cmp::Reverse;
use std::collections::{HashMap, VecDeque};

use imperium_nocopy_swarm_live::SwarmLiveFeed;
use sketch::SketchThermalChannel;
use thiserror::Error;

pub mod controller;
pub mod replay;
mod sketch;
pub mod trace;

pub use controller::{Command, Controller, LoopSource, Settings, Snapshot, LOG_CAP};
pub use replay::ReplaySource;
pub use trace::{parse_trace, read_trace, TraceError, TraceSource, TRACE_MAX_SAMPLES};

/// Errors. Everything fails closed: no source, no samples, no panic.
#[derive(Debug, Error)]
pub enum RamMonitorError {
    /// A source ended before the requested sample count.
    #[error("source ended after {0} samples")]
    SourceEnded(u64),
}

/// One sampled memory access. `addr` is the data (virtual) address, or the
/// `phys` optionally carries a simulator-provided physical identity; zero means
/// the uploaded trace uses `addr` as its mathematical fingerprint input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AddressKind {
    #[default]
    Unknown,
    Synthetic,
    Instruction,
    VirtualData,
    PhysicalData,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RegionIdentity {
    kind: AddressKind,
    pid: u32,
    block: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AccessSample {
    pub addr: u64,
    pub phys: u64,
    /// Pid of the sampled process when sampled (0 = unknown). Used only for
    /// the self-skip guard and diagnostics — never for detection.
    pub pid: u32,
    pub time_ns: Option<u64>,
    pub address_kind: AddressKind,
}

/// A source of sampled access addresses. Implementations yield addresses in
/// observation order; `None` means the source is exhausted (looping sources
/// never return `None`).
pub trait AccessSource {
    fn next_sample(&mut self) -> Option<AccessSample>;
}

/// Address-block geometry. A block is not a verified physical DRAM row.
/// Full shifted addresses identify concentration; the class projection is separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowGeometry {
    /// Bits below this are within-line; the fingerprint starts here.
    pub row_shift: u32,
    /// Bits used only by the legacy swarm class projection.
    pub row_bits: u32,
}

impl Default for RowGeometry {
    /// Full 64-byte block identity with an 8-bit legacy class projection.
    fn default() -> Self {
        Self {
            row_shift: 6,
            row_bits: 8,
        }
    }
}

impl RowGeometry {
    /// The row-region fingerprint of an address.
    #[must_use]
    pub fn fingerprint(&self, addr: u64) -> u64 {
        addr >> self.row_shift
    }

    /// Legacy projection for swarm classes; never used as concentration identity.
    #[must_use]
    pub fn class_for_fingerprint(&self, fingerprint: u64) -> u8 {
        let mask = if self.row_bits >= 64 {
            u64::MAX
        } else {
            (1u64 << self.row_bits) - 1
        };
        ((fingerprint & mask) % 8) as u8
    }

    /// The swarm feed class of a fingerprint (0..=7).
    #[must_use]
    pub fn class_of(&self, addr: u64) -> u8 {
        self.class_for_fingerprint(self.fingerprint(addr))
    }
}

/// Bounded sliding window over fingerprints. When the top fingerprint holds at
/// least `thresh` of the last `window` samples, the access stream is
/// concentrated on one region — the hammering shape.
#[derive(Clone, Debug)]
pub struct SlidingWindow {
    window: usize,
    thresh: usize,
    counts: HashMap<RegionIdentity, usize>,
    order: VecDeque<RegionIdentity>,
}

impl SlidingWindow {
    #[must_use]
    pub fn new(window: usize, thresh: usize) -> Self {
        assert!(window > 0, "window must be positive");
        assert!(
            thresh > 0 && thresh <= window,
            "thresh must be in 1..=window"
        );
        Self {
            window,
            thresh,
            counts: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// Push one fingerprint. Returns `Some(top_fingerprint)` when the stream is
    /// concentrated (top count >= thresh), else `None`.
    pub fn push(&mut self, fp: u64) -> Option<u64> {
        self.push_identity(RegionIdentity {
            kind: AddressKind::Synthetic,
            pid: 0,
            block: fp,
        })
        .map(|identity| identity.block)
    }

    fn push_identity(&mut self, fp: RegionIdentity) -> Option<RegionIdentity> {
        *self.counts.entry(fp).or_insert(0) += 1;
        self.order.push_back(fp);
        while self.order.len() > self.window {
            if let Some(old) = self.order.pop_front() {
                if let Some(n) = self.counts.get_mut(&old) {
                    *n -= 1;
                    if *n == 0 {
                        self.counts.remove(&old);
                    }
                }
            }
        }
        match self.top_identity() {
            Some((fp, count)) if count >= self.thresh => Some(fp),
            _ => None,
        }
    }

    /// Current number of distinct fingerprints in the window.
    #[must_use]
    pub fn distinct(&self) -> usize {
        self.counts.len()
    }

    /// Current top fingerprint and its count, for diagnostics.
    #[must_use]
    pub fn top(&self) -> Option<(u64, usize)> {
        self.top_identity()
            .map(|(identity, count)| (identity.block, count))
    }

    fn top_identity(&self) -> Option<(RegionIdentity, usize)> {
        // A tie must not depend on HashMap's per-process iteration seed: the
        // selected class is fed into the stateful swarm and affects receipts.
        self.counts
            .iter()
            .max_by_key(|(fp, n)| (**n, Reverse(**fp)))
            .map(|(fp, n)| (*fp, *n))
    }
}

/// Evidence provenance is independent of the Atom pattern receipt.
#[derive(Clone, Debug, Default)]
pub struct EvidenceCounts {
    kinds: [u64; 5],
    timed: u64,
}
impl EvidenceCounts {
    fn observe(&mut self, kind: AddressKind, timed: bool) {
        self.kinds[kind as usize] += 1;
        self.timed += u64::from(timed);
    }
    pub fn summary(&self) -> String {
        format!("unknown={} synthetic={} instruction={} virtual_data={} physical_data={} timestamped={} activation_evidence=unavailable",
            self.kinds[0], self.kinds[1], self.kinds[2], self.kinds[3], self.kinds[4], self.timed)
    }
}

/// Numerical counters of one monitor run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MonitorStats {
    /// Samples consumed from the source.
    pub samples: u64,
    /// Concentrated observations emitted to the swarm.
    pub emitted: u64,
    /// Swarm trips.
    pub trips: u64,
    /// Reads-to-trip of the first receipt, if any.
    pub trip_reads: Option<u64>,
}

/// The monitor: drives `feed` from `source` through the concentration gate.
/// Observe-only — the feed's receipt is returned, never acted on.
pub struct RamMonitor {
    pub geometry: RowGeometry,
    pub window: SlidingWindow,
    pub evidence: EvidenceCounts,
    secondary: [RecurrenceChannel; 3],
    secondary_feeds: [SwarmLiveFeed; 3],
    timed_secondary: ThermalRecurrenceChannel,
    timed_secondary_feed: SwarmLiveFeed,
    sketch_secondary: SketchThermalChannel,
    sketch_secondary_feed: SwarmLiveFeed,
    secondary_enabled: bool,
}

struct RecurrenceChannel {
    divisor: u64,
    window: usize,
    threshold: usize,
    counts: HashMap<RegionIdentity, usize>,
    order: VecDeque<RegionIdentity>,
}

struct ThermalRecurrenceChannel {
    tau_ns: f64,
    threshold: f64,
    retention_ns: u64,
    capacity: usize,
    last: HashMap<RegionIdentity, u64>,
    order: VecDeque<(RegionIdentity, u64)>,
    last_time_ns: Option<u64>,
    heat: f64,
    lead_level: u64,
}
struct ThermalObservation {
    cooled: bool,
    dangerous: bool,
    lead_class: Option<u8>,
}
impl ThermalRecurrenceChannel {
    fn new(tau_ns: f64, threshold: f64) -> Self {
        Self {
            tau_ns,
            threshold,
            retention_ns: 4_000_000,
            capacity: 65_536,
            last: HashMap::new(),
            order: VecDeque::new(),
            last_time_ns: None,
            heat: 0.0,
            lead_level: 0,
        }
    }
    fn observe(&mut self, identity: RegionIdentity, time_ns: u64) -> ThermalObservation {
        let mut cooled = false;
        if self.last_time_ns.is_some_and(|last| time_ns < last) {
            self.last.clear();
            self.order.clear();
            self.heat = 0.0;
            self.lead_level = 0;
            cooled = true;
        }
        if let Some(last) = self.last_time_ns {
            let previous_heat = self.heat;
            self.heat *= (-((time_ns - last) as f64) / self.tau_ns).exp();
            cooled |= previous_heat >= 1.0 && self.heat < 1.0;
            if cooled {
                self.lead_level = 0;
            }
        }
        if self.last.contains_key(&identity) {
            self.heat += 1.0;
        }
        self.last_time_ns = Some(time_ns);
        while let Some((old_identity, old_time)) = self.order.front().copied() {
            let expired = time_ns
                .checked_sub(old_time)
                .is_some_and(|delta| delta > self.retention_ns);
            if !expired && self.order.len() < self.capacity {
                break;
            }
            self.order.pop_front();
            if self.last.get(&old_identity).copied() == Some(old_time) {
                self.last.remove(&old_identity);
            }
        }
        self.last.insert(identity, time_ns);
        self.order.push_back((identity, time_ns));
        let heat_level = self.heat.floor().max(0.0) as u64;
        let lead_class = if heat_level >= 122 && heat_level > self.lead_level {
            self.lead_level = heat_level;
            Some(7)
        } else {
            None
        };
        ThermalObservation {
            cooled,
            dangerous: self.heat >= self.threshold,
            lead_class,
        }
    }
}
impl RecurrenceChannel {
    fn new(divisor: u64, window: usize, threshold: usize) -> Self {
        Self {
            divisor,
            window,
            threshold,
            counts: HashMap::new(),
            order: VecDeque::new(),
        }
    }
    fn observe(&mut self, mut identity: RegionIdentity) -> Option<u8> {
        identity.block /= self.divisor;
        *self.counts.entry(identity).or_insert(0) += 1;
        self.order.push_back(identity);
        while self.order.len() > self.window {
            if let Some(old) = self.order.pop_front() {
                if let Some(count) = self.counts.get_mut(&old) {
                    *count -= 1;
                    if *count == 0 {
                        self.counts.remove(&old);
                    }
                }
            }
        }
        (self.counts.get(&identity).copied().unwrap_or(0) >= self.threshold)
            .then_some((identity.block % 8) as u8)
    }
}

fn immutable_feed() -> SwarmLiveFeed {
    fn mix(mut value: u64) -> u64 {
        value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
    let mut feed = SwarmLiveFeed::new();
    let seeds = core::array::from_fn(|index| mix(index as u64 ^ 0x5eed));
    feed.set_arming_config(imperium_nocopy_swarm_live::SwarmLiveConfig::s3(
        [0, 1, 2, 3],
        seeds,
        imperium_nocopy_swarm_live::Order::L,
    ));
    for label in 0..4_u64 {
        let copy = (0..32)
            .map(|position| (mix(label * 32 + position ^ 0xc0ffee) & 0xff) as u8)
            .collect();
        let _ = feed.ingest_key_copy(label, copy);
    }
    feed
}

impl RamMonitor {
    #[must_use]
    pub fn new(geometry: RowGeometry, window: usize, thresh: usize) -> Self {
        Self {
            geometry,
            window: SlidingWindow::new(window, thresh),
            evidence: EvidenceCounts::default(),
            secondary: [
                RecurrenceChannel::new(1, 64, 5),
                RecurrenceChannel::new(2, 64, 7),
                RecurrenceChannel::new(1, 768, 11),
            ],
            secondary_feeds: std::array::from_fn(|_| immutable_feed()),
            timed_secondary: ThermalRecurrenceChannel::new(500_000.0, 128.0),
            timed_secondary_feed: immutable_feed(),
            sketch_secondary: SketchThermalChannel::new(),
            sketch_secondary_feed: immutable_feed(),
            secondary_enabled: thresh < window,
        }
    }

    /// Process one sampled address. Returns the swarm receipt if this sample
    /// produced a concentrated observation that tripped the swarm.
    pub fn observe_addr(
        &mut self,
        feed: &mut SwarmLiveFeed,
        addr: u64,
    ) -> Option<imperium_nocopy_swarm_live::SwarmReceipt> {
        self.observe_sample(
            feed,
            AccessSample {
                addr,
                address_kind: AddressKind::Synthetic,
                ..AccessSample::default()
            },
        )
    }

    /// Process one sampled access, choosing the fingerprint key.
    pub fn observe_sample(
        &mut self,
        feed: &mut SwarmLiveFeed,
        sample: AccessSample,
    ) -> Option<imperium_nocopy_swarm_live::SwarmReceipt> {
        let (key, kind, pid) = if sample.phys != 0 {
            (sample.phys, AddressKind::PhysicalData, 0)
        } else {
            (sample.addr, sample.address_kind, sample.pid)
        };
        self.evidence.observe(kind, sample.time_ns.is_some());
        let identity = RegionIdentity {
            kind,
            pid,
            block: self.geometry.fingerprint(key),
        };
        let thermal = sample
            .time_ns
            .map(|time_ns| self.timed_secondary.observe(identity, time_ns));
        let sketch = sample
            .time_ns
            .map(|time_ns| self.sketch_secondary.observe(identity, time_ns));
        let locally_hot = sample.time_ns.is_none()
            || thermal
                .as_ref()
                .is_some_and(|observation| observation.dangerous);
        let primary_top = self.window.push_identity(identity);
        let primary = match (primary_top, locally_hot) {
            (Some(top), true) => feed.observe_class(self.geometry.class_for_fingerprint(top.block)),
            _ => None,
        };
        let mut secondary = None;
        if self.secondary_enabled {
            if thermal
                .as_ref()
                .is_some_and(|observation| observation.cooled)
            {
                self.timed_secondary_feed = immutable_feed();
            }
            if sketch
                .as_ref()
                .is_some_and(|observation| observation.renewed)
            {
                self.sketch_secondary_feed = immutable_feed();
            }
            for index in 0..self.secondary.len() {
                if let Some(class) = self.secondary[index].observe(identity) {
                    if locally_hot {
                        if let Some(receipt) = self.secondary_feeds[index].observe_class(class) {
                            secondary.get_or_insert(receipt);
                        }
                    }
                }
            }
            if let Some(class) = thermal.and_then(|observation| observation.lead_class) {
                if let Some(receipt) = self.timed_secondary_feed.observe_class(class) {
                    secondary.get_or_insert(receipt);
                }
            }
            if let Some(class) = sketch.and_then(|observation| observation.lead_class) {
                if let Some(receipt) = self.sketch_secondary_feed.observe_class(class) {
                    secondary.get_or_insert(receipt);
                }
            }
        }
        primary.or(secondary)
    }

    pub fn secondary_trips(&self) -> u64 {
        self.secondary_feeds
            .iter()
            .map(|feed| feed.stats().trips)
            .sum::<u64>()
            + self.timed_secondary_feed.stats().trips
            + self.sketch_secondary_feed.stats().trips
    }
    pub fn secondary_frames(&self) -> u64 {
        self.secondary_feeds
            .iter()
            .map(SwarmLiveFeed::frames)
            .sum::<u64>()
            + self.timed_secondary_feed.frames()
            + self.sketch_secondary_feed.frames()
    }
    pub fn secondary_armed(&self) -> bool {
        self.secondary_feeds.iter().all(SwarmLiveFeed::is_armed)
            && self.timed_secondary_feed.is_armed()
            && self.sketch_secondary_feed.is_armed()
    }
    pub fn last_secondary_receipt(&self) -> Option<imperium_nocopy_swarm_live::SwarmReceipt> {
        self.secondary_feeds
            .iter()
            .filter_map(|feed| feed.receipts().last().copied())
            .chain(self.timed_secondary_feed.receipts().last().copied())
            .chain(self.sketch_secondary_feed.receipts().last().copied())
            .last()
    }

    /// Run until the source is exhausted or `max_samples` is reached.
    pub fn run<S: AccessSource + ?Sized>(
        &mut self,
        feed: &mut SwarmLiveFeed,
        source: &mut S,
        max_samples: u64,
    ) -> MonitorStats {
        let mut stats = MonitorStats::default();
        let frames_before = feed.frames();
        while stats.samples < max_samples {
            let Some(sample) = source.next_sample() else {
                break;
            };
            stats.samples += 1;
            if let Some(receipt) = self.observe_sample(feed, sample) {
                stats.trips += 1;
                if stats.trip_reads.is_none() {
                    stats.trip_reads = Some(receipt.trip_reads);
                }
            }
        }
        stats.emitted = feed.frames() - frames_before;
        stats
    }
}

#[cfg(test)]
mod evidence_tests {
    use super::*;
    #[test]
    fn distant_blocks_do_not_alias() {
        let mut m = RamMonitor::new(RowGeometry::default(), 64, 16);
        let mut f = SwarmLiveFeed::new();
        for k in 0..64 {
            m.observe_addr(&mut f, k * 16384);
        }
        assert_eq!(m.window.distinct(), 64);
        assert_eq!(m.window.top().unwrap().1, 1);
        assert_eq!(f.frames(), 0);
    }
    #[test]
    fn process_and_address_kind_are_independent_identities() {
        let mut m = RamMonitor::new(RowGeometry::default(), 64, 16);
        let mut f = SwarmLiveFeed::new();
        for pid in 1..=32 {
            m.observe_sample(
                &mut f,
                AccessSample {
                    addr: 4096,
                    pid,
                    address_kind: AddressKind::VirtualData,
                    ..Default::default()
                },
            );
            m.observe_sample(
                &mut f,
                AccessSample {
                    addr: 4096,
                    pid,
                    address_kind: AddressKind::Instruction,
                    ..Default::default()
                },
            );
        }
        assert_eq!(m.window.distinct(), 64);
        assert_eq!(f.frames(), 0);
    }
    #[test]
    fn physical_identity_can_be_shared_across_processes() {
        let mut m = RamMonitor::new(RowGeometry::default(), 64, 16);
        let mut f = SwarmLiveFeed::new();
        for pid in 1..=32 {
            m.observe_sample(
                &mut f,
                AccessSample {
                    addr: 4096,
                    phys: 8192,
                    pid,
                    address_kind: AddressKind::VirtualData,
                    ..Default::default()
                },
            );
        }
        assert_eq!(m.window.distinct(), 1);
        assert_eq!(f.frames(), 17);
    }
    #[test]
    fn thermal_recurrence_state_expires_and_stays_bounded() {
        let mut channel = ThermalRecurrenceChannel::new(500_000.0, 128.0);
        for index in 0..100_000_u64 {
            channel.observe(
                RegionIdentity {
                    kind: AddressKind::Synthetic,
                    pid: 0,
                    block: index,
                },
                index * 100,
            );
        }
        assert!(channel.last.len() <= 40_001);
        assert!(channel.order.len() <= 40_002);
    }
}
