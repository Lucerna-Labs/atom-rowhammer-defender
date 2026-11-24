//! BUILD-23 gates — precommitted in `docs/BUILD23-ACQUIRED-EVIDENCE.md` BEFORE
//! the seam. The engine declares the source; it never knows one.

use atom_cognitive_engine::evidence::{Evidence, EvidenceError, EvidenceSource};
use atom_cognitive_engine::harness::{run, run_with_acquisition, HarnessReport, Task};
use std::cell::Cell;

const EVIDENCE_SRC: &str = include_str!("../src/evidence.rs");
const ENGINE_MANIFEST: &str = include_str!("../Cargo.toml");

/// A counting fixture source — the instrument that measures how often the
/// engine asks.
struct CountingSource {
    calls: Cell<usize>,
    evidence: Evidence,
}

impl EvidenceSource for CountingSource {
    fn acquire(&self, _query: &str) -> Result<Evidence, EvidenceError> {
        self.calls.set(self.calls.get() + 1);
        Ok(self.evidence.clone())
    }
}

/// A source that cannot answer — the fail-closed arm.
struct FailingSource;

impl EvidenceSource for FailingSource {
    fn acquire(&self, _query: &str) -> Result<Evidence, EvidenceError> {
        Err(EvidenceError("source unavailable".into()))
    }
}

fn fixture_evidence(receipt: &str) -> Evidence {
    Evidence {
        source: "fixture:test-world@1".into(),
        receipt_hash: receipt.into(),
        nodes: 3,
        edges: 2,
    }
}

fn task_with(target_derivable: bool) -> Task {
    Task {
        blocks: vec![(0, 8), (10, 0)],
        scene: vec![(2, 2), (6, 2), (4, 2)],
        target: if target_derivable { (0, 16) } else { (1, 0) },
        query: ((2, 2), (6, 2), (4, 2)),
        source_block: (1, 0),
    }
}

/// The eight derivation fields — everything except the provenance.
fn derivation_fields(
    report: &HarnessReport,
) -> (usize, bool, usize, bool, bool, bool, usize, usize) {
    (
        report.scene_support,
        report.judge,
        report.closure_before,
        report.sufficient_before,
        report.sufficient_after,
        report.derivable_now,
        report.economy_blocks,
        report.economy_answers,
    )
}

/// E1 (precommitted: pass) — ACQUISITION IS UNCONDITIONAL AND CONTENT-BLIND:
/// the source is asked exactly once for a derivable and a non-derivable task
/// alike, and the derivation fields equal a direct run's.
#[test]
fn gate_e1_acquisition_unconditional() {
    for derivable in [false, true] {
        let task = task_with(derivable);
        let source = CountingSource {
            calls: Cell::new(0),
            evidence: fixture_evidence("aa"),
        };
        let report = run_with_acquisition(&task, &source, "a declared query").expect("acquisition");
        assert_eq!(
            source.calls.get(),
            1,
            "the source is asked exactly once (derivable={derivable})"
        );
        assert_eq!(report.acquired, Some(fixture_evidence("aa")));
        assert_eq!(
            derivation_fields(&report),
            derivation_fields(&run(&task)),
            "the derivation route is identical to a direct run"
        );
    }
    eprintln!("E1: one call in both cases; derivation fields identical to the direct run");
}

/// E2 (precommitted: pass) — THE RECEIPT IS PROVENANCE, NOT CONTENT: two
/// receipts change only the `acquired` field.
#[test]
fn gate_e2_receipt_is_provenance_not_content() {
    let task = task_with(false);
    let first = CountingSource {
        calls: Cell::new(0),
        evidence: fixture_evidence("1111"),
    };
    let second = CountingSource {
        calls: Cell::new(0),
        evidence: fixture_evidence("2222"),
    };
    let a = run_with_acquisition(&task, &first, "q").expect("a");
    let b = run_with_acquisition(&task, &second, "q").expect("b");
    assert_eq!(
        derivation_fields(&a),
        derivation_fields(&b),
        "different receipts; identical derivations"
    );
    assert_ne!(a.acquired, b.acquired, "only the acquired field differs");
    eprintln!("E2: two receipts -> identical derivation fields, differing provenance");
}

/// E3 (precommitted: pass) — FAIL-CLOSED: an erroring source produces a typed
/// error and NO report; there is no fallback run.
#[test]
fn gate_e3_fail_closed() {
    let outcome = run_with_acquisition(&task_with(false), &FailingSource, "q");
    match outcome {
        Err(EvidenceError(message)) => assert_eq!(message, "source unavailable"),
        Ok(report) => panic!("a failing source must produce no report, got {report:?}"),
    }
    eprintln!("E3: failing source -> typed error, no fallback report");
}

/// E4 (precommitted: pass) — NO COUPLING: the seam is a trait; the engine
/// manifest still names no source.
#[test]
fn gate_e4_no_coupling() {
    assert!(
        !ENGINE_MANIFEST.contains("lightcone"),
        "the engine manifest must not mention the lightcone"
    );
    assert!(
        !ENGINE_MANIFEST.contains("bridge"),
        "the engine manifest must not mention the bridge"
    );
    eprintln!("E4: engine manifest clean — the seam is a trait, not a dependency");
}

/// E6 (precommitted: pass) — AUDIT: the seam contains no branch, no storage;
/// evidence is counts and hashes only.
#[test]
fn gate_e6_audit() {
    assert!(!EVIDENCE_SRC.contains("if "), "no conditional in the seam");
    assert!(!EVIDENCE_SRC.contains("match "), "no branch in the seam");
    assert!(
        !EVIDENCE_SRC.contains("write") && !EVIDENCE_SRC.contains("persist"),
        "nothing persisted"
    );
    let evidence = Evidence {
        source: "s".into(),
        receipt_hash: "h".into(),
        nodes: 1,
        edges: 2,
    };
    assert_eq!((evidence.nodes, evidence.edges), (1, 2));
    eprintln!("E6: seam audit — no branch, no storage; counts and hashes only");
}
