use crate::property::{DynRef, Timestamp};
use ankurah::Model;
use serde::{Deserialize, Serialize};

/// Words a person or an agent said or wrote, quoted.
///
/// Who recorded them is `Created`; whose words they are, when someone else
/// recorded them, is `Attribution`. Both are on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "text.quotation")]
#[model(no_ffi)]
pub struct Quotation {
    #[active_type(LWW)]
    pub text: String,
    /// When the words were said, where known.
    #[active_type(LWW)]
    pub said: Option<Timestamp>,
    /// The record the words were said or written in, such as a message, where
    /// known.
    pub source: Option<DynRef>,
}
