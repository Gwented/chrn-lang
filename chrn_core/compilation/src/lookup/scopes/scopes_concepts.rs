use chrn_utils::id_types::{ModuleId, ScopeId, SymbolId};
use lang::{
    chrn_classifier::{ChrnClassifiable, ChrnClassified},
    types::builtins::BuiltinTypeKind,
};

use crate::semantic::hir::{hir_concepts::Table, hir_symbols::SymbolKindFlat};

//TODO: Maybe this is the point where the scope wrapper comes in
/// Structure that holds scope data
#[derive(Debug)]
pub struct ScopeInfo {
    pub scope: Scope,
    /// For debugging purposes so that the symbol of origin is known for where a namespace lookup
    /// occured, beyond just the module or scope of origin.
    pub sym_owner: Option<SymbolId>,
    pub mod_owner: ModuleId,
}

impl ScopeInfo {
    pub fn new(scope: Scope, sym_owner: Option<SymbolId>, mod_owner: ModuleId) -> ScopeInfo {
        ScopeInfo {
            scope,
            sym_owner,
            mod_owner,
        }
    }
}

pub const SCOPE_CORE: u8 = 1 << 0;
pub const SCOPE_NEUTRAL: u8 = 1 << 1;
pub const SCOPE_VAR: u8 = 1 << 2;
pub const SCOPE_NEST: u8 = 1 << 3;
pub const SCOPE_COMPLEX: u8 = 1 << 4;
pub const SCOPE_OVERRIDE: u8 = 1 << 5;
pub const SCOPE_LOCAL: u8 = 1 << 6;
pub const SCOPE_COMPILER: u8 = 1 << 7;

//NOTE: These must always end with Core so that the "- 1" semantics work when `NamespaceOnly` is picked.
//Question mark

pub static SCOPE_CORE_ENCODED_SCOPES: [ScopeType; 1] = [ScopeType::Core];

/// Elements ordered to fit the languages rules of section `neutral`
pub static SCOPE_NEUTRAL_ENCODED_SCOPES: [ScopeType; 3] =
    [ScopeType::Neutral, ScopeType::Compiler, ScopeType::Core];

/// Elements ordered to fit the languages rules of section `var`
pub static SCOPE_VAR_ENCODED_SCOPES: [ScopeType; 4] = [
    ScopeType::Nest,
    ScopeType::Neutral,
    ScopeType::Compiler,
    ScopeType::Core,
];

/// Elements ordered to fit the languages rules of section `nest`
pub static SCOPE_NEST_ENCODED_SCOPES: [ScopeType; 5] = [
    ScopeType::Var,
    ScopeType::Nest,
    ScopeType::Neutral,
    ScopeType::Compiler,
    ScopeType::Core,
];

// Doesn't have itself because complex assigns properties for types, nothing more.
/// Elements ordered to fit the needs of scope `complex`
pub static SCOPE_COMPLEX_ENCODED_SCOPES: [ScopeType; 6] = [
    ScopeType::Var,
    ScopeType::Nest,
    ScopeType::Neutral,
    ScopeType::Compiler,
    ScopeType::Complex,
    ScopeType::Core,
];

// /// Elements ordered to fit the needs of scope `override`
// pub static SCOPE_OVERRIDE_ENCODED_SCOPES: [ScopeType; 6] = [
//     ScopeType::Override,
//     ScopeType::Var,
//     ScopeType::Nest,
//     ScopeType::Neutral,
//     ScopeType::Compiler,
//     ScopeType::Core,
// ];

//WARN: Suspicious accessibility
pub static SCOPE_LOCAL_ENCODED_SCOPES: [ScopeType; 1] = [ScopeType::Local];
pub static SCOPE_VAR_ONLY: [ScopeType; 1] = [ScopeType::Var];
pub static SCOPE_NEST_ONLY: [ScopeType; 1] = [ScopeType::Nest];

// Neutral, var, nest, and complex scopes can only access variables from neutral and nest.
// Override is unsure
#[derive(Debug)]
pub struct Scope {
    pub table: Table,
    /// Own `ScopeId`
    pub self_id: ScopeId,
    /// `ScopeType` this scope represents
    pub scope_type: ScopeType,
    /// An `Option` scope that is intrinsically a part of this scope
    pub intrinsic_scope: Option<ScopeId>,
    /// Boolean of whether or not the scope is intrinsic
    pub is_intrinsic: bool,
    /// Pre-determined list of `SceopType`s that this scope can access.
    pub accessible_scopes: &'static [ScopeType],
}

