//! `.akev` files: minted creation events, written as text, so that a reader sees what was
//! minted and a node rebuilds the events' bytes from what it reads (`crate::rebuilding`).
//!
//! A file states one model's definitions, beside the model's struct, or the records of
//! the kinds of edge the Commons mints. Each entity takes one line: what it is first, then
//! its id and what its creation event holds that its values do not, the time it was minted
//! and its nonce. An edge kind's description takes the indented line beneath its kind, so
//! that free text needs no escaping.
//!
//! An entity's id is the hash of its creation event, so the ids follow from the rest of the
//! text: a file states them for its readers, and they check the events rebuilt from it.
//!
//! A file parses only in exactly the form the writer gives it, so that two readers never
//! disagree about one.

use std::fmt;

use ankurah::EntityId;

use crate::rebuilding::RebuildError;

/// What a creation event holds besides the values on its entity's line: the time it was
/// minted and its nonce, both chosen when it was minted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Creation {
    /// Unix milliseconds, as the clock read when the entity was minted.
    pub at: u64,
    pub nonce: [u8; 32],
}

/// A model's minted definitions: the model, then each of its properties with the
/// model-property link that gives the model that property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDefinitions {
    pub label: String,
    /// The name of the model's struct.
    pub name: String,
    pub model: Creation,
    pub properties: Vec<PropertyDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyDefinition {
    pub name: String,
    pub value_type: String,
    pub backend: String,
    pub optional: bool,
    pub property: Creation,
    pub link: Creation,
}

/// The records of the kinds of edge the Commons mints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeKinds {
    pub kinds: Vec<EdgeKindRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeKindRecord {
    pub name: String,
    pub description: String,
    pub record: Creation,
}

/// Why a file does not parse: the line, counted from one, and what is wrong there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub problem: Problem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Problem {
    /// Not the line this place in the file takes, or a missing one.
    Unexpected,
    /// A field that does not read as what it holds.
    Field(&'static str),
    /// A link that names a property other than the one above it.
    LinkToAnother,
    /// Every field reads, but the text is not in the form the writer gives it.
    NotCanonical,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let problem = match self.problem {
            Problem::Unexpected => "not a line this place in the file takes".to_owned(),
            Problem::Field(field) => format!("its {field} does not read"),
            Problem::LinkToAnother => "a link that names a property other than the one above it".to_owned(),
            Problem::NotCanonical => "not in the form the writer gives it".to_owned(),
        };
        write!(f, "line {}: {problem}", self.line)
    }
}

impl std::error::Error for ParseError {}

impl ModelDefinitions {
    /// The file that states these definitions, each line with the id of the creation event
    /// rebuilt for its entity.
    pub fn write(&self) -> Result<String, RebuildError> {
        let ids: Vec<EntityId> = self.events()?.iter().map(|event| event.entity_id).collect();
        Ok(self.text(&ids))
    }

    /// Reads a file in exactly the form the writer gives it: the definitions it states, and
    /// the id each line states, in the file's order. The ids are read, not checked:
    /// `events_as_stated` checks them.
    pub fn parse(text: &str) -> Result<(Self, Vec<EntityId>), ParseError> {
        let mut reader = Reader::new(text);
        let mut ids = Vec::new();
        let (label, name, model) = match reader.tokens().as_deref() {
            Some(["model", label, name, tail @ ..]) => (label.to_string(), name.to_string(), reader.creation(tail, &mut ids)?),
            _ => return Err(reader.unexpected()),
        };
        let mut properties = Vec::new();
        while let Some(tokens) = reader.tokens() {
            let ["property", name, value_type, backend, tail @ ..] = tokens.as_slice() else { return Err(reader.unexpected()) };
            let (name, value_type, backend, property) =
                (name.to_string(), value_type.to_string(), backend.to_string(), reader.creation(tail, &mut ids)?);
            let (optional, link) = match reader.tokens().as_deref() {
                Some(["link", linked, optional, tail @ ..]) if *linked == name => {
                    (reader.optional(optional)?, reader.creation(tail, &mut ids)?)
                }
                Some(["link", ..]) => return Err(reader.error(Problem::LinkToAnother)),
                _ => return Err(reader.unexpected()),
            };
            properties.push(PropertyDefinition { name, value_type, backend, optional, property, link });
        }
        let definitions = ModelDefinitions { label, name, model, properties };
        canonical(text, &definitions.text(&ids))?;
        Ok((definitions, ids))
    }

