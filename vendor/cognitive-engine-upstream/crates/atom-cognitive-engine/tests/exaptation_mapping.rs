//! Catalytic routing / exaptation (#16, D53) — the ADMISSION VERDICT
//! suite. The protocol's novelty bar asks whether the candidate
//! contributes something no existing primitive expresses. It does not:
//! its two named capabilities are each already expressed, and these
//! gates are the receipts.
//!
//! 1. "Influence pathways without becoming evidence" = the stigmergy
//!    routing-not-evidence law (D52), re-proven here in the recruitment
//!    geometry.
//! 2. "Recruit old structure for new use" = echo-bond expansion: a
//!    memory built in one facet answers a question posed in another,
//!    through routing — its evidence votes, its trace never moves, and
//!    nothing is copied into the asking facet. imagine() (D38) records
//!    the same recruitment as parent provenance for blends.
//! 3. The risk "hidden routing bias" is barred by construction: the
//!    recruited member is CITED in the activations and rides in the
//!    response hash.
//!
//! The one candidate mechanism that was NOT already expressed — filtering
//! "hub" members out of the mixture — cannot distinguish a hub from the
//! bridged answer without an oracle (both sit at low query similarity
//! behind the bond), so it is rejected rather than guessed.

use atom_cognitive_engine::stacks::{mother_query_stack, observe_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig, QueryTuning};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

/// Facet A (the old structure, heavily reinforced, value 1) and facet B
/// (the asking facet, value 2), bridged B → A.
fn bridged_world() -> CognitiveEngine {
    let mut engine = CognitiveEngine::new(EngineConfig {
        initial_ttl: 512,
        ttl_per_hit: 16,
        decay_support_rate: 0.0005,
        decay_mass_rate: 0.0003,
        conservation_budget: 1.0e6,
        capacity: 16,
        ..EngineConfig::default()
    })
    .expect("config");
    let observe = observe_stack();
    // The old structure: twelve reinforcements make it the dominant
    // voice wherever it is reached.
    for _ in 0..12 {
        engine
            .observe(&observe, &cue12(0), 1, 1.0)
            .expect("observe");
    }
    // The asking facet: lightly supported, value 2.
    for _ in 0..2 {
        engine
            .observe(&observe, &cue12(1), 2, 1.0)
            .expect("observe");
    }
    let ids: Vec<u64> = engine.traces().iter().map(|t| t.id).collect();
    engine.add_bond(ids[1], ids[0]).expect("bridge");
    engine
}

#[test]
fn gate_old_structure_joins_a_new_question() {
    // The question is posed in facet B's cue space. The asking facet
    // governs its own answer (order preserved — a bridged member never
    // outvotes the direct anchor under attention math), but the OLD
    // STRUCTURE joins the composition with a meaningful, cited voice:
    // its evidence votes in a question it was never observed for.
    let engine = bridged_world();
    let traces_before = engine.traces().to_vec();
    let outcome = engine
        .query(&mother_query_stack(), &cue12(1))
        .expect("query");
    assert_eq!(
        outcome.claim.map(|(v, _)| v),
        Some(2),
        "the asking facet governs its own question"
    );
    // Cited, never hidden: A appears in the activations with weight.
    let recruited = engine.traces().first().expect("old structure").id;
    let weight = outcome
        .activations
        .iter()
        .find(|(id, _)| *id == recruited)
        .map(|(_, weight)| *weight)
        .unwrap_or(0.0);
    assert!(
        weight >= 0.10,
        "the recruited structure must hold a meaningful voice (weight {weight})"
    );
    // Influence without becoming evidence and without duplication: every
    // trace (both facets) is byte-identical after the query.
    assert_eq!(
        engine.traces(),
        traces_before.as_slice(),
        "recruitment is routing — no trace may move, no evidence may be copied"
    );
}

#[test]
fn gate_marked_routes_strengthen_recruitment_without_touching_evidence() {
    // The catalytic core, proven as stigmergy in the recruitment
    // geometry: laying a trail toward the old structure strengthens its
    // recruitment weight while every trace stays byte-identical.
    let mut engine = bridged_world();
    let query = mother_query_stack();
    let traces_before = engine.traces().to_vec();
    let tuning = QueryTuning {
        stigmergy_rate: 0.5,
        stigmergy_gain: 1.0,
        ..QueryTuning::from_config(engine.config())
    };
    let mut plain_weight = 0.0;
    for _ in 0..10 {
        let outcome = engine
            .query_marking(&query, &cue12(1), tuning)
            .expect("marking query");
        let recruited = engine.traces().first().unwrap().id;
        plain_weight = outcome
            .outcome
            .activations
            .iter()
            .find(|(id, _)| *id == recruited)
            .map(|(_, w)| *w)
            .expect("recruited member reached");
    }
    assert_eq!(
        engine.traces(),
        traces_before.as_slice(),
        "influence pathways never become evidence (D52 law, recruitment geometry)"
    );
    // The trail raised the recruited member's weight above what the
    // unmarked route yields in an identical control.
    let control = bridged_world();
    let control_outcome = control
        .query_tuned(&query, &cue12(1), tuning)
        .expect("control query");
    let recruited = control.traces().first().unwrap().id;
    let control_weight = control_outcome
        .activations
        .iter()
        .find(|(id, _)| *id == recruited)
        .map(|(_, w)| *w)
        .expect("control reaches the recruited member");
    assert!(
        plain_weight > control_weight,
        "the marked pathway must strengthen recruitment ({plain_weight} vs {control_weight})"
    );
}

#[test]
fn gate_recruitment_visibility_rides_in_the_hash() {
    // The anti-hidden-bias receipt: routing that changes the reached set
    // changes the RESPONSE HASH — recruitment cannot happen in secret.
    let bridged = bridged_world();
    let with = bridged
        .query(&mother_query_stack(), &cue12(1))
        .expect("bridged query");
    // Identical world without the bridge: no recruitment, different
    // activation set, different hash.
    let mut unbridged = bridged_world();
    unbridged.clear_bonds();
    let without = unbridged
        .query(&mother_query_stack(), &cue12(1))
        .expect("unbridged query");
    assert_ne!(
        with.response_hash, without.response_hash,
        "recruited members ride in the response hash"
    );
    assert_eq!(
        without.claim.map(|(v, _)| v),
        Some(2),
        "without the bridge the asking facet answers from itself"
    );
}
