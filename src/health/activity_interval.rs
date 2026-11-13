use super::PhysicalActivity;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// A named portion of one physical activity, such as a work interval, rest, or lap.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "health.activity_interval")]
pub struct ActivityInterval {
    pub activity: Ref<PhysicalActivity>,
    /// An open name in snake case, such as work, rest, or lap.
    #[active_type(LWW)]
    pub kind: String,
    /// Start in Unix milliseconds, inclusive.
    pub started_at: i64,
    /// End in Unix milliseconds, exclusive, when known.
    #[active_type(LWW)]
    pub ended_at: Option<i64>,
}
