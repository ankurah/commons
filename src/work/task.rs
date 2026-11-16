use crate::support::string_enum;
use ankurah::{property::Json, Model};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TaskStatus {
    Open,
    InProgress,
    Done,
    Cancelled,
}
string_enum!(TaskStatus);

/// A piece of work someone can pick up and finish.
///
/// Who filed it is `Created`, whom it waits on is `Assignment`, whether it
/// was accepted when proposed is `Acceptance`, when it can be acted on is
/// `Deferred`, and how it ended is `Closed`, all on the same entity. Each
/// hand-in of it is a `Submission` of its own.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "work.task")]
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
    /// The reason the task is on hold. Set means held.
    #[active_type(LWW)]
    pub hold: Option<String>,
    /// 0 is most urgent.
    #[active_type(LWW)]
    pub priority: Option<i64>,
    /// An array of strings.
    pub labels: Json,
}
