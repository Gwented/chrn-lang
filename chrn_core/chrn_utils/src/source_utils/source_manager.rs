use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

// Maybe most of this goes to compilation eventually
use crate::{
    id_types::{
        SourceId,
        id_tags::{ArenaIndexTag, TaggedId},
    },
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
    Path(PathBuf),
    InMemory,
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
        self.srcs.push(SourceKind::Path(path.clone()));
        self.path_to_src_id.insert(path, id);
        id
    }

    /// Registers `SourceId` for a `SourceKind::InMemory`
    pub fn register_in_memory(&mut self) -> TaggedId<SourceId, InMemoryTag> {
        let id = SourceId::new(self.srcs.len() as u32).into_tagged::<InMemoryTag>();
        self.srcs.push(SourceKind::InMemory);
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

    /// Returns the stored path's ID if present
    pub fn get_source_id_from_path(&self, path: &Path) -> Option<TaggedId<SourceId, PathTag>> {
        self.path_to_src_id.get(path).copied()
    }

    /// Returns the path assuming the ID exists
    pub fn get_path(&self, id: TaggedId<SourceId, PathTag>) -> Option<&Path> {
        match self.get_source(id.inner())? {
            SourceKind::Path(path) => Some(path.as_path()),
            SourceKind::InMemory => unreachable!("path-tagged source must contain a path"),
        }
    }

    /// Returns the path assuming the id is valid
    pub fn extract_path(&self, id: TaggedId<SourceId, PathTag>) -> &Path {
        match self.extract_source(id.inner()) {
            SourceKind::Path(path) => path.as_path(),
            SourceKind::InMemory => unreachable!("path-tagged source must contain a path"),
        }
    }

    /// Returns unit if the ID exists, trusting its in-memory tag.
    pub fn get_in_memory(&self, id: TaggedId<SourceId, InMemoryTag>) -> Option<()> {
        match self.get_source(id.inner())? {
            SourceKind::InMemory => Some(()),
            SourceKind::Path(_) => unreachable!("in-memory-tagged source must be in memory"),
        }
    }
}
