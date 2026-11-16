//! Mints the definitions of every listed model, and the listed kinds of edge: registers
//! each model on a durable node built on a throwaway store under a key drawn for the run,
//! reads back through the node's catalog what registration defined, creates an `EdgeKind`
//! record for each kind, and takes from the store the event that created each entity.
//!
//! Registration creates three kinds of definition entity: one for the model, one for each
//! of its properties, and one model-property link for each property, which gives the
//! model the property and says whether it is optional.

use std::sync::Arc;

use ankurah::{
    core::{node::DuplicatePluginName, schema::registration::RegistrationError},
    error::{MutationError, NodeReadinessError, RetrievalError},
    policy::DEFAULT_CONTEXT,
    proto::{Attested, EntityId, Event},
    storage::StorageEngine,
    Context, ModelId, Node, NodeKey, PermissiveAgent, PropertyId,
};
use ankurah_commons::graph::EdgeKind;
use ankurah_storage_sled::SledStorageEngine;
use thiserror::Error;

use crate::namespaces::{DeclaredModel, Namespace};

type MintingNode = Node<SledStorageEngine, PermissiveAgent>;

/// What minting one namespace produced: its models as the node's catalog lists them, the
/// kinds of edge minted with it, and every entity minted for it, with its creation event.
pub struct MintedNamespace {
    pub name: &'static str,
    pub models: Vec<ListedModel>,
    pub edge_kinds: Vec<ListedEdgeKind>,
    /// The definitions registration created, then the edge kinds' records.
    pub entities: Vec<MintedEntity>,
}

/// A model as the node's catalog lists it, with the ids of its definition and of its
/// properties'.
pub struct ListedModel {
    pub label: String,
    /// The name of the model's struct.
    pub name: String,
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

/// A kind of edge as it was minted: its name and description, and the id of its
/// `EdgeKind` record.
pub struct ListedEdgeKind {
    pub name: &'static str,
    pub description: &'static str,
    pub id: EntityId,
}

/// One minted entity, a definition or a record: its id, and the event that created it.
pub struct MintedEntity {
    pub id: EntityId,
    pub creation: Event,
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
    #[error("{label}'s property `{property}` was minted for another model or names a target model, which an .akev file cannot state")]
    PropertyNotWritable { label: &'static str, property: &'static str },
    #[error("creating the edge kind `{name}`: {source}")]
    EdgeKind { name: &'static str, source: Box<MutationError> },
    #[error("committing the edge kinds: {0}")]
    EdgeKindsCommit(Box<MutationError>),
    #[error("the store holds {count} events of the minted entity {id}; a fresh store creates each in one")]
    NotOneEvent { id: EntityId, count: usize },
    #[error("the one event the store holds of the minted entity {id} does not create it")]
    NotCreation { id: EntityId },
    #[error("the store gave the creation of the minted entity {id} {count} attestations; this tool writes none")]
    Attested { id: EntityId, count: usize },
}

/// Registers every model of `namespaces` on a durable node and creates their kinds of
/// edge, and returns what it minted. The node's store is a temporary directory, removed
/// before this returns.
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
            let mut namespace_minted =
                MintedNamespace { name: namespace.name, models: Vec::new(), edge_kinds: Vec::new(), entities: Vec::new() };
            for model in namespace.models {
                mint_model(&node, &context, model, &mut namespace_minted).await?;
            }
            mint_edge_kinds(&node, &context, namespace.edge_kinds, &mut namespace_minted).await?;
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
    minted.entities.push(minted_entity(node, model_id).await?);

    let mut properties = Vec::with_capacity(descriptor.properties.len());
    for declared in descriptor.properties {
        let property = declared.name;
        let Some(PropertyId::EntityId(property_id)) = catalog.property_id(&registered, property)? else {
            return Err(MintError::PropertyNotInCatalog { label, property });
        };
        let property_row = catalog.property_by_id(&property_id)?.ok_or(MintError::PropertyNotInCatalog { label, property })?;
        if property_row.minted_for != Some(model_id) || property_row.target_model.is_some() {
            return Err(MintError::PropertyNotWritable { label, property });
        }
        let (link_id, link) = catalog.membership(&model_id, &property_id)?.ok_or(MintError::LinkNotInCatalog { label, property })?;
        minted.entities.push(minted_entity(node, property_id).await?);
        minted.entities.push(minted_entity(node, link_id).await?);
        properties.push(ListedProperty {
            name: property_row.name,
            backend: property_row.backend,
            value_type: property_row.value_type,
            optional: link.optional,
            id: property_id,
            link_id,
        });
    }
    minted.models.push(ListedModel { label: model_row.label, name: model_row.name, id: model_id, properties });
    Ok(())
}

/// Creates an `EdgeKind` record for each of `kinds`, its name and description, in one
/// transaction, and adds to `minted` each kind with its record's id, and each record.
async fn mint_edge_kinds(
    node: &MintingNode,
    context: &Context,
    kinds: &'static [(&'static str, &'static str)],
    minted: &mut MintedNamespace,
) -> Result<(), MintError> {
    if kinds.is_empty() {
        return Ok(());
    }
    let transaction = context.begin();
    let mut created = Vec::with_capacity(kinds.len());
    for &(name, description) in kinds {
        let kind = EdgeKind { name: name.to_owned(), description: description.to_owned() };
        let record = transaction.create(&kind).await.map_err(|source| MintError::EdgeKind { name, source: Box::new(source) })?;
        created.push(ListedEdgeKind { name, description, id: record.id() });
    }
    transaction.commit().await.map_err(|source| MintError::EdgeKindsCommit(Box::new(source)))?;
    for kind in &created {
        minted.entities.push(minted_entity(node, kind.id).await?);
    }
    minted.edge_kinds.extend(created);
    Ok(())
}

/// The minted entity `id` with its creation event, which must carry no attestation, since
/// an .akev file states none.
async fn minted_entity(node: &MintingNode, id: EntityId) -> Result<MintedEntity, MintError> {
    let events = node.storage.dump_entity_events(id).await?;
    let [creation]: [Attested<Event>; 1] =
        events.try_into().map_err(|events: Vec<_>| MintError::NotOneEvent { id, count: events.len() })?;
    if !creation.payload.is_entity_create() {
        return Err(MintError::NotCreation { id });
    }
    if !creation.attestations.is_empty() {
        return Err(MintError::Attested { id, count: creation.attestations.len() });
    }
    Ok(MintedEntity { id, creation: creation.payload })
}
