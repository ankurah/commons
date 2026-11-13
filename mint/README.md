# ankurah-commons-mint

Mints the definitions of the Commons' models, and checks a minted directory. `mint`
registers each model and keeps each definition as two files, the event that created it
and its state, with a page a reader can follow; `verify` checks those files. Whether a
mint is kept is the runner's choice.

The tool uses parts of Ankurah that exist only on its node identity branch
(`feat/node-identity-attestation`), so it is outside the workspace while the crate builds
against Ankurah's main. It builds and its tests pass at commit `a545fe9` of this
repository, where the whole workspace pins that branch at `a164293e`. It returns to the
workspace when the branch reaches Ankurah's main.

A kept mint is for seeding: another node, on a fresh store, is given the files before
it is built, and then uses the very same ids. Before building the node, the user opens
its storage engine and, in one storage transaction, calls
`set_state(&Clock::default(), &state)` for every state file, which holds a plain
`EntityState`, and `add_events` for every event, then commits. The node then finds the
definitions in its catalog, which is built from states, and registering models that
declare exactly what was minted binds them to the minted ids.

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
so the minter is nobody's identity. `verify` shows that the files are intact and
consistent; it does not show who made them. What vouches for a kept mint is the
repository it is committed to. Statements about the events can be added later without
changing a file's format or any id.

## Running it

    cargo run -p ankurah-commons-mint -- mint [--out <dir>]
    cargo run -p ankurah-commons-mint -- verify --dir <dir>

`mint` writes into a directory that must not exist yet. By default that is
`target/mint` in the workspace, which git ignores, so that a trial run leaves nothing
in the repository; a mint that is to be kept is written into the repository with
`--out` and committed.

`verify` exits with a failure, naming the first entry that fails and why, unless every
check below passes. It refuses any file in the directory that is not part of a mint, so
nothing else is to be put there.

## What `mint` writes

1. It builds a durable Ankurah node on a store in a temporary directory, with no peer,
   so that it registers models itself, under a key drawn for the run and discarded.
2. It registers every model of every namespace. Registration creates the definitions,
   as entities of Ankurah's catalog: one for the model, one for each of its properties,
   and one model-property link for each property, which gives the model the property
   and says whether it is optional.
3. It reads each model and property as the node's catalog lists it, and takes from the
   store the event that created each definition, requiring exactly one, with no
   attestation, and the definition's state.
4. It writes:

        <dir>/<namespace>/<id>.event  the event that created the definition <id>, as
                                      Ankurah encodes an event with its attestations,
                                      here an empty set; a creation event's id is its
                                      entity's id
        <dir>/<namespace>/<id>.state  the definition's state, in the tool's own encoding
        <dir>/<namespace>.md          a page listing every model with its label and id,
                                      and under it every property: its name, backend,
                                      value type, whether it is optional, its id and its
                                      model-property link's id

The state file is in the tool's own encoding, because Ankurah publishes none for a
state (it does for an event): bincode 1.3.3 with fixed-width integers, as Ankurah encodes
its own values, refusing bytes left over. It depends on the layout of Ankurah's state
types at the pinned revision, so a later revision may not read it. The store keeps a
state without attestations.

The page lists what the node's catalog said; it is a listing for readers that nothing
vouches for.

## What `verify` establishes

It checks, stopping at the first entry that fails:

1. The directory has the expected shape: directories holding, for each definition,
   exactly an event file and a state file named by its entity id; besides those, only
   each directory's page, which it does not read. Every entry it reads is checked for its type, without following a
   symlink, before it is opened, so a symlink, a FIFO or another special file is
   refused.
2. Each event file decodes with nothing left over, holds a well-formed event that
   creates an entity, and is named by that entity's id, recomputed from the event.
3. Each event carries no attestation: this tool checks none, so it refuses a file that
   has one rather than pass a statement nobody checked.
4. Each event creates a definition: it makes its entity a member of exactly one of the
   catalog's three models, of models, properties and model-property links. Only the
   memberships are read, not the values the event gives.
5. Each state file decodes with nothing left over, and holds the state of the same
   entity as its event, whose head is exactly that one creation event.
6. No definition appears twice in the directory.

It does not establish:

- that the definitions are those of any particular models, the Commons' or others: it
  reads no definition's values, and so does not compare them with any declaration;
- that a state's values are what its event gives: nothing public in Ankurah can derive
  a state from an event without a node;
- that a mint is complete: a mint is meant to be the whole seed for a store, and a node
  seeded with only part of it mints what is missing under its own key, but `verify`
  checks each definition it finds, not that none is missing;
- that the pages are true: it does not read them;
- who made the events: nothing is signed;
- that they are recent: an older mint verifies too.

## The list of models

`src/namespaces.rs` lists every namespace and its models: what `mint` registers. A test
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
  only while the Ankurah revision's event format and what a definition holds stay the
  same, and its state files stay readable only while the layout of Ankurah's state
  types does.
- **A reference property does not say what it refers to.** Registration writes a
  reference property's target model empty (Ankurah issue 513), so the definition of,
  say, `records.creation`'s `created_by` does not name `identity.principal`, and an
  absent target says nothing about what a Rust `Ref<T>` refers to.
- **Only a definition's creation is handled.** A definition changed after it was
  minted, renamed or with a property made optional, would have further events after
  its creation, and a state with a later head. `mint` always starts from an empty store
  and `verify` expects each event file to be a creation, so neither handles those yet.
  A definition's two files are named by its entity id, which is also its creation
  event's id; where a later event's file would go is not settled.
- **No mint under a kept key.** Minting under a particular key that is kept, so that
  whoever holds it could later be recognised by it, is not offered; it would be a new
  mint, with new ids.
