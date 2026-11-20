use super::*;
use crate::copies::{copied, edit_and_restate, files, mint, sources};
use ankurah::core::schema::{ModelStructDescriptor, PerSystemOnceCell, StructProperty};
use ankurah_commons::{
    akev::{self, Problem},
    namespaces::NAMESPACES,
    registration::Definition,
};
use std::{fs, sync::OnceLock};
use tempfile::TempDir;

/// A copy of the sources with the kept mint minted again.
fn minted() -> TempDir {
    let src = copied();
    mint(src.path());
    src
}

fn failure(src: &TempDir) -> VerifyError {
    verify(src.path(), NAMESPACES).unwrap_err()
}

/// Each entity keeps its nonce and timestamp, so minting the kept mint again under the
/// same declarations and the same encoding of Ankurah's events writes every file as it
/// was, the ids the declarations state with them.
#[test]
fn minting_the_kept_mint_again_changes_no_file_and_it_verifies() {
    let src = minted();
    assert_eq!(files(src.path()), files(&sources()));
    let verified = verify(src.path(), NAMESPACES).unwrap();
    let models = NAMESPACES.iter().map(|namespace| namespace.models.len()).sum::<usize>();
    let definitions: usize = NAMESPACES.iter().flat_map(|namespace| namespace.models).map(|model| 1 + 2 * model().properties.len()).sum();
    let edge_kinds: usize = NAMESPACES.iter().flat_map(|namespace| &namespace.edge_kinds).map(|edge_kinds| edge_kinds.kinds.len()).sum();
    assert_eq!((verified.files, verified.events), (models, definitions + edge_kinds));
}

/// A copy of the intention's struct and its kept mint, alone, for a namespace that lists only
/// the intention.
fn the_intention_alone() -> TempDir {
    let src = tempfile::tempdir().unwrap();
    let cognition = src.path().join("cognition");
    fs::create_dir(&cognition).unwrap();
    for file in ["intention.rs", "intention.akev"] {
        fs::copy(sources().join("cognition").join(file), cognition.join(file)).unwrap();
    }
    src
}

/// The id the kept mint gives the intention's property `title`, which its struct declares.
fn kept_title() -> &'static str {
    let intention = <ankurah_commons::cognition::Intention as ankurah::Model>::descriptor();
    intention.properties.iter().find(|property| property.field == "title").and_then(|property| property.explicit_id).unwrap()
}

/// `cognition.intention` as its struct declares it, but for its ids: another model id, and
/// none for its properties.
#[derive(ankurah::Model, Debug, Clone)]
#[model(label = "cognition.intention")]
#[model(id = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE")]
#[model(no_ffi)]
pub struct Intention {
    #[active_type(LWW)]
    pub title: String,
    #[active_type(LWW)]
    pub description: String,
}

/// After a mint that changed a model's id, its struct declares the old id until the tool is
/// built again; the file's id is the one that holds.
#[test]
fn a_struct_that_declares_another_id_than_its_file_states_fails_naming_the_file() {
    static NAMESPACES: &[Namespace] =
        &[Namespace { name: "cognition", models: &[<Intention as ankurah::Model>::descriptor], edge_kinds: None }];
    let src = the_intention_alone();
    let error = verify(src.path(), NAMESPACES).unwrap_err();
    assert_eq!(error.path, src.path().join("cognition").join("intention.akev"));
    let kept = EntityId::from_base64(<ankurah_commons::cognition::Intention as ankurah::Model>::descriptor().explicit_id.unwrap()).unwrap();
    assert!(
        matches!(error.failure, Failure::OtherId { stated, declared: Some("AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE") } if stated == kept),
        "{error}"
    );
}

/// The intention's declaration as the derive describes it, but for its field `title`, which
/// declares no id, as a field does until the mint gives it one.
fn intention_whose_title_declares_no_id() -> &'static ModelStructDescriptor {
    static DESCRIPTOR: OnceLock<ModelStructDescriptor> = OnceLock::new();
    DESCRIPTOR.get_or_init(|| {
        let intention = <ankurah_commons::cognition::Intention as ankurah::Model>::descriptor();
        let properties: Vec<StructProperty> = intention
            .properties
            .iter()
            .map(|property| StructProperty {
                explicit_id: property.explicit_id.filter(|_| property.field != "title"),
                resolved: PerSystemOnceCell::per_epoch(),
                ..*property
            })
            .collect();
        ModelStructDescriptor { properties: properties.leak(), resolved: PerSystemOnceCell::per_epoch(), ..*intention }
    })
}

/// A field declares no property id until the mint gives it one, as none did before the tool
/// gave properties their ids.
#[test]
fn a_field_that_declares_no_property_id_fails_naming_the_file_the_struct_and_the_field() {
    static WITHOUT_THE_ID: &[Namespace] =
        &[Namespace { name: "cognition", models: &[intention_whose_title_declares_no_id], edge_kinds: None }];
    let src = the_intention_alone();
    let error = verify(src.path(), WITHOUT_THE_ID).unwrap_err();
    assert_eq!(error.path, src.path().join("cognition").join("intention.akev"));
    let kept = EntityId::from_base64(kept_title()).unwrap();
    assert!(
        matches!(error.failure, Failure::NoPropertyId { struct_name: "Intention", field: "title", stated } if stated == kept),
        "{error}"
    );
}

