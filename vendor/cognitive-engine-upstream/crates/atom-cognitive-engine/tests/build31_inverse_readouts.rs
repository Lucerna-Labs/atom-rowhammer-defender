//! BUILD-31 gates — precommitted in docs/BUILD31-INVERSE-READOUTS.md BEFORE
//! this code. Every number was verified in Python first; the readouts are
//! measurements (a counted order, an equality, a path length), and the gates
//! cross-check them against the engine's own machinery and the march's own
//! readout.

use atom_cognitive_engine::causal_path::WalkOrder;
use atom_cognitive_engine::derivation_paths::WordGraph;
use atom_cognitive_engine::knowledge::{
    divides, gcd_of, generated_product, modular_inverse, quotient_within,
};

const M20: u64 = 1 << 20;
const M64: u64 = 64;

/// D1 (precommitted: pass) — GCD IS A COUNTED ORDER. The eight declared pairs
/// read exactly; the boundaries fail closed where the structure does not
/// exist; and the reading IS the closure's order inverted.
#[test]
fn gate_d1_gcd_is_a_counted_order() {
    // (a, b) → gcd, Python-verified against math.gcd.
    let pairs: [(u64, u64, u64); 8] = [
        (12, 20, 4),
        (638, M20, 2),
        (997, M20, 1),
        (12, M20, 4),
        (6, 20, 2),
        (15, 20, 5),
        (7, 100, 1),
        (100, 7, 1),
    ];
    for (a, b, expected) in pairs {
        assert_eq!(gcd_of(a, b), Some(expected), "gcd({a}, {b})");
    }

    // The boundaries.
    assert_eq!(gcd_of(0, 20), Some(20), "the zero generator's subgroup is trivial");
    assert_eq!(gcd_of(1, 20), Some(1));
    assert_eq!(gcd_of(20, 20), Some(20), "the generator is the modulus: one step returns");
    assert_eq!(gcd_of(5, 0), None, "no register, no reading (fail-closed)");

    // The reading IS the closure order inverted — not a re-implementation.
    let order = WordGraph::from_register(20, 15).member_count() as u64;
    assert_eq!(gcd_of(15, 20), Some(20 / order), "gcd = modulus / subgroup order");
    eprintln!(
        "D1: gcd(12,20)={:?} gcd(15,20)={:?} gcd(638,2^20)={:?} gcd(0,20)={:?} — the order 20/{} = 5",
        gcd_of(12, 20),
        gcd_of(15, 20),
        gcd_of(638, M20),
        gcd_of(0, 20),
        order
    );
}

/// D2 (precommitted: pass) — DIVISIBILITY IS REACHABILITY. Reachability of
/// the residue equals `gcd(b, m) | t`, and `divides` equals plain integer
/// divisibility on the declared pairs.
#[test]
fn gate_d2_divisibility_is_reachability() {
    // m = 2^20, b = 638 (gcd 2): even residues reach, odd do not.
    let reachable = WordGraph::from_register(M20 as i64, 638);
    assert_eq!(reachable.member_count(), (M20 / 2) as usize, "the subgroup of step 2");
    for (t, expected) in [(636086u64, true), (318042, true), (997, false), (318043, false)] {
        assert_eq!(
            reachable.shortest_length((t as i64, 0)).is_some(),
            expected,
            "m = 2^20, b = 638, t = {t}: reachable = {expected}"
        );
        assert_eq!(t % 2 == 0, expected, "the closed form: gcd(638, 2^20) = 2");
    }
    // The API reads the same structure.
    assert_eq!(quotient_within(M20, 638, 636086).is_some(), true);
    assert_eq!(quotient_within(M20, 638, 997), None);

    // m = 64, b = 12 (gcd 4).
    let small = WordGraph::from_register(M64 as i64, 12);
    assert_eq!(small.member_count(), 16, "the subgroup of step 12 in Z_64");
    for (t, expected) in [(8i64, true), (36, true), (7, false), (9, false)] {
        assert_eq!(small.shortest_length((t, 0)).is_some(), expected, "t = {t}");
    }

    // `divides` agrees with plain integer divisibility on the declared pairs.
    for (a, b) in [(3u64, 12u64), (5, 12), (4, 64), (7, 49), (6, 27), (12, 60)] {
        assert_eq!(divides(a, b), b % a == 0, "divides({a}, {b})");
    }
    assert!(!divides(3, 0), "no register for b = 0: the readout claims nothing");
    eprintln!("D2: reachability == gcd | t on all eight declared cases");
}

