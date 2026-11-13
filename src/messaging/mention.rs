use ankurah::Model;
use serde::{Deserialize, Serialize};

/// What a link has only when it records that a message's text names a record.
/// Added to the entity of a `mention` link, which runs from the message to the
/// record named; a message may name many, so each mention is an entity of its
/// own and not a component of the message.
///
/// Who caused it is the link's `Creation`; a name later edited out of the text
/// is the link's `Closure`.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "messaging.mention")]
pub struct Mention {
    /// Its place among the names in the text, counted from one.
    #[active_type(LWW)]
    pub ordinal: i64,
    /// True when an edit of the message added the name, and not the first
    /// writing of it.
    pub added_by_edit: bool,
}