    /// The writer's form, given an id for each entity in the file's order.
    fn text(&self, ids: &[EntityId]) -> String {
        let names = self.properties.iter().map(|property| property.name.len()).max().unwrap_or(0);
        let types = self.properties.iter().map(|property| property.value_type.len()).max().unwrap_or(0);
        let mut ids = ids.iter();
        let mut lines = vec![Line::new("model", format!("{}  {}", self.label, self.name), ids.next(), &self.model)];
        for property in &self.properties {
            let typed = format!("{:<names$}  {:<types$}  {}", property.name, property.value_type, property.backend);
            lines.push(Line::new("property", typed, ids.next(), &property.property));
            let link = format!("{:<names$}  {}", property.name, if property.optional { "optional" } else { "required" });
            lines.push(Line::new("link", link, ids.next(), &property.link));
        }
        write_lines(&lines)
    }
}

impl EdgeKinds {
    /// The file that states these records of the model `edge_kind` defines, each line with
    /// the id of the creation event rebuilt for its record.
    pub fn write(&self, edge_kind: &ModelDefinitions) -> Result<String, RebuildError> {
        let ids: Vec<EntityId> = self.events(edge_kind)?.iter().map(|event| event.entity_id).collect();
        Ok(self.text(&ids))
    }

    /// Reads a file in exactly the form the writer gives it: the records it states, and the
    /// id each line states, in the file's order. The ids are read, not checked:
    /// `events_as_stated` checks them.
    pub fn parse(text: &str) -> Result<(Self, Vec<EntityId>), ParseError> {
        let mut reader = Reader::new(text);
        let mut ids = Vec::new();
        let mut kinds = Vec::new();
        while let Some(tokens) = reader.tokens() {
            let ["kind", name, tail @ ..] = tokens.as_slice() else { return Err(reader.unexpected()) };
            let (name, record) = (name.to_string(), reader.creation(tail, &mut ids)?);
            let description = reader.description()?;
            kinds.push(EdgeKindRecord { name, description, record });
        }
        if kinds.is_empty() {
            return Err(reader.unexpected());
        }
        let edge_kinds = EdgeKinds { kinds };
        canonical(text, &edge_kinds.text(&ids))?;
        Ok((edge_kinds, ids))
    }

    /// The writer's form, given an id for each record in the file's order.
    fn text(&self, ids: &[EntityId]) -> String {
        let names = self.kinds.iter().map(|kind| kind.name.len()).max().unwrap_or(0);
        let mut ids = ids.iter();
        let lines: Vec<Line> = self
            .kinds
            .iter()
            .map(|kind| Line {
                description: Some(&kind.description),
                ..Line::new("kind", format!("{:<names$}", kind.name), ids.next(), &kind.record)
            })
            .collect();
        write_lines(&lines)
    }
}

/// One entity's line as the writer lays it out.
struct Line<'a> {
    keyword: &'static str,
    values: String,
    id: &'a EntityId,
    creation: &'a Creation,
    description: Option<&'a str>,
}

impl<'a> Line<'a> {
    /// `id` is the next of the ids given for the file's entities, one for each.
    fn new(keyword: &'static str, values: String, id: Option<&'a EntityId>, creation: &'a Creation) -> Self {
        Line { keyword, values, id: id.expect("an id for every entity in the file"), creation, description: None }
    }
}

/// Lays the lines out, each line's values padded to the widest, so that ids and creation
/// fields line up.
fn write_lines(lines: &[Line]) -> String {
    let keywords = lines.iter().map(|line| line.keyword.len()).max().unwrap_or(0);
    let values = lines.iter().map(|line| line.values.len()).max().unwrap_or(0);
    let mut text = String::new();
    for line in lines {
        let Creation { at, nonce } = line.creation;
        let id = line.id.to_base64();
        let row = format!("{:<keywords$}  {:<values$}  id {id}  at {at}  nonce {}", line.keyword, line.values, hex(nonce));
        text.push_str(&row);
        text.push('\n');
        if let Some(description) = line.description {
            text.push_str(&format!("    {description}\n"));
        }
    }
    text
}

/// Refuses `text` unless it is exactly `written`, the writer's form of what it parsed to,
/// naming the first line that differs.
fn canonical(text: &str, written: &str) -> Result<(), ParseError> {
    if text == written {
        return Ok(());
    }
    let differs = text.split_inclusive('\n').zip(written.split_inclusive('\n')).position(|(read, wrote)| read != wrote);
    let line = differs.unwrap_or_else(|| text.split_inclusive('\n').count().min(written.split_inclusive('\n').count()));
    Err(ParseError { line: line + 1, problem: Problem::NotCanonical })
}

/// Reads a file's lines in order.
struct Reader<'t> {
    lines: std::iter::Enumerate<std::str::Lines<'t>>,
    /// The number of the line read last, counted from one; one before any is read.
    line: usize,
}

impl<'t> Reader<'t> {
    fn new(text: &'t str) -> Self {
        Reader { lines: text.lines().enumerate(), line: 1 }
    }

