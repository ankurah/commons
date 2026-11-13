//! Definitions of models and their properties that Ankurah applications can share, so
//! that two applications, or two nodes, mean the same thing by "a message" or "a task".
//!
//! One module per namespace, one file per model. Each namespace is behind the cargo
//! feature of the same name, which also turns on the namespaces its models refer to.

#[cfg(any(feature = "identity", feature = "git", feature = "intention-graph", feature = "messaging"))]
mod support;

#[cfg(feature = "git")]
pub mod git;
#[cfg(feature = "health")]
pub mod health;
#[cfg(feature = "highlights")]
pub mod highlights;
#[cfg(feature = "identity")]
pub mod identity;
#[cfg(feature = "intention-graph")]
pub mod intention_graph;
#[cfg(feature = "messaging")]
pub mod messaging;
#[cfg(feature = "records")]
pub mod records;
