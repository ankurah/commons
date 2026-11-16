use ankurah::Model;
use serde::{Deserialize, Serialize};

/// Something someone asserts to be the case.
///
/// Who wrote it down is `Created`; whose claim it is, when someone else
/// wrote it down, is `Attribution`; its last rewording is `Edited`. All are
/// on the same entity. What supports it arrives by `supports` edges.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "cognition.claim")]
#[model(id = "THctMWzZhys_PU4JvYUUl84QsAXk_rkigH31agBk4G8")]
#[model(no_ffi)]
pub struct Claim {
    #[active_type(LWW)]
    pub title: String,
    #[active_type(LWW)]
    pub words: String,
}
