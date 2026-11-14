use crate::support::string_enum;
use ankurah::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ScopeStatus {
    Planned,
    Active,
    Paused,
    Blocked,
    Done,
    Abandoned,
}
string_enum!(ScopeStatus);

/// A body of work with one aim, which tasks are worked in.
///
/// Who opened it is `Creation`; when it was finished or abandoned is
/// `Closure`; where its work happens in git is `WorkLocation`, from the `git`
/// namespace. All are on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "intention-graph.scope")]
#[model(no_ffi)]
pub struct Scope {
    #[active_type(LWW)]
    pub status: ScopeStatus,
    #[active_type(LWW)]
    pub title: String,
    /// What the work in it is trying to achieve.
    // OPEN: how does this text relate to `Intention`? As a component on this
    // entity it would bring a second title and a second status; as a record
    // of its own, the scope would serve it by a link.
    #[active_type(LWW)]
    pub intention: String,
    /// When work on it began, which need not be when it was opened.
    #[active_type(LWW)]
    pub started: Option<i64>,
}
