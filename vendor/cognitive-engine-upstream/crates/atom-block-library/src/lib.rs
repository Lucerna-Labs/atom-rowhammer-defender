//! The block library — content slices for the harness (BUILD-17), with
//! declared content classes (BUILD-18).
//! Precommitted in `docs/BUILD17-BLOCK-LIBRARY.md` and
//! `docs/BUILD18-CONTENT-CLASSES.md` BEFORE this code.
//!
//! CONTENT ONLY. This crate has ZERO dependencies and cannot compute: it
//! stores facts (values, relations, records) with provenance, in separate
//! tables with declared classes, and serves them through data accessors.
//! The engines' intrinsic generators are the only things that compute;
//! sufficiency stays the harness's own measurement.
//!
//! CLASSES (BUILD-18): every partition declares its `Use`. The derivation
//! accessors are class-gated and fail closed — a `Context` partition can
//! never be fed to a derivation. The personal store is a different content
//! shape (`Record`), reachable only through its own retrieval accessors,
//! and it enters derivation state only by explicit acquisition.
//!
//! Rule 3 boundary: content is EVIDENCE with provenance, never an oracle.
//! No procedure, recipe, operation table, or target mapping may appear here.
//!
//! AUDIT: no dependencies; no arithmetic or control flow beyond lookup and
//! iteration; provenance strings are source labels only.

/// How a partition's content may be used (BUILD-18).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Use {
    /// Curated content the derivation path may consume (blocks, world facts).
    Derivation,
    /// Context content: retrievable, never implicitly derivable.
    Context,
}

/// One stored fact. Content kinds mirror what the engines consume: register
/// values (arithmetic) and lattice relations (geometry/knowledge).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Block {
    /// Arithmetic content: a magnitude, read at the partition's registers.
    Value(u64),
    /// Relational content: a lattice difference vector.
    Relation(i64, i64),
}

/// A personal-store entry (BUILD-18): the operator's own data — a key, a
/// body, and tags. Retrievable, and able to enter a derivation only by
/// explicit acquisition.
#[derive(Clone, Copy, Debug)]
pub struct Record {
    pub key: &'static str,
    pub body: &'static str,
    pub tags: &'static [&'static str],
}

/// A domain slice: content plus its provenance, class, and declared
/// registers.
#[derive(Clone, Copy, Debug)]
pub struct Partition {
    pub domain: &'static str,
    pub provenance: &'static str,
    /// The declared usage class (BUILD-18). The derivation accessors refuse
    /// anything that is not `Derivation`.
    pub class: Use,
    /// The registers at which this partition's values are read; empty when
    /// the partition carries no value content (structural declaration).
    pub registers: &'static [u64],
    pub blocks: &'static [Block],
}

/// The building-blocks table (the seeded domain slices). Adding a domain is
/// adding a table entry — data, not code.
pub const SEEDED: &[Partition] = &[
    Partition {
        domain: "arithmetic",
        provenance: "BUILD-11 demo inputs (2026-09-25)",
        class: Use::Derivation,
        registers: &[1 << 20, (1 << 20) - 1],
        blocks: &[Block::Value(997), Block::Value(638)],
    },
    Partition {
        domain: "geometry",
        provenance: "BUILD-2 scene-ABC differences as recorded (2026-09-25)",
        class: Use::Derivation,
        registers: &[],
        blocks: &[
            Block::Relation(0, 8),
            Block::Relation(10, 0),
            Block::Relation(10, 12),
        ],
    },
    // ---- BUILD-21: the library at scale (appended; the fixtures above are
    // recorded content and stay byte-identical) ----
    Partition {
        domain: "arithmetic.base",
        provenance: "curated bases (2026-09-26)",
        class: Use::Derivation,
        registers: &[1 << 20, (1 << 20) - 1],
        blocks: &[
            Block::Value(997),
            Block::Value(638),
            Block::Value(12),
            Block::Value(45),
        ],
    },
    Partition {
        domain: "arithmetic.steps",
        provenance: "curated multipliers (2026-09-26)",
        class: Use::Derivation,
        registers: &[1 << 20, (1 << 20) - 1],
        blocks: &[
            Block::Value(2),
            Block::Value(3),
            Block::Value(5),
            Block::Value(7),
        ],
    },
    Partition {
        domain: "geometry.extended",
        provenance: "seeded relations + (6,0) (2026-09-26): closure 50",
        class: Use::Derivation,
        registers: &[],
        blocks: &[
            Block::Relation(0, 8),
            Block::Relation(10, 0),
            Block::Relation(10, 12),
            Block::Relation(6, 0),
        ],
    },
];

