//! Declares a model's minted ids in the struct that declares the model, so that a node binds
//! the model and each of its properties by their ids, never by label or name: the model's id
//! as the `id` option of the struct's `#[model(...)]` attributes, and each property's as the
//! `id` option of the `#[property(...)]` attributes of the field that declares the property.
//! The source is parsed rather than read as lines, so that only an id's literal changes, or one
//! attribute is added for an id not yet declared, and every other option, attribute and comment
//! stays as written.

use std::{
    ops::Range,
    path::{Path, PathBuf},
};

use ankurah::EntityId;
use proc_macro2::{LineColumn, Span};
use syn::{punctuated::Punctuated, Attribute, Field, Ident, Item, ItemStruct, LitStr, Token, Visibility};
use thiserror::Error;

/// Why a struct's source cannot be given its minted ids.
#[derive(Debug, Error)]
pub enum DeclareError {
    #[error("{}: does not parse: {source}", path.display())]
    Unparsable { path: PathBuf, source: syn::Error },
    #[error("{}: declares no struct `{name}` that derives Model", path.display())]
    NoModel { path: PathBuf, name: String },
    #[error("{}: the #[model(...)] attributes of `{name}` do not read: {source}", path.display())]
    UnreadableAttributes { path: PathBuf, name: String, source: syn::Error },
    #[error("{}: `{name}` declares {count} ids", path.display())]
    SeveralIds { path: PathBuf, name: String, count: usize },
    /// The struct declares other properties than the declaration the tool was built from, as
    /// when `--src` names other sources.
    #[error("{}: `{name}` declares the properties {declared:?}, and the mint has ids for {minted:?}", path.display())]
    OtherProperties { path: PathBuf, name: String, declared: Vec<String>, minted: Vec<String> },
    #[error("{}: the #[property(...)] attributes of `{name}`'s field `{field}` do not read: {source}", path.display())]
    UnreadablePropertyAttributes { path: PathBuf, name: String, field: String, source: syn::Error },
    #[error("{}: `{name}`'s field `{field}` declares {count} ids", path.display())]
    SeveralPropertyIds { path: PathBuf, name: String, field: String, count: usize },
}

/// `text`, the source at `path`, with the struct `name` declaring `model` as its model's id, and
/// each field that declares a property the id that `properties` gives by the field's name.
/// `properties` names every such field, in the order the struct declares them. An id the struct
/// declares already has its literal replaced. A model that declares none is given
/// `#[model(id = "<id>")]` on the line after the attribute that gives its label, or after its
/// derive; a property that declares none is given `#[property(id = "<id>")]` directly before
/// its field, after the field's doc comment and other attributes.
pub fn declared(path: &Path, text: &str, name: &str, model: EntityId, properties: &[(&str, EntityId)]) -> Result<String, DeclareError> {
    let file = syn::parse_file(text).map_err(|source| DeclareError::Unparsable { path: path.to_owned(), source })?;
    let item = file
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(item) if item.ident == name && derives_model(&item.attrs) => Some(item),
            _ => None,
        })
        .ok_or_else(|| DeclareError::NoModel { path: path.to_owned(), name: name.to_owned() })?;
    let options = Options::read(item).map_err(|source| DeclareError::UnreadableAttributes {
        path: path.to_owned(),
        name: name.to_owned(),
        source,
    })?;
    let mut edits = Vec::with_capacity(1 + properties.len());
    edits.push(match options.ids[..] {
        [declared] => Edit::replacing(text, declared, model),
        [] => {
            let after = options.label.or_else(|| item.attrs.iter().find(|attribute| derives_model(std::slice::from_ref(attribute))));
            Edit::adding_model_id(text, after.expect("a struct that derives Model has an attribute that derives it"), model)
        }
        _ => return Err(DeclareError::SeveralIds { path: path.to_owned(), name: name.to_owned(), count: options.ids.len() }),
    });

    let fields: Vec<(&Ident, &Field)> = property_fields(item).collect();
    let names: Vec<String> = fields.iter().map(|(ident, _)| ident.to_string()).collect();
    if !names.iter().eq(properties.iter().map(|(field, _)| field)) {
        let minted = properties.iter().map(|&(field, _)| field.to_owned()).collect();
        return Err(DeclareError::OtherProperties { path: path.to_owned(), name: name.to_owned(), declared: names, minted });
    }
    for ((ident, field), &(_, id)) in fields.into_iter().zip(properties) {
        let ids = property_ids(field).map_err(|source| DeclareError::UnreadablePropertyAttributes {
            path: path.to_owned(),
            name: name.to_owned(),
            field: ident.to_string(),
            source,
        })?;
        edits.push(match ids[..] {
            [declared] => Edit::replacing(text, declared, id),
            [] => Edit::adding_property_id(text, ident, field, id),
            _ => {
                let (path, name, field) = (path.to_owned(), name.to_owned(), ident.to_string());
                return Err(DeclareError::SeveralPropertyIds { path, name, field, count: ids.len() });
            }
        });
    }
    Ok(edited(text, edits))
}

