//! atom-cognitive-core — the root set and the machine-checked stage ordering.
//!
//! House law: zero third-party dependencies. SHA-256 lives in `digest`.
//! The stage builder enforces the ordering contract from `docs/ORDERING.md`:
//! prerequisites are positional (a prerequisite must be assembled *before*
//! its dependent), cycles are rejected, unknown prerequisites are rejected.

pub mod digest;
pub mod roots;
pub mod stages;

pub use roots::Root;
pub use stages::{Stack, StackError, StageSpec};
