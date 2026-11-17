//! Why the files of a read do not read.

use std::{error::Error, fmt, path::PathBuf};

use ankurah::{proto::DecodeError, EntityId};

use crate::akev::{genesis::BuildError, timestamp::TimestampError};

/// Why the files of a read do not read: the file and the line, counted from one, where they
/// first fail, and what is wrong there.
#[derive(Debug)]
pub struct ReadError {
    pub path: PathBuf,
    pub line: usize,
    pub problem: Problem,
}

#[derive(Debug)]
pub enum Problem {
    UnexpectedCharacter(char),
    UnclosedComment,
    UnclosedText,
    /// A string literal holds a line break; it writes one as `\n`.
    LineBreakInText,
    UnknownEscape(char),
    /// Not what this place in a block takes: what it takes instead.
    Expected(&'static str),
    /// A header, an `id` or a reference that is not an entity id.
    Id(DecodeError),
    Nonce(DecodeError),
    Timestamp(TimestampError),
    /// A `member` that is neither a model id nor a name.
    ModelId(DecodeError),
    /// A key in a backend's values that is neither a built-in property, an entity id nor a
    /// name.
    Property(DecodeError),
    /// A value in a backend's values that is not one the text form writes; as it is no
    /// literal, it would be an entity id, which it is not.
    Value {
        word: String,
        source: DecodeError,
    },
    /// A word that looks like a name, holding `$` or `@`, and is not `$name` or
    /// `name@file`.
    Name(String),
    Bytes(base64::DecodeError),
    MissingNonce,
    MissingTimestamp,
    /// A reserved key, other than `member`, given twice.
    RepeatedKey(&'static str),
    RepeatedBackend(String),
    /// A `parent`: an update, which this version does not read.
    Update,
    Build(BuildError),
    UnknownName(String),
    /// A name that more entities of its file than one answer to.
    AmbiguousName {
        name: String,
        entities: usize,
    },
    /// A file named by a reference that is not one of the read's.
    UnknownFile(PathBuf),
    /// A reference to an entity whose event needs, through its own references, the event
    /// that refers.
    ReferenceCycle,
    /// The event the block states is not the entity its header names.
    NotTheHeadersEntity {
        header: EntityId,
        built: EntityId,
    },
    /// An `id` other than the header, which for a genesis is the event's id.
    IdNotTheHeader {
        header: EntityId,
        stated: EntityId,
    },
    /// A second genesis of an entity that an earlier block of the read states.
    SecondGenesis {
        entity: EntityId,
        first: PathBuf,
        first_line: usize,
    },
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.path.display(), self.line, self.problem)
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Problem::UnexpectedCharacter(character) => write!(f, "`{character}` stands nowhere in the text form"),
            Problem::UnclosedComment => f.write_str("a comment opened with `/*` is not closed"),
            Problem::UnclosedText => f.write_str("a string is not closed"),
            Problem::LineBreakInText => f.write_str("a string holds a line break; write it `\\n`"),
            Problem::UnknownEscape(character) => write!(f, "`\\{character}` is no escape; the escapes are `\\\"`, `\\\\` and `\\n`"),
            Problem::Expected(expected) => write!(f, "expected {expected}"),
            Problem::Id(source) => write!(f, "not an entity id: {source}"),
            Problem::Nonce(source) => write!(f, "not a nonce, 32 bytes in base64: {source}"),
            Problem::Timestamp(source) => write!(f, "not a timestamp: {source}"),
            Problem::ModelId(source) => write!(f, "neither a model id nor a name: {source}"),
            Problem::Property(source) => write!(f, "neither a built-in property, an entity id nor a name: {source}"),
            Problem::Value { word, .. } => {
                write!(f, "`{word}` is no value: a value is a string, `true`, `false`, `none`, an entity id or a name")
            }
            Problem::Name(word) => write!(f, "`{word}` is no name: a name is `$name` or `name@file`"),
            Problem::Bytes(source) => write!(f, "not operation bytes in base64: {source}"),
            Problem::MissingNonce => f.write_str("the block states no nonce"),
            Problem::MissingTimestamp => f.write_str("the block states no timestamp"),
            Problem::RepeatedKey(key) => write!(f, "a second `{key}`"),
            Problem::RepeatedBackend(backend) => write!(f, "a second `{backend}`"),
            Problem::Update => f.write_str("a block with a parent states an update, and updates are not read yet"),
            Problem::Build(source) => write!(f, "the operations do not build: {source}"),
            Problem::UnknownName(name) => write!(f, "no entity of the file answers to `{name}`"),
            Problem::AmbiguousName { name, entities } => write!(f, "{entities} entities of the file answer to `{name}`"),
            Problem::UnknownFile(path) => write!(f, "{} is not among the files read", path.display()),
            Problem::ReferenceCycle => f.write_str("the reference needs the event that makes it"),
            Problem::NotTheHeadersEntity { header, built } => {
                write!(f, "the header names {header}, and the event the block states is the genesis of {built}")
            }
            Problem::IdNotTheHeader { header, stated } => write!(f, "the id {stated} is not the header's, {header}"),
            Problem::SecondGenesis { entity, first, first_line } => {
                write!(f, "a second genesis of {entity}, stated first at {}:{first_line}", first.display())
            }
        }
    }
}

impl Error for ReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.problem.source()
    }
}

impl Error for Problem {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Problem::Id(source)
            | Problem::Nonce(source)
            | Problem::ModelId(source)
            | Problem::Property(source)
            | Problem::Value { source, .. } => Some(source),
            Problem::Timestamp(source) => Some(source),
            Problem::Bytes(source) => Some(source),
            Problem::Build(source) => Some(source),
            _ => None,
        }
    }
}

/// A problem at a line of a file not yet named.
#[derive(Debug)]
pub(super) struct At {
    pub line: usize,
    pub problem: Problem,
}
