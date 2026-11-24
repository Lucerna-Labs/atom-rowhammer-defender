//! BUILD-29 — derivation word-paths: the SEQUENCE of stored relations that
//! produces a member. Precommitted in `docs/BUILD29-DERIVATION-PATHS.md`
//! BEFORE this code, under vault Rules 1/2/3/4 and project rules 0/0a/0b.
//!
//! The closure answers *whether*: one membership bit and one count. It
//! cannot answer **how**, and it cannot tell two routes apart. This module
//! builds the structure that can: one node per closure member, one directed
//! edge per stored relation (`m → m + g`, so an edge names its generator),
//! and a walk from the identity IS a word.
//!
//! Audit (structural constants only): the lattice modulus (a caller
//! parameter), the identity origin `(0,0)` (the closure machinery's own
//! starting point), the caller's hop bound, BUILD-28's canonical
//! enumeration and digest. No threshold, no score, no ranking, no verdict:
//! every readout takes `&self` and returns sequences.

use crate::causal_path::{digest, CausalGraph, Direction, Node, PathError, WalkOrder};
use std::collections::HashSet;

/// One derivation: the relations applied in order, and the members visited.
/// `members[0]` is the origin and `members.last()` the derived member;
/// `relations[i]` is the step from `members[i]` to `members[i + 1]`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Word {
    pub relations: Vec<(i64, i64)>,
    pub members: Vec<(i64, i64)>,
}

impl Word {
    /// The word's length in applied relations.
    pub fn length(&self) -> usize {
        self.relations.len()
    }
}

/// A bounded word enumeration for one target. `shortest` is exact and
/// independent of the bound — `None` means the target is not a member at
/// all (definitive), while `Some(k)` with `k > max_hops` means the bound
/// withheld the words rather than that no derivation exists.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Words {
    pub target: (i64, i64),
    pub max_hops: usize,
    pub shortest: Option<usize>,
    pub words: Vec<Word>,
    pub truncated: bool,
}

impl Words {
    /// The canonical digest: over the target, the bound, the shortest
    /// length and the words as sorted sequences — so the enumeration
    /// variant cannot change it and the bound can.
    pub fn digest(&self) -> u64 {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&self.target.0.to_le_bytes());
        bytes.extend_from_slice(&self.target.1.to_le_bytes());
        bytes.extend_from_slice(&(self.max_hops as u64).to_le_bytes());
        match self.shortest {
            None => bytes.extend_from_slice(&u64::MAX.to_le_bytes()),
            Some(k) => bytes.extend_from_slice(&(k as u64).to_le_bytes()),
        }
        let mut words = self.words.clone();
        words.sort_by(|a, b| {
            (a.relations.len(), &a.relations).cmp(&(b.relations.len(), &b.relations))
        });
        for word in &words {
            bytes.extend_from_slice(&(word.relations.len() as u64).to_le_bytes());
            for (x, y) in &word.relations {
                bytes.extend_from_slice(&x.to_le_bytes());
                bytes.extend_from_slice(&y.to_le_bytes());
            }
        }
        bytes.push(self.truncated as u8);
        digest(&bytes)
    }
}

/// The derivation graph of a stored relation set: every closure member, and
/// one edge per relation. Built from content; it computes no new law — the
/// members are the same set `derivable` describes.
pub struct WordGraph {
    graph: CausalGraph,
    modulus: i64,
    generators: Vec<(i64, i64)>,
    origin: (i64, i64),
    members: Vec<(i64, i64)>,
}

impl WordGraph {
    /// Build the graph over the closure of the given relations. Relations
    /// are normalized mod `modulus` and deduplicated (the same relation in
    /// two partitions is one piece of knowledge); the zero relation is
    /// dropped (it is the identity, not a step).
    pub fn from_relations(relations: &[(i64, i64)], modulus: i64) -> Self {
        let mut generators: Vec<(i64, i64)> = Vec::new();
        for (x, y) in relations {
            let g = (x.rem_euclid(modulus), y.rem_euclid(modulus));
            if g != (0, 0) && !generators.contains(&g) {
                generators.push(g);
            }
        }
        generators.sort();

        let origin = (0, 0);
        // Membership by set (BUILD-31: the register lattice has ~1M members
        // at the recorded register size; a linear scan would be quadratic).
        let mut seen: HashSet<(i64, i64)> = HashSet::new();
        seen.insert(origin);
        let mut members: Vec<(i64, i64)> = vec![origin];
        let mut graph = CausalGraph::new();
        let mut index = 0;
        while index < members.len() {
            let member = members[index];
            index += 1;
            for g in &generators {
                let to = (
                    (member.0 + g.0).rem_euclid(modulus),
                    (member.1 + g.1).rem_euclid(modulus),
                );
                // There is no run to date: this structure is a static
                // closure, so its edges carry no epoch. The applied
                // relation is named by the edge's endpoints.
                graph.record(Node::content(member), Node::content(to), 0);
                if seen.insert(to) {
                    members.push(to);
                }
            }
        }
        members.sort();
        WordGraph {
            graph,
            modulus,
            generators,
            origin,
            members,
        }
    }

