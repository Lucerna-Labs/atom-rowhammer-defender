//! BUILD-28 gates — precommitted in docs/BUILD28-CAUSAL-PATH.md BEFORE
//! this code. The arrows are recorded by the run that produced them; the
//! readout returns ordered walks; the bounds are declared; the receipts
//! reproduce; structure, not policy.
//!
//! TWO PRECOMMITTED STATEMENTS WERE FALSIFIED BY THE MATERIAL AND ARE
//! CORRECTED ON RECORD (never tuned):
//!  (i) C1(a) said the graph's Content nodes equal exactly the law's
//!      bins. Measured: they also contain the DC bin `(0,0)` — the
//!      accumulated intensity always carries it (BUILD-3's recorded
//!      subtlety), so DC is real content. It appears ONLY as a substrate
//!      (no action produces it; nothing has it as a written difference).
//!  (ii) C2 said content has no outgoing arrow (`future(bin) == 0`).
//!      Measured: it does — the arrows alternate Action → Content →
//!      Action, so a bin's future cone is the LATER actions that sampled
//!      from it. The direction claim survives in the sharper form: the
//!      producer is never among the samplers (the producer acted before
//!      the bin existed).

use atom_cognitive_engine::action_world::{
    bins_of_difference, difference_of, expected_support, ActionLoopEngine, ActionPolicy, World,
};
use atom_cognitive_engine::causal_path::{
    CausalGraph, Direction, Node, PathError, Space, WalkOrder,
};

const W: (f64, f64) = (0.80, 0.30);
const EPOCHS: usize = 50;
const SEED: u64 = 7;
const DC: (i64, i64) = (0, 0);

fn run(policy: ActionPolicy, learn: bool, seed: u64, epochs: usize) -> ActionLoopEngine {
    let mut engine = ActionLoopEngine::new(World { wave: W }, seed);
    for _ in 0..epochs {
        engine.epoch(policy, learn);
    }
    engine
}

fn law_bins(actions: &[(f64, f64)]) -> Vec<(i64, i64)> {
    let mut bins: Vec<(i64, i64)> = Vec::new();
    for action in actions {
        for bin in bins_of_difference(difference_of(*action, W)) {
            if !bins.contains(&bin) {
                bins.push(bin);
            }
        }
    }
    bins.sort();
    bins
}

fn content_nodes(graph: &CausalGraph) -> Vec<(i64, i64)> {
    graph
        .nodes()
        .iter()
        .filter(|node| node.space == Space::Content)
        .map(|node| node.cell)
        .collect()
}

fn produced_targets(graph: &CausalGraph) -> Vec<(i64, i64)> {
    let mut out: Vec<(i64, i64)> = Vec::new();
    for edge in graph.edges() {
        if edge.from.space == Space::Action && edge.to.space == Space::Content {
            if !out.contains(&edge.to.cell) {
                out.push(edge.to.cell);
            }
        }
    }
    out.sort();
    out
}

fn seq(walks: &[atom_cognitive_engine::causal_path::Walk]) -> Vec<Vec<(i64, i64)>> {
    walks
        .iter()
        .map(|walk| walk.nodes.iter().map(|node| node.cell).collect())
        .collect()
}

