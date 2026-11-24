//! Multiscale renormalization (#5, D49) — the admission gates:
//! evidence conservation, answer preservation (the anti-W17 gate:
//! merging must NOT flip answers the way dedup's copy-dropping did),
//! minority dissent surviving inside the grain, the contradiction split
//! (what was removed becomes reachable again), and the off/determinism
//! controls.

use atom_cognitive_engine::stacks::{mother_query_stack, observe_stack, query_stack, record_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

fn total_evidence(engine: &CognitiveEngine) -> f64 {
    engine
        .active_traces()
        .iter()
        .map(|t| t.evidence.iter().sum::<f64>())
        .sum()
}

/// Build a facet: `n` record-mode memories on the SAME cue (record mode
/// always nucleates, so identical cues give distinct traces), the first
/// `majority` voting value 1 and the rest voting value 2. Durable
/// regime: without it, per-observation decay biases toward whichever
/// trace was observed LAST, and the facet is not a fair majority.
fn facet_engine(n: usize, majority: usize) -> CognitiveEngine {
    let mut engine = CognitiveEngine::new(EngineConfig {
        initial_ttl: 512,
        ttl_per_hit: 16,
        decay_support_rate: 0.0005,
        decay_mass_rate: 0.0003,
        conservation_budget: 1.0e6,
        // The PREDICT root surprise-boosts observations that contradict a
        // standing prediction (salience ×1.5) — right for learning, wrong
        // for a fair majority fixture. Neutralized here so every facet
        // member carries identical evidence.
        surprise_boost: 1.0,
        capacity: 32,
        ..EngineConfig::default()
    })
    .expect("config");
    let record = record_stack();
    for i in 0..n {
        let value = if i < majority { 1 } else { 2 };
        engine
            .observe(&record, &cue12(0), value, 1.0)
            .expect("record observe");
    }
    engine
}

#[test]
fn gate_evidence_conserved_and_answer_preserved() {
    // Five same-cue memories, 3 voting value 1 and 2 voting value 2.
    let mut engine = facet_engine(5, 3);
    let query = query_stack();
    let before = engine.query(&query, &cue12(0)).expect("query");
    let evidence_before = total_evidence(&engine);
    assert_eq!(
        before.claim.map(|(v, _)| v),
        Some(1),
        "setup: the majority must answer before merging"
    );
    let report = engine.renormalize(1.0).expect("renormalize");
    assert_eq!(report.grains, 1);
    assert_eq!(report.traces_merged, 4);
    // The conserved quantity: total active evidence is EXACTLY the same.
    let evidence_after = total_evidence(&engine);
    assert!(
        (evidence_before - evidence_after).abs() < 1e-12,
        "evidence must be conserved ({evidence_before} -> {evidence_after})"
    );
    assert_eq!(engine.active_traces().len(), 1, "one grain answers alone");
    // The anti-W17 gate: merging must not flip the answer (dedup's
    // copy-dropping flipped 4/11 Mother facets; merging holds).
    let after = engine.query(&query, &cue12(0)).expect("query");
    assert_eq!(
        after.claim.map(|(v, _)| v),
        Some(1),
        "coarse-graining must preserve the majority answer"
    );
}

#[test]
fn gate_minority_dissent_survives_inside_the_grain() {
    // 3 × value 1 + 1 × value 2: the merged distribution keeps ~25% of
    // its mass on value 2. Coarse-graining is honest — proportions are
    // preserved, not crushed into the mode.
    let mut engine = facet_engine(4, 3);
    engine.renormalize(1.0).expect("renormalize");
    let grain = &engine.active_traces()[0];
    let distribution = grain.value_distribution();
    assert!(
        (distribution[2] - 0.25).abs() < 0.02,
        "minority mass must survive (value-2 share {})",
        distribution[2]
    );
    assert!(!grain.grain_members.is_empty());
    assert_eq!(grain.grain_members.len(), 3);
}

#[test]
fn gate_contradiction_splits_the_grain() {
    // The rollback: after coarse-graining, a contradicting observation
    // binds to the grain (identical cue), takes the revise path, and the
    // grain SPLITS — members re-activate with their frozen evidence and
    // the falsified coarse description dies.
    let mut engine = facet_engine(4, 3);
    engine.renormalize(1.0).expect("renormalize");
    assert_eq!(engine.active_traces().len(), 1);
    let observe = observe_stack();
    let outcome = engine
        .observe(&observe, &cue12(0), 0, 1.0)
        .expect("contradicting observe");
    assert_eq!(outcome.action, "revise");
    // The ABSORBED members re-activate and the falsified grain dies:
    // n traces minus the seed grain leaves n − 1 active.
    assert_eq!(engine.active_traces().len(), 3);
    for trace in engine.active_traces() {
        assert!(
            trace.grain_members.is_empty(),
            "no trace may still be a grain after the split"
        );
    }
    // And the fine scale keeps answering with the majority.
    let query = query_stack();
    let after = engine.query(&query, &cue12(0)).expect("query");
    assert_eq!(after.claim.map(|(v, _)| v), Some(1));
}

#[test]
fn gate_bonds_rewire_to_the_grain() {
    // Connectivity survives re-description: a bond touching a member
    // follows it into the grain (self-loops and duplicates drop).
    let mut engine = CognitiveEngine::new(EngineConfig::default()).expect("config");
    let record = record_stack();
    engine.observe(&record, &cue12(0), 1, 1.0).expect("observe");
    engine.observe(&record, &cue12(0), 1, 1.0).expect("observe");
    engine.observe(&record, &cue12(5), 1, 1.0).expect("observe");
    let ids: Vec<u64> = engine.traces().iter().map(|t| t.id).collect();
    // second same-cue trace bonded to the distant trace
    engine.add_bond(ids[1], ids[2]).expect("bond");
    engine.renormalize(1.0).expect("renormalize");
    // The member's bond now points at the grain (the first same-cue id).
    assert_eq!(engine.bond_count(), 1);
    // Expansion (the mother stack carries the expand stage) reaches the
    // grain through the rewired bond.
    let outcome = engine
        .query(&mother_query_stack(), &cue12(5))
        .expect("query");
    assert!(outcome.activated_all.iter().any(|id| *id == ids[0]));
}

#[test]
fn gate_off_is_byte_identical_validation_and_determinism() {
    let mut engine = facet_engine(4, 3);
    let hash_before = engine
        .transitions()
        .last()
        .expect("observed")
        .state_hash
        .clone();
    // 1.01 = off: nothing merges, state byte-identical, zero report.
    let report = engine.renormalize(1.01).expect("off renormalize");
    assert_eq!(report.grains, 0);
    assert_eq!(report.traces_merged, 0);
    let hash_after = engine
        .transitions()
        .last()
        .expect("observed")
        .state_hash
        .clone();
    assert_eq!(hash_before, hash_after, "off must not touch the state");
    // Validation: out-of-range granularity rejected fail-closed.
    assert!(engine.renormalize(1.5).is_err());
    assert!(engine.renormalize(0.3).is_err());
    // Determinism: same construction, same renormalization, same hash.
    let mut left = facet_engine(4, 3);
    let mut right = facet_engine(4, 3);
    left.renormalize(1.0).expect("renormalize");
    right.renormalize(1.0).expect("renormalize");
    let left_hash = left
        .transitions()
        .last()
        .expect("renormalized")
        .state_hash
        .clone();
    let right_hash = right
        .transitions()
        .last()
        .expect("renormalized")
        .state_hash
        .clone();
    assert_eq!(left_hash, right_hash);
}
