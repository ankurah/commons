use ankurah::{
    property::{Json, Ref},
    Model,
};
use serde::{Deserialize, Serialize};

use super::Repository;

/// Where a body of work happens in git. Added to an entity that also carries
/// `Scope`, from the `work` namespace, when the scope is first given a place.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "git.work_location")]
#[model(no_ffi)]
pub struct WorkLocation {
    /// None, with no branches, once the scope's place is cleared: a component
    /// cannot be taken off an entity.
    #[active_type(LWW)]
    pub repository: Option<Ref<Repository>>,
    /// An array of branch names, each trimmed and listed once.
    pub branches: Json,
}
