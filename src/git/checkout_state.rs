use crate::support::string_enum;
use ankurah::{property::Json, Model};
use serde::{Deserialize, Serialize};

/// Which version of the file the lines were marked in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum DiffSide {
    Base,
    Modified,
    Worktree,
}
string_enum!(DiffSide);

/// What the checkout held when an anchor was made, so that a machine without
/// the anchor's basis can rebuild it. Only on an anchor made in a checkout,
/// on the entity that carries its `FileAnchor`.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "git.checkout_state")]
#[model(id = "ma3v4p3sBurgqG2sLcBeRjdDleP8lAqJ5oh9C2nyW40")]
#[model(no_ffi)]
pub struct CheckoutState {
    #[active_type(LWW)]
    #[property(id = "bFSx-YH_kx435Uxm2_4mXi3d-9btDffCqt2UbSQFX1Y")]
    pub side: DiffSide,
    /// The commit checked out when the anchor was made.
    #[active_type(LWW)]
    #[property(id = "m2XbxstBaMWK0iMxNv5nqWtkJK0VNAUN4-ZsboZVjh0")]
    pub commit_sha: Option<String>,

    // The neighbouring versions of the file that the basis can be rebuilt from,
    // named by git's id of their content.
    #[active_type(LWW)]
    #[property(id = "oWDaJBa839JIVEKbXOYfOhlq1_PSvknngdzMDNmAp3A")]
    pub head_blob: Option<String>,
    #[active_type(LWW)]
    #[property(id = "8rOW5kJOvEfvxqla6ZqO-xFZlCvrUv6SPD-tydR9BZc")]
    pub index_blob: Option<String>,
    /// Whichever of `head_blob` and `index_blob` differs least from the basis.
    #[active_type(LWW)]
    #[property(id = "ise3sx0JlljxW44SQmWiZ5GyI26b0muKnzkq3sBCjc8")]
    pub nearest_blob: Option<String>,
    /// A unified diff from `nearest_blob` to the basis.
    #[active_type(LWW)]
    #[property(id = "mnn80diYJ3J0aqVGu6z_HEnMDv8az8DPMpnT4YUWTwA")]
    pub basis_patch: Option<String>,
    /// Runs of the lines that stood the same way against HEAD and the index
    /// when the anchor was made. An array of `{"basis_lines": [start, end],
    /// "head_lines": [start, end] or null, "index_lines": [start, end] or
    /// null}`, counted from zero with the end excluded.
    #[property(id = "ZBwXXXJKG8hZfNRhFaSpvlEKGnnRegDYMM9SMTANFZA")]
    pub pieces: Json,
}
