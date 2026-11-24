//! The cognitive engine: executes the canonical cycles over the root
//! stages, driven by whatever ordering-validated stack it is handed.
//!
//! Execution is stage-order-driven by design: hand the engine a mis-ordered
//! (but legal) stack and it genuinely executes in that order — which is how
//! the permutation suite proves the ordering laws, and how the ungated
//! query variant (the O7 failure mode, kept deliberately reachable) shows
//! what happens without the gate.

pub mod action_world;
pub mod boundary;
pub mod causal_path;
pub mod chaos;
pub mod config;
pub mod derivation_paths;
pub mod engine;
pub mod evidence;
pub mod generation;
pub mod geometry;
pub mod harness;
pub mod interference_field;
pub mod knowledge;
pub mod perception;
pub mod perception_field;
pub mod preference;
pub mod perception_quantum;
pub mod reasoning_closure;
pub mod reasoning_types;
pub mod scratch_pad;
pub mod selection;
pub mod spatial;
pub mod stacks;
pub mod trace;
pub mod weave;

pub use config::EngineConfig;
pub use engine::{
    BlendReport, BottleneckReport, CognitiveEngine, EngineError, FluencyReport, FluencyState,
    FrontMember, MarkingOutcome, MergeReport, ObservationKind, ObservationPlan, ObserveOutcome,
    QueryOutcome, QueryTuning, QuorumReport, RenormalizationReport, Transition,
};
pub use perception::{Perception, PerceptionLabel, PerceptionOutcome, Reading};
pub use trace::{MemoryTrace, TraceTier, cosine_similarity, softmax};
