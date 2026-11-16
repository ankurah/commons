# Ankurah Commons

Definitions of models and their properties that Ankurah applications can share, so
that two applications, or two nodes, mean the same thing by "a message" or "a task".

**Status: draft, with a mint kept, and no id final.** The draft covers models
used by Fwd Centaur and proposed by BLE Activity Logger. The definitions of every model
in the nine namespaces, 39 models with 107 properties and their links, and the 20 kinds
of edge in the `graph` namespace's table are minted, unsigned, in the `.akev` files
beside the structs, in the format of the Ankurah revision this crate builds against.

The crate builds against a revision of Ankurah named in `Cargo.toml`: its branch
`fwd-pin/portable-genesis`, which is its main with a node that imports events made
outside it. An application that uses the crate pins the same revision, and seeds its
node with the kept mint (see Seeding) so that the models take the minted ids.

## Layout

One crate, `ankurah-commons`, with a module per namespace, one file per model, and
beside each model's file the `.akev` file of its minted definitions:

    src/cognition/         what parties want, assert, understand, sense and decide:
                           intentions, claims, perspectives, intuitions, decisions and
                           expectations
    src/work/              tasks, the scopes they are worked in, and submissions of
                           finished work
    src/graph/             edges between records, the kinds of edge, and the table of
                           the kinds the Commons mints
    src/messaging/         messages and mentions
    src/identity/          the people and agents who act, and the accounts and
                           sessions that act as them
    src/records/           what any record says about its own life, in three groups:
                           lifecycle/ (made, edited, accepted, checked, closed),
                           provenance/ (whose words, made from what) and
                           responsibility/ (waits on whom, owned by whom); and at the
                           top, a note, a deferral and a place among siblings
    src/health/            physical activity, workouts, distance and heart rate
    src/text/              parts of documents that someone marked, and words someone
                           said, quoted
    src/git/               repositories, where a marked part sits in a file and what
                           the checkout held when it was marked, and where a body of
                           work happens
    src/property.rs        the property types the namespaces share: Timestamp, DynRef,
                           SnapshotRef and RefMap
    src/support.rs         what the namespaces share that is not a model or a property
                           type
    src/akev.rs            the text form of minted creation events, `.akev` files
    src/rebuilding.rs      the creation events rebuilt from `.akev` files
    src/namespaces.rs      the namespaces a build enables, their models, and the text
                           of their kept mint
    src/seed.rs            seeding a node with the kept mint
    mint/                  the tool that mints the definitions and checks a mint
                           (mint/README.md)

The Rust declarations are the source: each `#[derive(Model)]` struct states a
model, its properties, how each is stored and which are optional, and names the id the
kept mint gives the model.

Every model is marked `no_ffi`, so the derive generates no JavaScript or Swift
bindings for it. Without the mark the crate does not compile in a build that
turns on Ankurah's `wasm` or `uniffi` derive, because it depends on neither
`wasm-bindgen` nor `uniffi`. Bindings can come later, behind a feature, when an
application needs them.

## Features

Each namespace is a cargo feature of the same name, which also turns on the
namespaces its models refer to; `identity` is the default, and `full` turns on all
nine:

    identity    identity
    records     records, identity
    health      health, identity
    text        text
    git         git
    graph       graph
    cognition   cognition
    work        work
    messaging   messaging

One more feature, `jiff`, converts between the Commons' `Timestamp` and
`jiff::Timestamp`.

A feature chooses what a build compiles and seeds, not what is minted: the kept mint is
always whole, whatever features a build uses, so that every build means the same ids. A
store seeded by a build with fewer features can be seeded again by one with more.

## One entity, several components

A model is an Ankurah component, and one entity may carry several. The models are
cut along the lines Fwd Centaur uses them: a part is its own component when some
entities carry it and others do not, or when several kinds of record share it.

    a walk          PhysicalActivity, with Distance when known
    a workout       PhysicalActivity + Workout, with Ownership when an owner is known
    a pulse         HeartRate, which can also be observed before or after exercise
    a task          Task + Created + Assignment, with Acceptance when proposed and
                    Deferred when put off, later Closed
    a hand-in       Submission + Created, later Acceptance, and Closed when returned
    a comment       Message + Created, later Edited and Closed
    a question      Message + Created + Assignment, later Closed
    a mention       DynEdge + Mention + Created, later Closed
    a decision      Decision + Created + Acceptance, later Closed
    an intention    Intention + Created, later Edited and Closed
    a perspective   Perspective + Created, with Attribution when someone else
                    recorded it, later Edited
    a forecast      Expectation + Created, later Closed
    a quotation     Quotation + Created; Attribution when someone else recorded it
    a scope         Scope + Created, WorkLocation when it has a place in git,
                    later Closed
    an edge         DynEdge + Created, with Acceptance, Via, Note and
                    FractionalPosition when they apply, later Closed; Checked on a
                    checked step
    a highlight     Fragment + FileAnchor + Created, with CheckoutState when it was
                    made in a checkout
    a person or     Principal, later Closed when retired
    an agent
    an account or   IdentityBinding + Created, later Closed when unbound
    a session

On a personal workout, `PhysicalActivity.performed_by` names the person who exercised.
`Ownership.owner` names the principal whose record it is, and `Created.by`
names the principal that created the record. On an imported ride the performer and
owner might be the same person while an importer creates the record a day later.
Recording an owner leaves authorization to the application's access policy.

A query that filters or sorts on properties of two components of one entity, such
as "the open tasks assigned to me", is a typed query over a tuple of views in
Ankurah. Still missing from Ankurah: a typed way to make an entity with several
components in one step.

## Publishing

A definition's id is the hash of its creation event, so two nodes mean the same
model only if they hold the same event. Publishing a model therefore means minting
that event once and keeping what was minted.

`mint/` does this: it mints every definition, and a record of each kind of edge in the
`graph` namespace's table, and writes each entity as a line of an `.akev` file, text
from which its creation event is rebuilt exactly: each model's definitions beside the
struct that declares it, and the edge kinds' records in `src/graph/edge_kinds.akev`. A
mint that is kept is committed to this repository, and that text is then the
authority. Each creation event names no system root and no author, so it is the same
entity on every node.

The mint is unsigned: what vouches for the events is this repository. Signed statements
can be added later without changing an id.

Each struct declares the id its kept mint gives its model, `#[model(id = "...")]`, which
the mint tool writes.

Still to come: changes to a definition after it is minted, as further events beside its
creation.

A comment that starts with `OPEN:` marks a question still to settle. Settling one that
changes a model changes its definitions, which then need minting again.

## Seeding

`seed` imports the kept mint of the namespaces a build enables into a durable node,
through Ankurah's `Node::import_events`, so that the models bind to the minted
definitions instead of minting their own:

    let node = Node::new_durable(Arc::new(engine), agent);
    node.system.create().await?; // a fresh store; on one that holds a root, wait_system_ready()
    ankurah_commons::seed(&node).await?;

Seed the node before it first binds any of the Commons' models: registering a model
binds it, and so do a fetch, a query, a get and a create of it. Each model declares its
minted id and binds by that id, never by its label. On a node not yet seeded it fails
with Ankurah's `RegistrationError::ExplicitModelIdNotFound` rather than minting
definitions of its own; on a seeded node it binds to its minted definition even where the
store holds an older one under the same label, and finds its properties within that
definition, by name.

Seeding a store that holds the kept mint already changes nothing, so a node can be
seeded at every start.
