use std::sync::OnceLock;

use super::*;
use crate::{
    namespaces::NAMESPACES,
    registering::{self, MintedNamespace},
    writing,
};
use ankurah::{
    proto::{Clock, Identity, Operation, OperationSet, Subject},
    NodeKey,
};
use tempfile::TempDir;

/// Every namespace, minted once for all the tests, because minting takes a second or two.
fn minted() -> &'static [MintedNamespace] {
    static MINTED: OnceLock<Vec<MintedNamespace>> = OnceLock::new();
    MINTED.get_or_init(|| {
        let runtime = tokio::runtime::Runtime::new().expect("a runtime to mint in");
        runtime.block_on(registering::mint(NAMESPACES)).expect("the namespaces mint")
    })
}

/// `minted` written, as `mint` writes it, into a new directory inside the returned
/// temporary one.
fn written() -> (TempDir, PathBuf) {
    let parent = tempfile::tempdir().unwrap();
    let out = parent.path().join("minted");
    writing::write(&out, minted()).unwrap();
    (parent, out)
}

/// The creation of an entity that is a member of `models`, under a minter of no account.
fn creation(models: &[ModelId]) -> Event {
    Event::genesis(
        Identity::Ed25519([6; 32]),
        OperationSet(models.iter().map(|model| Operation::Membership(Membership::Add(*model))).collect()),
    )
}

/// The event files of the directory `name`, sorted by name.
fn event_files(out: &Path, name: &str) -> Vec<PathBuf> {
    let paths = fs::read_dir(out.join(name)).unwrap().map(|entry| entry.unwrap().path());
    let mut files: Vec<PathBuf> = paths.filter(|path| path.extension().is_some_and(|extension| extension == EVENT_EXTENSION)).collect();
    files.sort();
    files
}

/// The state file beside the event file `event`.
fn state_file(event: &Path) -> PathBuf {
    event.with_extension(STATE_EXTENSION)
}

/// The id of the definition whose event file is `event`.
fn id_of(event: &Path) -> EntityId {
    EntityId::from_base64(event.file_stem().unwrap().to_str().unwrap()).unwrap()
}

fn read_event(path: &Path) -> Attested<Event> {
    Attested::decode(&fs::read(path).unwrap()).unwrap()
}

/// Writes `creation`, with no attestation, into the directory `name` as a definition's
/// event file, and returns the file's path.
fn place(out: &Path, name: &str, creation: Event) -> PathBuf {
    let path = out.join(name).join(format!("{}.{EVENT_EXTENSION}", creation.entity_id.to_base64()));
    fs::write(&path, Attested::from(creation).encode()).unwrap();
    path
}

fn failure(out: &Path) -> VerifyError {
    verify(out).unwrap_err()
}

/// Makes a FIFO at `path`, which opening for reading would wait on until a writer came.
fn fifo(path: &Path) {
    let path = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
    // SAFETY: `path` is a NUL-terminated string that outlives the call.
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o644) }, 0, "{}", io::Error::last_os_error());
}

#[test]
fn a_minted_directory_verifies() {
    let (_parent, out) = written();
    let verified = verify(&out).unwrap();
    let definitions: usize =
        NAMESPACES.iter().flat_map(|namespace| namespace.models).map(|model| 1 + 2 * (model.descriptor)().properties.len()).sum();
    assert_eq!((verified.directories, verified.definitions), (NAMESPACES.len(), definitions));
}

#[test]
fn flipping_a_byte_of_an_event_file_fails_verification_naming_that_file() {
    let (_parent, out) = written();
    let file = event_files(&out, "identity").remove(0);
    let mut bytes = fs::read(&file).unwrap();
    let middle = bytes.len() / 2;
    bytes[middle] ^= 0xff;
    fs::write(&file, &bytes).unwrap();
    assert_eq!(failure(&out).path, file);
}

/// Every byte of an event file is checked: flipping any one of them fails the file.
#[test]
fn flipping_any_byte_of_an_event_file_fails_that_file() {
    let (_parent, out) = written();
    let file = event_files(&out, "identity").remove(0);
    let original = fs::read(&file).unwrap();
    for index in 0..original.len() {
        let mut bytes = original.clone();
        bytes[index] ^= 0xff;
        fs::write(&file, &bytes).unwrap();
        assert!(check_event_file(&file, id_of(&file)).is_err(), "flipping byte {index} of {} passed", original.len());
    }
}

/// A file such as the `.DS_Store` a file browser leaves behind fails verification,
/// named, whether it lies beside the pages or among a directory's definition files.
#[test]
fn a_stray_file_fails_verification_naming_it() {
    let (_parent, out) = written();
    let beside_the_pages = out.join(".DS_Store");
    fs::write(&beside_the_pages, b"").unwrap();
    let error = failure(&out);
    assert_eq!(error.path, beside_the_pages);
    assert!(matches!(error.failure, Failure::Stray(EntryType::File)));
    fs::remove_file(&beside_the_pages).unwrap();

    let among_the_events = out.join("git").join(".DS_Store");
    fs::write(&among_the_events, b"").unwrap();
    let error = failure(&out);
    assert_eq!(error.path, among_the_events);
    assert!(matches!(error.failure, Failure::NotADefinitionFile));
}

