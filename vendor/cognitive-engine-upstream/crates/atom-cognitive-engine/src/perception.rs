//! PERCEPTION — named, revisable hypotheses over readings (E1 candidate;
//! precommitted spec in `docs/E1-PERCEPTION.md`, written before this code).
//!
//! Class law (receipted 2026-09-24): sensing produces readings, perception
//! forms a NAMED hypothesis about what was sensed, judgment decides an
//! action — three classes that must stay separate. Anti-collapse law: a
//! perception carries a label and the IDS of the readings that supported
//! it; no fused confidence number exists in this module by construction.

/// The six receipted sensor dimensions, in reading order.
pub const DIMENSIONS: [&str; 6] = [
    "rhythm_pe",
    "drift",
    "repeat_ratio",
    "exploration",
    "convergence",
    "rate_per_sec",
];

/// One sensed reading (SENSING's class — substrate, not a hypothesis).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reading {
    pub id: u64,
    pub rhythm_pe: f64,
    pub drift: f64,
    pub repeat_ratio: f64,
    pub exploration: f64,
    pub convergence: f64,
    pub rate_per_sec: f64,
}

impl Reading {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u64,
        rhythm_pe: f64,
        drift: f64,
        repeat_ratio: f64,
        exploration: f64,
        convergence: f64,
        rate_per_sec: f64,
    ) -> Self {
        Reading {
            id,
            rhythm_pe,
            drift,
            repeat_ratio,
            exploration,
            convergence,
            rate_per_sec,
        }
    }

    /// Dimension value by index (see `DIMENSIONS`).
    pub fn dim(&self, index: usize) -> f64 {
        match index {
            0 => self.rhythm_pe,
            1 => self.drift,
            2 => self.repeat_ratio,
            3 => self.exploration,
            4 => self.convergence,
            _ => self.rate_per_sec,
        }
    }

    /// A reading is well-formed when every dimension is finite; the attend
    /// stage of the perception cycle admits well-formed readings only.
    pub fn well_formed(&self) -> bool {
        [
            self.rhythm_pe,
            self.drift,
            self.repeat_ratio,
            self.exploration,
            self.convergence,
            self.rate_per_sec,
        ]
        .iter()
        .all(|v| v.is_finite())
    }

    /// The joint conjunction defining `label`. Deliberately single-
    /// dimension-ambiguous (E1): exploration-high is shared by
    /// CoordinatedSweep and ScattershotProbe; convergence-high is shared
    /// by CoordinatedSweep and ConvergentDrift.
    pub fn satisfies(&self, label: PerceptionLabel) -> bool {
        match label {
            PerceptionLabel::CoordinatedSweep => {
                self.exploration >= 0.70 && self.convergence >= 0.60
            }
            PerceptionLabel::ScattershotProbe => {
                self.exploration >= 0.70 && self.convergence <= 0.40
            }
            PerceptionLabel::ConvergentDrift => {
                self.exploration <= 0.40 && self.convergence >= 0.60
            }
        }
    }
}

/// A named hypothesis about readings — evidence references, not a fused
/// number; revisable (`supersedes`/`superseded`), never silently replaced.
#[derive(Debug, Clone, PartialEq)]
pub struct Perception {
    pub id: u64,
    pub label: PerceptionLabel,
    /// Ids of the readings that supported this hypothesis.
    pub evidence: Vec<u64>,
    /// The perception this one retracts, if any.
    pub supersedes: Option<u64>,
    /// True once a later perception has retracted this one.
    pub superseded: bool,
    /// Times a window has (re-)confirmed this hypothesis while active —
    /// the substrate the E1-chem catalyst scales its floor relief on.
    pub confirmations: u32,
}

/// The E1 joint-label vocabulary (minimal admission instrument).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerceptionLabel {
    CoordinatedSweep,
    ScattershotProbe,
    ConvergentDrift,
}

/// Every named label, in vocabulary order.
pub const NAMED_LABELS: [PerceptionLabel; 3] = [
    PerceptionLabel::CoordinatedSweep,
    PerceptionLabel::ScattershotProbe,
    PerceptionLabel::ConvergentDrift,
];

impl PerceptionLabel {
    pub fn name(self) -> &'static str {
        match self {
            PerceptionLabel::CoordinatedSweep => "coordinated-sweep",
            PerceptionLabel::ScattershotProbe => "scattershot-probe",
            PerceptionLabel::ConvergentDrift => "convergent-drift",
        }
    }
}

/// Outcome of one perception cycle: a named hypothesis, or an abstention
/// with its named reason. No probabilities anywhere.
#[derive(Debug, Clone, PartialEq)]
pub struct PerceptionOutcome {
    pub perception: Option<Perception>,
    pub abstain_reason: Option<&'static str>,
}

impl PerceptionOutcome {
    pub fn abstain(reason: &'static str) -> Self {
        PerceptionOutcome {
            perception: None,
            abstain_reason: Some(reason),
        }
    }
}

