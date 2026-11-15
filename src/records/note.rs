use ankurah::Model;
use serde::{Deserialize, Serialize};

/// A short remark someone attached to a record.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.note")]
#[model(no_ffi)]
pub struct Note {
    #[active_type(LWW)]
    pub note: Option<String>,
}
