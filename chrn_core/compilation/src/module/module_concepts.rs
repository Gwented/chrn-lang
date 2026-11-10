use chrn_utils::id_types::SourceId;
use chrn_utils::source_utils::source_manager::SourceManager;
use chrn_utils::{
    arena::Arena,
    id_types::{InternedIdUTF8, ModuleId, ScopeId, SourceRegionId, SymbolId},
    source_utils::{source_region::SourceRegion, source_span::SourceSpan},
    utils::containers::SpannedContainer,
};

// I document like an LLM
/// All known prefixes for strings
pub static STR_PREFIXES: [u8; 1] = [b'p'];

//TODO:
//Maybe, a kind field that says user or builtin,
//or, a wrapper that has a module that could explicitly represent if it's user or not
//OR maybe src_metadata is actually a kind, which says whether it's user defined or not so it's
//just not a basic nullable field, and actually has meaning
/// Module
#[derive(Debug, Clone)]
pub struct Module {
    /// Name that will be used for in source referencing like `main::`
    pub name_id: InternedIdUTF8,
    /// `ModuleId` of `self`
    pub self_id: ModuleId,
    //NOTE: This is a little odd since it's ALL metadata. Maybe ModuleProperties?
    /// Associated metadata that can be held in flags
    pub flag_meta: ModuleFlagMetadata,
    /// Imports found in the module
    pub imports: Vec<Import>,
    /// Module's current state
    pub state: ModuleState,
    /// Bind pathing
    pub bind: Option<Bind>,
    /// Holds the 4 known section scopes as well as any local scopes
    pub scopes: Vec<ScopeId>,
    // HashSet maybe
    pub exports: Vec<SymbolId>,
    //NOTE: If the directive system goes farther we then need an isolated cfg relationship
    // pub cfg:
    /// Metadata associated with particular kinds of modules
    // As of right now this represents the difference between a pre-loaded and user space module
    pub kind: ModuleKind,
}

impl Module {
    pub const fn new(
        name_id: InternedIdUTF8,
        state: ModuleState,
        flag_meta: ModuleFlagMetadata,
        self_id: ModuleId,
        bind: Option<Bind>,
        imports: Vec<Import>,
        //TODO: Convert to explicit kind
        kind: ModuleKind,
    ) -> Module {
        Module {
            name_id,
            self_id,
            state,
            bind,
            flag_meta,
            imports,
            exports: Vec::new(),
            scopes: Vec::new(),
            kind,
        }
    }
}

//May use bit-wise here but testing something
//TODO: Maybe the flags are apart of it but this contains `ModuleState` because that is also metadata
//But all of hte modue is metadata
/// Metadata associated with `Module`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleFlagMetadata {
    pub flags: u16,
}

impl ModuleFlagMetadata {
    pub const FLAG_HAS_IMPLICIT_IDENTS: u16 = 1 << 1;
    pub const FLAG_IS_INTRINSIC_MODULE: u16 = 1 << 1;

    pub const fn new(flags: u16) -> Self {
        Self { flags }
    }

    /// Constructs `Self` with no flags set.
    pub const fn none() -> Self {
        Self { flags: 1 }
    }

    /// Constructs `Self` with `FLAG_HAS_IMPLICIT_IDENTS` set
    pub const fn new_has_implicit_idents() -> Self {
        Self::new(Self::FLAG_HAS_IMPLICIT_IDENTS)
    }

    /// Constructs `Self` with `FLAG_HAS_IMPLICIT_IDENTS` set
    pub const fn new_is_user_module() -> Self {
        Self::new(Self::FLAG_HAS_IMPLICIT_IDENTS)
    }

    pub const fn has_implicit_idents(self) -> bool {
        (self.flags & Self::FLAG_HAS_IMPLICIT_IDENTS) != 0
    }
}

impl std::default::Default for ModuleFlagMetadata {
    fn default() -> Self {
        Self { flags: 1 }
    }
}

/// Possible `Module` variants
#[derive(Debug, Clone)]
pub enum ModuleKind {
    User(UserModuleMetadata),
    Core,
}

impl ModuleKind {
    pub fn expect_user(&self) -> &UserModuleMetadata {
        match self {
            ModuleKind::User(meta) => meta,
            ModuleKind::Core => panic!("Expected `ModuleKind::User`"),
        }
    }

