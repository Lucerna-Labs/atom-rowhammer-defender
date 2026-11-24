//! Memory traces: the engine's durable state units.

/// Crystallization tier. `Core` traces are protected identity substrate:
/// contradictions are counted but do not dissolve them, and capacity
/// eviction never selects them. `Standard` traces live the full
/// revise/evict lifecycle. (Mother's anchors vs career memories.)
/// `Candidate` traces are kinetic-proofreading probationers: staged but
/// uncommitted evidence — invisible to queries, fast-decaying, and
/// reset by contradiction — until promotion earns Standard/Core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceTier {
    Core,
    Standard,
    Candidate,
}

/// One associative memory trace: cue pattern bound to an evidence
/// distribution over values, with mass/support/ttl bookkeeping.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTrace {
    pub id: u64,
    pub cue: Vec<f64>,
    pub evidence: Vec<f64>,
    pub mass: f64,
    pub support: f64,
    pub ttl: u64,
    pub hits: u64,
    /// Kinetic-proofreading stage count: distinct-tick re-stagings this
    /// candidate has survived without contradiction.
    pub stages: u64,
    /// The tick of the last stage advance (same-tick re-stagings are not
    /// independent verification).
    pub last_tick_staged: u64,
    pub contradictions: u64,
    pub created_tick: u64,
    pub last_tick: u64,
    pub active: bool,
    pub tier: TraceTier,
    /// Provenance for generated blends: the two parent trace ids (empty
    /// for observed memories). A blend inherits structure from both and
    /// is born a KPR Candidate — the environment selects.
    pub parents: Vec<u64>,
    /// Multiscale renormalization (#5): the fine-scale ledger. Non-empty
    /// on a GRAIN — the active coarse trace that absorbed its members
    /// (ids, in absorption order; nested ledgers concatenate). Members
    /// rest inactive inside the grain with their evidence frozen; a
    /// contradiction at the coarse scale SPLITS the grain and re-activates
    /// them (what was removed becomes reachable again). Empty = ordinary
    /// trace.
    pub grain_members: Vec<u64>,
    /// Habituation/sensitization state (W15, Groves & Thompson dual
    /// process): 1.0 = fully novel. Decays toward 0 with repeated
    /// CONFIRMATION (habituation — attention stops saturating on
    /// constants); restored to 1.0 by contradiction/surprise
    /// (sensitization — strong stimuli regain attention). Observe-side
    /// only: queries READ this field and never write it.
    pub novelty: f64,
    /// Causal origin (#4): the attribution id of the source that last
    /// wrote this trace (0 = unattributed, the default stream). Two
    /// DIFFERENT explicit origins are CONCURRENT writers: a
    /// contradiction between them never dissolves the standing belief
    /// (arrival order is not causal order — no silent last-writer-wins);
    /// same-origin (or unattributed) revisions supersede as always.
    pub origin: u64,
}

impl MemoryTrace {
    pub fn reliability(&self) -> f64 {
        let hit_term = 1.0 - (-0.55 * self.hits as f64).exp();
        0.45 * self.support + 0.30 * self.mass.min(1.0) + 0.25 * hit_term
    }

    /// Current asserted value: argmax of the evidence distribution.
    pub fn value(&self) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_evidence = f64::MIN;
        for (index, evidence) in self.evidence.iter().enumerate() {
            if *evidence > best_evidence {
                best_evidence = *evidence;
                best = Some(index);
            }
        }
        best
    }

    /// Normalized evidence distribution (uniform when no evidence accrued).
    pub fn value_distribution(&self) -> Vec<f64> {
        let total: f64 = self.evidence.iter().sum();
        if total <= f64::EPSILON {
            vec![1.0 / self.evidence.len() as f64; self.evidence.len()]
        } else {
            self.evidence.iter().map(|e| e / total).collect()
        }
    }

    /// Compact state line for boundary hashing (O9).
    pub fn state_line(&self) -> String {
        let cue = self
            .cue
            .iter()
            .map(|v| format!("{v:.6}"))
            .collect::<Vec<_>>()
            .join(",");
        let evidence = self
            .evidence
            .iter()
            .map(|v| format!("{v:.6}"))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{}|{}|{}|{:.6}|{:.6}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{:.6}|{}",
            self.id,
            cue,
            evidence,
            self.mass,
            self.support,
            self.ttl,
            self.hits,
            self.contradictions,
            self.created_tick,
            self.last_tick,
            self.active,
            match self.tier {
                TraceTier::Core => "core",
                TraceTier::Standard => "std",
                TraceTier::Candidate => "cand",
            },
            if self.parents.is_empty() {
                "-".to_string()
            } else {
                self.parents
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(">")
            },
            if self.grain_members.is_empty() {
                "-".to_string()
            } else {
                self.grain_members
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join("+")
            },
            self.novelty,
            self.origin
        )
    }
}

/// Cosine similarity; zero when either vector is degenerate.
pub fn cosine_similarity(left: &[f64], right: &[f64]) -> f64 {
    debug_assert_eq!(left.len(), right.len());
    let mut dot = 0.0;
    let mut left_norm = 0.0;
    let mut right_norm = 0.0;
    for (a, b) in left.iter().zip(right.iter()) {
        dot += a * b;
        left_norm += a * a;
        right_norm += b * b;
    }
    let denominator = left_norm.sqrt() * right_norm.sqrt();
    if denominator <= f64::EPSILON {
        0.0
    } else {
        dot / denominator
    }
}

/// Numerically stable softmax.
pub fn softmax(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let maximum = values.iter().cloned().fold(f64::MIN, f64::max);
    let exponentials: Vec<f64> = values.iter().map(|v| (v - maximum).exp()).collect();
    let total: f64 = exponentials.iter().sum();
    exponentials.iter().map(|e| e / total).collect()
}

/// Deterministic unit value from the engine's xorshift-style stream.
pub fn seeded_unit(state: &mut u64) -> f64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    (x % 1_000_003) as f64 / 1_000_003.0 * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_is_argmax() {
        let trace = MemoryTrace {
            id: 1,
            cue: vec![1.0; 4],
            evidence: vec![0.1, 0.9, 0.2, 0.0],
            mass: 1.0,
            support: 0.7,
            ttl: 8,
            hits: 3,
            stages: 3,
            last_tick_staged: 0,
            contradictions: 0,
            created_tick: 0,
            last_tick: 0,
            active: true,
            tier: TraceTier::Standard,
            parents: Vec::new(),
            grain_members: Vec::new(),
            novelty: 1.0,
            origin: 0,
        };
        assert_eq!(trace.value(), Some(1));
    }

    #[test]
    fn softmax_sums_to_one() {
        let weights = softmax(&[0.2, 3.0, -1.0]);
        let total: f64 = weights.iter().sum();
        assert!((total - 1.0).abs() < 1e-12);
    }

    #[test]
    fn seeded_unit_is_deterministic_and_bounded() {
        let mut a = 12345u64;
        let mut b = 12345u64;
        for _ in 0..100 {
            let left = seeded_unit(&mut a);
            let right = seeded_unit(&mut b);
            assert_eq!(left, right);
            assert!((-1.0..=1.0).contains(&left));
        }
    }
}
