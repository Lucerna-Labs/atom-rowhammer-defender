//! Live wiring layer for the no-copy quorum swarm.
//!
//! [`SwarmLiveFeed`] connects the real traffic tap to the swarm: every observed
//! wire frame drives [`VpnObserver::observe_frame`] (the redacted length-class
//! tap), and when the feed is armed with four owned key **copies**, the quantized
//! [`FrameClass`] drives [`SwarmTripwire::observe_frame`]. A trip returns a
//! numerical [`SwarmReceipt`] — numbers only, never a verdict.
//!
//! Boundary: this crate is observe-only. It holds no bus handle, never touches
//! payloads, routes, counters, or live keys, has no dependency on
//! `imperium-net-mesh`, and has no method that accepts a live key — the caller
//! passes copies in via [`SwarmLiveFeed::adopt_key_copies`]. The tap never blocks
//! or alters the data path: all state is bounded and every result is discardable.
//! Not a Rowhammer defense; does not protect RAM or any other memory.

use std::collections::{BTreeMap, VecDeque};

use imperium_nocopy_swarm::{Feed, SwarmTripwire, SwarmTripwireConfig};
use imperium_nocopy_vpn_observer::{FrameClass, VpnObserver};
use thiserror::Error;

pub use imperium_nocopy_swarm::{Order, SwarmReceipt};
pub use nocopy_swarm_principles::SWARM_CELLS;

/// Bounded receipt ring: only the most recent receipts are retained.
pub const RECEIPT_RING: usize = 64;

/// Bounded pending-copy map: at most this many distinct session labels are held
/// before arming; a new label beyond the cap evicts the smallest label.
pub const PENDING_CAP: usize = 64;

/// Errors in live-feed arming. Malformed input is rejected, never corrected.
#[derive(Debug, Error)]
pub enum SwarmLiveError {
    /// The swarm rejected the key copies or config.
    #[error("swarm rejected the key copies: {0}")]
    Swarm(String),
}

/// Arming configuration for the live feed. Never selected from observation
/// content; fixed by the caller before any traffic is observed.
#[derive(Clone, Copy, Debug)]
pub struct SwarmLiveConfig {
    /// Feed-space label of each guarded key copy (frame class 0..=7). Distinct.
    pub homes: [u8; SWARM_CELLS],
    /// Per-cell engine seeds.
    pub seeds: [u64; SWARM_CELLS],
    /// Fixed observation order.
    pub order: Order,
    /// Software temperature of the profile (classical simulation parameter).
    pub thermal: f64,
    /// Hill profile on/off.
    pub hill: bool,
    /// Quorum coupling strength Q.
    pub coupling: f64,
    /// Warm threshold theta for the quorum receipt.
    pub warm_threshold: f64,
    /// Quorum size K.
    pub quorum: usize,
}

impl SwarmLiveConfig {
    /// The measured S3 live profile: Q=0.30, theta=0.15, K=4, T=0.0, hill on.
    #[must_use]
    pub fn s3(homes: [u8; SWARM_CELLS], seeds: [u64; SWARM_CELLS], order: Order) -> Self {
        Self {
            homes,
            seeds,
            order,
            thermal: 0.0,
            hill: true,
            coupling: 0.30,
            warm_threshold: 0.15,
            quorum: 4,
        }
    }
}

/// Numerical counters of the live feed. No verdicts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwarmLiveStats {
    /// Wire frames observed since creation (saturating).
    pub frames: u64,
    /// Receipts emitted since creation (saturating).
    pub trips: u64,
    /// Whether the swarm is armed with key copies.
    pub armed: bool,
    /// Key copies ingested since creation (saturating).
    pub key_copies: u64,
    /// How many times the swarm was (re)armed from ingested copies (saturating).
    pub arm_count: u64,
}

/// Result of one [`SwarmLiveFeed::ingest_key_copy`] call. Numerical only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyCopyIngest {
    /// Total key copies ingested since creation (saturating).
    pub key_copies: u64,
    /// Distinct session labels currently held.
    pub distinct_labels: usize,
    /// Whether the swarm is armed after this ingest.
    pub armed: bool,
    /// How many times the swarm was (re)armed so far (saturating).
    pub arm_count: u64,
}