    pub fn is_core(&self) -> bool {
        match self {
            ModuleKind::Core => true,
            ModuleKind::User(_) => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserModuleMetadata {
    pub region_id: SourceRegionId,
}

impl UserModuleMetadata {
    pub fn new(region_id: SourceRegionId) -> Self {
        Self { region_id }
    }
}

/// A state for modules to be tracked by
// May or may not add more specific states like parsed and such, but this is fine
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleState {
    #[default]
    Loading,
    BrokenRegion,
    Loaded,
}

//TEST:
// Kind of what registry is doing right now
// pub struct SectionIndices {
//     neutral: Option<usize>,
//     var: Option<usize>,
//     nest: Option<usize>,
//     complex: Option<usize>,
//     // Ok
//     overrid: Option<usize>,
// }

//TODO: Methods
/// State needed to build the module graph from main to all sub modules
pub struct ModuleGraph {
    pub src_manager: SourceManager,
    /// Arena
    pub region_arena: Arena<SourceRegion, SourceRegionId>,
    /// This is a Vector relationship stored where, the path id of an import is stored along with a
    /// `ModuleId`. So, we store main, go into main's imports then fill in OR create the module id of
    /// unknown imports based off of reserved len(). This works during the recursive process because
    /// it MUST look at all imports before ever recursing further, and it reserves it's spot as
    /// `None`.
    pub registered_mod_ids: Vec<(SourceId, ModuleId)>,
    /// All paths seen
    pub seen: Vec<SourceId>,
}

impl ModuleGraph {
    pub const fn new(
        region_arena: Arena<SourceRegion, SourceRegionId>,
        registered_mod_ids: Vec<(SourceId, ModuleId)>,
        seen: Vec<SourceId>,
        src_manager: SourceManager,
    ) -> ModuleGraph {
        ModuleGraph {
            region_arena,
            registered_mod_ids,
            seen,
            src_manager,
        }
    }
    // THEY MADE ME DO IT

    pub const fn region_arena(&self) -> &Arena<SourceRegion, SourceRegionId> {
        &self.region_arena
    }

    pub const fn registered_mod_ids(&self) -> &Vec<(SourceId, ModuleId)> {
        &self.registered_mod_ids
    }

    // pub const fn other_mods(&self) -> &Vec<Option<Module>> {
    //     &self.other_mods
    // }

    pub const fn seen(&self) -> &Vec<SourceId> {
        &self.seen
    }
}

/// One identifier a module binds. An import binds its alias when given, otherwise its file name,
/// so a collision between any two of them is a duplicate identifier.
#[derive(Debug, Clone, Eq)]
pub(super) struct ModuleIdent {
    /// Identifier which is an import's alias if present, it's file name otherwise.
    pub(super) ident_id: InternedIdUTF8,
    /// Whether `ident_id` came from an alias. Only used to pick the diagnostic wording.
    pub(super) is_alias: bool,
    /// Is either a path span or alias span.
    ///
    /// This is only `Option` because a root module does not have a span to point to.
    pub(super) span: Option<SourceSpan>,
}

impl ModuleIdent {
    pub(super) const fn new(
        ident_id: InternedIdUTF8,
        is_alias: bool,
        span: Option<SourceSpan>,
    ) -> Self {
        Self {
            ident_id,
            is_alias,
            span,
        }
    }
}

// Is this good though? It DOES encode the meaning we want, but it's also pretty transient, but it
// also would be fairly inconvenient to make another system that does what the tracker does just
// because it changes a trait. It's not like this is an operator overload it's just defining how
// this struct should be hashed.

// `is_alias` and `path_span` are metadata for reporting, so only the identifier is checked for
// equality and hashing. This is done to let `DuplicateTracker<T>` hash every colliding identifier
// together, whether it came from a file name or an alias.
impl PartialEq for ModuleIdent {
    fn eq(&self, other: &Self) -> bool {
        self.ident_id == other.ident_id
    }
}

impl std::hash::Hash for ModuleIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident_id.hash(state);
    }
}

/// Imports used by user and compiler generated modules
#[derive(Debug, Clone)]
pub struct Import {
    pub name_id: InternedIdUTF8,
    pub kind: ImportKind,
    pub sp_alias_id: Option<SpannedContainer<InternedIdUTF8>>,
}

impl Import {
    pub const fn new(
        name_id: InternedIdUTF8,
        kind: ImportKind,
        sp_alias_id: Option<SpannedContainer<InternedIdUTF8>>,
    ) -> Import {
        Import {
            name_id,
            kind,
            sp_alias_id,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ImportKind {
    /// Import that is from a source file and fully resolved.
    /// Contains it's spanned path and module id
    Source(SpannedContainer<SourceId>, ModuleId),
    /// Import from a source file that has a path attached to it, but no module id yet
    UnresolvedSource(SpannedContainer<SourceId>),
    /// Import from source that had an unrecoverable error occur.
    /// This means the import should NOT be touched in any resolution scenario, unless for
    /// reporting or storing metadata.
    ErrorSource(SpannedContainer<SourceId>),
    /// Core module originated import
    Core(ModuleId),
}

#[derive(Debug, Default, Clone)]
pub struct Bind {
    pub path_id: SourceId,
    pub path_span: SourceSpan,
}

impl Bind {
    pub const fn new(path_id: SourceId, path_span: SourceSpan) -> Bind {
        Bind { path_id, path_span }
    }
}
