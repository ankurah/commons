use ankurah::{EntityId, Model};
use serde::{Deserialize, Serialize};

/// Something a person or an agent said, quoted as supporting evidence for a
/// record: an intention, a claim or a decision.
///
/// Who recorded it is `Creation`; whose words they are, when someone else
/// recorded them, is `Attribution`. Both are on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "intention-graph.excerpt")]
#[model(no_ffi)]
pub struct Excerpt {
    /// The record it supports.
    pub of: EntityId,
    #[active_type(LWW)]
    pub text: String,
    /// When the words were said, in Unix milliseconds, where known.
    #[active_type(LWW)]
    pub said: Option<i64>,
    /// The message the words were said in, where they were said in one.
    pub source: Option<EntityId>,
}
