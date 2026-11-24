//! BUILD-13 — the boundary policy layer. Precommitted in
//! `docs/BUILD13-BOUNDARY.md` BEFORE this code.
//!
//! THE ONLY POLICY SURFACE IN THE PROJECT, kept deliberately tiny. The
//! orchestrator clause grants its verb: SELECT a fixed stack, invoke it,
//! report what happened. It never computes, never inspects content,
//! never touches an answer. Unknown shape -> Refusal (fail-closed, no
//! instruction fallback).
//!
//! AUDIT: the selector's body contains exactly ONE lookup. Task shapes
//! are DECLARED by the caller; nothing derives them from content.

use crate::harness::{run, Task as HarnessTask, HarnessReport};

/// A declared task shape (the caller says which shape its request is).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskShape {
    /// A scene question over stored relation blocks.
    SceneQuestion,
    /// A product over stored blocks at declared registers.
    ArithmeticProduct,
    /// BUILD-24: a scene question that also consults a declared evidence
    /// source (through `invoke_with_source` only).
    SceneQuestionWithEvidence,
    /// BUILD-27: a registered composition, invoked by declared name.
    Chain(&'static str),
    /// BUILD-30: a scene question whose report also carries the derivation's
    /// provenance — the words over the task's OWN declared blocks, bounded by
    /// the caller's declared hops.
    SceneQuestionWithProvenance,
}

/// BUILD-27: a declared composition — an ordered list of registered shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chain {
    pub name: &'static str,
    pub stages: &'static [TaskShape],
}

/// The shipped registry of declared chains (DATA — adding a composition is a
/// table entry, not code). BUILD-30 appends `provenance_then_product`; the
/// two BUILD-27 entries are unchanged.
pub const CHAINS: &[Chain] = &[
    Chain {
        name: "scene_then_product",
        stages: &[TaskShape::SceneQuestion, TaskShape::ArithmeticProduct],
    },
    Chain {
        name: "nested",
        stages: &[TaskShape::Chain("scene_then_product"), TaskShape::SceneQuestion],
    },
    Chain {
        name: "provenance_then_product",
        stages: &[
            TaskShape::SceneQuestionWithProvenance,
            TaskShape::ArithmeticProduct,
        ],
    },
];

/// BUILD-27: the composition registry. Validation is a construction gate
/// (unknown names and cycles never register); invocation walks the DECLARED
/// DAG — depth is structure, and nothing is computed between stages.
pub struct Registry;

impl Registry {
    /// Validate a chain table: every named stage must exist in the table, and
    /// the composition graph must be acyclic. Typed refusals, before any run.
    pub fn validate(chains: &[Chain]) -> Result<(), &'static str> {
        for chain in chains {
            for stage in chain.stages {
                if let TaskShape::Chain(name) = stage {
                    if !chains.iter().any(|c| c.name == *name) {
                        return Err("a chain names an unregistered sub-stack");
                    }
                }
            }
        }
        fn visit(chains: &[Chain], index: usize, state: &mut [u8]) -> Result<(), &'static str> {
            match state[index] {
                1 => return Err("the composition graph contains a cycle"),
                2 => return Ok(()),
                _ => {}
            }
            state[index] = 1;
            for stage in chains[index].stages {
                if let TaskShape::Chain(name) = stage {
                    if let Some(sub) = chains.iter().position(|c| c.name == *name) {
                        visit(chains, sub, state)?;
                    }
                }
            }
            state[index] = 2;
            Ok(())
        }
        let mut state = vec![0u8; chains.len()];
        for index in 0..chains.len() {
            visit(chains, index, &mut state)?;
        }
        Ok(())
    }

    /// Invoke a declared chain: stages run in declared order; leaf stages
    /// through the existing boundary, chain stages by recursion. A count
    /// mismatch refuses the chain; a refused stage surfaces IN PLACE and the
    /// remaining stages still report — orchestration reports what happened.
    pub fn invoke(shape: TaskShape, request: &Request) -> Outcome {
        match shape {
            TaskShape::Chain(name) => {
                let Some(chain) = CHAINS.iter().find(|c| c.name == name) else {
                    return Outcome::Refused("unknown chain");
                };
                let Request::Chain(requests) = request else {
                    return Outcome::Refused("declared shape does not match the request");
                };
                if requests.len() != chain.stages.len() {
                    return Outcome::Refused("declared shape does not match the request");
                }
                let outcomes = chain
                    .stages
                    .iter()
                    .zip(requests)
                    .map(|(stage, sub_request)| Registry::invoke(*stage, sub_request))
                    .collect();
                Outcome::Chain(outcomes)
            }
            leaf => Boundary::invoke(leaf, request),
        }
    }
}

