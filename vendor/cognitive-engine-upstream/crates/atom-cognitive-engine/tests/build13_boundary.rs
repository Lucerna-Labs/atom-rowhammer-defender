//! BUILD-13 gates — precommitted in docs/BUILD13-BOUNDARY.md BEFORE this
//! code. The one policy surface: select, invoke, report. Never compute.

use atom_cognitive_engine::boundary::{validate_registration, Boundary, Outcome, Request, TaskShape};
use atom_cognitive_engine::geometry::Point;
use atom_cognitive_engine::harness::{harness_stages, run, Task as HarnessTask};

fn scene_task(target_derivable: bool) -> HarnessTask {
    HarnessTask {
        blocks: vec![(0, 8), (10, 0)],
        scene: vec![(2, 2), (6, 2), (4, 2)],
        target: if target_derivable { (0, 16) } else { (1, 0) },
        query: ((2, 2), (6, 2), (4, 2)),
        source_block: (1, 0),
    }
}

/// P1 (precommitted: pass) — POLICY TOUCHES NOTHING: the boundary's
/// outcome for a stack is IDENTICAL to direct invocation.
#[test]
fn gate_p1_policy_touches_nothing() {
    let task = scene_task(false);
    let direct = run(&task);
    let via_boundary = Boundary::invoke(TaskShape::SceneQuestion, &Request::Scene(scene_task(false)));
    match via_boundary {
        Outcome::Scene(report) => {
            eprintln!("boundary report == direct report: {}", report == direct);
            assert_eq!(report, direct, "the boundary must not alter the outcome");
        }
        other => panic!("expected a scene report, got {other:?}"),
    }
    // Product shape: identical to direct generation too.
    let (n1, n2) = (1u64 << 20, (1u64 << 20) - 1);
    let m1 = atom_cognitive_engine::knowledge::generated_product(n1, 997, 638);
    let m2 = atom_cognitive_engine::knowledge::generated_product(n2, 997, 638);
    let direct_product = atom_cognitive_engine::knowledge::reconstruct(n1, m1, n2, m2);
    let via = Boundary::invoke(
        TaskShape::ArithmeticProduct,
        &Request::Product { modulus_1: n1, modulus_2: n2, block: 997, steps: 638 },
    );
    assert_eq!(via, Outcome::Product(direct_product));
}

/// P2 (precommitted: pass) — FAIL-CLOSED: a mismatched declaration is
/// refused; nothing runs, no fallback instructions exist.
#[test]
fn gate_p2_fail_closed() {
    let outcome = Boundary::invoke(
        TaskShape::ArithmeticProduct,
        &Request::Scene(scene_task(false)),
    );
    eprintln!("mismatched declaration -> {outcome:?}");
    assert!(matches!(outcome, Outcome::Refused(_)), "fail-closed on mismatch");
}

/// P3 (precommitted: pass) — CONTENT-BLINDNESS (measured): derivable and
/// non-derivable tasks (same declared shape) route identically; only
/// reported values differ.
#[test]
fn gate_p3_content_blindness() {
    let a = Boundary::invoke(TaskShape::SceneQuestion, &Request::Scene(scene_task(true)));
    let b = Boundary::invoke(TaskShape::SceneQuestion, &Request::Scene(scene_task(false)));
    match (a, b) {
        (Outcome::Scene(ra), Outcome::Scene(rb)) => {
            eprintln!("derivable: sufficient_before {} | non-derivable: {}", ra.sufficient_before, rb.sufficient_before);
            assert!(ra.sufficient_before, "same path; different measured value");
            assert!(!rb.sufficient_before);
            assert_eq!(ra.judge, rb.judge, "the same stage sequence judged both");
            assert_eq!(ra.scene_support, rb.scene_support);
        }
        other => panic!("both must complete through the same route, got {other:?}"),
    }
}

/// P4 (precommitted: pass) — REGISTRY VALIDATION: a mis-ordered stack
/// cannot be registered.
#[test]
fn gate_p4_registry_validation() {
    assert!(validate_registration(harness_stages()).is_ok());
    let mut stages = harness_stages();
    let idx = stages.iter().position(|s| s.id == "harness.sufficiency").unwrap();
    let stage = stages.remove(idx);
    stages.insert(3, stage); // before derive
    assert!(validate_registration(stages).is_err(), "mis-ordered registration rejected");
}

/// P5 (precommitted: measured) — LINE ECONOMY: the policy module vs the
/// engines, printed and bounded. The whole reason instructions are
/// refused.
#[test]
fn gate_p5_line_economy() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let read_lines = |name: &str| -> usize {
        std::fs::read_to_string(format!("{manifest}/src/{name}"))
            .map(|s| s.lines().count())
            .unwrap_or(0)
    };
    let policy = read_lines("boundary.rs");
    let engines: Vec<&str> = vec![
        "interference_field.rs", "perception.rs", "perception_quantum.rs", "reasoning_closure.rs",
        "perception_field.rs", "preference.rs", "action_world.rs", "generation.rs", "geometry.rs",
        "spatial.rs", "reasoning_types.rs", "knowledge.rs", "harness.rs", "weave.rs", "engine.rs",
        "stacks.rs", "config.rs", "trace.rs",
    ];
    let engine_lines: usize = engines.iter().map(|n| read_lines(n)).sum();
    let ratio = policy as f64 / engine_lines as f64;
    eprintln!("line economy: policy {policy} lines vs engines {engine_lines} lines = {:.2}%", ratio * 100.0);
    assert!(ratio < 0.05, "the policy surface must stay under 5% of the engines");
    let _ = Point::default;
}