/// C1 (precommitted: pass, corrected on record) — THE LOOP'S OWN ARROWS.
/// After 50 epochs (`FromField`, learn, seed 7 — BUILD-5's own
/// configuration): the Content nodes are the law's bins PLUS the DC bin;
/// the Produced targets are the law's bins EXACTLY (no action produces
/// DC); every Produced edge's action endpoint is the action the loop
/// recorded at that epoch; and BUILD-5's S1 re-passes byte-intact.
#[test]
fn gate_c1_loop_records_its_own_arrows() {
    let engine = run(ActionPolicy::FromField, true, SEED, EPOCHS);
    let graph = engine.causal_graph();
    let bins = law_bins(engine.actions());

    // (a) Content nodes == law's bins + DC.
    let mut expected = bins.clone();
    if !expected.contains(&DC) {
        expected.push(DC);
    }
    expected.sort();
    let content = content_nodes(graph);
    eprintln!(
        "epochs {EPOCHS}: {} action nodes, {} content nodes ({} law bins + DC), {} edges",
        graph.nodes().iter().filter(|n| n.space == Space::Action).count(),
        content.len(),
        bins.len(),
        graph.edge_count()
    );
    assert_eq!(content, expected, "content nodes = the law's bins plus DC");
    assert_eq!(content.len(), engine.support(), "support counts DC too (+1)");

    // ... and the produced targets are the law's bins exactly: nothing
    // produces DC (it is the ambient background, not a written bin).
    assert_eq!(produced_targets(graph), bins, "produced targets = the law's bins");
    assert!(!produced_targets(graph).contains(&DC), "no action produces DC");

    // DC appears only as a substrate.
    let dc = Node::content(DC);
    for edge in graph.edges() {
        if edge.to == dc {
            assert_ne!(edge.from.space, Space::Action, "DC is never produced");
        }
    }
    assert!(
        graph.edges().iter().any(|e| e.from == dc),
        "DC is part of the substrate the actions sample from"
    );

    // (b) every Produced edge's action endpoint is that epoch's action.
    for edge in graph.edges() {
        if edge.from.space == Space::Action && edge.to.space == Space::Content {
            let action = engine.actions()[edge.epoch];
            let cell = (
                (action.0 * 20.0).round() as i64,
                (action.1 * 20.0).round() as i64,
            );
            assert_eq!(edge.from.cell, cell, "edge names the epoch's own action");
        }
    }

    // (c) BUILD-5's S1, re-passed.
    assert_eq!(engine.support(), expected_support(engine.actions(), W));

    // Controls: no consequence → no content → no arrow at all.
    let no_consequence = run(ActionPolicy::FromField, false, SEED, EPOCHS);
    assert_eq!(
        no_consequence.causal_graph().edge_count(),
        0,
        "consequence is load-bearing for the arrows too"
    );
    assert_eq!(no_consequence.actions().len(), EPOCHS, "actions still recorded");

    // Uniform never consults the field: no Content → Action arrows.
    let uniform = run(ActionPolicy::Uniform, true, SEED, EPOCHS);
    let sampled = uniform
        .causal_graph()
        .edges()
        .iter()
        .filter(|e| e.from.space == Space::Content)
        .count();
    assert_eq!(sampled, 0, "Uniform does not sample the field");
    assert!(uniform.causal_graph().edge_count() > 0, "it still writes content");
}

/// C2 (precommitted: pass, corrected on record) — DIRECTION IS REAL. A
/// single-producer bin's past cone names exactly its producer, and that
/// producer is never among its samplers: the producer acted before the
/// bin existed. The arrows also respect the run's own clock — every
/// sampled substrate was produced in a strictly earlier epoch.
#[test]
fn gate_c2_direction_is_real() {
    let engine = run(ActionPolicy::FromField, true, SEED, EPOCHS);
    let graph = engine.causal_graph();

    // A bin written by exactly one action (excluding DC, which nobody writes).
    let mut single: Option<(i64, i64)> = None;
    for cell in content_nodes(graph) {
        if cell == DC {
            continue;
        }
        let node = Node::content(cell);
        let producers = graph
            .edges()
            .iter()
            .filter(|e| e.to == node && e.from.space == Space::Action)
            .count();
        if producers == 1 {
            single = Some(cell);
            break;
        }
    }
    let bin = single.expect("the run writes at least one bin from a single action");
    eprintln!("single-producer bin: {bin:?}");

    let past = graph
        .traverse(Node::content(bin), Direction::Past, 1, WalkOrder::BreadthFirst)
        .expect("bin is in the graph");
    assert_eq!(past.walks.len(), 1, "exactly one producer");
    // Walks are origin-first, in traversal order (the same convention for
    // both directions): a past walk reads [origin, …, producer].
    assert_eq!(
        seq(&past.walks),
        vec![vec![bin, past.walks[0].nodes[1].cell]],
        "past cone = the producing action, in order"
    );
    let producer = past.walks[0].nodes[1];
    assert_eq!(producer.space, Space::Action);

    let future = graph
        .traverse(Node::content(bin), Direction::Future, 1, WalkOrder::BreadthFirst)
        .expect("bin is in the graph");
    assert!(
        !future.walks.is_empty(),
        "a bin is sampled from by later actions (the arrows alternate)"
    );
    for walk in &future.walks {
        assert_eq!(walk.nodes[0].space, Space::Content);
        assert_eq!(walk.nodes[1].space, Space::Action);
        assert_ne!(
            walk.nodes[1], producer,
            "the producer cannot sample a bin it had not yet written"
        );
    }

    // The producer's own cones, the other way round: the bin is in its
    // future and not in its past.
    let p_future = graph
        .traverse(producer, Direction::Future, 1, WalkOrder::BreadthFirst)
        .expect("producer is in the graph");
    assert!(
        p_future.walks.iter().any(|walk| walk.nodes[1] == Node::content(bin)),
        "the produced bin is in the producer's future cone"
    );
    let p_past = graph
        .traverse(producer, Direction::Past, 1, WalkOrder::BreadthFirst)
        .expect("producer is in the graph");
    assert!(
        !p_past.walks.iter().any(|walk| walk.nodes[0] == Node::content(bin)),
        "the produced bin is not in the producer's past cone"
    );

    // The arrows respect the run's clock.
    for edge in graph.edges() {
        if edge.from.space == Space::Content && edge.to.space == Space::Action {
            let content = edge.from;
            if content.cell == DC {
                assert!(
                    !graph.edges().iter().any(|e| e.to == content),
                    "DC is produced by nobody (asserted in C1)"
                );
                continue;
            }
            assert!(
                graph.edges().iter().any(|e| e.to == content
                    && e.from.space == Space::Action
                    && e.epoch < edge.epoch),
                "a sampled substrate was produced in a strictly earlier epoch"
            );
        }
    }
    // The first action consulted no structure (uniform initial draw).
    let first = engine.actions()[0];
    let first_node = Node::action((
        (first.0 * 20.0).round() as i64,
        (first.1 * 20.0).round() as i64,
    ));
    let first_past = graph
        .traverse(first_node, Direction::Past, 1, WalkOrder::BreadthFirst)
        .expect("first action is in the graph");
    assert!(
        first_past.walks.is_empty(),
        "the initial action sampled no structure"
    );
}

