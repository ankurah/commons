//! Mints the definitions of every listed model, and the records of the listed kinds of
//! edge, without a node. An entity's operations are what registering its model's
//! declaration writes (`ankurah_commons::registration`), or a kind's record
//! (`edge_kinds`). It keeps the nonce and timestamp of a kept event of its file whose
//! operations are exactly those, and draws a fresh nonce and the current time otherwise.
//! Its id follows from those, so minting a kept mint again changes only what the
//! declarations changed.

use std::{
    fmt,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use ankurah::{
    core::schema::ModelStructDescriptor,
    proto::{Event, OperationSet},
    EntityId,
};
use ankurah_commons::{
    akev::{BuildError, Genesis},
    namespaces::Namespace,
    registration::{self, Definition},
};
use thiserror::Error;

use self::{edge_kinds::EdgeKindModel, kept::Kept};
use crate::placement::{self, PlaceError, EXTENSION};

mod edge_kinds;
mod kept;

pub use kept::KeptError;

/// What minting made of one model: the file of its definitions, beside the struct that
/// declares it, and, for the model whose records the kinds of edge are, those records after
/// the definitions.
pub struct MintedFile {
    pub descriptor: &'static ModelStructDescriptor,
    /// The struct's source, which declares the model's id.
    pub source: PathBuf,
    /// The `.akev` file beside it.
    pub path: PathBuf,
    /// The id of the model's definition.
    pub model: EntityId,
    pub geneses: Vec<Genesis>,
    /// What the file needs that no kept event matched, whose nonce and timestamp were drawn.
    pub drawn: Vec<Minted>,
    /// The kept events of the file that nothing it needs matched.
    pub unneeded: Vec<EntityId>,
}

/// An entity the mint makes: one of a model's definitions, or the record of a kind of edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Minted {
    Definition(Definition),
    EdgeKind(&'static str),
}

impl fmt::Display for Minted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Minted::Definition(Definition::Model) => f.write_str("the model's definition"),
            Minted::Definition(Definition::Property(name)) => write!(f, "the definition of the property `{name}`"),
            Minted::Definition(Definition::Link(name)) => write!(f, "the link that gives the model the property `{name}`"),
            Minted::EdgeKind(name) => write!(f, "the record of the kind of edge `{name}`"),
        }
    }
}

