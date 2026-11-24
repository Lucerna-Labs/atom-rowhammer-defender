//! BUILD-29 gates — precommitted in docs/BUILD29-DERIVATION-PATHS.md BEFORE
//! this code. The closure answers *whether*; the words answer *how*. Every
//! number below was verified in Python first, by two independent methods
//! (a DP over the group algebra and a brute force over all sequences).

use atom_block_library::{relations_for_derivation, BLOCKS};
use atom_cognitive_engine::causal_path::{Node, PathError, WalkOrder};
use atom_cognitive_engine::derivation_paths::WordGraph;
use atom_cognitive_engine::knowledge::{derivable, derivable_count};

const MODULUS: i64 = 20;

/// The stored relation blocks across the two derivation partitions that
/// carry relations: `geometry` (3, BUILD-2's recorded scene differences) and
/// `geometry.extended` (4, the same three plus `(6,0)`).
fn stored_relation_blocks() -> Vec<(i64, i64)> {
    let mut all = relations_for_derivation(BLOCKS, "geometry").expect("geometry");
    all.extend(relations_for_derivation(BLOCKS, "geometry.extended").expect("extended"));
    all
}

fn stored_generators() -> Vec<(i64, i64)> {
    relations_for_derivation(BLOCKS, "geometry.extended").expect("extended")
}

/// The precommitted shortest-word table for the stored graph (all 50
/// members).
const SHORTEST_TABLE: &[((i64, i64), usize)] = &[
    ((0, 0), 0), ((0, 4), 2), ((0, 8), 1), ((0, 12), 2), ((0, 16), 2),
    ((2, 0), 3), ((2, 4), 5), ((2, 8), 4), ((2, 12), 3), ((2, 16), 5),
    ((4, 0), 4), ((4, 4), 6), ((4, 8), 5), ((4, 12), 6), ((4, 16), 6),
    ((6, 0), 1), ((6, 4), 3), ((6, 8), 2), ((6, 12), 3), ((6, 16), 3),
    ((8, 0), 4), ((8, 4), 6), ((8, 8), 5), ((8, 12), 4), ((8, 16), 6),
    ((10, 0), 1), ((10, 4), 3), ((10, 8), 2), ((10, 12), 1), ((10, 16), 3),
    ((12, 0), 2), ((12, 4), 4), ((12, 8), 3), ((12, 12), 4), ((12, 16), 4),
    ((14, 0), 5), ((14, 4), 7), ((14, 8), 6), ((14, 12), 5), ((14, 16), 7),
    ((16, 0), 2), ((16, 4), 4), ((16, 8), 3), ((16, 12), 2), ((16, 16), 4),
    ((18, 0), 3), ((18, 4), 5), ((18, 8), 4), ((18, 12), 5), ((18, 16), 5),
];

fn replay(relations: &[(i64, i64)]) -> (i64, i64) {
    let mut sum = (0i64, 0i64);
    for (x, y) in relations {
        sum = ((sum.0 + x).rem_euclid(MODULUS), (sum.1 + y).rem_euclid(MODULUS));
    }
    sum
}

/// W1 (precommitted: pass) — THE WORD GRAPH IS THE CLOSURE. Membership
/// agrees cell for cell with the existing machinery over all 400 cells; the
/// recorded closure numbers reproduce; and the dedup is exact.
#[test]
fn gate_w1_word_graph_is_the_closure() {
    let blocks = stored_relation_blocks();
    assert_eq!(blocks.len(), 7, "seven stored relation blocks");
    let graph = WordGraph::from_relations(&blocks, MODULUS);
    eprintln!(
        "7 stored relation blocks -> {} distinct relations: {:?}",
        graph.generators().len(),
        graph.generators()
    );
    assert_eq!(
        graph.generators(),
        &[(0, 8), (6, 0), (10, 0), (10, 12)],
        "the deduped generator set (precommitted)"
    );
    assert_eq!(graph.member_count(), 50, "the closure of the stored relations");

    // 400/400 agreement with `derivable` — the same set, cell for cell.
    let generators = graph.generators().to_vec();
    let mut cells = 0;
    for x in 0..MODULUS {
        for y in 0..MODULUS {
            assert_eq!(
                graph.has_member((x, y)),
                derivable(&generators, (x, y)),
                "membership agrees at cell ({x},{y})"
            );
            cells += 1;
        }
    }
    assert_eq!(cells, 400);

    // The recorded closure numbers, reproduced through the new structure.
    let geometry = relations_for_derivation(BLOCKS, "geometry").unwrap();
    let extended = relations_for_derivation(BLOCKS, "geometry.extended").unwrap();
    assert_eq!(derivable_count(&geometry), 10, "recorded: geometry closure 10");
    assert_eq!(derivable_count(&extended), 50, "recorded: extended closure 50");
    let mut acquired = extended.clone();
    acquired.push((1, 0));
    assert_eq!(derivable_count(&acquired), 100, "recorded: acquired closure 100");
    assert_eq!(WordGraph::from_relations(&geometry, MODULUS).member_count(), 10);
}

