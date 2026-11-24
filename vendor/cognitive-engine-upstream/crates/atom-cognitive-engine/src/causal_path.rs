//! BUILD-28 — the causal-path primitive: directed typed relations, a
//! sequence-returning readout, and hop-bounded traces over our own
//! material. Precommitted in `docs/BUILD28-CAUSAL-PATH.md` BEFORE this
//! code, under vault Rules 1/2/3/4 and project rules 0/0a/0b.
//!
//! STRUCTURE, NOT POLICY. The readout enumerates walks and returns them
//! in order; it never scores, ranks, chooses, or answers. The only
//! declared parameters are the caller's hop bound and the declared
//! enumeration order. Every readout takes `&self` — traversing a graph
//! cannot change it, and nothing here is consulted by a derivation.
//!
//! Edges are recorded by the run that produced them (the
//! `action_world` loop records its own arrows); nothing in this module
//! knows where an edge came from, and nothing is authored.
//!
//! AUDIT (structural constants only): the declared canonical enumeration
//! (breadth-first, shortest-first then lexicographic within a length) and
//! the 64-bit FNV-1a digest (reproducibility, not cryptography). No
//! threshold, no default depth, no heuristic, no verdict.

use core::fmt;
use std::collections::BTreeSet;

/// Breadth-first enumeration (the canonical order's generator).
fn walks_breadth_first(adjacency: &[(Node, Node)], origin: Node, max_hops: usize) -> Vec<Walk> {
    let mut walks: Vec<Walk> = Vec::new();
    let mut frontier: Vec<Walk> = vec![Walk {
        nodes: vec![origin],
    }];
    for _ in 0..max_hops {
        let mut next: Vec<Walk> = Vec::new();
        for walk in &frontier {
            let last = walk.nodes[walk.nodes.len() - 1];
            for (_, node) in CausalGraph::step(adjacency, last) {
                let mut nodes = walk.nodes.clone();
                nodes.push(*node);
                next.push(Walk { nodes });
            }
        }
        walks.extend(next.iter().cloned());
        frontier = next;
    }
    walks
}

/// Depth-first enumeration (the Rule 2 variant: same set, different order).
fn walks_depth_first(adjacency: &[(Node, Node)], origin: Node, max_hops: usize) -> Vec<Walk> {
    let mut walks: Vec<Walk> = Vec::new();
    let mut stack: Vec<Walk> = vec![Walk {
        nodes: vec![origin],
    }];
    while let Some(walk) = stack.pop() {
        if walk.nodes.len() > 1 {
            walks.push(walk.clone());
        }
        if walk.nodes.len() > max_hops {
            continue;
        }
        let last = walk.nodes[walk.nodes.len() - 1];
        // Reversed push: the canonical successor order is consumed first.
        for (_, node) in CausalGraph::step(adjacency, last).iter().rev() {
            let mut nodes = walk.nodes.clone();
            nodes.push(*node);
            stack.push(Walk { nodes });
        }
    }
    walks
}

/// The node spaces: one lattice, two kinds of thing. An `Action` cell is
/// a wavevector the engine emitted; a `Content` cell is a produced
/// difference bin. The type is the space; the direction is the edge's
/// orientation.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Space {
    Action,
    Content,
}

/// A typed lattice cell.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Node {
    pub space: Space,
    pub cell: (i64, i64),
}

impl Node {
    pub fn action(cell: (i64, i64)) -> Self {
        Node {
            space: Space::Action,
            cell,
        }
    }

    pub fn content(cell: (i64, i64)) -> Self {
        Node {
            space: Space::Content,
            cell,
        }
    }
}

/// One recorded arrow: what produced what, in which epoch of the run.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Edge {
    pub from: Node,
    pub to: Node,
    pub epoch: usize,
}

/// The two cone directions. `Future` follows the arrows (what this can
/// produce); `Past` follows them backwards (what could have produced
/// this).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Past,
    Future,
}

/// The declared enumeration orders (Rule 2). Breadth-first is canonical:
/// shortest-first, then lexicographic within a length. Depth-first is the
/// measured variant — the same walk SET in a different order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WalkOrder {
    BreadthFirst,
    DepthFirst,
}

/// An ordered walk of at least one edge.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Walk {
    pub nodes: Vec<Node>,
}

/// A hop-bounded traversal. `truncated` is true when a walk endpoint at
/// the bound still has a continuation in this direction — the bound cut
/// structure, which is reported rather than silently dropped.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Traversal {
    pub origin: Node,
    pub direction: Direction,
    pub max_hops: usize,
    pub walks: Vec<Walk>,
    pub edges: Vec<Edge>,
    pub truncated: bool,
}

/// Fail-closed readout: an origin that is not in the graph is named.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PathError {
    UnknownNode(Node),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::UnknownNode(node) => write!(
                f,
                "unknown node: {:?} cell {:?}",
                node.space, node.cell
            ),
        }
    }
}

/// The traversal's receipt: what was asked, what came back, and a digest
/// over the canonical form (walk order and edge recording order cannot
/// change it). The digest is a 64-bit FNV-1a: a reproducibility claim,
/// not a cryptographic one.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Receipt {
    pub source: String,
    pub origin: Node,
    pub direction: Direction,
    pub max_hops: usize,
    pub walks: usize,
    pub edges: usize,
    pub truncated: bool,
    pub digest: u64,
}

/// The recorded directed graph.
#[derive(Clone, Default, Debug)]
pub struct CausalGraph {
    edges: Vec<Edge>,
}

impl CausalGraph {
    pub fn new() -> Self {
        CausalGraph { edges: Vec::new() }
    }

