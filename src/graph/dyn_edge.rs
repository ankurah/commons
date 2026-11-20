use super::EdgeKind;
use crate::property::DynRef;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// A connection from one record to another, of the kind an `EdgeKind` record
/// names: a task that depends on another, a decision that cites one, an
/// intention assigned to a principal.
///
/// Who stated it is `Created`, and what it was made from `Via`; whether it was
/// accepted, when it was put forward, is `Acceptance`; a remark on it is
/// `Note`, and its place among sibling steps `FractionalPosition`. Its
/// retraction is `Closed`. All are on the same entity.
// TODO: removable, so that retracting an edge takes it off, once Ankurah can
// remove a component from an entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "graph.dyn_edge")]
#[model(id = "wY_SwOM6bWYB1vBXl7C1NfYzD_dI09LwL0-FPXB-vp4")]
#[model(no_ffi)]
pub struct DynEdge {
    #[property(id = "wKqDbJxQ8HnB648YqVLVGQTz2KFi8UkTluEPx6NMF9Q")]
    pub from: DynRef,
    #[property(id = "1q8z17hjtThFrdsxRmftMMjkghnPcC17IsZLosmpcP0")]
    pub to: DynRef,
    /// What the connection means.
    #[property(id = "AM6khFY5gCeYKwfho882j6c8asrno5Br7uGDxPD1V1A")]
    pub kind: Ref<EdgeKind>,
}
