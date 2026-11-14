use crate::support::string_enum;
use ankurah::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum IntentionStatus {
    /// Wanted, with no one yet committed to act on it.
    Wished,
    /// Someone has committed to act on it.
    Committed,
    Fulfilled,
    Withdrawn,
}
string_enum!(IntentionStatus);

/// Something a person or an agent wants to be true, in their own words.
///
/// Who wrote it down is `Creation`; whose want it is, when someone else wrote
/// it down, is `Attribution`; its last rewording is `LastEdit`; how it was
/// fulfilled or withdrawn is `Closure`. All are on the same entity.
// OPEN: a wish and a committed intention are one model here, told apart by
// `status`. The alternative is two components on one entity. Who committed is
// not recorded yet; `Assignment` could say it.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "intention-graph.intention")]
#[model(no_ffi)]
pub struct Intention {
    #[active_type(LWW)]
    pub title: String,
    #[active_type(LWW)]
    pub words: String,
    #[active_type(LWW)]
    pub status: IntentionStatus,
}
