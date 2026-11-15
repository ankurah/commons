use ankurah::{property::Json, Model};
use serde::{Deserialize, Serialize};

/// What caused a record to be made: the message whose text named it, the
/// principal acting directly, or a migration that made it from older data.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.via")]
#[model(no_ffi)]
pub struct Via {
    // TODO: ask Ankurah for a way to hold a bag of tagged references (a tag
    // entity's id to a DynRef) in place of JSON.
    pub via: Json,
}
