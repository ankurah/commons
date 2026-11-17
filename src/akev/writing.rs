//! Writes events as a file, in one layout of the many a reader takes.

use std::{collections::HashMap, error::Error, fmt};

use ankurah::{
    core::property::backend::{LWWBackend, PropertyBackend},
    proto::{BackendOperation, Event, ModelId},
    value::Value,
    EntityId, PropertyId,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use super::{
    genesis::{BackendOperations, BuildError, Genesis},
    naming::{is_name, label_and_name, Names},
    timestamp,
};

/// The widest line that holds a block's values and memberships together.
const WIDTH: usize = 100;

/// Why events could not be written.
#[derive(Debug)]
pub enum WriteError {
    Build(BuildError),
    /// A timestamp after `timestamp::LAST`, which the one form of a timestamp does not write.
    Timestamp(u64),
    /// A backend's operations that are not one operation of at least one byte, which is
    /// what a block writes as one token.
    NotOneToken {
        backend: String,
    },
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::Build(source) => write!(f, "the operations do not build: {source}"),
            WriteError::Timestamp(ms) => write!(f, "the timestamp {ms} falls after the year 9999"),
            WriteError::NotOneToken { backend } => write!(f, "the `{backend}` backend's operations are not one operation of some bytes"),
        }
    }
}

impl Error for WriteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            WriteError::Build(source) => Some(source),
            WriteError::Timestamp(_) | WriteError::NotOneToken { .. } => None,
        }
    }
}

/// The file that states `geneses`, in their order. Each block writes the backends' values
/// as text where every value has a literal, and the operation's bytes otherwise; and it
/// writes `$name` for an id of an entity in `geneses` that answers to that one name.
pub fn write(geneses: &[Genesis]) -> Result<String, WriteError> {
    let events = geneses.iter().map(Genesis::event).collect::<Result<Vec<_>, _>>().map_err(WriteError::Build)?;
    let places: HashMap<EntityId, usize> = events.iter().enumerate().map(|(place, event)| (event.entity_id, place)).collect();
    let lww = <LWWBackend as PropertyBackend>::property_backend_name();
    let mut names = Names::default();
    let mut named = Vec::with_capacity(geneses.len());
    for (place, genesis) in geneses.iter().enumerate() {
        let name = match genesis.operations.backends.get(lww) {
            Some(BackendOperations::Values(values)) if values.iter().all(|(_, value)| has_literal(value)) => {
                let (label, name) = label_and_name(values.iter().map(|(property, value)| (property, value)));
                names.add(place, label, name);
                label.or(name)
            }
            _ => None,
        };
        named.push(name);
    }
    let writer = Writer { places, names, named };
    let mut text = String::new();
    for (genesis, event) in geneses.iter().zip(&events) {
        if !text.is_empty() {
            text.push('\n');
        }
        writer.block(&mut text, genesis, event)?;
    }
    Ok(text)
}

/// What the writer knows of the file's entities.
struct Writer<'g> {
    places: HashMap<EntityId, usize>,
    names: Names<'g>,
    /// The name each entity is written by, if it answers to one: its label, or else its name.
    named: Vec<Option<&'g str>>,
}

/// One field of a block after its nonce and timestamp.
struct Field {
    /// The field on one line.
    line: String,
    /// A backend's values with one entry to a line, when they are text.
    lines: Option<Vec<String>>,
    /// Whether the field holds an id or a name, which reads better alone on its line.
    refers: bool,
}

impl Writer<'_> {
    fn block(&self, text: &mut String, genesis: &Genesis, event: &Event) -> Result<(), WriteError> {
        let timestamp = timestamp::format(genesis.timestamp).ok_or(WriteError::Timestamp(genesis.timestamp))?;
        text.push_str(&format!("[{}]\nnonce: {}, timestamp: {timestamp}\n", event.entity_id, EntityId::from_bytes(genesis.nonce)));
        let mut fields = Vec::new();
        for (backend, operations) in &genesis.operations.backends {
            fields.push(match operations {
                BackendOperations::Values(values) if values.iter().all(|(_, value)| has_literal(value)) => self.values(backend, values),
                _ => {
                    let mut operations = event.operations().backend_operations(backend);
                    let token = match (operations.next(), operations.next()) {
                        (Some(BackendOperation { diff }), None) if !diff.is_empty() => URL_SAFE_NO_PAD.encode(diff),
                        _ => return Err(WriteError::NotOneToken { backend: backend.clone() }),
                    };
                    Field { line: format!("{backend}: {token}"), lines: None, refers: false }
                }
            });
        }
        for model in &genesis.operations.memberships {
            let model = match model {
                ModelId::EntityId(id) => self.reference(*id),
                ModelId::System(_) => model.to_string(),
            };
            fields.push(Field { line: format!("member: {model}"), lines: None, refers: false });
        }
        if fields.is_empty() {
            return Ok(());
        }
        let line = fields.iter().map(|field| field.line.as_str()).collect::<Vec<_>>().join(", ");
        if line.len() <= WIDTH && !fields.iter().any(|field| field.refers) {
            text.push_str(&line);
            text.push('\n');
            return Ok(());
        }
        for field in fields {
            for line in field.lines.unwrap_or_else(|| vec![field.line]) {
                text.push_str(&line);
                text.push('\n');
            }
        }
        Ok(())
    }

    fn values(&self, backend: &str, values: &[(PropertyId, Option<Value>)]) -> Field {
        let entries: Vec<String> = values
            .iter()
            .map(|(property, value)| {
                let property = match property {
                    PropertyId::EntityId(id) => self.reference(*id),
                    _ => property.to_string(),
                };
                let value = match value {
                    None => "none".to_owned(),
                    Some(Value::Bool(value)) => value.to_string(),
                    Some(Value::String(text)) => quoted(text),
                    Some(Value::EntityId(id)) => self.reference(*id),
                    Some(_) => unreachable!("only values with a literal are written as text"),
                };
                format!("{property}: {value}")
            })
            .collect();
        let refers =
            values.iter().any(|(property, value)| matches!(property, PropertyId::EntityId(_)) || matches!(value, Some(Value::EntityId(_))));
        if entries.is_empty() {
            return Field { line: format!("{backend}: {{}}"), lines: None, refers };
        }
        let lines =
            [format!("{backend}: {{")].into_iter().chain(entries.iter().map(|entry| format!("    {entry}"))).chain(["}".to_owned()]);
        Field { line: format!("{backend}: {{ {} }}", entries.join(", ")), lines: Some(lines.collect()), refers }
    }

    /// `$name` when `id` is an entity of the file that answers to the name it is written by,
    /// and the id otherwise.
    fn reference(&self, id: EntityId) -> String {
        let name = self.places.get(&id).and_then(|&place| {
            let name = self.named[place]?;
            (is_name(name) && self.names.resolve(name) == Ok(place)).then_some(name)
        });
        match name {
            Some(name) => format!("${name}"),
            None => id.to_base64(),
        }
    }
}

