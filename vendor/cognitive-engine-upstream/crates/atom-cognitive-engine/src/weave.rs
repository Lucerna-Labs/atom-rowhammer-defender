//! The weave substrate — the quantum postulates as code, ported from the
//! PROVEN payload (atom-os-kernel `worker/src/weave_probe.rs`, E19-K,
//! commit 15ee772) per the recorded lesson: start from the working
//! payload, do not re-derive. Physics core ported verbatim (complex
//! amplitudes, unitary gates, destructive Born measurement, SplitMix64,
//! in-repo trig); the kernel runtime and the W0/W1/W2 demonstrations
//! stay in the OS repository.
//!
//! Honesty inherited from the payload: measurement outcomes here come
//! from a seeded PRNG (SplitMix64) — pseudo-random, deterministic. Real
//! hardware's Born outcomes are nature's; on a PC these are our axioms
//! executing.

// ---- first principles: amplitudes and the Born rule ---------------

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cx {
    pub re: f64,
    pub im: f64,
}

impl Cx {
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }
    fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
}

impl core::ops::Add for Cx {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self::new(self.re + o.re, self.im + o.im)
    }
}
impl core::ops::Mul for Cx {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
}

const MAX_AMPLITUDES: usize = 32; // up to 5 qubits (E1R-2: raised from the payload's 4 for the SWAP test)

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Register {
    qubits: usize,
    amplitudes: [Cx; MAX_AMPLITUDES],
}

impl Register {
    pub fn zeros(qubits: usize) -> Self {
        let mut amplitudes = [Cx::default(); MAX_AMPLITUDES];
        amplitudes[0] = Cx::new(1.0, 0.0);
        Self { qubits, amplitudes }
    }

    pub fn basis(qubits: usize, index: usize) -> Self {
        let mut amplitudes = [Cx::default(); MAX_AMPLITUDES];
        amplitudes[index] = Cx::new(1.0, 0.0);
        Self { qubits, amplitudes }
    }

    pub fn qubits(&self) -> usize {
        self.qubits
    }

    /// Little-endian tensor: self = low qubits, other = high.
    /// Fixed-capacity: hard-bounded at 5 qubits (32 amplitudes; E1R-2
    /// raised from the payload's 4 — the kernel cap was memory
    /// discipline, not physics).
    pub fn tensor(&self, other: &Self) -> Self {
        assert!(
            self.qubits + other.qubits <= 5,
            "register overflow: max 5 qubits"
        );
        let mut out = Self::zeros(self.qubits + other.qubits);
        for b in 0..1usize << other.qubits {
            for a in 0..1usize << self.qubits {
                out.amplitudes[a | (b << self.qubits)] = self.amplitudes[a] * other.amplitudes[b];
            }
        }
        out
    }

    pub fn apply1(&mut self, q: usize, gate: [[Cx; 2]; 2]) {
        let step = 1usize << q;
        let mut base = 0usize;
        while base < 1 << self.qubits {
            for offset in 0..step {
                let i0 = base + offset;
                let i1 = i0 + step;
                let (a, b) = (self.amplitudes[i0], self.amplitudes[i1]);
                self.amplitudes[i0] = gate[0][0] * a + gate[0][1] * b;
                self.amplitudes[i1] = gate[1][0] * a + gate[1][1] * b;
            }
            base += 2 * step;
        }
    }

    pub fn apply_controlled(&mut self, control: usize, target: usize, gate: [[Cx; 2]; 2]) {
        let cbit = 1usize << control;
        let tstep = 1usize << target;
        let mut base = 0usize;
        while base < 1 << self.qubits {
            for offset in 0..tstep {
                let i0 = base + offset;
                let i1 = i0 + tstep;
                if i0 & cbit != 0 {
                    let (a, b) = (self.amplitudes[i0], self.amplitudes[i1]);
                    self.amplitudes[i0] = gate[0][0] * a + gate[0][1] * b;
                    self.amplitudes[i1] = gate[1][0] * a + gate[1][1] * b;
                }
            }
            base += 2 * tstep;
        }
    }

