//! Minimal reproduction of the hops.rs failure (2026-08-30): pure
//! engine, the EXACT hops.rs cue generator and load, no field. If hop 1
//! fails here, the wall is in the engine under this load; if it passes,
//! the failure is in the hop walk itself.

use atom_cognitive_engine::stacks::{mother_query_stack, observe_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig};

const CUE_DIM: usize = 24;
const CUE_WEIGHT: usize = 4;
const MAX_OVERLAP: usize = 2;
const CHAINS: usize = 6;
const CHAIN_LEN: usize = 9;
const FACTS: usize = CHAINS * CHAIN_LEN;
const SEED: u64 = 103;

fn rng_next(state: &mut u64) -> f64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    ((x >> 11) as f64) / ((1u64 << 53) as f64)
}

fn unit(mut values: Vec<f64>) -> Vec<f64> {
    let norm: f64 = values.iter().map(|v| v * v).sum::<f64>().sqrt();
    for value in &mut values {
        *value /= norm.max(1e-9);
    }
    values
}

fn fact_cues() -> Vec<Vec<f64>> {
    let mut cues: Vec<Vec<f64>> = Vec::new();
    for index in 0..FACTS {
        let mut best: Option<(usize, Vec<f64>)> = None;
        for attempt in 0..500 {
            let mut state = SEED
                ^ (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ (attempt as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            let mut bands: Vec<usize> = (0..CUE_DIM).collect();
            for slot in 0..CUE_WEIGHT {
                let pick = slot + (rng_next(&mut state) * (CUE_DIM - slot) as f64) as usize;
                bands.swap(slot, pick.min(CUE_DIM - 1));
            }
            let mut cue = vec![0.0_f64; CUE_DIM];
            for band in bands.iter().take(CUE_WEIGHT) {
                cue[*band] = 1.0;
            }
            let worst = cues
                .iter()
                .map(|other| {
                    other
                        .iter()
                        .zip(cue.iter())
                        .filter(|(a, b)| **a > 0.0 && **b > 0.0)
                        .count()
                })
                .fold(0usize, usize::max);
            match best {
                Some((best_score, _)) if worst >= best_score => {}
                _ => best = Some((worst, cue)),
            }
            if worst <= MAX_OVERLAP {
                break;
            }
        }
        let (_, cue) = best.expect("candidate");
        cues.push(unit(cue));
    }
    cues
}

fn battery_config() -> EngineConfig {
    EngineConfig {
        cue_dim: CUE_DIM,
        value_count: 8,
        initial_ttl: 512,
        ttl_per_hit: 16,
        decay_support_rate: 0.0005,
        decay_mass_rate: 0.0003,
        capacity: 224,
        bond_expansion_depth: 8,
        ..EngineConfig::default()
    }
}

#[test]
fn hop1_under_hops_load() {
    let cues = fact_cues();
    let mut engine = CognitiveEngine::new(battery_config()).expect("config");
    let observe = observe_stack();
    let mut ids = Vec::new();
    for index in 0..FACTS {
        for _ in 0..6 {
            engine
                .observe(&observe, &cues[index], index % 8, 1.0)
                .expect("observe");
        }
        let id = engine
            .active_traces()
            .iter()
            .find(|t| {
                let dot: f64 = t.cue.iter().zip(cues[index].iter()).map(|(a, b)| a * b).sum();
                dot > 0.999_999
            })
            .map(|t| t.id)
            .unwrap_or_else(|| panic!("fact {index} trace missing"));
        ids.push(id);
    }
    // Wire the first chain only: 0-1-2-...-8.
    for pair in ids[0..CHAIN_LEN].windows(2) {
        engine.add_bond(pair[0], pair[1]).expect("bond");
    }
    let active_count = engine.active_traces().len();
    let trace0 = engine.trace_by_id(ids[0]);
    let own_cosine = trace0
        .map(|t| t.cue.iter().zip(cues[0].iter()).map(|(a, b)| a * b).sum::<f64>())
        .unwrap_or(f64::NAN);
    let reliability = trace0.map(|t| t.reliability()).unwrap_or(f64::NAN);
    println!("active traces: {active_count}; trace0 present: {}", trace0.is_some());
    println!("trace0 own cosine: {own_cosine:.4} reliability: {reliability:.4}");
    if let Some(t) = trace0 {
        println!(
            "trace0 state: support {:.4} mass {:.4} ttl {} hits {} active {} tier {:?} created_tick {} last_tick {}",
            t.support, t.mass, t.ttl, t.hits, t.active, t.tier, t.created_tick, t.last_tick
        );
    }
    let active_after: Vec<usize> = (0..9)
        .map(|i| {
            engine
                .trace_by_id(ids[i])
                .map(|t| if t.cue.iter().zip(cues[i].iter()).map(|(a, b)| a * b).sum::<f64>() > 0.999 { 1 } else { 0 })
                .unwrap_or(0)
        })
        .collect();
    println!("active flags chain0 (cosine==1 => active?): {:?}", active_after);
    for i in 0..3 {
        if let Some(t) = engine.trace_by_id(ids[i]) {
            println!(
                "trace{i}: active_in_active_traces={} reliability={:.3}",
                engine.active_traces().iter().any(|t| t.id == ids[i]),
                t.reliability()
            );
        }
    }
    let outcome = engine
        .query(&mother_query_stack(), &cues[0].clone())
        .expect("query");
    println!("ids[0..3]: {:?} {:?} {:?}", ids[0], ids[1], ids[2]);
    println!("eligible_count: {}", outcome.eligible_count);
    println!("activations: {:?}", outcome.activations);
    println!(
        "activated_all contains ids[1]: {}",
        outcome.activated_all.contains(&ids[1])
    );
    println!(
        "activated_all contains ids[8]: {}",
        outcome.activated_all.contains(&ids[8])
    );
    assert!(
        outcome.activated_all.contains(&ids[1]),
        "bond 0-1 must carry activation at hop 1"
    );
}
