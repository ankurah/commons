use crate::identity::Principal;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// Whom a record waits on.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.assignment")]
#[model(id = "PYDgBCw8YrEL-v1aXA0hLfQCc198-QalzjqywpCXwow")]
#[model(no_ffi)]
pub struct Assignment {
    /// None while the record waits on no one in particular.
    #[property(id = "OHvXRou9_LJ7KqQNV_0vdpw7PuP7OuExio-od0fU4g4")]
    pub assignee: Option<Ref<Principal>>,
    /// Who last set the assignee. None when that is not known.
    #[property(id = "uKkGmHREKBdvRC_7KBRDD39CrOmZ3pvQYriyC0_vrVg")]
    pub assigned_by: Option<Ref<Principal>>,
}
