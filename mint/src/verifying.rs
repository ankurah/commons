//! Checks the `.akev` files a mint keeps in the crate's sources, stopping at the first that
//! fails:
//!
//! - every listed model has its file beside its struct, and no other `.akev` file is there;
//! - the files read: every block builds an event, the genesis of the entity its header names;
//! - each file states exactly the events the declarations need, the definitions of its model
//!   as its struct declares it and, beside `graph.edge_kind`'s, the records of the kinds of
//!   edge the `graph` namespace's table lists; and the struct declares the ids the file states:
//!   its model's, and each property's on the field that declares it.
//!
//! It does not check who minted the events, since nothing is signed.

use std::path::{Path, PathBuf};

use ankurah::EntityId;
use ankurah_commons::{akev::ReadError, namespaces::Namespace};
use thiserror::Error;

use crate::{
    minting::{self, KeptError, MintError, Minted},
    placement::{self, PlaceError, EXTENSION},
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
    #[error("cannot be placed: {0}")]
    Unplaced(PlaceError),
    #[error("is a minted file of nothing listed")]
    Stray,
    #[error("does not read, at line {}: {}", .0.line, .0.problem)]
    Unread(Box<ReadError>),
    #[error("cannot be minted: {0}")]
    Unminted(Box<MintError>),
    #[error("states no event of {0} as the declarations need it")]
    NotAsDeclared(Minted),
    #[error("states the genesis of {0}, which the declarations do not need")]
    Unneeded(EntityId),
    #[error("states the model's id as {stated}, and its struct declares {declared:?}")]
    OtherId { stated: EntityId, declared: Option<&'static str> },
    #[error("states the id of `{struct_name}`'s property `{field}` as {stated}, and the field declares none")]
    NoPropertyId { struct_name: &'static str, field: &'static str, stated: EntityId },
    #[error("states the id of `{struct_name}`'s property `{field}` as {stated}, and the field declares {declared}")]
    OtherPropertyId { struct_name: &'static str, field: &'static str, stated: EntityId, declared: &'static str },
}

/// What `verify` checked.
#[derive(Debug)]
pub struct Verified {
    pub files: usize,
    pub events: usize,
}

/// Checks the minted files of `namespaces` under `src`, the namespaces' parent directory.
pub fn verify(src: &Path, namespaces: &'static [Namespace]) -> Result<Verified, VerifyError> {
    let mut listed = Vec::new();
    for namespace in namespaces {
        let directory = src.join(namespace.name);
        for model in namespace.models {
            let source = placement::model_source(&directory, model().name)
                .map_err(|failure| VerifyError { path: directory.clone(), failure: Failure::Unplaced(failure) })?;
            listed.push(source.with_extension(EXTENSION));
        }
    }
    let files =
        placement::minted_files(src).map_err(|failure| VerifyError { path: src.to_owned(), failure: Failure::Unplaced(failure) })?;
    if let Some(missing) = listed.iter().find(|path| !files.contains(path)) {
        return Err(VerifyError { path: missing.clone(), failure: Failure::Missing });
    }
    if let Some(stray) = files.into_iter().find(|file| !listed.contains(file)) {
        return Err(VerifyError { path: stray, failure: Failure::Stray });
    }
    let minted = minting::mint(src, namespaces).map_err(|error| match error {
        MintError::Kept(KeptError::Unread(error)) => VerifyError { path: error.path.clone(), failure: Failure::Unread(Box::new(error)) },
        other => VerifyError { path: src.to_owned(), failure: Failure::Unminted(Box::new(other)) },
    })?;
    for file in &minted {
        let failed = |failure| Err(VerifyError { path: file.path.clone(), failure });
        if let Some(&needed) = file.drawn.first() {
            return failed(Failure::NotAsDeclared(needed));
        }
        if let Some(&unneeded) = file.unneeded.first() {
            return failed(Failure::Unneeded(unneeded));
        }
        if file.descriptor.explicit_id != Some(file.model.to_base64().as_str()) {
            return failed(Failure::OtherId { stated: file.model, declared: file.descriptor.explicit_id });
        }
        // `mint` gives the properties' ids in the order the struct declares the properties.
        let struct_name = file.descriptor.name;
        for (&(field, stated), declared) in
            file.properties.iter().zip(file.descriptor.properties.iter().map(|property| property.explicit_id))
        {
            match declared {
                None => return failed(Failure::NoPropertyId { struct_name, field, stated }),
                Some(declared) if declared != stated.to_base64() => {
                    return failed(Failure::OtherPropertyId { struct_name, field, stated, declared })
                }
                Some(_) => {}
            }
        }
    }
    Ok(Verified { files: minted.len(), events: minted.iter().map(|file| file.geneses.len()).sum() })
}