/// Pure matcher: name a window of readings by the label whose conjunction
/// the most well-formed readings satisfy, provided that count reaches
/// `support_floor` (an integer COUNT of evidence references, not a fused
/// number) and is not tied. Abstains otherwise. The returned perception
/// carries id 0 and no supersession — the engine assigns identity and
/// revisability at emission.
pub fn name_window(readings: &[Reading], support_floor: usize) -> PerceptionOutcome {
    name_window_catalyzed(readings, support_floor, None, 0.0)
}

/// Catalyzed matcher (E1-chem, docs/E1-PERCEPTION.md §E1-chem): a label
/// carrying an ACTIVE perception has its support floor relaxed by
/// `catalysis × confirmations` (bounded: effective floor never below 1).
/// A catalyst lowers the activation energy of an ONGOING reaction only —
/// labels without an active perception keep the full floor (it cannot
/// start a naming), and competition between qualifying labels is still
/// by evidence count (it cannot override stronger evidence or change
/// which labels are possible).
pub fn name_window_catalyzed(
    readings: &[Reading],
    support_floor: usize,
    active: Option<(PerceptionLabel, u32)>,
    catalysis: f64,
) -> PerceptionOutcome {
    let well_formed: Vec<&Reading> = readings.iter().filter(|r| r.well_formed()).collect();
    if well_formed.len() < support_floor {
        return PerceptionOutcome::abstain("window-below-support-floor");
    }
    let (active_label, relief) = match active {
        Some((label, confirmations)) => {
            let relief = (catalysis * f64::from(confirmations)).floor() as usize;
            (Some(label), relief)
        }
        None => (None, 0),
    };
    let named: Vec<(PerceptionLabel, Vec<u64>)> = NAMED_LABELS
        .iter()
        .map(|&label| {
            let evidence: Vec<u64> = well_formed
                .iter()
                .filter(|r| r.satisfies(label))
                .map(|r| r.id)
                .collect();
            (label, evidence)
        })
        .collect();
    let qualifying: Vec<&(PerceptionLabel, Vec<u64>)> = named
        .iter()
        .filter(|(label, evidence)| {
            let effective_floor = if Some(*label) == active_label {
                support_floor.saturating_sub(relief).max(1)
            } else {
                support_floor
            };
            evidence.len() >= effective_floor
        })
        .collect();
    if qualifying.is_empty() {
        return PerceptionOutcome::abstain("no-label-reaches-support-floor");
    }
    let top = qualifying.iter().map(|(_, evidence)| evidence.len()).max().unwrap_or(0);
    let winners: Vec<&(PerceptionLabel, Vec<u64>)> = qualifying
        .iter()
        .filter(|(_, evidence)| evidence.len() == top)
        .map(|entry| *entry)
        .collect();
    if winners.len() > 1 {
        return PerceptionOutcome::abstain("tied-evidence");
    }
    let (label, evidence) = winners[0];
    PerceptionOutcome {
        perception: Some(Perception {
            id: 0,
            label: *label,
            evidence: evidence.clone(),
            supersedes: None,
            superseded: false,
            confirmations: 1,
        }),
        abstain_reason: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(id: u64, exploration: f64, convergence: f64) -> Reading {
        Reading::new(id, 0.5, 0.5, 0.5, exploration, convergence, 0.5)
    }

    #[test]
    fn conjunctions_are_single_dimension_ambiguous() {
        // The corpus invariant E1 depends on: each marginal is shared by
        // two labels, so no single dimension can name a window alone.
        let sweep = reading(1, 0.8, 0.7);
        let scattershot = reading(2, 0.8, 0.3);
        let drift = reading(3, 0.3, 0.7);
        assert!(sweep.satisfies(PerceptionLabel::CoordinatedSweep));
        assert!(scattershot.satisfies(PerceptionLabel::ScattershotProbe));
        assert!(drift.satisfies(PerceptionLabel::ConvergentDrift));
        // exploration >= 0.70 holds for sweep AND scattershot; convergence
        // >= 0.60 holds for sweep AND drift.
        assert!(scattershot.exploration >= 0.70);
        assert!(drift.convergence >= 0.60);
    }

    #[test]
    fn ties_abstain() {
        let readings: Vec<Reading> = (0..6)
            .map(|k| {
                if k < 3 {
                    reading(k, 0.8, 0.7)
                } else {
                    reading(k, 0.8, 0.3)
                }
            })
            .collect();
        let outcome = name_window(&readings, 3);
        assert!(outcome.perception.is_none());
        assert_eq!(outcome.abstain_reason, Some("tied-evidence"));
    }

    #[test]
    fn below_floor_abstains() {
        let readings = vec![reading(1, 0.8, 0.7)];
        let outcome = name_window(&readings, 3);
        assert!(outcome.perception.is_none());
    }

    #[test]
    fn ill_formed_readings_do_not_count() {
        let mut bad = reading(1, f64::NAN, 0.7);
        bad.rhythm_pe = f64::NAN;
        assert!(!bad.well_formed());
        let readings = vec![bad; 5];
        let outcome = name_window(&readings, 3);
        assert!(outcome.perception.is_none());
        assert_eq!(outcome.abstain_reason, Some("window-below-support-floor"));
    }
}
