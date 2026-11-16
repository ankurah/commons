use crate::property::Timestamp;
use ankurah::Model;
use serde::{Deserialize, Serialize};

/// A forecast: what someone expects, how and when it will be checked, and what
/// would count against it, written before the outcome.
///
/// Each forecast is an entity of its own, linked to what it is about by a
/// `concerns` edge. Its outcome is a `Claim` that `supports` or `disputes` it,
/// and it ends through `Closed`: held, failed or inconclusive.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "cognition.expectation")]
#[model(id = "G08Dtf2xjJUfP8tCTnvtbJmox5B8tAhkxPqf58Ghxvs")]
#[model(no_ffi)]
pub struct Expectation {
    #[active_type(LWW)]
    pub expects: String,
    /// How it will be checked.
    #[active_type(LWW)]
    pub check: String,
    /// When it will be checked, where set.
    #[active_type(LWW)]
    pub due: Option<Timestamp>,
    /// What would count against it.
    #[active_type(LWW)]
    pub counts_against: String,
    /// How likely it is to hold, as a whole percentage, where given.
    #[active_type(LWW)]
    pub probability: Option<i64>,
}
