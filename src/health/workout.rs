use ankurah::Model;
use serde::{Deserialize, Serialize};

/// Classifies the physical activity on this entity as an exercise session.
/// Its performer and times come from PhysicalActivity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "health.workout")]
#[model(id = "k68MpVbs8iiCtqo4_SlGSvFl4QK6kQuF9ZKJUym5AN8")]
#[model(no_ffi)]
pub struct Workout {}
