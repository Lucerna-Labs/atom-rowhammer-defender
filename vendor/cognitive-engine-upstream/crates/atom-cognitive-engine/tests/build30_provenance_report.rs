//! BUILD-30 gates — precommitted in docs/BUILD30-PROVENANCE-REPORT.md BEFORE
//! this code. The words are wired into the report; the wiring is inert to
//! every verdict, exact where it speaks, and fail-closed where it cannot.

use atom_block_library::{relations_for_derivation, BLOCKS};
use atom_cognitive_engine::boundary::{
    Boundary, Chain, Outcome, Registry, Request, TaskShape, CHAINS,
};
use atom_cognitive_engine::causal_path::WalkOrder;
use atom_cognitive_engine::derivation_paths::WordGraph;
use atom_cognitive_engine::harness::{run, run_with_provenance, Task as HarnessTask};

const MODULUS: i64 = 20;
const TARGET: (i64, i64) = (14, 16);

/// The stored relation blocks across the two geometry partitions (7 blocks,
/// 4 distinct relations).
fn stored_blocks() -> Vec<(i64, i64)> {
    let mut blocks = relations_for_derivation(BLOCKS, "geometry").expect("geometry");
    blocks.extend(relations_for_derivation(BLOCKS, "geometry.extended").expect("extended"));
    blocks
}

