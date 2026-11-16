use crate::property::RefMap;
use ankurah::Model;
use serde::{Deserialize, Serialize};

/// What a record was made from, under the minted `EdgeKind` entities that name
/// each role: `discovered_from`, the record it surfaced from or whose words it
/// was made from, such as a message; and `generated_by`, the activity that
/// made it, such as an import run. Who acted is `Created.by`, not Via.
// OPEN: whether the keys become registered properties, once Ankurah can
// register a property outside a model.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.via")]
#[model(no_ffi)]
pub struct Via {
    pub via: RefMap,
}
