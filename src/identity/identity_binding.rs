use super::Principal;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// That an account at a service, or an agent's session, acts as a principal,
/// so that a command from it is attributed to that principal. One record per
/// account or session; a principal may have several. `Closed` when unbound.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "identity.identity_binding")]
#[model(no_ffi)]
pub struct IdentityBinding {
    pub principal: Ref<Principal>,
    /// For a person, the account at the service that signed them in, such as
    /// `github:<numeric id>`. For an agent, the session it runs in, such as
    /// `claude-code:<session>/<agent>`.
    #[active_type(LWW)]
    pub subject: String,
}
