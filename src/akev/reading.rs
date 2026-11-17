//! Reads a set of files into the events they state, each built as a node builds a
//! creation and checked to be the entity its header names.

use std::{
    collections::{BTreeMap, HashMap},
    path::{Component, Path, PathBuf},
};

use ankurah::{
    core::property::backend::{LWWBackend, PropertyBackend},
    proto::{Event, ModelId},
    value::Value,
    EntityId, PropertyId,
};

use super::{
    genesis::{BackendOperations, Genesis, Operations},
    lexing,
    naming::{label_and_name, Names, Unanswered},
    parsing::{self, Block, Name, Reference, StatedOperations},
    read_error::{At, Problem, ReadError},
};

#[cfg(test)]
mod tests;

/// The events of `files`, each given by its path and its text, in the order of the files
/// and of the blocks in each. A name `name@file` refers to one of these files, by its path
/// relative to the directory of the file that refers. Each entity has one genesis in a read.
pub fn read(files: &[(&Path, &str)]) -> Result<Vec<Vec<Event>>, ReadError> {
    let mut parsed = Vec::with_capacity(files.len());
    for &(path, text) in files {
        let located = |At { line, problem }| ReadError { path: path.to_owned(), line, problem };
        parsed.push(parsing::blocks(&lexing::tokens(text).map_err(located)?).map_err(located)?);
    }
    let names: Vec<Names> = parsed.iter().map(|blocks| names(blocks)).collect();
    let read = Read {
        paths: files.iter().map(|(path, _)| *path).collect(),
        files: files.iter().enumerate().map(|(file, (path, _))| (normal(path), file)).collect(),
        blocks: &parsed,
        names: &names,
    };
    read.one_genesis_each()?;
    read.events()
}

/// The names each entity of a file answers to, from its `lww` values.
fn names<'t>(blocks: &'t [Block<'_>]) -> Names<'t> {
    let mut names = Names::default();
    for (place, block) in blocks.iter().enumerate() {
        let lww = <LWWBackend as PropertyBackend>::property_backend_name();
        let Some(StatedOperations::Values(entries)) =
            block.backends.iter().find(|field| field.backend == lww).map(|field| &field.operations)
        else {
            continue;
        };
        let given = entries.iter().filter_map(|entry| match (&entry.property, &entry.value) {
            (Reference::Given(property), Reference::Given(value)) => Some((property, value)),
            _ => None,
        });
        let (label, name) = label_and_name(given);
        names.add(place, label, name);
    }
    names
}

/// A block by its file's place in the read and its own in the file.
type Place = (usize, usize);

/// Where the building of a block's event stands.
enum State {
    Unbuilt,
    /// Its event waits on events it refers to.
    Building,
    Built(Event),
}

struct Read<'r, 't> {
    paths: Vec<&'r Path>,
    /// Each file's place, by its path made normal.
    files: HashMap<PathBuf, usize>,
    blocks: &'r [Vec<Block<'t>>],
    names: &'r [Names<'r>],
}

impl Read<'_, '_> {
    fn one_genesis_each(&self) -> Result<(), ReadError> {
        let mut first: HashMap<EntityId, (usize, usize)> = HashMap::new();
        for (file, blocks) in self.blocks.iter().enumerate() {
            for block in blocks {
                if let Some(&(first_file, first_line)) = first.get(&block.header) {
                    let first = self.paths[first_file].to_path_buf();
                    return Err(self.error(file, block.line, Problem::SecondGenesis { entity: block.header, first, first_line }));
                }
                first.insert(block.header, (file, block.line));
            }
        }
        Ok(())
    }

    /// Every block's event, each built after those it refers to.
    fn events(&self) -> Result<Vec<Vec<Event>>, ReadError> {
        let mut states: Vec<Vec<State>> = self.blocks.iter().map(|blocks| blocks.iter().map(|_| State::Unbuilt).collect()).collect();
        for (file, blocks) in self.blocks.iter().enumerate() {
            for place in 0..blocks.len() {
                self.build_from((file, place), &mut states)?;
            }
        }
        let built = |state| match state {
            State::Built(event) => event,
            State::Unbuilt | State::Building => unreachable!("every block is built before the events are taken"),
        };
        Ok(states.into_iter().map(|states| states.into_iter().map(built).collect()).collect())
    }

    /// Builds the event of the block at `start`, after the events it refers to, depth first
    /// and without recursion, so that no chain of references, however long, can overflow
    /// the stack.
    fn build_from(&self, start: Place, states: &mut [Vec<State>]) -> Result<(), ReadError> {
        if !matches!(states[start.0][start.1], State::Unbuilt) {
            return Ok(());
        }
        states[start.0][start.1] = State::Building;
        let mut waiting = vec![start];
        while let Some(&place) = waiting.last() {
            match self.unbuilt_reference(place, states)? {
                Some(((file, block), line)) => {
                    if matches!(states[file][block], State::Building) {
                        return Err(self.error(place.0, line, Problem::ReferenceCycle));
                    }
                    states[file][block] = State::Building;
                    waiting.push((file, block));
                }
                None => {
                    states[place.0][place.1] = State::Built(self.build(place, states)?);
                    waiting.pop();
                }
            }
        }
        Ok(())
    }

