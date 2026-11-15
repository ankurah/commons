//! Property types the namespaces share.

use ankurah::{property::PropertyError, value::Value, EntityId, Property};
use serde::{Deserialize, Serialize};

/// A reference to a record of any model, for a field whose target is not one
/// model, such as either end of an edge. A field whose target is one model is
/// a `Ref` to that model instead.
// TODO: Ankurah's own reference to a record of any model, once it has one.
pub type DynRef = EntityId;

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
    use super::Timestamp;
    use ankurah::{value::Value, Property};

    // A field that held Unix milliseconds as an i64 reads its old values as timestamps.
    #[test]
    fn a_timestamp_is_stored_as_its_unix_milliseconds() {
        let timestamp = Timestamp::from_unix_millis(1_700_000_000_000);
        assert_eq!(timestamp.into_value().unwrap(), Some(Value::I64(1_700_000_000_000)));
        assert_eq!(Timestamp::from_value(Some(Value::I64(1_700_000_000_000))).unwrap(), timestamp);
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
