use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

use super::Repository;

/// Where a highlight sits in a file kept in git: a run of lines in one version
/// of the file. Added to an entity that also carries `Fragment`, from the
/// `text` namespace, and `CheckoutState` when the anchor was made in a
/// checkout.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "git.file_anchor")]
#[model(id = "V1NIGvz1uewWPssVaOiWxlTHjV0RrImDwFzmOB6ueHw")]
#[model(no_ffi)]
pub struct FileAnchor {
    #[property(id = "JGHdmb2QlSnN3m8im4prDaN-6nuxa3co7U4VKVICzwg")]
    pub repository: Ref<Repository>,
    /// Relative to the repository.
    #[active_type(LWW)]
    #[property(id = "lG225xiGZhzvKzMpi3q0y9idSJByikqa4ZpUFvuR5H4")]
    pub path: String,
    /// The version of the file the lines count in, named by git's id of its
    /// content.
    #[active_type(LWW)]
    #[property(id = "Z4qfvgH0rG2p4PkAL6CsuJXqrzKMKSk7vXinFsvEQos")]
    pub basis_blob: Option<String>,
    /// The first line marked, counted from zero; 0 when the whole file is.
    #[property(id = "iTVRnh4Vy2hSZXtiGeJKXInLlhN4wxkvR8pLSE3w-Ko")]
    pub start_line: i64,
    /// The line after the last one marked; 0 when the whole file is.
    #[property(id = "p3cxT3EgTJUHLbB272Va3st2u97zMXQK9A4idnvJcT0")]
    pub end_line: i64,
    /// True when the whole file is marked, and no run of its lines.
    #[property(id = "s020MccN2-1qqZN4dY2NxysLctzBTdb7N3poLdCPtck")]
    pub whole_file: bool,
}
