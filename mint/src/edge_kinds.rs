//! The records of the kinds of edge the Commons mints, as a node creates an `EdgeKind`:
//! its name and description under the ids of the model's property definitions, and the
//! entity a member of the model.

use std::collections::BTreeMap;

use ankurah::{
    core::property::backend::{LWWBackend, PropertyBackend},
    proto::ModelId,
    value::Value,
    EntityId, PropertyId,
};
use ankurah_commons::akev::{BackendOperations, Operations};

/// The definitions of the model whose records the kinds are.
pub struct EdgeKindModel {
    pub model: EntityId,
    /// The definitions of its properties `name` and `description`.
    pub name: EntityId,
    pub description: EntityId,
}

/// The operations of the record of the kind `name` that `description` describes.
pub fn record(model: &EdgeKindModel, name: &str, description: &str) -> Operations {
    let values = vec![
        (PropertyId::EntityId(model.name), Some(Value::String(name.to_owned()))),
        (PropertyId::EntityId(model.description), Some(Value::String(description.to_owned()))),
    ];
    let lww = <LWWBackend as PropertyBackend>::property_backend_name().to_owned();
    Operations { backends: BTreeMap::from([(lww, BackendOperations::Values(values))]), memberships: vec![ModelId::EntityId(model.model)] }
}

#[cfg(test)]
mod tests {
    use crate::{copies::sources, minting};
    use ankurah::{policy::DEFAULT_CONTEXT, storage::StorageEngine, Node, PermissiveAgent};
    use ankurah_commons::{
        graph::{EdgeKind, EDGE_KINDS},
        namespaces::NAMESPACES,
    };
    use ankurah_storage_sled::SledStorageEngine;
    use std::sync::Arc;

    /// The records of the kinds of edge that a node seeded with the kept mint creates hold
    /// the operations of the records the mint makes of the table, in its order.
    #[tokio::test]
    async fn the_kinds_of_edge_are_recorded_as_a_seeded_node_creates_them() {
        let node = Node::new_durable(Arc::new(SledStorageEngine::new_test().unwrap()), PermissiveAgent::new());
        node.system.create().await.unwrap();
        ankurah_commons::seed(&node).await.unwrap();
        let transaction = node.context_async(DEFAULT_CONTEXT).await.unwrap().begin();
        let mut created = Vec::new();
        for &(name, description) in EDGE_KINDS {
            let kind = EdgeKind { name: name.to_owned(), description: description.to_owned() };
            created.push(transaction.create(&kind).await.unwrap().id());
        }
        transaction.commit().await.unwrap();

        let minted = minting::mint(&sources(), NAMESPACES).unwrap();
        let file = minted.iter().find(|file| file.descriptor.label == "graph.edge_kind").unwrap();
        let records = &file.geneses[file.geneses.len() - EDGE_KINDS.len()..];
        for ((&(name, _), id), record) in EDGE_KINDS.iter().zip(created).zip(records) {
            let [written] = <[_; 1]>::try_from(node.storage.dump_entity_events(id).await.unwrap()).unwrap();
            assert_eq!(written.payload.operations(), &record.operations.build().unwrap(), "{name}");
        }
    }
}
