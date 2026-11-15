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
/// in.
///
/// What signs in as it is `Credential`, on the same entity. Its retirement is
/// `Closed`; a retired principal stays, so that what it did is still
/// attributed to it.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "identity.principal")]
#[model(no_ffi)]
pub struct Principal {
    #[active_type(LWW)]
    pub kind: PrincipalKind,
    /// The name others type. Not unique.
    #[active_type(LWW)]
    pub handle: String,
    #[active_type(LWW)]
    pub display_name: String,
}
