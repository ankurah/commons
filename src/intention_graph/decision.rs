use crate::identity::Principal;
use crate::support::string_enum;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

use super::Scope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum DecisionStatus {
    Proposed,
    Accepted,
    Rejected,
    Superseded,
    Repealed,
}
string_enum!(DecisionStatus);

/// A choice someone recorded so that others can rely on it.
///
/// Who wrote it down is `Creation`, and how it was rejected, superseded or
/// repealed is `Closure`, both on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "intention-graph.decision")]
#[model(no_ffi)]
pub struct Decision {
    /// Counted per project; cited as `D<number>`.
    pub number: i64,
    #[active_type(LWW)]
    pub title: String,
    #[active_type(LWW)]
    pub status: DecisionStatus,
    #[active_type(LWW)]
    pub area: Option<String>,
    /// The scope it governs; none means the whole project.
    pub scope: Option<Ref<Scope>>,
    #[active_type(LWW)]
    pub context: String,
    #[active_type(LWW)]
    pub decision: String,
    #[active_type(LWW)]
    pub consequences: String,
    /// Who ruled, which need not be who wrote it down. None while the
    /// decision is only proposed.
    pub decided_by: Option<Ref<Principal>>,
    #[active_type(LWW)]
    pub decided: Option<i64>,
}
