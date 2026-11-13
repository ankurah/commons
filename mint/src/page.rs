//! The page on which a reader who does not use Rust finds a namespace's models and the
//! ids of their definitions, each kept as its creation event and its state. `mint`
//! renders it from what the node's catalog said; it is a listing for readers that
//! nothing vouches for, and `verify` does not read it.

use crate::registering::ListedModel;

/// The page of the namespace `name`, whose models are `models`.
pub fn render(name: &str, models: &[ListedModel]) -> String {
    let mut page = format!(
        "# The `{name}` namespace\n\
         \n\
         The models in `{name}`, as the minting node's catalog listed them. Each definition is\n\
         kept in `{name}/` as two files named by its id: `.event`, the event that created it,\n\
         and `.state`, its state. A model-property link is the definition that gives a model\n\
         one of its properties and says whether it is optional. Nothing checks this page\n\
         against those files.\n"
    );
    for model in models {
        page.push_str(&format!(
            "\n## `{}`\n\nModel: `{}`\n\n\
             | Property | Backend | Value type | Optional | Property id | Model-property link id |\n\
             |---|---|---|---|---|---|\n",
            model.label, model.id
        ));
        for property in &model.properties {
            page.push_str(&format!(
                "| `{}` | {} | {} | {} | `{}` | `{}` |\n",
                property.name,
                property.backend,
                property.value_type,
                if property.optional { "yes" } else { "no" },
                property.id,
                property.link_id,
            ));
        }
    }
    page
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registering::ListedProperty;
    use ankurah::proto::EntityId;

    /// The whole page of a small namespace fixed by hand, to its last byte, so that the
    /// renderer can add no heading, row or line unnoticed.
    #[test]
    fn a_fixed_namespace_renders_exactly_its_page() {
        let id = |byte| EntityId::from_bytes([byte; 32]);
        let property = |property_byte, link_byte, name: &str, optional| ListedProperty {
            name: name.to_owned(),
            backend: "lww".to_owned(),
            value_type: "string".to_owned(),
            optional,
            id: id(property_byte),
            link_id: id(link_byte),
        };
        let models = [ListedModel {
            label: "example.thing".to_owned(),
            id: id(1),
            properties: vec![property(2, 3, "title", false), property(4, 5, "note", true)],
        }];
        let page = render("example", &models);
        assert_eq!(
            page,
            "# The `example` namespace\n\
             \n\
             The models in `example`, as the minting node's catalog listed them. Each definition is\n\
             kept in `example/` as two files named by its id: `.event`, the event that created it,\n\
             and `.state`, its state. A model-property link is the definition that gives a model\n\
             one of its properties and says whether it is optional. Nothing checks this page\n\
             against those files.\n\
             \n\
             ## `example.thing`\n\
             \n\
             Model: `AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE`\n\
             \n\
             | Property | Backend | Value type | Optional | Property id | Model-property link id |\n\
             |---|---|---|---|---|---|\n\
             | `title` | lww | string | no | `AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI` | `AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM` |\n\
             | `note` | lww | string | yes | `BAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQ` | `BQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQU` |\n"
        );
    }
}
