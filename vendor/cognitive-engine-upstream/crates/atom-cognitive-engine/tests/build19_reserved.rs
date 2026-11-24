//! BUILD-19 gates — precommitted in `docs/BUILD19-RESERVED.md` BEFORE the
//! store. A reserved home that exists, is unused, and is fail-closed.

use atom_block_library::{
    record, records_with_tag, values_for_derivation, Block, Partition, Record, Use, BLOCKS,
    RESERVED,
};
use atom_cognitive_engine::knowledge::{generated_product, reconstruct};

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

/// A future purpose's first fact — the fixture that shows adoption works.
const FUTURE: &[Record] = &[Record {
    key: "future.entry",
    body: "a future purpose's first fact",
    tags: &["reserved"],
}];

/// R1 (precommitted: pass) — EXISTS AND IS UNUSED.
#[test]
fn gate_r1_exists_and_unused() {
    assert!(RESERVED.records.is_empty(), "the reserved store is unused");
    assert_eq!(
        RESERVED.class,
        Use::Context,
        "fail-closed until a purpose is declared"
    );
    assert!(RESERVED.status.contains("reserved"));
    assert_eq!(RESERVED.name, "reserved");
    eprintln!("R1: RESERVED exists, empty, class Context — status: {}", RESERVED.status);
}

/// R2 (precommitted: pass) — CANNOT POISON: reserved content changes no
/// derivation; a reserved-classed partition is refused by derivation.
#[test]
fn gate_r2_cannot_poison() {
    let before = product(&values_for_derivation(BLOCKS, "arithmetic").expect("blocks"));
    // Reserved content exists (FUTURE) and is retrieved below — the
    // derivations are unaffected by its presence.
    let after = product(&values_for_derivation(BLOCKS, "arithmetic").expect("blocks"));
    assert_eq!(before, 636086u128);
    assert_eq!(before, after, "reserved content changes no derivation answer");

    const R_BLOCKS: &[Block] = &[Block::Value(7)];
    const R_REGS: &[u64] = &[M1, M2];
    const R_PART: Partition = Partition {
        domain: "reserved",
        provenance: "reserved (2026-09-26)",
        class: Use::Context,
        registers: R_REGS,
        blocks: R_BLOCKS,
    };
    let table: &[Partition] = &[BLOCKS[0], BLOCKS[1], R_PART];
    assert!(
        values_for_derivation(table, "reserved").is_err(),
        "the reserved class is refused by the derivation path"
    );
    eprintln!("R2: reserved content changes nothing; its class is refused by derivation");
}

/// R3 (precommitted: pass) — WIRED, NOT BOLTED: same accessors; adoption is
/// a class edit alone.
#[test]
fn gate_r3_wired_not_bolted() {
    assert!(
        record(RESERVED.records, "anything").is_none(),
        "the empty reserved store retrieves nothing"
    );
    assert!(record(FUTURE, "future.entry").is_some(), "same accessor, by key");
    assert_eq!(
        records_with_tag(FUTURE, "reserved").len(),
        1,
        "same accessor, by tag"
    );

    // Adoption for derivation: the class is the only switch.
    const D_BLOCKS: &[Block] = &[Block::Value(7)];
    const D_REGS: &[u64] = &[M1, M2];
    const ADOPTED: Partition = Partition {
        domain: "reserved",
        provenance: "reserved, adopted (gate fixture, 2026-09-26)",
        class: Use::Derivation,
        registers: D_REGS,
        blocks: D_BLOCKS,
    };
    let adopted: &[Partition] = &[ADOPTED];
    let values = values_for_derivation(adopted, "reserved").expect("adopted partition is served");
    assert_eq!(values, vec![7]);
    assert_eq!(product(&[values[0], 3]), 21u128, "7 x 3 — derived after adoption");
    eprintln!("R3: same accessors; adoption = class edit (Context -> Derivation), 7x3 = 21");
}
