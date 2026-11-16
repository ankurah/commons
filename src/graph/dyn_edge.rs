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
#[model(no_ffi)]
pub struct DynEdge {
    pub from: DynRef,
    pub to: DynRef,
    /// What the connection means.
    pub kind: Ref<EdgeKind>,
}
