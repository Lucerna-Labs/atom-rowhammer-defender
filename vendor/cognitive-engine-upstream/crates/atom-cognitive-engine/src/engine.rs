//! The engine executor. Stage dispatch is by stage id; every stage mutates
//! the shared cycle context, so order of execution is order of effect.
//!
//! v0.3 additions: directional echo **bonds** between traces (facet-
//! dissimilar memories linked explicitly), echo **consolidation** on write
//! (observing a downstream memory pays a support bonus to its upstream
//! partners), **tiers** (Core identity substrate is protected from revision
//! dissolution and capacity eviction), and bond-**expansion** retrieval
//! (queries activate bonded neighbors up to a configurable depth, subject
//! to the reliability gate). Queries remain read-only — bonds strengthen
//! only through observation, never through retrieval.

use crate::config::EngineConfig;
use crate::stacks;
use crate::trace::{MemoryTrace, TraceTier, cosine_similarity, seeded_unit, softmax};
use atom_cognitive_core::digest::canonical_hash;
use atom_cognitive_core::stages::Stack;
use std::collections::HashMap;

/// Engine construction and runtime failures. All fail-closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    InvalidConfig(String),
    InvalidObservation(String),
    InvalidQuery(String),
    StageNotRunnable {
        stage: String,
        cycle: &'static str,
    },
    /// Capacity is exhausted and every active trace is Core (protected).
    CapacityExhausted,
    /// Bond endpoints must exist, be distinct, and not duplicate.
    InvalidBond(String),
}

impl core::fmt::Display for EngineError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EngineError::InvalidConfig(what) => write!(f, "invalid config: {what}"),
            EngineError::InvalidObservation(what) => write!(f, "invalid observation: {what}"),
            EngineError::InvalidQuery(what) => write!(f, "invalid query: {what}"),
            EngineError::StageNotRunnable { stage, cycle } => {
                write!(f, "stage {stage} does not belong to the {cycle} cycle")
            }
            EngineError::CapacityExhausted => {
                write!(
                    f,
                    "capacity exhausted; every active trace is Core-protected"
                )
            }
            EngineError::InvalidBond(what) => write!(f, "invalid bond: {what}"),
        }
    }
}

impl core::error::Error for EngineError {}

/// Result of one observe cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct ObserveOutcome {
    pub tick: u64,
    pub action: &'static str,
    pub predicted: Option<usize>,
    pub surprised: bool,
    pub state_hash: String,
}

/// One generated blend (D37): a KPR Candidate born from two activated
/// parents, with its novelty score.
#[derive(Debug, Clone, PartialEq)]
pub struct BlendReport {
    pub id: u64,
    pub parents: (u64, u64),
    /// 1 − max similarity to any committed trace (how far the blend sits
    /// from everything already accepted).
    pub novelty: f64,
}

/// Multiscale renormalization report (#5): how the memory was
/// re-described.
pub struct RenormalizationReport {
    /// Grains formed (seeds that absorbed at least one member).
    pub grains: usize,
    /// Traces absorbed into grains (now resting inactive inside them).
    pub traces_merged: usize,
}

/// One Pareto-front contender (#6): an eligible trace no other eligible
/// trace dominates on (reliability, support, novelty). The front NAMES
/// the tension instead of collapsing it into one score — pure metadata:
/// it never changes the assertion and is excluded from the response
/// hash.
#[derive(Debug, Clone, PartialEq)]
pub struct FrontMember {
    pub trace_id: u64,
    /// The contender's asserted value (its evidence argmax).
    pub value: usize,
    pub reliability: f64,
    pub support: f64,
    pub novelty: f64,
}

/// A marking query's result (#14): the ordinary outcome plus how many
/// bond edges the expansion walked and received pheromone (bounded at
/// `pheromone_cap`). Plain queries never mark.
pub struct MarkingOutcome {
    pub outcome: QueryOutcome,
    pub edges_marked: usize,
}

/// CRDT merge report (#9): how the union landed.
pub struct MergeReport {
    /// Fresh traces absorbed (renumbered into this engine).
    pub traces_absorbed: usize,
    /// Ancestor-shared traces joined field-wise (max).
    pub traces_joined: usize,
    /// Exact-content duplicates collapsed (idempotent re-merge).
    pub traces_deduplicated: usize,
    /// Bonds added (existing pairs strengthen by pheromone max instead).
    pub bonds_added: usize,
}

/// Quorum-sensing read (#15): how many DISTINCT supporters backed this
/// retrieval (correlated copies cluster once) and whether that met the
/// quorum bar for expensive deliberation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuorumReport {
    pub distinct_supporters: usize,
    pub quorum_met: bool,
}

/// Information-bottleneck report (#8): what the compression removed,
/// accounted exactly.
pub struct BottleneckReport {
    /// Traces that had at least one dust entry pruned.
    pub traces_touched: usize,
    /// Evidence entries zeroed (shares below the dust threshold).
    pub entries_pruned: usize,
    /// Total evidence mass removed.
    pub mass_removed: f64,
}

/// The kind of observation the engine recommends (D39, expected
/// information gain).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationKind {
    /// A committed trace that matches the query but is blocked ONLY by
    /// its reliability floor: observe its cue again to raise reliability.
    Reinforce,
    /// The query's own cue: no committed structure is close enough —
    /// probe virgin territory and let KPR verify whatever answers.
    Explore,
}

/// A recommended next observation for an uncertain query. The score is a
/// deterministic gap proxy (smaller blocker gap → higher expected gain),
/// NOT a probability. Plans never assert and never mutate: they only pay
/// off through KPR-verified observation.
#[derive(Debug, Clone, PartialEq)]
pub struct ObservationPlan {
    pub probe_cue: Vec<f64>,
    pub target_trace: Option<u64>,
    pub kind: ObservationKind,
    pub score: f64,
    pub reason: String,
}

/// Result of one query cycle. `claim: None` is the deterministic
/// abstention — the engine's differentiator, never to be worked around.
/// `activations` lists the top retrieved traces (id, mixture weight) so
/// callers can cite the memories behind an answer.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryOutcome {
    pub claim: Option<(usize, f64)>,
    pub eligible_count: usize,
    /// Top activations for citation (id, mixture weight), capped at 8.
    pub activations: Vec<(u64, f64)>,
    /// The full ranked activation set (uncapped) — reachability surface for
    /// tests and bond diagnostics.
    pub activated_all: Vec<u64>,
    /// The metacognitive read (W16): the SHAPE of this retrieval as a
    /// confidence signal. Pure metadata — read-only, never upgrades an
    /// abstention into an assertion, and fluency does not imply
    /// correctness (a confident-wrong trace retrieves fluently).
    pub fluency: FluencyReport,
    /// The Pareto front (#6): non-dominated eligible traces over
    /// (reliability, support, novelty). Empty when the knob is off.
    /// Metadata only — never changes the claim and is excluded from the
    /// response hash.
    pub front: Vec<FrontMember>,
    /// The quorum read (#15): distinct supporters and whether
    /// deliberation was earned. Zero-supporter report when off.
    pub quorum: QuorumReport,
    pub response_hash: String,
}

/// The named fluency states (W16, after the tip-of-the-tongue
/// literature): how the retrieval FELT, independent of what it returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluencyState {
    /// Concentrated support, clear margin: "I know this."
    Fluent,
    /// Evidence on both sides of a near tie: "something's there, but it
    /// isn't settled." The natural input to deliberation or EIG.
    Contested,
    /// Thin support (few traces): "fragments, not a ground."
    Sparse,
    /// Nothing eligible: matches the abstention.
    Blank,
}

impl FluencyState {
    pub fn describe(self) -> &'static str {
        match self {
            FluencyState::Fluent => "fluent retrieval: concentrated support, clear margin",
            FluencyState::Contested => "contested retrieval: evidence on both sides of a near tie",
            FluencyState::Sparse => "sparse retrieval: thin support, fragile ground",
            FluencyState::Blank => "blank retrieval: nothing eligible",
        }
    }
}

/// The shape of one retrieval as a metacognitive signal (W16):
/// eligible count, the top member's weight share, and the mixture
/// margin between the winning value and its runner-up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FluencyReport {
    pub state: FluencyState,
    pub eligible_count: usize,
    pub top_weight: f64,
    pub margin: f64,
}

/// One boundary-verified transition record (bounded log, last 64 kept).
#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    pub tick: u64,
    pub action: &'static str,
    pub surprised: bool,
    pub predicted: Option<usize>,
    pub state_hash: String,
}

/// A directed echo bond: `upstream` is the earlier memory whose meaning
/// carries into `downstream` (kitchen → bakery). Consolidation flows
/// upstream; expansion walks both directions. `pheromone` (#14) is the
/// bounded stigmergic work-trace: marking queries deposit on the edges
/// they traverse, evaporation fades it on the time base, and retrieval
/// routing may read it — but evidence never does (the
/// routing-not-evidence law, D52).
#[derive(Debug, Clone, PartialEq)]
pub struct Bond {
    pub upstream: u64,
    pub downstream: u64,
    pub pheromone: f64,
}

/// Per-query knob overrides (the persona governor's control surface). The
/// persona may sharpen or soften attention and gating for one query — it
/// can never dismantle them: `clamped()` enforces hard safety floors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QueryTuning {
    pub attention_temperature: f64,
    pub query_threshold: f64,
    pub min_reliability: f64,
    pub bond_expansion_depth: usize,
    /// Robust aggregation (W14): max weight share per trace (1.0 = off).
    pub robust_weight_cap: f64,
    /// Robust aggregation (W14): lowest-weight members trimmed (0 = off).
    pub robust_trim: usize,
    /// Occam/dedup (W17): correlated near-copies collapse to their
    /// strongest member before the cap (1.01 = off).
    pub dedup_threshold: f64,
    /// Satisficing (W18): skip bond expansion when the eligible-only
    /// margin already meets this bar (0.0 = off — always deliberate).
    pub satisficing_margin: f64,
    /// Pareto preservation (#6): also return the non-dominated front of
    /// eligible traces over (reliability, support, novelty). Metadata
    /// only — the front never changes the assertion or response hash.
    pub pareto_front: bool,
    /// Stigmergy (#14): pheromone DEPOSITED on each traversed bond by a
    /// MARKING query (plain queries never deposit). 0.0 = off.
    pub stigmergy_rate: f64,
    /// Stigmergy (#14): how strongly retrieval routing reads bond
    /// pheromone (routing only — evidence never sees it). 0.0 = off.
    pub stigmergy_gain: f64,
    /// Lateral inhibition (#13): the winner softly suppresses its
    /// near-neighbors (×(1 − g·sim²)); the mixture re-normalizes.
    pub lateral_inhibition: f64,
    /// Quorum sensing (#15): expansion runs only when at least this
    /// many DISTINCT supporters (correlated copies count once) are
    /// eligible. 0 = off.
    pub quorum_min: usize,
}

impl QueryTuning {
    pub fn from_config(config: &EngineConfig) -> QueryTuning {
        QueryTuning {
            attention_temperature: config.attention_temperature,
            query_threshold: config.query_threshold,
            min_reliability: config.min_reliability,
            bond_expansion_depth: config.bond_expansion_depth,
            robust_weight_cap: config.robust_weight_cap,
            robust_trim: config.robust_trim,
            dedup_threshold: config.dedup_threshold,
            satisficing_margin: config.satisficing_margin,
            pareto_front: config.pareto_front,
            stigmergy_rate: config.stigmergy_rate,
            stigmergy_gain: config.stigmergy_gain,
            lateral_inhibition: config.lateral_inhibition,
            quorum_min: config.quorum_min,
        }
    }

