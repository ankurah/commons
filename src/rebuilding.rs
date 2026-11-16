//! The definitions registration writes for a model's declaration, and the creation events
//! that `.akev` files state, rebuilt through Ankurah's public types: each event's operations
//! as registration writes a catalog definition, or a node a record, then the genesis made of
//! those, the entity's time and its nonce. A genesis that names no system root and no author
//! is the same entity, with the same id, on every node.

use std::{collections::BTreeMap, error::Error, fmt};

use ankurah::{
    core::{
        property::backend::{LWWBackend, PropertyBackend},
        schema::ModelStructDescriptor,
    },
    error::MutationError,
    property::PropertyError,
    proto::{AuthorId, Event, Membership, ModelId, Operation, OperationSet, PropertyId, SystemModel, SystemProperty},
    value::Value,
    EntityId, Property,
};

use crate::akev::{Creation, EdgeKinds, ModelDefinitions, PropertyDefinition};

/// Why minted events could not be rebuilt, or are not the events a file states.
#[derive(Debug)]
pub enum RebuildError {
    NotTheMintedEvent { stated: EntityId, rebuilt: EntityId },
    IdCount { stated: usize, rebuilt: usize },
    NoSuchProperty(&'static str),
    Value(Box<PropertyError>),
    Operations(Box<MutationError>),
}

impl fmt::Display for RebuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RebuildError::NotTheMintedEvent { stated, rebuilt } => {
                write!(f, "the line of {stated} rebuilds the event of {rebuilt}: some value or creation field on it is not what was minted")
            }
            RebuildError::IdCount { stated, rebuilt } => write!(f, "{stated} ids are stated for {rebuilt} rebuilt events"),
            RebuildError::NoSuchProperty(name) => write!(f, "the edge kind model's definitions state no property `{name}`"),
            RebuildError::Value(source) => write!(f, "a value does not encode: {source}"),
            RebuildError::Operations(source) => write!(f, "the operations do not encode: {source}"),
        }
    }
}

// Written out rather than derived, so that the source of a boxed error is the error, not its
// box, and a caller can downcast it.
impl Error for RebuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RebuildError::Value(source) => Some(source.as_ref()),
            RebuildError::Operations(source) => Some(source.as_ref()),
            RebuildError::NotTheMintedEvent { .. } | RebuildError::IdCount { .. } | RebuildError::NoSuchProperty(_) => None,
        }
    }
}

