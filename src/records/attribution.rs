use crate::identity::Principal;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// Whose words a record holds, when someone else wrote them down: an agent
/// recording what a person said they want. Without it, the words are those of
/// whoever made the record.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.attribution")]
pub struct Attribution {
    pub speaker: Ref<Principal>,
}
