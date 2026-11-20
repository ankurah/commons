//! Definitions of models and their properties that Ankurah applications can share, so
//! that two applications, or two nodes, mean the same thing by "a message" or "a task".
//!
//! One module per namespace, one file per model. Each namespace is behind the cargo
//! feature of the same name, which also turns on the namespaces its models refer to.
//! `property` holds the property types the namespaces share, in every build. `dimension`
//! holds one for applications built on the Commons.
//!
//! The models' definitions are minted once and kept beside their structs, as events in the
//! text form of `akev`, and each struct declares the minted ids of its model and of each of
//! its properties. `seed` loads them into a durable node, where a model and each of its
//! properties bind by those ids, never by label or name; `registration` makes the events of
//! any declaration.

pub mod akev;
pub mod dimension;
pub mod namespaces;
pub mod property;
pub mod registration;
mod seed;
#[cfg(any(feature = "identity", feature = "records", feature = "git", feature = "work"))]
mod support;

#[cfg(feature = "cognition")]
pub mod cognition;
#[cfg(feature = "git")]
pub mod git;
#[cfg(feature = "graph")]
pub mod graph;
#[cfg(feature = "health")]
pub mod health;
#[cfg(feature = "identity")]
pub mod identity;
#[cfg(feature = "messaging")]
pub mod messaging;
#[cfg(feature = "records")]
pub mod records;
#[cfg(feature = "text")]
pub mod text;
#[cfg(feature = "work")]
pub mod work;

pub use seed::{seed, SeedError};
