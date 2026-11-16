use ankurah::Model;
use serde::{Deserialize, Serialize};

/// A part of a document that someone marked. It keeps the marked text and the
/// text on either side, so that the part can be found again after the
/// document changes.
///
/// This says nothing about which document, or where in it. A second component
/// on the same entity says that: `FileAnchor`, in the `git` namespace, for a
/// file kept in git. The two together are a highlight. Who marked it, and
/// when, is `Created`.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "text.fragment")]
#[model(id = "T1jwWExgRbdiPf5B0tVDoKdrrLzioT9W8LoghZ8iOlY")]
#[model(no_ffi)]
pub struct Fragment {
    /// The marked text. Empty when the whole document is marked.
    #[active_type(LWW)]
    pub anchored_text: String,
    #[active_type(LWW)]
    pub context_before: String,
    #[active_type(LWW)]
    pub context_after: String,
}
