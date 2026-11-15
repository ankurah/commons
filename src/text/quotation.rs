use crate::property::{DynRef, Timestamp};
use ankurah::Model;
use serde::{Deserialize, Serialize};

/// Something a person or an agent said, quoted as evidence for a record: an
/// intention, a claim or a decision.
///
/// Who recorded it is `Created`; whose words they are, when someone else
/// recorded them, is `Attribution`. Both are on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "text.quotation")]
#[model(no_ffi)]
pub struct Quotation {
    /// The record the words support.
    pub supports: DynRef,
    #[active_type(LWW)]
    pub text: String,
    /// When the words were said, where known.
    #[active_type(LWW)]
    pub said: Option<Timestamp>,
    /// The message the words were said in, where they were said in one. Not a
    /// `Ref<Message>`, because messaging depends on this namespace.
    pub source: Option<DynRef>,
}