    /// BUILD-31: the REGISTER LATTICE — positions `Z_modulus` with one step
    /// (the stored block), embedded in the graph's first coordinate. This is
    /// the structure the inverse readouts (`knowledge::gcd_of`,
    /// `divides`, `quotient_within`, `modular_inverse`) are readings of.
    pub fn from_register(modulus: i64, block: i64) -> Self {
        Self::from_relations(&[(block, 0)], modulus)
    }

    /// The distinct relations the words are made of (deduplicated,
    /// canonical).
    pub fn generators(&self) -> &[(i64, i64)] {
        &self.generators
    }

    /// The closure members, canonical. The same set `derivable` describes.
    pub fn members(&self) -> &[(i64, i64)] {
        &self.members
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// The recorded arrows (BUILD-31: one per member per generator — the
    /// scale an observer can check).
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn has_member(&self, cell: (i64, i64)) -> bool {
        let cell = (cell.0.rem_euclid(self.modulus), cell.1.rem_euclid(self.modulus));
        self.members.binary_search(&cell).is_ok()
    }

    /// The shortest word length to the target: exact, by breadth-first
    /// search over the graph's own edges. `None` when the target is not a
    /// member — a definitive answer, not a bound. The identity's shortest
    /// word is the **empty** word (no relation applied), so it reports 0
    /// even though the enumeration lists the non-trivial words that return
    /// to it.
    pub fn shortest_length(&self, target: (i64, i64)) -> Option<usize> {
        let target = (
            target.0.rem_euclid(self.modulus),
            target.1.rem_euclid(self.modulus),
        );
        if self.members.binary_search(&target).is_err() {
            return None;
        }
        if target == self.origin {
            return Some(0);
        }
        // Membership by set (BUILD-31: register scale).
        let mut visited: HashSet<(i64, i64)> = HashSet::new();
        visited.insert(self.origin);
        let mut frontier: Vec<(i64, i64)> = vec![self.origin];
        let mut hops = 0usize;
        while !frontier.is_empty() {
            hops += 1;
            let mut next: Vec<(i64, i64)> = Vec::new();
            for member in &frontier {
                for g in &self.generators {
                    let to = (
                        (member.0 + g.0).rem_euclid(self.modulus),
                        (member.1 + g.1).rem_euclid(self.modulus),
                    );
                    if to == target {
                        return Some(hops);
                    }
                    if visited.insert(to) {
                        next.push(to);
                    }
                }
            }
            frontier = next;
        }
        None
    }

    /// The words of 1..=`max_hops` relations that land on the target, in the
    /// asked enumeration order (BUILD-28's declared variants: breadth-first
    /// canonical, depth-first measured). Fails closed (the primitive's own
    /// typed refusal) when the graph carries no structure at all.
    pub fn words_to(
        &self,
        target: (i64, i64),
        max_hops: usize,
        order: WalkOrder,
    ) -> Result<Words, PathError> {
        let target = (
            target.0.rem_euclid(self.modulus),
            target.1.rem_euclid(self.modulus),
        );
        let traversal = self.graph.traverse(
            Node::content(self.origin),
            Direction::Future,
            max_hops,
            order,
        )?;
        let mut words: Vec<Word> = Vec::new();
        for walk in &traversal.walks {
            if walk.nodes[walk.nodes.len() - 1].cell != target {
                continue;
            }
            let members: Vec<(i64, i64)> = walk.nodes.iter().map(|node| node.cell).collect();
            let relations: Vec<(i64, i64)> = members
                .windows(2)
                .map(|pair| {
                    (
                        (pair[1].0 - pair[0].0).rem_euclid(self.modulus),
                        (pair[1].1 - pair[0].1).rem_euclid(self.modulus),
                    )
                })
                .collect();
            words.push(Word { relations, members });
        }
        Ok(Words {
            target,
            max_hops,
            shortest: self.shortest_length(target),
            words,
            truncated: traversal.truncated,
        })
    }
}
