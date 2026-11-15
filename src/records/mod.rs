//! What any record can say about its own life: who made it, whose words it
//! holds, who last edited it, whom it waits on, who closed it, who owns it,
//! what caused it, a note on it, and its place among its siblings.
//!
//! Each model here is a component added to a record's entity, beside the
//! model that says what the record is, such as a task or a message. Kept
//! apart so that every kind of record says these things the same way.

// TODO: `by` in `Created`, `Edited`, `Closed` and `Checked` generic over the
// model it names, defaulting to `Principal`, once Ankurah's model derive
// takes type parameters.

mod assignment;
mod attribution;
mod checked;
mod closed;
mod created;
mod edited;
mod note;
mod ownership;
mod relative_position;
mod via;

pub use assignment::*;
pub use attribution::*;
pub use checked::*;
pub use closed::*;
pub use created::*;
pub use edited::*;
pub use note::*;
pub use ownership::*;
pub use relative_position::*;
pub use via::*;
