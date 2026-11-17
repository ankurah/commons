//! A creation event as a file states it, and the event a node builds of it.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use ankurah::{
    core::property::backend::{LWWBackend, PropertyBackend, YrsBackend},
    error::MutationError,
    proto::{BackendOperation, Event, Membership, ModelId, Operation, OperationSet},
    value::Value,
    PropertyId,
};

/// A creation event as an `.akev` block states it, with every name resolved to its id.
#[derive(Debug, Clone, PartialEq)]
pub struct Genesis {
    pub nonce: [u8; 32],
    /// Unix milliseconds.
    pub timestamp: u64,
    pub operations: Operations,
}

/// An event's operations as a block states them: each property backend's, by the backend's
/// name, and the models the event adds its entity to, in order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Operations {
    pub backends: BTreeMap<String, BackendOperations>,
    pub memberships: Vec<ModelId>,
}

/// One property backend's operations for an event.
#[derive(Debug, Clone, PartialEq)]
pub enum BackendOperations {
    /// Values to set on a fresh backend, whose operations are then taken: the backend's text
    /// form. Only `lww` has one.
    Values(Vec<(PropertyId, Option<Value>)>),
    /// The backend's operation bytes, as they are.
    Bytes(BackendOperation),
}

/// Why a block's operations do not build.
#[derive(Debug)]
pub enum BuildError {
    /// Not the name of a backend a node registers.
    UnknownBackend(String),
    /// Values for a backend that has no text form.
    NoTextForm(String),
    /// The `id` pseudo-property, which a backend stores nothing under.
    IdProperty {
        backend: String,
    },
    RepeatedProperty {
        backend: String,
        property: PropertyId,
    },
    Encoding {
        backend: String,
        source: Box<MutationError>,
    },
}

impl BuildError {
    /// The backend whose operations do not build.
    pub(super) fn backend(&self) -> &str {
        match self {
            BuildError::UnknownBackend(backend)
            | BuildError::NoTextForm(backend)
            | BuildError::IdProperty { backend }
            | BuildError::RepeatedProperty { backend, .. }
            | BuildError::Encoding { backend, .. } => backend,
        }
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::UnknownBackend(backend) => write!(f, "`{backend}` is no property backend a node registers"),
            BuildError::NoTextForm(backend) => write!(f, "the `{backend}` backend has no text form: write its operations' bytes"),
            BuildError::IdProperty { backend } => write!(f, "the `{backend}` backend's values set `id`, which no backend stores"),
            BuildError::RepeatedProperty { backend, property } => write!(f, "the `{backend}` backend's values set `{property}` twice"),
            BuildError::Encoding { backend, source } => write!(f, "the `{backend}` backend's values do not encode: {source}"),
        }
    }
}

