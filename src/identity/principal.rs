use crate::support::string_enum;
use ankurah::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum PrincipalKind {
    Human,
    Agent,
}
string_enum!(PrincipalKind);

/// A person or an agent: who acts, not how they are recognised when they sign
/// in, which is an `IdentityBinding`. A principal spans projects, so it
/// carries no project's tag.
///
/// Its retirement is `Closed`; a retired principal stays, so that what it did
/// is still attributed to it.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "identity.principal")]
#[model(id = "ZITgy1aws0-JJAI_D6t__wr3kAKaAIPZImNVQr1YLmU")]
#[model(no_ffi)]
pub struct Principal {
    #[active_type(LWW)]
    #[property(id = "SNpy8mNteUTuGD6sCFVjH_za0g0Y4PcFvHvL4oZItBo")]
    pub kind: PrincipalKind,
    /// The name others type. Not unique.
    #[active_type(LWW)]
    #[property(id = "d8hkaJmIk-bvv_Nqoh-HakCLfy8V9XXdLKtUhwjm3d0")]
    pub handle: String,
    #[active_type(LWW)]
    #[property(id = "rZShtG3Z-n1Iq4nQOY8jgmhBPc5u57D8adSW4C7JHuU")]
    pub display_name: String,
}
