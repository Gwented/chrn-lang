use super::helpers::*;
use crate::lookup::scopes::find_sym_id;
use crate::lookup::scopes::scopes_concepts::{
    AssociatedScopeKind, BuiltinIdRepository, BuiltinIdRepositoryBuilder, ScopeLookupPattern,
    ScopeLookupPreferenceFlags, ScopeType,
};
use crate::parser::ast::ast_exprs::PathSegment;
use crate::script_compiler::compiler_consts::builtin_ty_to_id;
use crate::script_compiler::helpers::core_helpers::core_helpers_dataset::{
    BUILTIN_TYPES_RANGE, TYPES_NAMESPACE,
};
use crate::script_compiler::helpers::instantiation_symbols::{
    InstantiationSymbolBase, InstantiationSymbolKind, InstantiationTypeKind,
};
use crate::semantic::hir::hir_concepts::Type;
use crate::semantic::hir::hir_symbols::SymbolKind;
use crate::semantic::resolution::{
    resolution_concepts::{StaticAccessOption, StaticAccessResult},
    resolve_static_access,
};
use chrn_utils::err_codes::ErrorCode;
use chrn_utils::id_types::{ScopeId, SymbolId, TypeId};
use chrn_utils::intern;
use chrn_utils::utils::containers::SpannedContainer;
use chrn_utils::{id_types::InternedId, intern::UTF8Tag};
use lang::types::builtins::BuiltinType;
use lang::types::builtins::BuiltinTypeKind;

/// Runs lexing, parsing, and namespace resolution on a single-module script, returning the
/// compiler and interner so scope lookups can be performed directly.
fn ns_resolve(text: &str) -> (ScriptCompiler, Intern) {
    resolve_single_module(text, Stage::Namespace)
        .expect_ok()
        .into_state()
}

/// Gets the symbol registered under `name` in the given scope of the module.
fn sym_in_scope(
    compiler: &ScriptCompiler,
    interner: &Intern,
    scope_type: ScopeType,
    name: &str,
) -> (ScopeId, SymbolId) {
    let name_id = interner
        .get_interned_id_from_str(name)
        .unwrap_or_else(|| panic!("'{name}' was not interned"));
    let scope_id = compiler
        .get_scope_id(scope_type, ModuleId::new(0))
        .unwrap_or_else(|| panic!("{scope_type} scope should exist"));
    let sym_id = compiler.get_scope(scope_id).scope.table.interned_to_sym[&name_id];

    (scope_id, sym_id)
}

fn builtin_type_entry(
    base: &InstantiationSymbolBase,
) -> (u32, BuiltinType, &'static [InstantiationSymbolBase]) {
    let InstantiationSymbolKind::Type(ty) = &base.kind else {
        unreachable!("expected a built-in type descriptor");
    };
    let InstantiationTypeKind::BuiltinType(builtin_ty) = &ty.kind else {
        unreachable!("expected a built-in type descriptor");
    };
    (
        base.name_id.inner().id,
        builtin_ty.clone(),
        ty.ns.as_ref().map(|ns| ns.ns).unwrap_or(&[]),
    )
}

