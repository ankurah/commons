use crate::support::string_enum;
use ankurah::{
    property::{Json, Ref},
    Model,
};
use serde::{Deserialize, Serialize};

use super::Repository;

/// Which version of the file the lines were marked in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum DiffSide {
    Base,
    Modified,
    Worktree,
}
string_enum!(DiffSide);

/// Where a highlight sits in a file kept in git: a run of lines in one version
/// of the file. Added to an entity that also carries `Fragment`, from the
/// `highlights` namespace.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "git.file_anchor")]
pub struct FileAnchor {
    pub repository: Ref<Repository>,
    /// Relative to the repository.
    #[active_type(LWW)]
    pub path: String,
    #[active_type(LWW)]
    pub side: DiffSide,
    pub start_line: i64,
    pub end_line: i64,
    pub whole_file: Option<bool>,
    /// A path to the marked symbol in the file's syntax tree. Fwd Centaur
    /// declares it and does not write it yet.
    #[active_type(LWW)]
    pub symbol_path: Option<String>,

    // The version of the file the lines count in (the basis), named by git's
    // id of its content, and the neighbouring versions it can be rebuilt from.
    #[active_type(LWW)]
    pub basis_blob: Option<String>,
    #[active_type(LWW)]
    pub head_blob: Option<String>,
    #[active_type(LWW)]
    pub index_blob: Option<String>,
    /// Whichever of `head_blob` and `index_blob` differs least from the basis.
    #[active_type(LWW)]
    pub nearest_blob: Option<String>,
    /// A unified diff from `nearest_blob` to the basis, so that a machine
    /// without the basis can rebuild it.
    #[active_type(LWW)]
    pub basis_patch: Option<String>,
    /// The basis of anchors made before `basis_blob` existed. Read, never
    /// written now.
    #[active_type(LWW)]
    pub blob_sha: Option<String>,
    /// The commit checked out when the anchor was made.
    #[active_type(LWW)]
    pub commit_sha: Option<String>,
    /// Runs of the lines that stood the same way against HEAD and the index
    /// when the anchor was made. An array of `{"basis_lines": [start, end],
    /// "head_lines": [start, end] or null, "index_lines": [start, end] or
    /// null}`, counted from zero with the end excluded.
    pub pieces: Json,

    /// A hash of the repository, the path, the basis and the line range, so
    /// that making the same anchor twice finds the first.
    // OPEN: computed from the properties above. Should a computed value be a
    // published property?
    #[active_type(LWW)]
    pub anchor_key: Option<String>,
}