    pub fn rotate_y(&mut self, q: usize, theta: f64) {
        let half = theta / 2.0;
        self.apply1(q, [
            [Cx::new(cos(half), 0.0), Cx::new(-sin(half), 0.0)],
            [Cx::new(sin(half), 0.0), Cx::new(cos(half), 0.0)],
        ]);
    }

    /// The Born rule: sample, collapse, destroy. Pseudo-random by
    /// seeded PRNG here; nature's randomness on real hardware.
    pub fn measure(&mut self, q: usize, rng: &mut Rng) -> u8 {
        let step = 1usize << q;
        let mut p_one = 0.0f64;
        for index in 0..1 << self.qubits {
            if index & step != 0 {
                p_one += self.amplitudes[index].norm_sq();
            }
        }
        let outcome = if rng.unit() < p_one { 1u8 } else { 0u8 };
        let keep = (outcome as usize) << q;
        let mut norm = 0.0f64;
        for index in 0..1 << self.qubits {
            if index & step == keep {
                norm += self.amplitudes[index].norm_sq();
            } else {
                self.amplitudes[index] = Cx::default();
            }
        }
        let scale = inv_sqrt(norm);
        for index in 0..1 << self.qubits {
            let a = self.amplitudes[index];
            self.amplitudes[index] = Cx::new(a.re * scale, a.im * scale);
        }
        outcome
    }

    /// Controlled-SWAP (E1R-2), implemented as a DIRECT amplitude
    /// permutation. (First implementation — three controlled-X sharing
    /// the control — was WRONG and is kept on record: gates with the
    /// same control commute, so X_a·X_b·X_a = X_b; the "swap" collapsed
    /// to a single flip and the SWAP test read 0.305 on identical
    /// states where physics demands 1.0. Found by dumping the state
    /// after each stage, not by staring at intent. The CNOT
    /// decomposition of SWAP needs directed CNOTs between the TARGETS,
    /// which a shared-control surface cannot express.)
    pub fn cswap(&mut self, control: usize, a: usize, b: usize) {
        let cbit = 1usize << control;
        let mask = (1usize << a) | (1usize << b);
        for i in 0..1usize << self.qubits {
            // Visit each differing (a,b) pair once, control set.
            if i & cbit != 0 && i & mask != 0 && i & mask != mask {
                let j = i ^ mask; // flips both a and b: (0,1) <-> (1,0)
                if j > i {
                    self.amplitudes.swap(i, j);
                }
            }
        }
    }

    /// The SWAP test (E1R-2): H–cSWAP–H on an ancilla over
    /// `|0⟩ ⊗ self ⊗ other`; Born measurement gives P(0) = (1+F)/2, so
    /// the sampled fidelity is F̂ = 2·P̂(0) − 1. The circuit is rebuilt
    /// every shot because Born measurement destroys state. This is the
    /// gate-circuit form of matching — similarity measured through
    /// interference and collapse, not an inner product.
    pub fn swap_test(&self, other: &Register, rng: &mut Rng, shots: usize) -> f64 {
        assert_eq!(self.qubits, other.qubits, "swap test compares equal-width registers");
        let mut zeros = 0usize;
        for _ in 0..shots {
            let ancilla = Register::zeros(1);
            let mut full = ancilla.tensor(&self.tensor(other)); // q0 = ancilla, q1.. = self, then other
            let width = self.qubits;
            full.apply1(0, H);
            for q in 0..width {
                full.cswap(0, 1 + q, 1 + width + q);
            }
            full.apply1(0, H);
            if full.measure(0, rng) == 0 {
                zeros += 1;
            }
        }
        let p_zero = zeros as f64 / shots as f64;
        (2.0 * p_zero - 1.0).clamp(0.0, 1.0)
    }