// Written out rather than derived, so that the source of a boxed error is the error, not its
// box, and a caller can downcast it.
impl Error for BuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BuildError::Encoding { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl Genesis {
    /// The genesis these parts make, naming no system root and no author, so that it is the
    /// same entity, with the same id, on every node.
    pub fn event(&self) -> Result<Event, BuildError> {
        Ok(Event::genesis_from_parts(self.nonce, self.timestamp, self.operations.build()?))
    }
}

impl Operations {
    /// The operations a node builds for a creation (`take_operations` in Ankurah's local
    /// transaction): each backend's, in the order of the backends' names, then the
    /// memberships in order. A backend whose values are none builds no operation.
    pub fn build(&self) -> Result<OperationSet, BuildError> {
        let mut backends = BTreeMap::new();
        for (backend, operations) in &self.backends {
            let built = operations.build(backend)?;
            if !built.is_empty() {
                backends.insert(backend.clone(), built);
            }
        }
        let mut operations = OperationSet::from_backends(backends);
        for &model in &self.memberships {
            operations.push(Operation::Membership(Membership::Add(model)));
        }
        Ok(operations)
    }
}

impl BackendOperations {
    fn build(&self, backend: &str) -> Result<Vec<BackendOperation>, BuildError> {
        let lww = <LWWBackend as PropertyBackend>::property_backend_name();
        if backend != lww && backend != <YrsBackend as PropertyBackend>::property_backend_name() {
            return Err(BuildError::UnknownBackend(backend.to_owned()));
        }
        let values = match self {
            BackendOperations::Bytes(operation) => return Ok(vec![operation.clone()]),
            BackendOperations::Values(_) if backend != lww => return Err(BuildError::NoTextForm(backend.to_owned())),
            BackendOperations::Values(values) => values,
        };
        let fresh = LWWBackend::new();
        let mut set = BTreeSet::new();
        for (property, value) in values {
            // The backend would skip the one and keep the last of the other, silently.
            if *property == PropertyId::Id {
                return Err(BuildError::IdProperty { backend: backend.to_owned() });
            }
            if !set.insert(*property) {
                return Err(BuildError::RepeatedProperty { backend: backend.to_owned(), property: *property });
            }
            fresh.set(*property, value.clone());
        }
        let encoding = |source| BuildError::Encoding { backend: backend.to_owned(), source: Box::new(source) };
        Ok(fresh.to_operations().map_err(encoding)?.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ankurah::proto::{SystemModel, SystemProperty};

    fn values(values: Vec<(PropertyId, Option<Value>)>) -> Operations {
        Operations { backends: BTreeMap::from([("lww".to_owned(), BackendOperations::Values(values))]), memberships: Vec::new() }
    }

    fn label(text: &str) -> (PropertyId, Option<Value>) {
        (PropertyId::System(SystemProperty::Label), Some(Value::String(text.to_owned())))
    }

    /// The values build the diff a fresh backend gives them, the bytes go as they are, in the
    /// order of the backends' names, and the memberships follow in the order given.
    #[test]
    fn operations_build_as_a_node_builds_a_creations() {
        let backend = LWWBackend::new();
        backend.set(PropertyId::System(SystemProperty::Label), Some(Value::String("example.thing".to_owned())));
        let diff = backend.to_operations().unwrap().unwrap();
        let bytes = BackendOperation { diff: vec![1, 2, 3] };
        let models = vec![ModelId::System(SystemModel::Property), ModelId::System(SystemModel::Model)];
        let mut operations = values(vec![label("example.thing")]);
        operations.backends.insert("yrs".to_owned(), BackendOperations::Bytes(bytes.clone()));
        operations.memberships = models.clone();
        let mut expected = OperationSet::from_backends(BTreeMap::from([("lww".to_owned(), diff), ("yrs".to_owned(), vec![bytes])]));
        for model in models {
            expected.push(Operation::Membership(Membership::Add(model)));
        }
        assert_eq!(operations.build().unwrap(), expected);
    }

    #[test]
    fn no_values_build_no_backend_operation() {
        let operations = values(Vec::new());
        assert_eq!(operations.build().unwrap(), OperationSet::default());
    }

    #[test]
    fn values_the_backend_would_drop_or_overwrite_are_refused() {
        let id = values(vec![(PropertyId::Id, None)]).build().unwrap_err();
        assert!(matches!(id, BuildError::IdProperty { .. }), "{id}");
        let twice = values(vec![label("a"), label("b")]).build().unwrap_err();
        assert!(matches!(twice, BuildError::RepeatedProperty { property: PropertyId::System(SystemProperty::Label), .. }), "{twice}");
    }

    #[test]
    fn values_for_yrs_or_any_operations_for_an_unknown_backend_are_refused() {
        let mut operations = values(vec![label("a")]);
        let yrs = operations.backends.remove("lww").unwrap();
        operations.backends.insert("yrs".to_owned(), yrs);
        assert!(matches!(operations.build(), Err(BuildError::NoTextForm(backend)) if backend == "yrs"));
        let bytes = BackendOperations::Bytes(BackendOperation { diff: vec![1] });
        let unknown = Operations { backends: BTreeMap::from([("crdt".to_owned(), bytes)]), memberships: Vec::new() };
        assert!(matches!(unknown.build(), Err(BuildError::UnknownBackend(backend)) if backend == "crdt"));
    }

    #[test]
    fn an_encoding_error_is_the_source_of_the_build_error() {
        let error = BuildError::Encoding { backend: "lww".to_owned(), source: Box::new(MutationError::NotDurable) };
        assert!(matches!(error.source().and_then(|source| source.downcast_ref::<MutationError>()), Some(MutationError::NotDurable)));
    }
}
