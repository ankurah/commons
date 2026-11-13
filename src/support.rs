//! What the namespaces share that is not a model.

/// Stores a unit enum as its name in snake case (`in_progress`), so that a
/// query reads `status = 'in_progress'`. A stored name this build's enum does
/// not have fails to read, with `InvalidValue`.
///
/// The enum derives `strum::Display` and `strum::EnumString` with
/// `#[strum(serialize_all = "snake_case")]`.
// OPEN: this belongs in Ankurah. `#[derive(Property)]` stores an enum as its
// JSON encoding, quote characters included, so a query has to compare against
// `'"open"'`.
macro_rules! string_enum {
    ($t:ty) => {
        impl ::ankurah::Property for $t {
            const VALUE_TYPE: &'static str = "string";

            fn into_value(&self) -> Result<Option<::ankurah::value::Value>, ::ankurah::property::PropertyError> {
                Ok(Some(::ankurah::value::Value::String(self.to_string())))
            }

            fn from_value(value: Option<::ankurah::value::Value>) -> Result<Self, ::ankurah::property::PropertyError> {
                match value {
                    Some(value) => value.parse_as_string().map_err(|_| ::ankurah::property::PropertyError::InvalidValue {
                        value: value.to_string(),
                        ty: stringify!($t).to_owned(),
                    }),
                    None => Err(::ankurah::property::PropertyError::Missing),
                }
            }
        }
    };
}

pub(crate) use string_enum;
