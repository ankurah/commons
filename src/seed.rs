//! Seeding a node: the kept mint's events, read from the `.akev` text compiled into this
//! crate, imported into a durable node's store, so that the Commons' models bind to the
//! minted definitions there rather than minting their own.

use std::{error::Error, fmt, path::Path};

use ankurah::{
    error::MutationError,
    policy::PolicyAgent,
    proto::{Attested, Event},
    storage::StorageEngine,
    Node,
};

use crate::akev::{self, ReadError};

/// Why a node was not seeded. The kept mint is compiled into the crate and its tests read
/// all of it, so in a build whose tests pass only the import can fail.
#[derive(Debug)]
pub enum SeedError {
    Unreadable(ReadError),
    Import(Box<MutationError>),
}

impl fmt::Display for SeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SeedError::Unreadable(source) => write!(f, "the kept mint does not read: {source}"),
            SeedError::Import(source) => write!(f, "the node did not import the kept mint: {source}"),
        }
    }
}

// Written out rather than derived, so that the source of a boxed error is the error, not its
// box, and a caller can downcast it.
impl Error for SeedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SeedError::Unreadable(source) => Some(source),
            SeedError::Import(source) => Some(source.as_ref()),
        }
    }
}

/// Imports into `node` the kept mint of every namespace this build enables: the definitions
/// of their models, and the records of the kinds of edge. A model of the Commons then binds
/// to its minted definitions, and registering it mints nothing.
///
/// `node` must be durable, with its system root created or loaded. Seed it before it first
/// binds any of the Commons' models, which a fetch, a query, a get or a create does as well
/// as registration: a model declares its minted id, so binding it on a node not yet seeded
/// fails with `RegistrationError::ExplicitModelIdNotFound`. Seeding a node that holds the
/// kept mint already changes nothing, so a node can be seeded at every start.
pub async fn seed<SE, PA>(node: &Node<SE, PA>) -> Result<(), SeedError>
where
    SE: StorageEngine + Send + Sync + 'static,
    PA: PolicyAgent + Send + Sync + 'static,
{
    node.import_events(&kept_mint()?).await.map_err(|error| SeedError::Import(Box::new(error)))
}

/// Each file's path under `src/` and its text, compiled in.
macro_rules! kept {
    ($($path:literal),* $(,)?) => {
        &[$(($path, include_str!($path))),*]
    };
}

/// The kept mint of every namespace this build enables, as compiled into the crate: each
/// model's `.akev` file, by its path under `src/`.
const KEPT_MINT: &[&[(&str, &str)]] = &[
    #[cfg(feature = "identity")]
    kept!["identity/identity_binding.akev", "identity/principal.akev"],
    #[cfg(feature = "records")]
    kept![
        "records/deferred.akev",
        "records/fractional_position.akev",
        "records/lifecycle/acceptance.akev",
        "records/lifecycle/checked.akev",
        "records/lifecycle/closed.akev",
        "records/lifecycle/created.akev",
        "records/lifecycle/edited.akev",
        "records/note.akev",
        "records/provenance/attribution.akev",
        "records/provenance/via.akev",
        "records/responsibility/assignment.akev",
        "records/responsibility/ownership.akev",
    ],
    #[cfg(feature = "health")]
    kept![
        "health/activity_interval.akev",
        "health/distance.akev",
        "health/heart_rate.akev",
        "health/heart_rate_summary.akev",
        "health/physical_activity.akev",
        "health/workout.akev",
    ],
    #[cfg(feature = "text")]
    kept!["text/fragment.akev", "text/quotation.akev"],
    #[cfg(feature = "git")]
    kept!["git/checkout_state.akev", "git/file_anchor.akev", "git/repository.akev", "git/work_location.akev"],
    #[cfg(feature = "graph")]
    kept!["graph/dyn_edge.akev", "graph/edge_kind.akev"],
    #[cfg(feature = "cognition")]
    kept![
        "cognition/claim.akev",
        "cognition/decision.akev",
        "cognition/expectation.akev",
        "cognition/intention.akev",
        "cognition/intuition.akev",
        "cognition/perspective.akev",
    ],
    #[cfg(feature = "work")]
    kept!["work/scope.akev", "work/submission.akev", "work/task.akev"],
    #[cfg(feature = "messaging")]
    kept!["messaging/mention.akev", "messaging/message.akev"],
];

