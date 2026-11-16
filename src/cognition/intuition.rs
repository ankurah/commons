use ankurah::Model;
use serde::{Deserialize, Serialize};

/// A sense, hunch or suspicion, not necessarily asserted. What it is about
/// arrives by `concerns` edges.
///
/// Who wrote it down is `Created`; whose it is, when someone else wrote it
/// down, is `Attribution`; its last rewording is `Edited`; that it was tested
/// or let go is `Closed`. All are on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "cognition.intuition")]
#[model(id = "jVjAHgOeKeN18NTHLAYfxbHwgblM4mXAvWsOef4dLmc")]
#[model(no_ffi)]
pub struct Intuition {
    #[active_type(LWW)]
    pub title: String,
    #[active_type(LWW)]
    pub words: String,
}
