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
#[model(id = "t2Eoj6lcrbxFPw7u3Ig0YOVhGQ1Qzq4GaylpGn8tP8w")]
#[model(no_ffi)]
pub struct Task {
    #[active_type(LWW)]
    #[property(id = "S_IBUYg8QStL93yTW4v26MeORGHmjeq2sFf36uVPohE")]
    pub title: String,
    #[active_type(YrsString)]
    #[property(id = "i2kBgC5IXbo8yXlBxZ1eZetrm69joZvToDCW-salKZM")]
    pub what: String,
    #[active_type(YrsString)]
    #[property(id = "eH-ZqcoIcC96FABk6cf_HKwZKCyS3JfHw1SOHWM7nj4")]
    pub why: String,
    #[active_type(YrsString)]
    #[property(id = "zW8lWRp5MUP37WR3uFqiI-eUrYkPmvgW4Y41ep-F2aE")]
    pub done_when: String,
    #[active_type(LWW)]
    #[property(id = "2010qyoWFMap5_AUSk76c43Ur67C-zjbMC0N1qIR6tY")]
    pub status: TaskStatus,
    /// The reason the task is on hold. Set means held.
    #[active_type(LWW)]
    #[property(id = "_BqydT7p2z9GNcMMVybQkCkcqEjgq-_vW3JiA2o4V2U")]
    pub hold: Option<String>,
    /// 0 is most urgent.
    #[active_type(LWW)]
    #[property(id = "guiu-VI_31x6RXS2K7x7x3SNGnjsJP54XH_lwETHXjY")]
    pub priority: Option<i64>,
    /// An array of strings.
    #[property(id = "6TM5QGG5VRvtXBSqwdg3-ohmJ1q2BNV_Sb9hlpf8Wfo")]
    pub labels: Json,
}
