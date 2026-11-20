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
#[model(id = "sh6695XIEJUgLvmIg7g5ihdRDxAu7WGcKMYhawLMnsg")]
#[model(no_ffi)]
pub struct Checked {
    #[active_type(LWW)]
    #[property(id = "H7fSKvRKOrZmG_2z3voFMLQ4eeO06OM15IDZ4wuPkyE")]
    pub checked: Option<Timestamp>,
    #[active_type(LWW)]
    #[property(id = "2oBVJ-u-dW072-V7p5wt6a9foW_7WwUNUqMxiVRU89Q")]
    pub by: Option<Ref<Principal>>,
}
