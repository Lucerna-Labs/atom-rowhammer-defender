//! Quorum-swarm tripwire over four owned session-key copies.
//!
//! This crate is the Imperium integration of the no-copy quorum-swarm stack
//! (vendored as `nocopy-swarm-principles` / `nocopy-swarm-engine` under
//! `encrypted key protection/`). It owns exactly four [`KeyCell`]s, each
//! holding a **copy** of a mesh session key, passed in by the caller. It never
//! reads, references, or holds a handle to any live key in `imperium-net-mesh`
//! (`PeerSession::direction_keys` remains the single live key holder).
//!
//! Two input feeds exist, and each swarm instance serves exactly one of them:
//!
//! - [`Feed::Frames`]: redacted [`FrameClass`] values from
//!   `imperium-nocopy-vpn-observer` (length class only, no payload bytes).
//! - [`Feed::SessionEvents`]: [`SessionEvent`] kinds derived from
//!   `imperium-net-mesh` `DaemonStats` counters by the wiring layer.
//!
//! Site mapping (declared, never selected from observation content): an input
//! carrying the home label of copy j is a read of copy j and is addressed to
//! site j with the copy's identity fingerprint; any other input is a read of
//! other data, addressed to site (read counter mod 4) with a label fingerprint.
//! Each read drives one cell's engine; the cell absorbs its reaction plus the
//! quorum coupling Q * (sum of the other cells' heat). When an **observed**
//! pattern ruptures a cell or the warm quorum fires, the crate returns a
//! [`SwarmReceipt`] — numbers only, never a verdict.
//!
//! Boundary: the crate holds no bus handle, never touches payloads, routes,
//! counters, or live keys, and cannot see unobserved probes. Already-read data
//! is not recovered; external copies survive. This is not a Rowhammer defense
//! and does not protect RAM or any other memory. "Hot/cold" are profile
//! parameters of a classical simulation, not physical temperature.

use imperium_nocopy_vpn_observer::FrameClass;
use nocopy_sensor_engine::{KeyCell, KeyMeasurement};
use nocopy_swarm_engine::SwarmEngine;
use nocopy_swarm_principles::{SwarmConfig as SwarmParams, SWARM_CELLS};
use thiserror::Error;

pub use nocopy_sensor_engine::Order;

/// Errors in swarm operation. Malformed input is rejected, never corrected.
#[derive(Debug, Error)]
pub enum SwarmError {
    /// Frame class outside the bounded 8-class table.
    #[error("frame class outside the bounded 8-class table")]
    FrameClassOutOfRange,
    /// Input does not match this swarm's feed.
    #[error("input does not match this swarm's feed")]
    WrongFeed,
    /// The swarm already tripped; tripped cells are zeroized and not re-armed.
    #[error("already tripped: tripped cells are zeroized and the swarm is not re-armed")]
    AlreadyTripped,
    /// The caller passed an empty key copy.
    #[error("invalid key copy: empty")]
    InvalidKey,
    /// A home label is outside the feed's bounded table or duplicated.
    #[error("home label outside the feed's bounded table or duplicated")]
    InvalidHome,
    /// The swarm engine rejected the observation.
    #[error("engine error: {0}")]
    Engine(String),
}

/// Which input feed a swarm instance serves. One feed per instance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Feed {
    /// Redacted frame-length classes from `imperium-nocopy-vpn-observer`.
    Frames,
    /// Mesh session event kinds (from `DaemonStats` counter deltas).
    SessionEvents,
}

/// A mesh session event kind. The wiring layer maps `imperium-net-mesh`
/// `DaemonStats` counter deltas onto these; this crate does not depend on the
/// mesh crate and never sees payloads, addresses, or keys.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionEvent {
    /// Inbound Hello that reached full hybrid signature verification.
    Hello,
    /// HelloResponse sent after a verified Hello.
    HelloResponse,
    /// Handshake completed (session established).
    HandshakeCompleted,
    /// Session rotated onto fresh key material (byte budget spent).
    Rekey,
    /// Session expired as stale.
    Expire,
    /// Session rebound to a new source address (NAT).
    Rebind,
    /// Keepalive ping/pong. Lower read strength than access-like events.
    Ping,
}

impl SessionEvent {
    /// The event's label in the session-event label space (1..=7).
    pub fn tag(self) -> u8 {
        match self {
            Self::Hello => 1,
            Self::HelloResponse => 2,
            Self::HandshakeCompleted => 3,
            Self::Rekey => 4,
            Self::Expire => 5,
            Self::Rebind => 6,
            Self::Ping => 7,
        }
    }