impl Scope {
    pub(crate) fn new(
        self_id: ScopeId,
        scope_type: ScopeType,
        is_intrinsic: bool,
        intrinsic_scope: Option<ScopeId>,
    ) -> Scope {
        let accessible_scopes = scope_type.accessible_scopes();
        Scope {
            table: Table::new(),
            self_id,
            scope_type,
            intrinsic_scope,
            accessible_scopes,
            is_intrinsic,
            // pub visible_scopes: Vec<ScopeId>,
        }
    }

    pub(crate) fn with_table(
        self_id: ScopeId,
        scope_type: ScopeType,
        intrinsic_scope: Option<ScopeId>,
        is_intrinsic: bool,
        table: Table,
    ) -> Scope {
        let accessible_scopes = scope_type.accessible_scopes();
        Scope {
            table,
            self_id,
            scope_type,
            is_intrinsic,
            intrinsic_scope,
            accessible_scopes,
        }
    }
}

#[derive(Debug)]
pub struct SymbolLookupOutput {
    /// `SymbolId` of found symbol
    pub found_sym_id: SymbolId,
    /// `ScopeId` the found symbol was found in
    pub scope_found_in: ScopeId,
}

impl SymbolLookupOutput {
    pub fn new(found_sym_id: SymbolId, scope_found_in: ScopeId) -> SymbolLookupOutput {
        SymbolLookupOutput {
            found_sym_id,
            scope_found_in,
        }
    }
}

/// Enum representing all kinds of scopes usable in chrn
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ScopeType {
    Compiler,
    Core,
    Local,
    Neutral,
    Var,
    Nest,
    Complex,
    // Override,
}

impl ScopeType {
    /// Direct representation of how the language views scope accessibility.
    /// `needs_global` purely exists for all scope accessibility reasons
    pub fn accessible_scopes(self) -> &'static [ScopeType] {
        match self {
            ScopeType::Core => &SCOPE_CORE_ENCODED_SCOPES,
            // Mainly for internal usage, not an actual program recognizable scope
            // Neutral can only access neutral because this section is purely for declaring and
            // using in other sections
            ScopeType::Neutral => &SCOPE_NEUTRAL_ENCODED_SCOPES,
            ScopeType::Var => &SCOPE_VAR_ENCODED_SCOPES,
            // ScopeType::Override => &SCOPE_OVERRIDE_ENCODED_SCOPES,
            ScopeType::Nest => &SCOPE_NEST_ENCODED_SCOPES,
            ScopeType::Complex => &SCOPE_COMPLEX_ENCODED_SCOPES,
            ScopeType::Local => &SCOPE_LOCAL_ENCODED_SCOPES,
            // Should be a recognized builtin at this point
            ScopeType::Compiler => &[],
        }
    }

    pub(crate) fn to_u8(self) -> u8 {
        match self {
            ScopeType::Core => SCOPE_CORE,
            ScopeType::Neutral => SCOPE_NEUTRAL,
            ScopeType::Var => SCOPE_VAR,
            ScopeType::Nest => SCOPE_NEST,
            ScopeType::Complex => SCOPE_COMPLEX,
            // ScopeType::Override => SCOPE_OVERRIDE,
            ScopeType::Local => SCOPE_LOCAL,
            ScopeType::Compiler => SCOPE_COMPILER,
        }
    }

    pub(crate) fn has_intrinsic_scope(self) -> bool {
        match self {
            // ScopeType::Override => true,
            ScopeType::Complex  => true,
            ScopeType::Core
            | ScopeType::Local
            | ScopeType::Neutral
            | ScopeType::Nest
            // I don't know does it?
            | ScopeType::Compiler
            | ScopeType::Var => false,
        }
    }
}

