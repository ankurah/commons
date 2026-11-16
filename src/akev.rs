//! `.akev` files: minted creation events, written as text, so that a reader sees what was
//! minted and a node rebuilds the events' bytes from what it reads.
//!
//! A file states one model's definitions, beside the model's struct, or the records of
//! the kinds of edge the Commons mints. It opens with the minter every event names. Each
//! entity then takes one line: what it is first, then its id and what its creation event
//! holds that its values do not, the time it was minted and its nonce. An edge kind's
//! description takes the indented line beneath its kind, so that free text needs no
//! escaping. The id checks the rebuilt event, since an entity's id is the hash of its
//! creation event.
//!
//! A file parses only in exactly the form the writer gives it, so that two readers never
//! disagree about one.

use std::fmt;

use ankurah::EntityId;

/// What a creation event holds besides its minter and what the values on its line give.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Creation {
    pub id: EntityId,
    /// Unix milliseconds, as the minting node read its clock.
    pub at: u64,
    pub nonce: [u8; 32],
}

/// A model's minted definitions: the model, then each of its properties with the
/// model-property link that gives the model that property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelDefinitions {
    /// The ed25519 public key that every event names as its minter.
    pub minter: [u8; 32],
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
    /// The ed25519 public key that every event names as its minter.
    pub minter: [u8; 32],
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
    /// The first line names no ed25519 minter, or no blank line follows it.
    Minter,
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
            Problem::Minter => "expected `minter ed25519 <public key in hex>` and a blank line".to_owned(),
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
    pub fn write(&self) -> String {
        let names = self.properties.iter().map(|property| property.name.len()).max().unwrap_or(0);
        let types = self.properties.iter().map(|property| property.value_type.len()).max().unwrap_or(0);
        let mut lines = vec![Line::new("model", format!("{}  {}", self.label, self.name), &self.model)];
        for property in &self.properties {
            let typed = format!("{:<names$}  {:<types$}  {}", property.name, property.value_type, property.backend);
            lines.push(Line::new("property", typed, &property.property));
            let link = format!("{:<names$}  {}", property.name, if property.optional { "optional" } else { "required" });
            lines.push(Line::new("link", link, &property.link));
        }
        write_lines(&self.minter, &lines)
    }

    pub fn parse(text: &str) -> Result<Self, ParseError> {
        let mut reader = Reader::new(text)?;
        let (label, name, model) = match reader.tokens().as_deref() {
            Some(["model", label, name, tail @ ..]) => (label.to_string(), name.to_string(), reader.creation(tail)?),
            _ => return Err(reader.unexpected()),
        };
        let mut properties = Vec::new();
        while let Some(tokens) = reader.tokens() {
            let ["property", name, value_type, backend, tail @ ..] = tokens.as_slice() else { return Err(reader.unexpected()) };
            let (name, value_type, backend, property) =
                (name.to_string(), value_type.to_string(), backend.to_string(), reader.creation(tail)?);
            let (optional, link) = match reader.tokens().as_deref() {
                Some(["link", linked, optional, tail @ ..]) if *linked == name => (reader.optional(optional)?, reader.creation(tail)?),
                Some(["link", ..]) => return Err(reader.error(Problem::LinkToAnother)),
                _ => return Err(reader.unexpected()),
            };
            properties.push(PropertyDefinition { name, value_type, backend, optional, property, link });
        }
        let definitions = ModelDefinitions { minter: reader.minter, label, name, model, properties };
        canonical(text, &definitions.write())?;
        Ok(definitions)
    }
}

impl EdgeKinds {
    pub fn write(&self) -> String {
        let names = self.kinds.iter().map(|kind| kind.name.len()).max().unwrap_or(0);
        let lines: Vec<Line> = self
            .kinds
            .iter()
            .map(|kind| Line { description: Some(&kind.description), ..Line::new("kind", format!("{:<names$}", kind.name), &kind.record) })
            .collect();
        write_lines(&self.minter, &lines)
    }

    pub fn parse(text: &str) -> Result<Self, ParseError> {
        let mut reader = Reader::new(text)?;
        let mut kinds = Vec::new();
        while let Some(tokens) = reader.tokens() {
            let ["kind", name, tail @ ..] = tokens.as_slice() else { return Err(reader.unexpected()) };
            let (name, record) = (name.to_string(), reader.creation(tail)?);
            let description = reader.description()?;
            kinds.push(EdgeKindRecord { name, description, record });
        }
        if kinds.is_empty() {
            return Err(reader.unexpected());
        }
        let edge_kinds = EdgeKinds { minter: reader.minter, kinds };
        canonical(text, &edge_kinds.write())?;
        Ok(edge_kinds)
    }
}

