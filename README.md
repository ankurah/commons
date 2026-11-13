# Ankurah Commons

Definitions of models and their properties that Ankurah applications can share, so
that two applications, or two nodes, mean the same thing by "a message" or "a task".

**Status: draft. Nothing here is minted, and no id is final.** The draft covers
models used by Fwd Centaur and proposed by BLE Activity Logger.

## Layout

One crate, `ankurah-commons`, with a module per namespace, one file per model:

    src/intention_graph/   tasks, decisions, intentions and the links between them
    src/messaging/         messages and mentions
    src/identity/          the people and agents who act
    src/records/           what any record says about its own life: who made it, whom
                           it waits on, who closed it
    src/health/            physical activity, workouts, distance and heart rate
    src/highlights/        parts of documents that someone marked
    src/git/               repositories, where a marked part sits in a file, and where
                           a body of work happens
    src/support.rs         what the namespaces share that is not a model
    mint/                  the tool that mints the definitions and checks a mint

The Rust declarations are the source: each `#[derive(Model)]` struct states a
model, its properties, how each is stored and which are optional.

## Features

Each namespace is a cargo feature of the same name, which also turns on the
namespaces its models refer to; `identity` is the default, and `full` turns on all
seven:

    identity          identity
    records           records, identity
    health            health, identity
    highlights        highlights
    git               git
    intention-graph   intention-graph, identity
    messaging         messaging, highlights, intention-graph, identity

A feature chooses what a build compiles, not what is minted: a kept mint is always
whole, whatever features a build uses, because a seeded store cannot be given more
definitions later.

## One entity, several components

A model is an Ankurah component, and one entity may carry several. The models are
cut along the lines Fwd Centaur uses them: a part is its own component when some
entities carry it and others do not, or when several kinds of record share it.

    a walk        PhysicalActivity, with Distance when known
    a workout     PhysicalActivity + Workout, with Ownership when an owner is known
    a pulse       HeartRate, which can also be observed before or after exercise
    a task        Task + Creation + Assignment, later Submission and Closure
    a comment     Message + Creation, later LastEdit and Closure
    a question    Message + Creation + Assignment, later Closure
    a mention     Link + Mention + Creation, later Closure
    a decision    Decision + Creation, later Closure
    an intention  Intention + Creation, later LastEdit and Closure; Attribution
                  when someone else wrote it down
    a scope       Scope + Creation, WorkLocation when it has a place in git,
                  later Closure
    a link        Link + Creation, later Closure
    a highlight   Fragment + FileAnchor + Creation
    a person or   Principal, Credential once something signs in as it
    an agent

On a personal workout, `PhysicalActivity.performed_by` names the person who exercised.
`Ownership.owner` names the principal whose record it is, and `Creation.created_by`
names the principal that created the record. On an imported ride the performer and
owner might be the same person while an importer creates the record a day later.
Recording an owner leaves authorization to the application's access policy.

A query that filters or sorts on properties of two components of one entity, such
as "the open tasks assigned to me", is on Ankurah's main branch as a typed query
over a tuple of views; the revision this crate builds against does not have it
yet. Still missing from Ankurah: a typed way to make an entity with several
components in one step.

## Publishing

A definition's id is the hash of its creation event, so two nodes mean the same
model only if they hold the same event. Publishing a model therefore means minting
that event once and keeping what was minted.

`mint/` does this: it mints every definition, writes each one's creation event and
its state to files named by its id, with a page per namespace that lists the ids
for readers who do not use Rust, and checks that such a directory is intact. A
mint that is kept is committed to this repository, and those bytes are then the
authority. A node that uses the models writes those files into its store before
it first starts, and from then on it uses the minted ids.

The first mint is unsigned, and the key it is minted under is discarded: what
vouches for the events is this repository. Signed statements can be added later
without changing an id.

Still to come:

- the minted ids written into the declarations, so that a node refuses to start
  on a store that lacks the definitions, where today it would mint its own;
- changes to a definition after it is minted, as further events beside its
  creation.

A comment that starts with `OPEN:` marks a question to settle before anything is
minted.