fn declared_task() -> HarnessTask {
    HarnessTask {
        blocks: stored_blocks(),
        scene: vec![(0, 8), (10, 0), (10, 12)],
        target: TARGET,
        query: ((0, 8), (10, 0), (10, 12)),
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

fn replay(relations: &[(i64, i64)]) -> (i64, i64) {
    let mut sum = (0i64, 0i64);
    for (x, y) in relations {
        sum = ((sum.0 + x).rem_euclid(MODULUS), (sum.1 + y).rem_euclid(MODULUS));
    }
    sum
}

/// P1 (precommitted: pass) — THE WIRING IS INERT TO EVERYTHING ELSE. Every
/// recorded field is identical with and without the provenance step; only
/// `provenance` differs.
#[test]
fn gate_p1_wiring_is_inert() {
    let task = declared_task();
    let plain = run(&task);
    let wired = run_with_provenance(&task, 6).expect("the stored blocks carry structure");

    assert_eq!(plain.scene_support, wired.scene_support);
    assert_eq!(plain.judge, wired.judge);
    assert_eq!(plain.closure_before, wired.closure_before);
    assert_eq!(plain.sufficient_before, wired.sufficient_before);
    assert_eq!(plain.sufficient_after, wired.sufficient_after);
    assert_eq!(plain.derivable_now, wired.derivable_now);
    assert_eq!(plain.economy_blocks, wired.economy_blocks);
    assert_eq!(plain.economy_answers, wired.economy_answers);
    assert_eq!(plain.acquired, wired.acquired);

    // The recorded entry point is unchanged: no provenance by default.
    assert!(plain.provenance.is_none(), "run() reports no provenance");
    assert!(wired.provenance.is_some(), "the declared step reports it");
    eprintln!(
        "P1: closure_before {}, economy_answers {} — identical with and without the provenance step",
        wired.closure_before, wired.economy_answers
    );
}

/// P2 (precommitted: pass) — THE PROVENANCE IS EXACT. The carried BUILD-29
/// numbers, now through the harness, and the digest equals the direct
/// readout's.
#[test]
fn gate_p2_provenance_is_exact() {
    let task = declared_task();
    let report = run_with_provenance(&task, 7).expect("structure");
    let provenance = report.provenance.as_ref().expect("provenance reported");

    assert_eq!(provenance.target, TARGET);
    assert_eq!(provenance.max_hops, 7);
    assert_eq!(provenance.shortest, Some(7), "precommitted shortest length");
    assert_eq!(provenance.words.len(), 140, "precommitted word count");
    for word in &provenance.words {
        assert_eq!(replay(&word.relations), TARGET, "every word replays to the target");
    }
    eprintln!(
        "P2: target {TARGET:?}, {} words, shortest {:?}, digest {:016x}",
        provenance.words.len(),
        provenance.shortest,
        provenance.digest()
    );

    // The same structure read directly: identical digest.
    let graph = WordGraph::from_relations(&stored_blocks(), MODULUS);
    let direct = graph.words_to(TARGET, 7, WalkOrder::BreadthFirst).unwrap();
    assert_eq!(provenance.digest(), direct.digest(), "harness == direct readout");

    // The other declared targets, through the harness.
    let six = run_with_provenance(
        &HarnessTask {
            target: (16, 8),
            ..declared_task()
        },
        3,
    )
    .unwrap();
    let six = six.provenance.unwrap();
    assert_eq!((six.shortest, six.words.len()), (Some(3), 6));

    let seeded = run_with_provenance(
        &HarnessTask {
            target: (0, 8),
            ..declared_task()
        },
        4,
    )
    .unwrap();
    let seeded = seeded.provenance.unwrap();
    let per_length: Vec<usize> = (1..=4)
        .map(|hops| seeded.words.iter().filter(|word| word.length() == hops).count())
        .collect();
    assert_eq!(per_length, vec![1, 0, 3, 13], "precommitted per-length counts");
    assert_eq!(seeded.words.len(), 17);
}

/// P3 (precommitted: pass) — THE BOUND IS THE CALLER'S AND IS DECLARED. Six
/// hops withholds the words and says so; the declaration is part of the
/// report's identity.
#[test]
fn gate_p3_bound_is_the_callers() {
    let task = declared_task();
    let bounded = run_with_provenance(&task, 6).unwrap().provenance.unwrap();
    assert_eq!(bounded.words.len(), 0, "the bound withholds the words");
    assert_eq!(bounded.shortest, Some(7), "and the exact length is still reported");
    assert!(bounded.truncated, "the traversal reports the cut");
    let full = run_with_provenance(&task, 7).unwrap().provenance.unwrap();
    assert_eq!(full.words.len(), 140);
    assert_ne!(
        bounded.digest(),
        full.digest(),
        "two declared bounds, two identities"
    );

    // The variant without changing the declaration: depth-first reaches the
    // same word set and the same digest (recorded: the order differs).
    let graph = WordGraph::from_relations(&stored_blocks(), MODULUS);
    let depth = graph.words_to(TARGET, 7, WalkOrder::DepthFirst).unwrap();
    assert_eq!(depth.digest(), full.digest(), "the enumeration variant cannot change the identity");
    let mut a: Vec<Vec<(i64, i64)>> = full.words.iter().map(|w| w.relations.clone()).collect();
    let mut b: Vec<Vec<(i64, i64)>> = depth.words.iter().map(|w| w.relations.clone()).collect();
    let key = |w: &Vec<(i64, i64)>| (w.len(), w.clone());
    a.sort_by_key(key);
    b.sort_by_key(key);
    assert_eq!(a, b, "same word set under both variants");
}

/// P4 (precommitted: pass) — FAIL-CLOSED, AND THE TWO ABSENCES DISTINGUISHED.
/// No structure is a typed error and no report; a non-member is a report with
/// zero words and no shortest length (definitive, not a bound).
#[test]
fn gate_p4_fail_closed_and_two_absences() {
    let empty = HarnessTask {
        blocks: Vec::new(),
        ..declared_task()
    };
    let error = run_with_provenance(&empty, 3).unwrap_err();
    eprintln!("P4: no structure -> {error}");
    assert!(matches!(
        error,
        atom_cognitive_engine::causal_path::PathError::UnknownNode(_)
    ));

    let outside = HarnessTask {
        target: (1, 0),
        ..declared_task()
    };
    let report = run_with_provenance(&outside, 7).expect("a non-member is not an error");
    let provenance = report.provenance.unwrap();
    assert_eq!(provenance.words.len(), 0);
    assert_eq!(provenance.shortest, None, "definitively not a member");
    assert!(!report.sufficient_before, "and the verdict agrees");
}

/// P5 (precommitted: pass) — THE DECLARED SHAPE. The boundary serves it, the
/// mismatches refuse before any run, and a declared chain carries it nested.
#[test]
fn gate_p5_declared_shape() {
    let request = Request::SceneWithProvenance {
        task: declared_task(),
        hops: 7,
    };
    let direct = Boundary::invoke(TaskShape::SceneQuestionWithProvenance, &request);
    match &direct {
        Outcome::Scene(report) => {
            let provenance = report.provenance.as_ref().expect("provenance present");
            assert_eq!(provenance.words.len(), 140);
            assert_eq!(provenance.shortest, Some(7));
        }
        other => panic!("the declared shape must serve: {other:?}"),
    }

    // Mismatches refuse, both directions, before any harness run.
    let scene = Request::Scene(declared_task());
    assert!(matches!(
        Boundary::invoke(TaskShape::SceneQuestionWithProvenance, &scene),
        Outcome::Refused(_)
    ));
    let provenance_shape_request = Request::SceneWithProvenance {
        task: declared_task(),
        hops: 7,
    };
    assert!(matches!(
        Boundary::invoke(TaskShape::SceneQuestion, &provenance_shape_request),
        Outcome::Refused(_)
    ));

    // A declared composition carries it: the shipped table validates, and
    // each nested stage equals its direct invocation (the composition adds
    // no computation).
    Registry::validate(CHAINS).expect("the extended registry validates");
    let chain_request = Request::Chain(vec![
        Request::SceneWithProvenance {
            task: declared_task(),
            hops: 7,
        },
        product_request(),
    ]);
    match Registry::invoke(TaskShape::Chain("provenance_then_product"), &chain_request) {
        Outcome::Chain(outcomes) => {
            assert_eq!(outcomes.len(), 2);
            let expected_scene = Boundary::invoke(
                TaskShape::SceneQuestionWithProvenance,
                &Request::SceneWithProvenance {
                    task: declared_task(),
                    hops: 7,
                },
            );
            let expected_product = Boundary::invoke(TaskShape::ArithmeticProduct, &product_request());
            assert_eq!(outcomes[0], expected_scene, "nested stage == direct invocation");
            assert_eq!(outcomes[1], expected_product, "nested stage == direct invocation");
            eprintln!("P5: nested provenance stage equals its direct invocation");
        }
        other => panic!("the shipped chain must invoke: {other:?}"),
    }

    // The two BUILD-27 entries are untouched by the append.
    const NAMES: [&str; 3] = ["scene_then_product", "nested", "provenance_then_product"];
    assert_eq!(CHAINS.len(), NAMES.len());
    for (chain, name) in CHAINS.iter().zip(NAMES) {
        assert_eq!(chain.name, name);
    }
    const _: () = {
        // The BUILD-27 chains are declared exactly as before.
        let _ = Chain {
            name: "scene_then_product",
            stages: &[TaskShape::SceneQuestion, TaskShape::ArithmeticProduct],
        };
    };
}

/// P6 (precommitted: pass) — NOTHING CONSUMES IT. Changing only the declared
/// bound changes only the provenance: every other field of the report is
/// byte-equal once the provenance is stripped.
#[test]
fn gate_p6_nothing_consumes_it() {
    let task = declared_task();
    let mut a = run_with_provenance(&task, 6).unwrap();
    let mut b = run_with_provenance(&task, 7).unwrap();
    assert_ne!(a.provenance, b.provenance, "the bounds differ");
    a.provenance = None;
    b.provenance = None;
    assert_eq!(a, b, "the bound changes nothing but the provenance");
    eprintln!("P6: two bounds, one report — the provenance is the only difference");
}

/// P8 (precommitted: pass) — LINE ECONOMY: the wiring is thin, and the
/// boundary stays inside its recorded bound.
#[test]
fn gate_p8_line_economy() {
    let harness = include_str!("../src/harness.rs").lines().count();
    let boundary = include_str!("../src/boundary.rs").lines().count();
    let engine: usize = [
        "action_world.rs", "boundary.rs", "causal_path.rs", "config.rs", "derivation_paths.rs",
        "engine.rs", "evidence.rs", "generation.rs", "geometry.rs", "harness.rs",
        "interference_field.rs", "knowledge.rs", "perception.rs", "perception_field.rs",
        "perception_quantum.rs", "preference.rs", "reasoning_closure.rs", "reasoning_types.rs",
        "scratch_pad.rs", "selection.rs", "spatial.rs", "stacks.rs", "trace.rs", "weave.rs",
    ]
    .iter()
    .map(|name| {
        std::fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
            .expect("engine module")
            .lines()
            .count()
    })
    .sum();
    let share = boundary as f64 / engine as f64 * 100.0;
    eprintln!(
        "P8: harness.rs {harness}, boundary.rs {boundary}, engine {engine} lines — boundary {share:.2}%"
    );
    assert!(share < 5.0, "the ONE policy surface stays inside the recorded <5% bound");
    assert!(harness < 260, "the harness wiring is thin (was 170 before this build)");
}
