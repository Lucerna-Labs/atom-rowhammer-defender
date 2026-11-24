//! The cognitive engine selects a repair METHOD from verified past outcomes.
//! A separate actuator preserves changed bytes and verifies trusted bytes.
//! Cognitive memory cannot suppress a hard file-integrity trigger.

use std::fmt;

use atom_cognitive_engine::stacks::{observe_stack, query_stack};
use atom_cognitive_engine::{CognitiveEngine, EngineConfig, FluencyState};

pub const CUE_DIM: usize = 12;
pub const RECORD_BYTES: usize = 160;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepairMethod {
    InPlace,
    Atomic,
}
impl RepairMethod {
    fn value(self) -> usize {
        match self {
            Self::InPlace => 0,
            Self::Atomic => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionSource {
    HardObjectGate,
    CognitiveMemory,
    AbstentionFallback,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RepairContext {
    pub different_bytes: usize,
    pub object_changed: bool,
    pub file_mode: u32,
    pub parent_mode: u32,
    pub atom_long_mean: f64,
    pub prior_alerts: u32,
    /// Source attribution, excluded from the cue and never an action gate.
    pub origin: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Selection {
    pub method: RepairMethod,
    pub source: SelectionSource,
    pub confidence: Option<f64>,
    pub supporting_traces: Vec<u64>,
    pub fluency: FluencyState,
    pub response_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlError {
    InvalidContext,
    Engine(String),
}
impl fmt::Display for ControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidContext => f.write_str("invalid cognitive repair context"),
            Self::Engine(error) => write!(f, "cognitive engine: {error}"),
        }
    }
}
impl std::error::Error for ControlError {}

/// Session memory of successfully verified actions. Failed attempts never
/// become authority. Unknown or contested queries use the fixed fallback.
pub struct CognitiveRepairSelector {
    engine: CognitiveEngine,
}
impl CognitiveRepairSelector {
    pub fn new() -> Result<Self, ControlError> {
        let config = EngineConfig {
            capacity: 64,
            cue_dim: CUE_DIM,
            value_count: 2,
            conservation_budget: 64.0,
            initial_ttl: 32,
            ..EngineConfig::default()
        };
        let engine = CognitiveEngine::new(config)
            .map_err(|error| ControlError::Engine(error.to_string()))?;
        Ok(Self { engine })
    }

    pub fn successful_experiences(&self) -> u64 {
        self.engine.observations()
    }

    pub fn select(&self, context: RepairContext) -> Result<Selection, ControlError> {
        let cue = encode(context)?;
        let outcome = self
            .engine
            .query(&query_stack(), &cue)
            .map_err(|error| ControlError::Engine(error.to_string()))?;
        let contradicted = outcome.activations.iter().any(|(id, _)| {
            self.engine
                .traces()
                .iter()
                .any(|trace| trace.id == *id && trace.contradictions > 0)
        });
        let (method, source) = if context.object_changed {
            (RepairMethod::Atomic, SelectionSource::HardObjectGate)
        } else if outcome.fluency.state == FluencyState::Contested || contradicted {
            (RepairMethod::Atomic, SelectionSource::AbstentionFallback)
        } else {
            match outcome.claim.map(|(value, _)| value) {
                Some(0) => (RepairMethod::InPlace, SelectionSource::CognitiveMemory),
                Some(1) => (RepairMethod::Atomic, SelectionSource::CognitiveMemory),
                _ => (RepairMethod::Atomic, SelectionSource::AbstentionFallback),
            }
        };
        Ok(Selection {
            method,
            source,
            confidence: outcome.claim.map(|(_, confidence)| confidence),
            supporting_traces: outcome.activations.iter().map(|(id, _)| *id).collect(),
            fluency: outcome.fluency.state,
            response_hash: outcome.response_hash,
        })
    }

    /// Call only after the actuator has preserved evidence and verified the
    /// post-repair trusted bytes. This is the consequence entering memory.
    pub fn record_verified_success(
        &mut self,
        context: RepairContext,
        method: RepairMethod,
    ) -> Result<String, ControlError> {
        let cue = encode(context)?;
        let outcome = self
            .engine
            .observe_with_origin(
                &observe_stack(),
                &cue,
                &[method.value()],
                1.0,
                context.origin,
            )
            .map_err(|error| ControlError::Engine(error.to_string()))?;
        Ok(outcome.state_hash)
    }
}

pub fn encode(context: RepairContext) -> Result<[f64; CUE_DIM], ControlError> {
    if context.different_bytes > RECORD_BYTES
        || !context.atom_long_mean.is_finite()
        || context.atom_long_mean < 0.0
    {
        return Err(ControlError::InvalidContext);
    }
    let file_write = context.file_mode & 0o200 != 0;
    let dir_write = context.parent_mode & 0o200 != 0;
    let mut cue = [
        0.25,
        context.different_bytes as f64 / RECORD_BYTES as f64,
        if context.object_changed { 2.0 } else { 0.0 },
        if file_write { 2.0 } else { 0.0 },
        if file_write { 0.0 } else { 2.0 },
        if dir_write { 2.0 } else { 0.0 },
        if dir_write { 0.0 } else { 2.0 },
        if context.file_mode & 0o020 != 0 {
            0.5
        } else {
            0.0
        },
        if context.parent_mode & 0o020 != 0 {
            0.5
        } else {
            0.0
        },
        context.atom_long_mean.min(1.0),
        (context.prior_alerts.min(8) as f64) / 8.0,
        0.25,
    ];
    let norm = cue.iter().map(|value| value * value).sum::<f64>().sqrt();
    if !norm.is_finite() || norm <= 0.0 {
        return Err(ControlError::InvalidContext);
    }
    for value in &mut cue {
        *value /= norm;
    }
    Ok(cue)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn learned_success_changes_method_without_weakening_object_gate() -> Result<(), ControlError> {
        let mut control = CognitiveRepairSelector::new()?;
        let context = RepairContext {
            different_bytes: 1,
            object_changed: false,
            file_mode: 0o644,
            parent_mode: 0o555,
            atom_long_mean: 0.0,
            prior_alerts: 0,
            origin: 1,
        };
        assert_eq!(
            control.select(context)?.source,
            SelectionSource::AbstentionFallback
        );
        control.record_verified_success(context, RepairMethod::InPlace)?;
        let learned = control.select(context)?;
        assert_eq!(learned.method, RepairMethod::InPlace);
        assert_eq!(learned.source, SelectionSource::CognitiveMemory);
        assert!(!learned.supporting_traces.is_empty());
        let swapped = RepairContext {
            object_changed: true,
            ..context
        };
        assert_eq!(
            control.select(swapped)?.source,
            SelectionSource::HardObjectGate
        );
        assert_eq!(control.select(swapped)?.method, RepairMethod::Atomic);
        Ok(())
    }

    #[test]
    fn opposing_sources_do_not_authorize_in_place() -> Result<(), ControlError> {
        let mut control = CognitiveRepairSelector::new()?;
        let context = RepairContext {
            different_bytes: 1,
            object_changed: false,
            file_mode: 0o644,
            parent_mode: 0o555,
            atom_long_mean: 0.0,
            prior_alerts: 0,
            origin: 11,
        };
        control.record_verified_success(context, RepairMethod::InPlace)?;
        control.record_verified_success(
            RepairContext {
                origin: 22,
                ..context
            },
            RepairMethod::Atomic,
        )?;
        let outcome = control.select(RepairContext {
            origin: 33,
            ..context
        })?;
        assert_eq!(outcome.method, RepairMethod::Atomic);
        assert_eq!(outcome.source, SelectionSource::AbstentionFallback);
        Ok(())
    }
}
