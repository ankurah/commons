use crate::graph::DynEdge;
use crate::intention_graph::{Decision, Scope, Task};
use crate::support::string_enum;
use crate::text::Fragment;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum MessageStatus {
    Active,
    Resolved,
    Deleted,
}
string_enum!(MessageStatus);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Priority {
    Now,
    Next,
    Later,
}
string_enum!(Priority);

/// Something one principal said, about a record or in reply to another message.
///
/// Who said it, and when, is `Created`: a message is its creator's own
/// words. A later edit is `Edited`; who resolved it is `Closed`. All are
/// on the same entity.
///
/// A message that asks something also carries `Assignment`: with an assignee
/// it asks that principal, and with none it asks no one in particular.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "messaging.message")]
#[model(no_ffi)]
pub struct Message {
    /// A record named in the text is written as `[[<its id>]]`, or
    /// `[[<its id>@<version>]]` for a highlight at one version of its file.
    #[active_type(YrsString)]
    pub content: String,
    #[active_type(LWW)]
    pub status: MessageStatus,
    #[active_type(LWW)]
    pub priority: Priority,
    pub reply_to: Option<Ref<Message>>,
    /// The first message of the thread this one belongs to.
    pub thread: Option<Ref<Message>>,

    // What the message is about; more than one may be set.
    // OPEN: four typed fields, as here, or one `about` that takes any record?
    // The four make this namespace depend on intention-graph and text, and
    // let one query find the open messages about a task.
    pub task: Option<Ref<Task>>,
    pub decision: Option<Ref<Decision>>,
    pub scope: Option<Ref<Scope>>,
    /// The highlight's entity, which carries `Fragment` and an anchor.
    pub highlight: Option<Ref<Fragment>>,

    /// The scope its author was working in when writing it.
    pub written_in: Option<Ref<Scope>>,
    /// For a question that names no one: the scope whose triage agent answers
    /// it. None means the agent that serves the whole project.
    pub asks_scope: Option<Ref<Scope>>,
    /// Set on the message that gives the reason a task or a decision moved:
    /// which move, such as `returned`, `reopened` or `repealed`.
    #[active_type(LWW)]
    pub transition: Option<String>,
    /// Set on the message written as the first comment of a checklist step:
    /// that step's `step_of` edge.
    pub first_of_step: Option<Ref<DynEdge>>,
}
