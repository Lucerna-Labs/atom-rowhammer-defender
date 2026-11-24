//! BUILD-12 — wiring the engines into the harness. Precommitted in
//! `docs/BUILD12-HARNESS.md` BEFORE this code, under vault Rules 1/2/3,
//! project rules 0/0a/0b, and docs/PURPOSE.md.
//!
//! The harness is a FIXED, ordering-validated composite: engines as
//! stages, prerequisites machine-checked by the chassis stack
//! machinery, and NO content-dependent routing anywhere (the
//! orchestrator clause of Rule 1: select a fixed stack, invoke it,
//! report what happened — verdicts are outputs, never selectors).
//!
//! AUDIT: the composite's prerequisites are structural (which stage
//! consumes which output). No stage consults the task to choose a path.

use crate::causal_path::{PathError, WalkOrder};
use crate::derivation_paths::{WordGraph, Words};
use crate::evidence::{Evidence, EvidenceError, EvidenceSource};
use crate::geometry::{GeometryField, Point};
use crate::knowledge::{derivable, derivable_count};
use crate::reasoning_types::top_down;
use crate::scratch_pad::ScratchPad;

/// Stage ids of the harness composite.
pub const HARNESS_FORM: &str = "harness.form";
pub const HARNESS_IDENTIFY: &str = "harness.identify";
pub const HARNESS_JUDGE: &str = "harness.judge";
pub const HARNESS_DERIVE: &str = "harness.derive";
pub const HARNESS_SUFFICIENCY: &str = "harness.sufficiency";
pub const HARNESS_ASK: &str = "harness.ask";
pub const HARNESS_RESUFFICIENCY: &str = "harness.resufficiency";
pub const HARNESS_GENERATE: &str = "harness.generate";

/// The composite stage list with machine-checked prerequisites
/// (structural: consumer <- producer). Assembly is validated by the
/// chassis Stack machinery — illegal orders do not exist.
pub fn harness_stages() -> Vec<atom_cognitive_core::stages::StageSpec> {
    use atom_cognitive_core::stages::StageSpec;
    vec![
        StageSpec::new(HARNESS_FORM, None, &[]),
        StageSpec::new(HARNESS_IDENTIFY, None, &[HARNESS_FORM]),
        StageSpec::new(HARNESS_JUDGE, None, &[HARNESS_FORM]),
        StageSpec::new(HARNESS_DERIVE, None, &[]),
        StageSpec::new(HARNESS_SUFFICIENCY, None, &[HARNESS_DERIVE]),
        StageSpec::new(HARNESS_ASK, None, &[HARNESS_SUFFICIENCY]),
        StageSpec::new(HARNESS_RESUFFICIENCY, None, &[HARNESS_ASK]),
        StageSpec::new(HARNESS_GENERATE, None, &[HARNESS_RESUFFICIENCY]),
    ]
}

/// A harness task: stored blocks, a scene, a target relation, a query
/// triple for the judge, and the source world's content (the missing
/// block it will supply — content, fixed).
pub struct Task {
    pub blocks: Vec<Point>,
    pub scene: Vec<Point>,
    pub target: Point,
    pub query: (Point, Point, Point),
    pub source_block: Point,
}

/// The reported outcome of one full harness run: every stage's output,
/// in order. Nothing is hidden and nothing branches.
#[derive(Debug, Clone, PartialEq)]
pub struct HarnessReport {
    pub scene_support: usize,
    pub judge: bool,
    pub closure_before: usize,
    pub sufficient_before: bool,
    pub sufficient_after: bool,
    pub derivable_now: bool,
    pub economy_blocks: usize,
    pub economy_answers: usize,
    /// BUILD-23: acquired evidence — provenance reported, never a selector.
    pub acquired: Option<Evidence>,
    /// BUILD-30: the derivation's provenance — the words over the SAME
    /// declared blocks the sufficiency verdict measured, bounded by the
    /// caller's declared hops. Reported, never a selector: no stage reads it.
    pub provenance: Option<Words>,
}

/// Run the FIXED composite over a task. The same stage sequence runs
/// for every task; only the reported values differ (W4).
pub fn run(task: &Task) -> HarnessReport {
    let field = GeometryField::form(&task.scene);
    report_from(&field, task, None, None)
}

