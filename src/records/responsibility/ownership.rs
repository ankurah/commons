use crate::identity::Principal;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// The principal to whom the application attributes ownership of this record.
/// Applications enforce read and write permissions through their access policies.
/// Never on an intention, which belongs to no one.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.ownership")]
#[model(id = "QDbyx81eSYyg0U4Q6zFygOV8USLByHAYRnOPo5BcIkA")]
#[model(no_ffi)]
pub struct Ownership {
    #[property(id = "HZGDHguUBj2jRPaUFU-stkebCMx7zMxQP2GSb6MpjAI")]
    pub owner: Ref<Principal>,
}