/// The events of the kept mint of every namespace this build enables.
fn kept_mint() -> Result<Vec<Attested<Event>>, SeedError> {
    let files: Vec<(&Path, &str)> = KEPT_MINT.iter().flat_map(|files| files.iter()).map(|&(path, text)| (Path::new(path), text)).collect();
    let events = akev::read(&files).map_err(SeedError::Unreadable)?;
    Ok(events.into_iter().flatten().map(Attested::from).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::namespaces::NAMESPACES;
    use ankurah::{core::schema::ModelStructDescriptor, proto::EntityState, EntityId, ModelId, PermissiveAgent};
    use ankurah_storage_sled::SledStorageEngine;
    use ankurah_storage_sqlite::SqliteStorageEngine;
    use std::sync::Arc;

    async fn seeded<SE: StorageEngine + Send + Sync + 'static>(engine: SE) -> Node<SE, PermissiveAgent> {
        let node = Node::new_durable(Arc::new(engine), PermissiveAgent::new());
        node.system.create().await.unwrap();
        seed(&node).await.unwrap();
        node.wait_ready().await.unwrap();
        node
    }

    /// Each model this build lists, with the id it declares.
    fn declared_models() -> impl Iterator<Item = (&'static ModelStructDescriptor, EntityId)> {
        NAMESPACES.iter().flat_map(|namespace| namespace.models).map(|model| {
            let descriptor = model();
            (descriptor, EntityId::from_base64(descriptor.explicit_id.expect("every model declares its minted id")).unwrap())
        })
    }

    /// Registration binds a model through the catalog's local resolution before it would
    /// register anything; every model resolves there to its minted definition, so registering
    /// it mints nothing. The kinds of edge read as the table states them.
    async fn every_model_binds_to_its_minted_definition<SE: StorageEngine + Send + Sync + 'static>(engine: SE) {
        let node = seeded(engine).await;
        let epoch = node.system.system_epoch().unwrap();
        for (descriptor, id) in declared_models() {
            assert_eq!(descriptor.bind_local(&node.catalog, epoch).unwrap(), ModelId::EntityId(id), "{}", descriptor.label);
        }
        #[cfg(feature = "graph")]
        {
            use crate::graph::{EdgeKindView, EDGE_KINDS};
            let context = node.context_async(ankurah::policy::DEFAULT_CONTEXT).await.unwrap();
            let kinds: Vec<EdgeKindView> = context.fetch("true").await.unwrap();
            let mut read: Vec<(String, String)> = kinds.iter().map(|kind| (kind.name().unwrap(), kind.description().unwrap())).collect();
            let mut table: Vec<(String, String)> =
                EDGE_KINDS.iter().map(|&(name, description)| (name.to_owned(), description.to_owned())).collect();
            read.sort();
            table.sort();
            assert_eq!(read, table);
        }
    }

    #[tokio::test]
    async fn seeding_a_fresh_node_binds_every_model_to_its_minted_definition() {
        every_model_binds_to_its_minted_definition(SledStorageEngine::new_test().unwrap()).await;
        every_model_binds_to_its_minted_definition(SqliteStorageEngine::open_in_memory().await.unwrap()).await;
    }

    /// Every event and state the store holds for each entity of the kept mint, as stored.
    async fn stored<SE: StorageEngine + Send + Sync + 'static>(
        node: &Node<SE, PermissiveAgent>,
    ) -> Vec<(Vec<Attested<Event>>, Attested<EntityState>)> {
        let mut stored = Vec::new();
        for event in kept_mint().unwrap() {
            let id = event.payload.entity_id;
            stored.push((node.storage.dump_entity_events(id).await.unwrap(), node.storage.get_state(id).await.unwrap()));
        }
        stored
    }

    async fn seeding_again_changes_nothing<SE: StorageEngine + Send + Sync + 'static>(engine: SE) {
        let node = seeded(engine).await;
        let before = stored(&node).await;
        seed(&node).await.unwrap();
        assert_eq!(stored(&node).await, before);
    }

    /// An ephemeral node keeps no store to import into; the import's error is the seed's
    /// source.
    #[tokio::test]
    async fn seeding_an_ephemeral_node_fails_with_the_imports_error_as_its_source() {
        let node = Node::new(Arc::new(SledStorageEngine::new_test().unwrap()), PermissiveAgent::new());
        let error = seed(&node).await.unwrap_err();
        let source = error.source().and_then(|source| source.downcast_ref::<MutationError>());
        assert!(matches!(source, Some(MutationError::NotDurable)), "{error}");
    }

    #[tokio::test]
    async fn seeding_a_seeded_node_again_changes_no_stored_event_or_state() {
        seeding_again_changes_nothing(SledStorageEngine::new_test().unwrap()).await;
        seeding_again_changes_nothing(SqliteStorageEngine::open_in_memory().await.unwrap()).await;
    }

    /// Each model declares its minted id, so on a node that was not seeded it fails to
    /// register, rather than minting definitions of its own.
    #[cfg(feature = "identity")]
    #[tokio::test]
    async fn without_the_seed_a_model_fails_to_register_with_a_typed_error() {
        use crate::identity::Principal;
        use ankurah::{core::schema::registration::RegistrationError, policy::DEFAULT_CONTEXT, Model};

        let node = Node::new_durable(Arc::new(SledStorageEngine::new_test().unwrap()), PermissiveAgent::new());
        node.system.create().await.unwrap();
        let context = node.context_async(DEFAULT_CONTEXT).await.unwrap();
        let declared = EntityId::from_base64(Principal::descriptor().explicit_id.unwrap()).unwrap();
        let error = context.resolve_model_id::<Principal>().await.unwrap_err();
        assert!(matches!(error, RegistrationError::ExplicitModelIdNotFound { model } if model == declared), "{error}");
        assert_eq!(node.catalog.model_labels(), []);
    }

    /// A declaration of a minted model's label that names no id, as a build that bound the
    /// model by its label declared it.
    #[cfg(feature = "identity")]
    #[derive(ankurah::Model, Debug, Clone)]
    #[model(label = "identity.principal")]
    #[model(no_ffi)]
    pub struct Unminted {
        #[active_type(LWW)]
        pub handle: String,
    }

    /// A store can hold a definition that a node minted for a model's label before it was
    /// seeded, as a build that bound the model by its label left it. The model, declaring its
    /// minted id, binds to its minted definition all the same.
    #[cfg(feature = "identity")]
    #[tokio::test]
    async fn a_model_binds_to_its_minted_definition_beside_another_of_its_label() {
        use crate::identity::Principal;
        use ankurah::{policy::DEFAULT_CONTEXT, Model};

        let node = Node::new_durable(Arc::new(SledStorageEngine::new_test().unwrap()), PermissiveAgent::new());
        node.system.create().await.unwrap();
        let context = node.context_async(DEFAULT_CONTEXT).await.unwrap();
        let unminted = context.resolve_model_id::<Unminted>().await.unwrap();
        seed(&node).await.unwrap();
        let declared = EntityId::from_base64(Principal::descriptor().explicit_id.unwrap()).unwrap();
        assert_ne!(unminted, ModelId::EntityId(declared));
        assert_eq!(context.resolve_model_id::<Principal>().await.unwrap(), ModelId::EntityId(declared));
    }
}
