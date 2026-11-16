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
#[model(no_ffi)]
pub struct Scope {
    #[active_type(LWW)]
    pub title: String,
}
