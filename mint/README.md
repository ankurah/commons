# ankurah-commons-mint

Mints the definitions of the Commons' models and the Commons' kinds of edge, and checks
a mint. `mint` writes every entity it mints as a line of an `.akev` file, text from which
the entity's creation event is rebuilt (`ankurah_commons::rebuilding`): each model's
definitions beside the struct that declares it, and the edge kinds' records in
`src/graph/edge_kinds.akev`. `verify` checks those files.

Every creation event is a genesis in the format of the Ankurah revision the workspace
pins. It names no system root and no author, so it is the same entity, with the same id,
on every node that holds it. A kept mint is for seeding: `ankurah_commons::seed` imports
it into a durable node, which then uses the very same ids.

Nothing is signed. `verify` shows that the files rebuild the events their ids name; it
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

No node takes part. For each model of each namespace in the list below, `mint`:

1. takes from the struct's declaration what registering it writes
   (`ModelDefinitions::declared`): a definition of the model, its label and the name of
   its struct; a definition of each property, its name, value type and backend; and a
   model-property link for each property, which gives the model the property and says
   whether it is optional;
2. gives each of those entities the time and nonce of its line in the kept mint, when
   the kept mint has one, and otherwise the current time and a fresh nonce;
3. rebuilds each entity's creation event, which gives its id, and writes the
   definitions into `<model>.akev` beside the `<model>.rs` that declares it;
4. gives the struct its model's id: it parses the source and replaces the literal of the
   `id` option of the struct's `#[model(...)]` attributes, or, when the struct declares
   none, adds `#[model(id = "<id>")]` after the attribute that gives its label. Other
   options, attributes and comments stay as written.

Every file's text is made before any file is written, so a struct that declares two ids,
or whose attributes do not read, stops the mint with every file as it was.

It does the same for a record of each kind of edge in the table in the `graph`
namespace's source (`EDGE_KINDS`), its name and description, into
`src/graph/edge_kinds.akev`.

Minting a kept mint again therefore changes nothing unless a declaration, or the format
of Ankurah's events, changed; a new format changes only the ids. The ids the kept mint's
lines state are not read, so a mint made in an earlier format keeps its times and nonces
too. Nor are the ids the structs declare: a struct that declares none yet, or an old one,
is minted all the same.

Tests in the crate (`rebuilding`) compare the mint with Ankurah itself. Each of the
Commons' models is registered, without the id it declares, on a fresh durable node. The
mint makes its definitions from the declaration with the times and nonces that node drew,
writes them as text and reads them back, and the events rebuilt from that text, under the
node's root, must be the node's own, byte for byte. Between them the models declare every
value type a definition holds today (`bool`, `entityid`, `f64`, `i64`, `json`, `string`)
and both backends (`lww`, `yrs`). The same holds for a model with a property renamed from
a former name, and for the records of the kinds of edge, created on a node seeded with
the kept mint.

An `.akev` file takes one line for each entity: what it is, its id, and what its creation
event holds that its values do not, the time it was minted and its nonce. An edge kind's
description takes the indented line beneath it.

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
3. Each file states its model as the struct declares it: its label, its name, its id,
   and its properties in order, each with its value type, backend and whether it is
   optional. The edge kinds' file states the kinds of the `graph` namespace's table, in
   order. The declarations are the ones the tool was built with, so after a mint that
   changed an id, `verify` passes once the tool is built again, as `cargo run` does.
4. Every creation event rebuilt from a file has the id its line gives. An entity's id
   is the hash of its creation event, so the text is exactly what was minted.

It does not establish:

- who made the events: nothing is signed;
- that they are recent: an older mint of the same declarations verifies too.

## The list of models

The crate's `src/namespaces.rs` lists every namespace and the declarations of its models,
and gives `graph` the table of edge kinds from that namespace's source. `mint` mints the
whole list. A test there keeps the list complete, so that a model added to a namespace,
or a new namespace, cannot be forgotten: for every namespace's module, a directory in the
crate's `src/` named as the namespace, it compares the structs the module's sources
derive `Model` for with the models listed for that namespace, and fails naming the first
one missing on either side.

The text of the kept mint is not in that list: `src/seed.rs` compiles it in, in a list of
its own, so that the tool builds from the declarations alone. A model declared anew is
listed with no id, and `mint` writes its file and gives its struct the id; its file is
then added to the seed's list, which the crate's seed test checks.

It relies on one convention: a model is a struct with `#[derive(Model, ..)]` written
directly on it, in a file under its namespace's directory in `src/`. A model declared
any other way is not seen.

## Known limits

- **Ids hold only for one format.** An entity's id is the hash of its creation event, so
  the ids of a kept mint stay valid only while the Ankurah revision's event format, the
  way its last-writer-wins backend encodes values, and what a definition holds stay the
  same. Minting again after such a change keeps every time and nonce and gives new ids.
- **A reference property does not say what it refers to.** Registration writes a
  reference property's target model empty (Ankurah issue 513), so the definition of,
  say, `records.created`'s `by` does not name `identity.principal`, and an
  absent target says nothing about what a Rust `Ref<T>` refers to. An `.akev` file
  states no target model, and `mint` writes none, as registration writes none.
- **Only a definition's creation is handled.** A definition changed after it was
  minted, renamed or with a property made optional, would have further events after
  its creation. An `.akev` line states a creation, so `mint` gives a changed
  declaration new definitions, with new ids, rather than changing the old ones.
