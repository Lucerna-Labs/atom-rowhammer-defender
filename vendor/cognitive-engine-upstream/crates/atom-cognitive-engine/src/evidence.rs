//! BUILD-23 — acquired evidence: the reach-out door.
//! Precommitted in `docs/BUILD23-ACQUIRED-EVIDENCE.md` BEFORE this code.
//!
//! The engine declares the SEAM and never the source: an `EvidenceSource` is
//! asked once, unconditionally (exactly as the ask stage always runs), and
//! what comes back is PROVENANCE — a source identity, a receipt hash, and
//! counts. No payload is stored, nothing is inspected to route, and every
//! verdict remains computed from the engine's own blocks.
//!
//! AUDIT: no branch, no policy, no storage; counts and hashes only.

/// What an acquisition returns: where it came from, its receipt, and how much
/// material it carried. Content is NOT included — the admitted payload is not
/// ours to keep (a separate later decision).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evidence {
    pub source: String,
    pub receipt_hash: String,
    pub nodes: usize,
    pub edges: usize,
}

/// The seam. The engine asks; it never knows what answers (a lightcone
/// adapter is one implementation, a fixture another).
pub trait EvidenceSource {
    fn acquire(&self, query: &str) -> Result<Evidence, EvidenceError>;
}

/// A typed, closed failure: an acquisition that cannot complete produces no
/// report and no fallback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceError(pub String);