/// What a model's `#[model(...)]` attributes declare that placing its id needs.
struct Options<'a> {
    /// The literal of each `id` option.
    ids: Vec<Span>,
    /// The attribute that gives the model's label.
    label: Option<&'a Attribute>,
}

impl<'a> Options<'a> {
    fn read(model: &'a ItemStruct) -> syn::Result<Self> {
        let mut options = Options { ids: Vec::new(), label: None };
        for attribute in model.attrs.iter().filter(|attribute| attribute.path().is_ident("model")) {
            attribute.parse_nested_meta(|meta| {
                // Every option that takes a value takes a string; the others are flags.
                if meta.input.peek(Token![=]) {
                    let value: LitStr = meta.value()?.parse()?;
                    if meta.path.is_ident("id") {
                        options.ids.push(value.span());
                    } else if meta.path.is_ident("label") {
                        options.label = Some(attribute);
                    }
                }
                Ok(())
            })?;
        }
        Ok(options)
    }
}

/// The fields of `model` that the derive makes properties of, in the order the struct declares
/// them: every named field but those marked `#[model(ephemeral)]`.
fn property_fields(model: &ItemStruct) -> impl Iterator<Item = (&Ident, &Field)> {
    model.fields.iter().filter(|field| !ephemeral(field)).filter_map(|field| Some((field.ident.as_ref()?, field)))
}

/// Whether `field` is marked `#[model(ephemeral)]`, read as the derive reads the mark: an
/// attribute whose path has a segment `model` and whose one argument is `ephemeral`.
fn ephemeral(field: &Field) -> bool {
    field.attrs.iter().any(|attribute| {
        attribute.path().segments.iter().any(|segment| segment.ident == "model")
            && attribute.meta.require_list().ok().and_then(|list| list.parse_args::<Ident>().ok()).is_some_and(|mark| mark == "ephemeral")
    })
}

/// The literal of each `id` option of `field`'s `#[property(...)]` attributes.
fn property_ids(field: &Field) -> syn::Result<Vec<Span>> {
    let mut ids = Vec::new();
    for attribute in field.attrs.iter().filter(|attribute| attribute.path().is_ident("property")) {
        attribute.parse_nested_meta(|meta| {
            // Every option takes a string.
            let value: LitStr = meta.value()?.parse()?;
            if meta.path.is_ident("id") {
                ids.push(value.span());
            }
            Ok(())
        })?;
    }
    Ok(ids)
}

fn derives_model(attributes: &[Attribute]) -> bool {
    attributes.iter().filter(|attribute| attribute.path().is_ident("derive")).any(|attribute| {
        attribute
            .parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)
            .is_ok_and(|derived| derived.iter().any(|path| path.segments.last().is_some_and(|segment| segment.ident == "Model")))
    })
}

