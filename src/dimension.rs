//! A property type for the size of something along one line, such as a width, for
//! applications built on the Commons.

use ankurah::{property::PropertyError, value::Value, Property};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The size of something along one line, such as a width: finite, and not below zero.
/// Stored as its number of millimetres, an `f64`, so that a query compares two dimensions
/// as numbers.
// OPEN: `health::Distance` holds its metres as a bare number. Whether it becomes a
// `Dimension` is to settle.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Dimension(f64);

impl Dimension {
    pub fn from_millimeters(millimeters: f64) -> Result<Self, NotADimension> {
        if millimeters.is_finite() && millimeters >= 0.0 {
            Ok(Dimension(millimeters))
        } else {
            Err(NotADimension { millimeters })
        }
    }

    pub fn millimeters(self) -> f64 {
        self.0
    }
}

/// A number of millimetres that is no dimension: not finite, or below zero.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NotADimension {
    pub millimeters: f64,
}

impl std::fmt::Display for NotADimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} millimetres is no dimension", self.millimeters)
    }
}

impl std::error::Error for NotADimension {}

impl Serialize for Dimension {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for Dimension {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Dimension::from_millimeters(f64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl Property for Dimension {
    const VALUE_TYPE: &'static str = "f64";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::F64(self.0)))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(Value::F64(millimeters)) => Dimension::from_millimeters(millimeters)
                .map_err(|refused| PropertyError::InvalidValue { value: refused.millimeters.to_string(), ty: "Dimension".to_owned() }),
            Some(other) => Err(PropertyError::InvalidVariant { given: other, ty: "Dimension".to_owned() }),
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_dimension_is_stored_as_its_millimetres() {
        let width = Dimension::from_millimeters(101.6).unwrap();
        assert_eq!(width.into_value().unwrap(), Some(Value::F64(101.6)));
        assert_eq!(Dimension::from_value(Some(Value::F64(101.6))).unwrap(), width);
        assert_eq!(width.millimeters(), 101.6);
    }

    #[test]
    fn a_number_that_is_no_dimension_is_refused() {
        for refused in [-1.0, f64::NAN, f64::INFINITY] {
            assert!(Dimension::from_millimeters(refused).is_err(), "{refused}");
            let read = Dimension::from_value(Some(Value::F64(refused)));
            assert!(matches!(read, Err(PropertyError::InvalidValue { .. })), "{refused}");
        }
        assert!(matches!(Dimension::from_value(Some(Value::I64(4))), Err(PropertyError::InvalidVariant { .. })));
        assert!(matches!(Dimension::from_value(None), Err(PropertyError::Missing)));
        assert!(Dimension::from_millimeters(0.0).is_ok(), "nothing is still a size");
    }

    // Serde is a second way in, which must refuse what `from_millimeters` refuses.
    #[test]
    fn a_dimension_is_a_bare_number_in_json_and_is_checked_when_read() {
        let width: Dimension = serde_json::from_value(json!(101.6)).unwrap();
        assert_eq!(width.millimeters(), 101.6);
        assert_eq!(serde_json::to_value(width).unwrap(), json!(101.6));
        assert!(serde_json::from_value::<Dimension>(json!(-1.0)).is_err());
    }
}
