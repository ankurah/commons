use ankurah::{property::Json, Model};
use serde::{Deserialize, Serialize};

/// Something someone holds to be the case, which an intention or a decision
/// may rest on: what they observed, or what they sense without being able to
/// show it.
///
/// Who wrote it down is `Creation`; whose claim it is, when someone else
/// wrote it down, is `Attribution`; its last rewording is `LastEdit`. All are
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
    // material is better said by `cites` links. Whether a claim is an
    // observation or an intuition is not recorded apart from this.
    pub evidence: Option<Json>,
}