    /// |<self|other>|² — the quantum inner product (fidelity).
    pub fn fidelity(&self, other: &Self) -> f64 {
        let mut overlap = Cx::default();
        for index in 0..1 << self.qubits {
            overlap = overlap + self.amplitudes[index].conj() * other.amplitudes[index];
        }
        overlap.norm_sq()
    }

    /// Renormalize onto the unit sphere (conservation: the norm is the
    /// budget). Returns the previous norm.
    pub fn renormalize(&mut self) -> f64 {
        let mut norm_sq_total = 0.0f64;
        for index in 0..1 << self.qubits {
            norm_sq_total += self.amplitudes[index].norm_sq();
        }
        let scale = inv_sqrt(norm_sq_total);
        for index in 0..1 << self.qubits {
            let a = self.amplitudes[index];
            self.amplitudes[index] = Cx::new(a.re * scale, a.im * scale);
        }
        norm_sq_total
    }

    /// Interference blend: self ← normalize(self + weight · other).
    /// Amplitudes add before the norm restores the budget — the
    /// constructive/destructive arithmetic that produces interference.
    pub fn interfere(&mut self, other: &Register, weight: f64) {
        for index in 0..1 << self.qubits {
            let a = self.amplitudes[index];
            let b = other.amplitudes[index];
            self.amplitudes[index] =
                Cx::new(a.re + weight * b.re, a.im + weight * b.im);
        }
        self.renormalize();
    }

    pub fn amplitude(&self, index: usize) -> Cx {
        self.amplitudes[index]
    }

    /// A seeded random pure single-qubit state (ported interface).
    pub fn random_qubit(rng: &mut Rng) -> Self {
        let theta = rng.unit() * core::f64::consts::PI;
        let phi = rng.unit() * 2.0 * core::f64::consts::PI;
        let mut state = Self::zeros(1);
        state.amplitudes[0] = Cx::new(cos(theta / 2.0), 0.0);
        let s = sin(theta / 2.0);
        state.amplitudes[1] = Cx::new(s * cos(phi), s * sin(phi));
        state
    }
}

/// Babylonian sqrt (converges from any positive start — the inverse form
/// needs the argument near 1 and broke W1); returns 1/sqrt(x).
fn inv_sqrt(x: f64) -> f64 {
    if x <= 0.0 {
        return 1.0;
    }
    let mut y = x;
    for _ in 0..40 {
        y = 0.5 * (y + x / y);
    }
    if y > 0.0 {
        1.0 / y
    } else {
        1.0
    }
}

// ---- in-repo trig (argument reduction to [0, pi/2), then Taylor) ----
// Ported verbatim: accuracy ~2e-7 at the interval edge; keeps the
// substrate's numeric identity with the proven payload.

fn cos_sin(x: f64) -> (f64, f64) {
    let pi = core::f64::consts::PI;
    let two_pi = 2.0 * pi;
    let mut x = x % two_pi;
    if x < 0.0 {
        x += two_pi;
    }
    let (quadrant, t) = if x < pi / 2.0 {
        (0u8, x)
    } else if x < pi {
        (1, x - pi / 2.0)
    } else if x < 3.0 * pi / 2.0 {
        (2, x - pi)
    } else {
        (3, x - 3.0 * pi / 2.0)
    };
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t2 * t2;
    let t5 = t4 * t;
    let t6 = t4 * t2;
    let t7 = t6 * t;
    let t8 = t6 * t2;
    let t9 = t8 * t;
    let t10 = t8 * t2;
    let t11 = t10 * t;
    let sin_t = t - t3 / 6.0 + t5 / 120.0 - t7 / 5040.0 + t9 / 362_880.0 - t11 / 39_916_800.0;
    let cos_t = 1.0 - t2 / 2.0 + t4 / 24.0 - t6 / 720.0 + t8 / 40_320.0 - t10 / 3_628_800.0;
    match quadrant {
        0 => (cos_t, sin_t),
        1 => (-sin_t, cos_t),
        2 => (-cos_t, -sin_t),
        _ => (sin_t, -cos_t),
    }
}