    /// Fail-closed bounds: temperatures stay positive and bounded, the
    /// gate threshold never drops below 0.5, reliability floors never drop
    /// below 0.05, expansion never exceeds depth 8 (raised from 4 —
    /// 2026-08-30, operator: "increase the hops"), robust and dedup
    /// knobs stay in their validated ranges.
    pub fn clamped(self) -> QueryTuning {
        QueryTuning {
            attention_temperature: self.attention_temperature.clamp(0.5, 10.0),
            query_threshold: self.query_threshold.clamp(0.5, 0.999),
            min_reliability: self.min_reliability.clamp(0.05, 1.0),
            bond_expansion_depth: self.bond_expansion_depth.min(8),
            robust_weight_cap: self.robust_weight_cap.clamp(0.15, 1.0),
            robust_trim: self.robust_trim.min(8),
            dedup_threshold: self.dedup_threshold.clamp(0.5, 1.01),
            satisficing_margin: self.satisficing_margin.clamp(0.0, 1.0),
            pareto_front: self.pareto_front,
            stigmergy_rate: self.stigmergy_rate.clamp(0.0, 1.0),
            stigmergy_gain: self.stigmergy_gain.clamp(0.0, 1.0),
            lateral_inhibition: self.lateral_inhibition.clamp(0.0, 1.0),
            quorum_min: self.quorum_min.min(64),
        }
    }
}

/// Cycle-internal context. Field defaults are the no-op values; stages fill
/// them in execution order, which is exactly why order matters.
#[derive(Debug, Clone)]
struct ObserveCtx {
    cue: Vec<f64>,
    value: usize,
    /// Full multi-valued content (single-element for plain observe).
    values: Vec<usize>,
    salience: f64,
    predicted: Option<usize>,
    surprised: bool,
    effective_salience: f64,
    sims: Vec<(usize, f64)>,
    bind_target: Option<usize>,
    /// Set by the KPR candidate gate: nucleations land in the Candidate
    /// tier instead of the ingest tier.
    candidate_mode: bool,
    /// Causal origin (#4): the attribution id of this observation's
    /// source (0 = unattributed). Concurrency = two different explicit
    /// origins writing the same trace.
    origin: u64,
}

#[derive(Debug, Clone)]
struct QueryCtx {
    cue: Vec<f64>,
    logits: Vec<(usize, f64)>,
    eligible: Vec<usize>,
    /// Bond-expanded members discovered by the expand stage:
    /// (trace index, hop depth, edge pheromone at read time, parent id)
    /// — the walked edges, kept for stigmergic deposition (#14).
    expansions: Vec<(usize, usize, f64, u64)>,
    gate_ran: bool,
    mixture: Vec<f64>,
    member_weights: Vec<(u64, f64)>,
    /// Effective knobs for this query (config defaults or governor
    /// tuning, always clamped to safety floors).
    knobs: QueryTuning,
    /// Quorum sensing (#15): the distinct-supporter count and whether
    /// the quorum gate allowed deliberation. Metadata for the outcome.
    distinct_supporters: usize,
    quorum_met: bool,
}

#[derive(Clone)]
pub struct CognitiveEngine {
    config: EngineConfig,
    traces: Vec<MemoryTrace>,
    bonds: Vec<Bond>,
    ingest_tier: TraceTier,
    next_id: u64,
    tick: u64,
    rng_state: u64,
    explore_temperature: f64,
    actions: HashMap<&'static str, u64>,
    surprises: u64,
    forgotten: u64,
    observations: u64,
    transitions: Vec<Transition>,
    perceptions: Vec<crate::perception::Perception>,
}

impl CognitiveEngine {
    pub fn new(config: EngineConfig) -> Result<CognitiveEngine, EngineError> {
        config.validate().map_err(EngineError::InvalidConfig)?;
        let seed = config.seed;
        Ok(CognitiveEngine {
            rng_state: seed | 1,
            explore_temperature: config.explore_initial,
            config,
            traces: Vec::new(),
            bonds: Vec::new(),
            ingest_tier: TraceTier::Standard,
            next_id: 1,
            tick: 0,
            actions: HashMap::new(),
            surprises: 0,
            forgotten: 0,
            observations: 0,
            transitions: Vec::new(),
            perceptions: Vec::new(),
        })
    }

    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    pub fn traces(&self) -> &[MemoryTrace] {
        &self.traces
    }

