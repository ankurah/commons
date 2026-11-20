use crate::identity::Principal;
use crate::property::Timestamp;
use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

/// Who made a record, and when. Attached when the record is made.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.created")]
#[model(id = "b8gFJFUqwCgbg08XyEcfAszpwIFjhvLS-g-KShvrfBE")]
#[model(no_ffi)]
pub struct Created {
    #[property(id = "FvESAOcLp-fmWQOHamNm4h3fY5gobddxkTmapyqD2Fw")]
    pub created: Timestamp,
    #[property(id = "c6pglo2jowz4GSblapVg9Lwz6309q0PiNsw64oAWP3s")]
    pub by: Ref<Principal>,
}
