//! Creativity-layer gates (D37): imagine() as generation, KPR as
//! selection. Every creativity claim becomes an executable gate.

use atom_cognitive_engine::stacks::{idle_stack, kpr_stack, query_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig, TraceTier};

fn cue12(symbol: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[symbol % 12] = 1.0;
    cue
}

/// A cue sitting midway between two symbols (activates both).
fn mid_cue(a: usize, b: usize) -> Vec<f64> {
    let mut cue = vec![0.0; 12];
    cue[a % 12] = 1.0;
    cue[b % 12] = 1.0;
    cue
}

fn engine() -> CognitiveEngine {
    CognitiveEngine::new(EngineConfig::default()).expect("config valid")
}

/// Commit a trace through the KPR pipeline (persistent signal).
fn commit(engine: &mut CognitiveEngine, cue: &[f64], value: usize) {
    for _ in 0..4 {
        engine
            .observe(&kpr_stack(), cue, value, 1.0)
            .expect("observe");
    }
    assert!(
        engine
            .traces()
            .iter()
            .any(|t| t.cue == normalize(cue) && t.tier != TraceTier::Candidate),
        "commit helper: trace at {cue:?} must promote"
    );
}

fn normalize(cue: &[f64]) -> Vec<f64> {
    let norm: f64 = cue.iter().map(|v| v * v).sum::<f64>().sqrt();
    cue.iter().map(|v| v / norm).collect()
}

#[test]
fn gate_blends_are_candidates_not_assertions() {
    // Generation produces KPR Candidates: query-invisible, parents
    // recorded. A blend can NEVER serve an assertion until the
    // environment selects it.
    let query = query_stack();
    let mut engine = engine();
    commit(&mut engine, &cue12(0), 1);
    commit(&mut engine, &cue12(6), 2);
    let before_active = engine.active_traces().len();
    let blends = engine.imagine(&mid_cue(0, 6), 1).expect("imagine");
    assert_eq!(blends.len(), 1);
    let blend = &blends[0];
    let trace = engine.trace_by_id(blend.id).expect("blend exists");
    assert_eq!(trace.tier, TraceTier::Candidate);
    assert_eq!(trace.parents, vec![blend.parents.0, blend.parents.1]);
    assert_eq!(engine.active_traces().len(), before_active + 1);
    // The blend is invisible to queries even at its own cue.
    let outcome = engine
        .query(&query, &normalize(&mid_cue(0, 6)))
        .expect("query");
    assert!(
        !outcome.activated_all.contains(&blend.id),
        "a fresh blend must be query-invisible"
    );
}

#[test]
fn gate_blends_inherit_both_parents() {
    // Recombination, not invention: the blend's evidence carries mass at
    // BOTH parents' values, and its cue sits between them.
    let mut engine = engine();
    commit(&mut engine, &cue12(0), 1);
    commit(&mut engine, &cue12(6), 2);
    let blends = engine.imagine(&mid_cue(0, 6), 1).expect("imagine");
    let trace = engine.trace_by_id(blends[0].id).expect("blend");
    let distribution = trace.value_distribution();
    assert!(
        distribution[1] > distribution[0] && distribution[2] > distribution[0],
        "both parents' values inherited: {:?}",
        distribution
    );
    assert!(
        distribution[1] > distribution[3] && distribution[2] > distribution[3],
        "no mass at uninstantiated values"
    );
    let left = engine.trace_by_id(blends[0].parents.0).expect("parent");
    let right = engine.trace_by_id(blends[0].parents.1).expect("parent");
    let sim_left = atom_cognitive_engine::cosine_similarity(&trace.cue, &left.cue);
    let sim_right = atom_cognitive_engine::cosine_similarity(&trace.cue, &right.cue);
    assert!(
        sim_left > 0.5 && sim_right > 0.5,
        "the blend cue sits between its parents ({sim_left:.3}, {sim_right:.3})"
    );
    assert!(
        (sim_left - sim_right).abs() < 1e-9,
        "the blend is symmetric in its parents"
    );
}