/// Whether the text form writes `value`.
fn has_literal(value: &Option<Value>) -> bool {
    matches!(value, None | Some(Value::Bool(_) | Value::String(_) | Value::EntityId(_)))
}

fn quoted(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    for character in text.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            _ => quoted.push(character),
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::akev::{read, sample, Operations};
    use ankurah::proto::SystemProperty;
    use std::{collections::BTreeMap, path::Path};

    /// A genesis whose `lww` values are `values`, a member of no model.
    fn genesis(byte: u8, values: Vec<(PropertyId, Option<Value>)>) -> Genesis {
        let backends = BTreeMap::from([("lww".to_owned(), BackendOperations::Values(values))]);
        Genesis { nonce: [byte; 32], timestamp: 0, operations: Operations { backends, memberships: Vec::new() } }
    }

    fn named(text: &str) -> (PropertyId, Option<Value>) {
        (PropertyId::System(SystemProperty::Name), Some(Value::String(text.to_owned())))
    }

    /// The events the text written of `geneses` reads as.
    fn read_back(text: &str) -> Vec<Event> {
        read(&[(Path::new("example.akev"), text)]).unwrap().remove(0)
    }

    fn events(geneses: &[Genesis]) -> Vec<Event> {
        geneses.iter().map(|genesis| genesis.event().unwrap()).collect()
    }

    #[test]
    fn the_sample_is_written_as_the_writer_lays_it_out_with_names_for_the_files_ids() {
        assert_eq!(write(&sample::geneses()).unwrap(), sample::TEXT);
    }

    #[test]
    fn values_with_no_literal_are_written_as_their_bytes_which_read_as_the_same_event() {
        let geneses = [genesis(1, vec![named("count"), (PropertyId::System(SystemProperty::Item), Some(Value::I64(3)))])];
        let text = write(&geneses).unwrap();
        let lww = text.lines().nth(2).unwrap();
        assert!(lww.starts_with("lww: ") && !lww.contains('{'), "{text}");
        assert_eq!(read_back(&text), events(&geneses));
    }

    #[test]
    fn text_with_quotes_backslashes_and_line_breaks_reads_back_as_itself() {
        let geneses = [genesis(1, vec![named("a \"b\" \\ c\nd")])];
        let text = write(&geneses).unwrap();
        assert!(text.contains(r#"lww: { name: "a \"b\" \\ c\nd" }"#), "{text}");
        assert_eq!(read_back(&text), events(&geneses));
    }

    /// Two entities named `title`, one named `a b`, which is no name, and one that refers
    /// to each of them.
    #[test]
    fn an_id_is_written_where_its_entity_answers_to_no_one_name_it_could_be_written_by() {
        let mut geneses = vec![genesis(1, vec![named("title")]), genesis(2, vec![named("title")]), genesis(3, vec![named("a b")])];
        let ids: Vec<EntityId> = events(&geneses).iter().map(|event| event.entity_id).collect();
        let refers = ids.iter().map(|&id| (PropertyId::EntityId(id), Some(Value::Bool(true)))).collect();
        geneses.push(genesis(4, refers));
        let text = write(&geneses).unwrap();
        assert!(!text.contains('$'), "{text}");
        assert!(ids.iter().all(|id| text.contains(&format!("    {id}: true"))), "{text}");
        assert_eq!(read_back(&text), events(&geneses));
    }

    #[test]
    fn values_that_would_not_fit_in_a_line_take_a_line_for_each_entry() {
        let long = "x".repeat(WIDTH);
        let mut geneses = vec![genesis(1, vec![named(&long)])];
        geneses[0].operations.memberships.push(ModelId::System(ankurah::proto::SystemModel::Model));
        let text = write(&geneses).unwrap();
        assert!(text.ends_with(&format!("lww: {{\n    name: \"{long}\"\n}}\nmember: system:model\n")), "{text}");
        assert_eq!(read_back(&text), events(&geneses));
    }

    #[test]
    fn a_timestamp_after_the_year_9999_is_refused() {
        let mut late = genesis(1, Vec::new());
        late.timestamp = timestamp::LAST + 1;
        assert!(matches!(write(&[late]), Err(WriteError::Timestamp(ms)) if ms == timestamp::LAST + 1));
    }
}