/// A change to a source: the text at `range` replaced with `text`.
struct Edit {
    range: Range<usize>,
    text: String,
}

impl Edit {
    /// The id literal at `literal` in `source` made to read `id`.
    fn replacing(source: &str, literal: Span, id: EntityId) -> Edit {
        Edit { range: offset(source, literal.start())..offset(source, literal.end()), text: literal_of(id) }
    }

    /// `#[model(id = "<id>")]` added on the line after the attribute `after`, at its indentation.
    fn adding_model_id(source: &str, after: &Attribute, id: EntityId) -> Edit {
        let start = after.pound_token.span.start();
        let line = &source[offset(source, LineColumn { line: start.line, column: 0 })..];
        let indent = &line[..line.len() - line.trim_start().len()];
        let end = offset(source, after.bracket_token.span.close().end());
        Edit { range: end..end, text: format!("\n{indent}#[model(id = {})]", literal_of(id)) }
    }

    /// `#[property(id = "<id>")]` added directly before the field `ident` names, after its
    /// attributes: on a line of its own at the field's indentation, or, if something stands
    /// before the field on its line, before the field on that line.
    fn adding_property_id(source: &str, ident: &Ident, field: &Field, id: EntityId) -> Edit {
        // The field's first token after its attributes.
        let first = match &field.vis {
            Visibility::Public(token) => token.span,
            Visibility::Restricted(restricted) => restricted.pub_token.span,
            Visibility::Inherited => ident.span(),
        }
        .start();
        let at = offset(source, first);
        let before = &source[offset(source, LineColumn { line: first.line, column: 0 })..at];
        let attribute = format!("#[property(id = {})]", literal_of(id));
        let text = if before.trim().is_empty() { format!("{attribute}\n{before}") } else { format!("{attribute} ") };
        Edit { range: at..at, text }
    }
}

/// `source` with `edits`, of which no two overlap, made.
fn edited(source: &str, mut edits: Vec<Edit>) -> String {
    edits.sort_by_key(|edit| edit.range.start);
    let mut edited = source.to_owned();
    // The last first, so that the range of each edit still holds the text it was made for.
    for edit in edits.into_iter().rev() {
        edited.replace_range(edit.range, &edit.text);
    }
    edited
}

/// The string literal of `id`.
fn literal_of(id: EntityId) -> String {
    format!("\"{}\"", id.to_base64())
}

