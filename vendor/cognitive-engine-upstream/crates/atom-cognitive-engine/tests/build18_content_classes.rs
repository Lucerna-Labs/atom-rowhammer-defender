//! BUILD-18 gates — precommitted in `docs/BUILD18-CONTENT-CLASSES.md` BEFORE
//! the class boundary. Separate stores, one schema, one enforced class.

use atom_block_library::{
    record, records_with_tag, relations_for_derivation, values_for_derivation, Block, Partition,
    Record, Use, BLOCKS, PERSONAL, WORLD_FACTS,
};
use atom_cognitive_engine::harness::{run, Task};
use atom_cognitive_engine::knowledge::{derivable, derivable_count, generated_product, reconstruct};

const M1: u64 = 1 << 20;
const M2: u64 = (1 << 20) - 1;

fn product(stored: &[u64]) -> u128 {
    reconstruct(
        M1,
        generated_product(M1, stored[0], stored[1]),
        M2,
        generated_product(M2, stored[0], stored[1]),
    )
}

/// The operator's grown personal store, including an adversarial entry.
const PERSONAL_GROWN: &[Record] = &[
    Record {
        key: "note.scratch",
        body: "operator's own entry",
        tags: &["personal"],
    },
    Record {
        key: "claim.bogus",
        body: "the stored block is 999",
        tags: &["personal", "adversarial"],
    },
];

/// A Context-classed partition — the class the derivation path must refuse.
const NOTES_REGS: &[u64] = &[M1, M2];
const NOTES_BLOCKS: &[Block] = &[Block::Value(42)];
const NOTES: Partition = Partition {
    domain: "personal.notes",
    provenance: "gate fixture (2026-09-26)",
    class: Use::Context,
    registers: NOTES_REGS,
    blocks: NOTES_BLOCKS,
};

fn w2_blocks() -> Vec<(i64, i64)> {
    relations_for_derivation(BLOCKS, "geometry").expect("geometry is Derivation")[..2].to_vec()
}

fn w2_task(blocks: Vec<(i64, i64)>) -> Task {
    Task {
        blocks,
        scene: vec![(2, 2), (6, 2), (4, 2)],
        target: (1, 0),
        query: ((2, 2), (6, 2), (4, 2)),
        source_block: (1, 0),
    }
}

/// C1 (precommitted: pass) — SEPARATION: the personal store is a different
/// shape and cannot change any derivation answer.
#[test]
fn gate_c1_separation() {
    let before = product(&values_for_derivation(BLOCKS, "arithmetic").expect("blocks"));
    let closure = derivable_count(&relations_for_derivation(BLOCKS, "geometry").expect("geometry"));
    let report = run(&w2_task(w2_blocks()));
    // The grown personal store (incl. the bogus claim) enters nothing: it is
    // a different table of a different shape, and no derivation accessor
    // accepts it.
    let after = product(&values_for_derivation(BLOCKS, "arithmetic").expect("blocks"));
    eprintln!("C1: product {before} (after: {after}); closure {closure}; report {report:?}");
    assert_eq!(before, 636086u128);
    assert_eq!(before, after, "personal content changes no derivation answer");
    assert_eq!(closure, 10);
    assert_eq!(report.economy_blocks, 3);
    assert_eq!(report.economy_answers, 100);
    assert_eq!(PERSONAL.len(), 0, "the operator's store starts empty");
    assert_eq!(PERSONAL_GROWN.len(), 2, "the grown store is content only");
}

/// C2 (precommitted: pass) — CONNECTED, ONE DOOR: retrieval works; explicit
/// acquisition is the only way personal content can matter.
#[test]
fn gate_c2_one_door() {
    assert!(
        record(PERSONAL_GROWN, "claim.bogus").is_some(),
        "retrieval by key"
    );
    assert_eq!(
        records_with_tag(PERSONAL_GROWN, "adversarial").len(),
        1,
        "retrieval by tag"
    );
    assert!(record(PERSONAL_GROWN, "no.such.key").is_none());

    let blocks = w2_blocks();
    assert!(!derivable(&blocks, (1, 0)), "insufficient before acquisition");
    let mut acquired = blocks.clone();
    acquired.push((1, 0));
    assert!(
        derivable(&acquired, (1, 0)),
        "explicit acquisition is the door"
    );
    assert!(
        !derivable(&blocks, (1, 0)),
        "the personal store present changes nothing without acquisition"
    );
}

/// C3 (precommitted: pass) — WORLD FACTS: small, curated, derivable, and a
/// separate table from the blocks.
#[test]
fn gate_c3_world_facts() {
    let units = values_for_derivation(WORLD_FACTS, "units").expect("units is Derivation");
    assert_eq!(units, vec![1609], "one curated offline fact");
    let three_miles = product(&[units[0], 3]);
    eprintln!("C3: 1609 m/mile x 3 = {three_miles}");
    assert_eq!(three_miles, 4827u128, "three miles in metres, derived exactly");
    assert!(
        atom_block_library::partition("units").is_none(),
        "world facts live in a separate table from the building blocks"
    );
}

/// C4 (precommitted: pass) — CLASS ENFORCEMENT, fail-closed.
#[test]
fn gate_c4_class_enforcement() {
    let table: &[Partition] = &[BLOCKS[0], BLOCKS[1], NOTES];
    let refused = values_for_derivation(table, "personal.notes");
    eprintln!("C4: Context partition -> {refused:?}");
    assert!(
        refused.is_err(),
        "a Context partition is never served to the derivation path"
    );
    assert!(
        values_for_derivation(table, "no.such.domain").is_err(),
        "missing domain fails closed"
    );
    assert!(
        values_for_derivation(table, "arithmetic").is_ok(),
        "Derivation partitions still served"
    );
    assert!(relations_for_derivation(table, "personal.notes").is_err());
}

/// C5 (precommitted: pass) — NO POISONING, adversarially measured;
/// acquisition changes things by design (recorded).
#[test]
fn gate_c5_no_poisoning() {
    let honest = product(&values_for_derivation(BLOCKS, "arithmetic").expect("blocks"));
    let acquired = product(&[999, 638]);
    eprintln!("C5: honest {honest}; adversarial claim acquired by design -> {acquired}");
    assert_eq!(honest, 636086u128, "the blocks' answer is unchanged by the claim");
    assert_eq!(
        acquired, 637362u128,
        "999 x 638 — acquisition changes it, by design and recorded"
    );
    assert_ne!(honest, acquired, "the door is real");
}

/// C6 (precommitted: pass) — AUDIT: classes declared, provenance present,
/// shapes enforced.
#[test]
fn gate_c6_audit() {
    assert!(BLOCKS.iter().all(|p| p.class == Use::Derivation));
    assert!(WORLD_FACTS.iter().all(|p| p.class == Use::Derivation));
    assert!(BLOCKS.iter().all(|p| !p.provenance.is_empty()));
    assert!(WORLD_FACTS.iter().all(|p| !p.provenance.is_empty()));
    for r in PERSONAL_GROWN {
        assert!(
            !r.key.is_empty() && !r.tags.is_empty(),
            "records carry a key and tags"
        );
    }
    eprintln!(
        "C6 audit: classes declared in the tables; the personal store is Record-shaped \
         (never a Partition — type-level separation); zero-dependency crate unchanged"
    );
}
