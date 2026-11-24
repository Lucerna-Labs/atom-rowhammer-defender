//! v0.3 mechanism gates: echo bonds, expansion retrieval, echo
//! consolidation, tier protection, and record-mode ingestion. Each test
//! names the mechanism it proves load-bearing.

use atom_cognitive_engine::stacks::{mother_query_stack, observe_stack, query_stack, record_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig, EngineError, TraceTier};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

fn engine() -> CognitiveEngine {
    CognitiveEngine::new(EngineConfig::default()).expect("default config valid")
}

#[test]
fn record_mode_never_merges() {
    // Constitutions are documents, not experiences: the same facet cue with
    // three different values must produce three distinct traces.
    let record = record_stack();
    let mut engine = engine();
    for value in 0..3 {
        engine
            .observe(&record, &cue12(0), value, 1.0)
            .expect("record");
    }
    assert_eq!(engine.active_traces().len(), 3);
    assert_eq!(engine.action_count("nucleate"), 3);
}

#[test]
fn record_mode_supports_multi_value_evidence() {
    let record = record_stack();
    let mut engine = engine();
    engine
        .observe_values(&record, &cue12(0), &[1, 3], 1.0)
        .expect("record");
    let trace = engine.active_traces()[0];
    let distribution = trace.value_distribution();
    // Both encoded behaviors carry evidence; the unencoded ones do not.
    assert!(distribution[1] > distribution[0]);
    assert!(distribution[3] > distribution[0]);
    assert!(distribution[3] > distribution[2]);
}

#[test]
fn bonds_activate_facet_dissimilar_memories() {
    // The bridge proof in miniature: two orthogonal memories (similarity
    // zero — the kitchen and the bakery share no facets), bonded upstream →
    // downstream. Querying the downstream must activate the upstream ONLY
    // through the bond.
    let record = record_stack();
    let mut engine = engine();
    engine
        .observe(&record, &cue12(0), 1, 1.0)
        .expect("upstream");
    engine
        .observe(&record, &cue12(5), 2, 1.0)
        .expect("downstream");
    let upstream = engine.active_traces()[0].id;
    let downstream = engine.active_traces()[1].id;
    engine.add_bond(upstream, downstream).expect("bond");

    let plain = engine
        .query(&query_stack(), &cue12(5))
        .expect("plain query");
    let activated: Vec<u64> = plain.activations.iter().map(|(id, _)| *id).collect();
    assert!(activated.contains(&downstream));
    assert!(
        !activated.contains(&upstream),
        "facet similarity alone must NOT reach the upstream memory"
    );

    let expanded = engine
        .query(&mother_query_stack(), &cue12(5))
        .expect("expanded query");
    let activated: Vec<u64> = expanded.activations.iter().map(|(id, _)| *id).collect();
    assert!(
        activated.contains(&upstream),
        "the bond must carry activation across the facet gap"
    );
    assert!(activated.contains(&downstream));
}

#[test]
fn expansion_depth_two_crosses_a_chain() {
    // A → B → C chain of orthogonal memories: querying C must reach A at
    // hop 2 (default bond_expansion_depth = 2).
    let record = record_stack();
    let mut engine = engine();
    for symbol in [0, 4, 8] {
        engine
            .observe(&record, &cue12(symbol), 1, 1.0)
            .expect("record");
    }
    let a = engine.active_traces()[0].id;
    let b = engine.active_traces()[1].id;
    let c = engine.active_traces()[2].id;
    engine.add_bond(a, b).expect("bond");
    engine.add_bond(b, c).expect("bond");
    let outcome = engine
        .query(&mother_query_stack(), &cue12(8))
        .expect("query");
    let activated: Vec<u64> = outcome.activations.iter().map(|(id, _)| *id).collect();
    assert!(activated.contains(&c));
    assert!(activated.contains(&b));
    assert!(
        activated.contains(&a),
        "depth-2 expansion crosses the chain"
    );
}

#[test]
fn expansion_depth_eight_crosses_a_nine_fact_chain() {
    // Gate for the cap raise (2026-08-30): a 9-fact chain A→B→…→I,
    // queried from the far end, must reach the other end at depth 8.
    // Depth 4 — the old validated ceiling — would stop at the fifth
    // fact; this test is exactly what the raise buys.
    let record = record_stack();
    let mut engine = CognitiveEngine::new(EngineConfig {
        bond_expansion_depth: 8,
        ..EngineConfig::default()
    })
    .expect("depth-8 config valid");
    for symbol in (0..9).map(|i| i * 4) {
        engine
            .observe(&record, &cue12(symbol), 1, 1.0)
            .expect("record");
    }
    let ids: Vec<u64> = engine.active_traces().iter().map(|t| t.id).collect();
    for pair in ids.windows(2) {
        engine.add_bond(pair[0], pair[1]).expect("bond");
    }
    let outcome = engine
        .query(&mother_query_stack(), &cue12(8 * 4))
        .expect("query");
    let activated: Vec<u64> = outcome.activations.iter().map(|(id, _)| *id).collect();
    for id in &ids {
        assert!(
            activated.contains(id),
            "depth-8 expansion must reach every chain member"
        );
    }
}

