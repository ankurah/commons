//! Writes what `mint` minted into the crate's sources, as `.akev` files: each model's
//! definitions beside the struct that declares it, `<model>.akev` beside `<model>.rs`, and
//! the records of the kinds of edge in `graph/edge_kinds.akev`. A file the kept mint has is
//! replaced. Each struct then declares its model's minted id, `#[model(id = "<id>")]` on the
//! line after its label.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use ankurah::EntityId;
use ankurah_commons::rebuilding::RebuildError;
use thiserror::Error;

use crate::minting::MintedNamespace;

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
    #[error("{} has no line `#[model(label = \"{label}\")]`, or more than one", path.display())]
    NoLabel { path: PathBuf, label: String },
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

/// Writes the files of `minted` into `src`, the namespaces' parent directory.
pub fn write(src: &Path, minted: &[MintedNamespace]) -> Result<(), WriteError> {
    for MintedNamespace { namespace, models, edge_kinds } in minted {
        let directory = src.join(namespace.name);
        for definitions in models {
            let not_rebuilt = |source| WriteError::NotRebuilt { label: definitions.label.clone(), source };
            let source = model_source(&directory, &definitions.name)?;
            replace(&source.with_extension(EXTENSION), &definitions.write().map_err(not_rebuilt)?)?;
            // The model's event comes first.
            let id = definitions.events().map_err(not_rebuilt)?[0].entity_id;
            declare_id(&source, &definitions.label, id)?;
        }
        if let (Some(records), Some(listed)) = (edge_kinds, &namespace.edge_kinds) {
            let label = (listed.model)().label;
            let edge_kind = models.iter().find(|definitions| definitions.label == label).ok_or(WriteError::NoEdgeKindModel)?;
            let text = records.write(edge_kind).map_err(|source| WriteError::NotRebuilt { label: label.to_owned(), source })?;
            replace(&directory.join(EDGE_KINDS_FILE), &text)?;
        }
    }
    Ok(())
}

fn replace(path: &Path, text: &str) -> Result<(), WriteError> {
    fs::write(path, text).map_err(|source| io_error(path, source))
}

/// Declares `id` in the source at `path` as the id of the model labelled `label`, on the line
/// after the label's, in place of the id declared there.
fn declare_id(path: &Path, label: &str, id: EntityId) -> Result<(), WriteError> {
    let text = fs::read_to_string(path).map_err(|source| io_error(path, source))?;
    let mut lines: Vec<String> = text.split_inclusive('\n').map(str::to_owned).collect();
    let label_line = format!("#[model(label = \"{label}\")]");
    let labels: Vec<usize> = lines.iter().enumerate().filter(|(_, line)| line.trim_end() == label_line).map(|(index, _)| index).collect();
    let [label_index] = labels[..] else { return Err(WriteError::NoLabel { path: path.to_owned(), label: label.to_owned() }) };
    let id_line = format!("#[model(id = \"{}\")]\n", id.to_base64());
    match lines.get(label_index + 1) {
        Some(line) if line.starts_with("#[model(id = ") => lines[label_index + 1] = id_line,
        _ => lines.insert(label_index + 1, id_line),
    }
    replace(path, &lines.concat())
}

fn io_error(path: &Path, source: io::Error) -> WriteError {
    WriteError::Io { path: path.to_owned(), source }
}
