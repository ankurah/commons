//! Mints the definitions of every listed model, and the records of the listed kinds of edge,
//! without a node. An entity's values are what registering its model's declaration writes, or
//! a kind's name and description. Its time and nonce are those of its line in the kept mint,
//! when the kept mint has one, and are drawn now otherwise. Its id follows from those, so
//! minting a kept mint again changes only what the declarations or Ankurah's events changed.

use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use ankurah::{core::schema::ModelStructDescriptor, EntityId};
use ankurah_commons::{
    akev::{Creation, EdgeKindRecord, EdgeKinds, ModelDefinitions, ParseError, PropertyDefinition},
    namespaces::Namespace,
};
use thiserror::Error;

use crate::writing::{self, WriteError, EDGE_KINDS_FILE};

/// What minting one namespace made: each model's definitions, and the records of the kinds
/// of edge minted with it.
pub struct MintedNamespace {
    pub namespace: &'static Namespace,
    pub models: Vec<ModelDefinitions>,
    pub edge_kinds: Option<EdgeKinds>,
}

/// Why minting stopped.
#[derive(Debug, Error)]
pub enum MintError {
    #[error(transparent)]
    Placement(#[from] WriteError),
    #[error("{}: the kept mint cannot be read: {source}", path.display())]
    Unreadable { path: PathBuf, source: io::Error },
    #[error("{}: the kept mint does not parse: {source}", path.display())]
    Unparsable { path: PathBuf, source: ParseError },
}

/// Mints every model of `namespaces`, and their kinds of edge, keeping the times and nonces of
/// the kept mint under `src`, the namespaces' parent directory.
pub fn mint(src: &Path, namespaces: &'static [Namespace]) -> Result<Vec<MintedNamespace>, MintError> {
    namespaces
        .iter()
        .map(|namespace| {
            let directory = src.join(namespace.name);
            let mut models = Vec::with_capacity(namespace.models.len());
            for model in namespace.models {
                let descriptor = model();
                models.push(definitions(descriptor, kept(&writing::model_file(&directory, descriptor.name)?, ModelDefinitions::parse)?));
            }
            let edge_kinds = match &namespace.edge_kinds {
                Some(edge_kinds) => Some(records(edge_kinds.kinds, kept(&directory.join(EDGE_KINDS_FILE), EdgeKinds::parse)?)),
                None => None,
            };
            Ok(MintedNamespace { namespace, models, edge_kinds })
        })
        .collect()
}

/// The definitions that registering `descriptor`'s model writes, each entity keeping the time
/// and nonce of its line in `kept`.
fn definitions(descriptor: &ModelStructDescriptor, kept: Option<ModelDefinitions>) -> ModelDefinitions {
    let kept_property = |name: &str| kept.as_ref().and_then(|kept| kept.properties.iter().find(|property| property.name == name));
    let properties = descriptor
        .properties
        .iter()
        .map(|declared| {
            let kept = kept_property(declared.name);
            PropertyDefinition {
                name: declared.name.to_owned(),
                value_type: declared.value_type.to_owned(),
                backend: declared.backend.to_owned(),
                optional: declared.optional,
                property: kept.map_or_else(drawn, |kept| kept.property),
                link: kept.map_or_else(drawn, |kept| kept.link),
            }
        })
        .collect();
    let model = kept.as_ref().map_or_else(drawn, |kept| kept.model);
    ModelDefinitions { label: descriptor.label.to_owned(), name: descriptor.name.to_owned(), model, properties }
}

/// A record of each kind in `table`, its name and description, keeping the time and nonce of
/// its line in `kept`.
fn records(table: &[(&str, &str)], kept: Option<EdgeKinds>) -> EdgeKinds {
    let kept_record = |name: &str| kept.as_ref().and_then(|kept| kept.kinds.iter().find(|kind| kind.name == name)).map(|kind| kind.record);
    let kinds = table
        .iter()
        .map(|&(name, description)| EdgeKindRecord {
            name: name.to_owned(),
            description: description.to_owned(),
            record: kept_record(name).unwrap_or_else(drawn),
        })
        .collect();
    EdgeKinds { kinds }
}

/// How a minted file's text is read: what it states, and the id each line states.
type Parse<T> = fn(&str) -> Result<(T, Vec<EntityId>), ParseError>;

/// What the kept mint's file at `path` states, if there is one. The ids its lines state are
/// not read, so that a mint made in an earlier format of Ankurah's events keeps its times and
/// nonces.
fn kept<T>(path: &Path, parse: Parse<T>) -> Result<Option<T>, MintError> {
    match fs::read_to_string(path) {
        Ok(text) => {
            parse(&text).map(|(stated, _ids)| Some(stated)).map_err(|source| MintError::Unparsable { path: path.to_owned(), source })
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(MintError::Unreadable { path: path.to_owned(), source }),
    }
}

/// The time and nonce of an entity the kept mint does not hold: now, and a fresh nonce.
fn drawn() -> Creation {
    let since_1970 = SystemTime::now().duration_since(UNIX_EPOCH).expect("the clock reads after 1970");
    Creation { at: since_1970.as_millis().try_into().expect("milliseconds since 1970 fit in 64 bits"), nonce: rand::random() }
}
