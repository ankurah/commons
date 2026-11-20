use ankurah::Model;
use serde::{Deserialize, Serialize};

/// What a kind of edge means, as a record of its own, so that a kind is named
/// and described once and every edge of it refers to it by id. Every node must
/// hold the same record for a kind, so kinds are minted, as model definitions
/// are, rather than made by each node.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "graph.edge_kind")]
#[model(id = "yDVJXmGUhzHmue289YFApbB2-pPt8SiCG2NhiHy_zco")]
#[model(no_ffi)]
pub struct EdgeKind {
    /// In snake case, such as `depends_on`.
    #[active_type(LWW)]
    #[property(id = "so_9VblgkIP3qLuG-Bo4doL6exXukhP0SGHyDqgKeZ8")]
    pub name: String,
    #[active_type(LWW)]
    #[property(id = "sN0v89SyyA4FGQBZBU4uwV_p1BqPGCFEfMH-LNIt_kI")]
    pub description: String,
}

/// The kinds of edge the Commons mints, as `EdgeKind` records: each kind's
/// name, and its description, which says which way an edge of it runs and
/// what its ends may be.
pub const EDGE_KINDS: &[(&str, &str)] = &[
    (
        "serves",
        "From a task, a scope or an intention to an intention: the first is for the sake of the second. Many to \
         many; no cycles among intentions.",
    ),
    ("interprets", "From a decision to an intention: the decision reads the intention in a context."),
    (
        "supports",
        "From a claim, a quotation, a fragment or an intuition to a claim, an intention, a decision, an \
         intuition, an expectation or an edge: a reason to accept, want or rely on the second.",
    ),
    ("disputes", "From a claim to a claim, an expectation or an edge: a reason against the second. Never to a perspective."),
    ("cites", "From any record to any record: the first draws on the second, rests on it as a ruling, or was made in its context."),
    ("supersedes", "From a record to an earlier record of its kind: the first replaces the second, which stays readable."),
    (
        "assigned_to",
        "From an intention to a principal: the principal works toward the intention; several principals may each \
         hold one. An Acceptance on the edge records whether the principal took it up.",
    ),
    ("depends_on", "From a task to a task: the first is not ready until the second is done."),
    ("subtask_of", "From a task to a task: the first is part of the second."),
    ("worked_in", "From a task to a scope: the task's work happens in the scope."),
    ("step_of", "From any record to a task: the first is a step of the task's checklist."),
    ("duplicates", "From a task to a task: both are the same work."),
    ("discovered_from", "From a record to the record it surfaced from, or whose words it was made from, such as a message."),
    ("generated_by", "From a record to the activity that made it, such as an import run."),
    ("concerns", "From any record to any record: the first is about the second."),
    ("mention", "From a message to any record: the message's text names the record."),
    ("anchored_at", "From a task to a fragment: the text or code the task is about."),
    ("relates_to", "From any record to any record: an association not yet given a kind."),
    ("governs", "From a decision to a scope: the decision governs the scope's work."),
    (
        "stakeholder_of",
        "From a principal to an intention: the principal's perspective on the intention is sought, and it hears \
         of changes to it. It gives no power over the intention's words.",
    ),
];