    /// The first entity the block at `place` refers to whose event is not built, and the
    /// line of the reference.
    fn unbuilt_reference(&self, place: Place, states: &[Vec<State>]) -> Result<Option<(Place, usize)>, ReadError> {
        for (name, line) in references(&self.blocks[place.0][place.1]) {
            let referred = self.resolve(place.0, name, line)?;
            if !matches!(states[referred.0][referred.1], State::Built(_)) {
                return Ok(Some((referred, line)));
            }
        }
        Ok(None)
    }

    /// The event of the block at `place`, every entity it refers to built.
    fn build(&self, (file, place): Place, states: &[Vec<State>]) -> Result<Event, ReadError> {
        let block = &self.blocks[file][place];
        let id = |name: &Name, line| -> Result<EntityId, ReadError> {
            let (file, place) = self.resolve(file, name, line)?;
            match &states[file][place] {
                State::Built(event) => Ok(event.entity_id),
                State::Unbuilt | State::Building => unreachable!("a block is built after every entity it refers to"),
            }
        };
        let mut backends = BTreeMap::new();
        for field in &block.backends {
            let operations = match &field.operations {
                StatedOperations::Bytes(operation) => BackendOperations::Bytes(operation.clone()),
                StatedOperations::Values(entries) => {
                    let mut values = Vec::with_capacity(entries.len());
                    for entry in entries {
                        let property = match &entry.property {
                            Reference::Given(property) => *property,
                            Reference::Named(name) => PropertyId::EntityId(id(name, entry.line)?),
                        };
                        let value = match &entry.value {
                            Reference::Given(value) => value.clone(),
                            Reference::Named(name) => Some(Value::EntityId(id(name, entry.line)?)),
                        };
                        values.push((property, value));
                    }
                    BackendOperations::Values(values)
                }
            };
            backends.insert(field.backend.to_owned(), operations);
        }
        let mut memberships = Vec::with_capacity(block.memberships.len());
        for membership in &block.memberships {
            memberships.push(match &membership.value {
                Reference::Given(model) => *model,
                Reference::Named(name) => ModelId::EntityId(id(name, membership.line)?),
            });
        }
        let genesis = Genesis { nonce: block.nonce, timestamp: block.timestamp, operations: Operations { backends, memberships } };
        let event = genesis.event().map_err(|source| {
            let line = block.backends.iter().find(|field| field.backend == source.backend()).map_or(block.line, |field| field.line);
            self.error(file, line, Problem::Build(source))
        })?;
        if event.entity_id != block.header {
            return Err(self.error(file, block.line, Problem::NotTheHeadersEntity { header: block.header, built: event.entity_id }));
        }
        if let Some(stated) = block.id.as_ref().filter(|stated| stated.value != block.header) {
            return Err(self.error(file, stated.line, Problem::IdNotTheHeader { header: block.header, stated: stated.value }));
        }
        Ok(event)
    }

    /// The block that `name`, in the file at `file`, refers to.
    fn resolve(&self, file: usize, name: &Name, line: usize) -> Result<Place, ReadError> {
        let named = match name.file {
            None => file,
            Some(path) => {
                let path = normal(&self.paths[file].parent().unwrap_or(Path::new("")).join(path));
                *self.files.get(&path).ok_or_else(|| self.error(file, line, Problem::UnknownFile(path)))?
            }
        };
        let written = || match name.file {
            None => format!("${}", name.name),
            Some(path) => format!("{}@{path}", name.name),
        };
        match self.names[named].resolve(name.name) {
            Ok(place) => Ok((named, place)),
            Err(Unanswered::Unknown) => Err(self.error(file, line, Problem::UnknownName(written()))),
            Err(Unanswered::Ambiguous(entities)) => Err(self.error(file, line, Problem::AmbiguousName { name: written(), entities })),
        }
    }

    fn error(&self, file: usize, line: usize, problem: Problem) -> ReadError {
        ReadError { path: self.paths[file].to_owned(), line, problem }
    }
}

/// Every name `block` writes, with its line.
fn references<'b, 't>(block: &'b Block<'t>) -> impl Iterator<Item = (&'b Name<'t>, usize)> {
    let memberships = block.memberships.iter().filter_map(|membership| match &membership.value {
        Reference::Named(name) => Some((name, membership.line)),
        Reference::Given(_) => None,
    });
    let values = block.backends.iter().flat_map(|field| match &field.operations {
        StatedOperations::Values(entries) => entries.as_slice(),
        StatedOperations::Bytes(_) => &[],
    });
    let values = values.flat_map(|entry| {
        let property = match &entry.property {
            Reference::Named(name) => Some((name, entry.line)),
            Reference::Given(_) => None,
        };
        let value = match &entry.value {
            Reference::Named(name) => Some((name, entry.line)),
            Reference::Given(_) => None,
        };
        property.into_iter().chain(value)
    });
    memberships.chain(values)
}

/// `path` with its `.` components dropped and each `..` taking off the component before it.
fn normal(path: &Path) -> PathBuf {
    let mut normal = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir if matches!(normal.components().next_back(), Some(Component::Normal(_))) => {
                normal.pop();
            }
            other => normal.push(other),
        }
    }
    normal
}
