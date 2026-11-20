use ankurah::Model;
use serde::{Deserialize, Serialize};

/// Something someone asserts to be the case.
///
/// Who wrote it down is `Created`; whose claim it is, when someone else
/// wrote it down, is `Attribution`; its last rewording is `Edited`. All are
/// on the same entity. What supports it arrives by `supports` edges.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "cognition.claim")]
#[model(id = "THctMWzZhys_PU4JvYUUl84QsAXk_rkigH31agBk4G8")]
#[model(no_ffi)]
pub struct Claim {
    #[active_type(LWW)]
    #[property(id = "F1hEoa4E8S3xslilO0RED7M52SDpH9FPE_wOQ6ZJssY")]
    pub title: String,
    #[active_type(LWW)]
    #[property(id = "ScTsNP3ag-Kl8VkaqkUFRYFTqWqvcId6maYzX2kgqEc")]
    pub words: String,
}
