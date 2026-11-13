//! Files and work kept in git: the repositories, where a marked part of a file
//! sits, and where a body of work happens.

mod file_anchor;
mod repository;
mod work_location;

pub use file_anchor::*;
pub use repository::*;
pub use work_location::*;
