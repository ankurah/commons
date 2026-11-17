//! Reads a file's tokens as blocks, one for each event, with names not yet resolved.

use std::str::FromStr;

use ankurah::{
    proto::{BackendOperation, ModelId},
    value::Value,
    EntityId, PropertyId,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use super::{
    lexing::{Lexed, Token},
    naming::is_name,
    read_error::{At, Problem},
    timestamp,
};

/// One event as a file states it.
#[derive(Debug)]
pub(super) struct Block<'t> {
    /// The header's line.
    pub line: usize,
    pub header: EntityId,
    pub nonce: [u8; 32],
    pub timestamp: u64,
    pub id: Option<Stated<EntityId>>,
    pub backends: Vec<BackendField<'t>>,
    pub memberships: Vec<Stated<Reference<'t, ModelId>>>,
}

#[derive(Debug)]
pub(super) struct Stated<T> {
    pub line: usize,
    pub value: T,
}

/// Something given as it is, or the name of an entity whose id it is.
#[derive(Debug)]
pub(super) enum Reference<'t, T> {
    Given(T),
    Named(Name<'t>),
}

/// `$name`, an entity of the same file, or `name@file`, one of the file at `file`, relative
/// to the directory of the file that refers.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Name<'t> {
    pub name: &'t str,
    pub file: Option<&'t str>,
}

/// A backend's key and its operations.
#[derive(Debug)]
pub(super) struct BackendField<'t> {
    pub line: usize,
    pub backend: &'t str,
    pub operations: StatedOperations<'t>,
}

#[derive(Debug)]
pub(super) enum StatedOperations<'t> {
    Values(Vec<Entry<'t>>),
    Bytes(BackendOperation),
}

/// One of a backend's values, `property: value`.
#[derive(Debug)]
pub(super) struct Entry<'t> {
    pub line: usize,
    pub property: Reference<'t, PropertyId>,
    pub value: Reference<'t, Option<Value>>,
}

/// The blocks of a file's tokens: the runs of tokens between blank lines, but for a run
/// that holds only comments.
pub(super) fn blocks<'t>(tokens: &[Lexed<'t>]) -> Result<Vec<Block<'t>>, At> {
    tokens
        .split(|lexed| lexed.token == Token::Blank)
        .map(|run| {
            let start = run.iter().position(|lexed| lexed.token != Token::LineBreak).unwrap_or(run.len());
            let end = run.iter().rposition(|lexed| lexed.token != Token::LineBreak).map_or(start, |last| last + 1);
            &run[start..end]
        })
        .filter(|run| !run.is_empty())
        .map(|run| Parser { tokens: run, at: 0 }.block())
        .collect()
}

/// Reads one block's tokens, which neither start nor end with a line break.
struct Parser<'a, 't> {
    tokens: &'a [Lexed<'t>],
    at: usize,
}

