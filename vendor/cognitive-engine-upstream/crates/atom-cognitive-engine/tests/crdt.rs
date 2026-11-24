//! CRDT convergence (#9, D57) — the admission gates: merge is
//! COMMUTATIVE and ASSOCIATIVE up to canonical content (ids are
//! engine-local ordinals, renumbered on the way in — convergence is
//! content-equal, not ordinal-equal), IDEMPOTENT (re-merge absorbs
//! nothing), conflict-preserving (convergence ≠ truth: independent
//! sides' contradictory values both survive the union), and fail-closed
//! on shape mismatch.

use atom_cognitive_engine::stacks::{observe_stack, query_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

fn durable() -> EngineConfig {
    EngineConfig {
        initial_ttl: 512,
        ttl_per_hit: 16,
        decay_support_rate: 0.0005,
        decay_mass_rate: 0.0003,
        conservation_budget: 1.0e6,
        capacity: 64,
        ..EngineConfig::default()
    }
}

/// An independent world: `symbol`'s slot observed `times` with `value`.
fn world(symbol: usize, value: usize, times: usize) -> CognitiveEngine {
    let mut engine = CognitiveEngine::new(durable()).expect("config");
    let observe = observe_stack();
    for _ in 0..times {
        engine
            .observe(&observe, &cue12(symbol), value, 1.0)
            .expect("observe");
    }
    engine
}

/// Canonical projection: the multiset of memories as content (ids
/// stripped), bonds as endpoint pairs. Convergence is judged here.
fn canonical(engine: &CognitiveEngine) -> Vec<String> {
    let mut lines: Vec<String> = engine
        .traces()
        .iter()
        .map(|t| {
            format!(
                "{:?}|{:?}|{:.6}|{:.6}|{}|{:?}|{:?}|{}|{}",
                t.cue, t.evidence, t.support, t.mass, t.hits, t.tier, t.active, t.origin, t.novelty
            )
        })
        .collect();
    lines.sort();
    lines
}

#[test]
fn gate_merge_is_commutative() {
    // Two independent worlds with colliding id spaces (both start at
    // id 1 with different content): A.merge(B) and B.merge(A) converge
    // to the same canonical content.
    let mut left = world(0, 1, 4);
    let right = world(5, 2, 3);
    left.merge(&right).expect("merge L+R");
    let mut flipped = world(5, 2, 3);
    let other = world(0, 1, 4);
    flipped.merge(&other).expect("merge R+L");
    assert_eq!(
        canonical(&left),
        canonical(&flipped),
        "merge direction must not change the canonical union"
    );
    // And both sides answer queries over the union identically.
    let query = query_stack();
    for symbol in [0usize, 5] {
        let a = left.query(&query, &cue12(symbol)).expect("query");
        let b = flipped.query(&query, &cue12(symbol)).expect("query");
        assert_eq!(a.claim, b.claim);
    }
}

#[test]
fn gate_merge_is_idempotent() {
    let mut engine = world(0, 1, 4);
    let incoming = world(5, 2, 3);
    let first = engine.merge(&incoming).expect("first merge");
    assert_eq!(first.traces_absorbed, 1);
    assert_eq!(first.bonds_added, 0);
    // Re-merge: the incoming content is already present (under a
    // renumbered id), so it content-DEDUPES — nothing joins (the id was
    // renumbered on the way in), nothing absorbs, nothing changes.
    let second = engine.merge(&incoming).expect("re-merge");
    assert_eq!(second.traces_absorbed, 0);
    assert_eq!(second.traces_joined, 0);
    assert_eq!(second.traces_deduplicated, 1);
    // Canonical content is stable across further re-merges.
    let mut probe = engine.clone();
    probe.merge(&incoming).expect("third merge");
    assert_eq!(canonical(&engine), canonical(&probe));
}

#[test]
fn gate_merge_is_associative() {
    let a = world(0, 1, 3);
    let b = world(3, 2, 3);
    let c = world(6, 0, 2);
    let mut ab_first = a.clone();
    ab_first.merge(&b).expect("A+B");
    ab_first.merge(&c).expect("(A+B)+C");
    let mut bc_first = b.clone();
    bc_first.merge(&c).expect("B+C");
    bc_first.merge(&a).expect("A+(B+C)");
    assert_eq!(
        canonical(&ab_first),
        canonical(&bc_first),
        "grouping must not change the union"
    );
}

#[test]
fn gate_convergence_is_not_truth() {
    // Two worlds assert DIFFERENT values on the same cue. The merge
    // must preserve both masses (contested ground) — it manufactures no
    // winner and erases no side.
    let mut merged = world(0, 1, 4);
    let rival = world(0, 2, 3);
    // Rival's trace shares the cue but NOT the id path: independent
    // worlds collide on id 1 with the same cue → ancestor-join branch
    // merges evidence MAXIMA, keeping BOTH values' peaks.
    let report = merged.merge(&rival).expect("merge rivals");
    assert_eq!(report.traces_joined, 1);
    let trace = merged.traces().first().expect("the contested trace");
    assert!(trace.evidence[1] > 0.0, "side A's mass survives");
    assert!(trace.evidence[2] > 0.0, "side B's mass survives");
    // The union answers, but the ground is honest about the split.
    let query = query_stack();
    let outcome = merged.query(&query, &cue12(0)).expect("query");
    assert!(outcome.claim.is_some());
    assert!(
        outcome.fluency.state == atom_cognitive_engine::FluencyState::Contested,
        "converged rivals must read contested, not confidently resolved (margin {})",
        outcome.fluency.margin
    );
}

#[test]
fn gate_shape_mismatch_refused_and_deterministic() {
    // Fail-closed: different cue dimensions never merge.
    let mut engine = CognitiveEngine::new(durable()).expect("config");
    let mut wide = CognitiveEngine::new(EngineConfig {
        cue_dim: 16,
        ..durable()
    })
    .expect("config");
    let observe = observe_stack();
    wide.observe(
        &observe,
        &{
            let mut cue = vec![0.0; 16];
            cue[0] = 1.0;
            cue
        },
        1,
        1.0,
    )
    .expect("observe");
    assert!(engine.merge(&wide).is_err());
    // Determinism: identical merges produce identical digests.
    let (mut a, mut b) = (world(0, 1, 3), world(0, 1, 3));
    let c = world(9, 2, 2);
    a.merge(&c).expect("merge");
    b.merge(&c).expect("merge");
    assert_eq!(a.state_digest(), b.state_digest());
}
