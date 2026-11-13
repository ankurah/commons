use ankurah::Model;
use serde::{Deserialize, Serialize};

/// Classifies the physical activity on this entity as an exercise session.
/// Its performer and times come from PhysicalActivity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "health.workout")]
pub struct Workout {}