/// What the boundary returns: a report, or a refusal.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Scene(HarnessReport),
    Product(u128),
    /// Fail-closed: no registered stack serves this shape.
    Refused(&'static str),
    /// BUILD-27: a declared composition's nested outcomes, in declared order.
    Chain(Vec<Outcome>),
}

/// The registry: validated stacks, each declared shape -> fixed runner.
/// Registration validates assembly at entry.
pub struct Boundary;

impl Boundary {
    /// The declared registry: (shape, runner). Fixed by construction —
    /// nothing here is chosen at runtime beyond the shape lookup.
    pub fn invoke(shape: TaskShape, request: &Request) -> Outcome {
        match shape {
            TaskShape::SceneQuestion => match request {
                Request::Scene(task) => Outcome::Scene(run(task)),
                // The declared shape and the request's content disagree:
                // the boundary refuses rather than coercing (fail-closed).
                _ => Outcome::Refused("declared shape does not match the request"),
            },
            TaskShape::ArithmeticProduct => match request {
                Request::Product {
                    modulus_1,
                    modulus_2,
                    block,
                    steps,
                } => {
                    let m1 = crate::knowledge::generated_product(*modulus_1, *block, *steps);
                    let m2 = crate::knowledge::generated_product(*modulus_2, *block, *steps);
                    Outcome::Product(crate::knowledge::reconstruct(
                        *modulus_1, m1, *modulus_2, m2,
                    ))
                }
                _ => Outcome::Refused("declared shape does not match the request"),
            },
            // BUILD-24: the evidence shape is reached ONLY through
            // `invoke_with_source`; through the source-less entry it fails
            // closed.
            TaskShape::SceneQuestionWithEvidence => {
                Outcome::Refused("declared shape requires an evidence source")
            }
            // BUILD-27: chains are reached through `Registry::invoke`.
            TaskShape::Chain(_) => Outcome::Refused("declared shape is a composition"),
            // BUILD-30: the provenance shape reaches the harness's
            // provenance entry point; the primitive's refusal (no structure
            // in the declared blocks) is fail-closed and NO report exists.
            TaskShape::SceneQuestionWithProvenance => match request {
                Request::SceneWithProvenance { task, hops } => {
                    match crate::harness::run_with_provenance(task, *hops) {
                        Ok(report) => Outcome::Scene(report),
                        Err(_) => Outcome::Refused(
                            "the provenance readout refused the declared blocks (no structure)",
                        ),
                    }
                }
                _ => Outcome::Refused("declared shape does not match the request"),
            },
        }
    }

    /// BUILD-24: the boundary with a declared evidence source. The selector
    /// is unchanged — one more declared lookup; the acquisition runs
    /// unconditionally inside the harness (no branch on any verdict), and a
    /// source failure is a typed error with NO report (fail-closed).
    pub fn invoke_with_source(
        shape: TaskShape,
        request: &Request,
        source: &dyn crate::evidence::EvidenceSource,
    ) -> Result<Outcome, crate::evidence::EvidenceError> {
        match shape {
            TaskShape::SceneQuestionWithEvidence => match request {
                Request::SceneWithEvidence { task, query } => {
                    let report = crate::harness::run_with_acquisition(task, source, query)?;
                    Ok(Outcome::Scene(report))
                }
                _ => Ok(Outcome::Refused("declared shape does not match the request")),
            },
            _ => Ok(Outcome::Refused("declared shape does not match the request")),
        }
    }
}

/// The request payloads the boundary can carry (content — never
/// inspected by the selector itself).
pub enum Request {
    Scene(HarnessTask),
    Product {
        modulus_1: u64,
        modulus_2: u64,
        block: u64,
        steps: u64,
    },
    /// BUILD-24: a scene question that also consults a declared evidence
    /// source (content — the selector never inspects it).
    SceneWithEvidence { task: HarnessTask, query: String },
    /// BUILD-27: one request per declared stage of a chain (count-matched at
    /// the boundary; a mismatch refuses).
    Chain(Vec<Request>),
    /// BUILD-30: a scene question plus the caller's declared provenance
    /// bound (content — the selector never inspects it).
    SceneWithProvenance { task: HarnessTask, hops: usize },
}

/// Registration validation: a stack's stages must assemble — a
/// mis-ordered or cyclic stack is a construction error and cannot enter.
pub fn validate_registration(
    stages: Vec<atom_cognitive_core::stages::StageSpec>,
) -> Result<atom_cognitive_core::stages::Stack, atom_cognitive_core::stages::StackError> {
    atom_cognitive_core::stages::Stack::assemble(stages)
}