    pub fn active_traces(&self) -> Vec<&MemoryTrace> {
        self.traces.iter().filter(|t| t.active).collect()
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn explore_temperature(&self) -> f64 {
        self.explore_temperature
    }

    pub fn surprise_count(&self) -> u64 {
        self.surprises
    }

    pub fn observations(&self) -> u64 {
        self.observations
    }

    pub fn action_count(&self, action: &str) -> u64 {
        self.actions.get(action).copied().unwrap_or(0)
    }

    pub fn forgotten_count(&self) -> u64 {
        self.forgotten
    }

    pub fn transitions(&self) -> &[Transition] {
        &self.transitions
    }

    pub fn bond_count(&self) -> usize {
        self.bonds.len()
    }

    /// Remove every bond (placebo construction: identical traces, random
    /// wiring). Traces and evidence are untouched.
    pub fn clear_bonds(&mut self) {
        self.bonds.clear();
    }

    /// Generate `count` conceptual blends from the memories a cue
    /// activates (D37 — the generation half of blind variation +
    /// selective retention). Each blend mixes two activated parents'
    /// cues and evidence, is born a KPR CANDIDATE with the parents
    /// recorded as provenance, and carries a novelty score (1 − max
    /// similarity to any committed trace). Blends are query-invisible
    /// and never assert: the environment selects them by presenting
    /// recurring evidence near their cue (promotion), or they dissipate.
    pub fn imagine(&mut self, cue: &[f64], count: usize) -> Result<Vec<BlendReport>, EngineError> {
        self.validate_cue(cue)?;
        if !(1..=8).contains(&count) {
            return Err(EngineError::InvalidQuery(format!(
                "blend count {count} out of range 1..=8"
            )));
        }
        // Which committed memories does this cue activate? (The query gate
        // already excludes Candidates — generation reads only committed
        // structure.)
        let stack = stacks::query_stack();
        let outcome = self.query(&stack, cue)?;
        let activated: Vec<u64> = outcome.activated_all.into_iter().collect();
        if activated.len() < 2 {
            return Ok(Vec::new()); // nothing to blend
        }
        // Deterministic pairing by activation rank: (0,1), (2,3), ...
        let mut reports = Vec::new();
        let mut pairs = activated.chunks(2).filter(|pair| pair.len() == 2);
        for pair in pairs.by_ref() {
            if reports.len() >= count {
                break;
            }
            let (left, right) = (pair[0], pair[1]);
            let (left_trace, right_trace) = match (
                self.traces.iter().find(|t| t.id == left && t.active),
                self.traces.iter().find(|t| t.id == right && t.active),
            ) {
                (Some(l), Some(r)) => (l, r),
                _ => continue,
            };
            // Blending near-identical parents adds nothing.
            if cosine_similarity(&left_trace.cue, &right_trace.cue) > 0.95 {
                continue;
            }
            // Child cue: normalized elementwise mean.
            let mut child_cue: Vec<f64> = left_trace
                .cue
                .iter()
                .zip(right_trace.cue.iter())
                .map(|(a, b)| (a + b) / 2.0)
                .collect();
            let norm: f64 = child_cue.iter().map(|v| v * v).sum::<f64>().sqrt();
            if norm <= f64::EPSILON {
                continue;
            }
            for value in child_cue.iter_mut() {
                *value /= norm;
            }
            // Child evidence: elementwise mean (inherits both parents'
            // supported values — recombination, not invention).
            let child_evidence: Vec<f64> = left_trace
                .evidence
                .iter()
                .zip(right_trace.evidence.iter())
                .map(|(a, b)| (a + b) / 2.0)
                .collect();
            // Novelty: distance from everything committed — parents
            // INCLUDED. The parents are the accepted structure; a blend
            // of distant parents sits at ~0.71 from each (novelty ≈
            // 0.29), a blend of near ones at ~0.99 (novelty ≈ 0.01).
            let novelty = 1.0
                - self
                    .traces
                    .iter()
                    .filter(|t| t.active && t.tier != TraceTier::Candidate)
                    .map(|t| cosine_similarity(&child_cue, &t.cue))
                    .fold(0.0_f64, f64::max);
            let tick = self.tick;
            let id = self.next_id;
            self.traces.push(MemoryTrace {
                id,
                cue: child_cue,
                evidence: child_evidence,
                mass: self.config.initial_mass * 0.75,
                support: self.config.initial_support * 0.5,
                ttl: self.config.initial_ttl,
                hits: 1,
                stages: 1,
                last_tick_staged: tick,
                contradictions: 0,
                created_tick: tick,
                last_tick: tick,
                active: true,
                tier: TraceTier::Candidate,
                parents: vec![left, right],
                grain_members: Vec::new(),
                novelty: 1.0,
                origin: 0,
            });
            self.next_id += 1;
            *self.actions.entry("imagine").or_insert(0) += 1;
            reports.push(BlendReport {
                id,
                parents: (left, right),
                novelty,
            });
        }
        Ok(reports)
    }

    /// Multiscale renormalization (#5): coarse-grain the active memory
    /// while conserving evidence. Every still-active trace whose cue is
    /// at least `granularity` similar to a seed merges into it (seed =
    /// lowest id): EVIDENCE sums elementwise (the conserved quantity —
    /// minority dissent survives inside the merged distribution),
    /// support/mass keep the max (status, not substance), hits and
    /// contradictions add, novelty keeps the max (the freshest member
    /// keeps the grain sensitive). Members rest INACTIVE inside the
    /// grain with their evidence frozen; the fine ledger
    /// (`grain_members`) records what was removed. Bonds touching a
    /// member rewire to the grain so connectivity survives
    /// re-description. A later CONTRADICTION at the coarse scale splits
    /// the grain (see `split_grain`) — what was removed becomes
    /// reachable again. Unlike W17 dedup, nothing is dropped, only
    /// re-described. Candidates never merge (provisional evidence has
    /// no coarse form). 1.01 = off (merges nothing, state byte-equal).
    pub fn renormalize(&mut self, granularity: f64) -> Result<RenormalizationReport, EngineError> {
        if !(0.5..=1.01).contains(&granularity) {
            return Err(EngineError::InvalidObservation(format!(
                "renormalization granularity {granularity} out of range [0.5, 1.01]"
            )));
        }
        let mut grains = 0usize;
        let mut merged = 0usize;
        // Seeds by id order; membership re-checked per seed against the
        // CURRENT active set (absorbed traces can never re-absorb).
        let seeds: Vec<u64> = self
            .traces
            .iter()
            .filter(|t| t.active && t.tier != TraceTier::Candidate)
            .map(|t| t.id)
            .collect();
        for seed in seeds {
            let Some(seed_index) = self.traces.iter().position(|t| t.id == seed) else {
                continue;
            };
            if !self.traces[seed_index].active
                || self.traces[seed_index].tier == TraceTier::Candidate
            {
                continue; // absorbed by an earlier grain
            }
            let member_ids: Vec<u64> = self
                .traces
                .iter()
                .filter(|t| {
                    t.active
                        && t.id != seed
                        && t.tier != TraceTier::Candidate
                        && cosine_similarity(&t.cue, &self.traces[seed_index].cue) >= granularity
                })
                .map(|t| t.id)
                .collect();
            if member_ids.is_empty() {
                continue;
            }
            grains += 1;
            for member_id in member_ids {
                let member_index = self
                    .traces
                    .iter()
                    .position(|t| t.id == member_id)
                    .expect("active member located");
                let (evidence, support, mass, hits, contradictions, ttl, novelty, ledger) = {
                    let member = &self.traces[member_index];
                    (
                        member.evidence.clone(),
                        member.support,
                        member.mass,
                        member.hits,
                        member.contradictions,
                        member.ttl,
                        member.novelty,
                        member.grain_members.clone(),
                    )
                };
                {
                    let grain = &mut self.traces[seed_index];
                    for (slot, value) in evidence.iter().enumerate() {
                        grain.evidence[slot] += value;
                    }
                    grain.support = grain.support.max(support);
                    grain.mass = grain.mass.max(mass);
                    grain.hits += hits;
                    grain.contradictions += contradictions;
                    grain.ttl = grain.ttl.max(ttl);
                    grain.novelty = grain.novelty.max(novelty);
                    grain.grain_members.push(member_id);
                    grain.grain_members.extend(ledger); // nested ledgers concatenate
                    grain.last_tick = self.tick;
                }
                self.traces[member_index].active = false;
                merged += 1;
                self.rewire_bonds(member_id, seed);
            }
        }
        if merged == 0 {
            return Ok(RenormalizationReport {
                grains: 0,
                traces_merged: 0,
            });
        }
        *self.actions.entry("renormalize").or_insert(0) += 1;
        let state_hash = self.state_hash();
        self.transitions.push(Transition {
            tick: self.tick,
            action: "renormalize",
            surprised: false,
            predicted: None,
            state_hash,
        });
        if self.transitions.len() > 64 {
            self.transitions.remove(0);
        }
        Ok(RenormalizationReport {
            grains,
            traces_merged: merged,
        })
    }

    /// Information bottleneck (#8): compress every trace's evidence by
    /// pruning entries whose SHARE of the trace's total evidence is
    /// below `dust` — the dissent dust that weak pings accumulate (D54
    /// makes those dissents small by design). The target-predictive
    /// structure (argmax and its margin) is retained PROVABLY: if the
    /// argmax entry were below dust×total then every entry is, so
    /// total < value_count·dust·total — impossible while
    /// dust ≤ 1/(2·value_count), which validation enforces. The report
    /// accounts exactly what was removed. Explicit operation only
    /// (cannot fire by accident); dust 0.0 is a clean no-op.
    pub fn compress_evidence(&mut self, dust: f64) -> Result<BottleneckReport, EngineError> {
        let bound = 1.0 / (2.0 * self.config.value_count as f64);
        if !(0.0..=bound).contains(&dust) {
            return Err(EngineError::InvalidObservation(format!(
                "bottleneck dust {dust} out of range [0, {bound:.4}]"
            )));
        }
        let mut traces_touched = 0usize;
        let mut entries_pruned = 0usize;
        let mut mass_removed = 0.0f64;
        let tick = self.tick;
        for index in self.active_indices() {
            let trace = &mut self.traces[index];
            let total: f64 = trace.evidence.iter().sum();
            if total <= f64::EPSILON {
                continue;
            }
            let mut pruned_here = 0usize;
            for entry in trace.evidence.iter_mut() {
                if *entry > 0.0 && *entry < dust * total {
                    mass_removed += *entry;
                    *entry = 0.0;
                    pruned_here += 1;
                }
            }
            if pruned_here > 0 {
                traces_touched += 1;
                entries_pruned += pruned_here;
                trace.last_tick = tick;
            }
        }
        if entries_pruned == 0 {
            return Ok(BottleneckReport {
                traces_touched: 0,
                entries_pruned: 0,
                mass_removed: 0.0,
            });
        }
        *self.actions.entry("bottleneck").or_insert(0) += 1;
        let state_hash = self.state_hash();
        self.transitions.push(Transition {
            tick: self.tick,
            action: "bottleneck",
            surprised: false,
            predicted: None,
            state_hash,
        });
        if self.transitions.len() > 64 {
            self.transitions.remove(0);
        }
        Ok(BottleneckReport {
            traces_touched,
            entries_pruned,
            mass_removed,
        })
    }

    /// Replace every bond endpoint `from` with `to`; self-loops and
    /// duplicates formed by the rewrite drop out. Used by
    /// renormalization so connectivity survives re-description.
    fn rewire_bonds(&mut self, from: u64, to: u64) {
        let mut kept: Vec<Bond> = Vec::with_capacity(self.bonds.len());
        for bond in self.bonds.drain(..) {
            let upstream = if bond.upstream == from {
                to
            } else {
                bond.upstream
            };
            let downstream = if bond.downstream == from {
                to
            } else {
                bond.downstream
            };
            if upstream == downstream {
                continue;
            }
            if kept
                .iter()
                .any(|b| b.upstream == upstream && b.downstream == downstream)
            {
                continue;
            }
            kept.push(Bond {
                upstream,
                downstream,
                pheromone: bond.pheromone,
            });
        }
        self.bonds = kept;
    }

    /// The renormalization rollback: a contradicted grain SPLITS. Every
    /// member re-activates with the evidence it had at merge time
    /// (frozen — members must re-earn anything accrued since); the
    /// falsified coarse description is discarded (the grain
    /// deactivates, taking its post-merge earnings with it). Bond
    /// rewiring from the merge is NOT undone — a recorded asymmetry
    /// (D49): the split members do not inherit the grain's connections.
    fn split_grain(&mut self, grain_index: usize) {
        let member_ids = self.traces[grain_index].grain_members.clone();
        for member_id in &member_ids {
            if let Some(trace) = self.traces.iter_mut().find(|t| t.id == *member_id) {
                trace.active = true;
                trace.last_tick = self.tick;
            }
        }
        self.traces[grain_index].grain_members.clear();
        self.traces[grain_index].active = false;
        *self.actions.entry("split").or_insert(0) += 1;
    }

    /// Expected information gain (D39): when a query is uncertain or
    /// abstains, recommend the available observation expected to reduce
    /// uncertainty most. Read-only — recommending never changes state,
    /// and plans only pay off through KPR-verified observation.
    ///
    /// Two plan kinds, ranked by deterministic gap proxies (not
    /// probabilities):
    /// - **Reinforce**: a committed trace whose similarity already
    ///   clears the query gate but whose reliability is below the floor.
    ///   Observing its cue raises reliability through normal
    ///   reinforcement — the direct unlock.
    /// - **Explore**: the query's own cue when nothing committed is
    ///   close enough. Probing virgin territory lets KPR verify whatever
    ///   pattern answers. Scored by the best near-miss similarity
    ///   (plausibility that structure exists just out of reach); below
    ///   `eig_min_score` the honest answer is "no available observation
    ///   reduces this uncertainty."
    pub fn recommend_observation(&self, cue: &[f64]) -> Result<Vec<ObservationPlan>, EngineError> {
        self.validate_cue(cue)?;
        let knobs = QueryTuning::from_config(&self.config);
        let mut plans: Vec<ObservationPlan> = Vec::new();
        let mut best_near_miss: f64 = 0.0;
        for trace in self
            .traces
            .iter()
            .filter(|t| t.active && t.tier != TraceTier::Candidate)
        {
            let sim = cosine_similarity(cue, &trace.cue);
            let reliability = trace.reliability();
            if sim >= knobs.query_threshold {
                if reliability < knobs.min_reliability {
                    // Reliability-blocked: the direct unlock. Score = how
                    // close reliability sits to the floor.
                    let gap =
                        (knobs.min_reliability - reliability) / knobs.min_reliability.max(1e-9);
                    plans.push(ObservationPlan {
                        probe_cue: trace.cue.clone(),
                        target_trace: Some(trace.id),
                        kind: ObservationKind::Reinforce,
                        score: 1.0 - gap.clamp(0.0, 1.0),
                        reason: format!(
                            "trace {} matches (sim {sim:.3}) but reliability {reliability:.3} \
                             sits below the floor {:.3}; observing its cue again \
                             raises reliability through reinforcement",
                            trace.id, knobs.min_reliability
                        ),
                    });
                }
                // Eligible traces mean the query answers — no uncertainty
                // to reduce; eligible-and-reliable traces never appear here.
            } else if sim >= knobs.query_threshold - self.config.eig_margin {
                best_near_miss = best_near_miss.max(sim);
            }
        }
        if plans.is_empty() && best_near_miss > 0.0 {
            let gap = knobs.query_threshold - best_near_miss;
            let score = 1.0 - gap / self.config.eig_margin;
            if score >= self.config.eig_min_score {
                let norm: f64 = cue.iter().map(|v| v * v).sum::<f64>().sqrt();
                plans.push(ObservationPlan {
                    probe_cue: cue.iter().map(|v| v / norm).collect(),
                    target_trace: None,
                    kind: ObservationKind::Explore,
                    score,
                    reason: format!(
                        "no committed structure reaches the gate; nearest trace sits at \
                         sim {best_near_miss:.3} (gap {gap:.3}); probing this cue lets \
                         KPR verify whatever pattern answers"
                    ),
                });
            }
        }
        plans.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        plans.truncate(3);
        Ok(plans)
    }

    /// Total mass over active traces (conservation budget check).
    pub fn total_mass(&self) -> f64 {
        self.traces
            .iter()
            .filter(|t| t.active)
            .map(|t| t.mass)
            .sum()
    }

    fn trace_id(&self, index: usize) -> u64 {
        self.traces[index].id
    }

    pub fn trace_by_id(&self, id: u64) -> Option<&MemoryTrace> {
        self.traces.iter().find(|t| t.id == id)
    }

    /// Current state hash (traces + bonds + tick), the same digest the
    /// transition log carries. Public so auditors can see routing bias
    /// (pheromone rides in the bond line — D52's visibility law) without
    /// waiting for the next observe to record it.
    pub fn state_digest(&self) -> String {
        self.state_hash()
    }

    /// CRDT merge (#9): absorb another engine's memory into this one as
    /// a deterministic JOIN (commutative and idempotent up to content —
    /// convergence is CANONICAL, not ordinal, because trace ids are
    /// engine-local ordinals and are renumbered on the way in).
    ///
    /// Semantics, in pass order:
    /// 1. ANCESTOR JOIN — a shared id whose CUE matches on both sides is
    ///    the same memory diverged by replication: counters and evidence
    ///    join elementwise MAX (G-Counter), tier joins to the more
    ///    protected, active joins OR. A shared id whose cues DIFFER is an
    ///    independent-world id collision, not an ancestor: it is treated
    ///    as fresh content and renumbered (nothing is silently fused).
    /// 2. FRESH ABSORB — other-side traces not present here are
    ///    renumbered onto this engine's id space; exact-content
    ///    duplicates of traces already present collapse to the existing
    ///    id (this is what makes re-merge idempotent).
    /// 3. BONDS — endpoints remap through the same table; an existing
    ///    pair strengthens by pheromone MAX instead of duplicating
    ///    (OR-set), and self-loops formed by remapping drop out.
    ///
    /// Convergence ≠ truth: the merge manufactures no winners — evidence
    /// joins are maxima, so whatever each side honestly accumulated
    /// survives; conflicting values from independent sides both remain
    /// present (contested ground), decided later only by observation.
    /// Fail-closed on shape mismatch (cue dim / value count).
    pub fn merge(&mut self, other: &CognitiveEngine) -> Result<MergeReport, EngineError> {
        if self.config.cue_dim != other.config.cue_dim
            || self.config.value_count != other.config.value_count
        {
            return Err(EngineError::InvalidObservation(format!(
                "merge refused: shape mismatch (cue {} vs {}, values {} vs {})",
                self.config.cue_dim,
                other.config.cue_dim,
                self.config.value_count,
                other.config.value_count
            )));
        }
        let mut traces_joined = 0usize;
        let mut traces_absorbed = 0usize;
        let mut traces_deduplicated = 0usize;
        let mut bonds_added = 0usize;
        let tick = self.tick;

        // Pass 1: ancestor joins (shared id AND same cue = same memory).
        let shared: Vec<(usize, usize)> = other
            .traces
            .iter()
            .enumerate()
            .filter_map(|(other_index, trace)| {
                self.traces
                    .iter()
                    .position(|mine| mine.id == trace.id && mine.cue == trace.cue)
                    .map(|mine_index| (other_index, mine_index))
            })
            .collect();
        let mut remap: HashMap<u64, u64> = HashMap::new();
        for (other_index, mine_index) in &shared {
            let other_trace = &other.traces[*other_index];
            let mine = &mut self.traces[*mine_index];
            for (slot, value) in other_trace.evidence.iter().enumerate() {
                if *value > mine.evidence[slot] {
                    mine.evidence[slot] = *value;
                }
            }
            mine.support = mine.support.max(other_trace.support);
            mine.mass = mine.mass.max(other_trace.mass);
            mine.hits = mine.hits.max(other_trace.hits);
            mine.contradictions = mine.contradictions.max(other_trace.contradictions);
            mine.ttl = mine.ttl.max(other_trace.ttl);
            mine.stages = mine.stages.max(other_trace.stages);
            mine.novelty = mine.novelty.max(other_trace.novelty);
            mine.active = mine.active || other_trace.active;
            mine.tier = tier_join(mine.tier, other_trace.tier);
            mine.last_tick = mine.last_tick.max(other_trace.last_tick);
            remap.insert(other_trace.id, mine.id);
            traces_joined += 1;
        }

        // Pass 2: fresh absorb with exact-content dedup (idempotence).
        let content_of = |t: &MemoryTrace| {
            (
                t.cue.clone(),
                t.evidence.clone(),
                t.support,
                t.mass,
                t.hits,
                t.contradictions,
                t.tier,
                t.active,
                t.origin,
                t.novelty,
            )
        };
        let mut next_id = self.next_id;
        for other_trace in &other.traces {
            if remap.contains_key(&other_trace.id) {
                continue; // joined above
            }
            let content = content_of(other_trace);
            if let Some(existing) = self
                .traces
                .iter()
                .position(|mine| content_of(mine) == content)
            {
                remap.insert(other_trace.id, self.traces[existing].id);
                traces_deduplicated += 1;
                continue;
            }
            let new_id = next_id;
            next_id += 1;
            remap.insert(other_trace.id, new_id);
            let mut absorbed = other_trace.clone();
            absorbed.id = new_id;
            absorbed.parents = absorbed
                .parents
                .iter()
                .map(|p| remap.get(p).copied().unwrap_or(*p))
                .collect();
            absorbed.grain_members = absorbed
                .grain_members
                .iter()
                .map(|m| remap.get(m).copied().unwrap_or(*m))
                .collect();
            self.traces.push(absorbed);
            traces_absorbed += 1;
        }
        self.next_id = next_id;

        // Pass 3: bonds remap; OR-set dedupe with pheromone max.
        for bond in &other.bonds {
            let upstream = remap.get(&bond.upstream).copied().unwrap_or(bond.upstream);
            let downstream = remap
                .get(&bond.downstream)
                .copied()
                .unwrap_or(bond.downstream);
            if upstream == downstream {
                continue;
            }
            match self
                .bonds
                .iter_mut()
                .find(|b| b.upstream == upstream && b.downstream == downstream)
            {
                Some(existing) => existing.pheromone = existing.pheromone.max(bond.pheromone),
                None => {
                    self.bonds.push(Bond {
                        upstream,
                        downstream,
                        pheromone: bond.pheromone,
                    });
                    bonds_added += 1;
                }
            }
        }

        // Replicated counters join max; exploration joins to the COOLER
        // temperature (more settled wins).
        self.tick = self.tick.max(other.tick);
        self.forgotten = self.forgotten.max(other.forgotten);
        self.surprises = self.surprises.max(other.surprises);
        self.observations = self.observations.max(other.observations);
        self.explore_temperature = self.explore_temperature.min(other.explore_temperature);
        for (action, count) in &other.actions {
            let entry = self.actions.entry(action).or_insert(0);
            *entry = (*entry).max(*count);
        }
        let _ = tick;
        *self.actions.entry("merge").or_insert(0) += 1;
        let state_hash = self.state_hash();
        self.transitions.push(Transition {
            tick: self.tick,
            action: "merge",
            surprised: false,
            predicted: None,
            state_hash,
        });
        if self.transitions.len() > 64 {
            self.transitions.remove(0);
        }
        Ok(MergeReport {
            traces_absorbed,
            traces_joined,
            traces_deduplicated,
            bonds_added,
        })
    }

    /// Set the tier assigned to traces created by subsequent nucleations
    /// (record-mode ingestion of a constitution uses this).
    pub fn set_ingest_tier(&mut self, tier: TraceTier) {
        self.ingest_tier = tier;
    }

    /// Add a directed echo bond (upstream → downstream). Both endpoints
    /// must exist; self-bonds and duplicates are rejected.
    pub fn add_bond(&mut self, upstream: u64, downstream: u64) -> Result<(), EngineError> {
        if upstream == downstream {
            return Err(EngineError::InvalidBond("self-bond rejected".into()));
        }
        let exists = |id| self.traces.iter().any(|t| t.id == id);
        if !exists(upstream) {
            return Err(EngineError::InvalidBond(format!(
                "upstream {upstream} does not exist"
            )));
        }
        if !exists(downstream) {
            return Err(EngineError::InvalidBond(format!(
                "downstream {downstream} does not exist"
            )));
        }
        if self
            .bonds
            .iter()
            .any(|b| b.upstream == upstream && b.downstream == downstream)
        {
            return Err(EngineError::InvalidBond("duplicate bond".into()));
        }
        self.bonds.push(Bond {
            upstream,
            downstream,
            pheromone: 0.0,
        });
        Ok(())
    }

    /// Pheromone on the edge between two traces (both directions count —
    /// the route carries the trail whichever way it is walked).
    fn bond_pheromone(&self, a: u64, b: u64) -> f64 {
        self.bonds
            .iter()
            .filter(|bond| {
                (bond.upstream == a && bond.downstream == b)
                    || (bond.upstream == b && bond.downstream == a)
            })
            .map(|bond| bond.pheromone)
            .sum()
    }

    /// Upstream partners of a trace (consolidation targets).
    fn upstream_ids(&self, id: u64) -> Vec<u64> {
        self.bonds
            .iter()
            .filter(|b| b.downstream == id)
            .map(|b| b.upstream)
            .collect()
    }

    /// All bonded neighbors of a trace (expansion walks both directions).
    fn neighbor_ids(&self, id: u64) -> Vec<u64> {
        self.bonds
            .iter()
            .filter(|b| b.upstream == id || b.downstream == id)
            .map(|b| {
                if b.upstream == id {
                    b.downstream
                } else {
                    b.upstream
                }
            })
            .collect()
    }

    fn state_hash(&self) -> String {
        let lines: Vec<String> = self
            .traces
            .iter()
            .filter(|t| t.active)
            .map(MemoryTrace::state_line)
            .collect();
        // Pheromone rides in the bond line (D52's visibility law: routing
        // bias must appear in the state hash, never hide in it).
        let bond_lines: Vec<String> = self
            .bonds
            .iter()
            .map(|b| format!("{}>{}:{:.6}", b.upstream, b.downstream, b.pheromone))
            .collect();
        canonical_hash(
            "atom-cognitive-engine/state/v1",
            &[
                &self.tick.to_string(),
                &lines.join(";"),
                &bond_lines.join(";"),
            ],
        )
    }

    fn active_indices(&self) -> Vec<usize> {
        self.traces
            .iter()
            .enumerate()
            .filter(|(_, t)| t.active)
            .map(|(i, _)| i)
            .collect()
    }

    fn validate_cue(&self, cue: &[f64]) -> Result<(), EngineError> {
        if cue.len() != self.config.cue_dim {
            return Err(EngineError::InvalidQuery(format!(
                "cue dimension {} != {}",
                cue.len(),
                self.config.cue_dim
            )));
        }
        if !cue.iter().all(|v| v.is_finite() && v.abs() <= 8.0) {
            return Err(EngineError::InvalidQuery(
                "cue entries must be finite and within [-8, 8]".into(),
            ));
        }
        if cue.iter().all(|v| v.abs() < 1e-12) {
            return Err(EngineError::InvalidQuery("cue is all-zero".into()));
        }
        Ok(())
    }

    /// Run one observe cycle with the given (ordering-validated) stack.
    pub fn observe(
        &mut self,
        stack: &Stack,
        cue: &[f64],
        value: usize,
        salience: f64,
    ) -> Result<ObserveOutcome, EngineError> {
        self.observe_values(stack, cue, &[value], salience)
    }

    /// Observe with a CAUSAL ORIGIN (#4): the attribution id of the
    /// source making this observation (0 = unattributed, the default
    /// stream — byte-identical to plain observe). Two different explicit
    /// origins writing the same trace are CONCURRENT writers: the later
    /// one never dissolves the earlier one's evidence (arrival order is
    /// not causal order); its dissent joins at precision weight and both
    /// masses stay live. Same-origin revisions supersede as always.
    pub fn observe_with_origin(
        &mut self,
        stack: &Stack,
        cue: &[f64],
        values: &[usize],
        salience: f64,
        origin: u64,
    ) -> Result<ObserveOutcome, EngineError> {
        self.observe_impl(stack, cue, values, salience, origin)
    }

    /// Observe with multi-valued content (a memory encoding several
    /// behaviors at once). The first value drives prediction/revision; all
    /// values receive nucleation evidence. Used by record-mode corpus
    /// ingestion.
    pub fn observe_values(
        &mut self,
        stack: &Stack,
        cue: &[f64],
        values: &[usize],
        salience: f64,
    ) -> Result<ObserveOutcome, EngineError> {
        self.observe_impl(stack, cue, values, salience, 0)
    }

    fn observe_impl(
        &mut self,
        stack: &Stack,
        cue: &[f64],
        values: &[usize],
        salience: f64,
        origin: u64,
    ) -> Result<ObserveOutcome, EngineError> {
        self.validate_cue(cue).map_err(|e| match e {
            EngineError::InvalidQuery(what) => EngineError::InvalidObservation(what),
            other => other,
        })?;
        if values.is_empty() || values.len() > self.config.value_count {
            return Err(EngineError::InvalidObservation(format!(
                "value list length {} invalid",
                values.len()
            )));
        }
        let value = values[0];
        if !values.iter().all(|v| *v < self.config.value_count) {
            return Err(EngineError::InvalidObservation(format!(
                "value {value} out of range"
            )));
        }
        if !(0.25..=2.0).contains(&salience) {
            return Err(EngineError::InvalidObservation(format!(
                "salience {salience} out of range [0.25, 2.0]"
            )));
        }
        let mut ctx = ObserveCtx {
            cue: cue.to_vec(),
            value,
            values: values.to_vec(),
            salience,
            predicted: None,
            surprised: false,
            effective_salience: salience,
            sims: Vec::new(),
            bind_target: None,
            candidate_mode: false,
            origin,
        };
        let mut action: &'static str = "noop";
        let mut written_id: Option<u64> = None;
        for stage in stack.stages() {
            match stage.id {
                stacks::OBSERVE_VALIDATE => {}
                stacks::OBSERVE_PREDICT => self.step_predict(&mut ctx),
                stacks::OBSERVE_DECAY => self.step_decay(),
                stacks::OBSERVE_ATTEND => self.step_attend(&mut ctx),
                stacks::OBSERVE_GATE => self.step_gate(&mut ctx),
                stacks::OBSERVE_GATE_RECORD => {
                    // Record mode: always nucleate. Constitutions are
                    // documents, not experiences to compress.
                    ctx.bind_target = None;
                }
                stacks::OBSERVE_GATE_CANDIDATE => {
                    ctx.candidate_mode = true;
                    self.step_gate_candidate(&mut ctx);
                }
                stacks::OBSERVE_WRITE => {
                    let (write_action, write_id) = self.step_write(&mut ctx)?;
                    action = write_action;
                    written_id = write_id;
                }
                stacks::OBSERVE_PROMOTE => {
                    if let Some(promoted) = self.step_promote(written_id)? {
                        action = promoted;
                    }
                }
                stacks::OBSERVE_CONSERVE => self.step_conserve(),
                stacks::OBSERVE_VERIFY => {}
                other => {
                    return Err(EngineError::StageNotRunnable {
                        stage: other.into(),
                        cycle: "observe",
                    });
                }
            }
        }
        // Echo consolidation: observing a downstream memory pays a support
        // bonus to its bonded upstream partners (the kitchen is reinforced
        // when the bakery fires — no repetition required).
        if let Some(id) = written_id {
            let tick = self.tick;
            let bonus = self.config.echo_support_bonus;
            let upstreams = self.upstream_ids(id);
            for upstream in upstreams {
                if let Some(trace) = self
                    .traces
                    .iter_mut()
                    .find(|t| t.id == upstream && t.active)
                {
                    trace.support = (trace.support + bonus).min(1.0);
                    trace.last_tick = tick;
                }
            }
        }
        if ctx.surprised {
            self.surprises += 1;
        }
        self.observations += 1;
        *self.actions.entry(action).or_insert(0) += 1;
        // Formation cooling (D20: engine behavior, not a root — EXPLORE was
        // demoted; monotonic cooling stays as an invariant-tested mechanism).
        self.explore_temperature =
            (self.explore_temperature * self.config.cooling).max(self.config.explore_floor);
        let state_hash = self.state_hash();
        let outcome = ObserveOutcome {
            tick: self.tick,
            action,
            predicted: ctx.predicted,
            surprised: ctx.surprised,
            state_hash: state_hash.clone(),
        };
        self.transitions.push(Transition {
            tick: self.tick,
            action,
            surprised: ctx.surprised,
            predicted: ctx.predicted,
            state_hash,
        });
        if self.transitions.len() > 64 {
            self.transitions.remove(0);
        }
        self.tick += 1;
        Ok(outcome)
    }

    /// The perception history in emission order. Superseded entries are
    /// retained — perceptions are retractable, never silently overwritten
    /// (the revisability law from docs/E1-PERCEPTION.md).
    pub fn perceptions(&self) -> &[crate::perception::Perception] {
        &self.perceptions
    }

    /// Run one perception cycle (E1): form, gate, and emit a named,
    /// revisable hypothesis over a window of readings. Stage-order-driven
    /// like every other cycle; a stack without the perceive stage abstains
    /// (`perceive-stage-absent`) — that ablation is the load-bearing gate.
    ///
    /// Revisability: a newly emitted hypothesis that contradicts the
    /// active one supersedes it (the old entry is marked superseded and
    /// kept). A window that re-confirms the active hypothesis does not
    /// re-emit (stability rule). Judgment — deciding actions on
    /// perceptions — is deliberately outside this method.
    pub fn perceive(
        &mut self,
        stack: &Stack,
        readings: &[crate::perception::Reading],
    ) -> Result<crate::perception::PerceptionOutcome, EngineError> {
        let floor = self.config.perception_support_floor;
        let cap = self.config.perception_window_cap;
        let catalysis = self.config.perception_catalysis;
        let active: Option<(crate::perception::PerceptionLabel, u32)> = self
            .perceptions
            .iter()
            .rev()
            .find(|p| !p.superseded)
            .map(|p| (p.label, p.confirmations));
        let mut candidate: Option<crate::perception::PerceptionOutcome> = None;
        let mut window_ids: Vec<u64> = Vec::new();
        for stage in stack.stages() {
            match stage.id {
                stacks::PERCEIVE_VALIDATE => {
                    if readings.is_empty() {
                        return Ok(crate::perception::PerceptionOutcome::abstain(
                            "empty-window",
                        ));
                    }
                    if readings.len() > cap {
                        return Err(EngineError::InvalidObservation(format!(
                            "perception window of {} readings exceeds cap {cap}",
                            readings.len()
                        )));
                    }
                }
                stacks::PERCEIVE_ATTEND => {
                    // Attention over the window: the well-formed readings
                    // are the ones the matcher may weigh (see name_window).
                    window_ids = readings.iter().map(|r| r.id).collect();
                }
                stacks::PERCEIVE_PERCEIVE => {
                    // E1-chem: the catalyst relaxes the floor only for the
                    // ACTIVE label (an ongoing reaction); see
                    // name_window_catalyzed.
                    candidate = Some(crate::perception::name_window_catalyzed(
                        readings, floor, active, catalysis,
                    ));
                }
                stacks::PERCEIVE_GATE => {
                    // Emission eligibility (assert-or-abstain) is enforced
                    // by the abstention paths in name_window: below-floor,
                    // tied, or absent candidates never reach emission.
                }
                stacks::PERCEIVE_VERIFY => {
                    if let Some(perception) = candidate.as_ref().and_then(|o| o.perception.as_ref())
                    {
                        assert!(
                            perception
                                .evidence
                                .iter()
                                .all(|id| window_ids.contains(id)),
                            "verify: evidence ids must reference the window"
                        );
                    }
                }
                other => {
                    return Err(EngineError::StageNotRunnable {
                        stage: other.into(),
                        cycle: "perception",
                    });
                }
            }
        }
        let mut outcome = candidate
            .unwrap_or_else(|| crate::perception::PerceptionOutcome::abstain("perceive-stage-absent"));
        if let Some(mut perception) = outcome.perception.take() {
            if let Some(active) = self.perceptions.iter().rev().find(|p| !p.superseded) {
                if active.label == perception.label {
                    // Stability rule: re-confirmation returns the active
                    // hypothesis unchanged — no re-emission, no drift — and
                    // accrues one confirmation (the catalyst substrate).
                    let active_id = active.id;
                    let stored = self
                        .perceptions
                        .iter_mut()
                        .find(|p| p.id == active_id)
                        .expect("active perception exists");
                    stored.confirmations += 1;
                    outcome.perception = Some(stored.clone());
                    return Ok(outcome);
                }
                perception.supersedes = Some(active.id);
                let superseded_id = active.id;
                if let Some(previous) = self
                    .perceptions
                    .iter_mut()
                    .find(|p| p.id == superseded_id)
                {
                    previous.superseded = true;
                }
            }
            perception.id = self.next_id;
            self.next_id += 1;
            self.perceptions.push(perception.clone());
            outcome.perception = Some(perception);
        }
        Ok(outcome)
    }

    /// Run one query cycle with the given (ordering-validated) stack.
    /// `claim: None` is a deterministic abstention.
    pub fn query(&self, stack: &Stack, cue: &[f64]) -> Result<QueryOutcome, EngineError> {
        self.query_tuned(stack, cue, QueryTuning::from_config(&self.config))
    }

    /// Run one query cycle with per-query knob overrides (the persona
    /// governor's control surface). Tuning is clamped to safety floors
    /// before use; the engine's own config and state are untouched — the
    /// governor controls HOW the engine attends, never WHAT is true.
    pub fn query_tuned(
        &self,
        stack: &Stack,
        cue: &[f64],
        tuning: QueryTuning,
    ) -> Result<QueryOutcome, EngineError> {
        Ok(self.run_query(stack, cue, tuning)?.0)
    }

    /// A marking query (#14): the explicit stigmergic action. Runs the
    /// query exactly like `query_tuned`, THEN deposits pheromone on every
    /// edge the expansion walked (each bond capped at `pheromone_cap`).
    /// Plain queries never deposit — Stage-9 (queries are read-only)
    /// holds; marking is consented work. Returns the outcome plus how
    /// many edges were marked.
    pub fn query_marking(
        &mut self,
        stack: &Stack,
        cue: &[f64],
        tuning: QueryTuning,
    ) -> Result<MarkingOutcome, EngineError> {
        let (outcome, edges) = self.run_query(stack, cue, tuning)?;
        let rate = tuning.clamped().stigmergy_rate;
        let cap = self.config.pheromone_cap;
        let mut marked = 0usize;
        if rate > 0.0 {
            for (from, to) in &edges {
                for bond in &mut self.bonds {
                    if (bond.upstream == *from && bond.downstream == *to)
                        || (bond.upstream == *to && bond.downstream == *from)
                    {
                        bond.pheromone = (bond.pheromone + rate).min(cap);
                        marked += 1;
                    }
                }
            }
            if marked > 0 {
                *self.actions.entry("mark").or_insert(0) += 1;
            }
        }
        Ok(MarkingOutcome {
            outcome,
            edges_marked: marked,
        })
    }

    fn run_query(
        &self,
        stack: &Stack,
        cue: &[f64],
        tuning: QueryTuning,
    ) -> Result<(QueryOutcome, Vec<(u64, u64)>), EngineError> {
        self.validate_cue(cue)?;
        let mut ctx = QueryCtx {
            cue: cue.to_vec(),
            logits: Vec::new(),
            eligible: Vec::new(),
            expansions: Vec::new(),
            gate_ran: false,
            mixture: Vec::new(),
            member_weights: Vec::new(),
            knobs: tuning.clamped(),
            distinct_supporters: 0,
            quorum_met: true,
        };
        for stage in stack.stages() {
            match stage.id {
                stacks::QUERY_VALIDATE => {}
                stacks::QUERY_ATTEND => self.step_query_attend(&mut ctx),
                stacks::QUERY_GATE => {
                    self.step_query_gate(&mut ctx);
                    ctx.gate_ran = true;
                }
                stacks::QUERY_EXPAND => self.step_query_expand(&mut ctx),
                stacks::QUERY_RETRIEVE => self.step_query_retrieve(&mut ctx),
                stacks::QUERY_ASSERT => {}
                stacks::QUERY_VERIFY => {}
                other => {
                    return Err(EngineError::StageNotRunnable {
                        stage: other.into(),
                        cycle: "query",
                    });
                }
            }
        }
        let eligible_count = ctx.eligible.len();
        let claim = if ctx.mixture.is_empty() {
            None
        } else {
            let mut best = 0usize;
            let mut best_weight = f64::MIN;
            for (index, weight) in ctx.mixture.iter().enumerate() {
                if *weight > best_weight {
                    best_weight = *weight;
                    best = index;
                }
            }
            Some((best, best_weight))
        };
        let cue_line = ctx
            .cue
            .iter()
            .map(|v| format!("{v:.6}"))
            .collect::<Vec<_>>()
            .join(",");
        let claim_line = claim
            .map(|(v, w)| format!("{v}:{w:.6}"))
            .unwrap_or_else(|| "abstain".into());
        let activation_line = ctx
            .member_weights
            .iter()
            .map(|(id, w)| format!("{id}:{w:.4}"))
            .collect::<Vec<_>>()
            .join(",");
        let response_hash = canonical_hash(
            "atom-cognitive-engine/query/v1",
            &[
                &self.tick.to_string(),
                &cue_line,
                &claim_line,
                &activation_line,
            ],
        );
        let mut activations = ctx.member_weights.clone();
        activations.truncate(8);
        let fluency = fluency_report(
            eligible_count,
            &ctx.mixture,
            activations
                .first()
                .map(|(_, weight)| *weight)
                .unwrap_or(0.0),
        );
        // Pareto preservation (#6): the non-dominated contender set, pure
        // metadata (excluded from the response hash above — arming the
        // front may never change the recorded answer).
        let front = if ctx.knobs.pareto_front {
            let members: Vec<FrontMember> = ctx
                .eligible
                .iter()
                .map(|index| {
                    let trace = &self.traces[*index];
                    FrontMember {
                        trace_id: trace.id,
                        value: trace.value().unwrap_or(0),
                        reliability: trace.reliability(),
                        support: trace.support,
                        novelty: trace.novelty,
                    }
                })
                .collect();
            pareto_front(&members)
        } else {
            Vec::new()
        };
        let walked_edges: Vec<(u64, u64)> = ctx
            .expansions
            .iter()
            .map(|(index, _hop, _pheromone, parent)| (*parent, self.trace_id(*index)))
            .collect();
        Ok((
            QueryOutcome {
                claim,
                eligible_count,
                activations,
                activated_all: ctx.member_weights.iter().map(|(id, _)| *id).collect(),
                fluency,
                front,
                quorum: QuorumReport {
                    distinct_supporters: ctx.distinct_supporters,
                    quorum_met: ctx.quorum_met,
                },
                response_hash,
            },
            walked_edges,
        ))
    }

    /// Advance time: run the idle cycle (decay, conserve, verify) `steps`
    /// times with the given idle stack.
    pub fn idle(&mut self, stack: &Stack, steps: u64) -> Result<(), EngineError> {
        for _ in 0..steps {
            for stage in stack.stages() {
                match stage.id {
                    "idle.validate" => {}
                    "idle.decay" => self.step_decay(),
                    "idle.conserve" => self.step_conserve(),
                    "idle.verify" => {}
                    other => {
                        return Err(EngineError::StageNotRunnable {
                            stage: other.into(),
                            cycle: "idle",
                        });
                    }
                }
            }
            self.tick += 1;
        }
        Ok(())
    }

    // ---- observe steps -------------------------------------------------

    fn step_predict(&self, ctx: &mut ObserveCtx) {
        let mut best: Option<(usize, f64)> = None;
        for index in self.active_indices() {
            let trace = &self.traces[index];
            let sim = cosine_similarity(&ctx.cue, &trace.cue);
            if sim >= self.config.query_threshold
                && trace.reliability() >= self.config.min_reliability
                && best.map(|(_, s)| sim > s).unwrap_or(true)
            {
                best = Some((index, sim));
            }
        }
        if let Some((index, _)) = best {
            let predicted = self.traces[index].value();
            ctx.predicted = predicted;
            if predicted != Some(ctx.value) {
                ctx.surprised = true;
                ctx.effective_salience = (ctx.salience * self.config.surprise_boost).min(2.5);
            }
        }
    }

    fn step_decay(&mut self) {
        let decay_support = self.config.decay_support_rate;
        let decay_mass = self.config.decay_mass_rate;
        // Stigmergic evaporation (#14): trails fade with time on the same
        // clock as memory decay — no route becomes a permanent rut unless
        // traffic keeps renewing it. Zero stays exactly zero, so the
        // mechanism is invisible until armed.
        let pheromone_decay = self.config.pheromone_decay;
        for bond in &mut self.bonds {
            bond.pheromone *= pheromone_decay;
        }
        // Bet hedging (#10): the attention tail — the lowest-standing
        // fifth of the COMMITTED memory — decays at (1 − hedge_rate)
        // speed and keeps its ttl. Current losers persist as insurance
        // so a regime flip finds the alternative still alive. Ties break
        // by id (deterministic); candidates never hedge (KPR
        // dissipation is law); expiry floors still apply — hedging
        // delays decay, it never cancels the hard limits.
        let hedge_rate = self.config.hedge_rate;
        let hedge_ids: Vec<u64> = if hedge_rate > 0.0 {
            let mut committed: Vec<(f64, u64)> = self
                .traces
                .iter()
                .filter(|t| t.active && t.tier != TraceTier::Candidate)
                .map(|t| (t.reliability(), t.id))
                .collect();
            committed.sort_by(|a, b| {
                a.0.partial_cmp(&b.0)
                    .unwrap_or(core::cmp::Ordering::Equal)
                    .then(a.1.cmp(&b.1))
            });
            let quintile = (committed.len() / 5).max(1);
            committed.iter().take(quintile).map(|(_, id)| *id).collect()
        } else {
            Vec::new()
        };
        for index in self.active_indices() {
            let hedged = hedge_rate > 0.0 && hedge_ids.contains(&self.traces[index].id);
            let hedge_factor = if hedged { 1.0 - hedge_rate } else { 1.0 };
            let trace = &mut self.traces[index];
            let protection = (trace.hits.max(1) as f64).sqrt();
            // KPR dissipation: uncommitted candidates decay three times
            // faster — transient matches must persist to survive.
            let candidate_boost = if trace.tier == TraceTier::Candidate {
                3.0
            } else {
                1.0
            };
            trace.support *= 1.0 - (decay_support * candidate_boost * hedge_factor) / protection;
            trace.mass *=
                1.0 - (decay_mass * candidate_boost * hedge_factor) / trace.hits.max(1) as f64;
            if !hedged {
                trace.ttl = trace.ttl.saturating_sub(1);
            }
        }
        // Forget checks run after the decay pass, over all active traces.
        for index in self.active_indices() {
            let trace = &self.traces[index];
            let expired = (trace.ttl == 0 && trace.support < self.config.expire_ttl_support)
                || trace.support < self.config.expire_support
                || trace.mass < self.config.expire_support;
            if expired {
                self.traces[index].active = false;
                self.forgotten += 1;
            }
        }
    }

    fn step_attend(&self, ctx: &mut ObserveCtx) {
        ctx.sims = self
            .active_indices()
            .into_iter()
            .map(|index| {
                let sim = cosine_similarity(&ctx.cue, &self.traces[index].cue);
                (index, sim)
            })
            .collect();
    }

    /// KPR candidate gate: like the match gate, but (a) candidates are
    /// bindable targets (so persistent signals accumulate stages), and
    /// (b) below-threshold cues nucleate into the Candidate tier instead
    /// of the ingest tier.
    fn step_gate_candidate(&mut self, ctx: &mut ObserveCtx) {
        if ctx.sims.is_empty() {
            ctx.bind_target = None;
            return;
        }
        let mut best_score = f64::MIN;
        for (_, sim) in &ctx.sims {
            if *sim > best_score {
                best_score = *sim;
            }
        }
        let ties: Vec<usize> = ctx
            .sims
            .iter()
            .filter(|(_, sim)| (*sim - best_score).abs() < 1e-12)
            .map(|(index, _)| *index)
            .collect();
        let chosen = if ties.len() > 1 {
            let unit = (seeded_unit(&mut self.rng_state) + 1.0) * 0.5;
            let pick = ((unit * ties.len() as f64) as usize).min(ties.len() - 1);
            ties[pick]
        } else {
            ties[0]
        };
        // Candidates bind above a slightly relaxed threshold (they are
        // accumulating toward commitment, not asserting); committed
        // traces bind at the normal match threshold.
        let target_tier_is_candidate = self
            .traces
            .get(chosen)
            .is_some_and(|trace| trace.tier == TraceTier::Candidate);
        let threshold = if target_tier_is_candidate {
            self.config.match_threshold - 0.05
        } else {
            self.config.match_threshold
        };
        ctx.bind_target = if best_score >= threshold {
            Some(chosen)
        } else {
            None
        };
    }

    /// KPR promotion (selective retention + the reset): after the write,
    /// the touched trace — if Candidate — either accumulates a stage
    /// (distinct-tick only), gets RESET on contradiction, or is promoted
    /// once it has survived enough contradiction-free stages.
    fn step_promote(
        &mut self,
        written_id: Option<u64>,
    ) -> Result<Option<&'static str>, EngineError> {
        let Some(id) = written_id else {
            return Ok(None);
        };
        let Some(index) = self.traces.iter().position(|t| t.id == id) else {
            return Ok(None);
        };
        if self.traces[index].tier != TraceTier::Candidate {
            return Ok(None);
        }
        let tick = self.tick;
        let trace = &mut self.traces[index];
        // The KPR reset: a contradicted candidate loses its accumulated
        // stages — failure dissipates the progress toward commitment.
        if trace.contradictions > 0 && trace.last_tick == tick {
            trace.stages = 0;
            trace.contradictions = 0; // consumed by the reset
            return Ok(Some("dissipate"));
        }
        // Stage advance: only on a distinct tick (same-tick re-stagings
        // are not independent verification).
        if trace.last_tick_staged != tick {
            trace.stages += 1;
            trace.last_tick_staged = tick;
        }
        if trace.stages >= self.config.kpr_min_stages {
            trace.tier = self.ingest_tier;
            return Ok(Some("promote"));
        }
        Ok(None)
    }