/// D3 (precommitted: pass) — THE QUOTIENT IS A PATH LENGTH. The declared
/// `(m, b, t) → k` values exactly, and the word itself is returned with the
/// target as its endpoint.
#[test]
fn gate_d3_quotient_is_a_path_length() {
    assert_eq!(quotient_within(M20, 997, 636086), Some(638), "the recorded step count");
    assert_eq!(quotient_within(M20, 997, 299100), Some(300));
    assert_eq!(quotient_within(M20, 997, 0), Some(0), "zero steps land on the origin");
    assert_eq!(quotient_within(M64, 12, 36), Some(3));
    assert_eq!(quotient_within(M64, 12, 8), Some(6));

    // The word: one walk of exactly k steps, ending on the target, replaying.
    let graph = WordGraph::from_register(M20 as i64, 997);
    let words = graph
        .words_to((636086, 0), 638, WalkOrder::BreadthFirst)
        .expect("the register has structure");
    assert_eq!(words.words.len(), 1, "one least word at the shortest length");
    let word = &words.words[0];
    assert_eq!(word.relations.len(), 638, "the word's length IS the quotient");
    assert_eq!(*word.members.last().unwrap(), (636086, 0));
    let mut sum = 0i64;
    for (x, _) in &word.relations {
        sum = (sum + x).rem_euclid(M20 as i64);
    }
    assert_eq!(sum, 636086, "the word replays to the target");
    eprintln!("D3: quotient 638 = the word's relation count, endpoint (636086, 0)");
}

/// D4 (precommitted: pass) — THE MODULAR INVERSE. The same reading at
/// `target = 1`; no inverse exists where the gcd does not divide 1, and the
/// readout fails closed.
#[test]
fn gate_d4_modular_inverse() {
    let k = modular_inverse(M20, 997).expect("997 is coprime to 2^20");
    assert_eq!(k, 950765, "the precommitted inverse — 950765 * 997 mod 2^20 = 1");
    assert_eq!((k as u64 * 997) % M20, 1, "the inverse replays");
    assert_eq!(modular_inverse(M64, 12), None, "gcd(12, 64) = 4 does not divide 1");
    assert_eq!(modular_inverse(20, 7), Some(3), "3 * 7 = 21 = 1 (mod 20)");
    eprintln!("D4: 997^-1 mod 2^20 = {k} (measured, not computed by a routine)");
}

/// D5 (precommitted: pass; headline, the Rule 2 arm) — THE MIRROR. The
/// generator `-b` reaches the same subgroup; the least `k` follows
/// `k' = order - k`. The same membership, a different word.
#[test]
fn gate_d5_mirror_same_subgroup_different_word() {
    for (m, b, t) in [(M20 as i64, 997i64, 636086i64), (M64 as i64, 12, 36)] {
        let forward = WordGraph::from_register(m, b);
        let mirror = WordGraph::from_register(m, -b);
        let order = forward.member_count();
        assert_eq!(
            order,
            mirror.member_count(),
            "m = {m}, b = {b}: the two generators reach the same subgroup"
        );
        let k = forward.shortest_length((t, 0)).expect("reachable forward");
        let k_mirror = mirror.shortest_length((t, 0)).expect("reachable mirrored");
        assert_eq!(k_mirror, order - k, "k' = order - k (precommitted)");
        assert_ne!(k, k_mirror, "same membership, different word");
        eprintln!("D5: m = {m}, b = {b}: order {order}, k = {k} forward, {k_mirror} mirrored");
    }
}

/// D6 (precommitted: pass) — THE CROSS-CHECK. The march's own readout, fed
/// back through the word readout, returns the declared step count; the
/// folding boundary is named, not hidden.
#[test]
fn gate_d6_cross_check_with_the_march() {
    // Below the fold: the register's readout IS the residue.
    let unfolded = generated_product(M20, 997, 300);
    assert_eq!(unfolded, 299100, "the march's folded readout (unfolded case)");
    assert_eq!(
        quotient_within(M20, 997, unfolded as u64),
        Some(300),
        "the step count recovered from the engine's OWN readout"
    );

    // Above the fold: the residue carries the count; the register reports the
    // folded representative (BUILD-11's recorded Nyquist boundary).
    let folded = generated_product(M20, 997, 638);
    assert_eq!(folded, 636086 - M20 as i64, "the folded representative: -412490");
    assert_eq!(quotient_within(M20, 997, 636086), Some(638));
    eprintln!("D6: 997 x 300 -> readout {unfolded} -> 300 steps; 997 x 638 -> folded {folded}, residue 636086 -> 638 steps");
}

/// D7 (precommitted: pass) — REGISTER SCALE. The recorded register size
/// builds and reads out exactly, with the timings reported.
#[test]
fn gate_d7_register_scale() {
    let start = std::time::Instant::now();
    let graph = WordGraph::from_register(M20 as i64, 997);
    let build = start.elapsed();
    assert_eq!(graph.member_count(), M20 as usize, "gcd(997, 2^20) = 1: the whole register");
    assert_eq!(graph.edge_count(), M20 as usize, "one edge per member (one generator)");

    let start = std::time::Instant::now();
    let k = graph.shortest_length((636086, 0));
    let read = start.elapsed();
    assert_eq!(k, Some(638));

    let start = std::time::Instant::now();
    let inverse = graph.shortest_length((1, 0));
    let deep = start.elapsed();
    assert_eq!(inverse, Some(950765), "the deep readout traverses most of the register");
    eprintln!(
        "D7: 2^20 members + {M20} edges built in {build:?}; reads {read:?} (638 steps) and {deep:?} (950765 steps)"
    );
}