#[test]
fn scope_simple_test() {
    // -- NEUTRAL --
    let text = "
            let CONSTANT = 3
            ";

    let (compiler, _) = ns_resolve(text);

    let module = &compiler.mods[ModuleId::new(0)];

    assert_eq!(module.scopes.len(), 3);
    assert_eq!(
        compiler.get_scope(module.scopes[1]).scope.scope_type,
        ScopeType::Compiler
    );

    // -- VAR --
    let text = "
            var->
                variable: i32
            ";

    let (compiler, _) = ns_resolve(text);

    let module = &compiler.mods[ModuleId::new(0)];

    assert_eq!(module.scopes.len(), 3);
    assert_eq!(
        compiler.get_scope(module.scopes[2]).scope.scope_type,
        ScopeType::Var
    );

    // -- NEST --
    let text = "
            nest->
                struct Thing1 {}
                struct Thing2 {}
            ";

    let (compiler, _) = ns_resolve(text);

    let module = &compiler.mods[ModuleId::new(0)];

    assert_eq!(module.scopes.len(), 3);
    assert_eq!(
        compiler.get_scope(module.scopes[2]).scope.scope_type,
        ScopeType::Nest
    );

    // -- All scopes --

    //TODO: Complex and Override
    let text = "
            let NEUTRAL = 3
            var->
                e#var: Nest
            nest->
                struct Nest {}
            ";

    let (compiler, _) = ns_resolve(text);

    //TODO: Override and Complex
    let module = &compiler.mods[ModuleId::new(0)];
    assert_eq!(module.scopes.len(), 5);
    assert_eq!(
        compiler.get_scope(module.scopes[1]).scope.scope_type,
        ScopeType::Compiler
    );
    assert_eq!(
        compiler.get_scope(module.scopes[2]).scope.scope_type,
        ScopeType::Neutral
    );
    assert_eq!(
        compiler.get_scope(module.scopes[3]).scope.scope_type,
        ScopeType::Var
    );
}

#[test]
fn scope_lookup_pref_found_test() {
    // From `var` the scope order is Nest -> Neutral -> Compiler -> Core, so the struct is
    // seen before the variable.
    let text = "
            let Thing = 3
            nest->
                struct Thing {}
            ";

    let (compiler, interner) = ns_resolve(text);
    let thing_id = interner.get_interned_id_from_str("Thing").unwrap();

    // A variable preference skips the struct found in `nest` and returns the variable
    // found later in `neutral`.
    let found = find_sym_id(
        &compiler,
        AssociatedScopeKind::Module(ModuleId::new(0)),
        thing_id,
        ScopeType::Var,
        ScopeLookupPattern::NoRestrictions,
        ScopeLookupPreferenceFlags::new(ScopeLookupPreferenceFlags::VARIABLE.into()),
    )
    .expect("Symbol should be found");

    let (neutral_scope_id, var_sym_id) =
        sym_in_scope(&compiler, &interner, ScopeType::Neutral, "Thing");
    assert_eq!(found.found_sym_id, var_sym_id);
    assert_eq!(found.scope_found_in, neutral_scope_id);
    assert!(matches!(
        compiler.syms[found.found_sym_id].kind,
        SymbolKind::Variable(_)
    ));

    // A type preference matches the struct on the first scope searched.
    let found = find_sym_id(
        &compiler,
        AssociatedScopeKind::Module(ModuleId::new(0)),
        thing_id,
        ScopeType::Var,
        ScopeLookupPattern::NoRestrictions,
        ScopeLookupPreferenceFlags::new(ScopeLookupPreferenceFlags::TYPE.into()),
    )
    .expect("Symbol should be found");

    let (nest_scope_id, struct_sym_id) =
        sym_in_scope(&compiler, &interner, ScopeType::Nest, "Thing");
    assert_eq!(found.found_sym_id, struct_sym_id);
    assert_eq!(found.scope_found_in, nest_scope_id);
    assert!(matches!(
        compiler.syms[found.found_sym_id].kind,
        SymbolKind::Type(_)
    ));
}

#[test]
fn scope_lookup_pref_fallback_test() {
    // Only a type named `Thing` exists, so a variable preference can't be satisfied and the
    // lookup falls back to the non-preferred symbol it did find.
    let text = "
            nest->
                struct Thing {}
            ";

    let (compiler, interner) = ns_resolve(text);
    let thing_id = interner.get_interned_id_from_str("Thing").unwrap();

    let found = find_sym_id(
        &compiler,
        AssociatedScopeKind::Module(ModuleId::new(0)),
        thing_id,
        ScopeType::Nest,
        ScopeLookupPattern::NoRestrictions,
        ScopeLookupPreferenceFlags::new(ScopeLookupPreferenceFlags::VARIABLE.into()),
    )
    .expect("Fallback should return the non-preferred symbol");

    let (nest_scope_id, struct_sym_id) =
        sym_in_scope(&compiler, &interner, ScopeType::Nest, "Thing");
    assert_eq!(found.found_sym_id, struct_sym_id);
    assert_eq!(found.scope_found_in, nest_scope_id);

    // Mirror: only a variable named `Thing` exists, so a type preference falls back to it.
    let text = "
            let Thing = 3
            ";

    let (compiler, interner) = ns_resolve(text);
    let thing_id = interner.get_interned_id_from_str("Thing").unwrap();

    let found = find_sym_id(
        &compiler,
        AssociatedScopeKind::Module(ModuleId::new(0)),
        thing_id,
        ScopeType::Neutral,
        ScopeLookupPattern::NoRestrictions,
        ScopeLookupPreferenceFlags::new(ScopeLookupPreferenceFlags::TYPE.into()),
    )
    .expect("Fallback should return the non-preferred symbol");

    let (neutral_scope_id, var_sym_id) =
        sym_in_scope(&compiler, &interner, ScopeType::Neutral, "Thing");
    assert_eq!(found.found_sym_id, var_sym_id);
    assert_eq!(found.scope_found_in, neutral_scope_id);
}