/// Why minting stopped.
#[derive(Debug, Error)]
pub enum MintError {
    #[error(transparent)]
    Placement(#[from] PlaceError),
    #[error("the kept mint: {0}")]
    Kept(#[from] KeptError),
    #[error("{label}: the operations do not build: {source}")]
    Build { label: &'static str, source: BuildError },
    #[error("the kinds of edge are records of {label}, which declares no property `{property}`")]
    NoEdgeKindProperty { label: &'static str, property: &'static str },
}

/// Mints every model of `namespaces`, and their kinds of edge, keeping the nonces and
/// timestamps of the kept mint under `src`, the namespaces' parent directory.
pub fn mint(src: &Path, namespaces: &'static [Namespace]) -> Result<Vec<MintedFile>, MintError> {
    let kept = Kept::read(src)?;
    let mut files = Vec::new();
    for namespace in namespaces {
        let directory = src.join(namespace.name);
        for model in namespace.models {
            let descriptor = model();
            let source = placement::model_source(&directory, descriptor.name)?;
            let path = source.with_extension(EXTENSION);
            let mut keeping = Keeping::new(kept.events(&path));
            let built = |source| MintError::Build { label: descriptor.label, source };
            let geneses = registration::definitions(descriptor, |definition, operations| {
                keeping.creation(Minted::Definition(definition), operations)
            })
            .map_err(built)?;
            let ids = geneses.iter().map(|genesis| Ok(genesis.event()?.entity_id)).collect::<Result<Vec<_>, _>>().map_err(built)?;
            let mut file = MintedFile { descriptor, source, path, model: ids[0], geneses, drawn: Vec::new(), unneeded: Vec::new() };
            let records_of = namespace.edge_kinds.as_ref().filter(|edge_kinds| (edge_kinds.model)().label == descriptor.label);
            if let Some(edge_kinds) = records_of {
                // Each property's definition follows the model's, and the definition and link
                // of each property before it.
                let property = |property| {
                    let index = descriptor.properties.iter().position(|declared| declared.name == property);
                    let index = index.ok_or(MintError::NoEdgeKindProperty { label: descriptor.label, property })?;
                    Ok::<_, MintError>(ids[1 + 2 * index])
                };
                let model = EdgeKindModel { model: ids[0], name: property("name")?, description: property("description")? };
                for &(name, description) in edge_kinds.kinds {
                    let operations = edge_kinds::record(&model, name, description);
                    let (nonce, timestamp) = keeping.creation(Minted::EdgeKind(name), &operations.build().map_err(built)?);
                    file.geneses.push(Genesis { nonce, timestamp, operations });
                }
            }
            (file.drawn, file.unneeded) = keeping.finish();
            files.push(file);
        }
    }
    Ok(files)
}

/// The kept events of one file, and which of them the mint has kept.
struct Keeping<'k> {
    kept: &'k [Event],
    used: Vec<bool>,
    drawn: Vec<Minted>,
}

impl<'k> Keeping<'k> {
    fn new(kept: &'k [Event]) -> Self {
        Keeping { kept, used: vec![false; kept.len()], drawn: Vec::new() }
    }

    /// The nonce and timestamp of `minted`, whose operations are `operations`: those of the
    /// first kept event not yet used with exactly these operations, or drawn now.
    fn creation(&mut self, minted: Minted, operations: &OperationSet) -> ([u8; 32], u64) {
        match (0..self.kept.len()).find(|&index| !self.used[index] && self.kept[index].operations() == operations) {
            Some(index) => {
                self.used[index] = true;
                (*self.kept[index].nonce(), self.kept[index].timestamp())
            }
            None => {
                self.drawn.push(minted);
                drawn()
            }
        }
    }

    /// What was drawn, and the kept events nothing used.
    fn finish(self) -> (Vec<Minted>, Vec<EntityId>) {
        let unused = self.kept.iter().zip(&self.used).filter(|(_, used)| !**used).map(|(event, _)| event.entity_id).collect();
        (self.drawn, unused)
    }
}

/// The nonce and timestamp of an entity the kept mint does not hold: a fresh nonce, and now.
fn drawn() -> ([u8; 32], u64) {
    let since_1970 = SystemTime::now().duration_since(UNIX_EPOCH).expect("the clock reads after 1970");
    (rand::random(), since_1970.as_millis().try_into().expect("milliseconds since 1970 fit in 64 bits"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::copies::{copied, edit_and_restate};
    use ankurah_commons::{akev, namespaces::NAMESPACES};
    use std::fs;

    /// The events of the file at `path`.
    fn events(path: &Path) -> Vec<Event> {
        akev::read(&[(path, &fs::read_to_string(path).unwrap())]).unwrap().remove(0)
    }

    /// A kept link that holds another optionality than its property declares: the mint
    /// draws the link anew, every other entity of the file keeps its nonce and timestamp,
    /// and the stale link is no longer needed.
    #[test]
    fn a_changed_need_draws_its_event_anew_and_leaves_the_kept_one_unneeded() {
        let src = copied();
        let intention = src.path().join("cognition").join("intention.akev");
        let before = events(&intention);
        edit_and_restate(&intention, "optional: false", "optional: true");
        let kept = events(&intention);
        let stale: Vec<EntityId> =
            kept.iter().map(|event| event.entity_id).filter(|id| before.iter().all(|event| event.entity_id != *id)).collect();
        assert_eq!(stale.len(), 1);

        let minted = mint(src.path(), NAMESPACES).unwrap();
        let file = minted.iter().find(|file| file.path == intention).unwrap();
        assert_eq!(file.drawn, [Minted::Definition(Definition::Link("title"))]);
        assert_eq!(file.unneeded, stale);
        // The model's definition, then `title`'s, then the link drawn anew.
        let drawn = &file.geneses[2];
        assert!(kept.iter().all(|event| *event.nonce() != drawn.nonce));
        for (index, (genesis, event)) in file.geneses.iter().zip(&before).enumerate().filter(|(index, _)| *index != 2) {
            assert_eq!((genesis.nonce, genesis.timestamp), (*event.nonce(), event.timestamp()), "the genesis at {index}");
        }
    }
}
