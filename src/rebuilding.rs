//! Rebuilds the creation events that `.akev` files state, through Ankurah's public types:
//! each event's operations as registration writes a catalog definition, or a node a record,
//! then the genesis made of those, the entity's time and its nonce. A genesis that names no
//! system root and no author is the same entity, with the same id, on every node.

use std::collections::BTreeMap;

use ankurah::{
    core::property::backend::{LWWBackend, PropertyBackend},
    error::MutationError,
    property::PropertyError,
    proto::{AuthorId, Event, Membership, ModelId, Operation, OperationSet, PropertyId, SystemModel, SystemProperty},
    value::Value,
    EntityId, Property,
};
use thiserror::Error;

use crate::akev::{Creation, EdgeKinds, ModelDefinitions};

/// Why minted events could not be rebuilt, or are not the events a file states.
#[derive(Debug, Error)]
pub enum RebuildError {
    #[error("the line of {stated} rebuilds the event of {rebuilt}: some value or creation field on it is not what was minted")]
    NotTheMintedEvent { stated: EntityId, rebuilt: EntityId },
    #[error("{stated} ids are stated for {rebuilt} rebuilt events")]
    IdCount { stated: usize, rebuilt: usize },
    #[error("the edge kind model's definitions state no property `{0}`")]
    NoSuchProperty(&'static str),
    #[error("a value does not encode: {0}")]
    Value(Box<PropertyError>),
    #[error("the operations do not encode: {0}")]
    Operations(Box<MutationError>),
}

impl ModelDefinitions {
    /// The creation events of these definitions, in the file's order: the model's, then each
    /// property's followed by its model-property link's. Each names the definitions before it
    /// by the ids their own events derive.
    pub fn events(&self) -> Result<Vec<Event>, RebuildError> {
        let model = genesis(&self.model, model_operations(&self.label, &self.name)?);
        let model_id = model.entity_id;
        let mut events = vec![model];
        for property in &self.properties {
            let operations = property_operations(&property.name, &property.backend, &property.value_type, model_id)?;
            let definition = genesis(&property.property, operations);
            let link = genesis(&property.link, link_operations(model_id, definition.entity_id, property.optional)?);
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
                Ok(genesis(&kind.record, operations(model, values)?))
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

/// The genesis that `creation` and `operations` make, naming no system root and no author.
fn genesis(creation: &Creation, operations: OperationSet) -> Event {
    Event::genesis_from_parts(None, creation.nonce, creation.at, AuthorId::Unknown, operations)
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

    /// A model that a node registers itself, by its label.
    #[derive(Model, Debug, Clone)]
    #[model(label = "example.sample")]
    #[model(no_ffi)]
    pub struct Sample {
        #[active_type(LWW)]
        pub title: String,
        #[active_type(LWW)]
        pub body: Option<String>,
    }

    /// Registration mints a model's definitions, and a transaction creates a record, under the
    /// node's system root. With that root in place of none, the events rebuilt here from the
    /// same values, times and nonces are exactly the node's.
    #[tokio::test]
    async fn rebuilt_events_are_what_a_node_writes_but_for_the_root_they_name() {
        let node = Node::new_durable(Arc::new(SledStorageEngine::new_test().unwrap()), PermissiveAgent::new());
        node.system.create().await.unwrap();
        let context = node.context_async(DEFAULT_CONTEXT).await.unwrap();
        let ModelId::EntityId(model) = context.resolve_model_id::<Sample>().await.unwrap() else { panic!("a model registered by label") };
        let transaction = context.begin();
        let record = transaction.create(&Sample { title: "a title".to_owned(), body: Some("a body".to_owned()) }).await.unwrap().id();
        transaction.commit().await.unwrap();

        let root = node.system.root_id();
        let node = &node;
        let written_as = |id: EntityId, operations: OperationSet| async move {
            let [written] = <[_; 1]>::try_from(node.storage.dump_entity_events(id).await.unwrap()).unwrap();
            let written = written.payload;
            assert_eq!(written, Event::genesis_from_parts(root, *written.nonce(), written.timestamp(), AuthorId::Unknown, operations));
        };
        written_as(model, model_operations("example.sample", "Sample").unwrap()).await;
        let mut values = Vec::new();
        for (name, optional) in [("title", false), ("body", true)] {
            let (property, row) = node.catalog.property_by_name(&model, name).unwrap().unwrap();
            written_as(property, property_operations(name, &row.backend, &row.value_type, model).unwrap()).await;
            let (link, _) = node.catalog.membership(&model, &property).unwrap().unwrap();
            written_as(link, link_operations(model, property, optional).unwrap()).await;
            values.push((PropertyId::EntityId(property), Some(Value::String(format!("a {name}")))));
        }
        written_as(record, operations(ModelId::EntityId(model), values).unwrap()).await;
    }
}