/// W2 (precommitted: pass) — THE WORDS ARE EXACT. Every precommitted number:
/// the shortest lengths, the 140 words of `(14,16)`, the single word of
/// `(0,8)`, and the whole 50-member shortest table.
#[test]
fn gate_w2_words_are_exact() {
    let graph = WordGraph::from_relations(&stored_generators(), MODULUS);

    let headline = graph.words_to((14, 16), 7, WalkOrder::BreadthFirst).unwrap();
    eprintln!(
        "(14,16): shortest {:?}, words {} (precommitted 7 / 140)",
        headline.shortest,
        headline.words.len()
    );
    assert_eq!(headline.shortest, Some(7));
    assert_eq!(headline.words.len(), 140, "140 distinct words");
    for word in &headline.words {
        assert_eq!(word.length(), 7, "every word is at the shortest length");
        assert_eq!(replay(&word.relations), (14, 16), "each word replays to the target");
        assert_eq!(word.members[0], (0, 0), "words start at the identity");
        assert_eq!(*word.members.last().unwrap(), (14, 16));
        assert_eq!(word.members.len(), word.relations.len() + 1);
    }
    let mut sequences: Vec<Vec<(i64, i64)>> =
        headline.words.iter().map(|word| word.relations.clone()).collect();
    sequences.sort();
    let distinct = sequences.len();
    sequences.dedup();
    assert_eq!(sequences.len(), distinct, "the 140 words are pairwise distinct sequences");

    // No word of any shorter length, and the exact shortest length does not
    // move with the bound.
    for hops in 1..7usize {
        let short = graph.words_to((14, 16), hops, WalkOrder::BreadthFirst).unwrap();
        assert_eq!(short.words.len(), 0, "no word of length <= {hops}");
        assert_eq!(short.shortest, Some(7), "the exact shortest length is bound-independent");
    }

    // The stored relation itself: one word of length 1, and the precommitted
    // counts above it. CORRECTION (kept on record): my first version of this
    // gate asserted 1 word — the count at the SHORTEST length — while the
    // readout returns every word of length 1..=max_hops (17 for `(0,8)` at
    // four hops, per the Python table `{1:1, 2:0, 3:3, 4:13}`).
    let seeded = graph.words_to((0, 8), 4, WalkOrder::BreadthFirst).unwrap();
    let per_length: Vec<usize> = (1..=4)
        .map(|hops| seeded.words.iter().filter(|word| word.length() == hops).count())
        .collect();
    assert_eq!(seeded.shortest, Some(1));
    assert_eq!(per_length, vec![1, 0, 3, 13], "precommitted per-length counts");
    assert_eq!(seeded.words.len(), 17);
    let single: Vec<&atom_cognitive_engine::derivation_paths::Word> =
        seeded.words.iter().filter(|word| word.length() == 1).collect();
    assert_eq!(single.len(), 1);
    assert_eq!(single[0].relations, vec![(0, 8)]);
    assert_eq!(single[0].members, vec![(0, 0), (0, 8)]);

    // The whole precommitted shortest table.
    assert_eq!(SHORTEST_TABLE.len(), 50, "the table covers every member");
    for (cell, hops) in SHORTEST_TABLE {
        assert_eq!(
            graph.shortest_length(*cell),
            Some(*hops),
            "shortest word length to {cell:?}"
        );
    }
    assert_eq!(graph.shortest_length((1, 0)), None, "(1,0) is not a member");

    // The identity: the empty word derives it (length 0), and the
    // enumeration lists the non-trivial words that return to it — measured,
    // precommitted counts 0 / 1 / 6 / 7 at lengths 1..4.
    let origin = graph.words_to((0, 0), 4, WalkOrder::BreadthFirst).unwrap();
    let cycles: Vec<usize> = (1..=4)
        .map(|hops| origin.words.iter().filter(|word| word.length() == hops).count())
        .collect();
    assert_eq!(origin.shortest, Some(0), "the empty word derives the identity");
    assert_eq!(cycles, vec![0, 1, 6, 7], "precommitted cycle counts");
    for word in &origin.words {
        assert_eq!(replay(&word.relations), (0, 0));
    }
}

