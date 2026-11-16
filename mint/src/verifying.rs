//! Checks the `.akev` files a mint keeps in the crate's sources, stopping at the first that
//! fails:
//!
//! - every listed model has its file beside its struct, the `graph` namespace has the edge
//!   kinds' file, and no other `.akev` file is there;
//! - each file parses, in exactly the form the writer gives it;
//! - each states its model as the struct declares it: its label, its name, and its
//!   properties in order, each with its value type, backend and whether it is optional;
//!   the edge kinds' file states the kinds the `graph` namespace's table lists, in order;
//! - every creation event rebuilt from a file has the id its line gives.
//!
//! It does not check who minted the events, since nothing is signed.

use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

use ankurah::{core::schema::ModelStructDescriptor, Model};
use ankurah_commons::{
    akev::{EdgeKinds, ModelDefinitions, ParseError},
    graph::EdgeKind,
    rebuilding::RebuildError,
};
use thiserror::Error;

use crate::{
    namespaces::Namespace,
    writing::{self, WriteError, EDGE_KINDS_FILE},
};

#[cfg(test)]
mod tests;

/// The first file that failed, and why.
#[derive(Debug, Error)]
#[error("{}: {failure}", path.display())]
pub struct VerifyError {
    pub path: PathBuf,
    pub failure: Failure,
}

/// Why a file fails verification.
#[derive(Debug, Error)]
pub enum Failure {
    #[error("is missing")]
    Missing,
    #[error("cannot be read: {0}")]
    Unreadable(io::Error),
    #[error("cannot be placed: {0}")]
    Unplaced(WriteError),
    #[error("is a minted file of nothing listed")]
    Stray,
    #[error("does not parse: {0}")]
    Unparsable(ParseError),
    #[error("states a {0} other than the struct declares")]
    NotAsDeclared(&'static str),
    #[error("states other kinds of edge than the `graph` namespace's table lists")]
    NotAsListed,
    #[error("lies beside no definitions of the edge kinds' model")]
    NoEdgeKindModel,
    #[error("does not rebuild: {0}")]
    NotRebuilt(RebuildError),
}

/// What `verify` checked.
#[derive(Debug)]
pub struct Verified {
    pub files: usize,
    pub events: usize,
}

/// Checks the minted files of `namespaces` under `src`, the namespaces' parent directory.
pub fn verify(src: &Path, namespaces: &[Namespace]) -> Result<Verified, VerifyError> {
    let mut checked = BTreeSet::new();
    let mut events = 0;
    for namespace in namespaces {
        let directory = src.join(namespace.name.replace('-', "_"));
        let mut edge_kind = None;
        for model in namespace.models {
            let descriptor = (model.descriptor)();
            let path = writing::model_file(&directory, descriptor.name)
                .map_err(|failure| VerifyError { path: directory.clone(), failure: Failure::Unplaced(failure) })?;
            let failed = |failure| VerifyError { path: path.clone(), failure };
            let (definitions, ids) = ModelDefinitions::parse(&read(&path)?).map_err(|error| failed(Failure::Unparsable(error)))?;
            as_declared(&definitions, descriptor).map_err(failed)?;
            events += definitions.events_as_stated(&ids).map_err(|error| failed(Failure::NotRebuilt(error)))?.len();
            if descriptor.label == EdgeKind::descriptor().label {
                edge_kind = Some(definitions);
            }
            checked.insert(path);
        }
        if !namespace.edge_kinds.is_empty() {
            let path = directory.join(EDGE_KINDS_FILE);
            let failed = |failure| VerifyError { path: path.clone(), failure };
            let (kinds, ids) = EdgeKinds::parse(&read(&path)?).map_err(|error| failed(Failure::Unparsable(error)))?;
            let stated = kinds.kinds.iter().map(|kind| (kind.name.as_str(), kind.description.as_str()));
            if !stated.eq(namespace.edge_kinds.iter().copied()) {
                return Err(failed(Failure::NotAsListed));
            }
            let edge_kind = edge_kind.as_ref().ok_or(failed(Failure::NoEdgeKindModel))?;
            events += kinds.events_as_stated(edge_kind, &ids).map_err(|error| failed(Failure::NotRebuilt(error)))?.len();
            checked.insert(path);
        }
    }
    let files = writing::minted_files(src).map_err(|failure| VerifyError { path: src.to_owned(), failure: Failure::Unplaced(failure) })?;
    if let Some(stray) = files.into_iter().find(|file| !checked.contains(file)) {
        return Err(VerifyError { path: stray, failure: Failure::Stray });
    }
    Ok(Verified { files: checked.len(), events })
}

fn read(path: &Path) -> Result<String, VerifyError> {
    fs::read_to_string(path).map_err(|source| VerifyError {
        path: path.to_owned(),
        failure: match source.kind() {
            io::ErrorKind::NotFound => Failure::Missing,
            _ => Failure::Unreadable(source),
        },
    })
}

/// Checks that `definitions` states the model `descriptor` declares, naming the first part
/// that differs.
fn as_declared(definitions: &ModelDefinitions, descriptor: &ModelStructDescriptor) -> Result<(), Failure> {
    let differs = |part| Err(Failure::NotAsDeclared(part));
    if definitions.label != descriptor.label {
        return differs("label");
    }
    if definitions.name != descriptor.name {
        return differs("name");
    }
    if definitions.properties.len() != descriptor.properties.len() {
        return differs("number of properties");
    }
    for (stated, declared) in definitions.properties.iter().zip(descriptor.properties) {
        if stated.name != declared.name {
            return differs("property");
        }
        if stated.value_type != declared.value_type {
            return differs("value type");
        }
        if stated.backend != declared.backend {
            return differs("backend");
        }
        if stated.optional != declared.optional {
            return differs("optionality");
        }
    }
    Ok(())
}
