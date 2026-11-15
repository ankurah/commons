//! Text taken from somewhere: a part of a document that someone marked, and
//! words that someone said, quoted.
//!
//! A highlight is one entity with several components: `Fragment`, here, is
//! the marked part; an anchor in the namespace of the kind of document says
//! where it sits, such as `FileAnchor` in `git` for a file kept in git; and
//! who marked it is `Created`, from `records`.

mod fragment;
mod quotation;

pub use fragment::*;
pub use quotation::*;