/// After a mint that changed a property's id, its field declares the old id until the tool is
/// built again; the file's id is the one that holds.
#[test]
fn a_field_that_declares_another_property_id_than_its_file_states_fails_naming_the_file_the_struct_and_the_field() {
    let src = minted();
    let intention = src.path().join("cognition").join("intention.akev");
    // The definition of `title` drawn anew: another nonce, and so another id.
    let text = fs::read_to_string(&intention).unwrap();
    let start = text[..text.find("name: \"title\"").unwrap()].rfind("nonce: ").unwrap() + "nonce: ".len();
    let kept_nonce = &text[start..start + text[start..].find(',').unwrap()];
    let drawn = [2; 32];
    edit_and_restate(&intention, kept_nonce, &EntityId::from_bytes(drawn).to_base64());
    let text = fs::read_to_string(&intention).unwrap();
    let redrawn = akev::read(&[(&intention, &text)]).unwrap().remove(0).into_iter().find(|event| *event.nonce() == drawn).unwrap();
    let error = failure(&src);
    assert_eq!(error.path, intention);
    assert!(
        matches!(error.failure, Failure::OtherPropertyId { struct_name: "Intention", field: "title", stated, declared }
            if stated == redrawn.entity_id && declared == kept_title()),
        "{error}"
    );
}

/// A nonce changed by a character reads as another event than its header names.
#[test]
fn a_changed_nonce_fails_the_read_naming_the_file_and_the_line() {
    let src = minted();
    let file = src.path().join("cognition").join("intention.akev");
    let text = fs::read_to_string(&file).unwrap();
    let digit = text.find("nonce: ").unwrap() + "nonce: ".len();
    let changed = if &text[digit..digit + 1] == "A" { "B" } else { "A" };
    fs::write(&file, format!("{}{changed}{}", &text[..digit], &text[digit + 1..])).unwrap();
    let error = failure(&src);
    assert_eq!(error.path, file);
    let read = match error.failure {
        Failure::Unread(read) => read,
        failure => panic!("{failure}"),
    };
    assert!(matches!(*read, ReadError { line: 1, problem: Problem::NotTheHeadersEntity { .. }, .. }), "{read}");
}

/// A file must state what the sources declare, so that a mint left behind by a change to a
/// struct or to the table of edge kinds fails, though every file reads.
#[test]
fn a_file_that_states_other_events_than_the_sources_need_fails_naming_it() {
    let src = minted();
    let intention = src.path().join("cognition").join("intention.akev");
    let kept = fs::read_to_string(&intention).unwrap();
    edit_and_restate(&intention, "optional: false", "optional: true");
    let error = failure(&src);
    assert_eq!(error.path, intention);
    assert!(matches!(error.failure, Failure::NotAsDeclared(Minted::Definition(Definition::Link("title")))), "{error}");
    fs::write(&intention, kept).unwrap();

    let edge_kind = src.path().join("graph").join("edge_kind.akev");
    edit_and_restate(&edge_kind, "Many to many", "Many to one");
    let error = failure(&src);
    assert_eq!(error.path, edge_kind);
    assert!(matches!(error.failure, Failure::NotAsDeclared(Minted::EdgeKind("serves"))), "{error}");
}

/// A genesis that no declaration needs, beside every one that the declarations do.
#[test]
fn a_kept_event_no_declaration_needs_fails_naming_its_file_and_id() {
    let src = minted();
    let intention = src.path().join("cognition").join("intention.akev");
    let spare = format!(
        "member: system:model\n\n[{}]\nnonce: {}, timestamp: 1970-01-01T00:00:00.000Z\nlww: {{ name: \"spare\" }}\n",
        EntityId::from_bytes([0; 32]),
        EntityId::from_bytes([1; 32])
    );
    edit_and_restate(&intention, "member: system:model\n", &spare);
    let text = fs::read_to_string(&intention).unwrap();
    let spare = akev::read(&[(&intention, &text)]).unwrap().remove(0)[1].entity_id;
    let error = failure(&src);
    assert_eq!(error.path, intention);
    assert!(matches!(error.failure, Failure::Unneeded(id) if id == spare), "{error}");
}

#[test]
fn a_missing_or_stray_file_fails_naming_it() {
    let src = minted();
    let intention = src.path().join("cognition").join("intention.akev");
    let kept = fs::read_to_string(&intention).unwrap();
    fs::remove_file(&intention).unwrap();
    let error = failure(&src);
    assert_eq!(error.path, intention);
    assert!(matches!(error.failure, Failure::Missing), "{error}");
    fs::write(&intention, kept).unwrap();

    let stray = src.path().join("cognition").join("hunch.akev");
    fs::write(&stray, "").unwrap();
    let error = failure(&src);
    assert_eq!(error.path, stray);
    assert!(matches!(error.failure, Failure::Stray), "{error}");
}
