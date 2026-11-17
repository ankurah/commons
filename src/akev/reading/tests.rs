use super::*;
use crate::akev::{sample, BuildError, TimestampError};

/// The path the sample's own file has under `src/`.
fn path() -> &'static Path {
    Path::new("cognition/intention.akev")
}

fn read_one(text: &str) -> Result<Vec<Event>, ReadError> {
    read(&[(path(), text)]).map(|mut files| files.remove(0))
}

fn refused(text: &str) -> ReadError {
    read_one(text).unwrap_err()
}

fn ids(events: &[Event]) -> Vec<EntityId> {
    events.iter().map(|event| event.entity_id).collect()
}

#[test]
fn the_writers_layout_reads_as_the_events_the_headers_name() {
    let events = read_one(sample::TEXT).unwrap();
    assert_eq!(ids(&events), sample::ids());
    let geneses: Vec<Event> = sample::geneses().iter().map(|genesis| genesis.event().unwrap()).collect();
    assert_eq!(events, geneses);
}

/// Fields in another order, on one line or several, between commas or line breaks; the
/// values in another order; comments, an `id`, an id where the writer writes a name, and
/// a name with its own file.
#[test]
fn any_layout_the_rules_allow_reads_as_the_same_events() {
    let [model, title, link] = sample::ids();
    let text = format!(
        r#"
// The model.
[{model}] // its id
timestamp: 2026-11-16T02:14:23.846Z, member: system:model, nonce: I-QnwMppDz-TldUGtuitFV23BYGElMoId6A_2Xdqs2s
lww: {{ name: "Intention"
  label: "cognition.intention" }}


/* The property, a field to a line,
   with a comment among its values. */
[{title}]
nonce:9A4Pp2oT0O6ta-C2h3YWmfnX0KP5ZXTU9F8neXdS5fE
timestamp: 2026-11-16T02:14:23.846Z
id: {title}
lww: {{ name: "title", backend: "lww", /* its type */ value_type: "string", minted_for: {model}, target_model: none }}
member: system:property

[{link}]
member: system:model-property, nonce: gOYRvkWA5GYFz6xHvhJ8hw9kZEbkD0OGDCWXXgUkN1s, timestamp: 2026-11-16T02:14:23.846Z, lww: {{
    property: $title,
    model: cognition.intention@intention.akev
    optional: false }}
"#
    );
    assert_eq!(read_one(&text).unwrap(), read_one(sample::TEXT).unwrap());
}

/// The file that refers comes first, and names the other through a parent directory.
#[test]
fn a_name_with_a_file_refers_to_an_entity_of_that_file_by_a_path_from_the_one_that_refers() {
    let (model, rest) = sample::TEXT.split_once("\n\n").unwrap();
    let rest = rest.replace("$cognition.intention", "cognition.intention@../cognition/model.akev");
    let files = read(&[(Path::new("other/title.akev"), &rest), (Path::new("cognition/model.akev"), model)]).unwrap();
    let [model, title, link] = sample::ids();
    assert_eq!((ids(&files[0]), ids(&files[1])), (vec![title, link], vec![model]));
}

#[test]
fn a_second_genesis_of_an_entity_in_a_read_is_refused_naming_both_places() {
    let (model, _) = sample::TEXT.split_once("\n\n").unwrap();
    let again = format!("// Again.\n{model}");
    let error = read(&[(Path::new("first.akev"), model), (Path::new("again/second.akev"), &again)]).unwrap_err();
    assert_eq!((error.path.as_path(), error.line), (Path::new("again/second.akev"), 2));
    let first = |path: &Path| path == Path::new("first.akev");
    assert!(
        matches!(&error.problem, Problem::SecondGenesis { entity, first: path, first_line: 1 } if *entity == sample::ids()[0] && first(path)),
        "{error}"
    );
}

#[test]
fn a_block_without_a_nonce_or_a_timestamp_or_with_a_key_twice_is_refused_naming_the_line() {
    let no_nonce = sample::TEXT.replacen("nonce: I-QnwMppDz-TldUGtuitFV23BYGElMoId6A_2Xdqs2s, ", "", 1);
    assert!(matches!(refused(&no_nonce), ReadError { line: 1, problem: Problem::MissingNonce, .. }));
    let no_timestamp = sample::TEXT.replacen(", timestamp: 2026-11-16T02:14:23.846Z", "", 1);
    assert!(matches!(refused(&no_timestamp), ReadError { line: 1, problem: Problem::MissingTimestamp, .. }));
    let timestamp_twice =
        sample::TEXT.replacen("member: system:property", "member: system:property, timestamp: 2026-11-16T02:14:23.846Z", 1);
    assert!(matches!(refused(&timestamp_twice), ReadError { line: 14, problem: Problem::RepeatedKey("timestamp"), .. }));
    let lww_twice = sample::TEXT.replacen("member: system:model\n", "member: system:model\nlww: {}\n", 1);
    assert!(matches!(refused(&lww_twice), ReadError { line: 4, problem: Problem::RepeatedBackend(backend), .. } if backend == "lww"));
}

