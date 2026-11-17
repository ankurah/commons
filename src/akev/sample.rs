//! The first three events of `cognition/intention.akev`, the model and its `title`, as the
//! writer lays them out and as the geneses they state, for the tests of reading and writing.

use std::collections::BTreeMap;

use ankurah::{
    proto::{ModelId, SystemModel, SystemProperty},
    value::Value,
    EntityId, PropertyId,
};

use super::{BackendOperations, Genesis, Operations};

pub const TEXT: &str = r#"[W_PUyePLDVZm5ly-2sTyWjIHyu4zihZ5QWcCUGG-mHY]
nonce: I-QnwMppDz-TldUGtuitFV23BYGElMoId6A_2Xdqs2s, timestamp: 2026-11-16T02:14:23.846Z
lww: { label: "cognition.intention", name: "Intention" }, member: system:model

[BPKD7qvFa6kENqdAWQc62bzM_3lnAnaLbpEUYIxJDVA]
nonce: 9A4Pp2oT0O6ta-C2h3YWmfnX0KP5ZXTU9F8neXdS5fE, timestamp: 2026-11-16T02:14:23.846Z
lww: {
    name: "title"
    backend: "lww"
    value_type: "string"
    minted_for: $cognition.intention
    target_model: none
}
member: system:property

[B5u43ChznTfjWsFKshOel3IjvettoK35Zu_qwcE4Bck]
nonce: gOYRvkWA5GYFz6xHvhJ8hw9kZEbkD0OGDCWXXgUkN1s, timestamp: 2026-11-16T02:14:23.846Z
lww: {
    model: $cognition.intention
    property: $title
    optional: false
}
member: system:model-property
"#;

/// Each event's id, in the order of the text.
pub fn ids() -> [EntityId; 3] {
    [
        "W_PUyePLDVZm5ly-2sTyWjIHyu4zihZ5QWcCUGG-mHY",
        "BPKD7qvFa6kENqdAWQc62bzM_3lnAnaLbpEUYIxJDVA",
        "B5u43ChznTfjWsFKshOel3IjvettoK35Zu_qwcE4Bck",
    ]
    .map(|id| EntityId::from_base64(id).unwrap())
}

pub fn geneses() -> Vec<Genesis> {
    let [model, title, _] = ids();
    let genesis = |nonce: &str, values: Vec<(SystemProperty, Option<Value>)>, member| Genesis {
        nonce: EntityId::from_base64(nonce).unwrap().to_bytes(),
        timestamp: 1_794_795_263_846,
        operations: Operations {
            backends: BTreeMap::from([(
                "lww".to_owned(),
                BackendOperations::Values(values.into_iter().map(|(property, value)| (PropertyId::System(property), value)).collect()),
            )]),
            memberships: vec![ModelId::System(member)],
        },
    };
    let text = |text: &str| Some(Value::String(text.to_owned()));
    vec![
        genesis(
            "I-QnwMppDz-TldUGtuitFV23BYGElMoId6A_2Xdqs2s",
            vec![(SystemProperty::Label, text("cognition.intention")), (SystemProperty::Name, text("Intention"))],
            SystemModel::Model,
        ),
        genesis(
            "9A4Pp2oT0O6ta-C2h3YWmfnX0KP5ZXTU9F8neXdS5fE",
            vec![
                (SystemProperty::Name, text("title")),
                (SystemProperty::Backend, text("lww")),
                (SystemProperty::ValueType, text("string")),
                (SystemProperty::MintedFor, Some(Value::EntityId(model))),
                (SystemProperty::TargetModel, None),
            ],
            SystemModel::Property,
        ),
        genesis(
            "gOYRvkWA5GYFz6xHvhJ8hw9kZEbkD0OGDCWXXgUkN1s",
            vec![
                (SystemProperty::Model, Some(Value::EntityId(model))),
                (SystemProperty::Property, Some(Value::EntityId(title))),
                (SystemProperty::Optional, Some(Value::Bool(false))),
            ],
            SystemModel::ModelProperty,
        ),
    ]
}
