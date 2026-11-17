//! The names the entities of one file answer to. An entity answers to the label its `lww`
//! text holds and, for a name no entity's label is, to the name it holds: the two built-in
//! properties that name things.

use std::collections::HashMap;

use ankurah::{proto::SystemProperty, value::Value, PropertyId};

/// The entities of one file that answer to each name, by their places in the file.
#[derive(Debug, Default)]
pub(super) struct Names<'a> {
    labels: HashMap<&'a str, Vec<usize>>,
    names: HashMap<&'a str, Vec<usize>>,
}

/// Why a name answers to no one entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Unanswered {
    Unknown,
    Ambiguous(usize),
}

impl<'a> Names<'a> {
    /// Adds the entity at `place`, whose `lww` text holds `label` and `name`.
    pub fn add(&mut self, place: usize, label: Option<&'a str>, name: Option<&'a str>) {
        if let Some(label) = label {
            self.labels.entry(label).or_default().push(place);
        }
        if let Some(name) = name {
            self.names.entry(name).or_default().push(place);
        }
    }

    /// The place of the one entity that answers to `name`.
    pub fn resolve(&self, name: &str) -> Result<usize, Unanswered> {
        let answering = self.labels.get(name).or_else(|| self.names.get(name));
        match answering.map(Vec::as_slice) {
            Some(&[place]) => Ok(place),
            Some(places) if !places.is_empty() => Err(Unanswered::Ambiguous(places.len())),
            _ => Err(Unanswered::Unknown),
        }
    }
}

/// The label and the name that `values`, an entity's `lww` text, hold.
pub(super) fn label_and_name<'a>(
    values: impl IntoIterator<Item = (&'a PropertyId, &'a Option<Value>)>,
) -> (Option<&'a str>, Option<&'a str>) {
    let (mut label, mut name) = (None, None);
    for (property, value) in values {
        let (PropertyId::System(built_in), Some(Value::String(text))) = (property, value) else { continue };
        match built_in {
            SystemProperty::Label => label = label.or(Some(text.as_str())),
            SystemProperty::Name => name = name.or(Some(text.as_str())),
            _ => {}
        }
    }
    (label, name)
}

/// Whether `text` can stand as a name after `$` or before `@`: letters, digits, `_`, `.`
/// and `-`, none of which ends a word or splits a key from its value.
pub(super) fn is_name(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A model labelled `example.thing` and named `Thing`, with properties named `title`,
    /// and two named `note`.
    fn names() -> Names<'static> {
        let mut names = Names::default();
        names.add(0, Some("example.thing"), Some("Thing"));
        names.add(1, None, Some("title"));
        names.add(2, None, Some("note"));
        names.add(3, None, Some("note"));
        names
    }

    #[test]
    fn an_entity_answers_to_its_label_and_to_a_name_that_is_no_label() {
        assert_eq!(names().resolve("example.thing"), Ok(0));
        assert_eq!(names().resolve("Thing"), Ok(0));
        assert_eq!(names().resolve("title"), Ok(1));
    }

    #[test]
    fn a_label_hides_the_entities_named_so() {
        let mut names = names();
        names.add(4, Some("title"), None);
        assert_eq!(names.resolve("title"), Ok(4));
    }

    #[test]
    fn a_name_no_entity_or_several_answer_to_is_unanswered() {
        assert_eq!(names().resolve("description"), Err(Unanswered::Unknown));
        assert_eq!(names().resolve("note"), Err(Unanswered::Ambiguous(2)));
    }

    #[test]
    fn a_name_holds_no_character_that_ends_a_word_or_splits_a_key() {
        assert!(is_name("records.lifecycle_edited-2"));
        for text in ["", "a:b", "a/b", "a$b", "a@b", "a b", "é"] {
            assert!(!is_name(text), "{text}");
        }
    }
}
