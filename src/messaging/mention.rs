use ankurah::Model;
use serde::{Deserialize, Serialize};

/// What an edge has only when it records that a message's text names a record.
/// Added to the entity of a `mention` edge, which runs from the message to the
/// record named. Each occurrence of a name is an edge of its own, so a message
/// that names a record twice has two. How a name is written in the text is
/// the application's.
///
/// Who caused it is the edge's `Created`; a name later edited out of the text
/// is the edge's `Closed`.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "messaging.mention")]
#[model(id = "2FN2YPKQwf1U6QazGL1fA6NsPQyAFqTNJTeVcG1WP7E")]
#[model(no_ffi)]
pub struct Mention {
    /// Its place among the names in the text, counted from one.
    #[active_type(LWW)]
    #[property(id = "GiVpFVJWTm8bqYZ6FzJiPCbRW9ClmvEG32Y5rsWpXuw")]
    pub ordinal: i64,
    /// True when an edit of the message added the name, and not the first
    /// writing of it.
    #[property(id = "iSVc8q7vV0139lJjIz0xkyq5TnN-O0p8Dk36oBSyacg")]
    pub added_by_edit: bool,
}