/// The tool checks no statement, so a file carrying one fails rather than passes it,
/// even a statement properly sealed about that event.
#[test]
fn an_event_file_carrying_an_attestation_fails_verification() {
    let (_parent, out) = written();
    let file = event_files(&out, "identity").remove(0);
    let mut attested = read_event(&file);
    assert!(attested.attestations.is_empty(), "mint writes no attestation");
    let statement = NodeKey::generate().attest(Subject::event(&attested.payload), EntityId::from_bytes([1; 32]), Vec::new());
    attested.attestations.push(statement);
    fs::write(&file, attested.encode()).unwrap();
    let error = failure(&out);
    assert_eq!(error.path, file);
    assert!(matches!(error.failure, Failure::Attested(1)));
}

#[test]
fn an_update_in_place_of_a_creation_fails_verification() {
    let (_parent, out) = written();
    let file = event_files(&out, "identity").remove(0);
    let creation = read_event(&file).payload;
    let update = Event::update(creation.entity_id, Clock::singleton(&creation), OperationSet::default());
    fs::write(&file, Attested::from(update).encode()).unwrap();
    let error = failure(&out);
    assert_eq!(error.path, file);
    assert!(matches!(error.failure, Failure::NotCreation));
}

/// A creation is no definition unless it makes its entity a member of exactly one of the
/// catalog's models.
#[test]
fn a_creation_that_is_no_definition_fails_verification() {
    let (_parent, out) = written();
    let other_model = ModelId::EntityId(EntityId::from_bytes([7; 32]));
    let catalog = ModelId::System(SystemModel::Property);
    for models in [vec![], vec![other_model], vec![catalog, ModelId::System(SystemModel::Model)]] {
        let path = place(&out, "identity", creation(&models));
        let error = failure(&out);
        assert_eq!(error.path, path);
        assert!(matches!(&error.failure, Failure::NotADefinition(found) if *found == models), "{error}");
        fs::remove_file(path).unwrap();
    }
}

/// Directories are read by name, so the copy placed in `git` is the first place and the
/// original in `records` the second.
#[test]
fn the_same_definition_twice_fails_verification_naming_the_second() {
    let (_parent, out) = written();
    let original = event_files(&out, "records").remove(0);
    let copy = out.join("git").join(original.file_name().unwrap());
    fs::copy(&original, &copy).unwrap();
    fs::copy(state_file(&original), state_file(&copy)).unwrap();
    let error = failure(&out);
    assert_eq!(error.path, original);
    assert!(matches!(error.failure, Failure::InTwoPlaces { first } if first == copy));
}

/// Every entry `verify` reads must be what its role needs, read without following a
/// symlink: a symlink to valid content outside the directory is refused, and so is a
/// FIFO, before anything opens it.
#[test]
fn an_entry_that_is_not_what_its_role_needs_fails_verification_before_it_is_opened() {
    let (parent, out) = written();
    let wrong = |path: &Path, expected, found| {
        let error = failure(&out);
        assert_eq!(error.path, path);
        assert!(matches!(error.failure, Failure::WrongType { expected: e, found: f } if e == expected && f == found), "{error}");
    };

    let directory = out.join("git");
    let elsewhere = parent.path().join("git elsewhere");
    fs::rename(&directory, &elsewhere).unwrap();
    std::os::unix::fs::symlink(&elsewhere, &directory).unwrap();
    let error = failure(&out);
    assert_eq!(error.path, directory);
    assert!(matches!(error.failure, Failure::Stray(EntryType::Symlink)), "{error}");
    fs::remove_file(&directory).unwrap();
    fs::rename(&elsewhere, &directory).unwrap();

    let event = event_files(&out, "git").remove(0);
    let copy = parent.path().join("event elsewhere");
    fs::rename(&event, &copy).unwrap();
    fifo(&event);
    wrong(&event, EntryType::File, EntryType::Special);
    fs::remove_file(&event).unwrap();
    std::os::unix::fs::symlink(&copy, &event).unwrap();
    wrong(&event, EntryType::File, EntryType::Symlink);
    fs::remove_file(&event).unwrap();
    fs::rename(&copy, &event).unwrap();
}

/// A state file must be there, decode with nothing left over, and be the state of its
/// event's entity with that creation event alone as its head.
#[test]
fn a_state_that_does_not_belong_to_its_event_fails_verification() {
    let (_parent, out) = written();
    let [event, other_event] = [0, 1].map(|index| event_files(&out, "identity").remove(index));
    let state = state_file(&event);
    let original = fs::read(&state).unwrap();
    let fails_at = |path: &Path| {
        let error = failure(&out);
        assert_eq!(error.path, path);
        error.failure
    };

    fs::remove_file(&state).unwrap();
    assert!(matches!(fails_at(&event), Failure::NoState));

    fs::copy(state_file(&other_event), &state).unwrap();
    assert!(matches!(fails_at(&state), Failure::StateOfOtherEntity(id) if id == id_of(&other_event)));

    let mut headless = state_encoding::decode(&original).unwrap();
    headless.state.head = Clock::default();
    fs::write(&state, state_encoding::encode(&headless)).unwrap();
    assert!(matches!(fails_at(&state), Failure::HeadNotCreation));

    fs::write(&state, [original.as_slice(), &[0]].concat()).unwrap();
    assert!(matches!(fails_at(&state), Failure::UndecodableState(_)));

    fs::write(&state, &original).unwrap();
    fs::remove_file(&event).unwrap();
    assert!(matches!(fails_at(&state), Failure::NoEvent));
}
