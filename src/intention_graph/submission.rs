use crate::identity::Principal;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// What a task has once someone hands it in as finished: who handed it in,
/// what shows it is finished, and who accepted it. Added to an entity that
/// also carries `Task`.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "intention-graph.submission")]
pub struct Submission {
    pub submitted_by: Ref<Principal>,
    /// The commits, test runs and the like that show the work is done.
    #[active_type(LWW)]
    pub evidence: String,
    pub accepted_by: Option<Ref<Principal>>,
}
