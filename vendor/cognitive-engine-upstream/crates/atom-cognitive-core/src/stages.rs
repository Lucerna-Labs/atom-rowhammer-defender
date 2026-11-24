//! The machine-checked stage ordering (docs/ORDERING.md enforcement layer 1).
//!
//! A `StageSpec` names a stage, the root it implements, and its
//! prerequisites. `assemble` validates: unique ids, known prerequisites,
//! **positional** satisfaction (a prerequisite must be assembled before its
//! dependent), and acyclicity (Kahn's algorithm). A mis-ordered or cyclic
//! stack is a construction error — the stack does not exist.

use crate::roots::Root;

/// Declaration of one stage of a cognitive cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageSpec {
    /// Stable stage identifier, e.g. `"observe.predict"`.
    pub id: &'static str,
    /// The root this stage implements, if any (validation stages are None).
    pub root: Option<Root>,
    /// Stage ids that must be assembled strictly before this stage.
    pub prerequisites: &'static [&'static str],
}

impl StageSpec {
    pub const fn new(
        id: &'static str,
        root: Option<Root>,
        prerequisites: &'static [&'static str],
    ) -> Self {
        StageSpec {
            id,
            root,
            prerequisites,
        }
    }
}

/// Ways a stack assembly fails. All are construction errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackError {
    DuplicateStage(&'static str),
    UnknownPrerequisite {
        stage: &'static str,
        prerequisite: &'static str,
    },
    UnsatisfiedPrerequisite {
        stage: &'static str,
        prerequisite: &'static str,
        prerequisite_position: usize,
        dependent_position: usize,
    },
    CyclicOrdering,
    EmptyStack,
}

impl core::fmt::Display for StackError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StackError::DuplicateStage(id) => write!(f, "duplicate stage: {id}"),
            StackError::UnknownPrerequisite {
                stage,
                prerequisite,
            } => write!(
                f,
                "stage {stage} requires unknown prerequisite {prerequisite}"
            ),
            StackError::UnsatisfiedPrerequisite {
                stage,
                prerequisite,
                prerequisite_position,
                dependent_position,
            } => write!(
                f,
                "ordering law violated: {stage} (position {dependent_position}) requires \
                 {prerequisite} before it (found at position {prerequisite_position})"
            ),
            StackError::CyclicOrdering => write!(f, "cyclic stage ordering rejected"),
            StackError::EmptyStack => write!(f, "a stack must contain at least one stage"),
        }
    }
}

impl core::error::Error for StackError {}

/// An assembled, ordering-validated stack. The stage order inside is the
/// execution order; it always satisfies every declared prerequisite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack {
    stages: Vec<StageSpec>,
}

impl Stack {
    /// Validate and assemble. See `StackError` for the rejection classes.
    pub fn assemble(stages: Vec<StageSpec>) -> Result<Stack, StackError> {
        if stages.is_empty() {
            return Err(StackError::EmptyStack);
        }
        let mut seen: Vec<&'static str> = Vec::with_capacity(stages.len());
        for stage in &stages {
            if seen.contains(&stage.id) {
                return Err(StackError::DuplicateStage(stage.id));
            }
            seen.push(stage.id);
        }
        // Acyclicity (Kahn) over declared prerequisite edges, independent of
        // the given order: a true cycle is reported as a cycle, not as an
        // unsatisfiable position.
        let mut remaining_edges: Vec<(&'static str, &'static str)> = Vec::new();
        for stage in &stages {
            for prerequisite in stage.prerequisites {
                if !seen.contains(prerequisite) {
                    return Err(StackError::UnknownPrerequisite {
                        stage: stage.id,
                        prerequisite,
                    });
                }
                remaining_edges.push((*prerequisite, stage.id));
            }
        }
        let mut resolved: Vec<&'static str> = Vec::new();
        let total = stages.len();
        while resolved.len() < total {
            let before = resolved.len();
            for stage in &stages {
                if resolved.contains(&stage.id) {
                    continue;
                }
                let blocked = remaining_edges
                    .iter()
                    .any(|(from, to)| *to == stage.id && !resolved.contains(from));
                if !blocked {
                    resolved.push(stage.id);
                }
            }
            if resolved.len() == before {
                return Err(StackError::CyclicOrdering);
            }
        }
        // Positional satisfaction: every prerequisite must appear earlier.
        for (position, stage) in stages.iter().enumerate() {
            for prerequisite in stage.prerequisites {
                let prerequisite_position = stages
                    .iter()
                    .position(|s| s.id == *prerequisite)
                    .expect("existence checked above");
                if prerequisite_position >= position {
                    return Err(StackError::UnsatisfiedPrerequisite {
                        stage: stage.id,
                        prerequisite,
                        prerequisite_position,
                        dependent_position: position,
                    });
                }
            }
        }
        Ok(Stack { stages })
    }

    /// Execution order of stage ids.
    pub fn order(&self) -> Vec<&'static str> {
        self.stages.iter().map(|s| s.id).collect()
    }

    pub fn stages(&self) -> &[StageSpec] {
        &self.stages
    }

    pub fn len(&self) -> usize {
        self.stages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.stages.iter().any(|s| s.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roots::Root;

    const VALID: &[StageSpec] = &[
        StageSpec::new("validate", None, &[]),
        StageSpec::new("decay", Some(Root::Decay), &["validate"]),
        StageSpec::new("write", Some(Root::Bind), &["decay"]),
    ];

    #[test]
    fn valid_stack_assembles() {
        let stack = Stack::assemble(VALID.to_vec()).expect("valid stack");
        assert_eq!(stack.order(), vec!["validate", "decay", "write"]);
    }

    #[test]
    fn duplicate_stage_rejected() {
        let mut stages = VALID.to_vec();
        stages.push(StageSpec::new("write", Some(Root::Bind), &["decay"]));
        assert_eq!(
            Stack::assemble(stages),
            Err(StackError::DuplicateStage("write"))
        );
    }

    #[test]
    fn unknown_prerequisite_rejected() {
        let stages = vec![
            StageSpec::new("validate", None, &[]),
            StageSpec::new("write", Some(Root::Bind), &["nonexistent"]),
        ];
        assert_eq!(
            Stack::assemble(stages),
            Err(StackError::UnknownPrerequisite {
                stage: "write",
                prerequisite: "nonexistent"
            })
        );
    }

    #[test]
    fn positional_violation_rejected() {
        // decay declared before write in the list, but write does not require
        // decay — legal. The illegal case: write FIRST while requiring decay.
        let stages = vec![
            StageSpec::new("validate", None, &[]),
            StageSpec::new("write", Some(Root::Bind), &["decay"]),
            StageSpec::new("decay", Some(Root::Decay), &["validate"]),
        ];
        match Stack::assemble(stages) {
            Err(StackError::UnsatisfiedPrerequisite {
                stage,
                prerequisite,
                prerequisite_position,
                dependent_position,
            }) => {
                assert_eq!(stage, "write");
                assert_eq!(prerequisite, "decay");
                assert_eq!((prerequisite_position, dependent_position), (2, 1));
            }
            other => panic!("expected positional violation, got {other:?}"),
        }
    }

    #[test]
    fn cycle_rejected_as_cycle() {
        let stages = vec![
            StageSpec::new("a", Some(Root::Bind), &["b"]),
            StageSpec::new("b", Some(Root::Decay), &["a"]),
        ];
        assert_eq!(Stack::assemble(stages), Err(StackError::CyclicOrdering));
    }

    #[test]
    fn empty_stack_rejected() {
        assert_eq!(Stack::assemble(vec![]), Err(StackError::EmptyStack));
    }
}
