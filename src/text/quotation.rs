use crate::property::{DynRef, Timestamp};
use ankurah::Model;
use serde::{Deserialize, Serialize};

/// Words a person or an agent said or wrote, quoted.
///
/// Who recorded them is `Created`; whose words they are, when someone else
/// recorded them, is `Attribution`. Both are on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "text.quotation")]
#[model(id = "UpLx9aFmM5nZGI_yfZoUoIck6Fk05F_D89zGilEEoLE")]
#[model(no_ffi)]
pub struct Quotation {
    #[active_type(LWW)]
    #[property(id = "95tSKcm6MH1wnFNnw2K_AEcW_QkwyLn4h4gi7dA0djI")]
    pub text: String,
    /// When the words were said, where known.
    #[active_type(LWW)]
    #[property(id = "ymCkWj0VZjVMc6gRzrMgwazCc_G04wM0brtjyS9H8J4")]
    pub said: Option<Timestamp>,
    /// The record the words were said or written in, such as a message, where
    /// known.
    #[property(id = "1B3hxhMRFIWLZG1i3NS3Rixs_kI88YL13FJbcKfbX5o")]
    pub source: Option<DynRef>,
}
