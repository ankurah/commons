//! Writes what `mint` minted into a directory of its own:
//!
//! ```text
//! <out>/<namespace>.md           the namespace's page
//! <out>/<namespace>/<id>.event   a definition's creation event, as Ankurah encodes an
//!                                event with its attestations
//! <out>/<namespace>/<id>.state   the definition's state, in `state_encoding`
//! ```

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use ankurah::proto::EntityId;
use thiserror::Error;

use crate::{
    page,
    registering::{MintedDefinition, MintedNamespace},
    state_encoding,
};

/// The name of `namespace`'s page.
pub fn page_file(namespace: &str) -> String {
    format!("{namespace}.md")
}

/// What ends the name of a definition's event file. The name before it is the entity's id,
/// which is also its creation event's id; a later event of the same entity would need a
/// name of its own.
pub const EVENT_EXTENSION: &str = "event";

/// What ends the name of a definition's state file, after its entity's id.
pub const STATE_EXTENSION: &str = "state";

fn definition_file(id: EntityId, extension: &str) -> String {
    format!("{}.{extension}", id.to_base64())
}

/// Why the output directory could not be written.
#[derive(Debug, Error)]
pub enum WriteError {
    #[error("{} already exists; remove it, or name another directory with --out", .0.display())]
    Exists(PathBuf),
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
}

/// Refuses `out` if anything is there, so that `mint` stops before minting what it could
/// not write. `write` refuses it again, in case something appears there meanwhile.
pub fn refuse_existing(out: &Path) -> Result<(), WriteError> {
    match fs::symlink_metadata(out) {
        Ok(_) => Err(WriteError::Exists(out.to_owned())),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(io_error(out, source)),
    }
}

/// Creates the directory `out`, which must not exist yet, and writes into it the creation
/// events, states and pages of `minted`.
pub fn write(out: &Path, minted: &[MintedNamespace]) -> Result<(), WriteError> {
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|source| io_error(parent, source))?;
    }
    fs::create_dir(out).map_err(|source| match source.kind() {
        io::ErrorKind::AlreadyExists => WriteError::Exists(out.to_owned()),
        _ => io_error(out, source),
    })?;
    for namespace in minted {
        let directory = out.join(namespace.name);
        fs::create_dir(&directory).map_err(|source| io_error(&directory, source))?;
        for definition in &namespace.definitions {
            write_definition(&directory, definition)?;
        }
        let page = page::render(namespace.name, &namespace.models);
        create(&out.join(page_file(namespace.name)), page.as_bytes())?;
    }
    Ok(())
}

fn write_definition(directory: &Path, definition: &MintedDefinition) -> Result<(), WriteError> {
    create(&directory.join(definition_file(definition.id, EVENT_EXTENSION)), &definition.creation.encode())?;
    create(&directory.join(definition_file(definition.id, STATE_EXTENSION)), &state_encoding::encode(&definition.state))
}

/// Writes `bytes` into a new file at `path`, refusing one that exists.
fn create(path: &Path, bytes: &[u8]) -> Result<(), WriteError> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut file| file.write_all(bytes))
        .map_err(|source| io_error(path, source))
}

fn io_error(path: &Path, source: io::Error) -> WriteError {
    WriteError::Io { path: path.to_owned(), source }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_directory_that_exists_is_refused_before_minting_and_when_writing() {
        let parent = tempfile::tempdir().unwrap();
        let absent = parent.path().join("minted");
        refuse_existing(&absent).unwrap();

        let existing = parent.path();
        assert!(matches!(refuse_existing(existing), Err(WriteError::Exists(path)) if path == existing));
        let error = write(existing, &[]).unwrap_err();
        assert!(matches!(error, WriteError::Exists(path) if path == existing));
    }
}