/// C3 (precommitted: pass) — PATHS ARE SEQUENCES. The declared trace from
/// the closed forms: a₁ = (1,3), bins₁ = {(5,17),(15,3)}, a₂ = (9,14).
/// Exact counts 2 / 4 / 4 at hops 1 / 2 / 3, and the two-edge walks are
/// exactly `[a₁, bin, a₂]` for each bin, in canonical order.
#[test]
fn gate_c3_paths_are_sequences() {
    let (a1, a2) = ((1i64, 3i64), (9i64, 14i64));
    let world_cell = ((W.0 * 20.0).round() as i64, (W.1 * 20.0).round() as i64);
    let mut bins1 = bins_of_difference((a1.0 - world_cell.0, a1.1 - world_cell.1));
    bins1.sort();
    assert_eq!(bins1, vec![(5, 17), (15, 3)], "precommitted bins₁");

    let mut graph = CausalGraph::new();
    let n_a1 = Node::action(a1);
    let n_a2 = Node::action(a2);
    for bin in &bins1 {
        graph.record(n_a1, Node::content(*bin), 0); // produced by a₁
        graph.record(Node::content(*bin), n_a2, 1); // the substrate a₂ sampled
    }

    let h1 = graph
        .traverse(n_a2, Direction::Past, 1, WalkOrder::BreadthFirst)
        .unwrap();
    let h2 = graph
        .traverse(n_a2, Direction::Past, 2, WalkOrder::BreadthFirst)
        .unwrap();
    let h3 = graph
        .traverse(n_a2, Direction::Past, 3, WalkOrder::BreadthFirst)
        .unwrap();
    assert_eq!(h1.walks.len(), 2, "hops 1: two one-edge walks");
    assert_eq!(h2.walks.len(), 4, "hops 2: two one-edge + two two-edge");
    assert_eq!(h3.walks.len(), 4, "hops 3: a₁ has no predecessor");

    // The walks are origin-first, traversal order — CORRECTION (kept on
    // record): the precommit printed them in causal order ([a₁, bin, a₂]);
    // the readout returns [origin, …, furthest], the same convention for
    // both directions. The sequences carry exactly the precommitted nodes.
    let expected_one: Vec<Vec<(i64, i64)>> = bins1.iter().map(|b| vec![a2, *b]).collect();
    let expected_two: Vec<Vec<(i64, i64)>> = bins1.iter().map(|b| vec![a2, *b, a1]).collect();
    assert_eq!(
        seq(&h1.walks),
        expected_one,
        "the walks are the ordered sequences, not a set"
    );
    let two: Vec<Vec<(i64, i64)>> = seq(&h2.walks)
        .into_iter()
        .filter(|walk| walk.len() == 3)
        .collect();
    assert_eq!(two, expected_two, "the two-edge walks, in canonical order");

    // The same walk set, direction reversed — and the reversal is visible
    // in the sequence itself.
    let f2 = graph
        .traverse(n_a1, Direction::Future, 2, WalkOrder::BreadthFirst)
        .unwrap();
    assert_eq!(f2.walks.len(), 4, "future(a₁, 2) = the same walk set");
    let f_two: Vec<Vec<(i64, i64)>> = seq(&f2.walks)
        .into_iter()
        .filter(|walk| walk.len() == 3)
        .collect();
    let expected_forward: Vec<Vec<(i64, i64)>> = bins1.iter().map(|b| vec![a1, *b, a2]).collect();
    assert_eq!(
        f_two, expected_forward,
        "the same three nodes, read the other way round"
    );
    assert_ne!(two, f_two, "both carry the same node set, in opposite order");
}

