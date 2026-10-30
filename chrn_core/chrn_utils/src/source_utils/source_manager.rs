use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
};

// Maybe most of this goes to compilation eventually
use crate::{
    id_types::{
        InternedIdUTF8, SourceId,
        id_tags::{ArenaIndexTag, TaggedId},
    },
    intern::Intern,
    tag_decl,
};

tag_decl!(PathTag, InMemoryTag);

/// Append-only source mapper.
#[derive(Debug, Default)]
pub struct SourceManager {
    srcs: Vec<SourceKind>,
    path_to_src_id: HashMap<PathBuf, TaggedId<SourceId, PathTag>>,
}

/// Possible sources to store and index
#[derive(Debug)]
pub enum SourceKind {
    Path(SourcePath),
    InMemory(SourceInMemory),
}

/// Metadata for a filesystem source.
#[derive(Debug)]
pub struct SourcePath {
    pub path: PathBuf,
}

/// Metadata for an in-memory source.
#[derive(Debug)]
pub struct SourceInMemory {
    /// Optional display label, interned in the associated string interner.
    /// Labels do not determine source identity and need not be unique.
    pub name: Option<InternedIdUTF8>,
}

impl SourceManager {
    /// Creates empty `SourceManager`
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates `SourceManager` with capacity for `srcs` and `path_to_src_id`
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            srcs: Vec::with_capacity(capacity),
            path_to_src_id: HashMap::with_capacity(capacity),
        }
    }

    /// Registers a path, reusing its ID if already stored.
    pub fn register_path(&mut self, path: &Path) -> TaggedId<SourceId, PathTag> {
        if let Some(id) = self.path_to_src_id.get(path) {
            return *id;
        }

        let id = SourceId::new(self.srcs.len() as u32).into_tagged::<PathTag>();
        let path = path.to_path_buf();
        self.srcs
            .push(SourceKind::Path(SourcePath { path: path.clone() }));
        self.path_to_src_id.insert(path, id);
        id
    }

    /// Registers a distinct in-memory source with an optional display label.
    pub fn register_in_memory(
        &mut self,
        name: Option<InternedIdUTF8>,
    ) -> TaggedId<SourceId, InMemoryTag> {
        let id = SourceId::new(self.srcs.len() as u32).into_tagged::<InMemoryTag>();
        self.srcs
            .push(SourceKind::InMemory(SourceInMemory { name }));
        id
    }

    /// Returns the source kind if the ID exists in this manager.
    pub fn get_source(&self, id: SourceId) -> Option<&SourceKind> {
        self.srcs.get(id.id as usize)
    }

    /// Returns the source kind, assuming the ID exists in this manager.
    pub fn extract_source(&self, id: SourceId) -> &SourceKind {
        &self.srcs[id.id as usize]
    }

    /// Returns a displayable name for the found `SourceKind`
    pub fn display_source<'a>(&'a self, id: SourceId, interner: &'a Intern) -> Cow<'a, str> {
        match self.extract_source(id) {
            SourceKind::Path(src) => src.path.to_string_lossy(),
            SourceKind::InMemory(src) => {
                let src_name = src
                    .name
                    .map_or("source", |name_id| interner.extract_utf8_str(name_id));
                Cow::Borrowed(src_name)
            }
        }
    }

    /// Returns the stored path's ID if present
    pub fn get_source_id_from_path(&self, path: &Path) -> Option<TaggedId<SourceId, PathTag>> {
        self.path_to_src_id.get(path).copied()
    }

    /// Returns path metadata if the ID exists, trusting its path tag.
    pub fn get_path(&self, id: TaggedId<SourceId, PathTag>) -> Option<&SourcePath> {
        match self.get_source(id.inner())? {
            SourceKind::Path(src) => Some(src),
            SourceKind::InMemory(_) => unreachable!("path-tagged source must contain a path"),
        }
    }

    /// Returns path metadata, assuming the ID is valid.
    pub fn extract_path(&self, id: TaggedId<SourceId, PathTag>) -> &SourcePath {
        match self.extract_source(id.inner()) {
            SourceKind::Path(src) => src,
            SourceKind::InMemory(_) => unreachable!("path-tagged source must contain a path"),
        }
    }

    /// Returns in-memory metadata if the ID exists
    pub fn get_in_memory(&self, id: TaggedId<SourceId, InMemoryTag>) -> Option<&SourceInMemory> {
        match self.get_source(id.inner())? {
            SourceKind::InMemory(src) => Some(src),
            SourceKind::Path(_) => unreachable!("in-memory-tagged source must be in memory"),
        }
    }
}