#[test]
fn scope_lookup_no_preference_test() {
    // From `var` the scope order is Nest -> Neutral -> Compiler -> Core, so with no
    // preference the struct is returned since `nest` is searched first.
    let text = "
            let Thing = 3
            nest->
                struct Thing {}
            ";

    let (compiler, interner) = ns_resolve(text);
    let thing_id = interner.get_interned_id_from_str("Thing").unwrap();

    let found = find_sym_id(
        &compiler,
        AssociatedScopeKind::Module(ModuleId::new(0)),
        thing_id,
        ScopeType::Var,
        ScopeLookupPattern::NoRestrictions,
        ScopeLookupPreferenceFlags::none(),
    )
    .expect("Symbol should be found");

    let (_, struct_sym_id) = sym_in_scope(&compiler, &interner, ScopeType::Nest, "Thing");
    assert_eq!(found.found_sym_id, struct_sym_id);
}

/// Every built-in that declares an intrinsic namespace, as `(source name, its `TypeId`)`.
fn namespaced_builtins() -> Vec<(String, TypeId)> {
    let interner = Intern::init();

    TYPES_NAMESPACE[BUILTIN_TYPES_RANGE]
        .iter()
        .map(builtin_type_entry)
        .filter(|(_, _, ns)| !ns.is_empty())
        .map(|(interned, builtin_ty, _)| {
            (
                interner
                    .extract_utf8_str(InternedId::new(interned).into_tagged::<UTF8Tag>())
                    .to_string(),
                TypeId::new(builtin_ty_to_id(builtin_ty.kind())),
            )
        })
        .collect()
}

/// The scope a built-in's symbol owns, which is where its intrinsic constants live.
fn ns_scope_of(compiler: &ScriptCompiler, type_id: TypeId) -> ScopeId {
    let sym_id = compiler
        .get_sym_id_from_type_id(type_id)
        .expect("expected a builtin symbol id");

    match compiler.syms[sym_id].associated_scope {
        Some(AssociatedScopeKind::Scope(scope_id)) => scope_id,
        other => panic!("builtin at {type_id:?} owns no namespace scope, got {other:?}"),
    }
}

