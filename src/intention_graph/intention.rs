use crate::support::string_enum;
use ankurah::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum IntentionStatus {
    Open,
    Fulfilled,
    Withdrawn,
}
string_enum!(IntentionStatus);

/// A statement of something wanted, which each party is aligned or not
/// aligned with. It belongs to no one: it is not anyone's words.
///
/// Who wrote it down is `Creation`; its last rewording is `LastEdit`; how it
/// was fulfilled or withdrawn is `Closure`. All are on the same entity. What
/// people said in support of it are `Excerpt` records, and each session it is
/// assigned to is a `Link` of kind `assigned_to`.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "intention-graph.intention")]
#[model(no_ffi)]
pub struct Intention {
    #[active_type(LWW)]
    pub title: String,
    #[active_type(LWW)]
    pub description: String,
    #[active_type(LWW)]
    pub status: IntentionStatus,
}