/// The live feed: one observer tap plus an optional armed swarm.
pub struct SwarmLiveFeed {
    observer: VpnObserver,
    swarm: Option<SwarmTripwire>,
    receipts: VecDeque<SwarmReceipt>,
    frames: u64,
    trips: u64,
    /// Ingested key copies by session label, bounded to [`PENDING_CAP`].
    pending: BTreeMap<u64, Vec<u8>>,
    /// Caller-fixed arming config; without it, copies are only counted.
    arming: Option<SwarmLiveConfig>,
    key_copies: u64,
    arm_count: u64,
}

impl SwarmLiveFeed {
    /// An unarmed feed: records every frame, drives no swarm.
    #[must_use]
    pub fn new() -> Self {
        Self {
            observer: VpnObserver::new(),
            swarm: None,
            receipts: VecDeque::new(),
            frames: 0,
            trips: 0,
            pending: BTreeMap::new(),
            arming: None,
            key_copies: 0,
            arm_count: 0,
        }
    }

    /// Fix the arming config before any traffic. Without it, ingested copies are
    /// counted but the swarm is never armed. Never selected from observation
    /// content — fixed by the caller.
    pub fn set_arming_config(&mut self, config: SwarmLiveConfig) {
        self.arming = Some(config);
    }

    /// Ingest one owned key copy under its session `label` (e.g. from a mesh
    /// `KeyCopyEvent`). When an arming config is set and at least
    /// [`SWARM_CELLS`] distinct labels are held, the swarm is armed with the
    /// first four copies in label order; re-ingesting an existing label (rekey)
    /// replaces its copy and re-arms. A new label beyond four while armed is
    /// ignored. Returns the numerical ingest result.
    pub fn ingest_key_copy(&mut self, label: u64, copy: Vec<u8>) -> KeyCopyIngest {
        self.key_copies = self.key_copies.saturating_add(1);
        let replaced = self.pending.insert(label, copy).is_some();
        if self.pending.len() > PENDING_CAP {
            if let Some(first) = self.pending.keys().next().copied() {
                self.pending.remove(&first);
            }
        }
        if let Some(config) = self.arming {
            if self.pending.len() >= SWARM_CELLS && (!self.is_armed() || replaced) {
                let mut values = self.pending.values();
                let copies: [Vec<u8>; SWARM_CELLS] =
                    core::array::from_fn(|_| values.next().cloned().unwrap_or_default());
                if self.adopt_key_copies(copies, config).is_ok() {
                    self.arm_count = self.arm_count.saturating_add(1);
                }
            }
        }
        KeyCopyIngest {
            key_copies: self.key_copies,
            distinct_labels: self.pending.len(),
            armed: self.is_armed(),
            arm_count: self.arm_count,
        }
    }

    /// Arm the swarm with four owned **copies** of session keys. The caller keeps
    /// the live keys; this crate only ever sees the copies it now owns.
    pub fn adopt_key_copies(
        &mut self,
        key_copies: [Vec<u8>; SWARM_CELLS],
        config: SwarmLiveConfig,
    ) -> Result<(), SwarmLiveError> {
        let tripwire = SwarmTripwire::adopt_key_copies(
            Feed::Frames,
            key_copies,
            SwarmTripwireConfig {
                homes: config.homes,
                seeds: config.seeds,
                order: config.order,
                thermal: config.thermal,
                hill: config.hill,
                placement: 0,
                coupling: config.coupling,
                warm_threshold: config.warm_threshold,
                quorum: config.quorum,
            },
        )
        .map_err(|error| SwarmLiveError::Swarm(format!("{error:?}")))?;
        self.swarm = Some(tripwire);
        Ok(())
    }

    /// Observe one wire frame of `wire_len` bytes. Always records the redacted
    /// class in the observer; when armed, also drives the swarm. Returns the
    /// numerical receipt if this frame tripped the swarm. Never fails into the
    /// data path: engine errors (e.g. already tripped) are swallowed to `None`.
    pub fn observe_wire_frame(&mut self, wire_len: usize) -> Option<SwarmReceipt> {
        self.observer.observe_frame(wire_len);
        self.drive(FrameClass::quantize(wire_len))
    }

