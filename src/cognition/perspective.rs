use crate::property::SnapshotRef;
use ankurah::Model;
use serde::{Deserialize, Serialize};

/// One party's account, in its own words, of any record: an intention, a
/// claim, a decision, a task or another perspective.
///
/// Who wrote it down, and when, is `Created`; whose perspective it is, when
/// someone else wrote it down, is `Attribution`; its last rewording is
/// `Edited`. All are on the same entity.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "cognition.perspective")]
#[model(no_ffi)]
pub struct Perspective {
    /// The record it is of, at the head it read.
    pub of: SnapshotRef,
    #[active_type(LWW)]
    pub words: String,
}

#[cfg(test)]
mod tests {
    use super::{Perspective, PerspectiveView};
    use crate::property::SnapshotRef;
    use ankurah::{
        policy::DEFAULT_CONTEXT,
        proto::{Clock, EventId},
        EntityId, Node, PermissiveAgent,
    };
    use ankurah_storage_sled::SledStorageEngine;
    use std::sync::Arc;

    #[tokio::test]
    async fn a_fetch_by_the_entity_a_perspective_is_of_finds_it() {
        let node = Node::new_durable(Arc::new(SledStorageEngine::new_test().unwrap()), PermissiveAgent::new());
        node.system.create().await.unwrap();
        let context = node.context_async(DEFAULT_CONTEXT).await.unwrap();
        let [intention, claim] = [1, 2].map(|byte| EntityId::from_bytes([byte; 32]));
        let head = Clock::new(vec![(1, EventId::from_bytes([3; 32]))]).unwrap();
        let transaction = context.begin();
        for (record, words) in [(intention, "of the intention"), (claim, "of the claim")] {
            let perspective = Perspective { of: SnapshotRef { entity: record, head: head.clone() }, words: words.to_owned() };
            transaction.create(&perspective).await.unwrap();
        }
        transaction.commit().await.unwrap();

        let found: Vec<PerspectiveView> = context.fetch(format!("of.entity = '{}'", claim.to_base64())).await.unwrap();
        let words: Vec<String> = found.iter().map(|perspective| perspective.words().unwrap()).collect();
        assert_eq!(words, ["of the claim"]);
    }
}