    fn step_gate(&mut self, ctx: &mut ObserveCtx) {
        if ctx.sims.is_empty() {
            ctx.bind_target = None;
            return;
        }
        let mut best_score = f64::MIN;
        for (_, sim) in &ctx.sims {
            if *sim > best_score {
                best_score = *sim;
            }
        }
        // Seeded, cooling tie-break among exact ties on the best score
        // (D20: a mechanism, not a root). Only genuine ties flip; everything
        // else is order-free.
        let ties: Vec<usize> = ctx
            .sims
            .iter()
            .filter(|(_, sim)| (*sim - best_score).abs() < 1e-12)
            .map(|(index, _)| *index)
            .collect();
        let chosen = if ties.len() > 1 {
            let unit = (seeded_unit(&mut self.rng_state) + 1.0) * 0.5; // [0, 1)
            let pick = ((unit * ties.len() as f64) as usize).min(ties.len() - 1);
            ties[pick]
        } else {
            ties[0]
        };
        ctx.bind_target = if best_score >= self.config.match_threshold {
            Some(chosen)
        } else {
            None
        };
    }

    fn step_write(
        &mut self,
        ctx: &mut ObserveCtx,
    ) -> Result<(&'static str, Option<u64>), EngineError> {
        let tick = self.tick;
        let evidence_rate = self.config.evidence_rate;
        let salience = ctx.effective_salience;
        match ctx.bind_target {
            Some(index) => {
                let value = ctx.value;
                let core = self.traces[index].tier == TraceTier::Core;
                let trace = &mut self.traces[index];
                if trace.value() == Some(value) {
                    trace.hits += 1;
                    // Agreement is agreement regardless of origin — the
                    // trace's attribution follows its latest confirming
                    // writer (sequential supersession).
                    if ctx.origin != 0 {
                        trace.origin = ctx.origin;
                    }
                    trace.mass = (trace.mass + self.config.reinforcement_mass * salience).min(2.0);
                    trace.support =
                        (trace.support + self.config.reinforcement_support * salience).min(1.0);
                    trace.ttl = self.config.initial_ttl + self.config.ttl_per_hit * trace.hits;
                    trace.evidence[value] = (trace.evidence[value] + evidence_rate * salience)
                        .min(self.config.evidence_cap);
                    // HABITUATION (W15): repeated confirmation decays
                    // novelty — attention stops saturating on constants.
                    trace.novelty *= 1.0 - self.config.habituation_rate;
                    trace.last_tick = tick;
                    Ok(("reinforce", Some(trace.id)))
                } else {
                    // REVISE: dissolve first, then replace (receipt C.1) —
                    // unless the trace is Core identity substrate: the
                    // contradiction is recorded and the new evidence joins,
                    // but the standing identity is not dissolved.
                    trace.contradictions += 1;
                    // Precision-weighted prediction error (#7): the
                    // strength of a revision scales with the SOURCE's
                    // estimated precision — its (surprise-boosted)
                    // salience relative to the standing belief's
                    // reliability. A weak ping against a strong belief
                    // barely dissolves and casts only a small dissent
                    // vote; a strong contradiction keeps nearly full
                    // force. gain 0 → weight 1 → exactly the historical
                    // revision (byte-identical).
                    let standing = trace.reliability().max(1e-6);
                    let weight = if self.config.precision_gain > 0.0 {
                        let strength = (ctx.effective_salience / 2.0).max(1e-6);
                        strength / (strength + self.config.precision_gain * standing)
                    } else {
                        1.0
                    };
                    let support_cost =
                        1.0 - weight * (1.0 - self.config.contradiction_support_factor);
                    let dissolve = self.config.contradiction_evidence_factor.powf(weight);
                    let dissent = 1.0 - self.config.precision_gain * (1.0 - weight);
                    // CAUSAL ORDERING (#4): two different explicit origins
                    // are CONCURRENT writers — arrival order is not causal
                    // order, so the later one never dissolves the standing
                    // belief. Its dissent joins at precision weight; both
                    // masses stay live. Same-origin (or unattributed)
                    // revisions supersede exactly as before.
                    let concurrent =
                        ctx.origin != 0 && trace.origin != 0 && trace.origin != ctx.origin;
                    if !core && !concurrent {
                        trace.support *= support_cost;
                        for evidence in trace.evidence.iter_mut() {
                            *evidence *= dissolve;
                        }
                    }
                    trace.evidence[value] = (trace.evidence[value]
                        + evidence_rate * 1.5 * salience * dissent)
                        .min(self.config.evidence_cap);
                    trace.hits += 1;
                    trace.ttl = self.config.initial_ttl + self.config.ttl_per_hit * trace.hits;
                    // SENSITIZATION (W15): surprise restores novelty to
                    // full — a contradicted trace REGAINS attention.
                    trace.novelty = 1.0;
                    trace.last_tick = tick;
                    // RENORMALIZATION ROLLBACK (#5): contradiction at the
                    // coarse scale splits the grain — the fine structure
                    // it absorbed re-activates with its frozen evidence.
                    let grain_id = trace.id;
                    let has_grain = !trace.grain_members.is_empty();
                    if has_grain {
                        self.split_grain(index);
                    }
                    Ok(("revise", Some(grain_id)))
                }
            }
            None => {
                // Nucleate; retire the weakest first when at capacity.
                // Core traces are never eviction candidates.
                if self.active_traces().len() >= self.config.capacity {
                    let mut weakest: Option<usize> = None;
                    let mut weakest_score = f64::MAX;
                    for index in self.active_indices() {
                        let trace = &self.traces[index];
                        if trace.tier == TraceTier::Core {
                            continue;
                        }
                        let score = trace.reliability() + 0.02 * trace.ttl as f64;
                        if score < weakest_score {
                            weakest_score = score;
                            weakest = Some(index);
                        }
                    }
                    match weakest {
                        Some(index) => {
                            self.traces[index].active = false;
                            self.forgotten += 1;
                        }
                        None => return Err(EngineError::CapacityExhausted),
                    }
                }
                let norm: f64 = ctx.cue.iter().map(|v| v * v).sum::<f64>().sqrt();
                let mut evidence = vec![0.0; self.config.value_count];
                for value in &ctx.values {
                    evidence[*value] = (evidence_rate * salience).min(self.config.evidence_cap);
                }
                let tier = if ctx.candidate_mode {
                    TraceTier::Candidate
                } else {
                    self.ingest_tier
                };
                let id = self.next_id;
                self.traces.push(MemoryTrace {
                    id,
                    cue: ctx.cue.iter().map(|v| v / norm).collect(),
                    evidence,
                    mass: self.config.initial_mass * salience,
                    support: self.config.initial_support * salience,
                    ttl: self.config.initial_ttl,
                    hits: 1,
                    stages: 1,
                    last_tick_staged: tick,
                    contradictions: 0,
                    created_tick: tick,
                    last_tick: tick,
                    active: true,
                    tier,
                    parents: Vec::new(),
                    grain_members: Vec::new(),
                    novelty: 1.0,
                    origin: ctx.origin,
                });
                self.next_id += 1;
                Ok(("nucleate", Some(id)))
            }
        }
    }