    /// Observe one explicit frame class (e.g. from a non-wire tap such as a RAM
    /// row-concentration monitor). Same semantics as [`Self::observe_wire_frame`]
    /// with the class given directly instead of quantized from a length.
    /// Returns the numerical receipt if this observation tripped the swarm.
    pub fn observe_class(&mut self, class: u8) -> Option<SwarmReceipt> {
        if class > 7 {
            return None;
        }
        self.drive(FrameClass(class))
    }

    fn drive(&mut self, class: FrameClass) -> Option<SwarmReceipt> {
        self.frames = self.frames.saturating_add(1);
        let swarm = self.swarm.as_mut()?;
        if swarm.is_tripped() {
            return None;
        }
        match swarm.observe_frame(class) {
            Ok(Some(receipt)) => {
                self.trips = self.trips.saturating_add(1);
                if self.receipts.len() == RECEIPT_RING {
                    self.receipts.pop_front();
                }
                self.receipts.push_back(receipt);
                Some(receipt)
            }
            Ok(None) | Err(_) => None,
        }
    }

    /// Whether the swarm is armed with key copies.
    #[must_use]
    pub fn is_armed(&self) -> bool {
        self.swarm.is_some()
    }

    /// Wire frames observed since creation (saturating).
    #[must_use]
    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// Receipts emitted since creation (saturating).
    #[must_use]
    pub fn trips(&self) -> u64 {
        self.trips
    }

    /// Frames recorded by the observer tap (saturating).
    #[must_use]
    pub fn observer_frames(&self) -> u64 {
        self.observer.frames()
    }

    /// The retained receipts, oldest first (at most [`RECEIPT_RING`]).
    #[must_use]
    pub fn receipts(&self) -> Vec<SwarmReceipt> {
        self.receipts.iter().copied().collect()
    }

    /// Current per-cell heat of the armed swarm, if armed. Numerical only.
    #[must_use]
    pub fn heats(&self) -> Option<[f64; SWARM_CELLS]> {
        self.swarm
            .as_ref()
            .map(|swarm| swarm.key_measurements().map(|m| m.heat))
    }

    /// Numerical counters of the feed.
    #[must_use]
    pub fn stats(&self) -> SwarmLiveStats {
        SwarmLiveStats {
            frames: self.frames,
            trips: self.trips,
            armed: self.is_armed(),
            key_copies: self.key_copies,
            arm_count: self.arm_count,
        }
    }
}

impl Default for SwarmLiveFeed {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn splitmix(mut value: u64) -> u64 {
        value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    // Test-only synthetic bytes, same construction as the swarm eval.
    fn synthetic_key(base: usize, index: usize) -> Vec<u8> {
        match base {
            0 => (0..32)
                .map(|position| 1 + (splitmix((index * 32 + position) as u64) % 255) as u8)
                .collect(),
            1 => {
                let source = format!("synthetic-handshake-frame-{index:02}").into_bytes();
                (0..32)
                    .map(|position| source[position % source.len()])
                    .collect()
            }
            _ => (0..32)
                .map(|position| 1 + (splitmix((position * 17 + index * 13) as u64) % 255) as u8)
                .collect(),
        }
    }

    // Fixed harness config: base 0, index 0, placement 0, homes [0,1,2,3].
    fn armed_feed(order: Order) -> SwarmLiveFeed {
        let copies: [Vec<u8>; SWARM_CELLS] =
            core::array::from_fn(|j| synthetic_key(0, j % 16));
        let seeds: [u64; SWARM_CELLS] =
            core::array::from_fn(|j| splitmix(((j % 16) * 31) as u64));
        let mut feed = SwarmLiveFeed::new();
        feed.adopt_key_copies(copies, SwarmLiveConfig::s3([0, 1, 2, 3], seeds, order))
            .expect("arm");
        feed
    }

    // wire_len for class c: smallest length in that class per the quantize table.
    fn len_of(class: u8) -> usize {
        [1, 65, 129, 257, 513, 1025, 1201, 1473][class as usize]
    }

