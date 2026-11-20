use ankurah::Model;
use serde::{Deserialize, Serialize};

/// A statement of something wanted, in words that belong to no one, so it
/// never carries `Attribution` or `Ownership`.
///
/// Who wrote it down is `Created`; its last rewording is `Edited`; how it
/// ended, fulfilled or withdrawn, is `Closed`. All are on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "cognition.intention")]
#[model(id = "W_PUyePLDVZm5ly-2sTyWjIHyu4zihZ5QWcCUGG-mHY")]
#[model(no_ffi)]
pub struct Intention {
    #[active_type(LWW)]
    #[property(id = "BPKD7qvFa6kENqdAWQc62bzM_3lnAnaLbpEUYIxJDVA")]
    pub title: String,
    #[active_type(LWW)]
    #[property(id = "iDl_GO55nXHHAgtTXOPJFECE_Luscg5XcT5LYRTyiv0")]
    pub description: String,
}
