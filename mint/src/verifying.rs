//! Checks a directory `mint` wrote. In order, and stopping at the first entry that fails:
//!
//! - the directory holds only directories of definition files and each such directory's
//!   page, which `verify` does not read, and every entry read is of the type its role
//!   needs, checked without following a symlink before it is opened;
//! - each directory holds, for each definition, exactly an event file and a state file
//!   named by the definition's entity id;
//! - every event file decodes with nothing left over, and holds a well-formed event that
//!   creates that entity;
//! - the event carries no attestation, since this tool checks none;
//! - the entity it creates is a definition, a member of exactly one of Ankurah's catalog
//!   models;
//! - the state file decodes with nothing left over, and holds the state of that entity
//!   whose head is that creation event alone;
//! - no definition appears twice.
//!
//! It does not check that a state's values are what its event gives: nothing public in
//! Ankurah can derive a state from an event without a node.

use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use ankurah::proto::{
    Attested, Clock, DecodeError, EntityId, Event, EventStructureError, Membership, ModelId, SystemModel, UndecodableBytes,
    UnreadableOperation,
};
use thiserror::Error;

use crate::{
    state_encoding,
    writing::{page_file, EVENT_EXTENSION, STATE_EXTENSION},
};

#[cfg(test)]
mod tests;

/// The first entry that failed, and why.
#[derive(Debug, Error)]
#[error("{}: {failure}", path.display())]
pub struct VerifyError {
    pub path: PathBuf,
    pub failure: Failure,
}

/// Why an entry fails verification.
#[derive(Debug, Error)]
pub enum Failure {
    #[error("cannot read it: {0}")]
    Unreadable(io::Error),
    #[error("is a {found}, where a {expected} belongs")]
    WrongType { expected: EntryType, found: EntryType },
    #[error("is a stray {0}: not a directory of definition files, or the page of one")]
    Stray(EntryType),
    #[error("is not a definition's file: its name does not end in `.{EVENT_EXTENSION}` or `.{STATE_EXTENSION}`")]
    NotADefinitionFile,
    #[error("is not a definition's file: its name does not begin with an entity id ({0})")]
    NameNotAnId(DecodeError),
    #[error("is a definition's state file with no event file beside it")]
    NoEvent,
    #[error("is a definition's event file with no state file beside it")]
    NoState,
    #[error("does not decode as an event with its attestations: {0}")]
    Undecodable(UndecodableBytes),
    #[error("holds a malformed event: {0}")]
    Malformed(EventStructureError),
    #[error("holds an event that creates no entity")]
    NotCreation,
    #[error("holds the creation event of {0}, not of the definition the file is named for")]
    OtherEntity(EntityId),
    #[error("carries {0} attestations; this tool checks none, so it refuses any rather than pass one unchecked")]
    Attested(usize),
    #[error("holds an event whose operations cannot be read: {0}")]
    UnreadableOperations(UnreadableOperation),
    #[error("holds an event that makes its entity a member of [{}], not of exactly one of the catalog's models", joined(.0))]
    NotADefinition(Vec<ModelId>),
    #[error("does not decode as a state: {0}")]
    UndecodableState(bincode::Error),
    #[error("holds the state of {0}, not of the definition the file is named for")]
    StateOfOtherEntity(EntityId),
    #[error("holds a state whose head is not the definition's creation event alone")]
    HeadNotCreation,
    #[error("holds the definition that {} holds too", .first.display())]
    InTwoPlaces { first: PathBuf },
}

/// What an entry of the directory is, read without following a symlink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Directory,
    File,
    Symlink,
    /// A FIFO, a socket or a device.
    Special,
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Directory => "directory",
            Self::File => "regular file",
            Self::Symlink => "symlink",
            Self::Special => "special file",
        })
    }
}

/// What `verify` checked.
#[derive(Debug)]
pub struct Verified {
    pub directories: usize,
    pub definitions: usize,
}

/// The definitions' files in one directory: their event files and their state files, each
/// by the definition's id.
#[derive(Default)]
struct DefinitionFiles {
    events: BTreeMap<EntityId, PathBuf>,
    states: BTreeMap<EntityId, PathBuf>,
}

/// Checks the directory `dir`.
pub fn verify(dir: &Path) -> Result<Verified, VerifyError> {
    let directories = definition_directories(dir)?;
    let mut found: BTreeMap<EntityId, PathBuf> = BTreeMap::new();
    for directory in &directories {
        let DefinitionFiles { events, states } = definition_files(directory)?;
        let failed = |path: &Path, failure| VerifyError { path: path.to_owned(), failure };
        if let Some(state) = states.iter().find_map(|(id, state)| (!events.contains_key(id)).then_some(state)) {
            return Err(failed(state, Failure::NoEvent));
        }
        for (id, event) in events {
            let creation = check_event_file(&event, id).map_err(|failure| failed(&event, failure))?;
            let Some(state) = states.get(&id) else {
                return Err(failed(&event, Failure::NoState));
            };
            check_state_file(state, id, &creation).map_err(|failure| failed(state, failure))?;
            if let Some(first) = found.get(&id) {
                return Err(failed(&event, Failure::InTwoPlaces { first: first.clone() }));
            }
            found.insert(id, event);
        }
    }
    Ok(Verified { directories: directories.len(), definitions: found.len() })
}