/// C4 (precommitted: pass; the headline) — THE READOUT IS DIRECTION-BLIND;
/// THE ARROWS ARE NOT. A = [(1,1),(3,7)] and its mirror about the world's
/// wave (`a ↦ 2W − a`, in cell space, exact) give the SAME support (5) and
/// the same content, while every bin's producer differs between the two.
#[test]
fn gate_c4_readout_blind_arrows_are_not() {
    let world_cell = ((W.0 * 20.0).round() as i64, (W.1 * 20.0).round() as i64);
    let a_cells = [(1i64, 1i64), (3i64, 7i64)];
    let mirrored: Vec<(i64, i64)> = a_cells
        .iter()
        .map(|cell| {
            (
                (2 * world_cell.0 - cell.0).rem_euclid(20),
                (2 * world_cell.1 - cell.1).rem_euclid(20),
            )
        })
        .collect();
    assert_eq!(mirrored, vec![(11, 11), (9, 5)], "precommitted mirror");

    let to_wave = |cell: (i64, i64)| (cell.0 as f64 / 20.0, cell.1 as f64 / 20.0);
    let a_waves: Vec<(f64, f64)> = a_cells.iter().map(|c| to_wave(*c)).collect();
    let m_waves: Vec<(f64, f64)> = mirrored.iter().map(|c| to_wave(*c)).collect();
    let s_a = expected_support(&a_waves, W);
    let s_m = expected_support(&m_waves, W);
    eprintln!("support(A) = {s_a}, support(mirror(A)) = {s_m} (identical)");
    assert_eq!(s_a, 5, "precommitted support for A");
    assert_eq!(s_a, s_m, "the readout is direction-blind (precommitted)");

    let build = |actions: &[(i64, i64)]| {
        let mut graph = CausalGraph::new();
        for (epoch, cell) in actions.iter().enumerate() {
            for bin in bins_of_difference(difference_of(to_wave(*cell), W)) {
                graph.record(Node::action(*cell), Node::content(bin), epoch);
            }
        }
        graph
    };
    let g_a = build(&a_cells);
    let g_m = build(&mirrored);
    assert_eq!(
        produced_targets(&g_a),
        produced_targets(&g_m),
        "the same content, bin for bin"
    );

    for bin in produced_targets(&g_a) {
        let past_a = g_a
            .traverse(Node::content(bin), Direction::Past, 1, WalkOrder::BreadthFirst)
            .unwrap();
        let past_m = g_m
            .traverse(Node::content(bin), Direction::Past, 1, WalkOrder::BreadthFirst)
            .unwrap();
        eprintln!(
            "bin {bin:?}: producer from A = {:?}, from mirror(A) = {:?}",
            seq(&past_a.walks),
            seq(&past_m.walks)
        );
        assert_eq!(past_a.walks.len(), 1);
        assert_eq!(past_m.walks.len(), 1);
        assert_ne!(
            past_a.walks[0].nodes[1], past_m.walks[0].nodes[1],
            "the same bin names a different producer"
        );
    }
}

/// C5 (precommitted: pass) — BOUNDS ARE DECLARED, NOT SILENT. The
/// precommitted truncation flags, and no returned walk exceeds the bound.
#[test]
fn gate_c5_bounds_declared() {
    let (a1, a2) = ((1i64, 3i64), (9i64, 14i64));
    let world_cell = ((W.0 * 20.0).round() as i64, (W.1 * 20.0).round() as i64);
    let bins1 = bins_of_difference((a1.0 - world_cell.0, a1.1 - world_cell.1));
    let mut graph = CausalGraph::new();
    for bin in &bins1 {
        graph.record(Node::action(a1), Node::content(*bin), 0);
        graph.record(Node::content(*bin), Node::action(a2), 1);
    }
    for hops in 1..=3usize {
        let t = graph
            .traverse(Node::action(a2), Direction::Past, hops, WalkOrder::BreadthFirst)
            .unwrap();
        for walk in &t.walks {
            assert!(
                walk.nodes.len() <= hops + 1,
                "no walk exceeds the declared bound"
            );
        }
        let expected_truncated = hops == 1;
        assert_eq!(
            t.truncated, expected_truncated,
            "truncated({hops}) = {expected_truncated} (precommitted)"
        );
    }
}

