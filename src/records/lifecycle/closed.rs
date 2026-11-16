use crate::identity::Principal;
use crate::property::Timestamp;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// That a record is finished with, who finished it, how and why: a task
/// closed, a question resolved, a decision repealed, an edge retracted, a
/// scope ended, a principal retired. Attached when the record closes.
///
/// The record is closed while `closed` holds a time. Reopening it sets every
/// field to none.
// TODO: fields not optional, and the component taken off on reopening, once
// Ankurah can remove a component from an entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.closed")]
#[model(no_ffi)]
pub struct Closed {
    #[active_type(LWW)]
    pub closed: Option<Timestamp>,
    #[active_type(LWW)]
    pub by: Option<Ref<Principal>>,
    /// How it ended, as one word in snake case: an intention `fulfilled` or
    /// `withdrawn`; a decision `superseded` or `repealed`; a message
    /// `resolved` or `deleted`; an expectation `held`, `failed` or
    /// `inconclusive`; a scope `done` or `abandoned`.
    #[active_type(LWW)]
    pub outcome: Option<String>,
    #[active_type(LWW)]
    pub reason: Option<String>,
}
