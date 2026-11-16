use crate::identity::Principal;
use crate::property::Timestamp;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// Who made a record, and when. Attached when the record is made.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.created")]
#[model(id = "b8gFJFUqwCgbg08XyEcfAszpwIFjhvLS-g-KShvrfBE")]
#[model(no_ffi)]
pub struct Created {
    pub created: Timestamp,
    pub by: Ref<Principal>,
}