/// The building-blocks table, named for its class.
pub const BLOCKS: &[Partition] = SEEDED;

/// World facts (BUILD-18): small, curated, offline-convenience content.
/// BUILD-21 appends `units.mass` (the fixture `units` stays byte-identical).
pub const WORLD_FACTS: &[Partition] = &[
    Partition {
        domain: "units",
        provenance: "miles->metres, curated offline fact (2026-09-26)",
        class: Use::Derivation,
        registers: &[1 << 20, (1 << 20) - 1],
        blocks: &[Block::Value(1609)],
    },
    Partition {
        domain: "units.mass",
        provenance: "pounds->grams, curated offline fact (2026-09-26)",
        class: Use::Derivation,
        registers: &[1 << 20, (1 << 20) - 1],
        blocks: &[Block::Value(454)],
    },
];

/// The operator's personal store (BUILD-18). Starts empty; entries are the
/// operator's own data and never touch the derivation path implicitly.
pub const PERSONAL: &[Record] = &[];

/// A reserved store: declared, unused, and fail-closed until a purpose is
/// declared for it (BUILD-19). Adoption is a data/class change only.
#[derive(Clone, Copy, Debug)]
pub struct ReservedStore {
    pub name: &'static str,
    /// The declared status — why it exists and how it may be adopted.
    pub status: &'static str,
    /// Fail-closed default: an undefined purpose may not feed derivations.
    pub class: Use,
    pub records: &'static [Record],
}

/// The reserved store (BUILD-19): exists, is unused, and is wired through
/// the same accessors as everything else.
pub const RESERVED: ReservedStore = ReservedStore {
    name: "reserved",
    status: "reserved — purpose undefined by decision (2026-09-26); adoption is a data/class change only",
    class: Use::Context,
    records: &[],
};

/// Look up a seeded partition by domain name.
pub fn partition(domain: &str) -> Option<&'static Partition> {
    SEEDED.iter().find(|p| p.domain == domain)
}

/// The value content of a partition in a table, in stored order (raw read).
pub fn values(partitions: &[Partition], domain: &str) -> Vec<u64> {
    partitions
        .iter()
        .find(|p| p.domain == domain)
        .map(|p| {
            p.blocks
                .iter()
                .filter_map(|b| match b {
                    Block::Value(v) => Some(*v),
                    Block::Relation(_, _) => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The relational content of a partition in a table, in stored order (raw
/// read).
pub fn relations(partitions: &[Partition], domain: &str) -> Vec<(i64, i64)> {
    partitions
        .iter()
        .find(|p| p.domain == domain)
        .map(|p| {
            p.blocks
                .iter()
                .filter_map(|b| match b {
                    Block::Relation(x, y) => Some((*x, *y)),
                    Block::Value(_) => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The derivation path's value accessor: CLASS-GATED, fail-closed. A
/// partition whose class is not `Derivation` is refused, never coerced.
pub fn values_for_derivation(
    partitions: &[Partition],
    domain: &str,
) -> Result<Vec<u64>, &'static str> {
    match partitions.iter().find(|p| p.domain == domain) {
        None => Err("no such partition"),
        Some(p) if p.class != Use::Derivation => {
            Err("partition class is not Derivation (fail-closed)")
        }
        Some(p) => Ok(p
            .blocks
            .iter()
            .filter_map(|b| match b {
                Block::Value(v) => Some(*v),
                Block::Relation(_, _) => None,
            })
            .collect()),
    }
}

/// The derivation path's relational accessor: CLASS-GATED, fail-closed.
pub fn relations_for_derivation(
    partitions: &[Partition],
    domain: &str,
) -> Result<Vec<(i64, i64)>, &'static str> {
    match partitions.iter().find(|p| p.domain == domain) {
        None => Err("no such partition"),
        Some(p) if p.class != Use::Derivation => {
            Err("partition class is not Derivation (fail-closed)")
        }
        Some(p) => Ok(p
            .blocks
            .iter()
            .filter_map(|b| match b {
                Block::Relation(x, y) => Some((*x, *y)),
                Block::Value(_) => None,
            })
            .collect()),
    }
}

/// Retrieve a personal record by exact key.
pub fn record<'a>(records: &'a [Record], key: &str) -> Option<&'a Record> {
    records.iter().find(|r| r.key == key)
}

/// Retrieve personal records carrying a tag.
pub fn records_with_tag<'a>(records: &'a [Record], tag: &str) -> Vec<&'a Record> {
    records.iter().filter(|r| r.tags.contains(&tag)).collect()
}