    #[test]
    fn l1_quantize_boundaries() {
        let cases = [
            (0usize, 0u8),
            (1, 0),
            (64, 0),
            (65, 1),
            (128, 1),
            (256, 2),
            (512, 3),
            (1024, 4),
            (1200, 5),
            (1472, 6),
            (1473, 7),
            (65535, 7),
        ];
        for (len, class) in cases {
            assert_eq!(FrameClass::quantize(len), FrameClass(class), "len {len}");
        }
    }

    #[test]
    fn l2_flood_trips_within_two_reads() {
        let mut feed = armed_feed(Order::L);
        let mut receipt = None;
        for _ in 0..16 {
            if let Some(r) = feed.observe_wire_frame(len_of(0)) {
                receipt = Some(r);
                break;
            }
        }
        let receipt = receipt.expect("flood must trip");
        assert_eq!(receipt.trip_reads, 2, "replica predicted trip=2");
        assert!((receipt.heats[0] - 0.385444).abs() < 1e-6, "heats {:?}", receipt.heats);
        assert_eq!(feed.trips(), 1);
    }

    #[test]
    fn l3_dist_flood_trips_within_eight() {
        let mut feed = armed_feed(Order::L);
        let mut receipt = None;
        for k in 0..16 {
            if let Some(r) = feed.observe_wire_frame(len_of((k % 4) as u8)) {
                receipt = Some(r);
                break;
            }
        }
        let receipt = receipt.expect("dist-flood must trip");
        assert!(receipt.trip_reads <= 8, "trip_reads {}", receipt.trip_reads);
    }

    #[test]
    fn l4_dist_scan_trips_within_twelve() {
        let mut feed = armed_feed(Order::M);
        let mut receipt = None;
        for k in 0..16 {
            if let Some(r) = feed.observe_wire_frame(len_of((4 + k % 4) as u8)) {
                receipt = Some(r);
                break;
            }
        }
        let receipt = receipt.expect("dist-scan must trip");
        assert!(receipt.trip_reads <= 12, "trip_reads {}", receipt.trip_reads);
    }

    #[test]
    fn l5_benign_does_not_trip() {
        let mut feed = armed_feed(Order::L);
        for j in 0..4u8 {
            assert!(feed.observe_wire_frame(len_of(j)).is_none());
        }
        assert_eq!(feed.trips(), 0);
    }

    #[test]
    fn l6_move_does_not_trip() {
        let mut feed = armed_feed(Order::L);
        for _ in 0..4 {
            assert!(feed.observe_wire_frame(len_of(7)).is_none());
        }
        assert_eq!(feed.trips(), 0);
    }

    #[test]
    fn l7_mixed_long_trips_within_eight() {
        let mut feed = armed_feed(Order::L);
        let mut receipt = None;
        for k in 0..32 {
            if let Some(r) = feed.observe_wire_frame(len_of((k % 8) as u8)) {
                receipt = Some(r);
                break;
            }
        }
        let receipt = receipt.expect("mixed-long must trip");
        assert!(receipt.trip_reads <= 8, "trip_reads {}", receipt.trip_reads);
    }

    #[test]
    fn l8_unarmed_counts_frames_without_trips() {
        let mut feed = SwarmLiveFeed::new();
        assert!(!feed.is_armed());
        for k in 0..1000usize {
            assert!(feed.observe_wire_frame(64 * (k % 8) + 1).is_none());
        }
        assert_eq!(feed.frames(), 1000);
        assert_eq!(feed.observer_frames(), 1000);
        assert_eq!(feed.trips(), 0);
        assert!(feed.receipts().is_empty());
    }

    #[test]
    fn l9_post_trip_feed_stays_bounded_and_silent() {
        let mut feed = armed_feed(Order::L);
        for _ in 0..16 {
            let _ = feed.observe_wire_frame(len_of(0));
        }
        assert_eq!(feed.trips(), 1);
        for k in 0..1000usize {
            assert!(feed.observe_wire_frame(64 * (k % 8) + 1).is_none());
        }
        assert_eq!(feed.frames(), 1016);
        assert_eq!(feed.trips(), 1);
        assert_eq!(feed.receipts().len(), 1);
        assert!(feed.receipts().len() <= RECEIPT_RING);
    }