/// W3 (precommitted: pass; the headline) — THE BIT CANNOT EXPRESS THE WORDS.
/// One membership bit, 140 derivations; and two members with the same
/// shortest length carry different word counts.
#[test]
fn gate_w3_the_bit_cannot_express_the_words() {
    let graph = WordGraph::from_relations(&stored_generators(), MODULUS);
    let generators = graph.generators().to_vec();

    let many = graph.words_to((14, 16), 7, WalkOrder::BreadthFirst).unwrap();
    for word in &many.words {
        assert!(
            derivable(&generators, replay(&word.relations)),
            "every word lands on a member the closure reports with one bit"
        );
    }
    eprintln!(
        "member (14,16): derivable = {} (one bit), derivations = {}",
        derivable(&generators, (14, 16)),
        many.words.len()
    );

    // Same shortest length, different counts — a count is a property of the
    // set, never of a route.
    let a = graph.words_to((0, 4), 2, WalkOrder::BreadthFirst).unwrap();
    let b = graph.words_to((0, 12), 2, WalkOrder::BreadthFirst).unwrap();
    assert_eq!(a.shortest, b.shortest, "same shortest length");
    assert_eq!((a.words.len(), b.words.len()), (1, 2), "different word counts (precommitted)");
    assert_ne!(a.digest(), b.digest(), "different routes, different identity");
}

/// W4 (precommitted: pass) — THE ORDER IS FREE, THE PATH IS NOT (the Rule 2
/// arm). All six permutations of the declared word reach the same member,
/// and they are exactly the six words of that length.
#[test]
fn gate_w4_order_is_free_the_path_is_not() {
    let graph = WordGraph::from_relations(&stored_generators(), MODULUS);
    let word = [(0i64, 8i64), (6, 0), (10, 0)];
    assert_eq!(replay(&word), (16, 8), "the declared word's target");

    let permutations: Vec<Vec<(i64, i64)>> = vec![
        vec![(0, 8), (6, 0), (10, 0)],
        vec![(0, 8), (10, 0), (6, 0)],
        vec![(6, 0), (0, 8), (10, 0)],
        vec![(6, 0), (10, 0), (0, 8)],
        vec![(10, 0), (0, 8), (6, 0)],
        vec![(10, 0), (6, 0), (0, 8)],
    ];
    let words = graph.words_to((16, 8), 3, WalkOrder::BreadthFirst).unwrap();
    assert_eq!(words.shortest, Some(3));
    assert_eq!(words.words.len(), 6, "exactly six words of length three");
    for permutation in &permutations {
        assert!(
            words.words.iter().any(|candidate| candidate.relations == *permutation),
            "the permutation {permutation:?} is present"
        );
        assert_eq!(replay(permutation), (16, 8), "and it reaches the same member");
    }
    eprintln!("6 permutations of {word:?} all reach (16,8): the order is free, the path is not");
}

/// W5 (precommitted: pass) — BOUNDS ARE DECLARED, NOT SILENT. A bound below
/// the shortest length withholds the words and says so; it never reads as
/// "no derivation exists".
#[test]
fn gate_w5_bounds_declared() {
    let graph = WordGraph::from_relations(&stored_generators(), MODULUS);
    let bounded = graph.words_to((14, 16), 6, WalkOrder::BreadthFirst).unwrap();
    assert_eq!(bounded.words.len(), 0, "the bound withholds the words");
    assert_eq!(bounded.shortest, Some(7), "and the readout still reports the exact length");
    assert!(bounded.truncated, "the traversal reports that the bound cut structure");
    let full = graph.words_to((14, 16), 7, WalkOrder::BreadthFirst).unwrap();
    assert_eq!(full.words.len(), 140);
    assert_ne!(bounded.digest(), full.digest(), "the bound is part of the identity");
}

