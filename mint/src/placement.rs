//! Where the kept mint's files sit in the crate's sources: each model's `.akev` file beside
//! the struct that declares it, `<model>.akev` beside `<model>.rs`.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use thiserror::Error;

/// What ends the name of a minted file.
pub const EXTENSION: &str = "akev";

/// Why a model's file cannot be placed.
#[derive(Debug, Error)]
pub enum PlaceError {
    #[error("no file in {} is named {file}, where {model} is declared", .directory.display())]
    NoSource { model: String, directory: PathBuf, file: String },
    #[error("{count} files in {} are named {file}", .directory.display())]
    SeveralSources { count: usize, directory: PathBuf, file: String },
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
}

/// Every `.akev` file under `dir`, in no particular order.
pub fn minted_files(dir: &Path) -> Result<Vec<PathBuf>, PlaceError> {
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

/// The path of the source that declares the model `name`, by convention
/// `<name in snake case>.rs` somewhere under its namespace's directory.
pub fn model_source(namespace: &Path, name: &str) -> Result<PathBuf, PlaceError> {
    let file = format!("{}.rs", snake_case(name));
    let mut found = Vec::new();
    find(namespace, &file, &mut found)?;
    match found.len() {
        1 => Ok(found.remove(0)),
        0 => Err(PlaceError::NoSource { model: name.to_owned(), directory: namespace.to_owned(), file }),
        count => Err(PlaceError::SeveralSources { count, directory: namespace.to_owned(), file }),
    }
}

fn find(dir: &Path, file: &str, found: &mut Vec<PathBuf>) -> Result<(), PlaceError> {
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

fn io_error(path: &Path, source: io::Error) -> PlaceError {
    PlaceError::Io { path: path.to_owned(), source }
}
