use crate::identity::Principal;
use crate::property::Timestamp;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// When a record's words were last changed, and by whom, so that a reader
/// knows they are not as first written. Attached at the first edit and
/// updated at each later one; a record never edited does not carry it.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.edited")]
#[model(no_ffi)]
pub struct Edited {
    #[active_type(LWW)]
    pub edited: Timestamp,
    /// Optional so that an importer can say a record was edited without
    /// naming who; an application may require it.
    #[active_type(LWW)]
    pub by: Option<Ref<Principal>>,
}
