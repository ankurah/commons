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
    pub measured_person: Ref<Principal>,
    /// The activity this window describes, including a recovery window after its end.
    pub activity: Option<Ref<PhysicalActivity>>,
    /// Start and end of the measurement window, [start, end).
    pub window_start: Timestamp,
    pub window_end: Timestamp,
    /// Number of valid readings included in the range, at least one.
    pub sample_count: i64,
    pub min_bpm: f64,
    pub max_bpm: f64,
}
