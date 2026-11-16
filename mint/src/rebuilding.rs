//! Rebuilds the creation events an `.akev` file states, through Ankurah's public types:
//! each event's operations from the values on its line, as a node's last-writer-wins
//! backend writes them, then the event from those, its minter, its time and its nonce.
//! Each rebuilt event must have the id its line gives, since an entity's id is the hash of
//! its creation event.
// OPEN: the state each event makes, which a seeded store also needs, is not rebuilt here:
// Ankurah gives no public way to apply an event to an empty entity (`EventLayer`, which a
// backend applies, has a crate-private constructor).

use std::collections::BTreeMap;

use ankurah::{
    core::property::backend::{LWWBackend, PropertyBackend},
    error::MutationError,
    property::PropertyError,
    proto::{Event, Identity, Membership, ModelId, Operation, OperationSet, PropertyId, SystemModel, SystemProperty},
    value::Value,
    EntityId, Property,
};
use ankurah_commons::akev::{Creation, EdgeKinds, ModelDefinitions};
use thiserror::Error;

/// Why an event could not be rebuilt from its line.
#[derive(Debug, Error)]
pub enum RebuildError {
    #[error("the line of {stated} rebuilds the event of {rebuilt}: some value or creation field on it is not what was minted")]
    NotTheMintedEvent { stated: EntityId, rebuilt: EntityId },
    #[error("the edge kind model's definitions state no property `{0}`")]
    NoSuchProperty(&'static str),
    #[error("a value does not encode: {0}")]
    Value(Box<PropertyError>),
    #[error("the operations do not encode: {0}")]
    Operations(Box<MutationError>),
}

/// The creation events of a model's definitions, in the file's order: the model's, then
/// each property's followed by its model-property link's.
pub fn model_events(definitions: &ModelDefinitions) -> Result<Vec<Event>, RebuildError> {
    let minter = Identity::Ed25519(definitions.minter);
    let model = definitions.model.id;
    let values = [(SystemProperty::Label, value(&definitions.label)?), (SystemProperty::Name, value(&definitions.name)?)];
    let mut events = vec![rebuild(minter, &definitions.model, ModelId::System(SystemModel::Model), catalog(values))?];
    for property in &definitions.properties {
        let values = [
            (SystemProperty::Name, value(&property.name)?),
            (SystemProperty::Backend, value(&property.backend)?),
            (SystemProperty::ValueType, value(&property.value_type)?),
            (SystemProperty::MintedFor, value(&Some(model))?),
            (SystemProperty::TargetModel, value(&None::<EntityId>)?),
        ];
        events.push(rebuild(minter, &property.property, ModelId::System(SystemModel::Property), catalog(values))?);
        let values = [
            (SystemProperty::Model, value(&model)?),
            (SystemProperty::Property, value(&property.property.id)?),
            (SystemProperty::Optional, value(&property.optional)?),
        ];
        events.push(rebuild(minter, &property.link, ModelId::System(SystemModel::ModelProperty), catalog(values))?);
    }
    Ok(events)
}

/// The creation events of the edge kinds' records, each a record of the model that
/// `edge_kind` defines.
pub fn edge_kind_events(kinds: &EdgeKinds, edge_kind: &ModelDefinitions) -> Result<Vec<Event>, RebuildError> {
    let property = |name: &'static str| {
        let property = edge_kind.properties.iter().find(|property| property.name == name).ok_or(RebuildError::NoSuchProperty(name))?;
        Ok::<_, RebuildError>(PropertyId::EntityId(property.property.id))
    };
    let (name, description) = (property("name")?, property("description")?);
    let model = ModelId::EntityId(edge_kind.model.id);
    kinds
        .kinds
        .iter()
        .map(|kind| {
            let values = [(name, value(&kind.name)?), (description, value(&kind.description)?)];
            rebuild(Identity::Ed25519(kinds.minter), &kind.record, model, values)
        })
        .collect()
}

/// `value` as a node's last-writer-wins backend is given it.
fn value<T: Property>(value: &T) -> Result<Option<Value>, RebuildError> {
    value.into_value().map_err(|error| RebuildError::Value(Box::new(error)))
}

/// The values of a catalog definition, whose properties are the system's own.
fn catalog<const N: usize>(values: [(SystemProperty, Option<Value>); N]) -> [(PropertyId, Option<Value>); N] {
    values.map(|(property, value)| (PropertyId::System(property), value))
}

/// The creation event that gives an entity `values` and makes it a member of `model`,
/// checked against the id its line gives.
fn rebuild(
    minter: Identity,
    creation: &Creation,
    model: ModelId,
    values: impl IntoIterator<Item = (PropertyId, Option<Value>)>,
) -> Result<Event, RebuildError> {
    let backend = LWWBackend::new();
    for (property, value) in values {
        backend.set(property, value);
    }
    let name = <LWWBackend as PropertyBackend>::property_backend_name().to_owned();
    let mut operations = OperationSet::from_backends(BTreeMap::from([(
        name,
        backend.to_operations().map_err(|error| RebuildError::Operations(Box::new(error)))?.unwrap_or_default(),
    )]));
    operations.push(Operation::Membership(Membership::Add(model)));
    let event = Event::genesis_from_parts(minter, creation.nonce, creation.at, operations);
    match event.entity_id == creation.id {
        true => Ok(event),
        false => Err(RebuildError::NotTheMintedEvent { stated: creation.id, rebuilt: event.entity_id }),
    }
}
