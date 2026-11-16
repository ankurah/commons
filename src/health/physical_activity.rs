use crate::identity::Principal;
use crate::property::Timestamp;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// One bout of physical activity by one person.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "health.physical_activity")]
#[model(id = "48bPysiZLc_-nQxCCC0W6GUfwdls3jNgqLRdgd07H8E")]
#[model(no_ffi)]
pub struct PhysicalActivity {
    /// The human who did the activity.
    pub performed_by: Ref<Principal>,
    /// An open activity name in snake case, such as walking, yoga, or indoor_skiing.
    #[active_type(LWW)]
    pub kind: String,
    /// Start of the activity.
    pub started_at: Timestamp,
    /// End of the activity, when known. Pauses may fall within these bounds.
    #[active_type(LWW)]
    pub ended_at: Option<Timestamp>,
}
