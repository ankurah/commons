//! `.akev` files: events written as text, so that a reader sees what each event holds and a
//! node builds the very events, byte for byte, from what it reads.
//!
//! A file is a list of blocks, one for each event, with blank lines between them:
//!
//! ```text
//! [W_PUyePLDVZm5ly-2sTyWjIHyu4zihZ5QWcCUGG-mHY]
//! nonce: I-QnwMppDz-TldUGtuitFV23BYGElMoId6A_2Xdqs2s, timestamp: 2026-11-16T02:14:23.846Z
//! lww: { label: "cognition.intention", name: "Intention" }, member: system:model
//! ```
//!
//! A block's header is the id of the entity it creates, there to be checked against the
//! event built from the block. Its fields, `key: value`, stand on one line or several,
//! between commas or line breaks. `nonce`, `timestamp`, `member`, `parent` and `id` are the
//! event's own fields; every other key names a property backend, whose operations for the
//! event are text that is run through the backend, `{ property: value, ... }`, or the bytes
//! of one operation in base64. A name, `$name` for an entity of the same file or
//! `name@file.akev` for one of another, stands for the id of the entity whose `lww` values
//! hold that `label`, or else that `name`. Comments are `//` to the end of the line and
//! `/* ... */`.
//!
//! The reader knows no models: only Ankurah's built-in models and properties, by the names
//! Ankurah prints them by, and the two built-in properties that name an entity.

mod genesis;
mod lexing;
mod naming;
mod parsing;
mod read_error;
mod reading;
#[cfg(test)]
mod sample;
mod timestamp;

pub use genesis::{BackendOperations, BuildError, Genesis, Operations};
pub use read_error::{Problem, ReadError};
pub use reading::read;
pub use timestamp::TimestampError;
