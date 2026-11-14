use crate::support::string_enum;
use ankurah::{EntityId, Model};
use serde::{Deserialize, Serialize};

/// Where a party stands on what it read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Stance {
    Agree,
    Disagree,
    Question,
}
string_enum!(Stance);

/// One party's account, in its own words, of how it understands a record and
/// what it will do under it.
///
/// Who wrote it down, and when, is `Creation`; whose account it is, when
/// someone else wrote it down, is `Attribution`; its last rewording is
/// `LastEdit`. All are on the same entity.
///
/// A reading written before the record's own last edit is a prompt to read
/// again. The two times come from clocks, so this does not prove which words
/// were read.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "intention-graph.reading")]
#[model(no_ffi)]
pub struct Reading {
    /// The intention, claim or decision read.
    pub of: EntityId,
    #[active_type(LWW)]
    pub words: String,
    #[active_type(LWW)]
    pub stance: Option<Stance>,
}

// OPEN, not modelled yet: what an agent's session declares. `WorksUnder` names
// the intentions; what they imply for its next action, and where its authority
// ends, have no home yet. A reading could carry them.
