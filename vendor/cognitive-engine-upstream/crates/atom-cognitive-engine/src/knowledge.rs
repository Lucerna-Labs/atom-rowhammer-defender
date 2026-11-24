//! BUILD-11 — generative knowledge: blocks stored, answers generated.
//! Precommitted in `docs/BUILD11-KNOWLEDGE.md` BEFORE this code, under
//! vault Rules 1/2/3, project rules 0/0a/0b, and docs/PURPOSE.md.
//!
//! STORED: block values and source contents (facts — content).
//! NOT STORED: any procedure. Multiplication IS iterated translation
//! (the BUILD-6 march in its single-mode state); sufficiency IS closure
//! membership (a measurement).
//!
//! AUDIT: the register sizes (structural), the step (a block value),
//! the reduction of the single-mode field to its phase (derivable from
//! BUILD-6's step on a single-mode field — documented, not a rule).

use crate::geometry::Point;
use crate::interference_field::N;

/// The single-mode register: a field state reduced to its phase (the
/// general field step of BUILD-6 reduces exactly to this on one mode).
/// The step is ITERATED TRANSLATION — multiplication by the step count.
pub struct MarchRegister {
    modulus: u64,
    step: u64, // the stored block
    phase: f64,
}

impl MarchRegister {
    /// A register of size `modulus` holding the block `step` at phase 0.
    pub fn new(modulus: u64, step: u64) -> Self {
        MarchRegister {
            modulus,
            step,
            phase: 0.0,
        }
    }

    /// One iteration: advance by the step (repeated addition IS the
    /// multiplication; nothing here knows the target).
    pub fn step_once(&mut self) {
        self.phase += self.step as f64 / self.modulus as f64;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }

    /// Folded readout: the phase as a frequency in (−N/2, N/2]
    /// (Nyquist — the band a phase measurement can see).
    pub fn folded_readout(&self) -> i64 {
        let mut frequency = self.phase * self.modulus as f64;
        let half = self.modulus as f64 / 2.0;
        if frequency > half {
            frequency -= self.modulus as f64;
        }
        frequency.round() as i64
    }
}

/// Run `steps` iterations and read the folded product.
pub fn generated_product(modulus: u64, block: u64, steps: u64) -> i64 {
    let mut register = MarchRegister::new(modulus, block);
    for _ in 0..steps {
        register.step_once();
    }
    register.folded_readout()
}

/// Reconstruct the exact product from two folded readouts at coprime
/// register sizes (n2 = n1 - 1; the inverse of n2 mod n1 is n1 - 1).
pub fn reconstruct(n1: u64, m1: i64, n2: u64, m2: i64) -> u128 {
    debug_assert_eq!(n2, n1 - 1);
    let n1_128 = n1 as u128;
    let m1_mod = (m1.rem_euclid(n1 as i64)) as u128;
    let m2_mod = (m2.rem_euclid(n2 as i64)) as u128;
    let diff = (m1_mod + n1_128 - m2_mod % n1_128) % n1_128;
    let k = (diff * (n1_128 - 1)) % n1_128;
    m2_mod + k * (n2 as u128)
}

/// SUFFICIENCY — a measurement: is the target in the closure of the
/// stored relation blocks? (Exact integer membership.)
pub fn derivable(blocks: &[Point], target: Point) -> bool {
    let modulus = N as i64;
    let mut reachable: Vec<Point> = vec![(0, 0)];
    let mut frontier = vec![(0i64, 0i64)];
    while let Some((x, y)) = frontier.pop() {
        for (gx, gy) in blocks {
            let next = ((x + gx).rem_euclid(modulus), (y + gy).rem_euclid(modulus));
            if !reachable.contains(&next) {
                reachable.push(next);
                frontier.push(next);
            }
        }
    }
    reachable.contains(&(target.0.rem_euclid(modulus), target.1.rem_euclid(modulus)))
}

/// The number of answers derivable from the stored blocks (closure
/// size) — the knowledge-economy numerator.
pub fn derivable_count(blocks: &[Point]) -> usize {
    let modulus = N as i64;
    let mut reachable: Vec<Point> = vec![(0, 0)];
    let mut frontier = vec![(0i64, 0i64)];
    while let Some((x, y)) = frontier.pop() {
        for (gx, gy) in blocks {
            let next = ((x + gx).rem_euclid(modulus), (y + gy).rem_euclid(modulus));
            if !reachable.contains(&next) {
                reachable.push(next);
                frontier.push(next);
            }
        }
    }
    reachable.len()
}

// ---- BUILD-31: the inverse readouts -------------------------------------
//
// The operations the sufficiency inventory recorded as MISSING — division,
// divisibility, the modular inverse, gcd — are here READINGS of structures
// the composition already produces: a counted subgroup order, an equality,
// a shortest word length. There is no Euclid, no extended Euclid and no
// division loop anywhere below. The readouts are MODULAR (within a declared
// register); the unbounded range is reached only through the CRT pair above.
// `derivable`/`derivable_count` stay bound to the engine's lattice N; these
// take the modulus as a parameter, because the register lattice is what
// BUILD-29's word graph generalizes to.

use crate::derivation_paths::WordGraph;

/// gcd(a, b) — `b` divided by the order of the subgroup `⟨a⟩` in `Z_b`, the
/// closure the register lattice already builds. `None` when `b == 0`: there
/// is no register, so no reading — fail-closed, not zero.
pub fn gcd_of(a: u64, b: u64) -> Option<u64> {
    if b == 0 {
        return None;
    }
    let order = WordGraph::from_register(b as i64, a as i64).member_count() as u64;
    Some(b / order)
}

/// `a` divides `b` — the gcd reading's equality against `a`. `b == 0` has no
/// register and the readout makes no claim (false) rather than reporting the
/// conventional answer its structure cannot measure.
pub fn divides(a: u64, b: u64) -> bool {
    b != 0 && gcd_of(a, b) == Some(a)
}

/// The least `k >= 0` with `k · block ≡ target (mod modulus)` — the shortest
/// word length from the register's origin to the residue: how many steps of
/// the stored block land on it. `None` when the residue is not reachable
/// within the register (definitive, not a bound).
pub fn quotient_within(modulus: u64, block: u64, target: u64) -> Option<usize> {
    WordGraph::from_register(modulus as i64, block as i64).shortest_length((target as i64, 0))
}

/// The least `k >= 0` with `k · block ≡ 1 (mod modulus)` — the same reading
/// at `target = 1`. `None` when no inverse exists (fail-closed).
pub fn modular_inverse(modulus: u64, block: u64) -> Option<usize> {
    quotient_within(modulus, block, 1)
}
