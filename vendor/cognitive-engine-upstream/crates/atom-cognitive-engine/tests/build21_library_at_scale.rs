//! BUILD-21 gates — precommitted in `docs/BUILD21-LIBRARY-AT-SCALE.md` BEFORE
//! the content. The library at scale: the economy measured, the reach-out
//! exercised, hygiene and the class boundary intact.

use atom_block_library::{
    relations, relations_for_derivation, values, values_for_derivation, Block, Partition, Use,
    BLOCKS, WORLD_FACTS,
};
use atom_cognitive_engine::knowledge::{derivable, derivable_count, generated_product, reconstruct};
use std::collections::BTreeSet;

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

/// The 16 derived products (stored: 8 values; the products are never stored).
const EXPECTED: [(u64, u64, u128); 16] = [
    (997, 2, 1994),
    (997, 3, 2991),
    (997, 5, 4985),
    (997, 7, 6979),
    (638, 2, 1276),
    (638, 3, 1914),
    (638, 5, 3190),
    (638, 7, 4466),
    (12, 2, 24),
    (12, 3, 36),
    (12, 5, 60),
    (12, 7, 84),
    (45, 2, 90),
    (45, 3, 135),
    (45, 5, 225),
    (45, 7, 315),
];

const CONTEXT_REGS: &[u64] = &[M1, M2];
const CONTEXT_BLOCKS: &[Block] = &[Block::Value(42)];
const CONTEXT_PART: Partition = Partition {
    domain: "personal.notes",
    provenance: "gate fixture (2026-09-26)",
    class: Use::Context,
    registers: CONTEXT_REGS,
    blocks: CONTEXT_BLOCKS,
};

/// K1 (precommitted: pass) — SCALE WITH PROVENANCE; the seeded fixtures are
/// unchanged (growth by addition, never by editing recorded content).
#[test]
fn gate_k1_scale_with_provenance() {
    let bases = values_for_derivation(BLOCKS, "arithmetic.base").expect("bases");
    let steps = values_for_derivation(BLOCKS, "arithmetic.steps").expect("steps");
    let ext = relations_for_derivation(BLOCKS, "geometry.extended").expect("extended");
    let mass = values_for_derivation(WORLD_FACTS, "units.mass").expect("units.mass");
    assert_eq!(bases, vec![997, 638, 12, 45]);
    assert_eq!(steps, vec![2, 3, 5, 7]);
    assert_eq!(ext, vec![(0, 8), (10, 0), (10, 12), (6, 0)]);
    assert_eq!(mass, vec![454]);
    let added = bases.len() + steps.len() + ext.len() + mass.len();
    assert_eq!(added, 13, "thirteen new stored facts");

    // The recorded fixtures stay byte-identical.
    assert_eq!(values(BLOCKS, "arithmetic"), vec![997, 638]);
    assert_eq!(relations(BLOCKS, "geometry"), vec![(0, 8), (10, 0), (10, 12)]);
    assert_eq!(values_for_derivation(WORLD_FACTS, "units").expect("units"), vec![1609]);

    for domain in ["arithmetic.base", "arithmetic.steps", "geometry.extended"] {
        let p = BLOCKS.iter().find(|p| p.domain == domain).expect("partition");
        assert_eq!(p.class, Use::Derivation);
        assert!(!p.provenance.is_empty(), "{domain} carries provenance");
    }
    eprintln!("K1: 13 new stored facts across 4 partitions; seeded fixtures intact");
}

/// K2 (precommitted: pass) — KNOWLEDGE ECONOMY AT SIZE: 8 stored values →
/// 16 derived products, each equal to plain arithmetic through the engine's
/// own CRT path; 4 stored relations → closure 50.
#[test]
fn gate_k2_economy_at_size() {
    let stored = 8usize;
    let mut derived = 0usize;
    for (base, steps, expected) in EXPECTED {
        let got = product(&[base, steps]);
        assert_eq!(got, expected, "{base} x {steps}");
        derived += 1;
    }
    let ext = relations_for_derivation(BLOCKS, "geometry.extended").expect("extended");
    let closure = derivable_count(&ext);
    eprintln!(
        "K2: {stored} stored values -> {derived} derived products; {} stored relations -> closure {closure}",
        ext.len()
    );
    assert_eq!((stored, derived), (8, 16), "the economy at size");
    assert_eq!((ext.len(), closure), (4, 50));
}

/// K3 (precommitted: pass) — REACH-OUT: outside the stored closure is
/// measured; explicit acquisition supplies the missing relation; the verdict
/// flips exactly.
#[test]
fn gate_k3_reach_out() {
    let ext = relations_for_derivation(BLOCKS, "geometry.extended").expect("extended");
    assert!(!derivable(&ext, (1, 0)), "target is outside the stored closure");
    assert_eq!(derivable_count(&ext), 50);
    let mut acquired = ext.clone();
    acquired.push((1, 0)); // the source's content — the outside world
    assert!(derivable(&acquired, (1, 0)), "derivable after explicit acquisition");
    assert_eq!(derivable_count(&acquired), 100, "the closure doubles exactly");
    eprintln!("K3: (1,0) false -> acquired -> true; closure 50 -> 100");
}

/// K4 (precommitted: pass) — HYGIENE: no duplicate blocks in any partition;
/// every partition provenanced.
#[test]
fn gate_k4_hygiene() {
    for p in BLOCKS.iter().chain(WORLD_FACTS.iter()) {
        let mut seen_values: BTreeSet<u64> = BTreeSet::new();
        let mut seen_relations: BTreeSet<(i64, i64)> = BTreeSet::new();
        for block in p.blocks {
            match block {
                Block::Value(v) => assert!(
                    seen_values.insert(*v),
                    "{}: duplicate value {v}",
                    p.domain
                ),
                Block::Relation(x, y) => assert!(
                    seen_relations.insert((*x, *y)),
                    "{}: duplicate relation ({x},{y})",
                    p.domain
                ),
            }
        }
        assert!(!p.provenance.is_empty(), "{} is provenanced", p.domain);
    }
    eprintln!("K4: no duplicate blocks; every partition carries provenance");
}

/// K5 (precommitted: pass) — the CLASS boundary holds with the library grown.
#[test]
fn gate_k5_class_boundary_at_scale() {
    let table: &[Partition] = &[
        BLOCKS[0],
        BLOCKS[1],
        BLOCKS[2],
        BLOCKS[3],
        BLOCKS[4],
        WORLD_FACTS[0],
        WORLD_FACTS[1],
        CONTEXT_PART,
    ];
    assert!(
        values_for_derivation(table, "personal.notes").is_err(),
        "Context is still refused with the library grown"
    );
    assert!(values_for_derivation(table, "arithmetic.base").is_ok());
    assert!(relations_for_derivation(table, "geometry.extended").is_ok());
    eprintln!("K5: class boundary holds at scale; zero-dependency crate unchanged");
}