    fn step_conserve(&mut self) {
        let total = self.total_mass();
        let budget = self.config.conservation_budget;
        if total > budget && total > f64::EPSILON {
            let scale = budget / total;
            for index in self.active_indices() {
                self.traces[index].mass *= scale;
            }
        }
    }

    // ---- query steps ---------------------------------------------------

    fn step_query_attend(&self, ctx: &mut QueryCtx) {
        let temperature = ctx.knobs.attention_temperature;
        let novelty_gain = self.config.novelty_gain;
        ctx.logits = self
            .active_indices()
            .into_iter()
            .map(|index| {
                let trace = &self.traces[index];
                let sim = cosine_similarity(&ctx.cue, &trace.cue);
                let reliability = trace.reliability().max(1e-6);
                // W15: novelty is READ here, never written — queries stay
                // read-only. With gain 0 (the default) the term vanishes.
                (
                    index,
                    temperature * (sim + reliability.ln().max(-4.0)) + novelty_gain * trace.novelty,
                )
            })
            .collect();
    }

    fn step_query_gate(&self, ctx: &mut QueryCtx) {
        ctx.eligible = self
            .active_indices()
            .into_iter()
            .filter(|index| {
                let trace = &self.traces[*index];
                // Uncommitted candidates are query-invisible: staged
                // evidence never serves an assertion (KPR law).
                trace.tier != TraceTier::Candidate
                    && cosine_similarity(&ctx.cue, &trace.cue) >= ctx.knobs.query_threshold
                    && trace.reliability() >= ctx.knobs.min_reliability
            })
            .collect();
    }

