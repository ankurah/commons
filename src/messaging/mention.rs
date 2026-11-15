use ankurah::Model;
use serde::{Deserialize, Serialize};

/// What an edge has only when it records that a message's text names a record.
/// Added to the entity of a `mention` edge, which runs from the message to the
/// record named; a message may name many, so each mention is an entity of its
/// own and not a component of the message.
///
/// Who caused it is the edge's `Created`; a name later edited out of the text
/// is the edge's `Closed`.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "messaging.mention")]
#[model(no_ffi)]
pub struct Mention {
    /// Its place among the names in the text, counted from one.
    #[active_type(LWW)]
    pub ordinal: i64,
    /// True when an edit of the message added the name, and not the first
    /// writing of it.
    pub added_by_edit: bool,
}
