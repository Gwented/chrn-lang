//! This module (em-dash) contains free functions that type check given a particular context.
// Maybe use some sort of candidate enum eventually where we can have some sort of general typecheck
// failure preset error, which takes in a sort of candidate or encoded expected information so that
// the dynamic help and notes can still be used with the engine and stoof.

pub mod typechecker_concepts;

use chrn_utils::{
    arena::Arena,
    id_types::{SymbolId, TypeId},
    loop_abort,
};
use lang::types::builtins::BuiltinType;

use crate::{
    lookup::scopes::scopes_concepts::AssociatedScopeKind,
    resolvers::typechecker::typechecker_concepts::{ExpectedKind, ExpectedKindType},
    script_compiler::ScriptCompiler,
    semantic::hir::{
        hir_concepts::{Type, TypeInfo},
        hir_symbols::SymbolKind,
    },
    walk_type_id_deferred,
};
//TODO: Typechecker helpers?

//TODO: data structure typecheck
pub fn is_same_ty(types: &Arena<TypeInfo, TypeId>, mut a_id: TypeId, mut b_id: TypeId) -> bool {
    // What if we had a macro that looped like a for loop? In C?
    for _ in 0..chrn_utils::MAX_LOOPS {
        let checked_a = walk_type_id_deferred!(types, a_id);
        let checked_b = walk_type_id_deferred!(types, b_id);

        let a_ty = &types[checked_a.inner].ty;
        let b_ty = &types[checked_b.inner].ty;

        //If both have the same id and aren't data structures then must be valid
        if checked_a.inner == checked_b.inner {
            //TODO: Format the err msg so that it displays the data structure and type.
            if let (Type::BuiltinTypeInfo(a_info), Type::BuiltinTypeInfo(b_info)) = (a_ty, b_ty) {
                match (&a_info.ty, &b_info.ty) {
                    //WARN: Recursion usage.
                    (BuiltinType::List(a_inner), BuiltinType::List(b_inner))
                    | (BuiltinType::Set(a_inner), BuiltinType::Set(b_inner)) => {
                        a_id = *a_inner;
                        b_id = *b_inner;
                    }
                    (BuiltinType::Tuple(a_tup), BuiltinType::Tuple(b_tup)) => {
                        if a_tup.len() != b_tup.len() {
                            return false;
                        }

                        for i in 0..a_tup.len() {
                            let a_elem = a_tup[i];
                            let b_elem = b_tup[i];
                            // Lost to recursion
                            if !is_same_ty(types, a_elem, b_elem) {
                                return false;
                            };
                        }
                        // Is this right?
                        return true;
                    }
                    (BuiltinType::Map(a_key, a_val), BuiltinType::Map(b_key, b_val)) => {
                        return is_same_ty(types, *a_key, *b_key)
                            && is_same_ty(types, *a_val, *b_val);
                    }
                    // If no inner is had by the given `BuiltinType` then it must be valid since
                    // `TypeId` aligns
                    _ => return true,
                }
            } else {
                // If !builtin && same type id then must be valid
                return true;
            }
        } else {
            // If they DON'T have the same id, they can ONLY be valid as data structures
            //WARN: Temp Code duplication
            if let (Type::BuiltinTypeInfo(a_info), Type::BuiltinTypeInfo(b_info)) = (a_ty, b_ty) {
                match (&a_info.ty, &b_info.ty) {
                    (BuiltinType::List(a_inner), BuiltinType::List(b_inner))
                    | (BuiltinType::Set(a_inner), BuiltinType::Set(b_inner)) => {
                        a_id = *a_inner;
                        b_id = *b_inner;
                    }
                    (BuiltinType::Tuple(a_tup), BuiltinType::Tuple(b_tup)) => {
                        if a_tup.len() != b_tup.len() {
                            return false;
                        }

                        for i in 0..a_tup.len() {
                            let a_elem = a_tup[i];
                            let b_elem = b_tup[i];
                            if !is_same_ty(types, a_elem, b_elem) {
                                return false;
                            };
                        }
                        return true;
                    }
                    (BuiltinType::Map(a_key, a_val), BuiltinType::Map(b_key, b_val)) => {
                        return is_same_ty(types, *a_key, *b_key)
                            && is_same_ty(types, *a_val, *b_val);
                    }
                    // Must be invalid since built-in type ids must match
                    _ => return false,
                }
            } else {
                // Must be a built-in for type ids not being the same to be a valid possibility
                return false;
            }
        }
    }
    loop_abort!()
}

