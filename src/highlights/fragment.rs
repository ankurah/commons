use ankurah::Model;
use serde::{Deserialize, Serialize};

/// A part of a document that someone marked. It keeps the marked text and the
/// text on either side, so that the part can be found again after the
/// document changes.
///
/// This says nothing about which document, or where in it. A second component
/// on the same entity says that: `FileAnchor`, in the `git` namespace, for a
/// file kept in git. The two together are a highlight. Who marked it, and
/// when, is `Creation`.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "highlights.fragment")]
pub struct Fragment {
    /// The marked text. Empty when the whole document is marked.
    #[active_type(LWW)]
    pub anchored_text: String,
    #[active_type(LWW)]
    pub context_before: String,
    #[active_type(LWW)]
    pub context_after: String,
    /// A hash of the marked text with runs of whitespace collapsed, for
    /// recognising two highlights on the same text. Absent when the whole
    /// document is marked.
    // OPEN: computed from `anchored_text`. Should a computed value be a
    // published property?
    #[active_type(LWW)]
    pub span_key: Option<String>,
}
