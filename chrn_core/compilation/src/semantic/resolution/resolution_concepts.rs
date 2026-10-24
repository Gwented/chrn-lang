// Not sure about splitting this because then it just looks like, "resolve_directive, resolve_type_expr"
// as the literal module name, despite them just being free functions. Maybe just a "concepts" separation.

use chrn_utils::{
    id_types::{ModuleId, SymbolId, TypeId, InternedIdUTF8},
    source_map::source_span::SourceSpan,
    utils::containers::SpannedContainer,
};

use crate::lookup::scopes::scopes_concepts::AssociatedScopeKind;

//TODO: As of right now this is basically entirely hinging off of being usable for reporting, which
//is probably not the most concise design but it works for now so may not need changing.
/// Result type for type expr resolution attempts. This exists due to the fact that there is no `Ok` or `Err`
/// inherit concept behind whether or not something was found.
#[derive(Debug)]
pub enum TypeExprResult {
    /// Found a type with no issues
    Type(TypeId),
    // smybol
    /// Found a smybol but it wasn't a type
    NotAType {
        /// `SymbolId` of `found_sym_id`
        found_sym_id: SymbolId,
        /// Spanned identifier of symbol that is not a type
        sp_name_id: SpannedContainer<InternedIdUTF8>,
        scope_found_in: AssociatedScopeKind,
    },
    /// (Identifier not found as any symbol, scope searched)
    SymbolNotFound(SpannedContainer<InternedIdUTF8>, AssociatedScopeKind),
    /// Symbol found but private to another module
    PrivateTypeAccess {
        sp_found_type_id: SpannedContainer<TypeId>,
        found_sym_id: SymbolId,
        current_mod_id: ModuleId,
    },
    /// Found a valid data structure but the inputs exceed the expected
    InvalidGenericArgCount {
        base: InternedIdUTF8,
        expected: usize,
        inputs_span: SourceSpan,
    },
    /// Found an identifier using generic parameters after it while not being a known data structure
    UnknownGenericIdent(SpannedContainer<InternedIdUTF8>),
    /// Static access variant
    StaticAccessFailure(StaticAccessResult),
}

impl TypeExprResult {
    // pub fn type_id(&self) -> Option<TypeId> {
    //     //TODO: Maybe give the Type variant the output as well as any others that it may be relevant
    //     //for
    //     match self {
    //         TypeExprResult::Type(type_id) => Some(*type_id),
    //         TypeExprResult::NotAType { .. }
    //         | TypeExprResult::PrivateTypeAccess { .. }
    //         | TypeExprResult::InvalidGenericArgCount { .. }
    //         | TypeExprResult::SymbolNotFound(_, _)
    //         | TypeExprResult::UnknownGenericIdent(_)
    //         | TypeExprResult::StaticAccessFailure(_) => None,
    //     }
    // }
}

/// Result type for static access resolution attempts. This exists due to the fact that there is no
/// `Ok` or `Err` inherit concept behind whether or not something was found
#[derive(Debug)]
pub enum StaticAccessResult {
    /// Scope found with no issues
    Scope(AssociatedScopeKind),
    /// If `prev_seg` is `None` then `current_seg` was a symbol that wasn't found.
    /// If it's `None` then then `current_seg` was found by going through `prev_seg`
    // Should be different variants
    SymNotFound {
        current_seg: SpannedContainer<InternedIdUTF8>,
        prev_seg: Option<SpannedContainer<InternedIdUTF8>>,
    },
    /// A segment was found but does not expose a further namespace
    NoNamespace(SpannedContainer<InternedIdUTF8>),
    /// A generic found using "::" access
    /// (Generic span)
    GenericUsingStaticPath(SourceSpan),
    //// The parser cannot process a generic inside of exprs
    // GenericInExpr(SourceSpan),
}

impl StaticAccessResult {
    /// Tries to get associated scope out of result
    pub fn associated_scope(&self) -> Option<AssociatedScopeKind> {
        match self {
            StaticAccessResult::Scope(associated_scope) => Some(*associated_scope),
            StaticAccessResult::SymNotFound { .. }
            | StaticAccessResult::NoNamespace(_)
            | StaticAccessResult::GenericUsingStaticPath(_) => None,
        }
    }
}

// -- ACKNOWLEDGE THIS --
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum StaticAccessOption {
    None,
    Type,
    Val,
}

// We can come up with a better name for this..
// Not sure what this should @$)#*$
#[derive(Debug)]
pub enum AmbiguousAccessOutput {
    /// Reached end of access and obtained a symbol (Which could be a type)
    Symbol(SymbolId),
}
