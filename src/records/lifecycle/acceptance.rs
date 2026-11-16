use crate::identity::Principal;
use crate::property::Timestamp;
use crate::support::string_enum;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AcceptanceStatus {
    Proposed,
    Accepted,
    Rejected,
}
string_enum!(AcceptanceStatus);

/// Whether a record put forward for acceptance is accepted, and who answered.
/// Carried by a task, a decision, an edge or a submission put forward, and by
/// a ruling recorded as given, accepted at creation by whoever ruled. A
/// rejected record is also `Closed`. Never on an intention.
// OPEN: whether accepting an `assigned_to` edge records that the assignee
// undertakes the intention.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.acceptance")]
#[model(no_ffi)]
pub struct Acceptance {
    #[active_type(LWW)]
    pub status: AcceptanceStatus,
    /// Who answered. None while the record is proposed.
    #[active_type(LWW)]
    pub by: Option<Ref<Principal>>,
    /// When it was answered. None while the record is proposed.
    #[active_type(LWW)]
    pub answered: Option<Timestamp>,
}
