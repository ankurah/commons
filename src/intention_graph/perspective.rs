use crate::property::DynRef;
use crate::support::string_enum;
use ankurah::Model;
use serde::{Deserialize, Serialize};

/// Where a party stands on the record its perspective is about.
// OPEN: these values are a draft, to settle with the design of perspectives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Stance {
    Agree,
    Disagree,
    Question,
}
string_enum!(Stance);

/// One party's account, in its own words, of how it understands a record.
///
/// Who wrote it down, and when, is `Created`; whose perspective it is, when
/// someone else wrote it down, is `Attribution`; its last rewording is
/// `Edited`. All are on the same entity.
///
/// A perspective written before the record's last edit was written about an
/// earlier wording. Both times come from clocks, so they do not prove which
/// wording it was written about.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "intention-graph.perspective")]
#[model(no_ffi)]
pub struct Perspective {
    /// The intention, claim, decision or perspective it is about.
    pub of: DynRef,
    #[active_type(LWW)]
    pub words: String,
    #[active_type(LWW)]
    pub stance: Option<Stance>,
}
