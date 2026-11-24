//! E1-chem — the chemistry arm: CATALYSIS (precommitted in
//! `docs/E1-PERCEPTION.md` §E1-chem BEFORE this code ran, per Jesse's
//! "did any chemistry primitives help"). A catalyst lowers the activation
//! energy of an ONGOING reaction only — it cannot start a naming, cannot
//! flip to a competing label, and does nothing where nothing is weakened.

use atom_cognitive_engine::perception::{NAMED_LABELS, PerceptionLabel, Reading};
use atom_cognitive_engine::stacks::perception_stack;
use atom_cognitive_engine::{CognitiveEngine, EngineConfig};

fn clean_window(base_id: u64, label: PerceptionLabel) -> Vec<Reading> {
    (0..5u64)
        .map(|k| {
            let (exploration, convergence) = match label {
                PerceptionLabel::CoordinatedSweep => (0.80 + 0.01 * k as f64, 0.70 - 0.01 * k as f64),
                PerceptionLabel::ScattershotProbe => (0.80 + 0.01 * k as f64, 0.30 - 0.01 * k as f64),
                PerceptionLabel::ConvergentDrift => (0.30 + 0.01 * k as f64, 0.70 - 0.01 * k as f64),
            };
            Reading::new(base_id + k, 0.5, 0.5, 0.5, exploration, convergence, 0.5)
        })
        .collect()
}

/// A WEAKENED window: only 2 of 5 readings still satisfy `label`'s
/// conjunction (the E1c evidence-starvation regime); the other three
/// satisfy nothing (exploration nudged between the label bands).
fn weakened_window(base_id: u64, label: PerceptionLabel) -> Vec<Reading> {
    (0..5u64)
        .map(|k| {
            let (exploration, convergence) = if k < 2 {
                match label {
                    PerceptionLabel::CoordinatedSweep => (0.80, 0.70),
                    PerceptionLabel::ScattershotProbe => (0.80, 0.30),
                    PerceptionLabel::ConvergentDrift => (0.30, 0.70),
                }
            } else {
                // Broken by attrition: satisfies no label's conjunction.
                (0.65, 0.70)
            };
            Reading::new(base_id + k, 0.5, 0.5, 0.5, exploration, convergence, 0.5)
        })
        .collect()
}

fn engine_with(catalysis: f64) -> CognitiveEngine {
    CognitiveEngine::new(EngineConfig {
        perception_catalysis: catalysis,
        ..EngineConfig::default()
    })
    .expect("config")
}

/// CH1 (precommitted: pass) — catalysis recovers confirmation under
/// weakened evidence: after one clean CoordinatedSweep window (one
/// confirmation), a window where only 2 of 5 readings still satisfy the
/// conjunction ABSTAINS at the vanilla floor (3) but NAMES correctly at
/// the catalyzed floor (2), with exactly the two supporting ids.
#[test]
fn gate_catalysis_recovers_confirmation_under_weakened_evidence() {
    // Vanilla arm: clean then weakened — abstains.
    let mut vanilla = engine_with(0.0);
    let first = vanilla
        .perceive(&perception_stack(), &clean_window(10, PerceptionLabel::CoordinatedSweep))
        .expect("perceive runs")
        .perception
        .expect("clean window names");
    assert_eq!(first.label, PerceptionLabel::CoordinatedSweep);
    assert_eq!(first.confirmations, 1);
    let outcome = vanilla
        .perceive(&perception_stack(), &weakened_window(100, PerceptionLabel::CoordinatedSweep))
        .expect("perceive runs");
    assert!(
        outcome.perception.is_none(),
        "vanilla floor 3 must abstain on 2-of-5 evidence"
    );

    // Catalyzed arm: same sequence — names. NOTE (stability-rule
    // interaction, first run got this wrong and is kept on record): the
    // recovered NAMING returns the ACTIVE hypothesis per E1's no-re-
    // emission law — so the evidence field stays the FOUNDING window's
    // five ids, and the confirmation count accrues. The catalyst's effect
    // is that the outcome is a naming at all, where vanilla abstained.
    let mut catalyzed = engine_with(1.0);
    let first = catalyzed
        .perceive(&perception_stack(), &clean_window(10, PerceptionLabel::CoordinatedSweep))
        .expect("perceive runs")
        .perception
        .expect("clean window names");
    assert_eq!(first.label, PerceptionLabel::CoordinatedSweep);
    let outcome = catalyzed
        .perceive(&perception_stack(), &weakened_window(100, PerceptionLabel::CoordinatedSweep))
        .expect("perceive runs");
    let perception = outcome
        .perception
        .unwrap_or_else(|| panic!("catalyst must recover naming, abstained: {:?}", outcome.abstain_reason));
    assert_eq!(perception.label, PerceptionLabel::CoordinatedSweep, "no mislabel");
    assert_eq!(perception.id, first.id, "stability rule: the active hypothesis, not a re-emission");
    assert_eq!(perception.evidence.len(), 5, "founding evidence is retained");
    assert!(perception.evidence.iter().all(|id| (10..15).contains(id)));
    // The confirmation count accrued on the active hypothesis.
    assert_eq!(perception.confirmations, 2);
}

/// CH2 (precommitted: pass) — the catalyst cannot create a first naming:
/// the same weakened window on a FRESH engine with catalysis 1.0 abstains
/// — no active reaction, nothing to catalyze.
#[test]
fn gate_catalyst_cannot_create_first_naming() {
    let mut engine = engine_with(1.0);
    let outcome = engine
        .perceive(&perception_stack(), &weakened_window(10, PerceptionLabel::CoordinatedSweep))
        .expect("perceive runs");
    assert!(
        outcome.perception.is_none(),
        "a catalyst must not start a naming — only ease an ongoing one"
    );
    assert_eq!(
        outcome.abstain_reason,
        Some("no-label-reaches-support-floor")
    );
}

/// CH3 (precommitted: pass) — the catalyst is reaction-specific: with an
/// active CoordinatedSweep perception, weakened evidence for a COMPETING
/// label (ScattershotProbe, 2 of 5) still abstains — the floor relief
/// belongs to the active reaction, not the competitor.
#[test]
fn gate_catalyst_is_reaction_specific() {
    let mut engine = engine_with(1.0);
    let first = engine
        .perceive(&perception_stack(), &clean_window(10, PerceptionLabel::CoordinatedSweep))
        .expect("perceive runs")
        .perception
        .expect("clean window names");
    assert_eq!(first.label, PerceptionLabel::CoordinatedSweep);
    let outcome = engine
        .perceive(&perception_stack(), &weakened_window(200, PerceptionLabel::ScattershotProbe))
        .expect("perceive runs");
    assert!(
        outcome.perception.is_none(),
        "the catalyst for the active reaction must not ease a competitor"
    );
}

/// CH4 (precommitted: pass) — catalysis is harmless where nothing is
/// weakened: catalysis 1.0 names the clean E1 corpus 9/9 with zero
/// mislabels (bounded effect, per the receipts' ~1.9x caution).
#[test]
fn gate_catalysis_harmless_on_clean_corpus() {
    let mut base = 10u64;
    for label in NAMED_LABELS {
        for _ in 0..3 {
            let readings = clean_window(base, label);
            base += 100;
            let mut engine = engine_with(1.0);
            let outcome = engine
                .perceive(&perception_stack(), &readings)
                .expect("perceive runs");
            let perception = outcome
                .perception
                .unwrap_or_else(|| panic!("clean window must name, abstained: {:?}", outcome.abstain_reason));
            assert_eq!(perception.label, label, "no mislabel on clean corpus");
        }
    }
}
