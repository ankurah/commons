use ankurah::Model;
use serde::{Deserialize, Serialize};

/// Total distance covered during the activity or interval on this entity.
/// Indoor equipment may supply an equivalent distance.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "health.distance")]
#[model(id = "WBfBouIRGwozhLge5M5YCwLarOaPLr4ngTYr6IFafDo")]
#[model(no_ffi)]
pub struct Distance {
    #[property(id = "-_XFMjEA3xhBpOIqiv0OUIn9vmMqK2UK6BguEcnF36E")]
    pub meters: f64,
}
