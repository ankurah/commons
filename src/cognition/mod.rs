//! Records of what parties want, assert, understand, sense and decide:
//! cognition in the broad sense of cognitive science, done together through
//! shared records. Only a Claim asserts that something is the case.
//!
//! What every kind of record shares is not repeated here: who made it, who
//! last edited it and who closed it are components from the `records`
//! namespace, added to the same entity. How records connect is the `graph`
//! namespace's edges.

mod claim;
mod decision;
mod expectation;
mod intention;
mod intuition;
mod perspective;

pub use claim::*;
pub use decision::*;
pub use expectation::*;
pub use intention::*;
pub use intuition::*;
pub use perspective::*;
