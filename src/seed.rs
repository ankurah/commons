//! Seeding a node: the kept mint's creation events, rebuilt from the `.akev` text compiled
//! into this crate, imported into a durable node's store, so that the Commons' models bind
//! to the minted definitions there rather than minting their own.

use ankurah::{
    error::MutationError,
    policy::PolicyAgent,
    proto::{Attested, Event},
    storage::StorageEngine,
    Node,
};
use thiserror::Error;

use crate::{
    akev::{EdgeKinds, ModelDefinitions, ParseError},
    namespaces::Declaration,
    rebuilding::RebuildError,
};

/// Why a node was not seeded. The kept mint is compiled into the crate and its tests
/// rebuild all of it, so in a build whose tests pass only the import can fail.
#[derive(Debug, Error)]
pub enum SeedError {
    #[error("the kept mint of the {namespace} namespace does not parse: {source}")]
    Unparsable { namespace: &'static str, source: ParseError },
    #[error("the kept mint of the {namespace} namespace does not rebuild: {source}")]
    NotRebuilt { namespace: &'static str, source: RebuildError },
    #[error("the node did not import the kept mint: {0}")]
    Import(Box<MutationError>),
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

/// The kept mint of one namespace, as compiled into the crate: the text of each of its
/// models' `.akev` files, and of the records of the kinds of edge it mints.
struct Kept {
    namespace: &'static str,
    models: &'static [&'static str],
    edge_kinds: Option<KeptEdgeKinds>,
}

struct KeptEdgeKinds {
    /// The model whose records they are.
    model: Declaration,
    akev: &'static str,
}

/// The kept mint of every namespace this build enables, as compiled into the crate.
const KEPT_MINT: &[Kept] = &[
    #[cfg(feature = "identity")]
    Kept {
        namespace: "identity",
        models: &[include_str!("identity/identity_binding.akev"), include_str!("identity/principal.akev")],
        edge_kinds: None,
    },
    #[cfg(feature = "records")]
    Kept {
        namespace: "records",
        models: &[
            include_str!("records/deferred.akev"),
            include_str!("records/fractional_position.akev"),
            include_str!("records/lifecycle/acceptance.akev"),
            include_str!("records/lifecycle/checked.akev"),
            include_str!("records/lifecycle/closed.akev"),
            include_str!("records/lifecycle/created.akev"),
            include_str!("records/lifecycle/edited.akev"),
            include_str!("records/note.akev"),
            include_str!("records/provenance/attribution.akev"),
            include_str!("records/provenance/via.akev"),
            include_str!("records/responsibility/assignment.akev"),
            include_str!("records/responsibility/ownership.akev"),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "health")]
    Kept {
        namespace: "health",
        models: &[
            include_str!("health/activity_interval.akev"),
            include_str!("health/distance.akev"),
            include_str!("health/heart_rate.akev"),
            include_str!("health/heart_rate_summary.akev"),
            include_str!("health/physical_activity.akev"),
            include_str!("health/workout.akev"),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "text")]
    Kept { namespace: "text", models: &[include_str!("text/fragment.akev"), include_str!("text/quotation.akev")], edge_kinds: None },
    #[cfg(feature = "git")]
    Kept {
        namespace: "git",
        models: &[
            include_str!("git/checkout_state.akev"),
            include_str!("git/file_anchor.akev"),
            include_str!("git/repository.akev"),
            include_str!("git/work_location.akev"),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "graph")]
    Kept {
        namespace: "graph",
        models: &[include_str!("graph/dyn_edge.akev"), include_str!("graph/edge_kind.akev")],
        edge_kinds: Some(KeptEdgeKinds {
            model: crate::namespaces::declared::<crate::graph::EdgeKind>(),
            akev: include_str!("graph/edge_kinds.akev"),
        }),
    },
    #[cfg(feature = "cognition")]
    Kept {
        namespace: "cognition",
        models: &[
            include_str!("cognition/claim.akev"),
            include_str!("cognition/decision.akev"),
            include_str!("cognition/expectation.akev"),
            include_str!("cognition/intention.akev"),
            include_str!("cognition/intuition.akev"),
            include_str!("cognition/perspective.akev"),
        ],
        edge_kinds: None,
    },
    #[cfg(feature = "work")]
    Kept {
        namespace: "work",
        models: &[include_str!("work/scope.akev"), include_str!("work/submission.akev"), include_str!("work/task.akev")],
        edge_kinds: None,
    },
    #[cfg(feature = "messaging")]
    Kept {
        namespace: "messaging",
        models: &[include_str!("messaging/mention.akev"), include_str!("messaging/message.akev")],
        edge_kinds: None,
    },
];

/// The creation events of the kept mint of every namespace this build enables.
fn kept_mint() -> Result<Vec<Attested<Event>>, SeedError> {
    let mut events = Vec::new();
    for kept in KEPT_MINT {
        let namespace = kept.namespace;
        let mut models = Vec::with_capacity(kept.models.len());
        for text in kept.models {
            let (definitions, ids) = ModelDefinitions::parse(text).map_err(|source| SeedError::Unparsable { namespace, source })?;
            events.extend(definitions.events_as_stated(&ids).map_err(|source| SeedError::NotRebuilt { namespace, source })?);
            models.push(definitions);
        }
        if let Some(edge_kinds) = &kept.edge_kinds {
            let label = (edge_kinds.model)().label;
            let edge_kind =
                models.iter().find(|definitions| definitions.label == label).expect("a namespace mints records of its own models");
            let (records, ids) = EdgeKinds::parse(edge_kinds.akev).map_err(|source| SeedError::Unparsable { namespace, source })?;
            events.extend(records.events_as_stated(edge_kind, &ids).map_err(|source| SeedError::NotRebuilt { namespace, source })?);
        }
    }
    Ok(events.into_iter().map(Attested::from).collect())
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