    /// Bond expansion: BFS from eligible traces along echo bonds up to the
    /// configured depth. Expansion is admission, not license — expanded
    /// members must still clear the reliability floor (O3).
    fn step_query_expand(&self, ctx: &mut QueryCtx) {
        let depth = ctx.knobs.bond_expansion_depth;
        if depth == 0 || ctx.eligible.is_empty() {
            return;
        }
        // Quorum sensing (#15): expensive deliberation runs only on
        // DISTRIBUTED support. Distinct supporters cluster the eligible
        // set at cue similarity 0.90 — correlated copies count ONCE, so
        // correlated worlds cannot fake a quorum. Below the bar the
        // expansion is skipped (the fast, un-deliberated path).
        if ctx.knobs.quorum_min > 0 {
            let mut clusters: Vec<usize> = Vec::new();
            for index in &ctx.eligible {
                let cue = &self.traces[*index].cue;
                let correlated = clusters.iter().any(|representative| {
                    cosine_similarity(cue, &self.traces[*representative].cue) >= 0.90
                });
                if !correlated {
                    clusters.push(*index);
                }
            }
            ctx.distinct_supporters = clusters.len();
            ctx.quorum_met = clusters.len() >= ctx.knobs.quorum_min;
            if !ctx.quorum_met {
                return; // un-earned deliberation: no expansion
            }
        }
        // Satisficing (W18): when the eligible-only retrieval already has
        // a wide enough margin, the answer is settled — skip the BFS and
        // return the fast path. Good enough is earned; effort below this
        // bar is still spent in full.
        if ctx.knobs.satisficing_margin > 0.0 {
            let logits: Vec<f64> = ctx
                .eligible
                .iter()
                .map(|index| {
                    ctx.logits
                        .iter()
                        .find(|(i, _)| i == index)
                        .map(|(_, l)| *l)
                        .unwrap_or(0.0)
                })
                .collect();
            let weights = softmax(&logits);
            let mut mixture = vec![0.0; self.config.value_count];
            for (position, index) in ctx.eligible.iter().enumerate() {
                let distribution = self.traces[*index].value_distribution();
                for (value, weight) in distribution.iter().enumerate() {
                    mixture[value] += weights[position] * weight;
                }
            }
            let mut sorted = mixture;
            sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(core::cmp::Ordering::Equal));
            let margin = if sorted.len() >= 2 {
                sorted[0] - sorted[1]
            } else {
                sorted.first().copied().unwrap_or(0.0)
            };
            if margin >= ctx.knobs.satisficing_margin {
                return; // the fast path: no expansion, no extra members
            }
        }
        let mut frontier: Vec<(u64, usize)> = ctx
            .eligible
            .iter()
            .map(|index| (self.trace_id(*index), 0usize))
            .collect();
        let mut visited: Vec<u64> = frontier.iter().map(|(id, _)| *id).collect();
        // Walked edges: (member index, hop, edge pheromone at read time,
        // parent id) — the deposition record for marking queries (#14).
        let mut expansions: Vec<(usize, usize, f64, u64)> = Vec::new();
        while let Some((id, hop)) = frontier.pop() {
            if hop >= depth {
                continue;
            }
            for neighbor in self.neighbor_ids(id) {
                if visited.contains(&neighbor) {
                    continue;
                }
                visited.push(neighbor);
                if let Some(index) = self
                    .traces
                    .iter()
                    .position(|t| t.id == neighbor && t.active)
                {
                    let trace = &self.traces[index];
                    if trace.reliability() >= ctx.knobs.min_reliability {
                        let pheromone = self.bond_pheromone(id, neighbor);
                        expansions.push((index, hop + 1, pheromone, id));
                        frontier.push((neighbor, hop + 1));
                    }
                }
            }
        }
        ctx.expansions = expansions;
    }

    /// Mixture over eligible traces plus bond-expanded members. When the
    /// gate stage did NOT run (the deliberately reachable O7 failure mode),
    /// the mixture falls back to every active trace — attention without
    /// gating, the thing the law exists to prevent. Expanded members carry
    /// an echo logit bonus that decays with hop depth.
    fn step_query_retrieve(&self, ctx: &mut QueryCtx) {
        let mut members: Vec<(usize, f64)> = if ctx.gate_ran {
            ctx.eligible.iter().map(|index| (*index, 0.0)).collect()
        } else {
            self.active_indices()
                .into_iter()
                .map(|index| (index, 0.0))
                .collect()
        };
        for (index, hop, pheromone, _parent) in &ctx.expansions {
            if !members.iter().any(|(existing, _)| existing == index) {
                // Routing bonus: the echo term decays with hop depth, and
                // the stigmergic term reads the trail (routing-only —
                // pheromone never touches evidence or reliability).
                members.push((
                    *index,
                    self.config.echo_activation_bonus / *hop as f64
                        + ctx.knobs.stigmergy_gain * pheromone,
                ));
            }
        }
        if members.is_empty() {
            ctx.mixture = Vec::new();
            ctx.member_weights = Vec::new();
            return;
        }
        let logits: Vec<f64> = members
            .iter()
            .map(|(index, bonus)| {
                let base = ctx
                    .logits
                    .iter()
                    .find(|(i, _)| i == index)
                    .map(|(_, l)| *l)
                    .unwrap_or(0.0);
                base + *bonus
            })
            .collect();
        let selected = dedup_correlated(
            softmax(&logits),
            &members,
            &self.traces,
            ctx.knobs.dedup_threshold,
        );
        // Lateral inhibition (#13): the WINNER softly suppresses its
        // near-neighbors — each non-top member's weight is multiplied by
        // (1 − g·sim²) against the top member's cue (quadratic: distinct
        // voices keep their weight, the redundant chorus around the
        // winner fades) — then the mixture re-normalizes so share flows
        // to the distinct contributors. Suppression never changes WHO
        // the winner is; it changes how much the chorus echoes.
        let weights = if ctx.knobs.lateral_inhibition > 0.0 && selected.len() > 1 {
            let mut top = 0usize;
            for (index, weight) in selected.iter().enumerate() {
                if *weight > selected[top] || (*weight == selected[top] && index < top) {
                    top = index;
                }
            }
            let top_cue = self.traces[members[top].0].cue.clone();
            let mut inhibited: Vec<f64> = Vec::with_capacity(selected.len());
            for (index, weight) in selected.iter().enumerate() {
                if index == top || *weight <= 0.0 {
                    inhibited.push(*weight);
                    continue;
                }
                let sim = cosine_similarity(&top_cue, &self.traces[members[index].0].cue).max(0.0);
                inhibited.push(*weight * (1.0 - ctx.knobs.lateral_inhibition * sim * sim));
            }
            let total: f64 = inhibited.iter().sum();
            if total > f64::EPSILON {
                for weight in inhibited.iter_mut() {
                    *weight /= total;
                }
            }
            inhibited
        } else {
            selected
        };
        let weights = robustify_weights(weights, &ctx.knobs);
        let mut mixture = vec![0.0; self.config.value_count];
        for (position, index) in members.iter().map(|(index, _)| index).enumerate() {
            let distribution = self.traces[*index].value_distribution();
            for (value, weight) in distribution.iter().enumerate() {
                mixture[value] += weights[position] * weight;
            }
        }
        // Rank all activations; the outcome carries the top 8 for citation
        // and the full ranked set for reachability checks.
        let mut ranked: Vec<(u64, f64)> = members
            .iter()
            .map(|(index, _)| self.trace_id(*index))
            .zip(weights.iter().cloned())
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));
        ctx.member_weights = ranked;
        ctx.mixture = mixture;
    }
}

