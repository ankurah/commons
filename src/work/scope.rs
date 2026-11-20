use ankurah::Model;
use serde::{Deserialize, Serialize};

/// A body of work with one aim. Its tasks are `worked_in` it, and it `serves`
/// the intentions it is for: both are edges.
///
/// Who opened it is `Created`; how it ended, done or abandoned, is `Closed`;
/// where its work happens in git is `WorkLocation`, from the `git` namespace.
/// All are on the same entity.
// OPEN: planned, active, paused and blocked move to a component of fwd's own,
// which is still to be shaped.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "work.scope")]
#[model(id = "iT1lR2sfIvf735FsTAYXpXI76PXKwkAxRZ5cyPti8gQ")]
#[model(no_ffi)]
pub struct Scope {
    #[active_type(LWW)]
    #[property(id = "Plt5j-bbvzLDTdpgDGcNwEr5E9DfN5UU98Ncmapejg4")]
    pub title: String,
}
