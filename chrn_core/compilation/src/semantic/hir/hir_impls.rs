use chrn_utils::{
    id_types::{
        AstId, ConfigRootId, ExprId, ImplId, ImplMemberId, InternedId, MemberId, SymbolId, TypeId,
        id_tags::TaggedId,
    },
    source_map::source_span::SourceSpan,
    utils::containers::SpannedContainer,
};

use crate::{
    id_tag_decls::{
        ConfigMemberTag, ConfigRootTag, ExternTypeTag, MultiTypeAssignmentTag,
        OptionAssignmentMemberTag, OptionAssignmentRootTag,
    },
    lookup::scopes::scopes_concepts::{ScopeLookupPattern, ScopeType},
};

#[derive(Debug)]
pub struct ImplHir {
    pub impl_id: ImplId,
    pub kind: ImplHirKind,
    pub scope_origin: ScopeType,
    pub ast_id: Option<AstId>,
}

impl ImplHir {
    pub fn new(
        impl_id: ImplId,
        kind: ImplHirKind,
        scope_origin: ScopeType,
        ast_id: Option<AstId>,
    ) -> Self {
        Self {
            impl_id,
            kind,
            scope_origin,
            ast_id,
        }
    }
}

#[derive(Debug)]
pub enum ImplHirKind {
    Config(ConfigRootId),
}

#[derive(Debug)]
pub enum ImplMemberKind {
    /// `ConfigMember`
    ConfigMember(ConfigMember),
    /// Root specific option assignment
    OptAssignmentRoot(OptionAssignmentRoot),
    /// Member specific option assignment
    OptAssignmentMember(OptionAssignmentMember),
    /// Takes lhs types, and assigns to a single rhs `SymbolKind::ExternType`
    MultiTypeAssignment(MultiTypeAssignment),
    /// Member that has reserved a slot but not yet defined
    Unknown {
        sp_name_id: SpannedContainer<InternedId>,
        reserved_memb_id: ImplMemberId,
    },
}

impl ImplMemberKind {
    pub fn is_unknown(&self) -> bool {
        match self {
            ImplMemberKind::Unknown { .. } => true,
            _ => false,
        }
    }
}

/// Common inner of config roots
#[derive(Debug)]
pub struct ConfigRootCommon {
    /// `ImplId` of `self`
    pub impl_id: TaggedId<ImplId, ConfigRootTag>,
    /// ConfigId of `self`
    pub cfg_root_id: ConfigRootId,
    /// Lookup pattern that needs to be used to properly discern if
    /// `ScopeLookupPattern::Namespace/OnlyVar` should be used to search for the symbol associated with
    /// thie config
    pub lookup_pat: ScopeLookupPattern,
    /// `ConfigMembers` held by `self`
    pub cfg_membs: Vec<TaggedId<ImplMemberId, ConfigMemberTag>>,
}

impl ConfigRootCommon {
    pub fn new(
        impl_id: TaggedId<ImplId, ConfigRootTag>,
        cfg_root_id: ConfigRootId,
        lookup_pat: ScopeLookupPattern,
        // kind: ConfigRootKindFlat,
        cfg_membs: Vec<TaggedId<ImplMemberId, ConfigMemberTag>>,
    ) -> Self {
        Self {
            impl_id,
            cfg_root_id,
            lookup_pat,
            cfg_membs,
        }
    }
}

// TODO: Readiness for skipping during resolution
/// Intended to represent a config block environment that consumes options for a field.
/// Specifically tied to `overrid` section semantics
#[derive(Debug)]
pub struct ConfigRoot {
    pub common: ConfigRootCommon,
    /// During name resolution, we can't actually lookup the symbol since it may or may not be
    /// registered, so it's Option since it actually is `None` at some point, and could remain
    /// `None` if in a later stage it doesn't have it's target symbol found.
    ///
    /// Must be `Namespace` or `Type`
    // Wrap in `Checked`?
    pub linked_sym_id: Option<SymbolId>,
    /// Expects: Option assignments. No other statements can be made at root.
    pub stmts: Vec<ImplMemberId>,
    pub kind: ConfigRootKind,
}