    // Deterministic pseudo-random 32-byte copies, standing in for real
    // handshake session keys (test-only; no live key is read or touched).
    fn random_copy(tag: u64) -> Vec<u8> {
        (0..32)
            .map(|p| 1 + (splitmix((tag * 32 + p as u64) ^ 0xA5A5) % 255) as u8)
            .collect()
    }

    fn armed_from_ingest(order: Order) -> SwarmLiveFeed {
        let seeds: [u64; SWARM_CELLS] = core::array::from_fn(|j| splitmix(j as u64 ^ 0x5EED));
        let mut feed = SwarmLiveFeed::new();
        feed.set_arming_config(SwarmLiveConfig::s3([0, 1, 2, 3], seeds, order));
        for j in 0..SWARM_CELLS {
            feed.ingest_key_copy(j as u64, random_copy(j as u64 + 1));
        }
        feed
    }

    #[test]
    fn a2_ingest_arms_at_four_copies_and_rearms_on_rekey() {
        let mut feed = SwarmLiveFeed::new();
        feed.set_arming_config(SwarmLiveConfig::s3([0, 1, 2, 3], [1, 2, 3, 4], Order::L));
        let r = feed.ingest_key_copy(10, vec![1u8; 32]);
        assert!(!r.armed && r.key_copies == 1 && r.distinct_labels == 1 && r.arm_count == 0);
        feed.ingest_key_copy(20, vec![2u8; 32]);
        feed.ingest_key_copy(30, vec![3u8; 32]);
        let r = feed.ingest_key_copy(40, vec![4u8; 32]);
        assert!(r.armed, "fourth distinct label must arm");
        assert_eq!(r.arm_count, 1);
        assert_eq!(r.distinct_labels, 4);
        // A fifth distinct label while armed is ignored (first four kept).
        let r = feed.ingest_key_copy(50, vec![5u8; 32]);
        assert!(r.armed && r.arm_count == 1 && r.distinct_labels == 5);
        // Re-ingesting an existing label (rekey) replaces the copy and re-arms.
        let r = feed.ingest_key_copy(40, vec![6u8; 32]);
        assert!(r.armed && r.arm_count == 2, "rekey must re-arm");
        let stats = feed.stats();
        assert_eq!(stats.key_copies, 6);
        assert_eq!(stats.arm_count, 2);
        assert!(stats.armed);
    }

    #[test]
    fn a2_no_arming_config_means_copies_counted_never_armed() {
        // Control C1 (unit): without an arming config the feed only counts.
        let mut feed = SwarmLiveFeed::new();
        for j in 0..4u64 {
            feed.ingest_key_copy(j, vec![j as u8; 32]);
        }
        let stats = feed.stats();
        assert_eq!(stats.key_copies, 4);
        assert!(!stats.armed);
        assert_eq!(stats.arm_count, 0);
        assert!(feed.observe_wire_frame(len_of(0)).is_none());
        assert_eq!(feed.trips(), 0);
    }

    #[test]
    fn a3_armed_from_ingested_copies_flood_trips_within_two_benign_silent() {
        // Replica /tmp/arm_predict.py (seed 20261030, 48 random key sets):
        // flood 48/48 trip with trip_reads <= 2 (max 2); benign 0/48 trips.
        let mut flood_feed = armed_from_ingest(Order::L);
        assert!(flood_feed.is_armed());
        let mut receipt = None;
        for _ in 0..16 {
            if let Some(r) = flood_feed.observe_wire_frame(len_of(0)) {
                receipt = Some(r);
                break;
            }
        }
        let receipt = receipt.expect("flood must trip (replica: 48/48 le2 max 2)");
        assert!(receipt.trip_reads <= 2, "trip_reads {}", receipt.trip_reads);
        assert_eq!(flood_feed.trips(), 1);

        let mut benign_feed = armed_from_ingest(Order::L);
        for j in 0..4u8 {
            assert!(benign_feed.observe_wire_frame(len_of(j)).is_none());
        }
        assert_eq!(benign_feed.trips(), 0, "benign must stay silent (replica: 0/48)");
    }
}
