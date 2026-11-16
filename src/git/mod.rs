//! Files and work kept in git: the repositories, where a marked part of a file
//! sits and what the checkout held when it was marked, and where a body of
//! work happens.

mod checkout_state;
mod file_anchor;
mod repository;
mod work_location;

pub use checkout_state::*;
pub use file_anchor::*;
pub use repository::*;
pub use work_location::*;