/// Robust aggregation (W14): transform softmax weights so no single trace
/// can dominate the mixture. Two operations, both deterministic:
///
/// 1. TRIM: the `robust_trim` lowest-weighted members are zeroed
///    (noise-floor removal; at least one member always survives).
/// 2. CAP: no member may exceed `robust_weight_cap` of the total. Excess
///    is redistributed proportionally to the uncapped members
///    (water-filling, bounded passes). A confident-but-wrong trace can
///    therefore be outvoted by a coalition — the breakdown bound — but a
///    genuine majority still wins.
///
/// With cap = 1.0 and trim = 0 (the defaults) the weights pass through
/// unchanged: validated surfaces run exactly as before.
/// The metacognitive read (W16): classify the SHAPE of a retrieval.
/// States, checked in order: Blank (nothing eligible) → Contested (the
/// mixture's top two values sit within `CONTESTED_MARGIN` of each
/// other) → Sparse (at most `SPARSE_ELIGIBLE` supporting traces) →
/// Fluent (everything else). Deterministic, read-only, and deliberately
/// independent of truth: a confident-wrong trace still reads Fluent —
/// the gate that proves this is the honesty condition for the whole
/// primitive.
fn fluency_report(eligible_count: usize, mixture: &[f64], top_weight: f64) -> FluencyReport {
    const CONTESTED_MARGIN: f64 = 0.15;
    const SPARSE_ELIGIBLE: usize = 2;
    let state = if eligible_count == 0 {
        FluencyState::Blank
    } else {
        let mut sorted = mixture.to_vec();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(core::cmp::Ordering::Equal));
        let margin = if sorted.len() >= 2 {
            sorted[0] - sorted[1]
        } else {
            sorted.first().copied().unwrap_or(0.0)
        };
        if margin < CONTESTED_MARGIN {
            FluencyState::Contested
        } else if eligible_count <= SPARSE_ELIGIBLE {
            FluencyState::Sparse
        } else {
            FluencyState::Fluent
        }
    };
    let margin = {
        let mut sorted = mixture.to_vec();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(core::cmp::Ordering::Equal));
        if sorted.len() >= 2 {
            sorted[0] - sorted[1]
        } else {
            sorted.first().copied().unwrap_or(0.0)
        }
    };
    FluencyReport {
        state,
        eligible_count,
        top_weight,
        margin,
    }
}