impl ConfigRoot {
    pub fn new(
        common: ConfigRootCommon,
        linked_sym_id: Option<SymbolId>,
        stmts: Vec<ImplMemberId>,
        kind: ConfigRootKind,
    ) -> ConfigRoot {
        ConfigRoot {
            common,
            linked_sym_id,
            stmts,
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigRootKind {
    Override,
    Complex,
}

#[derive(Debug)]
pub struct ConfigMemberCommon {
    /// Is a name id instead of symbol id since `NameResolver` merely registers names, with no
    /// knowledge of symbol specifics. A dependency system may be used in the future.
    pub name_id: InternedId,
    // This is not a `SpannedContainer` because it may become an Option
    pub name_span: SourceSpan,
    /// `ImplMemberId` of `self`
    pub impl_memb_id: TaggedId<ImplMemberId, ConfigMemberTag>,
}

impl ConfigMemberCommon {
    pub fn new(
        name_id: InternedId,
        name_span: SourceSpan,
        impl_memb_id: TaggedId<ImplMemberId, ConfigMemberTag>,
    ) -> Self {
        Self {
            name_id,
            name_span,
            impl_memb_id,
        }
    }
}

/// The member inside of a `ConfigDef` or `ConfigMember` which is the same structure,
/// but with ties to an `ImplMemberKind` instead of a `ImplHir`
#[derive(Debug)]
pub struct ConfigMember {
    pub common: ConfigMemberCommon,
    /// complex expects: Option assignments
    /// override expects: MultiTypeAssignment
    pub stmts: Vec<ImplMemberId>,
    // These configs are supposed to be usable by override too so maybe this becomes an enum where
    // it exposes metadata depending on override or not.
    pub meta: ConfigMemberMetadataKind,
    //NOTE: Members use `ScopeLookupPattern::NamespaceOnly` only but this is kept here for now
    //because it may be used in the future (was used in the past)
    //
    /// Lookup pattern that needs to be used to properly discern if
    /// `ScopeLookupPattern::Namespace/OnlyVar` should be used to search for the member associacted with
    /// this config member
    pub lookup_pat: ScopeLookupPattern,
    /// `ConfigMember`s held
    pub cfg_members: Vec<TaggedId<ImplMemberId, ConfigMemberTag>>,
}

impl ConfigMember {
    pub fn new(
        common: ConfigMemberCommon,
        meta: ConfigMemberMetadataKind,
        lookup_pat: ScopeLookupPattern,
        stmts: Vec<ImplMemberId>,
        cfg_members: Vec<TaggedId<ImplMemberId, ConfigMemberTag>>,
    ) -> ConfigMember {
        ConfigMember {
            common,
            meta,
            stmts,
            lookup_pat,
            cfg_members,
        }
    }
}

// Maybe embed this into lookup instead?
#[derive(Debug, Copy, Clone)]
pub enum ConfigRootMetadataKind {
    Complex,
    Override,
}

#[derive(Debug)]
pub enum ConfigMemberMetadataKind {
    Complex(ConfigMemberComplexMetadata),
    Override(ConfigMemberOverrideMetadata),
}

#[derive(Debug)]
pub struct ConfigMemberComplexMetadata {
    /// `MemberId` of the member symbol this is attached to
    pub linked_memb_id: MemberId,
    // This is mostly here because the padding is going to make it 80 bytes anyways so why not store
    // the type to avoid extra lookups
    /// `TypeId` of the member symbol this is attached to
    /// This is `Option` because a type like a variant doesn't have a type, so this is not
    /// guaranteed
    pub linked_memb_type_id: Option<TypeId>,
}

impl ConfigMemberComplexMetadata {
    pub fn new(linked_memb_id: MemberId, linked_memb_type_id: Option<TypeId>) -> Self {
        Self {
            linked_memb_id,
            linked_memb_type_id,
        }
    }
}

#[derive(Debug)]
pub struct ConfigMemberOverrideMetadata {
    pub linked: LinkedConfigOverrideMemberKind,
}

impl ConfigMemberOverrideMetadata {
    pub const fn new(linked: LinkedConfigOverrideMemberKind) -> Self {
        Self { linked }
    }
}

// This is getting. A bit. Just a little, concerning.
// May or may not make this actually used by impl members
/// All valid states for `override` links
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkedConfigOverrideMemberKind {
    /// if it's a global override usage like "override C {}" not being linked
    /// would just be another state
    Global,
    /// A `complex` type at the root being given an override means that it's overrides need to
    /// account for a `SymbolId`
    Root(SymbolId),
    /// A `complex` type member being given an override means it needs to restrict itself to
    /// targetting a `MemberSymbolKind`
    Member(MemberId),
}

// Would be:
// Person {
//      identifiers = "person" <--- This is a root opt
//      name {
//          default_val = 3 <--- This is a member opt
//      }
// }
/// Represents options and their values assigned by the user in a root config
#[derive(Debug)]
pub struct OptionAssignmentRoot {
    /// `ImplId` of the `ConfigRoot` this is applied to
    pub parent_impl_id: TaggedId<ImplId, ConfigRootTag>,
    /// `ImplMemberId` of `self`
    pub impl_memb_id: TaggedId<ImplMemberId, OptionAssignmentRootTag>,
    /// Identifier of option
    pub name_id: InternedId,
    /// Span of identifier
    pub name_span: SourceSpan,
    /// Values defined for this option
    pub array_expr_id: ExprId,
}

impl OptionAssignmentRoot {
    pub const fn new(
        parent_impl_id: TaggedId<ImplId, ConfigRootTag>,
        impl_memb_id: TaggedId<ImplMemberId, OptionAssignmentRootTag>,
        name_id: InternedId,
        name_span: SourceSpan,
        array_expr_id: ExprId,
    ) -> OptionAssignmentRoot {
        OptionAssignmentRoot {
            parent_impl_id,
            impl_memb_id,
            name_id,
            name_span,
            array_expr_id,
        }
    }
}

//TODO: Maybe make this OptionAssignment<SymbolId/MemberId> where parent member id is that
// Would be:
// Person {
//      .identifiers = "person" <--- This is a root opt
//      name {
//          .default_val = 3 <--- This is a member opt
//      }
// }
/// Represents options and their values assigned by the user inside of a config member
#[derive(Debug)]
pub struct OptionAssignmentMember {
    // /// `ImplMemberId`
    // pub parent_impl_memb_id: ImplMemberId,
    // FIX:
    /// `MemberId` of the `ConfigMember` it is derivative of
    pub parent_memb_id: MemberId,
    /// `ImplMemberId` of `self`
    pub impl_memb_id: TaggedId<ImplMemberId, OptionAssignmentMemberTag>,
    /// Identifier of option
    pub name_id: InternedId,
    /// Span of identifier
    pub name_span: SourceSpan,
    /// Values defined for this option
    pub array_expr_id: ExprId,
}

impl OptionAssignmentMember {
    pub const fn new(
        // parent_impl_memb_id: ImplMemberId,
        parent_memb_id: MemberId,
        impl_memb_id: TaggedId<ImplMemberId, OptionAssignmentMemberTag>,
        name_id: InternedId,
        name_span: SourceSpan,
        array_expr_id: ExprId,
    ) -> OptionAssignmentMember {
        OptionAssignmentMember {
            // parent_impl_memb_id,
            parent_memb_id,
            impl_memb_id,
            name_id,
            name_span,
            array_expr_id,
        }
    }
}

//TODO: We really need some sort of generic type enfrocement where, yes it is still just a symbol id
//but SymbolId<ExternType> is fully trust-worthy.
#[derive(Debug)]
pub struct MultiTypeAssignment {
    /// `ImplMemberId` of `self`
    pub impl_memb_id: TaggedId<ImplMemberId, MultiTypeAssignmentTag>,
    pub to_assign: Vec<TypeId>,
    // Maybe there will be `ExternTypeId` usage but not sure about that.
    /// `ExternType` to assign `to_assign` to
    pub assign_to: TaggedId<SymbolId, ExternTypeTag>,
}

impl MultiTypeAssignment {
    pub fn new(
        impl_memb_id: TaggedId<ImplMemberId, MultiTypeAssignmentTag>,
        to_assign: Vec<TypeId>,
        assign_to: TaggedId<SymbolId, ExternTypeTag>,
    ) -> Self {
        Self {
            impl_memb_id,
            to_assign,
            assign_to,
        }
    }
}
