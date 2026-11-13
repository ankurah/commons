//! Mints the definitions of every listed model: registers each model on a durable node
//! built on a throwaway store under a key drawn for the run, reads back through the node's
//! catalog what registration defined, and takes from the store, as the store gives them,
//! the event that created each definition and the definition's state. A node's catalog is
//! built from states, so a store is seeded with both.
//!
//! Registration creates three kinds of definition entity: one for the model, one for each
//! of its properties, and one model-property link for each property, which gives the
//! model the property and says whether it is optional.

use std::sync::Arc;

use ankurah::{
    core::{node::DuplicatePluginName, schema::registration::RegistrationError},
    error::{NodeReadinessError, RetrievalError},
    policy::DEFAULT_CONTEXT,
    proto::{Attested, EntityId, EntityState, Event},
    storage::StorageEngine,
    Context, ModelId, Node, NodeKey, PermissiveAgent, PropertyId,
};
use ankurah_storage_sled::SledStorageEngine;
use thiserror::Error;

use crate::namespaces::{DeclaredModel, Namespace};

type MintingNode = Node<SledStorageEngine, PermissiveAgent>;

/// What minting one namespace produced: its models as the node's catalog lists them, and
/// the definitions registration created, each with its creation event and its state.
pub struct MintedNamespace {
    pub name: &'static str,
    pub models: Vec<ListedModel>,
    pub definitions: Vec<MintedDefinition>,
}

/// A model as the node's catalog lists it, with the ids of its definition and of its
/// properties'.
pub struct ListedModel {
    pub label: String,
    pub id: EntityId,
    pub properties: Vec<ListedProperty>,
}

/// A property as the node's catalog lists it for its model, with the ids of its
/// definition and of its model-property link.
pub struct ListedProperty {
    pub name: String,
    pub backend: String,
    pub value_type: String,
    pub optional: bool,
    pub id: EntityId,
    pub link_id: EntityId,
}

/// One definition entity: its id, the event that created it with the empty set of
/// attestations the store gave it, and its state. Keeping the set in the event's file
/// keeps the file's format, and every id, unchanged if statements are added later; the
/// store keeps a state without attestations.
pub struct MintedDefinition {
    pub id: EntityId,
    pub creation: Attested<Event>,
    pub state: EntityState,
}

