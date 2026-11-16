//! A record's course: who made it and when, who last edited it, whether it
//! was accepted, whether it is checked off, and who closed it and how it
//! ended.

// TODO: `by` in `Created`, `Edited`, `Closed`, `Checked` and `Acceptance`
// generic over the model it names, defaulting to `Principal`, once Ankurah's
// model derive takes type parameters.

mod acceptance;
mod checked;
mod closed;
mod created;
mod edited;

pub use acceptance::*;
pub use checked::*;
pub use closed::*;
pub use created::*;
pub use edited::*;
