use ankurah::Model;
use serde::{Deserialize, Serialize};

/// Where a record stands among its siblings, such as a step in a task's
/// checklist, as a key that sorts between its neighbours' keys, so that a
/// record can be put between two others without renumbering any.
///
/// The key is a fraction written in base-62 digits after the point, and keys
/// compare as text. The digits are `0` to `9`, `A` to `Z` and `a` to `z`, in
/// that order, which is also their order as text, and no key ends in `0`, so
/// that comparing two keys as text compares their fractions. Records with the
/// same key stand in the order of their entity ids.
// OPEN: whether to name it DivisiblePosition instead.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.fractional_position")]
#[model(id = "dj9ZT3_8j6ryDe-JoSnrhkGdgOGoevly5WJIzkQ5rAw")]
#[model(no_ffi)]
pub struct FractionalPosition {
    #[active_type(LWW)]
    #[property(id = "LpXXe33UW5ZuhLTzFdRGS0tsB2_nO8CswNNo77jy82U")]
    pub key: String,
}
