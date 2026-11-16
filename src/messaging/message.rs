use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// Something one principal said, perhaps in reply to another message.
///
/// Who said it, and when, is `Created`: a message is its creator's own
/// words. A later edit is `Edited`; its resolution or deletion is `Closed`.
/// All are on the same entity. What it is about arrives by `concerns` edges.
///
/// A message that asks something also carries `Assignment`: with an assignee
/// it asks that principal, and with none it asks no one in particular.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "messaging.message")]
#[model(no_ffi)]
pub struct Message {
    #[active_type(YrsString)]
    pub content: String,
    pub reply_to: Option<Ref<Message>>,
    /// The first message of the thread this one belongs to.
    pub thread: Option<Ref<Message>>,
}
