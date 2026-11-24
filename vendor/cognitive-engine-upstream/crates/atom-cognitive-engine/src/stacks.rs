//! Canonical cycle stacks (docs/ORDERING.md) plus the ablation and
//! permutation helpers used by the test suite.
//!
//! Prerequisites here are the *machine-checked safety* edges only
//! (ORDERING.md layer 1). The behavioral order laws (layer 2 — decay before
//! write, conserve after write) are legal to assemble around and are proven
//! by permutation tests, not by prerequisite edges.

use atom_cognitive_core::roots::Root;
use atom_cognitive_core::stages::{Stack, StageSpec};

pub const OBSERVE_VALIDATE: &str = "observe.validate";
pub const OBSERVE_PREDICT: &str = "observe.predict";
pub const OBSERVE_DECAY: &str = "observe.decay";
pub const OBSERVE_ATTEND: &str = "observe.attend";
pub const OBSERVE_GATE: &str = "observe.gate";
/// Record-mode gate: always nucleates. Constitutions are documents, not
/// experiences to compress — record ingestion never merges memories.
pub const OBSERVE_GATE_RECORD: &str = "observe.gate_record";
/// KPR candidate gate: like the match gate, but nucleations land in the
/// Candidate tier (staged, uncommitted, query-invisible) and binds to a
/// candidate advance its stage count only on distinct ticks.
pub const OBSERVE_GATE_CANDIDATE: &str = "observe.gate_candidate";
pub const OBSERVE_WRITE: &str = "observe.write";
/// KPR promotion stage: a candidate with enough contradiction-free
/// distinct-tick stages is promoted to the ingest tier (selective
/// retention); a contradicted candidate has its stages RESET (the
/// kinetic-proofreading reset — failure dissipates accumulation).
pub const OBSERVE_PROMOTE: &str = "observe.promote";
pub const OBSERVE_CONSERVE: &str = "observe.conserve";
pub const OBSERVE_VERIFY: &str = "observe.verify";

pub const QUERY_VALIDATE: &str = "query.validate";
pub const QUERY_ATTEND: &str = "query.attend";
pub const QUERY_GATE: &str = "query.gate";
/// Bond expansion: BFS along echo bonds from eligible traces (depth-capped;
/// expanded members still clear the reliability floor).
pub const QUERY_EXPAND: &str = "query.expand";
pub const QUERY_RETRIEVE: &str = "query.retrieve";
pub const QUERY_ASSERT: &str = "query.assert";
pub const QUERY_VERIFY: &str = "query.verify";

pub const PERCEIVE_VALIDATE: &str = "perceive.validate";
pub const PERCEIVE_ATTEND: &str = "perceive.attend";
/// The joint-naming stage: forms the named hypothesis over the window of
/// readings (label + supporting reading ids — E1, docs/E1-PERCEPTION.md).
pub const PERCEIVE_PERCEIVE: &str = "perceive.perceive";
/// Emission eligibility (assert-or-abstain). NOT an action decision —
/// actions remain judgment's class per the sensing/perception/judgment
/// class law.
pub const PERCEIVE_GATE: &str = "perceive.gate";
pub const PERCEIVE_VERIFY: &str = "perceive.verify";

/// The canonical observe cycle: validate → predict → decay → attend →
/// gate → write → conserve → verify. (O1, O6 behavioral; O2/O4 structural.)
pub fn observe_stack() -> Stack {
    Stack::assemble(vec![
        StageSpec::new(OBSERVE_VALIDATE, None, &[]),
        StageSpec::new(OBSERVE_PREDICT, Some(Root::Predict), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_DECAY, Some(Root::Decay), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_ATTEND, Some(Root::Attend), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_GATE, Some(Root::Gate), &[OBSERVE_ATTEND]),
        StageSpec::new(
            OBSERVE_WRITE,
            Some(Root::Bind),
            &[OBSERVE_GATE, OBSERVE_ATTEND, OBSERVE_PREDICT],
        ),
        StageSpec::new(OBSERVE_CONSERVE, Some(Root::Conserve), &[OBSERVE_WRITE]),
        StageSpec::new(OBSERVE_VERIFY, None, &[OBSERVE_CONSERVE]),
    ])
    .expect("canonical observe stack satisfies its own contract")
}

/// The canonical query cycle: validate → attend → gate → retrieve →
/// assert-or-abstain → verify. (O3, O7.)
pub fn query_stack() -> Stack {
    Stack::assemble(vec![
        StageSpec::new(QUERY_VALIDATE, None, &[]),
        StageSpec::new(QUERY_ATTEND, Some(Root::Attend), &[QUERY_VALIDATE]),
        StageSpec::new(QUERY_GATE, Some(Root::Gate), &[QUERY_ATTEND]),
        StageSpec::new(QUERY_RETRIEVE, None, &[QUERY_GATE]),
        StageSpec::new(QUERY_ASSERT, None, &[QUERY_GATE, QUERY_RETRIEVE]),
        StageSpec::new(QUERY_VERIFY, None, &[QUERY_ASSERT]),
    ])
    .expect("canonical query stack satisfies its own contract")
}