/// One entity's line as the writer lays it out.
struct Line<'a> {
    keyword: &'static str,
    values: String,
    creation: &'a Creation,
    description: Option<&'a str>,
}

impl<'a> Line<'a> {
    fn new(keyword: &'static str, values: String, creation: &'a Creation) -> Self {
        Line { keyword, values, creation, description: None }
    }
}

/// Lays the lines out under the minter, each line's values padded to the widest, so that
/// ids and creation fields line up.
fn write_lines(minter: &[u8; 32], lines: &[Line]) -> String {
    let keywords = lines.iter().map(|line| line.keyword.len()).max().unwrap_or(0);
    let values = lines.iter().map(|line| line.values.len()).max().unwrap_or(0);
    let mut text = format!("minter ed25519 {}\n\n", hex(minter));
    for line in lines {
        let Creation { id, at, nonce } = line.creation;
        let row = format!("{:<keywords$}  {:<values$}  id {}  at {at}  nonce {}", line.keyword, line.values, id.to_base64(), hex(nonce));
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

/// Reads a file's lines in order, after its minter.
struct Reader<'t> {
    lines: std::iter::Peekable<std::iter::Enumerate<std::str::Lines<'t>>>,
    minter: [u8; 32],
    /// The number of the line read last, counted from one.
    line: usize,
}

impl<'t> Reader<'t> {
    fn new(text: &'t str) -> Result<Self, ParseError> {
        let mut lines = text.lines().enumerate().peekable();
        let minter = match (lines.next(), lines.next()) {
            (Some((_, first)), Some((_, ""))) => first.strip_prefix("minter ed25519 ").and_then(unhex),
            _ => None,
        };
        let minter = minter.ok_or(ParseError { line: 1, problem: Problem::Minter })?;
        Ok(Reader { lines, minter, line: 2 })
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

    /// The fields that end every entity's line: `id <id>  at <ms>  nonce <hex>`.
    fn creation(&self, tail: &[&str]) -> Result<Creation, ParseError> {
        let ["id", id, "at", at, "nonce", nonce] = tail else { return Err(self.unexpected()) };
        Ok(Creation {
            id: EntityId::from_base64(id).map_err(|_| self.error(Problem::Field("id")))?,
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
        Creation { id: EntityId::from_bytes([byte; 32]), at: 1_794_794_543_822, nonce: [byte; 32] }
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
            minter: [9; 32],
            label: "example.intention".to_owned(),
            name: "Intention".to_owned(),
            model: creation(1),
            properties: vec![property("title", false, 2), property("description", true, 4)],
        }
    }

    /// The writer's form, fixed to its last byte, reads back as what was written.
    #[test]
    fn a_minted_file_round_trips_through_its_text() {
        let tail = |byte: u8| {
            let id = EntityId::from_bytes([byte; 32]).to_base64();
            format!("id {id}  at 1794794543822  nonce {}", format!("{byte:02x}").repeat(32))
        };
        let text = intention().write();
        assert_eq!(
            text,
            format!(
                "minter ed25519 {}\n\n\
                 model     example.intention  Intention  {}\n\
                 property  title        string  lww      {}\n\
                 link      title        required         {}\n\
                 property  description  string  lww      {}\n\
                 link      description  optional         {}\n",
                "09".repeat(32),
                tail(1),
                tail(2),
                tail(3),
                tail(4),
                tail(5)
            )
        );
        assert_eq!(ModelDefinitions::parse(&text), Ok(intention()));

        let kinds = EdgeKinds {
            minter: [9; 32],
            kinds: vec![EdgeKindRecord {
                name: "links".to_owned(),
                description: "From a thing to a thing.".to_owned(),
                record: creation(6),
            }],
        };
        let text = kinds.write();
        assert_eq!(text, format!("minter ed25519 {}\n\nkind  links  {}\n    From a thing to a thing.\n", "09".repeat(32), tail(6)));
        assert_eq!(EdgeKinds::parse(&text), Ok(kinds));
    }

    #[test]
    fn a_file_not_in_the_writers_form_is_refused_naming_the_line() {
        let text = intention().write();
        let refused = |text: &str| ModelDefinitions::parse(text).unwrap_err();
        let respaced = text.replacen("title        string", "title  string", 1);
        assert_eq!(refused(&respaced), ParseError { line: 4, problem: Problem::NotCanonical });
        let relinked = text.replacen("link      title      ", "link      description", 1);
        assert_eq!(refused(&relinked), ParseError { line: 5, problem: Problem::LinkToAnother });
        assert_eq!(refused(&text.replacen("property", "proprety", 1)), ParseError { line: 4, problem: Problem::Unexpected });
        assert_eq!(refused(&text.replacen("minter ed25519", "minter rsa", 1)), ParseError { line: 1, problem: Problem::Minter });
    }
}
