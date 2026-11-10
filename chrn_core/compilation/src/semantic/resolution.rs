//! General resolution functions

use chrn_utils::{id_types::InternedId, intern::UTF8Tag};
pub(crate) mod resolution_concepts;
pub(crate) mod resolution_helpers;

use chrn_utils::{
    id_types::TypeId, intern, source_utils::source_span::SourceSpan,
    utils::containers::SpannedContainer,
};
use lang::types::builtins::{BuiltinType, BuiltinTypeKind};

use crate::semantic::resolution::resolution_concepts::FoundStaticAccessInfo;
use crate::{
    lookup::scopes::{
        self,
        scopes_concepts::{
            self, AssociatedScopeKind, ScopeLookupPattern, ScopeLookupPreferenceFlags, ScopeType,
            SymbolLookupOutput,
        },
    },
    parser::ast::{
        ast_concepts::AbstractDirectiveUnparameterized,
        ast_exprs::{AbstractGeneric, PathSegment, TypeExpr},
    },
    resolvers::resolver_env::ResolverEnv,
    script_compiler::ScriptCompiler,
    semantic::{
        hir::{
            hir_concepts::{BuiltinTypeInfo, Type, TypeInfo},
            hir_directives::DirectiveUnparameterized,
            hir_symbols::{Symbol, SymbolKind, SymbolOrigin},
        },
        resolution::resolution_concepts::{StaticAccessOption, StaticAccessResult, TypeExprResult},
    },
};
/// - associated_scope: The environment to search in
/// - sp_ty_expr: The current type expression
/// - scope_type: The ScopeType environment
/// - lookup_pattern: The type of lookup which is recursively changed depending on if a direct
/// member access is being searched, or if a library such as core can be searched externally.
pub fn resolve_type_expr(
    compiler: &mut ScriptCompiler,
    associated_scope: AssociatedScopeKind,
    sp_ty_expr: &SpannedContainer<TypeExpr>,
    scope_type: ScopeType,
    lookup_pat: ScopeLookupPattern,
    env: &ResolverEnv,
) -> TypeExprResult {
    let lookup_pref = ScopeLookupPreferenceFlags::new(ScopeLookupPreferenceFlags::TYPE.into());
    match &sp_ty_expr.inner {
        TypeExpr::Var(name_id) => {
            let sp_name_id = SpannedContainer::new(*name_id, sp_ty_expr.span);

            //NOTE: Should encode that, it prefers a namespace over a type,
            match scopes::find_sym_id(
                compiler,
                associated_scope,
                *name_id,
                scope_type,
                lookup_pat,
                lookup_pref,
            ) {
                Some(SymbolLookupOutput { found_sym_id, .. }) => {
                    let found_sym = &compiler.syms[found_sym_id];
                    match found_sym.kind {
                        SymbolKind::Type(type_id) => {
                            let sym = &compiler.syms[found_sym_id];

                            // If the symbol being looked up is private and the owner isn't the current
                            // module then failed
                            // Should probably return the type id with it since this isn't
                            // supposed to assume any errors
                            if let SymbolOrigin::Module(mod_origin_id) = sym.sym_origin {
                                if sym.is_priv && mod_origin_id != env.current_mod {
                                    return TypeExprResult::PrivateTypeAccess {
                                        sp_found_type_id: SpannedContainer::new(
                                            type_id,
                                            sp_ty_expr.span,
                                        ),
                                        found_sym_id,
                                        current_mod_id: env.current_mod,
                                    };
                                }
                            }

                            TypeExprResult::Type(type_id)
                        }
                        SymbolKind::Namespace | SymbolKind::Variable(_) => {
                            //TODO: Declaration span for this?
                            return TypeExprResult::NotAType {
                                found_sym_id,
                                sp_name_id,
                                scope_found_in: associated_scope,
                            };
                        }
                        // NOTE:
                        // External types are too restricted to their particular scope that this is
                        // not actually possible. `override` doesn't have access to type expression
                        // declarations.
                        // This is IMPOSSIBLE to reach unless extern types become some general type
                        // usable, which would be against the language's intent.
                        SymbolKind::ExternType(_) => unreachable!(),
                    }
                }
                None => {
                    return TypeExprResult::SymbolNotFound(sp_name_id, associated_scope);
                }
            }
        }
        TypeExpr::Generic(generic) => resolve_generic(
            compiler,
            generic,
            associated_scope,
            sp_ty_expr.span,
            scope_type,
            env,
        ),
        TypeExpr::Path(sp_path_segs) => {
            let info = match resolve_static_access(
                compiler,
                sp_path_segs,
                associated_scope,
                scope_type,
                ScopeLookupPreferenceFlags::new(ScopeLookupPreferenceFlags::TYPE.into()),
                StaticAccessOption::Type,
            ) {
                StaticAccessResult::Found(info) => info,
                other => return TypeExprResult::StaticAccessFailure(other),
            };

            // TODO: Should this be something where `resolve_static_access` explicitly returns it so
            // that the caller isn't in charge of this? Probably.
            //
            // If len == 1 then nothing was accessed so no restrictions given
            let new_lookup_pat = if sp_path_segs.len() == 1 {
                ScopeLookupPattern::NoRestrictions
            } else {
                ScopeLookupPattern::NamespaceOnly
            };

            let last_segment = &sp_path_segs[sp_path_segs.len() - 1];

            match &last_segment.inner {
                PathSegment::Ident(interned_id) => {
                    // Is this a sign that resolve_type_expr should have an identifier only version
                    // which doesn't need this translation layer and instead is just a spanned container?
                    let inline_ty_expr =
                        SpannedContainer::new(TypeExpr::Var(*interned_id), last_segment.span);

                    resolve_type_expr(
                        compiler,
                        info.associated_scope,
                        &inline_ty_expr,
                        scope_type,
                        info.lookup_pat_found_in,
                        env,
                    )
                }
                PathSegment::Generic(generic) => {
                    //NOTE: This avoids turning the generic into a new owned type expr by giving the
                    //generic a specific resolution function, which isn't under a generic
                    //requirement for an owned `TypeExpr`
                    return resolve_generic(
                        compiler,
                        generic,
                        info.associated_scope,
                        sp_ty_expr.span,
                        scope_type,
                        env,
                    );
                }
            }
        }
    }
}

