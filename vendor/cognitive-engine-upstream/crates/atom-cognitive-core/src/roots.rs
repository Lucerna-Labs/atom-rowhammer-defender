//! The cognitive roots (docs/ROOTS.md v0.2). Eight, provisional on
//! ablations — EXPLORE was demoted at the first revision (decision D20):
//! v1's exploration mechanisms are behaviorally inert, and the ecosystem's
//! receipts (gate-before-blend; routed memories beat blended ones) say
//! imprecision in matching corrupts rather than explores.

/// A cognitive root operation. Roots are irreducible, load-bearing,
/// composable, and evidenced — see docs/ROOTS.md for the admission record
/// and the demotion log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Root {
    Attend,
    Bind,
    Reinforce,
    Decay,
    Predict,
    Revise,
    Gate,
    Conserve,
    /// Admitted v0.3 (E1, 2026-09-25) — hypothesis formation over
    /// readings. See docs/ROOTS.md and docs/E1-PERCEPTION.md.
    Perceive,
}

impl Root {
    /// Every admitted root, in ROOTS.md order.
    pub const ALL: [Root; 9] = [
        Root::Attend,
        Root::Bind,
        Root::Reinforce,
        Root::Decay,
        Root::Predict,
        Root::Revise,
        Root::Gate,
        Root::Conserve,
        Root::Perceive,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Root::Attend => "attend",
            Root::Bind => "bind",
            Root::Reinforce => "reinforce",
            Root::Decay => "decay",
            Root::Predict => "predict",
            Root::Revise => "revise",
            Root::Gate => "gate",
            Root::Conserve => "conserve",
            Root::Perceive => "perceive",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots_have_unique_names() {
        let mut names: Vec<&str> = Root::ALL.iter().map(|r| r.name()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "root names must be unique");
        assert_eq!(count, 9, "ROOTS.md v0.3 admits nine roots");
    }
}
