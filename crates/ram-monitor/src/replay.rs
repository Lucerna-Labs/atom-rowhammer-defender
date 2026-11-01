//! Replay source: recorded or synthetic access addresses through the identical
//! monitor pipeline, so every gate is reproducible from generated observations.

use crate::{AccessSample, AccessSource};

/// A source over a fixed list of addresses.
pub struct ReplaySource {
    addrs: Vec<u64>,
    pos: usize,
}

impl ReplaySource {
    #[must_use]
    pub fn new(addrs: Vec<u64>) -> Self {
        Self { addrs, pos: 0 }
    }

    /// Hammer scenario: every sample inside one region of class `class`
    /// (region base = class << row_shift, default geometry).
    #[must_use]
    pub fn hammer(class: u8, n: usize, row_shift: u32) -> Self {
        let base = u64::from(class) << row_shift;
        Self::new((0..n).map(|k| base + (k % 64) as u64).collect())
    }

    /// Scatter control: walk 256 distinct regions before repeating, so no
    /// fingerprint recurs inside the 64-sample observer windows.
    #[must_use]
    pub fn scatter(n: usize, row_shift: u32) -> Self {
        Self::new((0..n).map(|k| ((k % 256) as u64) << row_shift).collect())
    }

    /// Mixed: 90% hammer class 0 + 10% scatter (deterministic splitmix).
    #[must_use]
    pub fn mixed(n: usize, row_shift: u32) -> Self {
        Self::new(
            (0..n)
                .map(|k| {
                    if splitmix(k as u64 ^ 0xA5A5).is_multiple_of(10) {
                        ((k % 8) as u64) << row_shift
                    } else {
                        (k % 64) as u64
                    }
                })
                .collect(),
        )
    }

    /// Benign: pseudo-random regions.
    #[must_use]
    pub fn benign(n: usize, row_shift: u32) -> Self {
        Self::new(
            (0..n)
                .map(|k| (splitmix(k as u64 ^ 0xBEEF) << row_shift) & ((1 << 40) - 1))
                .collect(),
        )
    }
}

fn splitmix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

impl AccessSource for ReplaySource {
    fn next_sample(&mut self) -> Option<AccessSample> {
        let addr = *self.addrs.get(self.pos)?;
        self.pos += 1;
        Some(AccessSample {
            addr,
            address_kind: crate::AddressKind::Synthetic,
            ..AccessSample::default()
        })
    }
}
