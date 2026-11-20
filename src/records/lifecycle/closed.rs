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
#[model(id = "9J52GuMASQzfCUgKJ3PhFO7trfK9CjxVmTYUT65X-YY")]
#[model(no_ffi)]
pub struct Closed {
    #[active_type(LWW)]
    #[property(id = "PF26Ux3-82XoWD_gSrdZH5rIlraucNAJTdghBRGuLlE")]
    pub closed: Option<Timestamp>,
    #[active_type(LWW)]
    #[property(id = "jSXliJhbzAxIhPBoS0qqsqqUnuj6I7rmq9wBg1XEl-U")]
    pub by: Option<Ref<Principal>>,
    /// How it ended, as one word in snake case: an intention `fulfilled` or
    /// `withdrawn`; a decision `superseded` or `repealed`; a message
    /// `resolved` or `deleted`; an expectation `held`, `failed` or
    /// `inconclusive`; a scope `done` or `abandoned`.
    #[active_type(LWW)]
    #[property(id = "Pz8uI2UII5DiKMoa1MXLGUKharPn1dtMmen3ZK7C0bY")]
    pub outcome: Option<String>,
    #[active_type(LWW)]
    #[property(id = "pvDpaO5E8DWlt4Ik9_xOzXknPydsdnT-8n2iYnBuLBc")]
    pub reason: Option<String>,
}
