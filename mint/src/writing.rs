//! Writes what `mint` minted into the crate's sources: each model's file of events beside
//! the struct that declares it, which is given its model's minted id (`declaring`). A file
//! the kept mint has is replaced.

use std::{fs, io, path::PathBuf};

use ankurah_commons::akev;
use thiserror::Error;

use crate::{
    declaring::{self, DeclareError},
    minting::MintedFile,
};

/// Why the minted files could not be written.
#[derive(Debug, Error)]
pub enum WriteError {
    #[error("{}: {source}", path.display())]
    Akev { path: PathBuf, source: akev::WriteError },
    #[error(transparent)]
    Declare(#[from] DeclareError),
    #[error("{}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
}

/// Writes the files of `minted`. Every file's text is made before any is written, so that a
/// declaration that cannot be given its id stops the mint with every file as it was.
pub fn write(minted: &[MintedFile]) -> Result<(), WriteError> {
    let mut files = Vec::with_capacity(2 * minted.len());
    for file in minted {
        let text = akev::write(&file.geneses).map_err(|source| WriteError::Akev { path: file.path.clone(), source })?;
        let source = fs::read_to_string(&file.source).map_err(|source| WriteError::Io { path: file.source.clone(), source })?;
        let declared = declaring::declared(&file.source, &source, file.descriptor.name, file.model)?;
        files.push((&file.path, text));
        files.push((&file.source, declared));
    }
    for (path, text) in files {
        // An unchanged file is left alone, so that a mint that changes nothing touches nothing.
        if fs::read_to_string(path).ok().as_ref() != Some(&text) {
            fs::write(path, text).map_err(|source| WriteError::Io { path: path.clone(), source })?;
        }
    }
    Ok(())
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

        let error = write(&minting::mint(src.path(), NAMESPACES).unwrap()).unwrap_err();
        assert!(matches!(error, WriteError::Declare(DeclareError::SeveralIds { count: 2, .. })), "{error}");
        assert_eq!(files(src.path()), before);
    }
}
