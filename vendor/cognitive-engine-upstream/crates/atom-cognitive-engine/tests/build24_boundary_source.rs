//! BUILD-24 gates — the boundary's evidence shape: additive, fail-closed,
//! and policy-free like everything at the boundary.

use atom_cognitive_engine::boundary::{Boundary, Outcome, Request, TaskShape};
use atom_cognitive_engine::evidence::{Evidence, EvidenceError, EvidenceSource};
use atom_cognitive_engine::harness::{run_with_acquisition, Task};
use std::cell::Cell;

struct CountingSource {
    calls: Cell<usize>,
}

impl EvidenceSource for CountingSource {
    fn acquire(&self, _query: &str) -> Result<Evidence, EvidenceError> {
        self.calls.set(self.calls.get() + 1);
        Ok(Evidence {
            source: "fixture:test-world@1".into(),
            receipt_hash: "aa".into(),
            nodes: 3,
            edges: 2,
        })
    }
}

struct FailingSource;

impl EvidenceSource for FailingSource {
    fn acquire(&self, _query: &str) -> Result<Evidence, EvidenceError> {
        Err(EvidenceError("source unavailable".into()))
    }
}

fn scene_task(derivable: bool) -> Task {
    Task {
        blocks: vec![(0, 8), (10, 0)],
        scene: vec![(2, 2), (6, 2), (4, 2)],
        target: if derivable { (0, 16) } else { (1, 0) },
        query: ((2, 2), (6, 2), (4, 2)),
        source_block: (1, 0),
    }
}

fn evidence_request(derivable: bool) -> Request {
    Request::SceneWithEvidence {
        task: scene_task(derivable),
        query: "a declared query".into(),
    }
}

/// B1 (precommitted: pass) — POLICY TOUCHES NOTHING, EXTENDED: the evidence
/// shape's boundary outcome equals a direct `run_with_acquisition` field for
/// field (the P1 precedent on the new entry point).
#[test]
fn gate_b1_policy_touches_nothing_extended() {
    let source = CountingSource {
        calls: Cell::new(0),
    };
    let outcome = Boundary::invoke_with_source(
        TaskShape::SceneQuestionWithEvidence,
        &evidence_request(false),
        &source,
    )
    .expect("boundary run");
    let direct = run_with_acquisition(&scene_task(false), &source, "a declared query")
        .expect("direct run");
    match outcome {
        Outcome::Scene(report) => {
            eprintln!("B1: boundary report == direct report: {}", report == direct);
            assert_eq!(report, direct, "the boundary must not alter the outcome");
        }
        other => panic!("expected a scene report, got {other:?}"),
    }
    assert_eq!(source.calls.get(), 2, "one call per invocation");
}

/// B2 (precommitted: pass) — FAIL-CLOSED on all three doors.
#[test]
fn gate_b2_fail_closed() {
    let source = CountingSource {
        calls: Cell::new(0),
    };
    let via_plain = Boundary::invoke(TaskShape::SceneQuestionWithEvidence, &evidence_request(false));
    eprintln!("B2: source-less entry -> {via_plain:?}");
    assert!(
        matches!(via_plain, Outcome::Refused(_)),
        "the evidence shape is refused without a source"
    );

    let mismatched = Boundary::invoke_with_source(
        TaskShape::SceneQuestion,
        &evidence_request(false),
        &source,
    )
    .expect("no acquisition attempted");
    assert!(
        matches!(mismatched, Outcome::Refused(_)),
        "declared/actual mismatch is refused"
    );

    let failed = Boundary::invoke_with_source(
        TaskShape::SceneQuestionWithEvidence,
        &evidence_request(false),
        &FailingSource,
    );
    assert!(
        matches!(failed, Err(EvidenceError(_))),
        "a source failure yields no report"
    );
    assert_eq!(
        source.calls.get(),
        0,
        "no source is consulted on refused paths"
    );
    eprintln!("B2: three doors, all fail-closed; the source was never consulted on refusals");
}

/// B3 (precommitted: pass) — CONTENT-BLINDNESS PRESERVED: one call either
/// way through the evidence shape; only reported values differ.
#[test]
fn gate_b3_content_blindness_preserved() {
    for derivable in [false, true] {
        let source = CountingSource {
            calls: Cell::new(0),
        };
        let outcome = Boundary::invoke_with_source(
            TaskShape::SceneQuestionWithEvidence,
            &evidence_request(derivable),
            &source,
        )
        .expect("run");
        assert_eq!(
            source.calls.get(),
            1,
            "asked exactly once (derivable={derivable})"
        );
        match outcome {
            Outcome::Scene(report) => assert!(report.acquired.is_some(), "the receipt is reported"),
            other => panic!("expected a scene report, got {other:?}"),
        }
    }

    let a = Boundary::invoke_with_source(
        TaskShape::SceneQuestionWithEvidence,
        &evidence_request(true),
        &CountingSource {
            calls: Cell::new(0),
        },
    )
    .expect("a");
    let b = Boundary::invoke_with_source(
        TaskShape::SceneQuestionWithEvidence,
        &evidence_request(false),
        &CountingSource {
            calls: Cell::new(0),
        },
    )
    .expect("b");
    match (a, b) {
        (Outcome::Scene(ra), Outcome::Scene(rb)) => {
            assert!(ra.sufficient_before, "same path; different measured value");
            assert!(!rb.sufficient_before);
            assert_eq!(ra.scene_support, rb.scene_support);
        }
        other => panic!("both must complete, got {other:?}"),
    }
    eprintln!("B3: one call either way; the verdicts differ, the route does not");
}