/// Intrinsic constants such as `i8::MAX` live in a scope hanging off the built-in's symbol, so
/// they are reachable through that scope and through nothing else. A section scope must not see
/// them, otherwise a bare `MAX` would resolve and the namespaces would collide on one name.
#[test]
fn scope_core_type_namespace_test() {
    let (compiler, _) = ns_resolve("let CONSTANT = 3");
    let max_id = InternedId::new(intern::INTERNED_MAX_UPPER).into_tagged::<UTF8Tag>();
    let min_id = InternedId::new(intern::INTERNED_MIN_UPPER).into_tagged::<UTF8Tag>();
    let bits_id = InternedId::new(intern::INTERNED_BITS_UPPER).into_tagged::<UTF8Tag>();
    let bytes_id = InternedId::new(intern::INTERNED_BYTES_UPPER).into_tagged::<UTF8Tag>();
    let tab_id = InternedId::new(intern::INTERNED_TAB_UPPER).into_tagged::<UTF8Tag>();

    for (name, type_id) in namespaced_builtins() {
        let scope_id = ns_scope_of(&compiler, type_id);
        // `f128` carries metadata attributes but no MAX bound, `bool` carries only
        // width metadata, and `str` carries shared constants rather than bounds.
        let bounds_to_check = if name == "f128" || name == "bool" {
            vec![bits_id, bytes_id]
        } else if name == "str" {
            vec![tab_id]
        } else {
            vec![max_id, min_id]
        };

        for bound_id in bounds_to_check {
            let found = find_sym_id(
                &compiler,
                AssociatedScopeKind::Scope(scope_id),
                bound_id,
                ScopeType::Core,
                ScopeLookupPattern::NamespaceOnly,
                ScopeLookupPreferenceFlags::none(),
            )
            .unwrap_or_else(|| panic!("`{name}` should hold {bound_id:?} in {scope_id:?}"));

            assert_eq!(found.scope_found_in, scope_id);
            assert!(
                matches!(
                    compiler.syms[found.found_sym_id].kind,
                    SymbolKind::Variable(_)
                ),
                "`{name}::{bound_id:?}` should be a variable"
            );
        }
    }

    // The same identifier from the module's own scopes, which is what a bare `MAX` searches
    for bound_id in [max_id, min_id] {
        assert!(
            find_sym_id(
                &compiler,
                AssociatedScopeKind::Module(ModuleId::new(0)),
                bound_id,
                ScopeType::Neutral,
                ScopeLookupPattern::NoRestrictions,
                ScopeLookupPreferenceFlags::none(),
            )
            .is_none(),
            "{bound_id:?} must not be reachable without naming the builtin that owns it"
        );
    }
}

/// A bare `MAX` names nothing, and a built-in with no intrinsic namespace has nothing to
/// traverse into. Both must be diagnosed rather than resolving to some other built-in's bound.
#[test]
fn scope_core_type_namespace_unreachable_test() {
    let diags = resolve_single_module("let CONSTANT = MAX", Stage::Constraint);
    assert!(
        diags.err_count() > 0,
        "a bare `MAX` should not resolve: {:?}",
        diags.ty
    );

    // `f128` carries metadata attributes but no MAX bound
    let f128_res = resolve_single_module("let CONSTANT = f128::MAX", Stage::Constraint);
    assert!(
        f128_res.err_count() > 0,
        "`f128` declares no MAX bound, so `f128::MAX` should not resolve"
    );

    // `sized`/`unsized` are pointer-sized, so neither carries an intrinsic namespace
    for (interned, builtin_ty, ns) in TYPES_NAMESPACE[BUILTIN_TYPES_RANGE]
        .iter()
        .map(builtin_type_entry)
    {
        let interner = Intern::init();
        let name = interner.extract_utf8_str(InternedId::new(interned).into_tagged::<UTF8Tag>());

        if !ns.is_empty() {
            continue;
        }

        let text = format!("let CONSTANT = {name}::MAX");
        let res = resolve_single_module(&text, Stage::Constraint);

        assert!(
            res.err_count() > 0,
            "`{:?}` declares no MAX bound, so `{name}::MAX` should not resolve",
            builtin_ty.kind()
        );
    }
}

#[test]
fn builtin_id_repository_builder_full_build() {
    let mut builder = BuiltinIdRepository::builder();
    for (idx, (_, builtin_ty, _)) in TYPES_NAMESPACE[BUILTIN_TYPES_RANGE]
        .iter()
        .map(builtin_type_entry)
        .enumerate()
    {
        builder.add(builtin_ty.kind(), SymbolId::new(idx as u32));
    }
    let repo = builder.build();
    assert_eq!(repo.i8, SymbolId::new(0));
    assert_eq!(repo.get(BuiltinTypeKind::I8), Some(SymbolId::new(0)));
    assert_eq!(repo.get(BuiltinTypeKind::List), None);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic]
fn builtin_id_repository_builder_duplicate_debug_assert() {
    let mut builder = BuiltinIdRepository::builder();
    builder.add(BuiltinTypeKind::I8, SymbolId::new(1));
    builder.add(BuiltinTypeKind::I8, SymbolId::new(2));
}

