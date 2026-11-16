use super::*;
use crate::{minting, namespaces::NAMESPACES};
use tempfile::TempDir;

fn sources() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("src")
}

/// A copy of the crate's Rust sources, with the kept mint's files.
fn copied() -> TempDir {
    let copy = tempfile::tempdir().unwrap();
    copy_sources(&sources(), copy.path());
    copy
}

fn copy_sources(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            fs::create_dir(&target).unwrap();
            copy_sources(&path, &target);
        } else if path.extension().is_some_and(|extension| extension == "rs" || extension == writing::EXTENSION) {
            fs::copy(&path, &target).unwrap();
        }
    }
}

/// Mints into `src` as `mint` does.
fn mint(src: &Path) {
    writing::write(src, &minting::mint(src, NAMESPACES).unwrap()).unwrap();
}

/// A copy of the sources with the kept mint minted again.
fn minted() -> TempDir {
    let src = copied();
    mint(src.path());
    src
}

fn failure(src: &TempDir) -> VerifyError {
    verify(src.path(), NAMESPACES).unwrap_err()
}

/// Edits the first occurrence of `from` in the file at `path`.
fn edit(path: &Path, from: &str, to: &str) {
    let text = fs::read_to_string(path).unwrap();
    assert!(text.contains(from), "{} holds no `{from}`", path.display());
    fs::write(path, text.replacen(from, to, 1)).unwrap();
}

/// Each entity keeps its time and nonce, so minting the kept mint again under the same
/// declarations and the same format of Ankurah's events writes every file as it was.
#[test]
fn minting_the_kept_mint_again_changes_no_file_and_it_verifies() {
    let src = minted();
    for kept in writing::minted_files(&sources()).unwrap() {
        let minted = src.path().join(kept.strip_prefix(sources()).unwrap());
        assert_eq!(fs::read_to_string(&minted).unwrap(), fs::read_to_string(&kept).unwrap(), "{}", kept.display());
    }
    let verified = verify(src.path(), NAMESPACES).unwrap();
    let models = NAMESPACES.iter().map(|namespace| namespace.models.len()).sum::<usize>();
    let definitions: usize =
        NAMESPACES.iter().flat_map(|namespace| namespace.models).map(|model| 1 + 2 * (model.descriptor)().properties.len()).sum();
    let edge_kinds: usize = NAMESPACES.iter().map(|namespace| namespace.edge_kinds.len()).sum();
    assert_eq!((verified.files, verified.events), (models + 1, definitions + edge_kinds));
}

/// An entity the kept mint lacks is minted with a time and nonce drawn for it.
#[test]
fn a_model_the_kept_mint_lacks_is_minted_anew() {
    let src = copied();
    let intention = src.path().join("cognition").join("intention.akev");
    let kept = fs::read_to_string(&intention).unwrap();
    fs::remove_file(&intention).unwrap();
    mint(src.path());
    assert_ne!(fs::read_to_string(&intention).unwrap(), kept);
    verify(src.path(), NAMESPACES).unwrap();
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
