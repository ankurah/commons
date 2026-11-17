# ankurah-commons-mint

Mints the definitions of the Commons' models and the Commons' kinds of edge, and checks
a mint. `mint` writes the creation event of every entity it mints as a block of an
`.akev` file (`ankurah_commons::akev`), text from which a node builds the very event:
each model's definitions beside the struct that declares it, and the records of the
kinds of edge after `EdgeKind`'s definitions, in `src/graph/edge_kind.akev`. `verify`
checks those files.

Every creation event is a genesis in the format of the Ankurah revision the workspace
pins. It names no system root and no author, so it is the same entity, with the same id,
on every node that holds it. A kept mint is for seeding: `ankurah_commons::seed` imports
it into a durable node, which then uses the very same ids.

Nothing is signed. `verify` shows that the files hold the events their headers name; it
does not show who made them. What vouches for a kept mint is the repository it is
committed to. Statements about the events can be added later without changing any id.

## Running it

    cargo run -p ankurah-commons-mint -- mint [--src <dir>]
    cargo run -p ankurah-commons-mint -- verify [--src <dir>]

Both work on the namespaces' parent directory, the crate's `src/` by default. `mint`
replaces the kept mint's files there; git keeps the old ones. A trial run is given a copy
of `src/` with `--src`. A mint that is to be kept is written into `src/` and committed.

`verify` exits with a failure, naming the first file that fails and why, unless every
check below passes.

## What `mint` writes

No node takes part. `mint` reads every kept file under the directory, and for each model
of each namespace in the list below it:

1. takes from the struct's declaration the events that registering it writes
   (`ankurah_commons::registration`): a definition of the model, its label and the name
   of its struct; a definition of each property, its name, value type and backend; and
   a model-property link for each property, which gives the model the property and says
   whether it is optional;
2. gives each event the nonce and timestamp of an event of the model's kept file whose
   operations are exactly the same, when the file holds one, and otherwise a fresh nonce
   and the current time;
3. writes the events into `<model>.akev` beside the `<model>.rs` that declares the model;
4. gives the struct its model's id: it parses the source and replaces the literal of the
   `id` option of the struct's `#[model(...)]` attributes, or, when the struct declares
   none, adds `#[model(id = "<id>")]` after the attribute that gives its label. Other
   options, attributes and comments stay as written.

Every file's text is made before any file is written, so a struct that declares two ids,
or whose attributes do not read, stops the mint with every file as it was.

It does the same for a record of each kind of edge in the table in the `graph`
namespace's source (`EDGE_KINDS`), its name and description, after `EdgeKind`'s
definitions in `src/graph/edge_kind.akev`.

Minting a kept mint again therefore changes nothing unless a declaration changed: an
event whose operations no declaration needs any more gives way to one drawn anew, with a
new id, and `mint` says how many it drew. It ignores the ids the structs declare: a
struct that declares none yet, or an old one, is minted all the same.

Tests compare the mint with Ankurah itself. Each of the Commons' models is registered,
without the id it declares, on a fresh durable node; `registration` makes its events from
the declaration with the nonces and timestamps that node drew, and under the node's root
they must be the node's own events, byte for byte, and written as text they must read
back as themselves. Between them the models declare every value type a definition holds
today (`bool`, `entityid`, `f64`, `i64`, `json`, `string`) and both backends (`lww`,
`yrs`). The same holds for a model with a property renamed from a former name, and, in
this tool, for the records of the kinds of edge, created on a node seeded with the kept
mint.

## The text form

A file is a list of events, one block each, with blank lines between the blocks. The
first three of `src/cognition/intention.akev`:

    [W_PUyePLDVZm5ly-2sTyWjIHyu4zihZ5QWcCUGG-mHY]
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

- **The header** is the id of the entity the block creates, alone on its line. It is
  there to be checked: the reader builds the event from the rest of the block and
  refuses it, stating both ids, unless it is that entity's genesis. No name stands in a
  header.
- **Fields**, `key: value`, follow it in any order, between commas or line breaks, on one
  line or several.
- **The event's own fields** are the reserved keys. `nonce` is the event's 32 bytes, in
  the base64 of an id. `timestamp` is its Unix milliseconds in one form only, UTC with
  three decimals. `member` adds the entity to a model, as Ankurah prints a model's id or
  by a name; it may repeat, and the memberships follow the order written. `parent` would
  make the block an update, which is not read yet. `id`, where a block has one, is the
  event's id, checked against the header.
