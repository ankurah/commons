use ankurah::Model;
use serde::{Deserialize, Serialize};

/// Total distance covered during the activity or interval on this entity.
/// Indoor equipment may supply an equivalent distance.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "health.distance")]
pub struct Distance {
    pub meters: f64,
}
