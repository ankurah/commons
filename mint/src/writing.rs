//! Writes what `mint` minted into the crate's sources, as `.akev` files: each model's
//! definitions beside the struct that declares it, `<model>.akev` beside `<model>.rs`, and
//! the records of the kinds of edge in `graph/edge_kinds.akev`. Each file is checked, before
//! it is written, to rebuild exactly the events the minting node made.

use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use ankurah::{
    proto::{EntityId, Event, Identity},
    Model,
};
use ankurah_commons::{
    akev::{Creation, EdgeKindRecord, EdgeKinds, ModelDefinitions, PropertyDefinition},
    graph::EdgeKind,
};
use ankurah_commons_mint::rebuilding::{self, RebuildError};
use thiserror::Error;

use crate::registering::MintedNamespace;

/// What ends the name of a minted file.
pub const EXTENSION: &str = "akev";

/// The file, in the `graph` namespace's directory, that holds the edge kinds' records.
pub const EDGE_KINDS_FILE: &str = "edge_kinds.akev";

/// Why the minted files could not be written.
#[derive(Debug, Error)]
pub enum WriteError {
    #[error("{} is already a minted file; remove every .akev file to mint again", .0.display())]
    Exists(PathBuf),
    #[error("no file in {} is named {file}, where {model} is declared", .directory.display())]
    NoSource { model: String, directory: PathBuf, file: String },
    #[error("{count} files in {} are named {file}", .directory.display())]
    SeveralSources { count: usize, directory: PathBuf, file: String },
    #[error("the minting node holds no creation event of {0}")]
    NoCreation(EntityId),
    #[error("{label}'s definitions: {source}")]
    NotRebuilt { label: String, source: RebuildError },
    #[error("the edge kinds are minted with no definitions of their model beside them")]
    NoEdgeKindModel,
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
}

/// Refuses `src` if it holds a minted file already, so that `mint` stops before minting what
/// it could not write and never replaces a kept mint.
pub fn refuse_kept(src: &Path) -> Result<(), WriteError> {
    match minted_files(src)?.into_iter().next() {
        Some(path) => Err(WriteError::Exists(path)),
        None => Ok(()),
    }
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

/// The path of the file that holds the definitions of the model `name`, declared by
/// convention in `<name in snake case>.rs` somewhere under its namespace's directory.
pub fn model_file(namespace: &Path, name: &str) -> Result<PathBuf, WriteError> {
    let file = format!("{}.rs", snake_case(name));
    let mut found = Vec::new();
    find(namespace, &file, &mut found)?;
    match found.len() {
        1 => Ok(found.remove(0).with_extension(EXTENSION)),
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

/// Writes the files of `minted` into `src`, the namespaces' parent directory.
pub fn write(src: &Path, minted: &[MintedNamespace]) -> Result<(), WriteError> {
    for namespace in minted {
        let directory = src.join(namespace.name.replace('-', "_"));
        let creations: BTreeMap<EntityId, &Event> = namespace.entities.iter().map(|entity| (entity.id, &entity.creation)).collect();
        let creation = |id: EntityId| {
            let event = creations.get(&id).ok_or(WriteError::NoCreation(id))?;
            let Some(Identity::Ed25519(minter)) = event.minter() else { return Err(WriteError::NoCreation(id)) };
            Ok((minter, Creation { id, at: event.timestamp(), nonce: *event.nonce() }))
        };
        let mut edge_kind = None;
        for model in &namespace.models {
            let (minter, model_creation) = creation(model.id)?;
            let mut properties = Vec::with_capacity(model.properties.len());
            for property in &model.properties {
                properties.push(PropertyDefinition {
                    name: property.name.clone(),
                    value_type: property.value_type.clone(),
                    backend: property.backend.clone(),
                    optional: property.optional,
                    property: creation(property.id)?.1,
                    link: creation(property.link_id)?.1,
                });
            }
            let definitions =
                ModelDefinitions { minter, label: model.label.clone(), name: model.name.clone(), model: model_creation, properties };
            rebuilding::model_events(&definitions).map_err(|source| WriteError::NotRebuilt { label: model.label.clone(), source })?;
            create(&model_file(&directory, &model.name)?, &definitions.write())?;
            if model.label == EdgeKind::descriptor().label {
                edge_kind = Some(definitions);
            }
        }
        if let Some(first) = namespace.edge_kinds.first() {
            let edge_kind = edge_kind.as_ref().ok_or(WriteError::NoEdgeKindModel)?;
            let minter = creation(first.id)?.0;
            let kinds = namespace
                .edge_kinds
                .iter()
                .map(|kind| {
                    let record = creation(kind.id)?.1;
                    Ok(EdgeKindRecord { name: kind.name.to_owned(), description: kind.description.to_owned(), record })
                })
                .collect::<Result<_, WriteError>>()?;
            let kinds = EdgeKinds { minter, kinds };
            let label = edge_kind.label.clone();
            rebuilding::edge_kind_events(&kinds, edge_kind).map_err(|source| WriteError::NotRebuilt { label, source })?;
            create(&directory.join(EDGE_KINDS_FILE), &kinds.write())?;
        }
    }
    Ok(())
}

/// Writes `text` into a new file at `path`, refusing one that exists.
fn create(path: &Path, text: &str) -> Result<(), WriteError> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .map_err(|source| io_error(path, source))
}

fn io_error(path: &Path, source: io::Error) -> WriteError {
    WriteError::Io { path: path.to_owned(), source }
}
