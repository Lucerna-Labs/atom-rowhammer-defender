//! BUILD-27 gates — hierarchical composition: a registered stack invoking
//! registered stacks, nested by declaration.

use atom_cognitive_engine::boundary::{
    Boundary, Chain, Outcome, Registry, Request, TaskShape, CHAINS,
};
use atom_cognitive_engine::harness::Task;

fn scene_task(derivable: bool) -> Task {
    Task {
        blocks: vec![(0, 8), (10, 0)],
        scene: vec![(2, 2), (6, 2), (4, 2)],
        target: if derivable { (0, 16) } else { (1, 0) },
        query: ((2, 2), (6, 2), (4, 2)),
        source_block: (1, 0),
    }
}

fn product_request() -> Request {
    Request::Product {
        modulus_1: 1 << 20,
        modulus_2: (1 << 20) - 1,
        block: 997,
        steps: 638,
    }
}

/// H1 (precommitted: pass) — DECLARED AND VALIDATED: the shipped table
/// validates; unknown names and cycles are construction errors.
#[test]
fn gate_h1_declared_and_validated() {
    Registry::validate(CHAINS).expect("the shipped registry validates");
    let names: Vec<&str> = CHAINS.iter().map(|c| c.name).collect();
    eprintln!("H1: shipped chains {names:?}");

    const UNKNOWN: &[Chain] = &[Chain {
        name: "a",
        stages: &[TaskShape::Chain("missing")],
    }];
    assert!(
        Registry::validate(UNKNOWN).is_err(),
        "an unregistered sub-stack never registers"
    );

    const CYCLIC: &[Chain] = &[
        Chain {
            name: "a",
            stages: &[TaskShape::Chain("b")],
        },
        Chain {
            name: "b",
            stages: &[TaskShape::Chain("a")],
        },
    ];
    let error = Registry::validate(CYCLIC).expect_err("a cycle never registers");
    eprintln!("H1: cycle refused: {error}");
    assert!(error.contains("cycle"), "the refusal names the cycle");
}

/// H2 (precommitted: pass) — INVOCATION TOUCHES NOTHING: every stage's
/// nested outcome equals its DIRECT invocation, field for field.
#[test]
fn gate_h2_invocation_touches_nothing() {
    let request = Request::Chain(vec![
        Request::Scene(scene_task(false)),
        product_request(),
    ]);
    let nested = Registry::invoke(TaskShape::Chain("scene_then_product"), &request);
    let direct_scene =
        Boundary::invoke(TaskShape::SceneQuestion, &Request::Scene(scene_task(false)));
    let direct_product = Boundary::invoke(TaskShape::ArithmeticProduct, &product_request());
    match nested {
        Outcome::Chain(outcomes) => {
            assert_eq!(outcomes.len(), 2, "two declared stages, two outcomes");
            assert_eq!(outcomes[0], direct_scene, "stage 1 == its direct invocation");
            assert_eq!(outcomes[1], direct_product, "stage 2 == its direct invocation");
        }
        other => panic!("expected a chain outcome, got {other:?}"),
    }
    eprintln!("H2: both stages equal their direct invocations, field for field");
}

/// H3 (precommitted: pass) — NESTING OBSERVED: the nested chain's first
/// outcome is itself a two-outcome chain result.
#[test]
fn gate_h3_nesting_observed() {
    let request = Request::Chain(vec![
        Request::Chain(vec![Request::Scene(scene_task(true)), product_request()]),
        Request::Scene(scene_task(false)),
    ]);
    match Registry::invoke(TaskShape::Chain("nested"), &request) {
        Outcome::Chain(outer) => {
            assert_eq!(outer.len(), 2, "the outer chain has two stages");
            match &outer[0] {
                Outcome::Chain(inner) => {
                    assert_eq!(inner.len(), 2, "the sub-chain is itself a two-outcome result");
                }
                other => panic!("the first outcome must be the sub-chain, got {other:?}"),
            }
            assert!(
                matches!(outer[1], Outcome::Scene(_)),
                "the second stage reports its own scene outcome"
            );
        }
        other => panic!("expected a chain outcome, got {other:?}"),
    }
    eprintln!("H3: depth is structure — the nested report shows the sub-chain");
}

/// H4 (precommitted: pass) — FAIL-CLOSED PROPAGATES: a count mismatch
/// refuses the chain; a mismatched stage surfaces as Refused in place.
#[test]
fn gate_h4_fail_closed() {
    let short = Registry::invoke(
        TaskShape::Chain("scene_then_product"),
        &Request::Chain(vec![Request::Scene(scene_task(false))]),
    );
    assert!(
        matches!(short, Outcome::Refused(_)),
        "a count mismatch refuses the whole chain"
    );

    let misplaced = Registry::invoke(
        TaskShape::Chain("scene_then_product"),
        &Request::Chain(vec![product_request(), product_request()]),
    );
    match misplaced {
        Outcome::Chain(outcomes) => {
            assert!(
                matches!(outcomes[0], Outcome::Refused(_)),
                "the mismatched stage ran nothing"
            );
            assert!(
                matches!(outcomes[1], Outcome::Product(_)),
                "the next stage still reports what happened"
            );
        }
        other => panic!("expected a chain outcome, got {other:?}"),
    }

    let unknown = Registry::invoke(TaskShape::Chain("no_such_chain"), &Request::Chain(vec![]));
    assert!(matches!(unknown, Outcome::Refused(_)), "an unknown chain refuses");
    eprintln!("H4: refusals surface in place; nothing runs that was refused");
}

/// H5 (precommitted: pass) — CONTENT-BLINDNESS PRESERVED AT DEPTH.
#[test]
fn gate_h5_content_blindness() {
    let run = |derivable: bool| {
        Registry::invoke(
            TaskShape::Chain("scene_then_product"),
            &Request::Chain(vec![Request::Scene(scene_task(derivable)), product_request()]),
        )
    };
    match (run(true), run(false)) {
        (Outcome::Chain(first), Outcome::Chain(second)) => {
            assert_eq!(first.len(), 2);
            assert_eq!(second.len(), 2);
            match (&first[0], &second[0]) {
                (Outcome::Scene(a), Outcome::Scene(b)) => {
                    assert!(
                        a.sufficient_before && !b.sufficient_before,
                        "same route; different measured values"
                    );
                    assert_eq!(a.scene_support, b.scene_support);
                }
                other => panic!("expected scene outcomes, got {other:?}"),
            }
            assert_eq!(
                first[1], second[1],
                "the product stage reports identically for both tasks"
            );
        }
        other => panic!("expected chain outcomes, got {other:?}"),
    }
    eprintln!("H5: both stages run for both tasks; only the verdicts differ");
}

/// H6 (precommitted: pass) — LINE ECONOMY: the composition surface is thin —
/// the registry is DATA, the invocation is one declared walk.
#[test]
fn gate_h6_line_economy() {
    // BUILD-30 RECORDED EDIT (2026-09-26): the count moved from two to three
    // when BUILD-30 appended the declared composition
    // `provenance_then_product` (provenance inside a composition). The gate's
    // substance is unchanged — the registry is DATA and every entry is a
    // two-stage declaration — and the edit is named here rather than quietly
    // loosened.
    assert_eq!(CHAINS.len(), 3, "declared compositions ship as data");
    assert_eq!(CHAINS[0].stages.len(), 2);
    assert_eq!(CHAINS[1].stages.len(), 2);
    assert_eq!(CHAINS[2].stages.len(), 2);
    eprintln!("H6: the registry is a table; the invocation is one walk over declared stages");
}
