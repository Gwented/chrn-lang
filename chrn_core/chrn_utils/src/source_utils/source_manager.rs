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

// Arena?
/// Append-only source manager.
///
/// Sources valid in this arena are ones that act as access points.
/// It must be what would be considered the source of truth location to get the source.
#[derive(Debug, Default)]
pub struct SourceManager {
    srcs: Vec<SourceKind>,
    path_to_src_id: HashMap<PathBuf, TaggedId<SourceId, PathTag>>,
    // Bytes? Also two hashmaps :(
    // This uses `PathBuf` because path ids do not exist until the manager makes it. Interned ones
    // do exist prior therefore this doesn't use str/bytes or anything
    interned_id_to_src_id: HashMap<InternedIdUTF8, SourceId>,
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

/// Metadata for an in-memory source.
#[derive(Debug)]
pub struct SourceInMemory {
    // I document like an LLM.
    /// Required name for in-memory source
    pub name: InternedIdUTF8,
    /// Full source data
    pub src: Vec<u8>,
}

impl SourceInMemory {
    pub fn new(name: InternedIdUTF8, src: Vec<u8>) -> Self {
        Self { name, src }
    }
}

impl SourceManager {
    /// Creates empty `SourceManager`
    pub fn new() -> Self {
        Self::with_capacities(0, 0, 0)
    }

    /// Creates `SourceManager` with capacity for sources and registration maps.
    pub fn with_capacities(
        srcs_cap: usize,
        path_src_id_cap: usize,
        interned_id_to_src_id_cap: usize,
    ) -> Self {
        Self {
            srcs: Vec::with_capacity(srcs_cap),
            path_to_src_id: HashMap::with_capacity(path_src_id_cap),
            interned_id_to_src_id: HashMap::with_capacity(interned_id_to_src_id_cap),
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

    /// Registers an in-memory source, reusing its ID if the name is already stored.
    /// Re-registering a name preserves the original source data.
    pub fn register_in_memory(
        &mut self,
        name_id: InternedIdUTF8,
        src: Vec<u8>,
    ) -> TaggedId<SourceId, InMemoryTag> {
        if let Some(id) = self.interned_id_to_src_id.get(&name_id) {
            return id.into_tagged::<InMemoryTag>();
        }

        let id = SourceId::new(self.srcs.len() as u32).into_tagged::<InMemoryTag>();
        self.srcs
            .push(SourceKind::InMemory(SourceInMemory::new(name_id, src)));
        self.interned_id_to_src_id.insert(name_id, id.inner());
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
            SourceKind::Path(src_p) => src_p.path.to_string_lossy(),
            SourceKind::InMemory(src_in_mem) => {
                Cow::Borrowed(interner.extract_utf8_str(src_in_mem.name))
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

    /// Returns in-memory metadata if the ID exists
    pub fn get_in_memory_from_interned_id(&self, id: InternedIdUTF8) -> Option<&SourceInMemory> {
        // Has to exist at this point
        let src_id = self.get_source_id_from_interned_id(id)?;
        match self.extract_source(src_id) {
            SourceKind::InMemory(src) => Some(src),
            SourceKind::Path(_) => None,
        }
    }

    pub fn get_source_id_from_interned_id(&self, id: InternedIdUTF8) -> Option<SourceId> {
        self.interned_id_to_src_id.get(&id).copied()
    }
}