/// One of a model's definitions: the model's own, a property's, or the link that gives the
/// model a property, named by the property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Definition<'a> {
    Model,
    Property(&'a str),
    Link(&'a str),
}

impl ModelDefinitions {
    /// The definitions that registering `descriptor` writes on a store that holds none of them,
    /// each created at the time and with the nonce `creation` gives it. The ids the declaration
    /// names and a property's former name are not read: registration reads them only to find
    /// definitions that already exist.
    pub fn declared(descriptor: &ModelStructDescriptor, mut creation: impl FnMut(Definition<'_>) -> Creation) -> Self {
        let properties = descriptor
            .properties
            .iter()
            .map(|declared| PropertyDefinition {
                name: declared.name.to_owned(),
                value_type: declared.value_type.to_owned(),
                backend: declared.backend.to_owned(),
                optional: declared.optional,
                property: creation(Definition::Property(declared.name)),
                link: creation(Definition::Link(declared.name)),
            })
            .collect();
        let model = creation(Definition::Model);
        ModelDefinitions { label: descriptor.label.to_owned(), name: descriptor.name.to_owned(), model, properties }
    }

    /// The creation events of these definitions, in the file's order: the model's, then each
    /// property's followed by its model-property link's. Each names the definitions before it
    /// by the ids their own events derive.
    pub fn events(&self) -> Result<Vec<Event>, RebuildError> {
        self.events_in(None)
    }

    /// The creation events as a node whose system root is `system` would make them.
    fn events_in(&self, system: Option<EntityId>) -> Result<Vec<Event>, RebuildError> {
        let model = genesis(system, &self.model, model_operations(&self.label, &self.name)?);
        let model_id = model.entity_id;
        let mut events = vec![model];
        for property in &self.properties {
            let operations = property_operations(&property.name, &property.backend, &property.value_type, model_id)?;
            let definition = genesis(system, &property.property, operations);
            let link = genesis(system, &property.link, link_operations(model_id, definition.entity_id, property.optional)?);
            events.extend([definition, link]);
        }
        Ok(events)
    }

    /// The creation events, each checked to have the id a file states for its entity: `stated`
    /// as `parse` returns it.
    pub fn events_as_stated(&self, stated: &[EntityId]) -> Result<Vec<Event>, RebuildError> {
        let events = self.events()?;
        check(stated, &events)?;
        Ok(events)
    }
}

impl EdgeKinds {
    /// The creation events of the kinds' records, in the file's order, each a record of the
    /// model `edge_kind` defines.
    pub fn events(&self, edge_kind: &ModelDefinitions) -> Result<Vec<Event>, RebuildError> {
        self.events_in(edge_kind, None)
    }

    /// The creation events as a node whose system root is `system`, holding the definitions of
    /// `edge_kind`, would make them.
    fn events_in(&self, edge_kind: &ModelDefinitions, system: Option<EntityId>) -> Result<Vec<Event>, RebuildError> {
        let definitions = edge_kind.events()?;
        // A property's definition follows the model's, and the definition and link of each
        // property before it.
        let property = |name: &'static str| {
            let index = edge_kind.properties.iter().position(|property| property.name == name).ok_or(RebuildError::NoSuchProperty(name))?;
            Ok::<_, RebuildError>(PropertyId::EntityId(definitions[1 + 2 * index].entity_id))
        };
        let (name, description) = (property("name")?, property("description")?);
        let model = ModelId::EntityId(definitions[0].entity_id);
        self.kinds
            .iter()
            .map(|kind| {
                let values = [(name, value(&kind.name)?), (description, value(&kind.description)?)];
                Ok(genesis(system, &kind.record, operations(model, values)?))
            })
            .collect()
    }

    /// The creation events, each checked to have the id a file states for its record:
    /// `stated` as `parse` returns it.
    pub fn events_as_stated(&self, edge_kind: &ModelDefinitions, stated: &[EntityId]) -> Result<Vec<Event>, RebuildError> {
        let events = self.events(edge_kind)?;
        check(stated, &events)?;
        Ok(events)
    }
}

fn check(stated: &[EntityId], events: &[Event]) -> Result<(), RebuildError> {
    if stated.len() != events.len() {
        return Err(RebuildError::IdCount { stated: stated.len(), rebuilt: events.len() });
    }
    match stated.iter().zip(events).find(|(stated, event)| **stated != event.entity_id) {
        Some((&stated, event)) => Err(RebuildError::NotTheMintedEvent { stated, rebuilt: event.entity_id }),
        None => Ok(()),
    }
}

/// The genesis that `creation` and `operations` make under the system root `system`, naming
/// no author. A minted entity names no root.
fn genesis(system: Option<EntityId>, creation: &Creation, operations: OperationSet) -> Event {
    Event::genesis_from_parts(system, creation.nonce, creation.at, AuthorId::Unknown, operations)
}

/// What registration writes for a model's definition.
fn model_operations(label: &str, name: &str) -> Result<OperationSet, RebuildError> {
    let values = [(SystemProperty::Label, value(&label.to_owned())?), (SystemProperty::Name, value(&name.to_owned())?)];
    operations(ModelId::System(SystemModel::Model), catalog(values))
}

/// What registration writes for the definition of a property it mints for `model`.
fn property_operations(name: &str, backend: &str, value_type: &str, model: EntityId) -> Result<OperationSet, RebuildError> {
    let values = [
        (SystemProperty::Name, value(&name.to_owned())?),
        (SystemProperty::Backend, value(&backend.to_owned())?),
        (SystemProperty::ValueType, value(&value_type.to_owned())?),
        (SystemProperty::MintedFor, value(&Some(model))?),
        (SystemProperty::TargetModel, value(&None::<EntityId>)?),
    ];
    operations(ModelId::System(SystemModel::Property), catalog(values))
}

/// What registration writes for the link that gives `model` the property `property`.
fn link_operations(model: EntityId, property: EntityId, optional: bool) -> Result<OperationSet, RebuildError> {
    let values = [
        (SystemProperty::Model, value(&model)?),
        (SystemProperty::Property, value(&property)?),
        (SystemProperty::Optional, value(&optional)?),
    ];
    operations(ModelId::System(SystemModel::ModelProperty), catalog(values))
}

/// `value` as a node's last-writer-wins backend is given it.
fn value<T: Property>(value: &T) -> Result<Option<Value>, RebuildError> {
    value.into_value().map_err(|error| RebuildError::Value(Box::new(error)))
}

/// The values of a catalog definition, whose properties are the system's own.
fn catalog<const N: usize>(values: [(SystemProperty, Option<Value>); N]) -> [(PropertyId, Option<Value>); N] {
    values.map(|(property, value)| (PropertyId::System(property), value))
}

/// The operations that give an entity `values` and make it a member of `model`.
fn operations(model: ModelId, values: impl IntoIterator<Item = (PropertyId, Option<Value>)>) -> Result<OperationSet, RebuildError> {
    let backend = LWWBackend::new();
    for (property, value) in values {
        backend.set(property, value);
    }
    let name = <LWWBackend as PropertyBackend>::property_backend_name().to_owned();
    let diffs = backend.to_operations().map_err(|error| RebuildError::Operations(Box::new(error)))?.unwrap_or_default();
    let mut operations = OperationSet::from_backends(BTreeMap::from([(name, diffs)]));
    operations.push(Operation::Membership(Membership::Add(model)));
    Ok(operations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ankurah::{policy::DEFAULT_CONTEXT, storage::StorageEngine, Model, Node, PermissiveAgent};
    use ankurah_storage_sled::SledStorageEngine;
    use std::sync::Arc;

    type TestNode = Node<SledStorageEngine, PermissiveAgent>;

    /// The time and nonce of the event that created `id` on `node`, which must be its only one.
    async fn creation(node: &TestNode, id: EntityId) -> (Creation, Event) {
        let [created] = <[_; 1]>::try_from(node.storage.dump_entity_events(id).await.unwrap()).unwrap();
        let created = created.payload;
        (Creation { at: created.timestamp(), nonce: *created.nonce() }, created)
    }

    /// Minted from the same times and nonces, through the text a mint keeps, a model's
    /// definitions rebuild, under the registrar's root, as exactly the events the registrar
    /// wrote when it registered `registered`.
    async fn minted_as_registered(registrar: &TestNode, descriptor: &ModelStructDescriptor, registered: &ankurah::proto::RegisteredModel) {
        let property = |name: &str| registered.properties.iter().find(|property| property.name == name).unwrap();
        let mut written = Vec::new();
        let mut creations = BTreeMap::new();
        let ids =
            [registered.id].into_iter().chain(registered.properties.iter().flat_map(|property| [property.id, property.membership_id]));
        for id in ids {
            let (created, event) = creation(registrar, id).await;
            creations.insert(id, created);
            written.push(event);
        }
        let definitions = ModelDefinitions::declared(descriptor, |definition| match definition {
            Definition::Model => creations[&registered.id],
            Definition::Property(name) => creations[&property(name).id],
            Definition::Link(name) => creations[&property(name).membership_id],
        });
        let (kept, stated) = ModelDefinitions::parse(&definitions.write().unwrap()).unwrap();
        assert_eq!(kept, definitions);
        kept.events_as_stated(&stated).unwrap();
        assert_eq!(kept.events_in(registrar.system.root_id()).unwrap(), written, "{}", descriptor.label);
    }

    /// A durable node that registers declarations sent by the other, which it is connected to.
    async fn registrar() -> (TestNode, TestNode, impl Sized) {
        use ankurah_connector_local_process::LocalProcessConnection;
        let registrar = Node::new_durable(Arc::new(SledStorageEngine::new_test().unwrap()), PermissiveAgent::new());
        registrar.system.create().await.unwrap();
        let requester = Node::new(Arc::new(SledStorageEngine::new_test().unwrap()), PermissiveAgent::new());
        let connection = LocalProcessConnection::new(&requester, &registrar).await.unwrap();
        requester.system.wait_system_ready().await.unwrap();
        (registrar, requester, connection)
    }

    /// Registers `descriptor` on the registrar through the requester, without the ids it
    /// declares, as a node registers a declaration for the first time.
    async fn register(requester: &TestNode, registrar: &TestNode, descriptor: &ModelStructDescriptor) -> ankurah::proto::RegisteredModel {
        use ankurah::proto::{NodeRequestBody, NodeResponseBody, RegisterModel};
        let mut model = RegisterModel::from(descriptor);
        model.explicit_id = None;
        for property in &mut model.properties {
            property.explicit_id = None;
        }
        match requester.request(registrar.id, &DEFAULT_CONTEXT, NodeRequestBody::RegisterSchema { model }).await.unwrap() {
            NodeResponseBody::SchemaRegistered { model } => model,
            other => panic!("registering {}: {other}", descriptor.label),
        }
    }

    /// Every model of the Commons, registered for the first time by a durable node, is what
    /// the mint makes of its declaration, but for the root the node names. Between them the
    /// models declare every value type and both backends a definition holds.
    #[cfg(feature = "full")]
    #[tokio::test]
    async fn every_model_is_minted_as_a_node_registers_it_but_for_the_root() {
        use std::collections::BTreeSet;
        let (registrar, requester, _connection) = registrar().await;
        let mut declared = BTreeSet::new();
        for model in crate::namespaces::NAMESPACES.iter().flat_map(|namespace| namespace.models) {
            let descriptor = model();
            minted_as_registered(&registrar, descriptor, &register(&requester, &registrar, descriptor).await).await;
            declared.extend(descriptor.properties.iter().map(|property| (property.value_type, property.backend)));
        }
        let value_types: BTreeSet<&str> = declared.iter().map(|(value_type, _)| *value_type).collect();
        let backends: BTreeSet<&str> = declared.iter().map(|(_, backend)| *backend).collect();
        assert_eq!(value_types, BTreeSet::from(["bool", "entityid", "f64", "i64", "json", "string"]));
        assert_eq!(backends, BTreeSet::from(["lww", "yrs"]));
    }

    /// An error that does not encode is the source of the error that wraps it.
    #[test]
    fn an_encoding_error_is_the_source_of_the_rebuild_error() {
        let error = RebuildError::Operations(Box::new(MutationError::NotDurable));
        assert!(matches!(error.source().and_then(|source| source.downcast_ref::<MutationError>()), Some(MutationError::NotDurable)));
        let error = RebuildError::Value(Box::new(PropertyError::Missing));
        assert!(matches!(error.source().and_then(|source| source.downcast_ref::<PropertyError>()), Some(PropertyError::Missing)));
    }

    /// A model with a property renamed from a former name.
    #[derive(Model, Debug, Clone)]
    #[model(label = "example.renamed")]
    #[model(no_ffi)]
    pub struct Renamed {
        #[property(renamed_from = "former")]
        #[active_type(LWW)]
        pub title: String,
    }

    /// A store that holds no definition of a model has no property under a former name, so
    /// registration mints the property under the name it has now, as the mint does.
    #[tokio::test]
    async fn a_property_renamed_from_a_former_name_is_minted_as_a_node_registers_it() {
        let (registrar, requester, _connection) = registrar().await;
        let descriptor = Renamed::descriptor();
        minted_as_registered(&registrar, descriptor, &register(&requester, &registrar, descriptor).await).await;
    }

    /// The records of the kinds of edge, created on a node seeded with the kept mint, are what
    /// the mint makes of the table, but for the root the node names.
    #[cfg(feature = "graph")]
    #[tokio::test]
    async fn the_kinds_of_edge_are_minted_as_a_seeded_node_creates_them_but_for_the_root() {
        use crate::akev::EdgeKindRecord;
        use crate::graph::{EdgeKind, EDGE_KINDS};

        let node = Node::new_durable(Arc::new(SledStorageEngine::new_test().unwrap()), PermissiveAgent::new());
        node.system.create().await.unwrap();
        crate::seed(&node).await.unwrap();
        let transaction = node.context_async(DEFAULT_CONTEXT).await.unwrap().begin();
        let mut created = Vec::new();
        for &(name, description) in EDGE_KINDS {
            let kind = EdgeKind { name: name.to_owned(), description: description.to_owned() };
            created.push((name, description, transaction.create(&kind).await.unwrap().id()));
        }
        transaction.commit().await.unwrap();

        let mut kinds = Vec::new();
        let mut written = Vec::new();
        for (name, description, id) in created {
            let (record, event) = creation(&node, id).await;
            kinds.push(EdgeKindRecord { name: name.to_owned(), description: description.to_owned(), record });
            written.push(event);
        }
        let (edge_kind, _) = ModelDefinitions::parse(include_str!("graph/edge_kind.akev")).unwrap();
        let kinds = EdgeKinds { kinds };
        let (kept, stated) = EdgeKinds::parse(&kinds.write(&edge_kind).unwrap()).unwrap();
        assert_eq!(kept, kinds);
        kept.events_as_stated(&edge_kind, &stated).unwrap();
        assert_eq!(kept.events_in(&edge_kind, node.system.root_id()).unwrap(), written);
    }
}
