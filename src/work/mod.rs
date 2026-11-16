//! Work and what is handed in for it: tasks, the scopes they are worked in,
//! and submissions of finished work.
//!
//! What every kind of record shares is not repeated here: who made it, whom
//! it waits on, whether it was accepted and who closed it are components
//! from the `records` namespace, added to the same entity. How records
//! connect is the `graph` namespace's edges.

mod scope;
mod submission;
mod task;

pub use scope::*;
pub use submission::*;
pub use task::*;