    /// Record one arrow. Called by the run that produced it; the graph
    /// never infers or authors an edge.
    pub fn record(&mut self, from: Node, to: Node, epoch: usize) {
        self.edges.push(Edge { from, to, epoch });
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// The graph's nodes, canonical (sorted, deduplicated).
    pub fn nodes(&self) -> Vec<Node> {
        let mut nodes: BTreeSet<Node> = BTreeSet::new();
        for edge in &self.edges {
            nodes.insert(edge.from);
            nodes.insert(edge.to);
        }
        nodes.into_iter().collect()
    }

    /// The successors in the asked direction — a sorted, deduplicated pair
    /// list, so a traversal costs a sort rather than a map build and a
    /// dense graph does not make the readout quadratic.
    fn step(adjacency: &[(Node, Node)], node: Node) -> &[(Node, Node)] {
        let start = adjacency.partition_point(|(from, _)| *from < node);
        let end = adjacency.partition_point(|(from, _)| *from <= node);
        &adjacency[start..end]
    }

    /// The adjacency in one direction, canonical.
    fn adjacency(&self, direction: Direction) -> Vec<(Node, Node)> {
        let mut pairs: Vec<(Node, Node)> = self
            .edges
            .iter()
            .map(|edge| match direction {
                Direction::Future => (edge.from, edge.to),
                Direction::Past => (edge.to, edge.from),
            })
            .collect();
        pairs.sort();
        pairs.dedup();
        pairs
    }

    /// The hop-bounded readout. Enumerates walks of 1..=max_hops edges in
    /// the asked direction and returns them in the declared order —
    /// sequences, not a membership bit.
    pub fn traverse(
        &self,
        origin: Node,
        direction: Direction,
        max_hops: usize,
        order: WalkOrder,
    ) -> Result<Traversal, PathError> {
        if !self
            .edges
            .iter()
            .any(|edge| edge.from == origin || edge.to == origin)
        {
            return Err(PathError::UnknownNode(origin));
        }
        let adjacency = self.adjacency(direction);
        let mut walks: Vec<Walk> = match order {
            WalkOrder::BreadthFirst => walks_breadth_first(&adjacency, origin, max_hops),
            WalkOrder::DepthFirst => walks_depth_first(&adjacency, origin, max_hops),
        };
        if order == WalkOrder::BreadthFirst {
            // Canonical: shortest-first, then lexicographic within a length.
            walks.sort_by(|a, b| (a.nodes.len(), &a.nodes).cmp(&(b.nodes.len(), &b.nodes)));
        }
        walks.dedup();

        // The traversed edges: the recorded arrows the walks used (the
        // same set as scanning every walk against every edge).
        let mut used_pairs: Vec<(Node, Node)> = Vec::new();
        for walk in &walks {
            for pair in walk.nodes.windows(2) {
                used_pairs.push((pair[0], pair[1]));
            }
        }
        used_pairs.sort();
        used_pairs.dedup();
        let mut edges: Vec<Edge> = self
            .edges
            .iter()
            .filter(|edge| {
                used_pairs.binary_search(&(edge.from, edge.to)).is_ok()
                    || used_pairs.binary_search(&(edge.to, edge.from)).is_ok()
            })
            .copied()
            .collect();
        edges.sort();

        // The bound cut structure iff a walk endpoint at the bound still
        // has a continuation in this direction.
        let mut frontier: Vec<Node> = walks
            .iter()
            .filter(|walk| walk.nodes.len() == max_hops + 1)
            .map(|walk| walk.nodes[walk.nodes.len() - 1])
            .collect();
        frontier.sort();
        frontier.dedup();
        let truncated = frontier
            .iter()
            .any(|node| !Self::step(&adjacency, *node).is_empty());

        Ok(Traversal {
            origin,
            direction,
            max_hops,
            walks,
            edges,
            truncated,
        })
    }
}

impl Traversal {
    /// The receipt over the canonical form: independent of the walk order
    /// and of the order the edges were recorded in.
    pub fn receipt(&self, source: &str) -> Receipt {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(source.as_bytes());
        bytes.push(0);
        bytes.push(match self.direction {
            Direction::Past => 1,
            Direction::Future => 2,
        });
        bytes.extend_from_slice(&(self.max_hops as u64).to_le_bytes());

        let mut walks = self.walks.clone();
        walks.sort_by(|a, b| (a.nodes.len(), &a.nodes).cmp(&(b.nodes.len(), &b.nodes)));
        for walk in &walks {
            bytes.extend_from_slice(&(walk.nodes.len() as u64).to_le_bytes());
            for node in &walk.nodes {
                push_node(&mut bytes, *node);
            }
        }
        let mut edges = self.edges.clone();
        edges.sort();
        for edge in &edges {
            push_node(&mut bytes, edge.from);
            push_node(&mut bytes, edge.to);
            bytes.extend_from_slice(&(edge.epoch as u64).to_le_bytes());
        }
        bytes.push(self.truncated as u8);

        Receipt {
            source: source.to_string(),
            origin: self.origin,
            direction: self.direction,
            max_hops: self.max_hops,
            walks: self.walks.len(),
            edges: self.edges.len(),
            truncated: self.truncated,
            digest: fnv1a_64(&bytes),
        }
    }
}

fn push_node(bytes: &mut Vec<u8>, node: Node) {
    bytes.push(match node.space {
        Space::Action => 1,
        Space::Content => 2,
    });
    bytes.extend_from_slice(&node.cell.0.to_le_bytes());
    bytes.extend_from_slice(&node.cell.1.to_le_bytes());
}

/// The canonical-form digest (BUILD-28's, shared with the builds that reuse
/// this primitive): 64-bit FNV-1a over a canonical byte stream — a
/// reproducibility claim, never a cryptographic one.
pub fn digest(bytes: &[u8]) -> u64 {
    fnv1a_64(bytes)
}

/// 64-bit FNV-1a over the canonical byte stream.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
