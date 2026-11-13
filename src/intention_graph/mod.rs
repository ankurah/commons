//! The graph of what is wanted and the work done for it: intentions, the
//! claims they rest on, the decisions that read them and the tasks that serve
//! them.
//!
//! `task`, `decision`, `scope` and `link` are what Fwd Centaur has today, and
//! `submission` is what it keeps inside a task. `intention`, `claim` and
//! `reading` are new.
//!
//! What every kind of record shares is not repeated here: who made it, who
//! last edited it, whom it waits on and who closed it are components from
//! the `records` namespace, added to the same entity.

mod claim;
mod decision;
mod intention;
mod link;
mod reading;
mod scope;
mod submission;
mod task;

pub use claim::*;
pub use decision::*;
pub use intention::*;
pub use link::*;
pub use reading::*;
pub use scope::*;
pub use submission::*;
pub use task::*;
