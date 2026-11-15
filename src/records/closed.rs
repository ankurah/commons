use crate::identity::Principal;
use crate::property::{DynRef, Timestamp};
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// That a record is finished with, who finished it, and why: a task closed, a
/// question resolved, a decision repealed, an edge retracted, a scope ended,
/// a principal retired. Attached when the record closes. How it ended is the
/// record's own status where it has one; an edge or a principal has none.
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
    #[active_type(LWW)]
    pub reason: Option<String>,
    /// The message that gives the reason at more length. Not a `Ref<Message>`,
    /// so that this namespace does not depend on messaging.
    #[active_type(LWW)]
    pub reason_message: Option<DynRef>,
}