/// Pareto preservation (#6): keep every non-dominated member over the
/// objectives (reliability, support, novelty). A dominates B when A is
/// ≥ B on every axis and strictly > on at least one; the front is every
/// member no other member dominates. Deterministic order: reliability
/// descending, ties by trace id ascending. Bounded by the eligible
/// count. O(n²) over the contender set, deliberately simple and exact.
fn pareto_front(members: &[FrontMember]) -> Vec<FrontMember> {
    let mut front: Vec<FrontMember> = Vec::new();
    for candidate in members {
        let dominated = members.iter().any(|other| {
            other.trace_id != candidate.trace_id
                && other.reliability >= candidate.reliability
                && other.support >= candidate.support
                && other.novelty >= candidate.novelty
                && (other.reliability > candidate.reliability
                    || other.support > candidate.support
                    || other.novelty > candidate.novelty)
        });
        if !dominated {
            front.push(candidate.clone());
        }
    }
    front.sort_by(|a, b| {
        b.reliability
            .partial_cmp(&a.reliability)
            .unwrap_or(core::cmp::Ordering::Equal)
            .then(a.trace_id.cmp(&b.trace_id))
    });
    front
}

/// Occam/dedup selection (W17): correlated near-copies collapse to their
/// strongest member. Members are visited in weight-rank order; any later
/// member whose CUE similarity to an already-kept representative meets
/// the threshold has its weight DROPPED (not merged — correlated
/// evidence is not independent evidence, so the copy adds nothing).
/// Survivors are renormalized. With threshold > 1.0 (the default) the
/// weights pass through unchanged.
fn dedup_correlated(
    mut weights: Vec<f64>,
    members: &[(usize, f64)],
    traces: &[MemoryTrace],
    threshold: f64,
) -> Vec<f64> {
    if threshold > 1.0 || weights.len() < 2 {
        return weights;
    }
    // Visit in descending weight order (ties broken by position —
    // deterministic).
    let mut order: Vec<usize> = (0..weights.len()).collect();
    order.sort_by(|&a, &b| {
        weights[b]
            .partial_cmp(&weights[a])
            .unwrap_or(core::cmp::Ordering::Equal)
    });
    let mut kept: Vec<usize> = Vec::new();
    for position in order {
        let trace_index = members[position].0;
        let redundant = kept.iter().any(|&kept_position| {
            let kept_index = members[kept_position].0;
            cosine_similarity(&traces[trace_index].cue, &traces[kept_index].cue) >= threshold
        });
        if redundant {
            weights[position] = 0.0;
        } else {
            kept.push(position);
        }
    }
    // Renormalize the survivors.
    let total: f64 = weights.iter().sum();
    if total > f64::EPSILON {
        for weight in weights.iter_mut() {
            *weight /= total;
        }
    }
    weights
}

fn robustify_weights(mut weights: Vec<f64>, knobs: &QueryTuning) -> Vec<f64> {
    let off = knobs.robust_weight_cap >= 1.0 && knobs.robust_trim == 0;
    if off || weights.len() < 2 {
        return weights;
    }
    // Trim the lowest `trim` weights to zero (keep at least one member).
    if knobs.robust_trim > 0 {
        let mut order: Vec<usize> = (0..weights.len()).collect();
        order.sort_by(|&a, &b| {
            weights[a]
                .partial_cmp(&weights[b])
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        let removable = knobs.robust_trim.min(weights.len().saturating_sub(1));
        for &index in &order[..removable] {
            weights[index] = 0.0;
        }
    }
    // Cap with water-filling redistribution, bounded passes.
    if knobs.robust_weight_cap < 1.0 {
        for _ in 0..weights.len() {
            let total: f64 = weights.iter().sum();
            if total <= f64::EPSILON {
                break;
            }
            let limit = knobs.robust_weight_cap * total;
            let mut excess = 0.0;
            let mut free_total = 0.0;
            for weight in weights.iter_mut() {
                if *weight > limit {
                    excess += *weight - limit;
                    *weight = limit;
                } else {
                    free_total += *weight;
                }
            }
            if excess <= f64::EPSILON || free_total <= f64::EPSILON {
                break;
            }
            for weight in weights.iter_mut() {
                if *weight < limit && *weight > 0.0 {
                    *weight += excess * (*weight / free_total);
                }
            }
        }
    }
    // Renormalize (trimmed members stay at zero).
    let total: f64 = weights.iter().sum();
    if total > f64::EPSILON {
        for weight in weights.iter_mut() {
            *weight /= total;
        }
    }
    weights
}

/// Tier join for CRDT merges (#9): the more protected tier wins — a
/// memory promoted on one replica stays promoted after the union.
fn tier_join(left: TraceTier, right: TraceTier) -> TraceTier {
    use TraceTier::*;
    match (left, right) {
        (Core, _) | (_, Core) => Core,
        (Standard, _) | (_, Standard) => Standard,
        (Candidate, Candidate) => Candidate,
    }
}
