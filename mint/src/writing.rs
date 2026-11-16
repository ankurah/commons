//! Writes what `mint` minted into the crate's sources, as `.akev` files: each model's
//! definitions beside the struct that declares it, `<model>.akev` beside `<model>.rs`, and
//! the records of the kinds of edge in `graph/edge_kinds.akev`. A file the kept mint has is
//! replaced. Each struct is given its model's minted id (`declaring`).

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use ankurah_commons::rebuilding::RebuildError;
use thiserror::Error;

use crate::{
    declaring::{self, DeclareError},
    minting::MintedNamespace,
};

/// What ends the name of a minted file.
pub const EXTENSION: &str = "akev";

/// The file, in the `graph` namespace's directory, that holds the edge kinds' records.
pub const EDGE_KINDS_FILE: &str = "edge_kinds.akev";

/// Why the minted files could not be written.
#[derive(Debug, Error)]
pub enum WriteError {
    #[error("no file in {} is named {file}, where {model} is declared", .directory.display())]
    NoSource { model: String, directory: PathBuf, file: String },
    #[error("{count} files in {} are named {file}", .directory.display())]
    SeveralSources { count: usize, directory: PathBuf, file: String },
    #[error("{label}'s definitions: {source}")]
    NotRebuilt { label: String, source: RebuildError },
    #[error("the edge kinds are minted with no definitions of their model beside them")]
    NoEdgeKindModel,
    #[error(transparent)]
    Declare(#[from] DeclareError),
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
}

/// Every `.akev` file under `dir`, in no particular order.
pub fn minted_files(dir: &Path) -> Result<Vec<PathBuf>, WriteError> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|source| io_error(dir, source))? {
        let path = entry.map_err(|source| io_error(dir, source))?.path();
        if path.is_dir() {
            files.extend(minted_files(&path)?);
        } else if path.extension().is_some_and(|extension| extension == EXTENSION) {
            files.push(path);
        }
    }
    Ok(files)
}

/// The path of the file that holds the definitions of the model `name`, beside its source.
pub fn model_file(namespace: &Path, name: &str) -> Result<PathBuf, WriteError> {
    Ok(model_source(namespace, name)?.with_extension(EXTENSION))
}

/// The path of the source that declares the model `name`, by convention
/// `<name in snake case>.rs` somewhere under its namespace's directory.
fn model_source(namespace: &Path, name: &str) -> Result<PathBuf, WriteError> {
    let file = format!("{}.rs", snake_case(name));
    let mut found = Vec::new();
    find(namespace, &file, &mut found)?;
    match found.len() {
        1 => Ok(found.remove(0)),
        0 => Err(WriteError::NoSource { model: name.to_owned(), directory: namespace.to_owned(), file }),
        count => Err(WriteError::SeveralSources { count, directory: namespace.to_owned(), file }),
    }
}

fn find(dir: &Path, file: &str, found: &mut Vec<PathBuf>) -> Result<(), WriteError> {
    for entry in fs::read_dir(dir).map_err(|source| io_error(dir, source))? {
        let path = entry.map_err(|source| io_error(dir, source))?.path();
        if path.is_dir() {
            find(&path, file, found)?;
        } else if path.file_name().is_some_and(|name| name == file) {
            found.push(path);
        }
    }
    Ok(())
}

fn snake_case(name: &str) -> String {
    let mut snake = String::with_capacity(name.len() + 4);
    for (index, character) in name.char_indices() {
        if character.is_uppercase() && index > 0 {
            snake.push('_');
        }
        snake.extend(character.to_lowercase());
    }
    snake
}

/// Writes the files of `minted` into `src`, the namespaces' parent directory. Every file's
/// text is made before any is written, so that a declaration that cannot be given its id
/// stops the mint with every file as it was.
pub fn write(src: &Path, minted: &[MintedNamespace]) -> Result<(), WriteError> {
    let mut files = Vec::new();
    for MintedNamespace { namespace, models, edge_kinds } in minted {
        let directory = src.join(namespace.name);
        for definitions in models {
            let not_rebuilt = |source| WriteError::NotRebuilt { label: definitions.label.clone(), source };
            let source = model_source(&directory, &definitions.name)?;
            // The model's event comes first.
            let id = definitions.events().map_err(not_rebuilt)?[0].entity_id;
            let text = fs::read_to_string(&source).map_err(|error| io_error(&source, error))?;
            let declared = declaring::declared(&source, &text, &definitions.name, id)?;
            files.push((source.with_extension(EXTENSION), definitions.write().map_err(not_rebuilt)?));
            files.push((source, declared));
        }
        if let (Some(records), Some(listed)) = (edge_kinds, &namespace.edge_kinds) {
            let label = (listed.model)().label;
            let edge_kind = models.iter().find(|definitions| definitions.label == label).ok_or(WriteError::NoEdgeKindModel)?;
            let text = records.write(edge_kind).map_err(|source| WriteError::NotRebuilt { label: label.to_owned(), source })?;
            files.push((directory.join(EDGE_KINDS_FILE), text));
        }
    }
    for (path, text) in &files {
        // An unchanged file is left alone, so that a mint that changes nothing touches nothing.
        if fs::read_to_string(path).ok().as_ref() != Some(text) {
            fs::write(path, text).map_err(|source| io_error(path, source))?;
        }
    }
    Ok(())
}

fn io_error(path: &Path, source: io::Error) -> WriteError {
    WriteError::Io { path: path.to_owned(), source }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        copies::{copied, edit, files},
        minting,
    };
    use ankurah::EntityId;
    use ankurah_commons::namespaces::NAMESPACES;

    /// A struct that cannot be given its id stops the mint before any file is written, even
    /// those of models it comes after.
    #[test]
    fn a_declaration_that_cannot_take_its_id_stops_the_mint_with_every_file_as_it_was() {
        let src = copied();
        let cognition = src.path().join("cognition");
        // A model the mint would write again, listed before the one it cannot write.
        fs::remove_file(cognition.join("claim.akev")).unwrap();
        let second = format!("#[model(no_ffi, id = \"{}\")]", EntityId::from_bytes([1; 32]).to_base64());
        edit(&cognition.join("intention.rs"), "#[model(no_ffi)]", &second);
        let before = files(src.path());

        let error = write(src.path(), &minting::mint(src.path(), NAMESPACES).unwrap()).unwrap_err();
        assert!(matches!(error, WriteError::Declare(DeclareError::SeveralIds { count: 2, .. })), "{error}");
        assert_eq!(files(src.path()), before);
    }
}
