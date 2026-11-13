//! What any record can say about its own life: who made it, whose words it
//! holds, who last edited it, whom it waits on, who closed it, and who owns it.
//!
//! Each model here is a component added to a record's entity, beside the
//! model that says what the record is, such as a task or a message. Kept
//! apart so that every kind of record says these things the same way.

mod assignment;
mod attribution;
mod closure;
mod creation;
mod last_edit;
mod ownership;

pub use assignment::*;
pub use attribution::*;
pub use closure::*;
pub use creation::*;
pub use last_edit::*;
pub use ownership::*;
