use crate::identity::Principal;
use crate::support::string_enum;
use ankurah::{
    property::{Json, Ref},
    Model,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TaskStatus {
    Proposed,
    Open,
    InProgress,
    InReview,
    Done,
    Cancelled,
    Rejected,
}
string_enum!(TaskStatus);

/// A piece of work someone can pick up and finish.
///
/// Who filed it is `Creation`, whom it is assigned to is `Assignment`, how it
/// was handed in is `Submission`, and how it ended is `Closure`, all on the
/// same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "intention-graph.task")]
#[model(no_ffi)]
pub struct Task {
    #[active_type(LWW)]
    pub title: String,
    #[active_type(YrsString)]
    pub what: String,
    #[active_type(YrsString)]
    pub why: String,
    #[active_type(YrsString)]
    pub done_when: String,
    #[active_type(LWW)]
    pub status: TaskStatus,
    /// Whether someone other than the submitter must accept it before it is done.
    pub needs_review: bool,
    /// Who approved the task when it was proposed.
    pub approved_by: Option<Ref<Principal>>,
    /// The reason the task is on hold. Set means held.
    #[active_type(LWW)]
    pub hold: Option<String>,
    /// Not actionable before this moment.
    // OPEN: a date component of its own, so that other kinds of record can be
    // dated too. The views that hide a task until it is due also filter on
    // `status`, so that waits for a query across two components.
    #[active_type(LWW)]
    pub not_before: Option<i64>,
    /// 0 is most urgent.
    #[active_type(LWW)]
    pub priority: Option<i64>,
    /// An array of strings.
    pub labels: Json,
    /// When anything about the task last changed, its status included.
    pub updated: i64,
}
