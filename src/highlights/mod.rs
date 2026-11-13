//! Parts of documents that someone marked, so a comment or a task can point
//! at them and still find them after the document changes.
//!
//! A highlight is one entity with several components. `Fragment`, here, is the
//! marked part. An anchor says where it sits, and lives in the namespace of
//! the kind of document: `FileAnchor` in `git` for a file kept in git. An
//! anchor for a page at a URL would be another, added when something needs it.
//! Who marked it is `Creation`, from `records`.

mod fragment;

pub use fragment::*;
