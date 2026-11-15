use ankurah::{property::Json, Model};
use serde::{Deserialize, Serialize};

/// Something someone asserts to be the case, which an intention or a decision
/// may rest on.
///
/// Who wrote it down is `Created`; whose claim it is, when someone else
/// wrote it down, is `Attribution`; its last rewording is `Edited`. All are
/// on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "intention-graph.claim")]
#[model(no_ffi)]
pub struct Claim {
    #[active_type(LWW)]
    pub title: String,
    #[active_type(LWW)]
    pub words: String,
    /// What supports it: messages, files, measurements.
    // OPEN: the shape of this JSON is not settled, nor whether supporting
    // material is better said by `cites` edges.
    pub evidence: Option<Json>,
}
