//! What any record can say about its own life: its course from creation to
//! closing, whose words it holds and what it was made from, whom it waits on
//! and who owns it, a note on it, when it can be acted on, and its place among
//! its siblings.
//!
//! Each model here is a component added to a record's entity, beside the
//! model that says what the record is, such as a task or a message. Kept
//! apart so that every kind of record says these things the same way.

mod deferred;
mod fractional_position;
mod lifecycle;
mod note;
mod provenance;
mod responsibility;

pub use deferred::*;
pub use fractional_position::*;
pub use lifecycle::*;
pub use note::*;
pub use provenance::*;
pub use responsibility::*;