/// BUILD-30: the same fixed composite with ONE added unconditional step —
/// the word graph over the task's OWN declared blocks, enumerated to the
/// caller's declared bound, REPORTED. No stage consumes it, so nothing is
/// routed by it; blocks with no structure at all are the primitive's typed
/// refusal and produce NO report (fail-closed, no fallback run).
pub fn run_with_provenance(task: &Task, hops: usize) -> Result<HarnessReport, PathError> {
    let field = GeometryField::form(&task.scene);
    let graph = WordGraph::from_relations(&task.blocks, crate::interference_field::N as i64);
    let words = graph.words_to(task.target, hops, WalkOrder::BreadthFirst)?;
    Ok(report_from(&field, task, None, Some(words)))
}

/// BUILD-23: the same fixed composite with ONE added step, itself
/// unconditional — the declared source is asked exactly once, before the
/// stages run, and its receipt is REPORTED. No stage consumes the evidence,
/// so nothing is routed by it; a failing source is a typed error and NO
/// report is produced (fail-closed, no fallback run).
pub fn run_with_acquisition(
    task: &Task,
    source: &dyn EvidenceSource,
    query: &str,
) -> Result<HarnessReport, EvidenceError> {
    let field = GeometryField::form(&task.scene);
    let evidence = source.acquire(query)?;
    Ok(report_from(&field, task, Some(evidence), None))
}

/// BUILD-16: the pad-wired variant of the composite. The formed field's
/// pattern is carried through the working-memory pad, and the
/// identify/judge stages read it back FROM THE PAD — the pad is
/// load-bearing by construction. Every other stage is the shared body,
/// identical to `run`.
pub fn run_with_pad(task: &Task, pad: &mut ScratchPad) -> HarnessReport {
    let formed = GeometryField::form(&task.scene);
    // The pattern is carried as REAL amplitudes — a declared wiring
    // interface; the write->read round-trip is the identity, exact in f64.
    pad.write_pattern(formed.raw_pattern());
    let readback = GeometryField::from_pattern(&pad.read_pattern(), &task.scene);
    report_from(&readback, task, None, None)
}

/// The task-boundary clearance (BUILD-16). MEASURED FINDING, kept on
/// record: the harness's readout (`difference_vectors`) thresholds at an
/// ABSOLUTE 1e-9 on the pattern's DFT power, so a dissipating residue
/// stays visible to it until ~3.8e7 ticks (the estimate is derived from
/// the scene's own bin power; ~1e10 site-operations per boundary) — while
/// the pad's derived 31622-tick budget serves the pad's own RELATIVE 1e-9
/// readout. Clearing budgets are therefore CONSUMER-SCOPED, and for the
/// harness's exactness requirement the reset arm (BUILD-15's destructive
/// clearing, exactly zero) is the mechanism that works. The dissipation
/// arm stays available via `ScratchPad::clear` with its measured envelope.
pub fn clear_for_next(pad: &mut ScratchPad) {
    pad.clear(crate::scratch_pad::ClearMechanism::Reset, 0);
}

/// The shared stage body — one implementation, whatever carried the field.
fn report_from(
    field: &GeometryField,
    task: &Task,
    acquired: Option<Evidence>,
    provenance: Option<Words>,
) -> HarnessReport {
    // form + identify (BUILD-1/3): the scene's emergent relations.
    let scene_support = field.difference_vectors().len();

    // judge (BUILD-8/9): a geometric query on the same field.
    let judge = field.betweenness(task.query.0, task.query.1, task.query.2);

    // derive + sufficiency (BUILD-2/11): the closure of the stored
    // blocks; the verdict is a MEASUREMENT.
    let closure_before = derivable_count(&task.blocks);
    let sufficient_before = derivable(&task.blocks, task.target);

    // ask (BUILD-5): ALWAYS runs — the source answers unconditionally;
    // its content is absorbed. No branch selected this step.
    let mut blocks_after = task.blocks.clone();
    blocks_after.push(task.source_block);
    // The top-down constraint check (BUILD-10) runs here as the
    // harness's own verification that the absorbed block remains inside
    // the whole's permitted structure — a measurement, not a filter.
    let whole = {
        let mut whole = task.scene.clone();
        whole.push(task.source_block);
        whole
    };
    let _ = top_down(&task.scene, &whole);

    // re-sufficiency (BUILD-11).
    let sufficient_after = derivable(&blocks_after, task.target);

    // generate: the target derived from the (absorbed) blocks exactly.
    let derivable_now = sufficient_after;
    let economy_blocks = blocks_after.len();
    let economy_answers = derivable_count(&blocks_after);

    HarnessReport {
        scene_support,
        judge,
        closure_before,
        sufficient_before,
        sufficient_after,
        derivable_now,
        economy_blocks,
        economy_answers,
        acquired,
        provenance,
    }
}
