//! Declares a model's minted id in the struct that declares the model: the `id` option of its
//! `#[model(...)]` attributes. The source is parsed rather than read as lines, so that only the
//! id's literal changes, or one attribute is added, and every other option, attribute and
//! comment stays as written.

use std::path::{Path, PathBuf};

use ankurah::EntityId;
use proc_macro2::{LineColumn, Span};
use syn::{punctuated::Punctuated, Attribute, Item, ItemStruct, LitStr, Token};
use thiserror::Error;

/// Why a struct's source cannot be given its model's id.
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
}

/// `text`, the source at `path`, with the struct `name` declaring `id` as its model's id: the
/// literal of the id it declares replaced, or, if it declares none, `#[model(id = "<id>")]`
/// added on the line after the attribute that gives its label, or after its derive.
pub fn declared(path: &Path, text: &str, name: &str, id: EntityId) -> Result<String, DeclareError> {
    let file = syn::parse_file(text).map_err(|source| DeclareError::Unparsable { path: path.to_owned(), source })?;
    let model = file
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(item) if item.ident == name && derives_model(&item.attrs) => Some(item),
            _ => None,
        })
        .ok_or_else(|| DeclareError::NoModel { path: path.to_owned(), name: name.to_owned() })?;
    let options = Options::read(model).map_err(|source| DeclareError::UnreadableAttributes {
        path: path.to_owned(),
        name: name.to_owned(),
        source,
    })?;
    let literal = format!("\"{}\"", id.to_base64());
    match options.ids[..] {
        [declared] => Ok(format!("{}{literal}{}", &text[..offset(text, declared.start())], &text[offset(text, declared.end())..])),
        [] => {
            let after = options.label.or_else(|| model.attrs.iter().find(|attribute| derives_model(std::slice::from_ref(attribute))));
            let after = after.expect("a struct that derives Model has an attribute that derives it");
            let start = after.pound_token.span.start();
            let line = &text[offset(text, LineColumn { line: start.line, column: 0 })..];
            let indent = &line[..line.len() - line.trim_start().len()];
            let end = offset(text, after.bracket_token.span.close().end());
            Ok(format!("{}\n{indent}#[model(id = {literal})]{}", &text[..end], &text[end..]))
        }
        _ => Err(DeclareError::SeveralIds { path: path.to_owned(), name: name.to_owned(), count: options.ids.len() }),
    }
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

fn derives_model(attributes: &[Attribute]) -> bool {
    attributes.iter().filter(|attribute| attribute.path().is_ident("derive")).any(|attribute| {
        attribute
            .parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)
            .is_ok_and(|derived| derived.iter().any(|path| path.segments.last().is_some_and(|segment| segment.ident == "Model")))
    })
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

    fn old() -> String {
        EntityId::from_bytes([1; 32]).to_base64()
    }

    fn declared(text: &str) -> Result<String, DeclareError> {
        super::declared(Path::new("example.rs"), text, "Example", id())
    }

    /// A struct as the Commons declares one, with `attributes` between its derive and itself.
    fn model(attributes: &str) -> String {
        format!("/// An example — café.\n#[derive(Model, Debug)]\n{attributes}\npub struct Example {{\n    pub title: String,\n}}\n")
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
}
