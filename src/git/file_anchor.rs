use ankurah::{property::Ref, Model};
use serde::{Deserialize, Serialize};

use super::Repository;

/// Where a highlight sits in a file kept in git: a run of lines in one version
/// of the file. Added to an entity that also carries `Fragment`, from the
/// `text` namespace, and `CheckoutState` when the anchor was made in a
/// checkout.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "git.file_anchor")]
#[model(no_ffi)]
pub struct FileAnchor {
    pub repository: Ref<Repository>,
    /// Relative to the repository.
    #[active_type(LWW)]
    pub path: String,
    /// The version of the file the lines count in, named by git's id of its
    /// content.
    #[active_type(LWW)]
    pub basis_blob: Option<String>,
    /// The first line marked, counted from zero; 0 when the whole file is.
    pub start_line: i64,
    /// The line after the last one marked; 0 when the whole file is.
    pub end_line: i64,
    /// True when the whole file is marked, and no run of its lines.
    pub whole_file: bool,
}
