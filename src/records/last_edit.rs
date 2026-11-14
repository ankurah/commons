use crate::identity::Principal;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// When a record's words were last changed, and by whom, so that a reader
/// knows they are not as first written. Added at the first edit; a record
/// never edited does not carry it.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.last_edit")]
#[model(no_ffi)]
pub struct LastEdit {
    #[active_type(LWW)]
    pub edited: i64,
    pub edited_by: Option<Ref<Principal>>,
}
