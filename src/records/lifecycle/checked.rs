use crate::identity::Principal;
use crate::property::Timestamp;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// That a step of a checklist is done, who checked it, and when. Attached when
/// the step is checked.
///
/// The step is checked while `checked` holds a time. Unchecking it sets both
/// fields to none.
// TODO: fields not optional, and the component taken off on unchecking, once
// Ankurah can remove a component from an entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.checked")]
#[model(no_ffi)]
pub struct Checked {
    #[active_type(LWW)]
    pub checked: Option<Timestamp>,
    #[active_type(LWW)]
    pub by: Option<Ref<Principal>>,
}
