use crate::support::string_enum;
use ankurah::{property::Json, EntityId, Model};
use serde::{Deserialize, Serialize};

/// What a link says, read "from, kind, to".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum LinkKind {
    // What Fwd Centaur has today.
    /// Task, task: the first cannot start until the second is done.
    DependsOn,
    /// Task, task.
    SubtaskOf,
    /// Task, task or message.
    MotivatedBy,
    /// Task, task or message.
    DiscoveredFrom,
    /// Anything, anything.
    RelatesTo,
    /// Task, task; decision, decision; principal, principal: the first
    /// replaces the second.
    // Fwd Centaur keeps a principal's successor as a field on the principal.
    Supersedes,
    /// Task, task.
    Duplicates,
    /// Task, highlight.
    AnchoredAt,
    /// Message, task, link or scope.
    Concerns,
    /// Task, message or decision, decision; decision, task or message.
    Cites,
    /// Task, scope: where the task is worked.
    WorkedIn,
    /// Anything, task: a step of the task's checklist, ordered by `position`.
    StepOf,
    /// Message, anything: the message's text names the record. The link's
    /// entity also carries `Mention`, from the `messaging` namespace.
    Mention,

    // New, for intentions.
    /// Task or intention, intention: the work, or the narrower aim, is for the
    /// broader one.
    Serves,
    /// Intention or decision, claim: the want or the choice assumes the claim.
    RestsOn,
    /// Principal, intention: an agent names what governs its current work.
    WorksUnder,
}
string_enum!(LinkKind);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum LinkStatus {
    Proposed,
    Accepted,
    Retracted,
    Rejected,
    Resolved,
}
string_enum!(LinkStatus);

/// A stated connection from one record to another. Who stated it is
/// `Creation`, and how it was retracted or rejected is `Closure`, both on the
/// same entity.
// OPEN: one model with a `kind`, as here, or one model for each kind?
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "intention-graph.link")]
#[model(no_ffi)]
pub struct Link {
    #[active_type(LWW)]
    pub kind: LinkKind,
    /// Named `from_id` and `to_id` because `from` is a reserved word in a
    /// query. Plain ids, because an end may be a record of any kind.
    pub from_id: EntityId,
    pub to_id: EntityId,
    #[active_type(LWW)]
    pub status: LinkStatus,
    #[active_type(LWW)]
    pub note: Option<String>,
    /// A step's place in its task's checklist: a key compared as text, chosen
    /// so that a step can go between two others without renumbering them.
    /// None on every other kind.
    #[active_type(LWW)]
    pub position: Option<String>,
    /// What caused the link to be made: `{"message_id": ..}` for a message,
    /// `{"actor_id": ..}` for a principal acting directly, and `"migrated"`
    /// beside it when a migration made the link from older data.
    // OPEN: JSON today. Who made the link is `Creation` alone, so `actor_id`
    // goes; `message_id` and `migrated` want real properties. A mention's
    // place in the text and whether an edit added it are `Mention`'s, and are
    // not kept here as well.
    pub via: Json,
}