/// The byte offset in `text` of a place as proc-macro2 gives it: a line counted from one, and
/// a column in characters counted from zero.
fn offset(text: &str, at: LineColumn) -> usize {
    let start: usize = text.split_inclusive('\n').take(at.line - 1).map(str::len).sum();
    let line = &text[start..];
    start + line.char_indices().nth(at.column).map_or(line.len(), |(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> EntityId {
        EntityId::from_bytes([7; 32])
    }

    /// The id the mint gives the property `title`.
    fn title() -> EntityId {
        EntityId::from_bytes([8; 32])
    }

    fn old() -> String {
        EntityId::from_bytes([1; 32]).to_base64()
    }

    /// `text` with the struct `Example` given its model's id and the id of its one property,
    /// `title`.
    fn declared(text: &str) -> Result<String, DeclareError> {
        super::declared(Path::new("example.rs"), text, "Example", id(), &[("title", title())])
    }

    /// A struct as the Commons declares one, with `attributes` between its derive and itself,
    /// and a property that declares its id.
    fn model(attributes: &str) -> String {
        let title = title().to_base64();
        format!(
            "/// An example — café.\n#[derive(Model, Debug)]\n{attributes}\npub struct Example {{\n    #[property(id = \"{title}\")]\n    pub title: String,\n}}\n"
        )
    }

    /// A struct that declares its model's id, with `fields` between its braces.
    fn with_fields(fields: &str) -> String {
        let id = id().to_base64();
        format!("#[derive(Model, Debug)]\n#[model(label = \"example.example\", id = \"{id}\")]\npub struct Example {{\n{fields}}}\n")
    }

    #[test]
    fn the_declared_id_is_replaced_where_it_stands_and_nothing_else_changes() {
        let old = old();
        let layouts = [
            format!("#[model(label = \"example.example\")]\n#[model(id = \"{old}\")]\n#[model(no_ffi)]"),
            format!("#[model(id = \"{old}\")]\n#[model(label = \"example.example\")]\n#[model(no_ffi)]"),
            format!("#[model(label = \"example.example\", id = \"{old}\", no_ffi)]"),
            format!("#[model(\n    label = \"example.example\",\n    id = \"{old}\",\n)]\n#[model(no_ffi)]"),
        ];
        for layout in layouts {
            let text = model(&layout);
            assert_eq!(declared(&text).unwrap(), text.replace(&old, &id().to_base64()), "{layout}");
        }
    }

    #[test]
    fn a_model_that_declares_no_id_is_given_one_after_its_label() {
        let text = model("#[model(label = \"example.example\", no_ffi)]\n#[serde(default)]");
        let given =
            model(&format!("#[model(label = \"example.example\", no_ffi)]\n#[model(id = \"{}\")]\n#[serde(default)]", id().to_base64()));
        assert_eq!(declared(&text).unwrap(), given);
    }

    /// Comments and literals that read like the attributes are not the struct's attributes.
    #[test]
    fn text_that_only_looks_like_an_attribute_is_left_alone() {
        let old = old();
        let lookalikes = format!(
            "/* #[model(label = \"example.example\")]\n#[model(id = \"{old}\")] */\nconst SAMPLE: &str = \"#[model(id = \\\"{old}\\\")]\";\n"
        );
        let text = format!("{lookalikes}{}", model(&format!("#[model(no_ffi, label = \"example.example\")]\n#[model(id = \"{old}\")]")));
        let given = format!(
            "{lookalikes}{}",
            model(&format!("#[model(no_ffi, label = \"example.example\")]\n#[model(id = \"{}\")]", id().to_base64()))
        );
        assert_eq!(declared(&text).unwrap(), given);
    }

    #[test]
    fn a_struct_that_declares_two_ids_or_none_that_is_a_model_is_refused() {
        let twice = model(&format!("#[model(label = \"example.example\", id = \"{}\")]\n#[model(id = \"{}\")]", old(), old()));
        assert!(matches!(declared(&twice), Err(DeclareError::SeveralIds { count: 2, .. })));
        let not_a_model = "#[derive(Debug)]\npub struct Example {}\n";
        assert!(matches!(declared(not_a_model), Err(DeclareError::NoModel { .. })));
        assert!(matches!(declared(&model("").replace("Example", "Other")), Err(DeclareError::NoModel { .. })));
    }

    /// A struct that declares no id at all, as a model declared anew does, is given its model's
    /// and its property's in one pass.
    #[test]
    fn a_struct_that_declares_no_id_is_given_every_one() {
        let text = "#[derive(Model, Debug)]\n#[model(label = \"example.example\")]\npub struct Example {\n    pub title: String,\n}\n";
        let given = format!(
            "#[derive(Model, Debug)]\n#[model(label = \"example.example\")]\n#[model(id = \"{}\")]\npub struct Example {{\n    #[property(id = \"{}\")]\n    pub title: String,\n}}\n",
            id().to_base64(),
            title().to_base64()
        );
        assert_eq!(declared(text).unwrap(), given);
    }

    /// The attribute goes on a line of its own at the field's indentation, below the field's doc
    /// comment, its comments and its other attributes, whatever its visibility; a field that
    /// shares its line with what stands before it is given its id on that line.
    #[test]
    fn a_property_that_declares_no_id_is_given_one_directly_above_its_field() {
        let attribute = format!("#[property(id = \"{}\")]", title().to_base64());
        let layouts = [
            (
                "    /// The title — café.\n    // OPEN: a question.\n    #[active_type(LWW)]\n    pub title: String,\n",
                format!("    /// The title — café.\n    // OPEN: a question.\n    #[active_type(LWW)]\n    {attribute}\n    pub title: String,\n"),
            ),
            ("    pub(crate) title: String,\n", format!("    {attribute}\n    pub(crate) title: String,\n")),
            ("\ttitle: String,\n", format!("\t{attribute}\n\ttitle: String,\n")),
            ("    #[active_type(LWW)] pub title: String,\n", format!("    #[active_type(LWW)] {attribute} pub title: String,\n")),
        ];
        for (fields, given) in layouts {
            assert_eq!(declared(&with_fields(fields)).unwrap(), with_fields(&given), "{fields}");
        }
    }

    #[test]
    fn a_declared_property_id_is_replaced_where_it_stands_and_nothing_else_changes() {
        let old = old();
        let layouts = [
            format!("    #[property(id = \"{old}\")]\n    pub title: String,\n"),
            format!("    #[active_type(LWW)]\n    #[property(renamed_from = \"heading\", id = \"{old}\")]\n    pub title: String,\n"),
            format!(
                "    #[property(\n        id = \"{old}\",\n    )]\n    #[property(renamed_from = \"heading\")]\n    pub title: String,\n"
            ),
        ];
        for layout in layouts {
            let text = with_fields(&layout);
            assert_eq!(declared(&text).unwrap(), text.replace(&old, &title().to_base64()), "{layout}");
        }
    }

    /// A property renamed from a former name keeps that option as written, and is given its id
    /// in an attribute of its own.
    #[test]
    fn a_property_that_declares_only_its_former_name_keeps_it_and_is_given_its_id() {
        let text = with_fields("    #[property(renamed_from = \"heading\")]\n    #[active_type(LWW)]\n    pub title: String,\n");
        let given = with_fields(&format!(
            "    #[property(renamed_from = \"heading\")]\n    #[active_type(LWW)]\n    #[property(id = \"{}\")]\n    pub title: String,\n",
            title().to_base64()
        ));
        assert_eq!(declared(&text).unwrap(), given);
    }

    #[test]
    fn a_field_that_declares_two_ids_is_refused_naming_it() {
        let old = old();
        let layouts = [
            format!(
                "    #[property(id = \"{old}\")]\n    #[property(renamed_from = \"heading\", id = \"{old}\")]\n    pub title: String,\n"
            ),
            format!("    #[property(id = \"{old}\", id = \"{old}\")]\n    pub title: String,\n"),
        ];
        for layout in layouts {
            let error = declared(&with_fields(&layout)).unwrap_err();
            assert!(matches!(&error, DeclareError::SeveralPropertyIds { field, count: 2, .. } if field == "title"), "{error}");
        }
    }

    /// The derive makes no property of a field marked `#[model(ephemeral)]`, so the mint has no
    /// id for it, and the field is given none.
    #[test]
    fn an_ephemeral_field_is_no_property_and_is_given_no_id() {
        let text = with_fields("    #[model(ephemeral)]\n    pub scratch: String,\n    pub title: String,\n");
        let given = with_fields(&format!(
            "    #[model(ephemeral)]\n    pub scratch: String,\n    #[property(id = \"{}\")]\n    pub title: String,\n",
            title().to_base64()
        ));
        assert_eq!(declared(&text).unwrap(), given);
    }

    /// Sources other than those the tool was built from may declare properties the mint has no
    /// id for, or lack one it has an id for; both lists are named.
    #[test]
    fn a_struct_whose_properties_are_not_those_minted_is_refused_naming_both() {
        let error = declared(&with_fields("    pub title: String,\n    pub subtitle: String,\n")).unwrap_err();
        assert!(
            matches!(&error, DeclareError::OtherProperties { declared, minted, .. } if *declared == ["title", "subtitle"] && *minted == ["title"]),
            "{error}"
        );
    }
}
