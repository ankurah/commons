use super::*;
use crate::copies::{copied, edit, files, mint, sources};
use ankurah_commons::namespaces::NAMESPACES;
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

/// Each entity keeps its time and nonce, so minting the kept mint again under the same
/// declarations and the same format of Ankurah's events writes every file as it was, the
/// ids the declarations state with them.
#[test]
fn minting_the_kept_mint_again_changes_no_file_and_it_verifies() {
    let src = minted();
    assert_eq!(files(src.path()), files(&sources()));
    let verified = verify(src.path(), NAMESPACES).unwrap();
    let models = NAMESPACES.iter().map(|namespace| namespace.models.len()).sum::<usize>();
    let definitions: usize = NAMESPACES.iter().flat_map(|namespace| namespace.models).map(|model| 1 + 2 * model().properties.len()).sum();
    let edge_kinds: usize = NAMESPACES.iter().flat_map(|namespace| &namespace.edge_kinds).map(|edge_kinds| edge_kinds.kinds.len()).sum();
    assert_eq!((verified.files, verified.events), (models + 1, definitions + edge_kinds));
}

/// A file whose model line gives an id other than the one the struct declares fails, naming
/// the file, before its events are rebuilt.
#[test]
fn a_model_line_whose_id_the_struct_does_not_declare_fails_naming_the_file() {
    let src = minted();
    let intention = src.path().join("cognition").join("intention.akev");
    let text = fs::read_to_string(&intention).unwrap();
    let (_, ids) = ModelDefinitions::parse(&text).unwrap();
    edit(&intention, &ids[0].to_base64(), &EntityId::from_bytes([1; 32]).to_base64());
    let error = failure(&src);
    assert_eq!(error.path, intention);
    assert!(matches!(error.failure, Failure::NotAsDeclared("id")), "{error}");
}

/// A creation field changed in the writer's form still parses, but rebuilds another event.
#[test]
fn a_changed_creation_field_fails_the_rebuild_naming_the_file() {
    let src = minted();
    let file = src.path().join("cognition").join("intention.akev");
    let text = fs::read_to_string(&file).unwrap();
    let digit = text.find(" nonce ").unwrap() + " nonce ".len();
    let changed = if &text[digit..digit + 1] == "0" { "1" } else { "0" };
    fs::write(&file, format!("{}{changed}{}", &text[..digit], &text[digit + 1..])).unwrap();
    let error = failure(&src);
    assert_eq!(error.path, file);
    assert!(matches!(error.failure, Failure::NotRebuilt(RebuildError::NotTheMintedEvent { .. })), "{error}");
}

/// A file must state what the sources declare, so that a mint left behind by a change to a
/// struct or to the table of edge kinds fails.
#[test]
fn a_file_that_states_other_than_the_sources_fails_naming_it() {
    let src = minted();
    let intention = src.path().join("cognition").join("intention.akev");
    edit(&intention, "required", "optional");
    let error = failure(&src);
    assert_eq!(error.path, intention);
    assert!(matches!(error.failure, Failure::NotAsDeclared("optionality")), "{error}");
    edit(&intention, "optional", "required");

    let edge_kinds = src.path().join("graph").join(EDGE_KINDS_FILE);
    edit(&edge_kinds, "Many to many", "Many to one");
    let error = failure(&src);
    assert_eq!(error.path, edge_kinds);
    assert!(matches!(error.failure, Failure::NotAsListed), "{error}");
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
