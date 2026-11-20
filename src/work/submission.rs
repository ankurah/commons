use crate::property::DynRef;
use ankurah::Model;
use serde::{Deserialize, Serialize};

/// Work handed in as finished, with what shows it is finished. Each hand-in is
/// a record of its own, so that work handed in, returned and handed in again
/// keeps every hand-in with its answer.
///
/// Who handed it in, and when, is `Created`; the answer is `Acceptance`; its
/// return is `Closed`. All are on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "work.submission")]
#[model(id = "vc9YgO8iHlsyARnnjQgKWwTQMbAjaQaoHHOKnHGYMFI")]
#[model(no_ffi)]
pub struct Submission {
    /// The work handed in, such as a task.
    #[property(id = "d5NMYDDAtPXWYAQktcNtSSubJ0YTxkshgfgOHR9cCCQ")]
    pub of: DynRef,
    /// The commits, test runs and the like that show the work is done.
    #[active_type(LWW)]
    #[property(id = "V23FeeQTh0UT5en2HGOV5lYIGu6ZXBAnmSPgCisvinI")]
    pub evidence: String,
}
