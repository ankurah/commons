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

/// A person or an agent taking part in a project: who acts, not how they are
/// recognised when they sign in.
///
/// What signs in as it is `Credential`, on the same entity. The principal that
/// replaced it, when an agent was relaunched, is a `supersedes` link.
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
    /// False once retired. The principal stays, so that what it did is still
    /// attributed to it.
    pub active: bool,
    /// Whether the principal belongs to the project whose graph holds it, as
    /// opposed to one that only read there or was only named there.
    pub member: Option<bool>,
    /// When the principal last wrote anything.
    #[active_type(LWW)]
    pub last_active: Option<i64>,
}
