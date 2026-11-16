use ankurah::Model;
use serde::{Deserialize, Serialize};

/// A choice someone recorded so that others can rely on it.
///
/// Who wrote it down is `Created`; who ruled, and when, is `Acceptance`; how
/// it ended, superseded or repealed, is `Closed`. All are on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "cognition.decision")]
#[model(id = "mJYaRYRX0_9kv5a5VEAN8VQQ-BY0bav3bSxqxZUYt7s")]
#[model(no_ffi)]
pub struct Decision {
    #[active_type(LWW)]
    pub title: String,
    #[active_type(LWW)]
    pub decision: String,
    /// What led to it, where recorded.
    #[active_type(LWW)]
    pub context: Option<String>,
    /// What follows from it, where recorded.
    #[active_type(LWW)]
    pub consequences: Option<String>,
}
