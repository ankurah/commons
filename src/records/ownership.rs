use crate::identity::Principal;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// The principal to whom the application attributes ownership of this record.
/// Applications enforce read and write permissions through their access policies.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.ownership")]
#[model(no_ffi)]
pub struct Ownership {
    pub owner: Ref<Principal>,
}
