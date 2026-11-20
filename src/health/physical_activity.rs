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
    #[property(id = "0uxUEjpityI4YvZgLb0ExkTpdzUjuY2ppmh9xDEoWDw")]
    pub performed_by: Ref<Principal>,
    /// An open activity name in snake case, such as walking, yoga, or indoor_skiing.
    #[active_type(LWW)]
    #[property(id = "g07qffJiFApuh5uthKyyEmZCooobA3_2IN3wTrU4U9E")]
    pub kind: String,
    /// Start of the activity.
    #[property(id = "1S8SePMq2pID-NdAP2uvaLbWxd3vYOk7XDEiVk7IpLQ")]
    pub started_at: Timestamp,
    /// End of the activity, when known. Pauses may fall within these bounds.
    #[active_type(LWW)]
    #[property(id = "twq-dRs3zzexHZ6JoB4n-tdQQtJXiHZj-7TrThDiLn8")]
    pub ended_at: Option<Timestamp>,
}