    /// Read strength of the event in the sensor model. Access-like events are
    /// full reads; pings are weak reads.
    fn quantity(self) -> f64 {
        match self {
            Self::Ping => 0.125,
            _ => 1.0,
        }
    }
}

/// Numerical receipt emitted when an observed distributed pattern trips the
/// swarm. Contains no verdict, label, score, or key material.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwarmReceipt {
    /// Which feed produced the tripping pattern.
    pub feed: Feed,
    /// Recipient observations since adoption at the moment the receipt fired.
    pub trip_reads: u64,
    /// Cells at/above the warm threshold at the moment of the trip.
    pub warm: usize,
    /// Ruptured cells at the moment of the trip.
    pub ruptured: usize,
    /// Per-cell heat at the moment of the trip.
    pub heats: [f64; SWARM_CELLS],
}

const FRAME_TAG: u8 = 0xF1;
const SESSION_TAG: u8 = 0xF2;

/// Test-only non-cryptographic label for one feed-space value: a feed tag
/// plus a label byte, mixed with a murmur3-style finalizer so distinct
/// labels land ~32 bits apart in Hamming distance. Never used on live key
/// material outside the owned copies.
fn label(tag: u8, value: u8) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64 ^ (u64::from(tag) << 56) ^ u64::from(value);
    for _ in 0..4 {
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(0xff51_afd7_ed55_8ccd);
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        hash ^= hash >> 33;
    }
    hash
}

