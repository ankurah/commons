use super::PhysicalActivity;
use crate::identity::Principal;
use crate::property::Timestamp;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// The observed heart rate range for a person within one wall-clock window.
/// For a workout and its recovery, use separate summaries for the two windows.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "health.heart_rate_summary")]
#[model(id = "ABJk8PXGYqZtdHOInmA-bpSzSxqf_rWGp0MvucPqa6k")]
#[model(no_ffi)]
pub struct HeartRateSummary {
    #[property(id = "ovxQM9oCKZXve92OOMZm9nZxv5dwWHXTs__fsIEfWec")]
    pub measured_person: Ref<Principal>,
    /// The activity this window describes, including a recovery window after its end.
    #[property(id = "OuWZdE6-vjtaXVeFcADelRnzRB9HO92270zuR22xweM")]
    pub activity: Option<Ref<PhysicalActivity>>,
    /// Start and end of the measurement window, [start, end).
    #[property(id = "Br5NJjrLWE_LB8_kE_A-2E7awF8rJQpb6QhZbzenZ3g")]
    pub window_start: Timestamp,
    #[property(id = "UU74poOAnWXTIYz7xsDQGojx0HVgClH4JNE3egCvhro")]
    pub window_end: Timestamp,
    /// Number of valid readings included in the range, at least one.
    #[property(id = "ug1V-DH6oVgir1GQQSVKwQXQsOHq0nUKB9_fk_gxHrk")]
    pub sample_count: i64,
    #[property(id = "eGttzYnezEWtmYw4YSWAqSQNWdGqwGL1Vfe-9crTIgw")]
    pub min_bpm: f64,
    #[property(id = "68fkGWZswuO6Ma0iv0R7bSjOlsIn2-cCcrIkj3pI0Sg")]
    pub max_bpm: f64,
}