/// Generic specific resolution function
fn resolve_generic(
    compiler: &mut ScriptCompiler,
    // Span me a new container
    generic: &AbstractGeneric,
    associated_scope: AssociatedScopeKind,
    ty_expr_span: SourceSpan,
    scope_type: ScopeType,
    env: &ResolverEnv,
) -> TypeExprResult {
    match BuiltinTypeKind::try_from_interned_id(generic.base.inner().id) {
        Some(kind) => match kind {
            BuiltinTypeKind::List | BuiltinTypeKind::Set => {
                if generic.inputs.len() != 1 {
                    return TypeExprResult::InvalidGenericArgCount {
                        base: generic.base,
                        expected: 1,
                        inputs_span: ty_expr_span,
                    };
                }

                let inner = match resolve_type_expr(
                    compiler,
                    associated_scope,
                    &generic.inputs[0],
                    scope_type,
                    ScopeLookupPattern::NoRestrictions,
                    env,
                ) {
                    TypeExprResult::Type(tid) => tid,
                    other => return other,
                };
                //TEST:

                let sym_id = compiler.syms.make_id();
                let type_id = compiler.types.make_id();

                let (interned_id, ty) = if kind == BuiltinTypeKind::List {
                    let ty = Type::BuiltinTypeInfo(BuiltinTypeInfo::new(BuiltinType::List(inner)));
                    (
                        InternedId::new(intern::INTERNED_LIST).into_tagged::<UTF8Tag>(),
                        ty,
                    )
                } else {
                    let ty = Type::BuiltinTypeInfo(BuiltinTypeInfo::new(BuiltinType::Set(inner)));
                    (
                        InternedId::new(intern::INTERNED_SET).into_tagged::<UTF8Tag>(),
                        ty,
                    )
                };

                //WARN: Definitely not sure about this setup
                let sym = Symbol::new(
                    interned_id,
                    sym_id,
                    None,
                    SymbolOrigin::Module(env.current_mod),
                    true,
                    None,
                    ScopeType::Compiler,
                    SymbolKind::Type(type_id),
                );

                //WARN: Still not sure about this
                let ty_info = TypeInfo::new(ty, compiler.intrinsic_registry.core_mod_id);
                compiler.types.push(ty_info);
                compiler.syms.push(sym);

                return TypeExprResult::Type(type_id);
            }
            //TEST: TEST:
            BuiltinTypeKind::Tuple => {
                let mut elems: Vec<TypeId> = Vec::with_capacity(generic.inputs.len());

                for input in &generic.inputs {
                    match resolve_type_expr(
                        compiler,
                        associated_scope,
                        input,
                        scope_type,
                        ScopeLookupPattern::NoRestrictions,
                        env,
                    ) {
                        TypeExprResult::Type(tid) => elems.push(tid),
                        other => return other,
                    }
                }

                let sym_id = compiler.syms.make_id();
                let type_id = compiler.types.make_id();

                //WARN: Definitely not sure about this setup
                let sym = Symbol::new(
                    InternedId::new(intern::INTERNED_TUPLE).into_tagged::<UTF8Tag>(),
                    sym_id,
                    None,
                    SymbolOrigin::Module(env.current_mod),
                    true,
                    None,
                    ScopeType::Compiler,
                    SymbolKind::Type(type_id),
                );

                let tuple = Type::BuiltinTypeInfo(BuiltinTypeInfo::new(BuiltinType::Tuple(elems)));

                let ty_info = TypeInfo::new(tuple, compiler.intrinsic_registry.core_mod_id);
                compiler.types.push(ty_info);
                compiler.syms.push(sym);

                return TypeExprResult::Type(type_id);
            }
            BuiltinTypeKind::Map => {
                if generic.inputs.len() != 2 {
                    return TypeExprResult::InvalidGenericArgCount {
                        base: generic.base,
                        expected: 2,
                        inputs_span: ty_expr_span,
                    };
                }

                let key = match resolve_type_expr(
                    compiler,
                    AssociatedScopeKind::Module(env.current_mod),
                    &generic.inputs[0],
                    scope_type,
                    ScopeLookupPattern::NoRestrictions,
                    env,
                ) {
                    TypeExprResult::Type(tid) => tid,
                    other => return other,
                };

                let val = match resolve_type_expr(
                    compiler,
                    AssociatedScopeKind::Module(env.current_mod),
                    &generic.inputs[1],
                    scope_type,
                    ScopeLookupPattern::NoRestrictions,
                    env,
                ) {
                    TypeExprResult::Type(tid) => tid,
                    other => return other,
                };

                //TEST: TEST:
                let sym_id = compiler.syms.make_id();
                let type_id = compiler.types.make_id();

                let sym = Symbol::new(
                    InternedId::new(intern::INTERNED_MAP).into_tagged::<UTF8Tag>(),
                    sym_id,
                    None,
                    SymbolOrigin::Module(env.current_mod),
                    true,
                    None,
                    // Compiler generated but user created...
                    // Not sure about this one buddy
                    ScopeType::Compiler,
                    SymbolKind::Type(type_id),
                );

                let map = Type::BuiltinTypeInfo(BuiltinTypeInfo::new(BuiltinType::Map(key, val)));

                let ty_info = TypeInfo::new(map, compiler.intrinsic_registry.core_mod_id);
                compiler.types.push(ty_info);
                compiler.syms.push(sym);

                return TypeExprResult::Type(type_id);
            }
            _ => (),
        },
        None => (),
    }

    TypeExprResult::UnknownGenericIdent(SpannedContainer::new(generic.base, ty_expr_span))
}

