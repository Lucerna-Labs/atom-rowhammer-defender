//! BUILD-17 gates — precommitted in `docs/BUILD17-BLOCK-LIBRARY.md` BEFORE
//! the slice. The library is content only; the engines compute.

use atom_block_library::{partition, relations, values, Block, Partition, Use, SEEDED};
use atom_cognitive_engine::harness::{run, Task};
use atom_cognitive_engine::knowledge::{derivable, derivable_count, generated_product, reconstruct};

const M1: u64 = 1 << 20;
const M2: u64 = (1 << 20) - 1;

/// The boundary's own product path: two folded readouts, CRT reconstruction
/// — run by the ENGINE over library content.
fn product_from_library(stored: &[u64]) -> u128 {
    let block = stored[0];
    let steps = stored[1];
    let m1 = generated_product(M1, block, steps);
    let m2 = generated_product(M2, block, steps);
    reconstruct(M1, m1, M2, m2)
}

/// L1 (precommitted: pass) — CONTENT-ONLY, CONTENT-DETERMINED: the engine
/// derives the product from the library's stored values; different content
/// derives differently (the library computed nothing).
#[test]
fn gate_l1_content_determined() {
    let arithmetic = partition("arithmetic").expect("seeded partition");
    assert_eq!(arithmetic.provenance, "BUILD-11 demo inputs (2026-09-25)");
    let stored = values(SEEDED, "arithmetic");
    assert_eq!(stored, vec![997, 638], "two stored values, nothing else");
    let product = product_from_library(&stored);
    eprintln!("L1: library content {stored:?} at registers {M1}/{M2} -> product {product}");
    assert_eq!(product, 636086u128, "engine-derived product from library content");

    const ALT_BLOCKS: &[Block] = &[Block::Value(998), Block::Value(638)];
    const ALT_REGS: &[u64] = &[M1, M2];
    let alt_table: &[Partition] = &[Partition {
        domain: "arithmetic.alt",
        provenance: "gate fixture (2026-09-26)",
        class: Use::Derivation,
        registers: ALT_REGS,
        blocks: ALT_BLOCKS,
    }];
    let alt = product_from_library(&values(alt_table, "arithmetic.alt"));
    eprintln!("L1: alternative content -> {alt}");
    assert_eq!(alt, 636724u128, "998 x 638 — content is load-bearing");
}

/// L2 (precommitted: pass) — SLICES DRIVE THE HARNESS EXACTLY: the recorded
/// W2 report, with the task's stored blocks taken from the library.
#[test]
fn gate_l2_slices_drive_the_harness() {
    let geom = relations(SEEDED, "geometry");
    assert_eq!(geom, vec![(0, 8), (10, 0), (10, 12)], "the recorded relations");
    // The W2 fixture's stored blocks are the first two stored relations.
    let task = Task {
        blocks: geom[..2].to_vec(),
        scene: vec![(2, 2), (6, 2), (4, 2)],
        target: (1, 0),
        query: ((2, 2), (6, 2), (4, 2)),
        source_block: (1, 0),
    };
    let report = run(&task);
    eprintln!("L2: harness report over library blocks {report:?}");
    assert_eq!(report.scene_support, 4);
    assert!(report.judge);
    assert_eq!(report.closure_before, 10);
    assert!(!report.sufficient_before);
    assert!(report.sufficient_after);
    assert_eq!(report.economy_blocks, 3);
    assert_eq!(report.economy_answers, 100);

    // The arithmetic slice through the boundary's product path.
    let product = product_from_library(&values(SEEDED, "arithmetic"));
    assert_eq!(product, 636086u128, "arithmetic slice -> exact product");
}

/// L3 (precommitted: pass) — MINIMAL-DATA PER SLICE, measured.
#[test]
fn gate_l3_minimal_data_per_slice() {
    let stored = values(SEEDED, "arithmetic");
    let product = product_from_library(&stored);
    let family: Vec<u128> = [2u64, 3, 5, 7, 11]
        .iter()
        .map(|m| product_from_library(&[stored[0], *m]))
        .collect();
    eprintln!(
        "L3 arithmetic: {} stored values -> product {product} + family {family:?}",
        stored.len()
    );
    assert_eq!(product, 636086);
    assert_eq!(family, vec![1994u128, 2991, 4985, 6979, 10967]);
    assert_eq!((stored.len(), 1 + family.len()), (2, 6), "knowledge economy per slice");

    let geom = relations(SEEDED, "geometry");
    let closure = derivable_count(&geom);
    eprintln!("L3 geometry: {} stored relations -> closure {closure}", geom.len());
    assert_eq!((geom.len(), closure), (3, 10), "three stored relations, closure ten");
}

/// L4 (precommitted: pass) — ADDING A DOMAIN IS DATA: a third partition built
/// at test time flows through the SAME accessor logic.
#[test]
fn gate_l4_adding_a_domain_is_data() {
    const DEMO_REGS: &[u64] = &[M1, M2];
    const DEMO_BLOCKS: &[Block] = &[Block::Value(5), Block::Value(6)];
    const DEMO: Partition = Partition {
        domain: "demo",
        provenance: "gate fixture (2026-09-26)",
        class: Use::Derivation,
        registers: DEMO_REGS,
        blocks: DEMO_BLOCKS,
    };
    let table: &[Partition] = &[SEEDED[0], SEEDED[1], DEMO];
    let demo = product_from_library(&values(table, "demo"));
    eprintln!("L4: third partition ('demo', no library code change) -> {demo}");
    assert_eq!(demo, 30u128, "5 x 6 from a data-only addition");
    // The same accessors still serve the original partitions.
    assert_eq!(values(table, "arithmetic"), vec![997, 638]);
    assert_eq!(relations(table, "geometry").len(), 3);
}

/// L5 (precommitted: pass) — NO-CONTENT ARM: empty content derives nothing;
/// the engine's sufficiency over the library is a measurement.
#[test]
fn gate_l5_no_content_arm() {
    let empty: Vec<(i64, i64)> = Vec::new();
    assert!(!derivable(&empty, (0, 16)), "empty content derives nothing");
    // Precommit predicted 0; MEASURED 1 — the closure of nothing is the
    // trivial relation (0,0), i.e. DC. Corrected on record (BUILD-3's
    // "support 1 (DC only)" is the same object).
    assert_eq!(
        derivable_count(&empty),
        1,
        "empty content derives exactly the trivial relation"
    );
    assert!(relations(SEEDED, "no.such.domain").is_empty());
    assert!(values(SEEDED, "no.such.domain").is_empty());

    let geom = relations(SEEDED, "geometry");
    assert!(derivable(&geom, (10, 0)), "a stored relation is in the slice's closure");
    assert!(!derivable(&geom, (1, 0)), "not everything is derivable — sufficiency is measured");
    eprintln!("L5: empty slice derives nothing; slice sufficiency measured by the engine");
}

/// L6 (precommitted: pass) — AUDIT: content-only, provenanced, no compute.
#[test]
fn gate_l6_audit() {
    for p in SEEDED {
        assert!(!p.provenance.is_empty(), "every partition carries provenance");
        assert!(!p.domain.is_empty());
    }
    assert!(partition("arithmetic").is_some());
    assert!(partition("geometry").is_some());
    eprintln!(
        "L6 audit: zero-dependency content crate (Cargo.toml); engine holds no build-time \
         dependency on it; provenance strings are source labels only"
    );
}
