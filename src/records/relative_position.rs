use crate::property::DynRef;
use ankurah::{property::PropertyError, value::Value, EntityId, Model, Property};
use serde::{Deserialize, Serialize};

/// Where a record stands among its siblings, such as a step in a task's
/// checklist, relative to another of them.
///
/// Records with the same position, such as two placed after one sibling,
/// stand in the order of their entity ids.
// OPEN: where a record stands when its sibling is missing, and how a cycle of
// placements, which concurrent moves can make, is broken.
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[model(label = "records.relative_position")]
#[model(no_ffi)]
pub struct RelativePosition {
    #[active_type(LWW)]
    pub position: Position,
}

/// A place among siblings. Stored as text: `first`, `last`, or `after:` or
/// `before:` and the sibling's id in base64.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Position {
    First,
    After(DynRef),
    Before(DynRef),
    Last,
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Position::First => write!(f, "first"),
            Position::After(id) => write!(f, "after:{}", id.to_base64()),
            Position::Before(id) => write!(f, "before:{}", id.to_base64()),
            Position::Last => write!(f, "last"),
        }
    }
}

impl std::str::FromStr for Position {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sibling = |id: &str| EntityId::from_base64(id).map_err(|e| format!("not a position: {s}: {e}"));
        match s.split_once(':') {
            None if s == "first" => Ok(Position::First),
            None if s == "last" => Ok(Position::Last),
            Some(("after", id)) => Ok(Position::After(sibling(id)?)),
            Some(("before", id)) => Ok(Position::Before(sibling(id)?)),
            _ => Err(format!("not a position: {s}")),
        }
    }
}

impl Property for Position {
    const VALUE_TYPE: &'static str = "string";

    fn into_value(&self) -> Result<Option<Value>, PropertyError> {
        Ok(Some(Value::String(self.to_string())))
    }

    fn from_value(value: Option<Value>) -> Result<Self, PropertyError> {
        match value {
            Some(value) => {
                value.parse_as_string().map_err(|_| PropertyError::InvalidValue { value: value.to_string(), ty: "Position".to_owned() })
            }
            None => Err(PropertyError::Missing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Position;
    use ankurah::{value::Value, Property};

    #[test]
    fn a_position_is_stored_as_its_text() {
        let sibling = ankurah::EntityId::from_bytes([7; 32]);
        let stored = Position::After(sibling).into_value().unwrap();
        assert_eq!(stored, Some(Value::String(format!("after:{}", sibling.to_base64()))));
        assert_eq!(Position::from_value(stored).unwrap(), Position::After(sibling));
        assert!(Position::from_value(Some(Value::String("middle".to_owned()))).is_err());
        assert!(Position::from_value(None).is_err());
    }

    #[test]
    fn a_position_reads_back_as_written() {
        let sibling = ankurah::EntityId::from_bytes([7; 32]);
        for position in [Position::First, Position::After(sibling), Position::Before(sibling), Position::Last] {
            assert_eq!(position.to_string().parse::<Position>(), Ok(position));
        }
        assert!("after:".parse::<Position>().is_err());
        assert!("middle".parse::<Position>().is_err());
    }
}