/// Returns `true` if the expected `ExpectedKind` aligns with the given `SymbolId`
pub fn is_expected_sym<E>(compiler: &ScriptCompiler, expected: E, sym_id: SymbolId) -> bool
where
    E: Into<ExpectedKind>,
{
    match expected.into() {
        ExpectedKind::Symbol(expected_sym) => {
            let sym = &compiler.syms[sym_id];
            sym.kind.to_flat() == expected_sym
        }
        ExpectedKind::Type(expected) => {
            let Some(type_id) = compiler.get_type_id_from_sym_id(sym_id) else {
                return false;
            };
            is_expected_ty(compiler, expected, type_id)
        }
    }
}

/// Returns `true` if the expected `ExpectedKindType` aligns with the given `TypeId`.
/// Type version of `is_expected`
pub fn is_expected_ty<E>(compiler: &ScriptCompiler, expected: E, type_id: TypeId) -> bool
where
    E: Into<ExpectedKindType>,
{
    match expected.into() {
        ExpectedKindType::AnyBuiltin => compiler.check_builtin(type_id),
        ExpectedKindType::Kind(expected_ty) => Type::to_kind(compiler, type_id) == expected_ty,
    }
}

//What about just a general walk deferred function that prevents this same code from being written
//everywhere
/// Fields and variants encode the same type semantics so this is shared.
///
/// Returns `true` if the type is a valid field or variant candidate, `false` if invalid
pub fn check_field_or_variant(types: &Arena<TypeInfo, TypeId>, mut type_id: TypeId) -> bool {
    let checked = walk_type_id_deferred!(types, type_id);
    match &types[checked.inner].ty {
        Type::Struct(_) | Type::Enum(_) | Type::BuiltinTypeInfo(_) => true,
        Type::Unknown | Type::Boundaries(_) | Type::Alias(_) | Type::TypeDef(_) | Type::Func(_) => {
            false
        }
        Type::Deferred(_) => unreachable!(),
    }
}

/// Returns `true` if the type is a valid config root candidate, `false` if invalid
pub fn check_cfg_root(compiler: &ScriptCompiler, sym_id: SymbolId) -> bool {
    let sym = &compiler.syms[sym_id];
    match sym.kind {
        SymbolKind::Type(mut type_id) => {
            let checked = walk_type_id_deferred!(&compiler.types, type_id);
            match &compiler.types[checked.inner].ty {
                Type::TypeDef(_) | Type::Struct(_) | Type::Enum(_) => return true,
                Type::BuiltinTypeInfo(_)
                | Type::Unknown
                | Type::Boundaries(_)
                | Type::Func(_)
                | Type::Alias(_) => return false,
                Type::Deferred(_) => unreachable!(),
            }
        }
        // Can only accept scope, needs to prevent "for module {}" from being possible as a root
        SymbolKind::Namespace
            if matches!(sym.associated_scope, Some(AssociatedScopeKind::Scope(_))) =>
        {
            true
        }
        SymbolKind::Namespace
        | SymbolKind::Variable(_)
        | SymbolKind::Directive(_)
        | SymbolKind::ExternType(_) => false,
    }
}

// This doesn't really have use because config members are required to search for other intrinsics
// anyways so a failure to find a symbol is the only type of failure. As of right now.
/// Returns `true` if the type is a valid override config member candidate, `false` if invalid
pub fn check_cfg_memb_override(compiler: &ScriptCompiler, sym_id: SymbolId) -> bool {
    let sym = &compiler.syms[sym_id];
    match sym.kind {
        // Only override section symbols can access a namespace in it's config root.
        SymbolKind::Namespace => true,
        SymbolKind::Variable(_)
        | SymbolKind::Type(_)
        | SymbolKind::Directive(_)
        | SymbolKind::ExternType(_) => false,
    }
}

// Not used either...
/// Returns `true` if the type is a valid complex config member candidate, `false` if invalid
pub fn check_cfg_memb_complex(compiler: &ScriptCompiler, sym_id: SymbolId) -> bool {
    let sym = &compiler.syms[sym_id];
    match sym.kind {
        SymbolKind::Type(mut type_id) => {
            let checked = walk_type_id_deferred!(&compiler.types, type_id);
            match &compiler.types[checked.inner].ty {
                Type::TypeDef(_) | Type::Struct(_) | Type::Enum(_) => return true,
                // Brain failing here
                Type::BuiltinTypeInfo(_)
                | Type::Unknown
                | Type::Boundaries(_)
                | Type::Func(_)
                | Type::Alias(_) => return false,
                Type::Deferred(_) => unreachable!(),
            }
        }
        // Only override section symbols can access a namespace in it's config root.
        SymbolKind::Namespace => true,
        SymbolKind::Variable(_) | SymbolKind::Directive(_) | SymbolKind::ExternType(_) => false,
    }
}