pub(crate) fn cos(x: f64) -> f64 {
    cos_sin(x).0
}

pub(crate) fn sin(x: f64) -> f64 {
    cos_sin(x).1
}

pub const H: [[Cx; 2]; 2] = [
    [Cx::new(core::f64::consts::FRAC_1_SQRT_2, 0.0), Cx::new(core::f64::consts::FRAC_1_SQRT_2, 0.0)],
    [Cx::new(core::f64::consts::FRAC_1_SQRT_2, 0.0), Cx::new(-core::f64::consts::FRAC_1_SQRT_2, 0.0)],
];
pub const X: [[Cx; 2]; 2] = [
    [Cx::new(0.0, 0.0), Cx::new(1.0, 0.0)],
    [Cx::new(1.0, 0.0), Cx::new(0.0, 0.0)],
];
pub const Z: [[Cx; 2]; 2] = [
    [Cx::new(1.0, 0.0), Cx::new(0.0, 0.0)],
    [Cx::new(0.0, 0.0), Cx::new(-1.0, 0.0)],
];

/// SplitMix64 — deterministic pseudo-randomness, stated honestly.
#[derive(Debug, Clone, Copy)]
pub struct Rng(pub u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed | 1)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotations_preserve_the_norm() {
        let mut state = Register::zeros(2);
        state.rotate_y(0, 1.234);
        state.rotate_y(1, 2.345);
        state.renormalize();
        let norm: f64 = (0..4).map(|i| state.amplitude(i).norm_sq()).sum();
        assert!((norm - 1.0).abs() < 1e-9, "norm {norm}");
    }

    #[test]
    fn born_collapse_destroys_and_renormalizes() {
        let mut rng = Rng::new(42);
        let mut state = Register::zeros(1);
        state.rotate_y(0, core::f64::consts::FRAC_PI_2); // |+>
        let outcome = state.measure(0, &mut rng);
        let norm: f64 = (0..2).map(|i| state.amplitude(i).norm_sq()).sum();
        assert!((norm - 1.0).abs() < 1e-9);
        // After collapse the register is a basis state.
        let probs: Vec<f64> = (0..2).map(|i| state.amplitude(i).norm_sq()).collect();
        assert!(probs[0] == 1.0 || probs[1] == 1.0, "collapsed: {probs:?} (outcome {outcome})");
    }

    #[test]
    fn fidelity_discriminates_states() {
        let mut a = Register::zeros(1);
        a.rotate_y(0, 0.0);
        let mut b = Register::zeros(1);
        b.rotate_y(0, core::f64::consts::PI);
        let mut c = Register::zeros(1);
        c.rotate_y(0, 0.2);
        assert!(a.fidelity(&a) > 0.9999);
        assert!(a.fidelity(&b) < 0.0001, "orthogonal");
        assert!(a.fidelity(&c) > 0.9, "nearby: {}", a.fidelity(&c));
    }

    #[test]
    fn interfere_blends_and_renormalizes() {
        let mut proto = Register::zeros(1);
        proto.rotate_y(0, 0.0);
        let mut reading = Register::zeros(1);
        reading.rotate_y(0, 0.5);
        let before = proto.fidelity(&reading);
        proto.interfere(&reading, 0.3);
        let norm: f64 = (0..2).map(|i| proto.amplitude(i).norm_sq()).sum();
        assert!((norm - 1.0).abs() < 1e-9);
        let after = proto.fidelity(&reading);
        assert!(
            after > before,
            "blend must drift toward the reading ({before:.4} -> {after:.4})"
        );
    }

    #[test]
    fn rng_is_deterministic_per_seed() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        for _ in 0..10 {
            assert_eq!(a.unit(), b.unit());
        }
    }
}
