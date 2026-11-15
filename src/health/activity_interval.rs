use super::PhysicalActivity;
use crate::property::Timestamp;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// A named portion of one physical activity, such as a work interval, rest, or lap.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "health.activity_interval")]
#[model(no_ffi)]
pub struct ActivityInterval {
    pub activity: Ref<PhysicalActivity>,
    /// An open name in snake case, such as work, rest, or lap.
    #[active_type(LWW)]
    pub kind: String,
    /// Start, inclusive.
    pub started_at: Timestamp,
    /// End, exclusive, when known.
    #[active_type(LWW)]
    pub ended_at: Option<Timestamp>,
}