#[test]
fn gate_novelty_is_measured() {
    // Novelty = distance from everything committed (parents included —
    // they ARE the accepted structure). Two isolated engines: one with
    // orthogonal parents, one with correlated parents (sim 0.5). The
    // orthogonal-parent blend must score higher novelty. (Near-parent
    // nucleation is impossible by design: an observation near an
    // existing trace BINDS it rather than creating a duplicate.)
    let mut distant_engine = engine();
    commit(&mut distant_engine, &cue12(0), 1);
    commit(&mut distant_engine, &cue12(6), 2);
    let distant = distant_engine.imagine(&mid_cue(0, 6), 1).expect("imagine");

    let mut near_engine = engine();
    // X and Y share a dimension (correlated, sim 0.5) but are distinct
    // enough to nucleate separately.
    let mut x_cue = vec![0.0; 12];
    x_cue[0] = 1.0;
    x_cue[1] = 1.0;
    let mut y_cue = vec![0.0; 12];
    y_cue[0] = 1.0;
    y_cue[2] = 1.0;
    let x = normalize(&x_cue);
    let y = normalize(&y_cue);
    commit(&mut near_engine, &x, 1);
    commit(&mut near_engine, &y, 2);
    // A query near both: the mean of X and Y.
    let mut query_cue = vec![0.0; 12];
    for (index, value) in query_cue.iter_mut().enumerate() {
        *value = (x[index] + y[index]) / 2.0;
    }
    let query_cue = normalize(&query_cue);
    let near = near_engine.imagine(&query_cue, 1).expect("imagine");

    assert!(!distant.is_empty(), "distant blend generated");
    assert!(!near.is_empty(), "near blend generated");
    assert!(
        distant[0].novelty > near[0].novelty,
        "orthogonal-parent blend must be more novel ({:.4} vs {:.4})",
        distant[0].novelty,
        near[0].novelty
    );
}

#[test]
fn gate_environment_selects_blends() {
    // THE selection test: a blend whose cue matches a recurring
    // observation pattern is selected (promotes); a blend in a dead
    // region dissipates. Blind variation, environmental retention.
    let kpr = kpr_stack();
    let mut selected = engine();
    commit(&mut selected, &cue12(0), 1);
    commit(&mut selected, &cue12(6), 2);
    let blends = selected.imagine(&mid_cue(0, 6), 1).expect("imagine");
    let blend_id = blends[0].id;
    // The environment keeps presenting evidence at the blend's cue.
    let pattern = normalize(&mid_cue(0, 6));
    for _ in 0..4 {
        selected.observe(&kpr, &pattern, 1, 1.0).expect("observe");
    }
    let trace = selected.trace_by_id(blend_id).expect("blend");
    assert!(
        trace.tier != TraceTier::Candidate,
        "a blend matching recurring evidence must be selected (promoted)"
    );

    // The dead-region control: same generation, no environmental support.
    let mut dead = engine();
    commit(&mut dead, &cue12(0), 1);
    commit(&mut dead, &cue12(6), 2);
    let blends = dead.imagine(&mid_cue(0, 6), 1).expect("imagine");
    let dead_id = blends[0].id;
    // Time passes with unrelated traffic only.
    for symbol in [9usize, 10, 11] {
        for _ in 0..4 {
            dead.observe(&kpr, &cue12(symbol), symbol % 4, 0.4)
                .expect("noise");
        }
    }
    dead.idle(&idle_stack(), 30).expect("idle");
    let trace = dead.trace_by_id(dead_id).expect("blend");
    assert!(
        trace.tier == TraceTier::Candidate || !trace.active,
        "an unsupported blend must stay uncommitted or dissipate"
    );
}

#[test]
fn gate_parents_are_never_mutated() {
    // Generation has no write authority over its own material: parents
    // byte-identical immediately before and after imagine(). (Later
    // observations decay every trace — that is engine time passing, not
    // imagine mutating; the comparison brackets the operation itself.)
    let mut engine = engine();
    commit(&mut engine, &cue12(0), 1);
    commit(&mut engine, &cue12(6), 2);
    let before: Vec<atom_cognitive_engine::MemoryTrace> = engine
        .traces()
        .iter()
        .filter(|t| t.parents.is_empty())
        .cloned()
        .collect();
    let blends = engine.imagine(&mid_cue(0, 6), 1).expect("imagine");
    let after: Vec<atom_cognitive_engine::MemoryTrace> = engine
        .traces()
        .iter()
        .filter(|t| t.parents.is_empty() && t.id != blends[0].id)
        .cloned()
        .collect();
    for parent_id in [blends[0].parents.0, blends[0].parents.1] {
        let before_parent = before.iter().find(|t| t.id == parent_id).expect("parent");
        let after_parent = after.iter().find(|t| t.id == parent_id).expect("parent");
        assert_eq!(
            before_parent, after_parent,
            "imagine must not mutate parent {parent_id}"
        );
    }
}

#[test]
fn gate_generation_is_bounded_and_validated() {
    let mut engine = engine();
    commit(&mut engine, &cue12(0), 1);
    commit(&mut engine, &cue12(6), 2);
    assert!(
        engine.imagine(&mid_cue(0, 6), 0).is_err(),
        "count 0 rejected"
    );
    assert!(
        engine.imagine(&mid_cue(0, 6), 99).is_err(),
        "count 99 rejected"
    );
    // An empty engine blends nothing: generation needs committed material.
    let mut empty = CognitiveEngine::new(EngineConfig::default()).expect("config");
    assert!(empty.imagine(&cue12(0), 1).expect("imagine").is_empty());
    // A single trace also cannot blend.
    let mut single = CognitiveEngine::new(EngineConfig::default()).expect("config");
    commit(&mut single, &cue12(0), 1);
    assert!(single.imagine(&cue12(0), 1).expect("imagine").is_empty());
}