/// Requires the entry at `path` to be `expected`, read without following a symlink or
/// opening the entry, so that every entry `verify` reads lies inside the directory and a
/// FIFO is never waited on.
fn require(path: &Path, expected: EntryType) -> Result<(), Failure> {
    let found = entry_type(path).map_err(Failure::Unreadable)?;
    if found == expected {
        Ok(())
    } else {
        Err(Failure::WrongType { expected, found })
    }
}

fn entry_type(path: &Path) -> io::Result<EntryType> {
    let file_type = fs::symlink_metadata(path)?.file_type();
    Ok(if file_type.is_dir() {
        EntryType::Directory
    } else if file_type.is_file() {
        EntryType::File
    } else if file_type.is_symlink() {
        EntryType::Symlink
    } else {
        EntryType::Special
    })
}

/// The directories of definition files in `dir`, once every other entry is the page of
/// one of those directories.
fn definition_directories(dir: &Path) -> Result<Vec<PathBuf>, VerifyError> {
    let mut directories = Vec::new();
    let mut others = Vec::new();
    for entry in entries(dir)? {
        let path = entry.path();
        match entry_type(&path) {
            Ok(EntryType::Directory) => directories.push(path),
            Ok(found) => others.push((path, found)),
            Err(source) => return Err(VerifyError { path, failure: Failure::Unreadable(source) }),
        }
    }
    let pages: Vec<String> = directories.iter().filter_map(|directory| directory.file_name()?.to_str()).map(page_file).collect();
    let is_page = |path: &Path| path.file_name().and_then(OsStr::to_str).is_some_and(|name| pages.iter().any(|page| page == name));
    match others.into_iter().find(|(path, _)| !is_page(path)) {
        Some((path, found)) => Err(VerifyError { path, failure: Failure::Stray(found) }),
        None => Ok(directories),
    }
}

/// The entries of `dir`, sorted by name.
fn entries(dir: &Path) -> Result<Vec<fs::DirEntry>, VerifyError> {
    let unreadable = |source| VerifyError { path: dir.to_owned(), failure: Failure::Unreadable(source) };
    let mut entries = fs::read_dir(dir).map_err(unreadable)?.collect::<Result<Vec<_>, _>>().map_err(unreadable)?;
    entries.sort_by_key(fs::DirEntry::file_name);
    Ok(entries)
}

/// The definitions' files in `directory`, once every entry there is named by an entity id
/// and `.event` or `.state`.
fn definition_files(directory: &Path) -> Result<DefinitionFiles, VerifyError> {
    let mut files = DefinitionFiles::default();
    for entry in entries(directory)? {
        let path = entry.path();
        let failed = |failure| VerifyError { path: path.clone(), failure };
        let name = entry.file_name();
        let named = name.to_str().and_then(|name| name.rsplit_once('.'));
        let Some((stem, extension)) = named.filter(|(_, extension)| [EVENT_EXTENSION, STATE_EXTENSION].contains(extension)) else {
            return Err(failed(Failure::NotADefinitionFile));
        };
        let id = EntityId::from_base64(stem).map_err(|source| failed(Failure::NameNotAnId(source)))?;
        let by_id = if extension == EVENT_EXTENSION { &mut files.events } else { &mut files.states };
        by_id.insert(id, path);
    }
    Ok(files)
}

/// The creation event of the definition `id` that the file at `path` holds, once it
/// checks out.
fn check_event_file(path: &Path, id: EntityId) -> Result<Event, Failure> {
    require(path, EntryType::File)?;
    let bytes = fs::read(path).map_err(Failure::Unreadable)?;
    let Attested { payload: event, attestations } = Attested::<Event>::decode(&bytes).map_err(Failure::Undecodable)?;
    event.validate_structure().map_err(Failure::Malformed)?;
    if !event.is_entity_create() {
        return Err(Failure::NotCreation);
    }
    let created = EntityId::from(event.id());
    if created != id {
        return Err(Failure::OtherEntity(created));
    }
    if !attestations.is_empty() {
        return Err(Failure::Attested(attestations.len()));
    }
    check_definition(&event)?;
    Ok(event)
}

/// Checks that the file at `path` holds the state of the definition `id`, whose head is
/// `creation` alone. Its values are not compared with what `creation` gives.
fn check_state_file(path: &Path, id: EntityId, creation: &Event) -> Result<(), Failure> {
    require(path, EntryType::File)?;
    let bytes = fs::read(path).map_err(Failure::Unreadable)?;
    let state = state_encoding::decode(&bytes).map_err(Failure::UndecodableState)?;
    if state.entity_id != id {
        return Err(Failure::StateOfOtherEntity(state.entity_id));
    }
    if state.state.head != Clock::singleton(creation) {
        return Err(Failure::HeadNotCreation);
    }
    Ok(())
}

/// Checks that `creation` makes its entity a member of exactly one of the catalog's
/// models, the mark of a definition. This reads only the event's memberships, not the
/// values it gives.
fn check_definition(creation: &Event) -> Result<(), Failure> {
    let operations = creation.operations().decode().map_err(Failure::UnreadableOperations)?;
    let models: Vec<ModelId> = operations.memberships().map(|Membership::Add(model)| *model).collect();
    match models[..] {
        [ModelId::System(SystemModel::Model | SystemModel::Property | SystemModel::ModelProperty)] => Ok(()),
        _ => Err(Failure::NotADefinition(models)),
    }
}

fn joined(models: &[ModelId]) -> String {
    models.iter().map(ModelId::to_string).collect::<Vec<_>>().join(", ")
}
