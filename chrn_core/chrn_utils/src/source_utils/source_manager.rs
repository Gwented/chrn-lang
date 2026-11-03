use std::{
    borrow::Cow,
    collections::HashMap,
    ops::Index,
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

// Arena?
/// Append-only source manager.
///
/// Sources valid in this arena are ones that act as access points.
/// It must be what would be considered the source of truth location to get the source.
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

impl SourcePath {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

//TODO: Kind of want something lkie "''''efl'afl'aefae" "imp virtual" what if import was imp?
// `import virtual "name"` where "name" is injected with a src. May still consider v"" but that
// seems like it's coupling string prefixes with syntactic properties, already turning it into vp""
// This could be implicit where if a "virtual" source exists, doing `import "thingy"` will search
// virtual first, see it, then process as normal, but would like for this to be more explicit at
// least for now.
/// Metadata for an in-memory source.
#[derive(Debug)]
pub struct SourceInMemory {
    /// Optional display name
    pub name: Option<InternedIdUTF8>,
    pub src: Vec<u8>,
}

impl SourceInMemory {
    pub fn new(name: Option<InternedIdUTF8>, src: Vec<u8>) -> Self {
        Self { name, src }
    }
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
            .push(SourceKind::Path(SourcePath::new(path.clone())));
        self.path_to_src_id.insert(path, id);
        id
    }

    /// Registers a distinct in-memory source with an optional display label.
    pub fn register_in_memory(
        &mut self,
        name: Option<InternedIdUTF8>,
        src: Vec<u8>,
    ) -> TaggedId<SourceId, InMemoryTag> {
        let id = SourceId::new(self.srcs.len() as u32).into_tagged::<InMemoryTag>();
        self.srcs
            .push(SourceKind::InMemory(SourceInMemory::new(name, src)));
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