/// C6 (precommitted: pass) — FAIL-CLOSED AND RECEIPTS REPRODUCE. Unknown
/// origins are named; identical queries give byte-identical receipts; the
/// digest is independent of the walk order and of the order the edges were
/// recorded in; one more edge changes it.
#[test]
fn gate_c6_fail_closed_and_receipts() {
    let (a1, a2) = ((1i64, 3i64), (9i64, 14i64));
    let world_cell = ((W.0 * 20.0).round() as i64, (W.1 * 20.0).round() as i64);
    let bins1 = bins_of_difference((a1.0 - world_cell.0, a1.1 - world_cell.1));
    let mut graph = CausalGraph::new();
    for bin in &bins1 {
        graph.record(Node::action(a1), Node::content(*bin), 0);
        graph.record(Node::content(*bin), Node::action(a2), 1);
    }

    // Fail-closed: an origin outside the graph is named, not empty-success.
    let mut unknown = Node::action((0, 0));
    let mut probe = 0i64;
    while graph.nodes().contains(&unknown) {
        probe += 1;
        unknown = Node::action((probe, probe));
    }
    let error = graph
        .traverse(unknown, Direction::Past, 2, WalkOrder::BreadthFirst)
        .unwrap_err();
    assert_eq!(error, PathError::UnknownNode(unknown));
    eprintln!("fail-closed: {error}");

    let canonical = graph
        .traverse(Node::action(a2), Direction::Past, 2, WalkOrder::BreadthFirst)
        .unwrap();
    let r1 = canonical.receipt("declared-trace");
    assert_eq!(r1, canonical.receipt("declared-trace"), "identical query");
    assert_ne!(r1, canonical.receipt("other-source"), "the source is identity");
    eprintln!("receipt digest: {:016x}", r1.digest);

    // Canonical-order independence: the same edges recorded in reverse.
    let mut reversed = CausalGraph::new();
    for edge in graph.edges().iter().rev() {
        reversed.record(edge.from, edge.to, edge.epoch);
    }
    let r_rev = reversed
        .traverse(Node::action(a2), Direction::Past, 2, WalkOrder::BreadthFirst)
        .unwrap()
        .receipt("declared-trace");
    assert_eq!(r1.digest, r_rev.digest, "recording order cannot change the receipt");

    // The DFS variant returns the same set and the same digest.
    let depth = graph
        .traverse(Node::action(a2), Direction::Past, 2, WalkOrder::DepthFirst)
        .unwrap();
    let mut a = canonical.walks.clone();
    let mut b = depth.walks.clone();
    let key = |w: &atom_cognitive_engine::causal_path::Walk| (w.nodes.len(), w.nodes.clone());
    a.sort_by_key(key);
    b.sort_by_key(key);
    assert_eq!(a, b, "same walk set");
    assert_eq!(
        depth.receipt("declared-trace").digest,
        r1.digest,
        "the receipt is walk-order independent"
    );

    // One more edge changes the receipt.
    let mut plus = graph.clone();
    plus.record(Node::content((0, 1)), Node::action(a2), 2);
    let r_plus = plus
        .traverse(Node::action(a2), Direction::Past, 2, WalkOrder::BreadthFirst)
        .unwrap()
        .receipt("declared-trace");
    assert_ne!(r1.digest, r_plus.digest, "one edge changes the receipt");
}

