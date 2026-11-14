use ankurah::Model;
use serde::{Deserialize, Serialize};

/// What signs in as a principal, so that a command can be attributed to the
/// principal it acts as. Added to an entity that also carries `Principal`,
/// when something first signs in as it.
// OPEN: one credential for each principal, as Fwd Centaur has it. A principal
// with several would need each to be an entity of its own. Ankurah's node
// identity work may replace this with a key.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "identity.credential")]
#[model(no_ffi)]
pub struct Credential {
    /// For a person, the account at the service that signed them in, such as
    /// `github:<numeric id>`. For an agent, the session it runs in, such as
    /// `claude-code:<session>/<agent>`. None once the principal is unbound,
    /// as when a retired principal's session passes to another.
    // OPEN: Fwd Centaur keeps these as two properties of the principal,
    // `auth_subject` for a person and `subject` for an agent.
    #[active_type(LWW)]
    pub subject: Option<String>,
}
