//! Property types the namespaces share.

use std::collections::BTreeMap;

use ankurah::{property::PropertyError, proto::Clock, value::Value, EntityId, Property};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// A reference to a record of any model, for a field whose target is not one
/// model, such as either end of an edge. A field whose target is one model is
/// a `Ref` to that model instead.
// TODO: Ankurah's own reference to a record of any model, once it has one.
pub type DynRef = EntityId;

/// A reference to a record as it was at a given head, so that what refers to
/// it says which version it read. Stored as JSON, `{"entity": <the record's
/// id in base64>, "head": <the head, as Ankurah serializes a clock>}`, so that
/// a query matches the record by path, as in `of.entity = '<id>'`.
///
/// Ankurah cannot yet read an entity at a past head; the head is recorded now
/// and the read comes later.
// TODO(x_hrOz3N): move into Ankurah core, with a read of an entity's state at a head.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotRef {
    pub entity: DynRef,
    pub head: Clock,
}

/// References to records, each under a key that says what that record is to
/// the one holding the map. The keys are minted entities named by meaning,
/// such as kinds of edge. Stored as a JSON object from key to id, both in
/// base64.
// TODO(tzgOiOS4): move into Ankurah core, with keys that are registered properties.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RefMap(pub BTreeMap<EntityId, DynRef>);

impl Property for SnapshotRef {
    const VALUE_TYPE: &'static str = "json";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        into_json(self)
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        from_json(value, "SnapshotRef")
    }
}

impl Property for RefMap {
    const VALUE_TYPE: &'static str = "json";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        into_json(self)
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        from_json(value, "RefMap")
    }
}

fn into_json<T: Serialize>(property: &T) -> Result<Option<Value>, PropertyError> {
    Value::json(property).map(Some).map_err(|error| PropertyError::SerializeError(Box::new(error)))
}

/// Reads only a JSON value, so that a value stored in another form fails rather
/// than being parsed as JSON text.
fn from_json<T: DeserializeOwned>(value: Option<Value>, ty: &str) -> Result<T, PropertyError> {
    match value {
        Some(json @ Value::Json(_)) => json.parse_as_json().map_err(|error| PropertyError::DeserializeError(Box::new(error))),
        Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: ty.to_owned() }),
        None => Err(PropertyError::Missing),
    }
}

/// A moment, as milliseconds since the Unix epoch, in no time zone. Stored as
/// an `i64`, so that a query compares two timestamps as numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Timestamp {
    pub fn from_unix_millis(millis: i64) -> Self {
        Timestamp(millis)
    }

    pub fn unix_millis(self) -> i64 {
        self.0
    }
}

impl Property for Timestamp {
    const VALUE_TYPE: &'static str = "i64";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::I64(self.0)))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::I64(millis)) => Ok(Timestamp(millis)),
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "Timestamp".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

/// Fails outside the years jiff represents, -9999 to 9999.
#[cfg(feature = "jiff")]
impl TryFrom<Timestamp> for jiff::Timestamp {
    type Error = jiff::Error;

    fn try_from(timestamp: Timestamp) -> Result<Self, Self::Error> {
        jiff::Timestamp::from_millisecond(timestamp.0)
    }
}

/// Fails on a jiff timestamp with a part below a millisecond, which a
/// `Timestamp` cannot hold, rather than dropping that part. Round it to the
/// millisecond first to convert it.
#[cfg(feature = "jiff")]
impl TryFrom<jiff::Timestamp> for Timestamp {
    type Error = SubMillisecondError;

    fn try_from(timestamp: jiff::Timestamp) -> Result<Self, Self::Error> {
        match timestamp.subsec_nanosecond() % 1_000_000 {
            0 => Ok(Timestamp(timestamp.as_millisecond())),
            _ => Err(SubMillisecondError(timestamp)),
        }
    }
}

/// A jiff timestamp with a part below a millisecond, which a `Timestamp`
/// cannot hold.
#[cfg(feature = "jiff")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubMillisecondError(pub jiff::Timestamp);

#[cfg(feature = "jiff")]
impl std::fmt::Display for SubMillisecondError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} has a part below a millisecond, which a Timestamp cannot hold", self.0)
    }
}

#[cfg(feature = "jiff")]
impl std::error::Error for SubMillisecondError {}

#[cfg(test)]
mod tests {
    use super::{RefMap, SnapshotRef, Timestamp};
    use ankurah::{
        proto::{Clock, EventId},
        value::Value,
        EntityId, Property,
    };
    use serde_json::json;

    // A field that held Unix milliseconds as an i64 reads its old values as timestamps.
    #[test]
    fn a_timestamp_is_stored_as_its_unix_milliseconds() {
        let timestamp = Timestamp::from_unix_millis(1_700_000_000_000);
        assert_eq!(timestamp.into_value().unwrap(), Some(Value::I64(1_700_000_000_000)));
        assert_eq!(Timestamp::from_value(Some(Value::I64(1_700_000_000_000))).unwrap(), timestamp);
    }

    #[test]
    fn a_snapshot_ref_is_stored_as_json_naming_its_entity_and_head() {
        let entity = EntityId::from_bytes([7; 32]);
        let tip = EventId::from_bytes([9; 32]);
        let snapshot = SnapshotRef { entity, head: Clock::new(vec![(3, tip.clone())]).unwrap() };
        let stored = snapshot.into_value().unwrap();
        assert_eq!(stored, Some(Value::Json(json!({"entity": entity.to_base64(), "head": [[3, tip.to_base64()]]}))));
        assert_eq!(SnapshotRef::from_value(stored).unwrap(), snapshot);
    }

    #[test]
    fn a_ref_map_is_stored_as_a_json_object_from_key_to_id() {
        let [discovered_from, generated_by, message, import_run] = [1, 2, 3, 4].map(|byte| EntityId::from_bytes([byte; 32]));
        let refs = RefMap([(discovered_from, message), (generated_by, import_run)].into());
        let stored = refs.into_value().unwrap();
        let object = json!({discovered_from.to_base64(): message.to_base64(), generated_by.to_base64(): import_run.to_base64()});
        assert_eq!(stored, Some(Value::Json(object)));
        assert_eq!(RefMap::from_value(stored).unwrap(), refs);
    }

    // Before 1970 too, where jiff's part below a second is negative.
    #[cfg(feature = "jiff")]
    #[test]
    fn a_timestamp_converts_to_jiff_and_back() {
        let timestamp = Timestamp::from_unix_millis(-1_700_000_000_123);
        let converted = jiff::Timestamp::try_from(timestamp).unwrap();
        assert_eq!(converted.as_millisecond(), -1_700_000_000_123);
        assert_eq!(Timestamp::try_from(converted), Ok(timestamp));
    }

    #[cfg(feature = "jiff")]
    #[test]
    fn a_conversion_that_would_lose_part_of_the_moment_fails() {
        let precise = jiff::Timestamp::new(1_700_000_000, 123_456_789).unwrap();
        assert_eq!(Timestamp::try_from(precise), Err(super::SubMillisecondError(precise)));
        assert!(jiff::Timestamp::try_from(Timestamp::from_unix_millis(i64::MAX)).is_err());
    }
}
