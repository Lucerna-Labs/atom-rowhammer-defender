//! BUILD-11 gates — precommitted in docs/BUILD11-KNOWLEDGE.md BEFORE
//! this code. Blocks stored; answers generated; sufficiency measured.

use atom_cognitive_engine::knowledge::{
    derivable, derivable_count, generated_product, reconstruct,
};

/// K1 (precommitted: pass) — EXACT GENERATED PRODUCT: 997x638 = 636086
/// from two folded readouts; the blocks are stored, the product is not.
#[test]
fn gate_k1_exact_generated_product() {
    let (n1, n2) = (1u64 << 20, (1u64 << 20) - 1);
    let m1 = generated_product(n1, 997, 638);
    let m2 = generated_product(n2, 997, 638);
    let product = reconstruct(n1, m1, n2, m2);
    eprintln!("stored blocks: 997, 638 | folded readouts ({m1}, {m2}) -> generated {product}");
    assert_eq!(m1, -412490, "folded readout register 1 (verified in Python)");
    assert_eq!(m2, -412489, "folded readout register 2");
    assert_eq!(product, 997 * 638, "636086 generated exactly, never stored");
    // Reversed order: 638 steps of 997 — the same product.
    let r1 = generated_product(n1, 638, 997);
    let r2 = generated_product(n2, 638, 997);
    assert_eq!(reconstruct(n1, r1, n2, r2), 997 * 638, "order-independence");
}

/// K2 (precommitted: pass) — A FAMILY FROM ONE BLOCK: from the single
/// stored block 997, five never-stored products, exact.
#[test]
fn gate_k2_family_from_blocks() {
    let (n1, n2) = (1u64 << 20, (1u64 << 20) - 1);
    let mut derived = Vec::new();
    for k in [2u64, 3, 5, 7, 11] {
        let m1 = generated_product(n1, 997, k);
        let m2 = generated_product(n2, 997, k);
        let product = reconstruct(n1, m1, n2, m2);
        eprintln!("997 x {k} = {product}");
        assert_eq!(product, 997 * k as u128, "exact from the same single block");
        derived.push(product);
    }
    eprintln!("stored entries: 1 block | derived answers: {} | knowledge economy measured", derived.len());
}

/// K3 (precommitted: pass) — SUFFICIENCY VERDICTS EXACT.
#[test]
fn gate_k3_sufficiency_verdicts() {
    let blocks = [(0i64, 8i64), (10, 0)];
    assert!(derivable(&blocks, (0, 16)), "2 x (0,8) is derivable: YES");
    assert!(derivable(&blocks, (10, 8)), "(10,0)+(0,8) is derivable: YES");
    assert!(!derivable(&blocks, (1, 0)), "not in the closure: NO (measured insufficiency)");
    eprintln!("verdicts: (0,16) YES, (10,8) YES, (1,0) NO — membership exact");
}

/// K4 (precommitted: pass) — ACQUIRE -> DERIVE: the verdict flips
/// NO->YES exactly when the source provides the missing block.
#[test]
fn gate_k4_acquire_and_derive() {
    let mut blocks = vec![(0i64, 8i64), (10, 0)];
    let target = (1i64, 0i64);
    let before = derivable(&blocks, target);
    // The source world (BUILD-5's loop) returns the missing block as
    // content — nothing else changes.
    blocks.push((1, 0));
    let after = derivable(&blocks, target);
    eprintln!("verdict before acquisition: {before} | after: {after}");
    assert!(!before, "insufficiency detected exactly");
    assert!(after, "derivable after the source provided the block");
}

/// K5 (precommitted: measured) — THE KNOWLEDGE ECONOMY: stored entries
/// vs derivable answers, printed.
#[test]
fn gate_k5_knowledge_economy() {
    let stored_relations = 2usize;
    let derivable_answers = derivable_count(&[(0i64, 8i64), (10, 0)]);
    eprintln!("knowledge economy: {stored_relations} stored relations -> {derivable_answers} derivable answers");
    assert!(derivable_answers > stored_relations * 3, "answers exceed blocks");
}