    /// The next line's fields, split at runs of spaces; none at the end of the file.
    fn tokens(&mut self) -> Option<Vec<&'t str>> {
        let (index, line) = self.lines.next()?;
        self.line = index + 1;
        Some(line.split_whitespace().collect())
    }

    /// The text of the indented line beneath a kind.
    fn description(&mut self) -> Result<String, ParseError> {
        let (index, line) = self.lines.next().ok_or(ParseError { line: self.line + 1, problem: Problem::Unexpected })?;
        self.line = index + 1;
        match line.strip_prefix("    ") {
            Some(description) if !description.trim().is_empty() => Ok(description.to_owned()),
            _ => Err(self.unexpected()),
        }
    }

    /// The fields that end every entity's line, `id <id>  at <ms>  nonce <hex>`, adding the
    /// id to `ids`.
    fn creation(&self, tail: &[&str], ids: &mut Vec<EntityId>) -> Result<Creation, ParseError> {
        let ["id", id, "at", at, "nonce", nonce] = tail else { return Err(self.unexpected()) };
        ids.push(EntityId::from_base64(id).map_err(|_| self.error(Problem::Field("id")))?);
        Ok(Creation {
            at: at.parse().map_err(|_| self.error(Problem::Field("time")))?,
            nonce: unhex(nonce).ok_or(self.error(Problem::Field("nonce")))?,
        })
    }

    fn optional(&self, word: &str) -> Result<bool, ParseError> {
        match word {
            "optional" => Ok(true),
            "required" => Ok(false),
            _ => Err(self.error(Problem::Field("optionality"))),
        }
    }

    fn unexpected(&self) -> ParseError {
        self.error(Problem::Unexpected)
    }

    fn error(&self, problem: Problem) -> ParseError {
        ParseError { line: self.line, problem }
    }
}

fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Exactly 64 lowercase hex digits.
fn unhex(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64 || !text.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')) {
        return None;
    }
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[2 * index..2 * index + 2], 16).ok()?;
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn creation(byte: u8) -> Creation {
        Creation { at: 1_794_794_543_822, nonce: [byte; 32] }
    }

    fn intention() -> ModelDefinitions {
        let property = |name: &str, optional, byte| PropertyDefinition {
            name: name.to_owned(),
            value_type: "string".to_owned(),
            backend: "lww".to_owned(),
            optional,
            property: creation(byte),
            link: creation(byte + 1),
        };
        ModelDefinitions {
            label: "example.intention".to_owned(),
            name: "Intention".to_owned(),
            model: creation(1),
            properties: vec![property("title", false, 2), property("description", true, 4)],
        }
    }

    fn ids(bytes: impl IntoIterator<Item = u8>) -> Vec<EntityId> {
        bytes.into_iter().map(|byte| EntityId::from_bytes([byte; 32])).collect()
    }

    /// The writer's form, fixed to its last byte, reads back as what was written, with the ids
    /// its lines state.
    #[test]
    fn a_minted_file_round_trips_through_its_text() {
        let tail = |byte: u8| {
            let id = EntityId::from_bytes([byte; 32]).to_base64();
            format!("id {id}  at 1794794543822  nonce {}", format!("{byte:02x}").repeat(32))
        };
        let text = intention().text(&ids(1..=5));
        assert_eq!(
            text,
            format!(
                "model     example.intention  Intention  {}\n\
                 property  title        string  lww      {}\n\
                 link      title        required         {}\n\
                 property  description  string  lww      {}\n\
                 link      description  optional         {}\n",
                tail(1),
                tail(2),
                tail(3),
                tail(4),
                tail(5)
            )
        );
        assert_eq!(ModelDefinitions::parse(&text), Ok((intention(), ids(1..=5))));

        let kinds = EdgeKinds {
            kinds: vec![EdgeKindRecord {
                name: "links".to_owned(),
                description: "From a thing to a thing.".to_owned(),
                record: creation(6),
            }],
        };
        let text = kinds.text(&ids([6]));
        assert_eq!(text, format!("kind  links  {}\n    From a thing to a thing.\n", tail(6)));
        assert_eq!(EdgeKinds::parse(&text), Ok((kinds, ids([6]))));
    }

    #[test]
    fn a_file_not_in_the_writers_form_is_refused_naming_the_line() {
        let text = intention().text(&ids(1..=5));
        let refused = |text: &str| ModelDefinitions::parse(text).unwrap_err();
        let respaced = text.replacen("title        string", "title  string", 1);
        assert_eq!(refused(&respaced), ParseError { line: 2, problem: Problem::NotCanonical });
        let relinked = text.replacen("link      title      ", "link      description", 1);
        assert_eq!(refused(&relinked), ParseError { line: 3, problem: Problem::LinkToAnother });
        assert_eq!(refused(&text.replacen("property", "proprety", 1)), ParseError { line: 2, problem: Problem::Unexpected });
    }
}