/// Why minting stopped.
#[derive(Debug, Error)]
pub enum MintError {
    #[error("the throwaway store's directory: {0}")]
    StoreDirectory(std::io::Error),
    #[error("the throwaway store did not open: {0:#}")]
    Store(anyhow::Error),
    #[error(transparent)]
    NodeBuild(#[from] DuplicatePluginName),
    #[error("the minting node did not become ready: {0}")]
    NodeReadiness(#[from] NodeReadinessError),
    #[error("registering {label}: {source}")]
    Registration { label: &'static str, source: Box<RegistrationError> },
    #[error("{label} registered as a system model, which has no definition entity")]
    SystemModel { label: &'static str },
    #[error("reading the minting node's catalog or store: {0}")]
    Retrieval(#[from] RetrievalError),
    #[error("registering {label} returned the model {id}, which the catalog does not hold")]
    ModelNotInCatalog { label: &'static str, id: EntityId },
    #[error("the catalog holds no definition of {label}'s property `{property}`")]
    PropertyNotInCatalog { label: &'static str, property: &'static str },
    #[error("the catalog holds no link from {label} to its property `{property}`")]
    LinkNotInCatalog { label: &'static str, property: &'static str },
    #[error("the store holds {count} events of the definition {id}; a fresh registration creates a definition in one")]
    NotOneEvent { id: EntityId, count: usize },
    #[error("the one event the store holds of the definition {id} does not create it")]
    NotCreation { id: EntityId },
    #[error("the store gave the creation of the definition {id} {count} attestations; this tool writes none")]
    Attested { id: EntityId, count: usize },
}

/// Registers every model of `namespaces` on a durable node, and returns what it minted.
/// The node's store is a temporary directory, removed before this returns.
pub async fn mint(namespaces: &[Namespace]) -> Result<Vec<MintedNamespace>, MintError> {
    let store = tempfile::tempdir().map_err(MintError::StoreDirectory)?;
    let minted = {
        let engine = SledStorageEngine::with_path(store.path().to_owned()).map_err(MintError::Store)?;
        // Every creation event names a minter, because Ankurah hashes one into the event's
        // id. Each run draws a fresh key for it and discards it, so the minter is nobody's
        // identity and nothing is signed. The key is drawn here rather than left to the
        // builder, which would note that a durable node's identity will not survive a
        // restart; this one is not meant to.
        let key = NodeKey::generate();
        let node = Node::builder(Arc::new(engine), PermissiveAgent::new()).durable().key(key).build()?;
        let context = node.context_async(DEFAULT_CONTEXT).await?;
        let mut minted = Vec::with_capacity(namespaces.len());
        for namespace in namespaces {
            let mut namespace_minted = MintedNamespace { name: namespace.name, models: Vec::new(), definitions: Vec::new() };
            for model in namespace.models {
                mint_model(&node, &context, model, &mut namespace_minted).await?;
            }
            minted.push(namespace_minted);
        }
        minted
    };
    store.close().map_err(MintError::StoreDirectory)?;
    Ok(minted)
}

/// Registers `model`, and adds to `minted` the model as the catalog lists it and the
/// definitions registering it created: the model's, then each property's and its link's.
async fn mint_model(node: &MintingNode, context: &Context, model: &DeclaredModel, minted: &mut MintedNamespace) -> Result<(), MintError> {
    let descriptor = (model.descriptor)();
    let label = descriptor.label;
    let registered = (model.register)(context).await.map_err(|source| MintError::Registration { label, source: Box::new(source) })?;
    let ModelId::EntityId(model_id) = registered else {
        return Err(MintError::SystemModel { label });
    };
    let catalog = &node.catalog;
    let model_row = catalog.model_by_id(&model_id)?.ok_or(MintError::ModelNotInCatalog { label, id: model_id })?;
    minted.definitions.push(definition(node, model_id).await?);

    let mut properties = Vec::with_capacity(descriptor.properties.len());
    for declared in descriptor.properties {
        let property = declared.name;
        let Some(PropertyId::EntityId(property_id)) = catalog.property_id(&registered, property)? else {
            return Err(MintError::PropertyNotInCatalog { label, property });
        };
        let property_row = catalog.property_by_id(&property_id)?.ok_or(MintError::PropertyNotInCatalog { label, property })?;
        let (link_id, link) = catalog.membership(&model_id, &property_id)?.ok_or(MintError::LinkNotInCatalog { label, property })?;
        minted.definitions.push(definition(node, property_id).await?);
        minted.definitions.push(definition(node, link_id).await?);
        properties.push(ListedProperty {
            name: property_row.name,
            backend: property_row.backend,
            value_type: property_row.value_type,
            optional: link.optional,
            id: property_id,
            link_id,
        });
    }
    minted.models.push(ListedModel { label: model_row.label, id: model_id, properties });
    Ok(())
}

/// The definition `id` with its creation event, which must carry no attestation, and its
/// state.
async fn definition(node: &MintingNode, id: EntityId) -> Result<MintedDefinition, MintError> {
    let events = node.storage.dump_entity_events(id).await?;
    let [creation]: [Attested<Event>; 1] =
        events.try_into().map_err(|events: Vec<_>| MintError::NotOneEvent { id, count: events.len() })?;
    if !creation.payload.is_entity_create() {
        return Err(MintError::NotCreation { id });
    }
    if !creation.attestations.is_empty() {
        return Err(MintError::Attested { id, count: creation.attestations.len() });
    }
    let state = node.storage.get_state(id).await?;
    Ok(MintedDefinition { id, creation, state })
}