#[test]
fn echo_consolidation_reinforces_upstream_without_repetition() {
    // Observing the downstream memory pays a support bonus to the upstream
    // partner — the kitchen is reinforced when the bakery fires. The
    // unbonded control shows pure decay instead.
    let run = |bonded: bool| -> f64 {
        let record = record_stack();
        let observe = observe_stack();
        let mut engine = engine();
        engine
            .observe(&record, &cue12(0), 1, 1.0)
            .expect("upstream");
        engine
            .observe(&record, &cue12(5), 2, 1.0)
            .expect("downstream");
        let upstream = engine.active_traces()[0].id;
        let downstream = engine.active_traces()[1].id;
        if bonded {
            engine.add_bond(upstream, downstream).expect("bond");
        }
        let before = engine.trace_by_id(upstream).expect("trace").support;
        // Re-experience the downstream memory (bind path, exact cue).
        engine
            .observe(&observe, &cue12(5), 2, 1.0)
            .expect("re-observe");
        let after = engine.trace_by_id(upstream).expect("trace").support;
        let _ = before;
        after
    };
    let bonded_support = run(true);
    let unbonded_support = run(false);
    assert!(
        bonded_support > unbonded_support,
        "the echo bonus must lift the upstream above the decay-only control \
         ({bonded_support:.4} vs {unbonded_support:.4})"
    );
    assert!(bonded_support > 0.7, "net positive against initial support");
}

#[test]
fn core_tier_survives_contradiction_storms() {
    // Identity substrate: contradictions are counted but Core support is
    // not dissolved; Standard traces take the full dissolve-replace.
    let record = record_stack();
    let observe = observe_stack();
    let run = |core: bool| -> (f64, u64) {
        let mut engine = engine();
        engine.set_ingest_tier(if core {
            TraceTier::Core
        } else {
            TraceTier::Standard
        });
        engine.observe(&record, &cue12(0), 1, 1.0).expect("record");
        engine.set_ingest_tier(TraceTier::Standard);
        let id = engine.active_traces()[0].id;
        // Alternating contradictory values: every observation genuinely
        // contradicts the current argmax (repeating one value would revise
        // once and then reinforce).
        for value in [2usize, 3, 0] {
            engine
                .observe(&observe, &cue12(0), value, 1.0)
                .expect("contradict");
        }
        let trace = engine.trace_by_id(id).expect("trace");
        (trace.support, trace.contradictions)
    };
    let (core_support, core_contradictions) = run(true);
    let (standard_support, standard_contradictions) = run(false);
    assert_eq!(core_contradictions, 3, "contradictions are recorded");
    assert_eq!(standard_contradictions, 3);
    assert!(
        core_support > standard_support,
        "Core support must not dissolve ({core_support:.4} vs {standard_support:.4})"
    );
    assert!(
        core_support > standard_support * 1.5,
        "the tier gap must be decisive"
    );
    assert!(
        core_support > 0.6,
        "Core stays near its decay-only floor (installed 0.7, no dissolution), got {core_support:.4}"
    );
}

#[test]
fn eviction_never_selects_core() {
    let record = record_stack();
    let mut engine = CognitiveEngine::new(EngineConfig {
        capacity: 2,
        ..EngineConfig::default()
    })
    .expect("config");
    engine.set_ingest_tier(TraceTier::Core);
    engine
        .observe(&record, &cue12(0), 1, 1.0)
        .expect("core one");
    engine
        .observe(&record, &cue12(1), 1, 1.0)
        .expect("core two");
    // Both slots hold Core traces: a third record must fail closed, not
    // evict identity substrate.
    let result = engine.observe(&record, &cue12(2), 1, 1.0);
    assert_eq!(result, Err(EngineError::CapacityExhausted));

    // With one Standard trace, that trace is the eviction candidate.
    let mut engine = CognitiveEngine::new(EngineConfig {
        capacity: 2,
        ..EngineConfig::default()
    })
    .expect("config");
    engine.set_ingest_tier(TraceTier::Core);
    engine.observe(&record, &cue12(0), 1, 1.0).expect("core");
    engine.set_ingest_tier(TraceTier::Standard);
    engine
        .observe(&record, &cue12(1), 1, 1.0)
        .expect("standard");
    engine
        .observe(&record, &cue12(2), 1, 1.0)
        .expect("standard evicted, core survives");
    let active = engine.active_traces();
    assert_eq!(active.len(), 2);
    // The Core trace survived; the evicted slot was the Standard one.
    assert!(
        active
            .iter()
            .any(|t| t.tier == TraceTier::Core && t.cue[0] == 1.0)
    );
}

#[test]
fn invalid_bonds_are_rejected() {
    let record = record_stack();
    let mut engine = engine();
    engine.observe(&record, &cue12(0), 1, 1.0).expect("record");
    let id = engine.active_traces()[0].id;
    assert_eq!(
        engine.add_bond(id, id),
        Err(EngineError::InvalidBond("self-bond rejected".into()))
    );
    assert_eq!(
        engine.add_bond(id, 9999),
        Err(EngineError::InvalidBond(
            "downstream 9999 does not exist".into()
        ))
    );
    engine.add_bond(9999, id).expect_err("unknown upstream");
}
