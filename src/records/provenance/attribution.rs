use crate::identity::Principal;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// Whose words a record holds, when they are not those of whoever made it: an
/// agent recording what a person said they want. With no Attribution, the
/// words are the creator's; with one whose speaker is None, they are someone
/// else's, speaker unknown.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.attribution")]
#[model(id = "DfGhlrbNm6EcEMTL1XOU1Cft54dEbjxZ1b0AX-R0ibE")]
#[model(no_ffi)]
pub struct Attribution {
    pub speaker: Option<Ref<Principal>>,
}