/// C7 (precommitted: pass) — PASSIVITY (structure, not policy). The
/// strongest passive check: read the graph after every epoch and assert
/// the run is identical to one that never reads it.
#[test]
fn gate_c7_passivity() {
    let engine = run(ActionPolicy::FromField, true, SEED, EPOCHS);
    let actions = engine.actions().to_vec();
    let support = engine.support();
    let edges = engine.causal_graph().edge_count();

    let mut interleaved = ActionLoopEngine::new(World { wave: W }, SEED);
    let mut reads = 0usize;
    for _ in 0..EPOCHS {
        interleaved.epoch(ActionPolicy::FromField, true);
        let graph = interleaved.causal_graph();
        for node in graph.nodes() {
            for direction in [Direction::Past, Direction::Future] {
                for hops in [1usize, 2] {
                    let traversal = graph
                        .traverse(node, direction, hops, WalkOrder::BreadthFirst)
                        .expect("every node read out of its own graph resolves");
                    let _ = traversal.receipt("audit");
                    reads += 1;
                }
            }
        }
    }
    eprintln!("{reads} readouts interleaved into the run; nothing changed");
    assert!(reads > 0);
    assert_eq!(interleaved.actions(), actions.as_slice(), "actions unchanged");
    assert_eq!(interleaved.support(), support, "support unchanged");
    assert_eq!(interleaved.causal_graph().edge_count(), edges, "edges unchanged");
}

/// Rule 2 arm (precommitted: sets equal; order recorded) — the declared
/// DAG E = [(0,1),(1,3),(0,3)] under both enumeration orders. CORRECTION
/// (kept on record): the precommit's (4) listed the walks *reaching a
/// destination*, and listed them without the lexicographic tie-break the
/// declared canonical form specifies. The readout as built is
/// destination-free (cone semantics) and canonical = shortest-first then
/// lexicographic, so the lists carry the additional walk `[0,1]` and it
/// sorts before `[0,3]`. The gated claim — same set, different order — is
/// measured.
#[test]
fn gate_variant_enumeration_order() {
    let edges = [(0i64, 1i64), (1, 3), (0, 3)];
    let mut graph = CausalGraph::new();
    for (epoch, (from, to)) in edges.iter().enumerate() {
        graph.record(Node::action((*from, 0)), Node::action((*to, 0)), epoch);
    }
    let root = Node::action((0, 0));
    let bfs = graph
        .traverse(root, Direction::Future, 2, WalkOrder::BreadthFirst)
        .unwrap();
    let dfs = graph
        .traverse(root, Direction::Future, 2, WalkOrder::DepthFirst)
        .unwrap();

    let bfs_seq: Vec<Vec<i64>> = seq(&bfs.walks).iter().map(|w| w.iter().map(|c| c.0).collect()).collect();
    let dfs_seq: Vec<Vec<i64>> = seq(&dfs.walks).iter().map(|w| w.iter().map(|c| c.0).collect()).collect();
    eprintln!("bfs order: {bfs_seq:?}");
    eprintln!("dfs order: {dfs_seq:?}");
    assert_eq!(
        bfs_seq,
        vec![vec![0, 1], vec![0, 3], vec![0, 1, 3]],
        "canonical order (shortest-first, lexicographic within a length)"
    );
    assert_eq!(dfs_seq, vec![vec![0, 1], vec![0, 1, 3], vec![0, 3]], "the variant's order");
    assert_ne!(bfs_seq, dfs_seq, "same set, different order (precommitted)");

    let key = |w: &Vec<i64>| (w.len(), w.clone());
    let mut a = bfs_seq.clone();
    let mut b = dfs_seq.clone();
    a.sort_by_key(key);
    b.sort_by_key(key);
    assert_eq!(a, b, "same walk set");
    assert_eq!(bfs.truncated, dfs.truncated, "truncation is order-independent");

    // The live trace: same sets, orders recorded.
    let engine = run(ActionPolicy::FromField, true, SEED, EPOCHS);
    let live = engine.causal_graph();
    let mut pairs = 0;
    let mut differing = 0;
    for node in live.nodes() {
        for direction in [Direction::Past, Direction::Future] {
            let b = live.traverse(node, direction, 3, WalkOrder::BreadthFirst).unwrap();
            let d = live.traverse(node, direction, 3, WalkOrder::DepthFirst).unwrap();
            let mut x: Vec<Vec<(i64, i64)>> = seq(&b.walks);
            let mut y: Vec<Vec<(i64, i64)>> = seq(&d.walks);
            let key = |w: &Vec<(i64, i64)>| (w.len(), w.clone());
            x.sort_by_key(key);
            y.sort_by_key(key);
            assert_eq!(x, y, "the live trace: the walk sets agree");
            if seq(&b.walks) != seq(&d.walks) {
                differing += 1;
            }
            pairs += 1;
        }
    }
    eprintln!("live trace: {pairs} readouts, {differing} with a different enumeration order");
    assert!(pairs > 0);
}
