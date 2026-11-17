//! The kept mint: every `.akev` file under the namespaces' directory, read together, so
//! that a name in one file can refer to an entity of another.

use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use ankurah::proto::Event;
use ankurah_commons::akev::{self, ReadError};
use thiserror::Error;

use crate::placement::{self, PlaceError};

/// Why the kept mint was not read.
#[derive(Debug, Error)]
pub enum KeptError {
    #[error(transparent)]
    Unlisted(#[from] PlaceError),
    #[error("{}: cannot be read: {source}", path.display())]
    Unreadable { path: PathBuf, source: io::Error },
    #[error("does not read: {0}")]
    Unread(#[from] ReadError),
}

/// The events of each kept file, by its path.
pub struct Kept {
    events: HashMap<PathBuf, Vec<Event>>,
}

impl Kept {
    /// Reads every `.akev` file under `src`.
    pub fn read(src: &Path) -> Result<Kept, KeptError> {
        let paths = placement::minted_files(src)?;
        let mut texts = Vec::with_capacity(paths.len());
        for path in &paths {
            texts.push(fs::read_to_string(path).map_err(|source| KeptError::Unreadable { path: path.clone(), source })?);
        }
        let files: Vec<(&Path, &str)> = paths.iter().map(PathBuf::as_path).zip(texts.iter().map(String::as_str)).collect();
        let events = akev::read(&files)?;
        Ok(Kept { events: paths.into_iter().zip(events).collect() })
    }

    /// The events the file at `path` holds; none if there is no such file.
    pub fn events(&self, path: &Path) -> &[Event] {
        self.events.get(path).map_or(&[], Vec::as_slice)
    }
}
