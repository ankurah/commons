use crate::identity::Principal;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// Who made a record, and when. The words in it are theirs too, unless the
/// record also carries `Attribution`.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.creation")]
pub struct Creation {
    pub created_by: Ref<Principal>,
    pub created: i64,
}