// Maybe Only(ScopeType)
/// This enum is intended to disallow core defined values from being searched for when syntax such
/// as "main::i32" is used. i32 is not owned by main, but innately main is attached to core, meaning
/// without the explicit noting of whether we are searching a singular module's namespace it would
/// innately allow for main.i32 to be interpreted the same as if just i32 was written, which is
/// wrong since the namespace "main" owns no such thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeLookupPattern {
    /// Applies no restriction to lookups. Meaning, core is automatically searched since it's
    /// intrinsic, any scope's accessible scopes can be searched with no restriction.
    NoRestrictions,
    // WHat
    /// Restricts lookup to only search what is within the given namespace, which restricts modules
    /// such as core, or anything not declared within the symbol's scope containment?
    NamespaceOnly,
    /// Lookup that only allows `nest` to be searched and enforces it's the only section
    /// that can be searched
    OnlyNest,
    /// Lookup that only allows `var` to be searched and enforces it's the only section
    /// that can be searched
    OnlyVar,
}

// TODO: Formattable
impl std::fmt::Display for ScopeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopeType::Core => write!(f, "core"),
            ScopeType::Neutral => write!(f, "neutral"),
            ScopeType::Var => write!(f, "var"),
            ScopeType::Nest => write!(f, "nest"),
            ScopeType::Complex => write!(f, "complex"),
            ScopeType::Local => write!(f, "local"),
            ScopeType::Compiler => write!(f, "compiler"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Scope type specifically for if a symbol has an associated scope tied to it
pub enum AssociatedScopeKind {
    // A bit redundant since odules already hold themselves as a scope
    /// Meaning the scope is inside of a module's vector of `ScopeId`
    Module(ModuleId),
    /// Meaning the scope is just attached to a symbol's namespace
    Scope(ScopeId),
}

impl AssociatedScopeKind {
    pub fn is_module(self) -> bool {
        matches!(self, AssociatedScopeKind::Module(_))
    }
}

impl ChrnClassifiable for AssociatedScopeKind {
    fn to_classified(&self) -> lang::chrn_classifier::ChrnClassified {
        match self {
            AssociatedScopeKind::Module(_) => ChrnClassified::Module,
            AssociatedScopeKind::Scope(_) => ChrnClassified::Namespace,
        }
    }
}

pub struct IntrinsicRegistry {
    pub core_mod_id: ModuleId,
    pub complex_scope_id: Option<ScopeId>,
    // TEST:
    pub builtin_repo: Option<BuiltinIdRepository>,
}

impl IntrinsicRegistry {
    pub fn new(
        core_mod_id: ModuleId,
        complex_scope_id: Option<ScopeId>,
        builtin_repo: Option<BuiltinIdRepository>,
    ) -> IntrinsicRegistry {
        IntrinsicRegistry {
            core_mod_id,
            complex_scope_id,
            builtin_repo,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinIdRepository {
    pub i8: SymbolId,
    pub u8: SymbolId,
    pub i16: SymbolId,
    pub u16: SymbolId,
    pub f16: SymbolId,
    pub i32: SymbolId,
    pub u32: SymbolId,
    pub f32: SymbolId,
    pub i64: SymbolId,
    pub u64: SymbolId,
    pub f64: SymbolId,
    pub i128: SymbolId,
    pub u128: SymbolId,
    pub f128: SymbolId,
    pub sized: SymbolId,
    pub r#unsized: SymbolId,
    pub str: SymbolId,
    pub char: SymbolId,
    pub nil: SymbolId,
    pub bool: SymbolId,
    pub bigint: SymbolId,
    pub bigfloat: SymbolId,
    pub runtime: SymbolId,
}

impl BuiltinIdRepository {
    pub fn builder() -> BuiltinIdRepositoryBuilder {
        BuiltinIdRepositoryBuilder::new()
    }

    pub fn get(&self, kind: BuiltinTypeKind) -> Option<SymbolId> {
        match kind {
            BuiltinTypeKind::I8 => Some(self.i8),
            BuiltinTypeKind::U8 => Some(self.u8),
            BuiltinTypeKind::I16 => Some(self.i16),
            BuiltinTypeKind::U16 => Some(self.u16),
            BuiltinTypeKind::F16 => Some(self.f16),
            BuiltinTypeKind::I32 => Some(self.i32),
            BuiltinTypeKind::U32 => Some(self.u32),
            BuiltinTypeKind::F32 => Some(self.f32),
            BuiltinTypeKind::I64 => Some(self.i64),
            BuiltinTypeKind::U64 => Some(self.u64),
            BuiltinTypeKind::F64 => Some(self.f64),
            BuiltinTypeKind::I128 => Some(self.i128),
            BuiltinTypeKind::U128 => Some(self.u128),
            BuiltinTypeKind::F128 => Some(self.f128),
            BuiltinTypeKind::Sized => Some(self.sized),
            BuiltinTypeKind::Unsized => Some(self.r#unsized),
            BuiltinTypeKind::Str => Some(self.str),
            BuiltinTypeKind::Char => Some(self.char),
            BuiltinTypeKind::Nil => Some(self.nil),
            BuiltinTypeKind::Bool => Some(self.bool),
            BuiltinTypeKind::BigInt => Some(self.bigint),
            BuiltinTypeKind::BigFloat => Some(self.bigfloat),
            BuiltinTypeKind::Runtime => Some(self.runtime),
            BuiltinTypeKind::List
            | BuiltinTypeKind::Set
            | BuiltinTypeKind::Map
            | BuiltinTypeKind::Tuple => None,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BuiltinIdRepositoryBuilder {
    pub i8: Option<SymbolId>,
    pub u8: Option<SymbolId>,
    pub i16: Option<SymbolId>,
    pub u16: Option<SymbolId>,
    pub f16: Option<SymbolId>,
    pub i32: Option<SymbolId>,
    pub u32: Option<SymbolId>,
    pub f32: Option<SymbolId>,
    pub i64: Option<SymbolId>,
    pub u64: Option<SymbolId>,
    pub f64: Option<SymbolId>,
    pub i128: Option<SymbolId>,
    pub u128: Option<SymbolId>,
    pub f128: Option<SymbolId>,
    pub sized: Option<SymbolId>,
    pub unsized_: Option<SymbolId>,
    pub str: Option<SymbolId>,
    pub char: Option<SymbolId>,
    pub nil: Option<SymbolId>,
    pub bool: Option<SymbolId>,
    pub bigint: Option<SymbolId>,
    pub bigfloat: Option<SymbolId>,
    pub runtime: Option<SymbolId>,
}

impl BuiltinIdRepositoryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, kind: BuiltinTypeKind, id: SymbolId) {
        let slot = match kind {
            BuiltinTypeKind::I8 => &mut self.i8,
            BuiltinTypeKind::U8 => &mut self.u8,
            BuiltinTypeKind::I16 => &mut self.i16,
            BuiltinTypeKind::U16 => &mut self.u16,
            BuiltinTypeKind::F16 => &mut self.f16,
            BuiltinTypeKind::I32 => &mut self.i32,
            BuiltinTypeKind::U32 => &mut self.u32,
            BuiltinTypeKind::F32 => &mut self.f32,
            BuiltinTypeKind::I64 => &mut self.i64,
            BuiltinTypeKind::U64 => &mut self.u64,
            BuiltinTypeKind::F64 => &mut self.f64,
            BuiltinTypeKind::I128 => &mut self.i128,
            BuiltinTypeKind::U128 => &mut self.u128,
            BuiltinTypeKind::F128 => &mut self.f128,
            BuiltinTypeKind::Sized => &mut self.sized,
            BuiltinTypeKind::Unsized => &mut self.unsized_,
            BuiltinTypeKind::Str => &mut self.str,
            BuiltinTypeKind::Char => &mut self.char,
            BuiltinTypeKind::Nil => &mut self.nil,
            BuiltinTypeKind::Bool => &mut self.bool,
            BuiltinTypeKind::BigInt => &mut self.bigint,
            BuiltinTypeKind::BigFloat => &mut self.bigfloat,
            BuiltinTypeKind::Runtime => &mut self.runtime,
            //TODO: Maybe unspecial case this
            BuiltinTypeKind::List
            | BuiltinTypeKind::Set
            | BuiltinTypeKind::Map
            | BuiltinTypeKind::Tuple => {
                panic!("`BuiltinIdRepositoryBuilder` cannot store compound builtin type `{kind:?}`")
            }
        };
        debug_assert!(
            slot.is_none(),
            "symbol id for builtin kind `{:?}` was already set in `BuiltinIdRepositoryBuilder`",
            kind
        );
        *slot = Some(id);
    }

    pub fn build(self) -> BuiltinIdRepository {
        BuiltinIdRepository {
            i8: self.i8.expect("i8 must be set in BuiltinIdRepository"),
            u8: self.u8.expect("u8 must be set in BuiltinIdRepository"),
            i16: self.i16.expect("i16 must be set in BuiltinIdRepository"),
            u16: self.u16.expect("u16 must be set in BuiltinIdRepository"),
            f16: self.f16.expect("f16 must be set in BuiltinIdRepository"),
            i32: self.i32.expect("i32 must be set in BuiltinIdRepository"),
            u32: self.u32.expect("u32 must be set in BuiltinIdRepository"),
            f32: self.f32.expect("f32 must be set in BuiltinIdRepository"),
            i64: self.i64.expect("i64 must be set in BuiltinIdRepository"),
            u64: self.u64.expect("u64 must be set in BuiltinIdRepository"),
            f64: self.f64.expect("f64 must be set in BuiltinIdRepository"),
            i128: self.i128.expect("i128 must be set in BuiltinIdRepository"),
            u128: self.u128.expect("u128 must be set in BuiltinIdRepository"),
            f128: self.f128.expect("f128 must be set in BuiltinIdRepository"),
            sized: self
                .sized
                .expect("sized must be set in BuiltinIdRepository"),
            r#unsized: self
                .unsized_
                .expect("unsized must be set in BuiltinIdRepository"),
            str: self.str.expect("str must be set in BuiltinIdRepository"),
            char: self.char.expect("char must be set in BuiltinIdRepository"),
            nil: self.nil.expect("nil must be set in BuiltinIdRepository"),
            bool: self.bool.expect("bool must be set in BuiltinIdRepository"),
            bigint: self
                .bigint
                .expect("bigint must be set in BuiltinIdRepository"),
            bigfloat: self
                .bigfloat
                .expect("bigfloat must be set in BuiltinIdRepository"),
            runtime: self
                .runtime
                .expect("runtime must be set in BuiltinIdRepository"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ScopeLookupPreferenceFlags {
    /// If `None`, no preference is accounted for meaning all preference checks are `true`
    flags: Option<u16>,
}

// This is more like a general purpose set of flags since it doesn't really matter if flat kinds are
// used or not
impl ScopeLookupPreferenceFlags {
    pub const TYPE: u16 = 1 << 0;
    pub const VARIABLE: u16 = 1 << 1;
    pub const NAMESPACE: u16 = 1 << 2;
    pub const DIRECTIVE: u16 = 1 << 3;
    pub const EXTERN_TYPE: u16 = 1 << 4;

    pub fn new(flags: Option<u16>) -> Self {
        Self { flags }
    }

    /// Creates `ScopeLookupPreferenceFlags` with only flag `NAMESPACE`
    pub fn new_namespace() -> Self {
        Self {
            flags: Some(Self::NAMESPACE),
        }
    }

    /// Creates lookup preference with no preferred options
    pub fn none() -> ScopeLookupPreferenceFlags {
        ScopeLookupPreferenceFlags::new(None)
    }

    pub fn is_none(self) -> bool {
        self.flags.is_none()
    }

    /// Checks if the `SymbolKindFlat` converted to a valid set of bits for `LookupPreferenceFlags`
    /// is contained within `self`
    pub fn is_preferred(self, kind: SymbolKindFlat) -> bool {
        if let Some(flags) = self.flags {
            flags & flat_sym_kind_to_preferred_bits(kind) != 0
        } else {
            // No options chosen. Anything attempted to be matched to a `None` preference succeeds.
            true
        }
    }
}

/// Local function to turn `SymbolKindFlat` into a preferred option.
/// This exists because the `to_bits()` from flat symbols are just direct mappings, meaning there is
/// no signifying bit usable to say "No options selected", hence the explicit translation layer here.
pub(super) const fn flat_sym_kind_to_preferred_bits(kind: SymbolKindFlat) -> u16 {
    match kind {
        SymbolKindFlat::Type => ScopeLookupPreferenceFlags::TYPE,
        SymbolKindFlat::Variable => ScopeLookupPreferenceFlags::VARIABLE,
        SymbolKindFlat::Namespace => ScopeLookupPreferenceFlags::NAMESPACE,
        SymbolKindFlat::Directive => ScopeLookupPreferenceFlags::DIRECTIVE,
        SymbolKindFlat::ExternType => ScopeLookupPreferenceFlags::EXTERN_TYPE,
    }
}