/// Traverses scopes inside of static access.
///
/// Takes in the segments to traverse, scope to start in, scope type for scoping rules, and
/// whether or not type expression restrictions should be applied.
/// - sp_path_segs: Segments that will be traversed
/// - current_scope: Scope environment to start in
/// - scope_type: ScopeType environment
/// - lookup_pref: Bias flags to adhere to for all scope checks.
/// - opt: What rules should be applied for when to return a particular result, given a value or
/// type only setting.
///
/// NOTE: If `sp_path_seg.len()` == 1, `current_scope` is returned as `AssociatedScopeKind`,
/// resulting in a no-op. This is done because if there is no `::` after the current segment,
/// then going further does no real help for resolution.
pub fn resolve_static_access(
    compiler: &ScriptCompiler,
    sp_path_segs: &[SpannedContainer<PathSegment>],
    mut current_scope: AssociatedScopeKind,
    scope_type: ScopeType,
    lookup_pref: ScopeLookupPreferenceFlags,
    opt: StaticAccessOption,
) -> StaticAccessResult {
    let mut current_lookup_pat = ScopeLookupPattern::NoRestrictions;
    //If the path segment len == 1 this will not do anything because this is not actually
    //accessing anything. Doing this because if we have something like "Thing::Thingy" and either
    //could be a type or symbol, being able to call this without explicitly at the call site
    //skipping sounds like a more valid argument than, if you call this without accounting for that
    //your system breaks. For example in the intrinsic scope "JAVA" if "JAVA {}" is typed, it goes
    //into the namespace of JAVA as though it acutally had access used, but in reality it should be
    //a no-op.
    if sp_path_segs.len() == 1 {
        let info = FoundStaticAccessInfo::new(current_scope, ScopeLookupPattern::NoRestrictions);
        return StaticAccessResult::Found(info);
    }

    for (i, sp_path_seg) in sp_path_segs.iter().enumerate() {
        // Means at most `module::something` was done, so nothing further should have access to
        // more than it's own namespace.
        if i > 0 {
            current_lookup_pat = ScopeLookupPattern::NamespaceOnly;
        }

        match &sp_path_seg.inner {
            PathSegment::Ident(interned_id) => {
                if let Some(SymbolLookupOutput { found_sym_id, .. }) = scopes::find_sym_id(
                    compiler,
                    current_scope,
                    *interned_id,
                    scope_type,
                    current_lookup_pat,
                    lookup_pref,
                ) {
                    let sym = &compiler.syms[found_sym_id];
                    match sym.associated_scope {
                        Some(new_scope) => {
                            // Transitioning to next namespace
                            current_scope = new_scope;

                            // Prevents situations like `core::u8` accessing then namespace inside
                            // `u8` by checking if we are 1 before the end so that we avoid trying
                            // to get a namespace out of the final segment.
                            if i + 2 == sp_path_segs.len() {
                                break;
                            }
                        }
                        None => {
                            // If the current iteration is the last iteration then we know it's an error
                            if i + 1 < sp_path_segs.len() {
                                return StaticAccessResult::NoNamespace(SpannedContainer::new(
                                    *interned_id,
                                    sp_path_seg.span,
                                ));
                            }
                        }
                    }
                } else {
                    let prev_seg = if i > 0 {
                        match &sp_path_segs[i - 1].inner {
                            PathSegment::Ident(prev_name_id) => Some(SpannedContainer::new(
                                *prev_name_id,
                                sp_path_segs[i - 1].span,
                            )),
                            PathSegment::Generic(_) => None,
                        }
                    } else {
                        None
                    };

                    // `another` failed here
                    return StaticAccessResult::SymNotFound {
                        current_seg: SpannedContainer::new(*interned_id, sp_path_seg.span),
                        prev_seg,
                    };
                }
            }
            PathSegment::Generic(_) if opt == StaticAccessOption::Val => {
                if i + 1 != sp_path_segs.len() {
                    return StaticAccessResult::GenericUsingStaticPath(sp_path_seg.span);
                }
                break;
            }
            PathSegment::Generic(_) => {
                unreachable!("Cannot have generics within expr at the language level");
                // return StaticAccessResult::GenericInExpr(sp_path_seg.span);
            }
        }
    }

    // Since len must be > 1 we know it's `NamespaceOnly`
    let info = FoundStaticAccessInfo::new(current_scope, ScopeLookupPattern::NamespaceOnly);
    StaticAccessResult::Found(info)
}

// I KNOW WHAT THIS LOOKS LIKE BUT IT MIGHT BECOME MORE THAN THIS SO IT STILL GETS ITS OWN FUNCTION
/// Resolves directive
pub fn resolve_directive(
    abs_directive: &AbstractDirectiveUnparameterized,
) -> Option<DirectiveUnparameterized> {
    DirectiveUnparameterized::try_from_interned_str(abs_directive.sp_name_id.inner)
}
