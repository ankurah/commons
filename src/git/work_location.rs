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
#[model(id = "R8za6z12K9IMC6UQcO-7xxzfAEO19W1nOd-5-bTOpB4")]
#[model(no_ffi)]
pub struct WorkLocation {
    /// None, with no branches, once the scope's place is cleared: a component
    /// cannot be taken off an entity.
    #[active_type(LWW)]
    #[property(id = "5hoQtGk3aJ01Rh3waEbIQmewNsmi05j7vgH_rG9jRuA")]
    pub repository: Option<Ref<Repository>>,
    /// An array of branch names, each trimmed and listed once.
    #[property(id = "-Ba421-NnCBV4M5pjXZvbjzM9x7-TxPHhAoetaHjPpQ")]
    pub branches: Json,
}
