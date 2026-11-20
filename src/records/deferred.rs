use crate::property::Timestamp;
use ankurah::Model;
use serde::{Deserialize, Serialize};

/// That a record is not actionable before a moment, such as a task put off.
/// Attached when the record is first put off; `until` is none once it no
/// longer is.
// TODO: not optional, and the component taken off when the record no longer
// waits, once Ankurah can remove a component from an entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.deferred")]
#[model(id = "2KmnCWnswY4uxkb3nYmE8J2cFiiR_fIUESAmw0rDSzw")]
#[model(no_ffi)]
pub struct Deferred {
    #[active_type(LWW)]
    #[property(id = "cBqg4i8tL80d6sgbESlAI-EBlHUYYA5LIbGAxLAFdbU")]
    pub until: Option<Timestamp>,
}