impl<'a, 't> Parser<'a, 't> {
    fn block(mut self) -> Result<Block<'t>, At> {
        let line = self.line_ahead();
        self.mark('[', "a header, `[<entity id>]`")?;
        let header = match self.next() {
            Some(&Token::Word(word)) => EntityId::from_base64(word).map_err(|source| self.problem(Problem::Id(source)))?,
            _ => return Err(self.expected("the header's entity id")),
        };
        self.mark(']', "`]` closing the header")?;
        let mut fields = Fields::default();
        if self.peek().is_some() {
            self.line_break("a line break after the header")?;
            loop {
                self.field(&mut fields)?;
                match self.next() {
                    None => break,
                    Some(Token::Mark(',')) => self.skip_line_breaks(),
                    Some(Token::LineBreak) => self.skip_line_breaks(),
                    Some(_) => return Err(self.expected("`,` or a line break between fields")),
                }
                if self.peek().is_none() {
                    return Err(self.expected("a field after `,`"));
                }
            }
        }
        let missing = |problem| At { line, problem };
        Ok(Block {
            line,
            header,
            nonce: fields.nonce.ok_or(missing(Problem::MissingNonce))?,
            timestamp: fields.timestamp.ok_or(missing(Problem::MissingTimestamp))?,
            id: fields.id,
            backends: fields.backends,
            memberships: fields.memberships,
        })
    }

    /// Reads a field, `key: value`, into `fields`.
    fn field(&mut self, fields: &mut Fields<'t>) -> Result<(), At> {
        let line = self.line_ahead();
        let (key, value) = self.key("a field, `key: value`")?;
        let repeated = |key| At { line, problem: Problem::RepeatedKey(key) };
        match key {
            "nonce" => {
                let nonce = EntityId::from_base64(self.word(value, "the nonce")?).map_err(|source| self.problem(Problem::Nonce(source)))?;
                fields.nonce.replace(nonce.to_bytes()).map_or(Ok(()), |_| Err(repeated("nonce")))
            }
            "timestamp" => {
                let timestamp =
                    timestamp::parse(self.word(value, "the timestamp")?).map_err(|source| self.problem(Problem::Timestamp(source)))?;
                fields.timestamp.replace(timestamp).map_or(Ok(()), |_| Err(repeated("timestamp")))
            }
            "id" => {
                let id = EntityId::from_base64(self.word(value, "the event's id")?).map_err(|source| self.problem(Problem::Id(source)))?;
                fields.id.replace(Stated { line, value: id }).map_or(Ok(()), |_| Err(repeated("id")))
            }
            "member" => {
                let word = self.word(value, "a model id or a name")?;
                let model = match self.name(word)? {
                    Some(name) => Reference::Named(name),
                    None => Reference::Given(ModelId::from_str(word).map_err(|source| self.problem(Problem::ModelId(source)))?),
                };
                fields.memberships.push(Stated { line, value: model });
                Ok(())
            }
            "parent" => Err(At { line, problem: Problem::Update }),
            backend => {
                if fields.backends.iter().any(|field| field.backend == backend) {
                    return Err(At { line, problem: Problem::RepeatedBackend(backend.to_owned()) });
                }
                let word = match value {
                    Some(word) => word,
                    None => match self.next() {
                        Some(Token::Mark('{')) => {
                            fields.backends.push(BackendField { line, backend, operations: StatedOperations::Values(self.values()?) });
                            return Ok(());
                        }
                        Some(&Token::Word(word)) => word,
                        _ => return Err(self.expected("the backend's values, `{ ... }`, or its operation's bytes in base64")),
                    },
                };
                let diff = URL_SAFE_NO_PAD.decode(word).map_err(|source| self.problem(Problem::Bytes(source)))?;
                let operations = StatedOperations::Bytes(BackendOperation { diff });
                fields.backends.push(BackendField { line, backend, operations });
                Ok(())
            }
        }
    }

    /// The entries of a backend's values, after the `{` that opens them.
    fn values(&mut self) -> Result<Vec<Entry<'t>>, At> {
        let mut entries = Vec::new();
        self.skip_line_breaks();
        if self.peek() == Some(&Token::Mark('}')) {
            self.at += 1;
            return Ok(entries);
        }
        loop {
            let line = self.line_ahead();
            let (key, value) = self.key("an entry, `property: value`")?;
            let property = match self.name(key)? {
                Some(name) => Reference::Named(name),
                None => Reference::Given(PropertyId::from_str(key).map_err(|source| self.problem(Problem::Property(source)))?),
            };
            let value = match value {
                Some(word) => self.value(word)?,
                None => match self.next() {
                    Some(Token::Text(text)) => Reference::Given(Some(Value::String(text.clone()))),
                    Some(&Token::Word(word)) => self.value(word)?,
                    _ => return Err(self.expected("a value")),
                },
            };
            entries.push(Entry { line, property, value });
            match self.next() {
                Some(Token::Mark('}')) => return Ok(entries),
                Some(Token::Mark(',')) => {
                    self.skip_line_breaks();
                    if self.peek() == Some(&Token::Mark('}')) {
                        return Err(self.expected("an entry after `,`"));
                    }
                }
                Some(Token::LineBreak) => {
                    self.skip_line_breaks();
                    if self.peek() == Some(&Token::Mark('}')) {
                        self.at += 1;
                        return Ok(entries);
                    }
                }
                _ => return Err(self.expected("`,`, a line break or `}` after an entry")),
            }
        }
    }

    /// A value written as a word.
    fn value(&self, word: &'t str) -> Result<Reference<'t, Option<Value>>, At> {
        Ok(match word {
            "true" => Reference::Given(Some(Value::Bool(true))),
            "false" => Reference::Given(Some(Value::Bool(false))),
            "none" => Reference::Given(None),
            _ => match self.name(word)? {
                Some(name) => Reference::Named(name),
                None => Reference::Given(Some(Value::EntityId(
                    EntityId::from_base64(word).map_err(|_| self.problem(Problem::Value(word.to_owned())))?,
                ))),
            },
        })
    }

    /// The name `word` writes, if it holds `$` or `@`, the marks of a name.
    fn name(&self, word: &'t str) -> Result<Option<Name<'t>>, At> {
        if !word.contains(['$', '@']) {
            return Ok(None);
        }
        let name = match word.strip_prefix('$') {
            Some(name) => Name { name, file: None },
            None => match word.split_once('@') {
                Some((name, file)) => Name { name, file: Some(file) },
                None => unreachable!("a word that holds `$` or `@`"),
            },
        };
        let file_reads = name.file.is_none_or(|file| !file.is_empty() && !file.contains(['$', '@', ':']));
        if is_name(name.name) && file_reads {
            Ok(Some(name))
        } else {
            Err(self.problem(Problem::Name(word.to_owned())))
        }
    }

    /// A key, the text of a word before its first colon, and the value after the colon in
    /// the same word, if any.
    fn key(&mut self, expected: &'static str) -> Result<(&'t str, Option<&'t str>), At> {
        let Some(&Token::Word(word)) = self.next() else { return Err(self.expected(expected)) };
        match word.split_once(':') {
            Some((key, value)) if !key.is_empty() => Ok((key, Some(value).filter(|value| !value.is_empty()))),
            _ => Err(self.expected(expected)),
        }
    }

    /// The word a field's value is: `value`, which followed the key's colon in its word, or
    /// the word that comes next.
    fn word(&mut self, value: Option<&'t str>, expected: &'static str) -> Result<&'t str, At> {
        match value {
            Some(word) => Ok(word),
            None => match self.next() {
                Some(&Token::Word(word)) => Ok(word),
                _ => Err(self.expected(expected)),
            },
        }
    }

    fn mark(&mut self, mark: char, expected: &'static str) -> Result<(), At> {
        match self.next() {
            Some(Token::Mark(found)) if *found == mark => Ok(()),
            _ => Err(self.expected(expected)),
        }
    }

    fn line_break(&mut self, expected: &'static str) -> Result<(), At> {
        match self.next() {
            Some(Token::LineBreak) => {
                self.skip_line_breaks();
                Ok(())
            }
            _ => Err(self.expected(expected)),
        }
    }

    fn skip_line_breaks(&mut self) {
        while self.peek() == Some(&Token::LineBreak) {
            self.at += 1;
        }
    }

    fn next(&mut self) -> Option<&'a Token<'t>> {
        let token = self.peek();
        self.at += 1;
        token
    }

    fn peek(&self) -> Option<&'a Token<'t>> {
        self.tokens.get(self.at).map(|lexed| &lexed.token)
    }

    /// The line of the token read last, which is the block's last at its end.
    fn line(&self) -> usize {
        self.tokens[self.at.clamp(1, self.tokens.len()) - 1].line
    }

    /// The line of the token to read next.
    fn line_ahead(&self) -> usize {
        self.tokens[self.at.min(self.tokens.len() - 1)].line
    }

    fn problem(&self, problem: Problem) -> At {
        At { line: self.line(), problem }
    }

    fn expected(&self, expected: &'static str) -> At {
        self.problem(Problem::Expected(expected))
    }
}