/// The idle cycle: time passes (decay), budgets hold (conserve), state is
/// verified. Used to demonstrate forgetting of unsupported traces.
pub fn idle_stack() -> Stack {
    Stack::assemble(vec![
        StageSpec::new("idle.validate", None, &[]),
        StageSpec::new("idle.decay", Some(Root::Decay), &["idle.validate"]),
        StageSpec::new("idle.conserve", Some(Root::Conserve), &["idle.decay"]),
        StageSpec::new("idle.verify", None, &["idle.conserve"]),
    ])
    .expect("canonical idle stack satisfies its own contract")
}

/// Behavioral permutation for law O1: decay moved AFTER write. Legal to
/// assemble (no prerequisite is violated) — the permutation tests prove the
/// order measurably changes state (the written trace is decayed on its own
/// tick).
pub fn misordered_decay_after_write() -> Stack {
    Stack::assemble(vec![
        StageSpec::new(OBSERVE_VALIDATE, None, &[]),
        StageSpec::new(OBSERVE_PREDICT, Some(Root::Predict), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_ATTEND, Some(Root::Attend), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_GATE, Some(Root::Gate), &[OBSERVE_ATTEND]),
        StageSpec::new(
            OBSERVE_WRITE,
            Some(Root::Bind),
            &[OBSERVE_GATE, OBSERVE_ATTEND, OBSERVE_PREDICT],
        ),
        StageSpec::new(OBSERVE_DECAY, Some(Root::Decay), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_CONSERVE, Some(Root::Conserve), &[OBSERVE_WRITE]),
        StageSpec::new(OBSERVE_VERIFY, None, &[OBSERVE_CONSERVE]),
    ])
    .expect("mis-ordered variant must assemble (it is legal, not safe)")
}

/// The deliberately reachable O7 failure mode: retrieval WITHOUT the
/// eligibility gate (retrieve requires attend only). Kept on purpose so the
/// suite can prove the gate is load-bearing — attention without gating
/// asserts from unreliable traces.
pub fn ungated_query_stack() -> Stack {
    Stack::assemble(vec![
        StageSpec::new(QUERY_VALIDATE, None, &[]),
        StageSpec::new(QUERY_ATTEND, Some(Root::Attend), &[QUERY_VALIDATE]),
        StageSpec::new(QUERY_RETRIEVE, None, &[QUERY_ATTEND]),
        StageSpec::new(QUERY_ASSERT, None, &[QUERY_RETRIEVE]),
        StageSpec::new(QUERY_VERIFY, None, &[QUERY_ASSERT]),
    ])
    .expect("ungated variant must assemble (it is legal, not safe)")
}

/// Record-mode observe cycle: like the canonical observe cycle but the gate
/// always nucleates — one trace per memory, no compression. For corpus and
/// constitution ingestion.
pub fn record_stack() -> Stack {
    Stack::assemble(vec![
        StageSpec::new(OBSERVE_VALIDATE, None, &[]),
        StageSpec::new(OBSERVE_PREDICT, Some(Root::Predict), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_DECAY, Some(Root::Decay), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_ATTEND, Some(Root::Attend), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_GATE_RECORD, Some(Root::Gate), &[OBSERVE_ATTEND]),
        StageSpec::new(
            OBSERVE_WRITE,
            Some(Root::Bind),
            &[OBSERVE_GATE_RECORD, OBSERVE_ATTEND, OBSERVE_PREDICT],
        ),
        StageSpec::new(OBSERVE_CONSERVE, Some(Root::Conserve), &[OBSERVE_WRITE]),
        StageSpec::new(OBSERVE_VERIFY, None, &[OBSERVE_CONSERVE]),
    ])
    .expect("record stack satisfies its own contract")
}

/// Kinetic-proofreading observe cycle (D35, candidate #1): evidence lands
/// in the Candidate tier and must survive contradiction-free distinct-tick
/// stages before promotion — transient matches dissipate, contradictions
/// reset accumulation.
pub fn kpr_stack() -> Stack {
    Stack::assemble(vec![
        StageSpec::new(OBSERVE_VALIDATE, None, &[]),
        StageSpec::new(OBSERVE_PREDICT, Some(Root::Predict), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_DECAY, Some(Root::Decay), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_ATTEND, Some(Root::Attend), &[OBSERVE_VALIDATE]),
        StageSpec::new(OBSERVE_GATE_CANDIDATE, Some(Root::Gate), &[OBSERVE_ATTEND]),
        StageSpec::new(
            OBSERVE_WRITE,
            Some(Root::Bind),
            &[OBSERVE_GATE_CANDIDATE, OBSERVE_ATTEND, OBSERVE_PREDICT],
        ),
        StageSpec::new(OBSERVE_PROMOTE, Some(Root::Gate), &[OBSERVE_WRITE]),
        StageSpec::new(OBSERVE_CONSERVE, Some(Root::Conserve), &[OBSERVE_PROMOTE]),
        StageSpec::new(OBSERVE_VERIFY, None, &[OBSERVE_CONSERVE]),
    ])
    .expect("kpr stack satisfies its own contract")
}