#[test]
#[should_panic]
fn builtin_id_repository_builder_incomplete_panics() {
    let mut builder = BuiltinIdRepository::builder();
    builder.add(BuiltinTypeKind::I8, SymbolId::new(1));
    builder.build();
}

#[test]
fn builtin_id_repository_initialized_in_compiler() {
    let (compiler, _) = ns_resolve("");
    let repo = compiler
        .intrinsic_registry
        .builtin_repo
        .as_ref()
        .expect("builtin_repo must be initialized in ScriptCompiler");

    for (_, builtin_ty, _) in TYPES_NAMESPACE[BUILTIN_TYPES_RANGE]
        .iter()
        .map(builtin_type_entry)
    {
        assert!(
            repo.get(builtin_ty.kind()).is_some(),
            "missing symbol for {:?} in builtin_repo",
            builtin_ty.kind()
        );
    }
}

#[test]
fn implicit_identifiers_respect_module_namespace_boundaries() {
    let result =
        resolve_cross_module("let VALUE = 1", "export let OTHER = 2", Stage::Namespace).expect_ok();
    let compiler = &result.compiler;
    let main_mod_id = ModuleId::new(0);
    let sub_mod_id = ModuleId::new(1);
    let core_mod_id = compiler.intrinsic_registry.core_mod_id;
    assert!(!compiler.mods[main_mod_id].flag_meta.has_implicit_idents());
    assert!(!compiler.mods[sub_mod_id].flag_meta.has_implicit_idents());
    assert!(compiler.mods[core_mod_id].flag_meta.has_implicit_idents());
    for name in ["i8", "Numeric"] {
        let name_id = result.interner.get_interned_id_from_str(name).unwrap();
        let expected = compiler.intrinsic_registry.glob_implicit_idents[&name_id];
        for mod_id in [main_mod_id, sub_mod_id] {
            let found = find_sym_id(
                compiler,
                AssociatedScopeKind::Module(mod_id),
                name_id,
                ScopeType::Neutral,
                ScopeLookupPattern::NoRestrictions,
                ScopeLookupPreferenceFlags::none(),
            )
            .unwrap();
            assert_eq!(found.found_sym_id, expected, "bare {name} in {mod_id:?}");
            assert!(
                find_sym_id(
                    compiler,
                    AssociatedScopeKind::Module(mod_id),
                    name_id,
                    ScopeType::Neutral,
                    ScopeLookupPattern::NamespaceOnly,
                    ScopeLookupPreferenceFlags::none()
                )
                .is_none(),
                "{mod_id:?} must not expose implicit {name} as its own member"
            );
        }
        assert!(
            find_sym_id(
                compiler,
                AssociatedScopeKind::Module(core_mod_id),
                name_id,
                ScopeType::Neutral,
                ScopeLookupPattern::NamespaceOnly,
                ScopeLookupPreferenceFlags::none()
            )
            .is_none(),
            "core::{name} must use the explicit core::types namespace"
        );
    }
    let types_id = result.interner.get_interned_id_from_str("types").unwrap();
    assert!(
        find_sym_id(
            compiler,
            AssociatedScopeKind::Module(main_mod_id),
            types_id,
            ScopeType::Neutral,
            ScopeLookupPattern::NoRestrictions,
            ScopeLookupPreferenceFlags::none()
        )
        .is_none(),
        "core::types is not an implicit global identifier"
    );
    assert!(
        find_sym_id(
            compiler,
            AssociatedScopeKind::Module(core_mod_id),
            types_id,
            ScopeType::Neutral,
            ScopeLookupPattern::NamespaceOnly,
            ScopeLookupPreferenceFlags::none()
        )
        .is_some()
    );
}

