//! The tool's own file encoding of a definition's state, kept so that a fresh store can be
//! seeded with it. Ankurah publishes an encoding for an event (`Attested<Event>::encode`
//! and `decode`) but none for a state, so this encodes an `EntityState` the way Ankurah's
//! private `proto/src/encoding.rs` encodes its values: bincode 1.3.3 with fixed-width
//! integers and no size limit, refusing bytes left over. It depends on the layout of
//! Ankurah's state types (`EntityState`, `State`, `StateBuffers`, `Clock`, and the
//! last-write-wins state buffer) at the pinned revision; a later revision may make kept
//! state files unreadable.

use ankurah::proto::EntityState;
use bincode::Options;

fn options() -> impl Options {
    bincode::DefaultOptions::new().with_fixint_encoding()
}

/// The bytes of `state`'s file.
pub fn encode(state: &EntityState) -> Vec<u8> {
    // Cannot fail: no size limit is set, writing to a vector cannot fail, and every
    // sequence and map in a state has a known length.
    options().serialize(state).expect("a state encodes")
}

/// The state a file's `bytes` hold; bytes left over after it make them undecodable.
pub fn decode(bytes: &[u8]) -> bincode::Result<EntityState> {
    options().deserialize(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ankurah::proto::{Clock, EntityId, EventId, ModelId, State, StateBuffers, SystemModel};
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn a_state_decodes_from_its_bytes_and_from_nothing_more() {
        let state = EntityState {
            entity_id: EntityId::from_bytes([1; 32]),
            state: State {
                state_buffers: StateBuffers(BTreeMap::from([("lww".to_owned(), vec![1, 2, 3])])),
                memberships: BTreeSet::from([ModelId::System(SystemModel::Model)]),
                head: Clock::genesis(EventId::from_bytes([1; 32])),
            },
        };
        let mut bytes = encode(&state);
        assert_eq!(decode(&bytes).unwrap(), state);
        bytes.push(0);
        assert!(decode(&bytes).is_err(), "a byte left over");
    }
}
