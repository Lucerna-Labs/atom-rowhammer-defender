//! Frozen method-selection gate with a matched permission rule and short chaos arm.
use atom_cognitive_engine::stacks::{observe_stack, query_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig, QueryTuning};
use ram_monitor_cognitive::{
    encode, CognitiveRepairSelector, RepairContext, RepairMethod, SelectionSource,
};

fn context(file_mode: u32, parent_mode: u32, difference: usize, origin: u64) -> RepairContext {
    RepairContext {
        different_bytes: difference,
        object_changed: false,
        file_mode,
        parent_mode,
        atom_long_mean: difference as f64 * 0.000001,
        prior_alerts: 0,
        origin,
    }
}

fn engine() -> Result<CognitiveEngine, Box<dyn std::error::Error>> {
    let cfg = EngineConfig {
        capacity: 64,
        cue_dim: 12,
        value_count: 2,
        conservation_budget: 64.0,
        initial_ttl: 32,
        ..EngineConfig::default()
    };
    Ok(CognitiveEngine::new(cfg)?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = context(0o644, 0o555, 1, 1);
    let b = context(0o444, 0o755, 1, 2);
    let mut selector = CognitiveRepairSelector::new()?;
    let blank = selector.select(a)?;
    if blank.method != RepairMethod::Atomic || blank.source != SelectionSource::AbstentionFallback {
        return Err("blank retrieval did not take safe fallback".into());
    }
    selector.record_verified_success(a, RepairMethod::InPlace)?;
    selector.record_verified_success(b, RepairMethod::Atomic)?;
    println!("group,case,method,source,expected,static_rule,atomic_first_fails,supporters");
    let mut right = 0;
    let mut static_right = 0;
    let mut atomic_first_failures = 0;
    for (group, file, parent, expected) in [
        ("readonly-parent", 0o644, 0o555, RepairMethod::InPlace),
        ("readonly-file", 0o444, 0o755, RepairMethod::Atomic),
    ] {
        for index in 2..=17 {
            let cue = context(file, parent, index, 100 + index as u64);
            let selected = selector.select(cue)?;
            let static_rule = if file & 0o200 != 0 && parent & 0o200 == 0 {
                RepairMethod::InPlace
            } else {
                RepairMethod::Atomic
            };
            right += usize::from(selected.method == expected);
            static_right += usize::from(static_rule == expected);
            atomic_first_failures += usize::from(expected == RepairMethod::InPlace);
            println!(
                "{group},{index},{:?},{:?},{expected:?},{static_rule:?},{},{}",
                selected.method,
                selected.source,
                expected == RepairMethod::InPlace,
                selected.supporting_traces.len()
            );
        }
    }
    let forced = selector.select(RepairContext {
        object_changed: true,
        ..a
    })?;
    if forced.method != RepairMethod::Atomic || forced.source != SelectionSource::HardObjectGate {
        return Err("object change escaped hard gate".into());
    }
    let mut conflict = CognitiveRepairSelector::new()?;
    conflict.record_verified_success(context(0o644, 0o555, 1, 11), RepairMethod::InPlace)?;
    conflict.record_verified_success(context(0o644, 0o555, 1, 22), RepairMethod::Atomic)?;
    let contested = conflict.select(context(0o644, 0o555, 1, 33))?;
    if contested.method != RepairMethod::Atomic
        || contested.source != SelectionSource::AbstentionFallback
    {
        return Err("opposite origins did not fall back".into());
    }

    let mut off = engine()?;
    off.observe_with_origin(&observe_stack(), &encode(a)?, &[0], 1.0, 1)?;
    off.observe_with_origin(&observe_stack(), &encode(b)?, &[1], 1.0, 2)?;
    let mut chaos_correct = 0;
    let mut plain_correct = 0;
    let mut chaos_selected = 0;
    for seed in 1..=3 {
        let mut logistic = 0.1 + 0.2 * seed as f64;
        let mut tuned = QueryTuning::from_config(off.config());
        for index in 2..=7 {
            logistic = 4.0 * logistic * (1.0 - logistic);
            let cue = context(0o644, 0o555, index, 1000 + seed * 10 + index as u64);
            let vector = encode(cue)?;
            let plain = off.query_tuned(&query_stack(), &vector, tuned)?;
            let mut variant = tuned;
            variant.attention_temperature *= 1.0 + 0.05 * (2.0 * logistic - 1.0);
            let candidate = off.query_tuned(&query_stack(), &vector, variant)?;
            let valid = candidate.claim == plain.claim && candidate.fluency.margin.is_finite();
            let keep = valid && candidate.fluency.margin > plain.fluency.margin + 1e-12;
            let chosen = if keep {
                tuned = variant;
                chaos_selected += 1;
                candidate
            } else {
                plain.clone()
            };
            plain_correct += usize::from(plain.claim.map(|(v, _)| v) == Some(0));
            chaos_correct += usize::from(chosen.claim.map(|(v, _)| v) == Some(0));
        }
    }
    eprintln!("heldout={right}/32 fixed_atomic_first_fails={atomic_first_failures} static_rule={static_right}/32 blank_fallback=pass object_gate=pass contradictory_fallback=pass chaos_plain={plain_correct}/18 chaos_on={chaos_correct}/18 selected={chaos_selected}");
    Ok(())
}