/// The fields of a block read so far.
#[derive(Default)]
struct Fields<'t> {
    nonce: Option<[u8; 32]>,
    timestamp: Option<u64>,
    id: Option<Stated<EntityId>>,
    backends: Vec<BackendField<'t>>,
    memberships: Vec<Stated<Reference<'t, ModelId>>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::akev::lexing::tokens;

    const ID: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    fn refused(text: &str) -> At {
        blocks(&tokens(text).unwrap()).unwrap_err()
    }

    /// A block's fields after its header.
    fn block(fields: &str) -> String {
        format!("[{ID}]\nnonce: {ID}, timestamp: 1970-01-01T00:00:00.000Z\n{fields}\n")
    }

    #[test]
    fn a_run_of_comments_between_blank_lines_is_no_block() {
        let text = format!("// A file's opening words.\n\n{}\n/* and closing ones */\n", block("member: system:model"));
        assert_eq!(blocks(&tokens(&text).unwrap()).unwrap().len(), 1);
    }

    #[test]
    fn a_header_is_an_entity_id_alone_on_its_line() {
        assert!(matches!(refused("[$thing]\n"), At { line: 1, problem: Problem::Id(_) }));
        assert!(matches!(refused(&format!("[{ID}] nonce: {ID}\n")), At { line: 1, problem: Problem::Expected(_) }));
    }

    #[test]
    fn a_comma_with_no_field_or_entry_after_it_is_refused() {
        assert!(matches!(refused(&block("member: system:model,")), At { line: 3, problem: Problem::Expected(_) }));
        assert!(matches!(refused(&block("lww: { name: \"a\", }")), At { line: 3, problem: Problem::Expected(_) }));
    }

    #[test]
    fn a_word_with_the_marks_of_a_name_that_is_no_name_is_refused() {
        for word in ["$", "$a@b.akev", "a@", "@b.akev", "$a:b"] {
            let problem = refused(&block(&format!("member: {word}"))).problem;
            assert!(matches!(&problem, Problem::Name(refused) if refused == word), "{word}: {problem}");
        }
    }

    #[test]
    fn a_value_that_is_no_literal_and_no_id_is_refused() {
        assert!(matches!(refused(&block("lww: { name: maybe }")).problem, Problem::Value(word) if word == "maybe"));
        assert!(matches!(refused(&block("member: model")).problem, Problem::ModelId(_)));
        assert!(matches!(refused(&block("lww: { title: \"a\" }")).problem, Problem::Property(_)));
        assert!(matches!(refused(&block("lww: a")).problem, Problem::Bytes(_)));
    }
}
