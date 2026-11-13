use ankurah::{property::Json, Model};
use serde::{Deserialize, Serialize};

/// Where a body of work happens in git, so that a session can find its scope
/// from the checkout it runs in. Added to an entity that also carries `Scope`,
/// from the `intention-graph` namespace, when the scope is first given a place.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "git.work_location")]
pub struct WorkLocation {
    /// The repository, named by the key of its upstream remote, never by a
    /// directory. None, with no branches, once the scope's place is cleared:
    /// a component cannot be taken off an entity.
    // OPEN: a key and not a reference to `Repository`, as Fwd Centaur has it.
    #[active_type(LWW)]
    pub repository_key: Option<String>,
    /// An array of branch names, each trimmed and listed once.
    pub branches: Json,
}