/// Mother query cycle: the canonical query cycle with bond expansion
/// between the gate and retrieval, so echo-linked memories activate even
/// when facet similarity alone would miss them.
pub fn mother_query_stack() -> Stack {
    Stack::assemble(vec![
        StageSpec::new(QUERY_VALIDATE, None, &[]),
        StageSpec::new(QUERY_ATTEND, Some(Root::Attend), &[QUERY_VALIDATE]),
        StageSpec::new(QUERY_GATE, Some(Root::Gate), &[QUERY_ATTEND]),
        StageSpec::new(QUERY_EXPAND, None, &[QUERY_GATE]),
        StageSpec::new(QUERY_RETRIEVE, None, &[QUERY_GATE]),
        StageSpec::new(QUERY_ASSERT, None, &[QUERY_GATE, QUERY_RETRIEVE]),
        StageSpec::new(QUERY_VERIFY, None, &[QUERY_ASSERT]),
    ])
    .expect("mother query stack satisfies its own contract")
}

/// The perception cycle (E1, docs/E1-PERCEPTION.md): validate → attend →
/// perceive → gate → verify. PERCEIVE admitted as root #9 at ROOTS.md
/// v0.3 by the four E1 gates; the gate is emission eligibility —
/// assert-or-abstain, never an action.
pub fn perception_stack() -> Stack {
    Stack::assemble(vec![
        StageSpec::new(PERCEIVE_VALIDATE, None, &[]),
        StageSpec::new(PERCEIVE_ATTEND, Some(Root::Attend), &[PERCEIVE_VALIDATE]),
        StageSpec::new(PERCEIVE_PERCEIVE, Some(Root::Perceive), &[PERCEIVE_ATTEND]),
        StageSpec::new(PERCEIVE_GATE, Some(Root::Gate), &[PERCEIVE_PERCEIVE]),
        StageSpec::new(PERCEIVE_VERIFY, None, &[PERCEIVE_GATE]),
    ])
    .expect("perception stack satisfies its own contract")
}

/// Remove one stage (and every stage that transitively requires it) from a
/// stack. Used for root ablations; the result still assembles legally.
pub fn omit(stack: &Stack, stage_id: &'static str) -> Stack {
    let mut drop_ids: Vec<&'static str> = vec![stage_id];
    loop {
        let mut changed = false;
        for stage in stack.stages() {
            if drop_ids.contains(&stage.id) {
                continue;
            }
            if stage.prerequisites.iter().any(|p| drop_ids.contains(p)) {
                drop_ids.push(stage.id);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let remaining: Vec<StageSpec> = stack
        .stages()
        .iter()
        .filter(|s| !drop_ids.contains(&s.id))
        .cloned()
        .collect();
    Stack::assemble(remaining).expect("ablated stack must remain assemblable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_orders_match_ordering_md() {
        assert_eq!(
            observe_stack().order(),
            vec![
                "observe.validate",
                "observe.predict",
                "observe.decay",
                "observe.attend",
                "observe.gate",
                "observe.write",
                "observe.conserve",
                "observe.verify"
            ]
        );
        assert_eq!(
            query_stack().order(),
            vec![
                "query.validate",
                "query.attend",
                "query.gate",
                "query.retrieve",
                "query.assert",
                "query.verify"
            ]
        );
    }

    #[test]
    fn misordered_variant_is_legal_but_different() {
        let canonical = observe_stack().order();
        let misordered = misordered_decay_after_write().order();
        assert_ne!(canonical, misordered);
        // Same stages, different order.
        let mut a = canonical.clone();
        let mut b = misordered.clone();
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b);
    }

    #[test]
    fn omit_drops_dependents_transitively() {
        // Removing decay keeps everything (nothing structurally requires it);
        // removing attend drops gate, write, conserve, verify.
        let stack = observe_stack();
        assert!(omit(&stack, OBSERVE_DECAY).contains(OBSERVE_WRITE));
        let without_attend = omit(&stack, OBSERVE_ATTEND);
        assert!(!without_attend.contains(OBSERVE_GATE));
        assert!(!without_attend.contains(OBSERVE_WRITE));
        assert!(!without_attend.contains(OBSERVE_CONSERVE));
        assert!(!without_attend.contains(OBSERVE_VERIFY));
    }
}
