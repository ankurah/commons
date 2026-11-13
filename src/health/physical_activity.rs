use crate::identity::Principal;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// One bout of physical activity by one person.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "health.physical_activity")]
pub struct PhysicalActivity {
    /// The human who did the activity.
    pub performed_by: Ref<Principal>,
    /// An open activity name in snake case, such as walking, yoga, or indoor_skiing.
    #[active_type(LWW)]
    pub kind: String,
    /// Start of the activity in Unix milliseconds.
    pub started_at: i64,
    /// End of the activity in Unix milliseconds, when known. Pauses may fall within these bounds.
    #[active_type(LWW)]
    pub ended_at: Option<i64>,
}
