use crate::identity::Principal;
use ankurah::{property::Ref, EntityId, Model};
use serde::{Deserialize, Serialize};

/// That a record is finished with, who finished it, and why: a task closed, a
/// question resolved, a decision repealed, a link retracted, a scope ended.
/// How it ended is the record's own `status`.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.closure")]
pub struct Closure {
    /// None again once the record is reopened, because a component cannot be
    /// taken off an entity.
    #[active_type(LWW)]
    pub closed: Option<i64>,
    pub closed_by: Option<Ref<Principal>>,
    #[active_type(LWW)]
    pub reason: Option<String>,
    /// The message that gives the reason at more length. An untyped id, so
    /// that this namespace does not depend on messaging.
    pub reason_message: Option<EntityId>,
}
