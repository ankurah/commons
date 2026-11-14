use crate::identity::Principal;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// Whom a record waits on: the one who is to do a task, or to answer a
/// question. A message that carries it is a question; a message without it
/// asks nothing.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.assignment")]
#[model(no_ffi)]
pub struct Assignment {
    /// None while the record waits on no one in particular.
    pub assignee: Option<Ref<Principal>>,
    /// Who last set the assignee. None when that is not known.
    pub assigned_by: Option<Ref<Principal>>,
}