- **Every other key names a property backend**, and its value is that backend's
  operations for the event: text, `{ property: value, ... }`, set on a fresh backend whose
  operations are then taken, or the bytes of one operation in base64. Only `lww` has a
  text form; `yrs`, and an `lww` value with no literal yet (a number, JSON, bytes), are
  written as bytes. A property is a built-in one by the name Ankurah prints it by, an id
  or a name; a value is a string, `true`, `false`, `none`, an id or a name.
- **Names.** `$name` is the entity of the same file whose `lww` text holds `label:
  "name"`, or, where no entity's does, `name: "name"`; `name@other.akev` is the same in
  another file, by its path from the directory of the file that refers. Exactly one
  entity must answer to a name. A name stands as a `member`'s value, or as a key or a
  value of `lww` text; never in a header or an `id`.
- **Comments** are `//` to the end of the line and `/* ... */`.

The writer lays each block out as above: the values on the line of their key, with the
memberships, where that fits in 100 columns and the values hold no id and no name, and
otherwise a line for each value. It writes a name for an id of an entity in the same file
that answers to that one name. The reader takes any layout the rules allow.

The text form has changed twice, each time by a one-time conversion that kept every nonce
and timestamp. The first dropped the minter line the mint at `a9ea175` opened each file
with. The second replaced the form of one line for each entity, a `model`, `property`,
`link` or `kind` line, with events: each file of the mint kept at `fa9ac91` was read with
that form's parser and written by the new writer, which kept every one of the 273 ids.
`mint` reads no other form, so a later change to the text form needs a conversion of the
same kind.

## What `verify` establishes

It checks, stopping at the first file that fails:

1. Every listed model has its file beside its struct, and no other `.akev` file is under
   the directory.
2. Every file reads: each block builds its event, the genesis of the entity its header
   names. An entity's id is the hash of its creation event, so the text is exactly what
   was minted.
3. Each file states exactly the events its model's declaration needs, and, for
   `graph.edge_kind`, the records of the kinds of the `graph` namespace's table: minting
   again would keep every one of its events and draw none. The struct declares the id the
   file states for its model. The declarations are the ones the tool was built with, so
   after a mint that changed an id, `verify` passes once the tool is built again, as
   `cargo run` does.

It does not establish:

- who made the events: nothing is signed;
- that they are recent: an older mint of the same declarations verifies too.

## The list of models

The crate's `src/namespaces.rs` lists every namespace and the declarations of its models,
and gives `graph` the table of edge kinds from that namespace's source. `mint` mints the
whole list. A test there keeps the list complete, so that a model added to a namespace,
or a new namespace, cannot be forgotten: for every namespace module, a module of the
crate's root behind the feature of its own name, it compares the structs the sources in
the module's directory derive `Model` for with the models listed for that namespace, and
fails naming the first one missing on either side.

The text of the kept mint is not in that list: `src/seed.rs` compiles it in, in a list of
its own, so that the tool builds from the declarations alone. A model declared anew is
listed with no id, and `mint` writes its file and gives its struct the id; its file is
then added to the seed's list, which the crate's seed test checks.

It relies on one convention more: a model is a struct with `#[derive(Model, ..)]` written
directly on it, in a file under its namespace's directory in `src/`. A model declared
any other way is not seen.

## Known limits

- **Ids hold only for one format.** An entity's id is the hash of its creation event, so
  the ids of a kept mint stay valid only while the Ankurah revision's event format, the
  way its last-writer-wins backend encodes values, and what registering a declaration
  writes stay the same. The two kinds of change play out differently:
  - A change to the event format, or to how `lww` encodes the values its text sets, means
    no block's text builds the entity its header names any more. No kept file then reads,
    and `mint` stops until a one-time conversion, like a change of the text form, carries
    the mint over, keeping every nonce and timestamp and giving new ids.
  - A change to what registering a declaration writes leaves every file readable.
    `verify` fails, naming the first file that lacks an event the declarations now need,
    and `mint` draws the changed definitions anew, with new ids.
- **A reference property does not say what it refers to.** Registration writes a
  reference property's target model empty (Ankurah issue 513), so the definition of,
  say, `records.created`'s `by` does not name `identity.principal`, and an
  absent target says nothing about what a Rust `Ref<T>` refers to. Each property's
  definition states `target_model: none`, as registration writes it.
- **Only a definition's creation is handled.** A definition changed after it was
  minted, renamed or with a property made optional, would have further events after
  its creation, and the reader does not read updates yet. So `mint` gives a changed
  declaration new definitions, with new ids, rather than changing the old ones.