/// Test-only non-cryptographic identity of an owned key copy: FNV-1a over
/// the copy's bytes, then a murmur3-style finalizer for full-width mixing.
fn key_identity(key_copy: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in key_copy {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    for _ in 0..2 {
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(0xff51_afd7_ed55_8ccd);
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        hash ^= hash >> 33;
    }
    hash
}

/// Fixed experiment structure for one swarm: the home label of each guarded
/// key copy, per-cell engine seeds, the engine order, the thermal/hill
/// profile, the donor placement, and the quorum parameters. Never selected
/// from observation content.
#[derive(Clone, Copy, Debug)]
pub struct SwarmTripwireConfig {
    /// Feed-space label of each guarded key copy (frame class 0..=7 for feed
    /// A, session event tag 1..=7 for feed B). Distinct per copy.
    pub homes: [u8; SWARM_CELLS],
    /// Per-cell engine seeds.
    pub seeds: [u64; SWARM_CELLS],
    /// Fixed observation order.
    pub order: Order,
    /// Software temperature of the profile (classical simulation parameter).
    pub thermal: f64,
    /// Hill profile on/off (squared-similarity weighting, unit resonance).
    pub hill: bool,
    /// Fixed donor placement (0..4).
    pub placement: u8,
    /// Quorum coupling strength Q.
    pub coupling: f64,
    /// Warm threshold theta for the quorum receipt.
    pub warm_threshold: f64,
    /// Quorum size K.
    pub quorum: usize,
}

/// One swarm: one feed, four engines, four owned key-copy cells.
pub struct SwarmTripwire {
    feed: Feed,
    swarm: SwarmEngine,
    homes: [u8; SWARM_CELLS],
    identities: [u64; SWARM_CELLS],
    tick: u64,
    reads: u64,
    tripped: bool,
    last_receipt: Option<SwarmReceipt>,
}

impl SwarmTripwire {
    /// Adopt four owned **copies** of session keys. The caller keeps the live
    /// keys; this crate only ever sees the copies it now owns.
    pub fn adopt_key_copies(
        feed: Feed,
        key_copies: [Vec<u8>; SWARM_CELLS],
        config: SwarmTripwireConfig,
    ) -> Result<Self, SwarmError> {
        let SwarmTripwireConfig {
            homes,
            seeds,
            order,
            thermal,
            hill,
            placement,
            coupling,
            warm_threshold,
            quorum,
        } = config;
        let bound = match feed {
            Feed::Frames => 7,
            Feed::SessionEvents => 7,
        };
        let min = match feed {
            Feed::Frames => 0,
            Feed::SessionEvents => 1,
        };
        for (j, home) in homes.iter().enumerate() {
            if *home < min || *home > bound || homes.iter().enumerate().any(|(k, other)| k != j && other == home) {
                return Err(SwarmError::InvalidHome);
            }
        }
        let params = SwarmParams::new(coupling, warm_threshold, quorum)
            .map_err(|_| SwarmError::InvalidHome)?;
        let mut key_vec = Vec::with_capacity(SWARM_CELLS);
        for copy in &key_copies {
            key_vec.push(KeyCell::new(copy.clone()).map_err(|_| SwarmError::InvalidKey)?);
        }
        let keys: [KeyCell; SWARM_CELLS] =
            key_vec.try_into().map_err(|_| SwarmError::InvalidKey)?;
        let identities = key_copies.each_ref().map(|copy| key_identity(copy));
        let swarm = SwarmEngine::new(seeds, keys, order, params, thermal, hill);
        let mut tripwire = Self {
            feed,
            swarm,
            homes,
            identities,
            tick: 0,
            reads: 0,
            tripped: false,
            last_receipt: None,
        };
        let _ = placement; // donor site is fixed per cell: site j
        for (site, &identity) in identities.iter().enumerate() {
            tripwire
                .swarm
                .register_donor(site, identity)
                .map_err(|error| SwarmError::Engine(format!("{error:?}")))?;
        }
        Ok(tripwire)
    }

    /// Feed A: one redacted frame class. Quantity is always a full read — a
    /// frame class carries no strength information. A frame of copy j's home
    /// class is modeled as a read of copy j; other classes are reads of other
    /// data, addressed round-robin over the cells.
    pub fn observe_frame(
        &mut self,
        class: FrameClass,
    ) -> Result<Option<SwarmReceipt>, SwarmError> {
        if self.feed != Feed::Frames {
            return Err(SwarmError::WrongFeed);
        }
        let FrameClass(value) = class;
        if value > 7 {
            return Err(SwarmError::FrameClassOutOfRange);
        }
        let (site, fp) = match self.homes.iter().position(|home| *home == value) {
            Some(j) => (j, self.identities[j]),
            None => ((self.reads % SWARM_CELLS as u64) as usize, label(FRAME_TAG, value)),
        };
        self.push(site, fp, 1.0)
    }

    /// Feed B: one mesh session event kind. An event of copy j's home kind is
    /// modeled as a read of copy j; other kinds are reads of other sessions,
    /// addressed round-robin over the cells.
    pub fn observe_session_event(
        &mut self,
        event: SessionEvent,
    ) -> Result<Option<SwarmReceipt>, SwarmError> {
        if self.feed != Feed::SessionEvents {
            return Err(SwarmError::WrongFeed);
        }
        let (site, fp) = match self.homes.iter().position(|home| *home == event.tag()) {
            Some(j) => (j, self.identities[j]),
            None => (
                (self.reads % SWARM_CELLS as u64) as usize,
                label(SESSION_TAG, event.tag()),
            ),
        };
        self.push(site, fp, event.quantity())
    }

    fn push(&mut self, site: usize, fingerprint: u64, quantity: f64) -> Result<Option<SwarmReceipt>, SwarmError> {
        if self.tripped {
            return Err(SwarmError::AlreadyTripped);
        }
        self.tick += 1;
        self.swarm
            .read(site, fingerprint, quantity, self.tick)
            .map_err(|error| SwarmError::Engine(format!("{error:?}")))?;
        self.reads += 1;
        let receipt = self.swarm.current_receipt();
        if receipt.trip_reads >= 0 {
            self.tripped = true;
            let swarm_receipt = SwarmReceipt {
                feed: self.feed,
                trip_reads: self.reads,
                warm: receipt.warm,
                ruptured: receipt.ruptured,
                heats: receipt.heats,
            };
            self.last_receipt = Some(swarm_receipt);
            return Ok(Some(swarm_receipt));
        }
        Ok(None)
    }

    /// Which feed this instance serves.
    #[must_use]
    pub fn feed(&self) -> Feed {
        self.feed
    }

    /// Whether an observed pattern has tripped the swarm.
    #[must_use]
    pub fn is_tripped(&self) -> bool {
        self.tripped
    }

    /// Recipient observations since adoption.
    #[must_use]
    pub fn reads(&self) -> u64 {
        self.reads
    }

    /// The receipt of the trip, if one occurred. Numerical only.
    #[must_use]
    pub fn last_receipt(&self) -> Option<SwarmReceipt> {
        self.last_receipt
    }

    /// The home labels of the guarded key copies in this feed's label space.
    #[must_use]
    pub fn homes(&self) -> [u8; SWARM_CELLS] {
        self.homes
    }

    /// Current measurement of each owned cell: heat, ruptured, nonzero bytes.
    #[must_use]
    pub fn key_measurements(&self) -> [KeyMeasurement; SWARM_CELLS] {
        let heats = self.swarm.heats();
        let flags = self.swarm.ruptured_flags();
        let nonzero = self.swarm.nonzero_bytes();
        core::array::from_fn(|site| KeyMeasurement {
            heat: heats[site],
            ruptured: flags[site],
            key_len: 32,
            nonzero_bytes: nonzero[site],
        })
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

    fn synthetic_key(index: usize) -> Vec<u8> {
        (0..32)
            .map(|position| 1 + (splitmix((index * 32 + position) as u64) % 255) as u8)
            .collect()
    }

    fn config(homes: [u8; 4], seed: u64) -> SwarmTripwireConfig {
        SwarmTripwireConfig {
            homes,
            seeds: [splitmix(seed), splitmix(seed ^ 1), splitmix(seed ^ 2), splitmix(seed ^ 3)],
            order: Order::L,
            thermal: 0.0,
            hill: true,
            placement: 0,
            coupling: 0.30,
            warm_threshold: 0.15,
            quorum: 4,
        }
    }

    fn copies() -> [Vec<u8>; 4] {
        [synthetic_key(0), synthetic_key(1), synthetic_key(2), synthetic_key(3)]
    }

    #[test]
    fn frame_single_flood_trips_within_two_reads() {
        for index in 0..8 {
            let mut swarm = SwarmTripwire::adopt_key_copies(
                Feed::Frames,
                copies(),
                config([0, 1, 2, 3], splitmix(index as u64)),
            )
            .unwrap();
            let mut receipt = None;
            for _ in 0..16 {
                if let Some(r) = swarm.observe_frame(FrameClass(0)).unwrap() {
                    receipt = Some(r);
                    break;
                }
            }
            let receipt = receipt.expect("single flood must trip");
            assert!(receipt.trip_reads <= 2, "trip_reads {}", receipt.trip_reads);
            assert_eq!(receipt.feed, Feed::Frames);
            assert!(swarm.is_tripped());
        }
    }

    #[test]
    fn frame_dist_flood_trips_within_eight_reads() {
        let mut swarm = SwarmTripwire::adopt_key_copies(
            Feed::Frames,
            copies(),
            config([0, 1, 2, 3], splitmix(42)),
        )
        .unwrap();
        let mut receipt = None;
        for k in 0..16 {
            if let Some(r) = swarm.observe_frame(FrameClass((k % 4) as u8)).unwrap() {
                receipt = Some(r);
                break;
            }
        }
        let receipt = receipt.expect("distributed flood must trip");
        assert!(receipt.trip_reads <= 8, "trip_reads {}", receipt.trip_reads);
    }

    #[test]
    fn benign_home_reads_do_not_trip() {
        let mut swarm = SwarmTripwire::adopt_key_copies(
            Feed::Frames,
            copies(),
            config([0, 1, 2, 3], splitmix(7)),
        )
        .unwrap();
        for j in 0..4_u8 {
            assert!(swarm.observe_frame(FrameClass(j)).unwrap().is_none());
        }
        assert!(!swarm.is_tripped());
        for m in swarm.key_measurements() {
            assert!(!m.ruptured);
            assert_eq!(m.nonzero_bytes, 32);
        }
    }

    #[test]
    fn session_event_flood_trips_and_wrong_feed_is_rejected() {
        let mut swarm = SwarmTripwire::adopt_key_copies(
            Feed::SessionEvents,
            copies(),
            config([1, 2, 3, 4], splitmix(9)),
        )
        .unwrap();
        assert!(swarm.observe_frame(FrameClass(1)).is_err());
        let mut receipt = None;
        for _ in 0..16 {
            if let Some(r) = swarm.observe_session_event(SessionEvent::Hello).unwrap() {
                receipt = Some(r);
                break;
            }
        }
        let receipt = receipt.expect("session flood must trip");
        assert!(receipt.trip_reads <= 2, "trip_reads {}", receipt.trip_reads);
    }

    #[test]
    fn invalid_homes_are_rejected() {
        assert!(SwarmTripwire::adopt_key_copies(
            Feed::Frames,
            copies(),
            config([0, 1, 2, 2], splitmix(1)),
        )
        .is_err());
        assert!(SwarmTripwire::adopt_key_copies(
            Feed::SessionEvents,
            copies(),
            config([0, 1, 2, 3], splitmix(1)),
        )
        .is_err());
    }
}
