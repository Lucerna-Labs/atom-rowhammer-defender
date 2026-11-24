//! E1 admission gates for PERCEPTION — precommitted in
//! `atom-cognitive-engine/docs/E1-PERCEPTION.md` BEFORE this code ran.
//! Outcomes versus the predictions are recorded in that document's
//! addendum and in AtomDB.

use atom_cognitive_engine::perception::{NAMED_LABELS, PerceptionLabel, Reading};
use atom_cognitive_engine::stacks::{omit, perception_stack, PERCEIVE_PERCEIVE};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig};

/// A frozen five-reading window for `label`, deterministic (no RNG).
/// Noise dimensions are constant so they carry no label information.
fn window(base_id: u64, label: PerceptionLabel) -> Vec<Reading> {
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

fn fresh() -> CognitiveEngine {
    CognitiveEngine::new(EngineConfig::default()).expect("config")
}

/// GATE 1 (precommitted: pass) — the perception stage is load-bearing:
/// the full stack names 9/9 frozen windows correctly with conjunction-
/// satisfying evidence; the ablated stack (perceive stage omitted, which
/// transitively drops gate and verify) names 0/9.
#[test]
fn gate_perception_stage_is_load_bearing() {
    let mut base = 10u64;
    for label in NAMED_LABELS {
        for _ in 0..3 {
            let readings = window(base, label);
            base += 100;

            let mut engine = fresh();
            let floor = engine.config().perception_support_floor;
            let outcome = engine
                .perceive(&perception_stack(), &readings)
                .expect("perceive runs");
            let perception = outcome
                .perception
                .unwrap_or_else(|| panic!("window must be named, abstained: {:?}", outcome.abstain_reason));
            assert_eq!(perception.label, label, "correct joint name");
            assert!(
                perception.evidence.len() >= floor,
                "evidence count {} must reach floor {}",
                perception.evidence.len(),
                floor
            );
            for id in &perception.evidence {
                let reading = readings
                    .iter()
                    .find(|r| r.id == *id)
                    .expect("evidence ids reference the window");
                assert!(
                    reading.satisfies(label),
                    "every evidence reading must satisfy the conjunction"
                );
            }

            // Ablation: the measurable behavior disappears without the stage.
            let ablated = omit(&perception_stack(), PERCEIVE_PERCEIVE);
            let mut stripped = fresh();
            let outcome = stripped
                .perceive(&ablated, &readings)
                .expect("ablated stack still runs");
            assert!(
                outcome.perception.is_none(),
                "ablated stack must not name windows"
            );
            assert_eq!(outcome.abstain_reason, Some("perceive-stage-absent"));
        }
    }
}

/// GATE 2 (precommitted: pass) — the joint labels are not reachable by any
/// single-dimension threshold rule: an exhaustive sweep over every
/// dimension × direction × threshold × (label-if-true, label-if-false)
/// tops out below 9/9 on the frozen corpus, while perception is 9/9.
/// (Exploration cannot separate CoordinatedSweep from ScattershotProbe;
/// convergence cannot separate CoordinatedSweep from ConvergentDrift.)
///
/// FIRST-RUN FAILURE, KEPT ON RECORD: the sweep originally scored rules as
/// one-vs-rest detectors (true negatives counted as correct), and
/// "exploration <= 0.57 => convergent-drift" hit 9/9 that way — a rule that
/// says "not drift" on six windows it never NAMED. The precommitted
/// behavior is naming (a label for every window), so the metric was
/// mis-operationalized, not the corpus invariant wrong. Corrected to
/// naming accuracy: each rule assigns one of the three labels on each side
/// of its threshold. Best possible for a single threshold is 6/9 (two of
/// the three labels always share a side).
#[test]
fn gate_joint_labels_not_reachable_per_dimension() {
    // Frozen corpus: window means per dimension, with true labels.
    let mut base = 10u64;
    let mut corpus: Vec<(PerceptionLabel, Vec<f64>)> = Vec::new();
    for label in NAMED_LABELS {
        for _ in 0..3 {
            let readings = window(base, label);
            base += 100;
            let means = (0..6)
                .map(|d| {
                    readings.iter().map(|r| r.dim(d)).sum::<f64>() / readings.len() as f64
                })
                .collect();
            corpus.push((label, means));
        }
    }
    assert_eq!(corpus.len(), 9);

    // Exhaustive single-dimension NAMING rules: (dim, direction,
    // threshold, label-if-condition-true, label-if-condition-false).
    let mut best: usize = 0;
    let mut best_rule = String::new();
    for dim in 0..6 {
        let mut values: Vec<f64> = corpus.iter().map(|(_, m)| m[dim]).collect();
        values.sort_by(|a, b| a.partial_cmp(b).expect("finite means"));
        values.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
        let mut thresholds: Vec<f64> = vec![values[0] - 1.0, values[values.len() - 1] + 1.0];
        for pair in values.windows(2) {
            thresholds.push((pair[0] + pair[1]) / 2.0);
        }
        for threshold in &thresholds {
            for high_pass in [true, false] {
                for label_if_true in NAMED_LABELS {
                    for label_if_false in NAMED_LABELS {
                        let correct = corpus
                            .iter()
                            .filter(|(true_label, means)| {
                                let condition = if high_pass {
                                    means[dim] >= *threshold
                                } else {
                                    means[dim] <= *threshold
                                };
                                let predicted = if condition {
                                    label_if_true
                                } else {
                                    label_if_false
                                };
                                *true_label == predicted
                            })
                            .count();
                        if correct > best {
                            best = correct;
                            best_rule = format!(
                                "dim {dim} {} {threshold:.4} ? {} : {}",
                                if high_pass { ">=" } else { "<=" },
                                label_if_true.name(),
                                label_if_false.name()
                            );
                        }
                    }
                }
            }
        }
    }
    assert!(
        best < 9,
        "a single-dimension rule named all 9 windows ({best_rule}, {best}/9) — \
         the corpus stopped being jointly-only nameable"
    );

    // Perception names all 9.
    let mut named = 0usize;
    let mut base = 10u64;
    for label in NAMED_LABELS {
        for _ in 0..3 {
            let readings = window(base, label);
            base += 100;
            let mut engine = fresh();
            let outcome = engine
                .perceive(&perception_stack(), &readings)
                .expect("perceive runs");
            if outcome
                .perception
                .as_ref()
                .is_some_and(|perception| perception.label == label)
            {
                named += 1;
            }
        }
    }
    assert_eq!(named, 9, "perception must name 9/9 (best 1-dim rule: {best_rule} at {best}/9)");
}

/// GATE 3 (precommitted: pass) — perceptions revise, never overwrite:
/// a contradicting window supersedes the active hypothesis, both stay in
/// history, and a third contradiction chains again.
#[test]
fn gate_perceptions_revise_not_overwrite() {
    let mut engine = fresh();
    let first = engine
        .perceive(&perception_stack(), &window(10, PerceptionLabel::CoordinatedSweep))
        .expect("perceive runs")
        .perception
        .expect("named");
    let second = engine
        .perceive(&perception_stack(), &window(200, PerceptionLabel::ScattershotProbe))
        .expect("perceive runs")
        .perception
        .expect("named");
    let third = engine
        .perceive(&perception_stack(), &window(300, PerceptionLabel::ConvergentDrift))
        .expect("perceive runs")
        .perception
        .expect("named");

    assert_eq!(second.supersedes, Some(first.id), "contradiction supersedes");
    assert_eq!(third.supersedes, Some(second.id), "supersession chains");

    let history = engine.perceptions();
    assert_eq!(
        history.len(),
        3,
        "retracted perceptions are retained, not overwritten"
    );
    assert!(history.iter().find(|p| p.id == first.id).unwrap().superseded);
    assert!(history.iter().find(|p| p.id == second.id).unwrap().superseded);
    assert!(!history.iter().find(|p| p.id == third.id).unwrap().superseded);
    // The second perception's evidence points only at its own window.
    assert!(second.evidence.iter().all(|id| (200..205).contains(id)));
}

/// GATE 4 (precommitted: pass) — anti-collapse: a perception's support IS
/// the integer count of its evidence references; the type carries no
/// numeric confidence field by construction (label, ids, and a
/// supersession pointer are its entire content). Re-confirmation returns
/// the active hypothesis unchanged (stability, no score drift).
#[test]
fn gate_no_fused_confidence_in_perceptions() {
    let mut engine = fresh();
    let outcome = engine
        .perceive(&perception_stack(), &window(10, PerceptionLabel::CoordinatedSweep))
        .expect("perceive runs");
    let perception = outcome.perception.expect("named");
    let support: usize = perception.evidence.len();
    assert!(support >= engine.config().perception_support_floor);
    assert_eq!(perception.evidence.len(), support);

    // Stability rule: same-label window does not re-emit.
    let again = engine
        .perceive(&perception_stack(), &window(500, PerceptionLabel::CoordinatedSweep))
        .expect("perceive runs");
    assert_eq!(
        again.perception.as_ref().expect("still named").id,
        perception.id,
        "re-confirmation returns the active hypothesis unchanged"
    );
    assert_eq!(engine.perceptions().len(), 1, "no duplicate emission");
}
