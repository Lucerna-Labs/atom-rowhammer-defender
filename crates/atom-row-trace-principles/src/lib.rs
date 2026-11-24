use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

pub const MEMORY_EVENTS: usize = 65_536;
pub const HEAT_DECAY_NS: u64 = 500_000;

/// Membership among the most recent events whose age lies inside the lifetime.
/// Counts distinguish simultaneous copies of the same identity during eviction.
pub struct Residence<K> {
    lifetime_ns: u64,
    events: VecDeque<(u64, K)>,
    counts: HashMap<K, usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeReversal;

impl std::fmt::Display for TimeReversal {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str("trace timestamp moved backward")
    }
}

impl std::error::Error for TimeReversal {}

impl<K: Copy + Eq + Hash> Residence<K> {
    pub fn new(lifetime_ns: u64) -> Self {
        Self {
            lifetime_ns,
            events: VecDeque::new(),
            counts: HashMap::new(),
        }
    }

    pub fn visit(&mut self, now: u64, identity: K) -> Result<bool, TimeReversal> {
        if self.events.back().is_some_and(|(time, _)| *time > now) {
            return Err(TimeReversal);
        }
        while self
            .events
            .front()
            .is_some_and(|(time, _)| now - time > self.lifetime_ns)
        {
            self.evict_oldest();
        }
        let present = self.counts.contains_key(&identity);
        if self.events.len() == MEMORY_EVENTS {
            self.evict_oldest();
        }
        self.events.push_back((now, identity));
        *self.counts.entry(identity).or_default() += 1;
        Ok(present)
    }

    fn evict_oldest(&mut self) {
        if let Some((_, key)) = self.events.pop_front() {
            if let std::collections::hash_map::Entry::Occupied(mut entry) = self.counts.entry(key) {
                *entry.get_mut() -= 1;
                if *entry.get() == 0 {
                    entry.remove();
                }
            }
        }
    }
}

/// Exact solution of first-order cooling between two impulses.
pub fn retained_fraction(elapsed_ns: u64) -> f64 {
    (-(elapsed_ns as f64) / HEAT_DECAY_NS as f64).exp()
}