#[test]
fn static_access_preserves_module_and_symbol_scope_permissions() {
    let result =
        resolve_cross_module("let VALUE = 1", "export let OTHER = 2", Stage::Namespace).expect_ok();
    let compiler = &result.compiler;
    let main_scope = AssociatedScopeKind::Module(ModuleId::new(0));
    let i8_id = result.interner.get_interned_id_from_str("i8").unwrap();
    let i8_sym_id = compiler.intrinsic_registry.glob_implicit_idents[&i8_id];
    let i8_scope = compiler.syms[i8_sym_id].associated_scope.unwrap();
    let core_scope_id =
        compiler.extract_scope_id(ScopeType::Core, compiler.intrinsic_registry.core_mod_id);
    let types_id = result.interner.get_interned_id_from_str("types").unwrap();
    let types_sym_id = compiler.scopes[core_scope_id].scope.table.interned_to_sym[&types_id];
    let types_scope = compiler.syms[types_sym_id].associated_scope.unwrap();
    for (path, expected_scope, expected_pat) in [
        (vec!["i8"], main_scope, ScopeLookupPattern::NoRestrictions),
        (
            vec!["sub_module", "OTHER"],
            AssociatedScopeKind::Module(ModuleId::new(1)),
            ScopeLookupPattern::NamespaceOnly,
        ),
        (
            vec!["i8", "MAX"],
            i8_scope,
            ScopeLookupPattern::NamespaceOnly,
        ),
        (
            vec!["core", "types", "i8"],
            types_scope,
            ScopeLookupPattern::NamespaceOnly,
        ),
        (
            vec!["core", "types", "i8", "MAX"],
            i8_scope,
            ScopeLookupPattern::NamespaceOnly,
        ),
    ] {
        let segs: Vec<_> = path
            .iter()
            .map(|name| {
                SpannedContainer::new(
                    PathSegment::Ident(result.interner.get_interned_id_from_str(name).unwrap()),
                    SourceSpan::default(),
                )
            })
            .collect();
        let found = resolve_static_access(
            compiler,
            &segs,
            main_scope,
            ScopeType::Neutral,
            ScopeLookupPreferenceFlags::none(),
            StaticAccessOption::None,
        );
        let StaticAccessResult::Found(info) = found else {
            panic!("{path:?}: {found:?}");
        };
        assert_eq!(info.associated_scope, expected_scope, "{path:?}");
        assert_eq!(info.lookup_pat_found_in, expected_pat, "{path:?}");
        let last_id = result
            .interner
            .get_interned_id_from_str(path.last().unwrap())
            .unwrap();
        assert!(
            find_sym_id(
                compiler,
                info.associated_scope,
                last_id,
                ScopeType::Neutral,
                info.lookup_pat_found_in,
                ScopeLookupPreferenceFlags::none()
            )
            .is_some(),
            "final segment of {path:?}"
        );
    }
}

#[test]
fn implicit_and_explicit_builtin_paths_resolve_same_value() {
    let result = resolve_single_module("let BARE = i8::MAX\nlet QUALIFIED = core::types::i8::MAX\nvar->\n first: i8\n second: core::types::i8", Stage::Constraint).expect_ok();
    for name in ["BARE", "QUALIFIED"] {
        assert!(
            matches!(
                result.value_of(name),
                Value::ArbitraryInt(ArbitraryIntKind::I64(127))
            ),
            "{name}"
        );
    }
}

#[test]
fn qualified_user_module_does_not_gain_implicit_builtin_values() {
    let result = resolve_cross_module(
        "let BAD = sub_module::i8",
        "export let OTHER = 2",
        Stage::Type,
    );
    assert_eq!(result.ns.err_count(), 0);
    assert_eq!(result.member.err_count(), 0);
    assert_eq!(
        result.ty.err_count(),
        1,
        "qualified user-module lookup must reject a builtin absent from that module: {:?}",
        result.ty
    );
    assert_eq!(result.ty.diags[0].err_code, Some(ErrorCode::ScopeErr));
    assert_eq!(
        result.ty.diags[0].annotations[0].span,
        SourceSpan::new(SourceRegionId::new(0), 22, 24)
    );
}

#[test]
fn qualified_core_predicate_functions_resolve() {
    resolve_single_module(
        "var->\n field: str [core::funcs::IsEmpty]",
        Stage::Type,
    )
    .expect_ok();
}
