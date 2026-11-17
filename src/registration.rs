//! The creation events that registering a model's declaration writes, made without a node:
//! a definition of the model, and for each property a definition of the property and the
//! link that gives the model the property. A node that imports them binds the model to
//! them, as if it had registered the declaration itself.

use std::collections::BTreeMap;

use ankurah::{
    core::{
        property::backend::{LWWBackend, PropertyBackend},
        schema::ModelStructDescriptor,
    },
    proto::{Event, ModelId, OperationSet, SystemModel, SystemProperty},
    value::Value,
    EntityId, PropertyId,
};

use crate::akev::{BackendOperations, BuildError, Genesis, Operations};

/// One of the entities that registering a model's declaration creates: the model's
/// definition, a property's, or the link that gives the model a property, named by the
/// property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Definition {
    Model,
    Property(&'static str),
    Link(&'static str),
}

/// The geneses of the definitions that registering `descriptor` writes on a store that holds
/// none of them: the model's, then each property's followed by its link's, each naming the
/// definitions before it by their ids. `creation` gives each definition its nonce and
/// timestamp, seeing the operations it is created with. The ids the declaration names and a
/// property's former name are not read: registration reads them only to find definitions
/// that already exist.
pub fn definitions(
    descriptor: &ModelStructDescriptor,
    mut creation: impl FnMut(Definition, &OperationSet) -> ([u8; 32], u64),
) -> Result<Vec<Genesis>, BuildError> {
    let mut geneses = Vec::with_capacity(1 + 2 * descriptor.properties.len());
    let mut create = |definition, operations: Operations| -> Result<EntityId, BuildError> {
        let built = operations.build()?;
        let (nonce, timestamp) = creation(definition, &built);
        geneses.push(Genesis { nonce, timestamp, operations });
        Ok(Event::genesis_from_parts(nonce, timestamp, built).entity_id)
    };
    let text = |text: &str| Some(Value::String(text.to_owned()));
    let model = create(
        Definition::Model,
        catalog(SystemModel::Model, [(SystemProperty::Label, text(descriptor.label)), (SystemProperty::Name, text(descriptor.name))]),
    )?;
    for declared in descriptor.properties {
        let values = [
            (SystemProperty::Name, text(declared.name)),
            (SystemProperty::Backend, text(declared.backend)),
            (SystemProperty::ValueType, text(declared.value_type)),
            (SystemProperty::MintedFor, Some(Value::EntityId(model))),
            // Registration writes no target, even for a reference (Ankurah issue 513).
            (SystemProperty::TargetModel, None),
        ];
        let property = create(Definition::Property(declared.name), catalog(SystemModel::Property, values))?;
        let values = [
            (SystemProperty::Model, Some(Value::EntityId(model))),
            (SystemProperty::Property, Some(Value::EntityId(property))),
            (SystemProperty::Optional, Some(Value::Bool(declared.optional))),
        ];
        create(Definition::Link(declared.name), catalog(SystemModel::ModelProperty, values))?;
    }
    Ok(geneses)
}

/// The operations of a catalog definition, a member of the system's model `model` whose
/// values are the system's own properties, set in the order registration sets them.
fn catalog<const N: usize>(model: SystemModel, values: [(SystemProperty, Option<Value>); N]) -> Operations {
    let values = values.into_iter().map(|(property, value)| (PropertyId::System(property), value)).collect();
    let lww = <LWWBackend as PropertyBackend>::property_backend_name().to_owned();
    Operations { backends: BTreeMap::from([(lww, BackendOperations::Values(values))]), memberships: vec![ModelId::System(model)] }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::akev;
    use ankurah::{
        policy::DEFAULT_CONTEXT,
        proto::{AuthorId, Clock, EventBody, EventId, RegisteredModel},
        storage::StorageEngine,
        Model, Node, PermissiveAgent,
    };
    use ankurah_storage_sled::SledStorageEngine;
    use std::{collections::HashMap, path::Path, sync::Arc};

    type TestNode = Node<SledStorageEngine, PermissiveAgent>;

    /// The event that created `id` on `node`, which must be its only one.
    async fn created(node: &TestNode, id: EntityId) -> Event {
        let [created] = <[_; 1]>::try_from(node.storage.dump_entity_events(id).await.unwrap()).unwrap();
        created.payload
    }

    /// The geneses a node whose system root is `root` makes of `geneses`, naming no author:
    /// the same values, each id of an earlier genesis among them taken as the node takes it.
    /// `Event::genesis_from_parts` names no root, so this builds each event from the parts
    /// its id derives from.
    fn under(root: Option<EntityId>, geneses: &[Genesis]) -> Vec<Event> {
        let mut rooted = HashMap::new();
        let mut events = Vec::new();
        for genesis in geneses {
            let mut operations = genesis.operations.clone();
            for values in operations.backends.values_mut() {
                let BackendOperations::Values(values) = values else { continue };
                for (property, value) in values {
                    if let PropertyId::EntityId(id) = property {
                        *id = rooted[id];
                    }
                    if let Some(Value::EntityId(id)) = value {
                        *id = rooted[id];
                    }
                }
            }
            let (nonce, timestamp, author, operations) = (genesis.nonce, genesis.timestamp, AuthorId::Unknown, operations.build().unwrap());
            let entity_id = EventId::from_genesis_parts(&root, &nonce, timestamp, &author, &operations).into();
            rooted.insert(genesis.event().unwrap().entity_id, entity_id);
            let body = EventBody::Genesis { system: root, nonce, timestamp, author, operations };
            events.push(Event { entity_id, parent: Clock::default(), body });
        }
        events
    }

    /// Made with the nonces and timestamps the registrar drew, a declaration's definitions
    /// are, under the registrar's root, exactly the events the registrar wrote when it
    /// registered the declaration as `registered`; and their text reads back as themselves.
    async fn made_as_registered(registrar: &TestNode, descriptor: &ModelStructDescriptor, registered: &RegisteredModel) {
        let property = |name: &str| registered.properties.iter().find(|property| property.name == name).unwrap();
        let mut written = Vec::new();
        let mut creations = BTreeMap::new();
        let ids =
            [registered.id].into_iter().chain(registered.properties.iter().flat_map(|property| [property.id, property.membership_id]));
        for id in ids {
            let event = created(registrar, id).await;
            creations.insert(id, (*event.nonce(), event.timestamp()));
            written.push(event);
        }
        let geneses = definitions(descriptor, |definition, _| match definition {
            Definition::Model => creations[&registered.id],
            Definition::Property(name) => creations[&property(name).id],
            Definition::Link(name) => creations[&property(name).membership_id],
        })
        .unwrap();
        assert_eq!(under(registrar.system.root_id(), &geneses), written, "{}", descriptor.label);
        let text = akev::write(&geneses).unwrap();
        let read = akev::read(&[(Path::new("example.akev"), &text)]).unwrap().remove(0);
        assert_eq!(read, geneses.iter().map(|genesis| genesis.event().unwrap()).collect::<Vec<_>>(), "{}", descriptor.label);
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
    async fn register(requester: &TestNode, registrar: &TestNode, descriptor: &ModelStructDescriptor) -> RegisteredModel {
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
    /// `definitions` makes of its declaration, but for the root the node names. Between them
    /// the models declare every value type and both backends a definition holds, so that a
    /// change in how Ankurah encodes any of them fails here.
    #[cfg(feature = "full")]
    #[tokio::test]
    async fn every_model_is_made_as_a_node_registers_it_but_for_the_root() {
        use std::collections::BTreeSet;
        let (registrar, requester, _connection) = registrar().await;
        let mut declared = BTreeSet::new();
        for model in crate::namespaces::NAMESPACES.iter().flat_map(|namespace| namespace.models) {
            let descriptor = model();
            made_as_registered(&registrar, descriptor, &register(&requester, &registrar, descriptor).await).await;
            declared.extend(descriptor.properties.iter().map(|property| (property.value_type, property.backend)));
        }
        let value_types: BTreeSet<&str> = declared.iter().map(|(value_type, _)| *value_type).collect();
        let backends: BTreeSet<&str> = declared.iter().map(|(_, backend)| *backend).collect();
        assert_eq!(value_types, BTreeSet::from(["bool", "entityid", "f64", "i64", "json", "string"]));
        assert_eq!(backends, BTreeSet::from(["lww", "yrs"]));
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
    /// registration creates the property under the name it has now, as `definitions` does.
    #[tokio::test]
    async fn a_property_renamed_from_a_former_name_is_made_as_a_node_registers_it() {
        let (registrar, requester, _connection) = registrar().await;
        let descriptor = Renamed::descriptor();
        made_as_registered(&registrar, descriptor, &register(&requester, &registrar, descriptor).await).await;
    }
}
