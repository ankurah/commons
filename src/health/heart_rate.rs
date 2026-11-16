use crate::identity::Principal;
use crate::property::Timestamp;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// A heart rate observed for one person, including at rest or during recovery.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "health.heart_rate")]
#[model(id = "msX5so6J9VKpdQ_80j7dgIpzNn6n78WKafft5qOfccI")]
#[model(no_ffi)]
pub struct HeartRate {
    /// The person whose heart rate was observed.
    pub measured_person: Ref<Principal>,
    /// Time of the observation. For live readings without a sensor timestamp,
    /// the recorder uses the time it received the reading.
    pub observed_at: Timestamp,
    /// A valid heart rate in beats per minute. Device codes for missing readings are omitted.
    pub beats_per_minute: f64,
}
