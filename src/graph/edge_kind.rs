use ankurah::Model;
use serde::{Deserialize, Serialize};

/// What a kind of edge means, as a record of its own, so that a kind is named
/// and described once and every edge of it refers to it by id. Every node must
/// hold the same record for a kind, so kinds are minted, as model definitions
/// are, rather than made by each node.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "graph.edge_kind")]
#[model(no_ffi)]
pub struct EdgeKind {
    /// In snake case, such as `depends_on`.
    #[active_type(LWW)]
    pub name: String,
    #[active_type(LWW)]
    pub description: String,
}
