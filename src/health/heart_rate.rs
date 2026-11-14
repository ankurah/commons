use crate::identity::Principal;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// A heart rate observed for one person, including at rest or during recovery.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "health.heart_rate")]
#[model(no_ffi)]
pub struct HeartRate {
    /// The person whose heart rate was observed.
    pub measured_person: Ref<Principal>,
    /// Time of the observation in Unix milliseconds. For live readings without a
    /// sensor timestamp, the recorder uses the time it received the reading.
    pub observed_at: i64,
    /// A valid heart rate in beats per minute. Device codes for missing readings are omitted.
    pub beats_per_minute: f64,
}
