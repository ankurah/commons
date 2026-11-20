use super::PhysicalActivity;
use crate::property::Timestamp;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// A named portion of one physical activity, such as a work interval, rest, or lap.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "health.activity_interval")]
#[model(id = "2Dw86UNp_L9Ad2m2A0DQkq0iL8iMmkpc5FBTBll3Yy8")]
#[model(no_ffi)]
pub struct ActivityInterval {
    #[property(id = "_sy6U2csjTQuZPlJA0FPOlS0jhixLYUD7rxvf2Y9LgI")]
    pub activity: Ref<PhysicalActivity>,
    /// An open name in snake case, such as work, rest, or lap.
    #[active_type(LWW)]
    #[property(id = "S_BqUaPLwNn-x6_XeMP8GvEHYS7nG6LAp0bIaxwVpSE")]
    pub kind: String,
    /// Start, inclusive.
    #[property(id = "rUGrovUAF7qVuGuU1Vgev_fowa9qKIxYHf3XW4Lj68k")]
    pub started_at: Timestamp,
    /// End, exclusive, when known.
    #[active_type(LWW)]
    #[property(id = "8r9hmQ1kkKu8Q6kIZvNP7RTeufbAj8wSXwlY_8n7hSE")]
    pub ended_at: Option<Timestamp>,
}