/// Two memberships are added in the order written, so that the event is another.
#[test]
fn member_may_repeat_and_its_order_is_the_events() {
    let both = sample::TEXT.replacen("member: system:model\n", "member: system:model, member: system:system\n", 1);
    let error = refused(&both);
    let built = match error.problem {
        Problem::NotTheHeadersEntity { built, .. } => built,
        _ => panic!("{error}"),
    };
    let mut genesis = sample::geneses().remove(0);
    genesis.operations.memberships.push(ModelId::System(ankurah::proto::SystemModel::System));
    assert_eq!(built, genesis.event().unwrap().entity_id);
}

#[test]
fn a_block_with_a_parent_is_refused_as_an_update() {
    let update = sample::TEXT.replacen("member: system:model\n", &format!("member: system:model, parent: {}\n", sample::ids()[1]), 1);
    assert!(matches!(refused(&update), ReadError { line: 3, problem: Problem::Update, .. }));
}

#[test]
fn an_unknown_backend_or_text_for_a_backend_without_a_text_form_is_refused_naming_its_line() {
    let unknown = sample::TEXT.replacen("lww: { label", "crdt: { label", 1);
    let error = refused(&unknown);
    assert!(
        matches!(&error, ReadError { line: 3, problem: Problem::Build(BuildError::UnknownBackend(backend)), .. } if backend == "crdt"),
        "{error}"
    );
    let yrs = sample::TEXT.replacen("lww: {\n    name", "yrs: {\n    name", 1);
    let error = refused(&yrs);
    assert!(
        matches!(&error, ReadError { line: 7, problem: Problem::Build(BuildError::NoTextForm(backend)), .. } if backend == "yrs"),
        "{error}"
    );
}

#[test]
fn a_block_whose_event_is_another_entity_than_its_header_names_is_refused_stating_both() {
    let error = refused(&sample::TEXT.replacen("\"Intention\"", "\"Intent\"", 1));
    assert!(
        matches!(error, ReadError { line: 1, problem: Problem::NotTheHeadersEntity { header, built }, .. } if header == sample::ids()[0] && built != header)
    );
}

#[test]
fn an_id_other_than_the_header_is_refused() {
    let [model, title, _] = sample::ids();
    let error = refused(&sample::TEXT.replacen("846Z\nlww", &format!("846Z\nid: {title}\nlww"), 1));
    assert!(
        matches!(error, ReadError { line: 3, problem: Problem::IdNotTheHeader { header, stated }, .. } if header == model && stated == title)
    );
}

#[test]
fn a_name_that_no_entity_or_several_answer_to_or_of_a_file_not_read_is_refused_naming_the_line() {
    let error = refused(&sample::TEXT.replacen("$title", "$subject", 1));
    assert!(matches!(&error, ReadError { line: 20, problem: Problem::UnknownName(name), .. } if name == "$subject"), "{error}");

    let another = "\n[AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA]\nnonce: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA, timestamp: 1970-01-01T00:00:00.000Z\nlww: { name: \"title\" }\n";
    let error = refused(&format!("{}{another}", sample::TEXT));
    assert!(
        matches!(&error, ReadError { line: 20, problem: Problem::AmbiguousName { name, entities: 2 }, .. } if name == "$title"),
        "{error}"
    );

    let error = refused(&sample::TEXT.replacen("$title", "title@missing.akev", 1));
    let missing = Path::new("cognition/missing.akev");
    assert!(matches!(&error, ReadError { line: 20, problem: Problem::UnknownFile(path), .. } if path == missing), "{error}");
}

#[test]
fn an_entity_that_needs_its_own_event_is_refused_as_a_cycle() {
    let text = "[AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA]\nnonce: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA, timestamp: 1970-01-01T00:00:00.000Z\nlww: {\n    label: \"loop\"\n    minted_for: $loop\n}\n";
    assert!(matches!(refused(text), ReadError { line: 5, problem: Problem::ReferenceCycle, .. }));
}

/// The source of a timestamp's error or of a value's is the reader's, so that a caller can
/// downcast it.
#[test]
fn a_problem_that_wraps_an_error_gives_it_as_its_source() {
    use ankurah::proto::DecodeError;
    use std::error::Error;
    let error = refused(&sample::TEXT.replacen("23.846Z", "23.85Z", 1));
    assert!(matches!(error.source().and_then(|source| source.downcast_ref::<TimestampError>()), Some(TimestampError::Form)), "{error}");
    let short = "A".repeat(42);
    let error = refused(&sample::TEXT.replacen("\"Intention\"", &short, 1));
    assert!(matches!(&error.problem, Problem::Value { word, .. } if *word == short), "{error}");
    assert_eq!(error.line, 3);
    assert!(matches!(error.source().and_then(|source| source.downcast_ref::<DecodeError>()), Some(DecodeError::InvalidLength)), "{error}");
}