/// W6 (precommitted: pass) — FAIL-CLOSED AND REPRODUCIBLE. A non-member is
/// definitive; no structure at all is the primitive's typed refusal; the
/// digest is stable, variant-independent, and bound-sensitive.
#[test]
fn gate_w6_fail_closed_and_reproducible() {
    let graph = WordGraph::from_relations(&stored_generators(), MODULUS);

    let outside = graph.words_to((1, 0), 7, WalkOrder::BreadthFirst).unwrap();
    assert_eq!(
        (outside.words.len(), outside.shortest),
        (0, None),
        "a non-member has no words, definitively"
    );

    // An empty library has one member (the identity — BUILD-17's recorded
    // correction) and no steps at all: the primitive's typed refusal.
    let empty = WordGraph::from_relations(&[], MODULUS);
    assert_eq!(empty.member_count(), 1);
    assert_eq!(empty.generators().len(), 0);
    assert_eq!(
        empty.words_to((0, 0), 3, WalkOrder::BreadthFirst).unwrap_err(),
        PathError::UnknownNode(Node::content((0, 0))),
        "no structure, no readout"
    );

    // Digits: stable under repetition, independent of the enumeration
    // variant, sensitive to the bound and the target.
    let canonical = graph.words_to((14, 16), 7, WalkOrder::BreadthFirst).unwrap();
    let repeat = graph.words_to((14, 16), 7, WalkOrder::BreadthFirst).unwrap();
    assert_eq!(canonical.digest(), repeat.digest());
    let depth = graph.words_to((14, 16), 7, WalkOrder::DepthFirst).unwrap();
    assert_eq!(canonical.digest(), depth.digest(), "the receipt is walk-order independent");
    let mut a: Vec<Vec<(i64, i64)>> = canonical.words.iter().map(|w| w.relations.clone()).collect();
    let mut b: Vec<Vec<(i64, i64)>> = depth.words.iter().map(|w| w.relations.clone()).collect();
    let key = |w: &Vec<(i64, i64)>| (w.len(), w.clone());
    a.sort_by_key(key);
    b.sort_by_key(key);
    assert_eq!(a, b, "the variant returns the same word set");
    assert_ne!(
        canonical.digest(),
        graph.words_to((16, 8), 7, WalkOrder::BreadthFirst).unwrap().digest(),
        "a different target is a different identity"
    );
    eprintln!("digest (14,16,7): {:016x} (stable, variant-independent)", canonical.digest());
}

/// W7 (precommitted: pass) — THE REACH-OUT HAS A WORD. Acquisition buys a
/// route, not just a bit.
#[test]
fn gate_w7_reach_out_has_a_word() {
    let stored = stored_generators();
    let before = WordGraph::from_relations(&stored, MODULUS);
    let none = before.words_to((1, 0), 7, WalkOrder::BreadthFirst).unwrap();
    assert_eq!((before.member_count(), none.words.len(), none.shortest), (50, 0, None));

    let mut acquired = stored.clone();
    acquired.push((1, 0));
    let after = WordGraph::from_relations(&acquired, MODULUS);
    assert_eq!(after.member_count(), 100, "the closure doubles exactly (recorded)");
    let words = after.words_to((1, 0), 7, WalkOrder::BreadthFirst).unwrap();
    let per_length: Vec<usize> = (1..=7)
        .map(|hops| words.words.iter().filter(|word| word.length() == hops).count())
        .collect();
    eprintln!(
        "after acquiring (1,0): closure {}, shortest {:?}, words by length {per_length:?}",
        after.member_count(),
        words.shortest
    );
    assert_eq!(words.shortest, Some(1));
    assert_eq!(per_length, vec![1, 0, 3, 24, 35, 146, 763], "precommitted per-length counts");
    assert_eq!(words.words.len(), 972);
    for word in &words.words {
        assert_eq!(replay(&word.relations), (1, 0));
    }
}

/// W8 (precommitted: pass) — SEPARATION (structure, not sufficiency). The
/// existing membership machinery is unchanged by the word readout, and the
/// words answer *how*, never *whether*.
#[test]
fn gate_w8_separation() {
    let graph = WordGraph::from_relations(&stored_generators(), MODULUS);
    let generators = graph.generators().to_vec();
    let before = derivable_count(&generators);
    assert_eq!(before, 50);

    // Enumerate words, then ask the old machinery the same questions.
    let _ = graph.words_to((14, 16), 7, WalkOrder::BreadthFirst).unwrap();
    let _ = graph.words_to((16, 8), 3, WalkOrder::DepthFirst).unwrap();
    let _ = graph.words_to((1, 0), 7, WalkOrder::BreadthFirst).unwrap();
    assert_eq!(derivable_count(&generators), before, "the closure is unchanged");
    for cell in [(0, 0), (0, 8), (10, 12), (6, 0), (14, 16), (16, 8), (1, 0), (19, 19)] {
        assert_eq!(derivable(&generators, cell), graph.has_member(cell));
    }

    // The words are answers to "how": every word is a valid member-to-member
    // sequence, and no readout returns a sufficiency verdict.
    let words = graph.words_to((16, 8), 3, WalkOrder::BreadthFirst).unwrap();
    for word in &words.words {
        for member in &word.members {
            assert!(graph.has_member(*member), "every visited member is in the closure");
        }
    }
    assert!(
        words.words.iter().all(|word| word.relations.iter().all(|r| graph.generators().contains(r))),
        "every step is a stored relation"
    );
}
