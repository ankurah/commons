# ankurah-commons-mint

Mints the definitions of the Commons' models and the Commons' kinds of edge, and checks
a mint. `mint` registers each model, creates an `EdgeKind` record for each kind, and
writes every entity it minted as a line of an `.akev` file, text from which the entity's
creation event is rebuilt: each model's definitions beside the struct that declares it,
and the edge kinds' records in `src/graph/edge_kinds.akev`. `verify` checks those files.
Whether a mint is kept is the runner's choice. The library in this package,
`rebuilding`, rebuilds the events from the files.

The tool uses parts of Ankurah that exist only on its node identity branch
(`feat/node-identity-attestation`), so it is outside the workspace while the crate builds
against Ankurah's main. It builds and its tests pass in a copy of the workspace
that pins that branch at `a164293e` and lists `mint` among its members, as commit
`a545fe9` of this repository did. It returns to the workspace when the branch reaches
Ankurah's main. The events it mints are in that branch's format, which Ankurah's main
does not read.

A kept mint is for seeding: another node, on a fresh store, is given the minted entities
before it is built, and then uses the very same ids. Before building the node, the user
opens its storage engine and, in one storage transaction, calls `add_events` for every
rebuilt event and `set_state(&Clock::default(), &state)` for every entity's state, then
commits. The node then finds the definitions in its catalog, which is built from states,
and registering models that declare exactly what was minted binds them to the minted
ids. The states are not rebuilt yet: Ankurah offers no public way to apply an event to an
empty entity.

A seeder has three things to get right, because Ankurah reports none of them as an
error:

- **The commit's outcome.** On a store that already holds the definitions, `commit`
  returns `StorageCommitOutcome::Conflict` and writes nothing. A second start looks
  like that, and so does a larger mint over a smaller one: a seeded store cannot be
  given more this way.
- **Completeness.** Whatever the node's models declare that the seed lacks, a model, a
  property or a model-property link, registration mints under the node's own key, with
  other ids.
- **Timing.** Definitions planted under a running node are not seen by it, and it mints
  its own beside them.

This tool does not seed.

Nothing is signed. Every creation event names a minter, a public key, because Ankurah
hashes one into the event's id. `mint` draws a fresh key for each run and discards it,
so the minter is nobody's identity. `verify` shows that the files rebuild the events
their ids name; it does not show who made them. What vouches for a kept mint is the
repository it is committed to. Statements about the events can be added later without
changing any id.

## Running it

    cargo run -p ankurah-commons-mint -- mint [--src <dir>]
    cargo run -p ankurah-commons-mint -- verify [--src <dir>]

Both work on the namespaces' parent directory, the crate's `src/` by default. `mint`
refuses a directory that holds an `.akev` file already, so that it never replaces a kept
mint; a trial run is given a copy of `src/` with `--src`. A mint that is to be kept is
written into `src/` and committed.

`verify` exits with a failure, naming the first file that fails and why, unless every
check below passes.

## What `mint` writes

1. It builds a durable Ankurah node on a store in a temporary directory, with no peer,
   so that it registers models itself, under a key drawn for the run and discarded.
2. It registers every model of every namespace. Registration creates the definitions,
   as entities of Ankurah's catalog: one for the model, one for each of its properties,
   and one model-property link for each property, which gives the model the property
   and says whether it is optional.
3. It creates, in one transaction, an `EdgeKind` record for each kind of edge in the
   table in the `graph` namespace's source (`EDGE_KINDS`).
4. It reads each model and property as the node's catalog lists it, and takes from the
   store the event that created each entity, requiring exactly one, with no
   attestation.
5. It writes each model's definitions into `<model>.akev` beside the `<model>.rs` that
   declares it, and the edge kinds' records into `src/graph/edge_kinds.akev`, after
   checking that the text rebuilds exactly the events the node made.

An `.akev` file opens with the minter every event names, a public key. Each entity then
takes one line: what it is, its id, and what its creation event holds that its values do
not, the time it was minted and its nonce. An edge kind's description takes the indented
line beneath it.

    minter ed25519 <public key in hex>

    model     <label>  <struct name>   id <id>  at <ms>  nonce <hex>
    property  <name>  <value type>  <backend>  id <id>  at <ms>  nonce <hex>
    link      <name>  required        id <id>  at <ms>  nonce <hex>

    kind  <name>  id <id>  at <ms>  nonce <hex>
        <description>

A property's line is followed by its model-property link's. A file parses only in
exactly the form the writer gives it (`ankurah_commons::akev`).

## What `verify` establishes

It checks, stopping at the first file that fails:

1. Every listed model has its file beside its struct, the `graph` namespace has the edge
   kinds' file, and no other `.akev` file is under the directory.
2. Each file parses, in exactly the form the writer gives it.
3. Each file states its model as the struct declares it: its label, its name, and its
   properties in order, each with its value type, backend and whether it is optional.
   The edge kinds' file states the kinds of the `graph` namespace's table, in order.
4. Every creation event rebuilt from a file has the id its line gives. An entity's id
   is the hash of its creation event, so the text is exactly what was minted.

It does not establish:

- what state each event makes: the states are not rebuilt;
- who made the events: nothing is signed;
- that they are recent: an older mint of the same declarations verifies too.

## The list of models

`src/namespaces.rs` lists every namespace and its models: what `mint` registers. It also
gives `graph` the table of edge kinds from that namespace's source. A test
there keeps the list complete, so that a model added to a namespace, or a new
namespace, cannot be forgotten: for every namespace's module, a directory in the
library's `src/` named as the namespace with `-` written `_`, it compares the structs
the module's sources derive `Model` for with the models listed for that namespace, and
fails naming the first one missing on either side.

It relies on one convention: a model is a struct with `#[derive(Model, ..)]` written
directly on it, in a file under its namespace's directory in `src/`. A model declared
any other way is not seen.

## Known limits

- **Ids last only as a kept mint.** Every run draws a fresh key, a new random nonce for
  each event, and reads the clock, and all go into the event's id; the definitions that
  refer to others carry those others' ids. So the ids differ on every run, and are
  fixed only by keeping one run's output. The ids of a kept mint stay valid
  only while the Ankurah revision's event format, the way its last-writer-wins backend
  encodes values, and what a definition holds stay the same.
- **A reference property does not say what it refers to.** Registration writes a
  reference property's target model empty (Ankurah issue 513), so the definition of,
  say, `records.created`'s `by` does not name `identity.principal`, and an
  absent target says nothing about what a Rust `Ref<T>` refers to. An `.akev` file
  states no target model, and `mint` fails on a property that has one.
- **Only a definition's creation is handled.** A definition changed after it was
  minted, renamed or with a property made optional, would have further events after
  its creation. `mint` always starts from an empty store and an `.akev` line states a
  creation, so neither handles those yet.
- **No mint under a kept key.** Minting under a particular key that is kept, so that
  whoever holds it could later be recognised by it, is not offered; it would be a new
  mint, with new ids.
