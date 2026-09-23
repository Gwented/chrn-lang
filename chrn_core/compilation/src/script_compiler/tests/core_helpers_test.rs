use chrn_utils::{
    arena::Arena,
    id_types::{InternedId, ScopeId, TypeId, VariableId},
    intern::{self, Intern},
    source_map::source_span::SourceSpan,
    utils::containers::SpannedContainer,
};
use dashu_int::IBig;
use lang::types::{
    builtins::{BuiltinType, BuiltinTypeKind},
    externs::{ExternPlatformType, java_types::JavaTypeKind, rust_types::RustTypeKind},
};

use crate::{
    lookup::scopes::scopes_concepts::{AssociatedScopeKind, ScopeType},
    module::module_concepts::{Import, ImportKind, Module},
    script_compiler::{
        ScriptCompiler,
        compiler_consts::{CORE_UNKNOWN, builtin_ty_to_id},
        helpers::{
            compiler_helpers::DIRECTIVES_DATASET,
            core_helpers::{
                CORE_BOUNDARIES_DATASET, CORE_BUILTIN_TYPES_DATASET, CORE_FUNCS_DATASET,
                core_instantiation_reservations, count_instantiation_bases,
            },
            instantiation_symbols::{
                InstantiationSymbolBase, InstantiationSymbolKind, InstantiationVariable,
                InstiationType, InstiationValue,
            },
        },
    },
    semantic::arbitraries::{ArbitraryFloatKind, ArbitraryIntKind},
    semantic::hir::{
        hir_concepts::{Type, TypeInfo},
        hir_exprs::{ExprHir, ResolvedExpr, ResolvedExprMetadata},
        hir_symbols::{Symbol, SymbolKind, SymbolOrigin, VarDef, VariableMetadata, VariableState},
    },
    semantic::values::{Value, ValueInfo},
};

/// Builds the core module the same way `ScriptCompiler::init` does, with no user modules
fn core_only_compiler() -> ScriptCompiler {
    ScriptCompiler::init(None, Arena::new())
}

#[test]
fn complex_scope_instantiates_extern_type_namespaces() {
    let mut compiler = core_only_compiler();
    let core_mod_id = compiler.intrinsic_registry.core_mod_id;
    compiler.push_scope(ScopeType::Complex, core_mod_id);
    let complex_scope_id = compiler
        .intrinsic_registry
        .complex_scope_id
        .expect("pushing a complex scope should create its intrinsic scope");

    let descend_namespace = |scope_id: ScopeId, name_id: u32| {
        let sym_id = compiler.scopes[scope_id]
            .scope
            .table
            .interned_to_sym
            .get(&InternedId::new(name_id))
            .copied()
            .expect("namespace should be instantiated");
        let sym = &compiler.syms[sym_id];
        assert!(matches!(sym.kind, SymbolKind::Namespace));
        let Some(AssociatedScopeKind::Scope(child_scope_id)) = sym.associated_scope else {
            panic!("namespace should own a scope");
        };
        child_scope_id
    };

    let rust_upper_scope = descend_namespace(complex_scope_id, intern::INTERNED_RUST_UPPER);
    let types_scope = descend_namespace(rust_upper_scope, intern::INTERNED_TYPES_LOWER);
    let rust_lower_scope = descend_namespace(types_scope, intern::INTERNED_RUST_LOWER);

    assert_eq!(
        compiler.scopes[rust_upper_scope]
            .scope
            .table
            .interned_to_sym
            .len(),
        1
    );
    assert_eq!(
        compiler.scopes[types_scope]
            .scope
            .table
            .interned_to_sym
            .len(),
        1
    );

    let expected = [
        (intern::INTERNED_U8, RustTypeKind::U8),
        (intern::INTERNED_I8, RustTypeKind::I8),
        (intern::INTERNED_U16, RustTypeKind::U16),
        (intern::INTERNED_I16, RustTypeKind::I16),
        (intern::INTERNED_U32, RustTypeKind::U32),
        (intern::INTERNED_I32, RustTypeKind::I32),
        (intern::INTERNED_F32, RustTypeKind::F32),
        (intern::INTERNED_U64, RustTypeKind::U64),
        (intern::INTERNED_I64, RustTypeKind::I64),
        (intern::INTERNED_F64, RustTypeKind::F64),
        (intern::INTERNED_I128, RustTypeKind::I128),
        (intern::INTERNED_U128, RustTypeKind::U128),
        (intern::INTERNED_F128, RustTypeKind::F128),
        (intern::INTERNED_USIZE, RustTypeKind::Usize),
        (intern::INTERNED_ISIZE, RustTypeKind::Isize),
        (intern::INTERNED_BOOL, RustTypeKind::Bool),
        (intern::INTERNED_CHAR, RustTypeKind::Char),
        (intern::INTERNED_STR, RustTypeKind::Str),
        (intern::INTERNED_STRING, RustTypeKind::String),
    ];
    let rust_table = &compiler.scopes[rust_lower_scope]
        .scope
        .table
        .interned_to_sym;

    assert_eq!(rust_table.len(), expected.len());
    for (name_id, rust_kind) in expected {
        let sym_id = rust_table
            .get(&InternedId::new(name_id))
            .copied()
            .expect("Rust extern type should be instantiated");
        assert!(
            matches!(
                compiler.syms[sym_id].kind,
                SymbolKind::ExternType(ExternPlatformType::Rust(found)) if found == rust_kind
            ),
            "Rust extern symbol for interned id {name_id} has the wrong type"
        );
    }

    let java_upper_scope = descend_namespace(complex_scope_id, intern::INTERNED_JAVA_UPPER);
    let types_scope = descend_namespace(java_upper_scope, intern::INTERNED_TYPES_LOWER);
    let java_lower_scope = descend_namespace(types_scope, intern::INTERNED_JAVA_LOWER);

    assert_eq!(
        compiler.scopes[java_upper_scope]
            .scope
            .table
            .interned_to_sym
            .len(),
        1
    );
    assert_eq!(
        compiler.scopes[types_scope]
            .scope
            .table
            .interned_to_sym
            .len(),
        1
    );

    let expected = [
        (intern::INTERNED_LONG, JavaTypeKind::Long),
        (intern::INTERNED_INT, JavaTypeKind::Int),
        (intern::INTERNED_SHORT, JavaTypeKind::Short),
        (intern::INTERNED_BYTE, JavaTypeKind::Byte),
        (intern::INTERNED_FLOAT_LOWER, JavaTypeKind::Float),
        (intern::INTERNED_DOUBLE, JavaTypeKind::Double),
        (intern::INTERNED_BOOLEAN, JavaTypeKind::Boolean),
        (intern::INTERNED_CHAR, JavaTypeKind::Char),
        (intern::INTERNED_STRING, JavaTypeKind::String),
    ];
    let java_table = &compiler.scopes[java_lower_scope]
        .scope
        .table
        .interned_to_sym;

    assert_eq!(java_table.len(), expected.len());
    for (name_id, java_kind) in expected {
        let sym_id = java_table
            .get(&InternedId::new(name_id))
            .copied()
            .expect("Java extern type should be instantiated");
        assert!(
            matches!(
                compiler.syms[sym_id].kind,
                SymbolKind::ExternType(ExternPlatformType::Java(found)) if found == java_kind
            ),
            "Java extern symbol for interned id {name_id} has the wrong type"
        );
    }
}

#[test]
fn core_builtin_type_id_alignment() {
    let compiler = core_only_compiler();
    let interner = Intern::init();

    for (idx, (interned, builtin_ty, _ns)) in CORE_BUILTIN_TYPES_DATASET.iter().cloned().enumerate()
    {
        // The dataset is loaded sequentially, so an entry's index is the `TypeId` it gets, and the
        // `CORE_*` constant behind `builtin_ty_to_id` must name that same id
        let core_id = idx as u32;
        assert_eq!(
            builtin_ty_to_id(builtin_ty.kind()),
            core_id,
            "CORE constant for {:?} does not match its position in the table",
            builtin_ty.kind()
        );

        let type_info: &TypeInfo = &compiler.types[TypeId::new(core_id)];

        let Type::BuiltinTypeInfo(info) = &type_info.ty else {
            panic!(
                "expected builtin type at CORE id {core_id}, got {:?}",
                type_info.ty
            );
        };

        assert_eq!(
            info.ty.kind(),
            builtin_ty.kind(),
            "builtin at CORE id {core_id} does not match the table entry"
        );

        let sym = &compiler.syms[info.sym_id];
        assert_eq!(
            sym.name_id,
            InternedId::new(interned),
            "symbol name for {} does not match its interned id",
            interner.search_idx(interned as usize)
        );
    }
}

#[test]
fn core_unknown_type_id_alignment() {
    let compiler = core_only_compiler();
    let type_info = &compiler.types[TypeId::new(CORE_UNKNOWN)];

    assert!(
        matches!(type_info.ty, Type::Unknown),
        "expected Type::Unknown at CORE_UNKNOWN, got {:?}",
        type_info.ty
    );
}

#[test]
fn core_boundaries_follow_builtins() {
    let compiler = core_only_compiler();

    // Boundaries are pushed directly after the unknown type
    let first = CORE_UNKNOWN + 1;

    for (idx, (interned, flags)) in CORE_BOUNDARIES_DATASET.into_iter().enumerate() {
        let type_id = TypeId::new(first + idx as u32);
        let type_info = &compiler.types[type_id];

        let Type::Boundaries(found) = type_info.ty else {
            panic!(
                "expected boundary type at {type_id:?}, got {:?}",
                type_info.ty
            );
        };

        assert_eq!(found, flags, "boundary at {type_id:?} does not match");

        let sym = compiler
            .syms
            .items
            .iter()
            .find(|sym| sym.name_id == InternedId::new(interned))
            .expect("boundary symbol should exist");

        assert!(
            matches!(sym.kind, SymbolKind::Type(id) if id == type_id),
            "boundary symbol does not point at its type"
        );
    }
}

/// `TypeId` of the first core function, which is pushed after the builtins, the unknown type and
/// the boundaries
fn first_core_func_type_id() -> u32 {
    CORE_UNKNOWN + 1 + CORE_BOUNDARIES_DATASET.len() as u32
}

#[test]
fn core_funcs_follow_boundaries() {
    let compiler = core_only_compiler();
    let first = first_core_func_type_id();

    for (idx, core_func) in CORE_FUNCS_DATASET.iter().enumerate() {
        let type_id = TypeId::new(first + idx as u32);
        let type_info = &compiler.types[type_id];

        let Type::Func(func_def) = &type_info.ty else {
            panic!("expected func type at {type_id:?}, got {:?}", type_info.ty);
        };

        assert_eq!(func_def.kind, core_func.kind, "kind at {type_id:?}");
        assert_eq!(
            func_def.name_id,
            InternedId::new(core_func.name),
            "name at {type_id:?}"
        );
        assert_eq!(
            func_def.kind.form().is_callable(),
            core_func.kind.form().is_callable(),
            "is_callable at {type_id:?}"
        );
        assert_eq!(
            func_def.affects_type_constraint, core_func.affects_type_constraint,
            "affects_type_constraint at {type_id:?}"
        );
        assert_eq!(
            func_def.type_constraints, core_func.type_constraints,
            "type_constraints at {type_id:?}"
        );
        assert_eq!(
            func_def.arg_constraints, core_func.arg_constraints,
            "arg_constraints at {type_id:?}"
        );
        assert_eq!(
            func_def.ret_type,
            TypeId::new(core_func.ret_type),
            "ret_type at {type_id:?}"
        );
    }
}

#[test]
fn core_funcs_are_symbols_in_core_scope() {
    let compiler = core_only_compiler();
    let first = first_core_func_type_id();

    let core_mod_id = compiler.intrinsic_registry.core_mod_id;
    let core_scope_id = compiler.extract_scope_id(ScopeType::Core, core_mod_id);
    let table = &compiler.get_scope(core_scope_id).scope.table;

    for (idx, core_func) in CORE_FUNCS_DATASET.iter().enumerate() {
        let type_id = TypeId::new(first + idx as u32);
        let name_id = InternedId::new(core_func.name);

        let sym_id = *table
            .interned_to_sym
            .get(&name_id)
            .unwrap_or_else(|| panic!("core func {:?} is not in the core scope", core_func.kind));

        let sym = &compiler.syms[sym_id];

        assert_eq!(sym.name_id, name_id, "symbol name for {:?}", core_func.kind);
        assert!(
            matches!(sym.kind, SymbolKind::Type(id) if id == type_id),
            "symbol for {:?} does not point at {type_id:?}",
            core_func.kind
        );

        let Type::Func(func_def) = &compiler.types[type_id].ty else {
            panic!("expected func type at {type_id:?}");
        };

        assert_eq!(
            func_def.self_id.inner(),
            sym_id,
            "func def for {:?} does not point back at its symbol",
            core_func.kind
        );
    }
}

#[test]
fn core_func_return_types_are_loaded() {
    let compiler = core_only_compiler();

    // Return types are indexes into types loaded before the funcs themselves
    for core_func in &CORE_FUNCS_DATASET {
        assert!(
            core_func.ret_type < first_core_func_type_id(),
            "return type of {:?} is not loaded before the funcs",
            core_func.kind
        );

        let type_info = &compiler.types[TypeId::new(core_func.ret_type)];
        assert!(
            matches!(type_info.ty, Type::BuiltinTypeInfo(_)),
            "return type of {:?} is not a builtin, got {:?}",
            core_func.kind,
            type_info.ty
        );
    }
}

#[test]
fn startup_reservations_match_loaded_data() {
    let compiler = core_only_compiler();
    let ns_counts = core_instantiation_reservations();

    let expected_type_count = CORE_BUILTIN_TYPES_DATASET.len()
        + CORE_BOUNDARIES_DATASET.len()
        + CORE_FUNCS_DATASET.len()
        + 1;
    assert_eq!(compiler.types.len(), expected_type_count);
    assert_eq!(compiler.types.capacity(), expected_type_count);

    // A module binds its own name plus one identifier per import -- the alias when given, the file
    // name otherwise. An identifier already bound in the module's scope is filtered out, which is
    // what happens to the implicit `core` import inside the `core` module itself.
    let expected_module_symbol_count: usize = compiler
        .mods
        .iter()
        .map(|module| {
            let mut idents: Vec<InternedId> = vec![module.name_id];
            for import in &module.imports {
                let ident = import
                    .sp_alias_id
                    .as_ref()
                    .map(|alias| alias.inner)
                    .unwrap_or(import.name_id);
                if !idents.contains(&ident) {
                    idents.push(ident);
                }
            }
            idents.len()
        })
        .sum();
    let expected_symbol_count = CORE_BUILTIN_TYPES_DATASET.len()
        + CORE_BOUNDARIES_DATASET.len()
        + CORE_FUNCS_DATASET.len()
        + DIRECTIVES_DATASET.len()
        + expected_module_symbol_count
        + ns_counts.symbols;
    assert_eq!(compiler.syms.len(), expected_symbol_count);
    assert_eq!(compiler.syms.capacity(), expected_symbol_count);

    assert_eq!(compiler.directives.len(), DIRECTIVES_DATASET.len());
    assert_eq!(compiler.directives.capacity(), DIRECTIVES_DATASET.len());

    // `load_core` creates one core scope plus a scope for every built-in carrying an intrinsic
    // namespace; module-symbol registration creates one compiler scope for every module, including
    // the implicit core module.
    let expected_scope_count = compiler.mods.len() + 1 + ns_counts.scopes;
    assert_eq!(compiler.scopes.len(), expected_scope_count);
    assert_eq!(compiler.scopes.capacity(), expected_scope_count);

    let core_mod = &compiler.mods[compiler.intrinsic_registry.core_mod_id];
    let expected_core_exports =
        CORE_BUILTIN_TYPES_DATASET.len() + CORE_BOUNDARIES_DATASET.len() + CORE_FUNCS_DATASET.len();
    assert_eq!(core_mod.exports.len(), expected_core_exports);
    assert_eq!(core_mod.exports.capacity(), expected_core_exports);
}

#[test]
fn startup_reservations_include_user_module_symbols() {
    let import = Import::new(
        InternedId::new(0),
        ImportKind::Core(Default::default()),
        Some(SpannedContainer::new(
            InternedId::new(1),
            SourceSpan::default(),
        )),
    );
    let module = Module::new(
        InternedId::new(2),
        Default::default(),
        Default::default(),
        None,
        vec![import],
        None,
    );
    let compiler = ScriptCompiler::init(None, Arena::from(vec![module]));
    let ns_counts = core_instantiation_reservations();

    // A module binds its own name plus one identifier per import -- the alias when given, the file
    // name otherwise. An identifier already bound in the module's scope is filtered out, which is
    // what happens to the implicit `core` import inside the `core` module itself.
    let expected_module_symbol_count: usize = compiler
        .mods
        .iter()
        .map(|module| {
            let mut idents: Vec<InternedId> = vec![module.name_id];
            for import in &module.imports {
                let ident = import
                    .sp_alias_id
                    .as_ref()
                    .map(|alias| alias.inner)
                    .unwrap_or(import.name_id);
                if !idents.contains(&ident) {
                    idents.push(ident);
                }
            }
            idents.len()
        })
        .sum();
    let expected_symbol_count = CORE_BUILTIN_TYPES_DATASET.len()
        + CORE_BOUNDARIES_DATASET.len()
        + CORE_FUNCS_DATASET.len()
        + DIRECTIVES_DATASET.len()
        + expected_module_symbol_count
        + ns_counts.symbols;

    assert_eq!(compiler.syms.len(), expected_symbol_count);
    assert_eq!(compiler.syms.capacity(), expected_symbol_count);
    let expected_scope_count = compiler.mods.len() + 1 + ns_counts.scopes;
    assert_eq!(compiler.scopes.len(), expected_scope_count);
    assert_eq!(compiler.scopes.capacity(), expected_scope_count);
}

// -- Intrinsic namespaces --

/// Both halves of a `MAX`/`MIN` pair, as they appear in the compiler after `load_core`.
struct RegisteredConstant<'a> {
    sym: &'a Symbol,
    var: &'a VarDef,
    val: &'a ValueInfo,
    expr: &'a ResolvedExpr,
}

/// Looks up `name_id` in the scope a namespaced built-in owns and gathers everything
/// `register_instantiation_var` pushed for it.
fn registered_constant<'a>(
    compiler: &'a ScriptCompiler,
    scope_id: ScopeId,
    name_id: InternedId,
) -> RegisteredConstant<'a> {
    let sym_id = compiler.get_scope(scope_id).scope.table.interned_to_sym[&name_id];
    let sym = &compiler.syms[sym_id];

    let SymbolKind::Variable(var_id) = sym.kind else {
        panic!(
            "`{name_id:?}` in {scope_id:?} is not a variable, got {:?}",
            sym.kind
        );
    };

    let var = &compiler.vars[var_id];

    let VariableState::Known(val_id) = var.state else {
        panic!("`{name_id:?}` in {scope_id:?} has no known value");
    };

    let val = &compiler.values[val_id];
    let expr = &compiler.exprs[val.expr_id];

    RegisteredConstant {
        sym,
        var,
        val,
        expr,
    }
}

/// The scope a namespaced built-in owns, or `None` when the entry declares no namespace.
/// Panics when the two disagree, since `register_builtin` derives one from the other.
fn builtin_ns_scope(
    compiler: &ScriptCompiler,
    type_id: TypeId,
    ns: &[InstantiationSymbolBase],
) -> Option<ScopeId> {
    let Type::BuiltinTypeInfo(info) = &compiler.types[type_id].ty else {
        panic!(
            "expected a builtin at {type_id:?}, got {:?}",
            compiler.types[type_id].ty
        );
    };

    match compiler.syms[info.sym_id].associated_scope {
        Some(AssociatedScopeKind::Scope(scope_id)) => {
            assert!(
                !ns.is_empty(),
                "builtin at {type_id:?} declares no namespace but owns {scope_id:?}"
            );
            Some(scope_id)
        }
        None => {
            assert!(
                ns.is_empty(),
                "builtin at {type_id:?} declares a namespace but owns no scope"
            );
            None
        }
        other => panic!("builtin at {type_id:?} has a non-scope association: {other:?}"),
    }
}

/// `Value` has no `PartialEq`, and only the variants `InstiationValue` can produce are compared.
fn same_value(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::ArbitraryInt(l), Value::ArbitraryInt(r)) => l == r,
        // Bit equality rather than `==` so a bound that lost precision on the way in fails
        (Value::ArbitraryFloat(l), Value::ArbitraryFloat(r)) => l.to_bits() == r.to_bits(),
        (Value::Bool(l), Value::Bool(r)) => l == r,
        (Value::Char(l), Value::Char(r)) => l == r,
        (Value::InternedStr(l), Value::InternedStr(r)) => l == r,
        _ => false,
    }
}

#[test]
fn core_namespaced_builtins_own_a_core_scope() {
    let compiler = core_only_compiler();
    let core_mod_id = compiler.intrinsic_registry.core_mod_id;

    for (idx, (_, builtin_ty, ns)) in CORE_BUILTIN_TYPES_DATASET.iter().enumerate() {
        let type_id = TypeId::new(idx as u32);
        let Some(scope_id) = builtin_ns_scope(&compiler, type_id, ns) else {
            continue;
        };

        let Type::BuiltinTypeInfo(info) = &compiler.types[type_id].ty else {
            unreachable!("checked by `builtin_ns_scope`");
        };
        let scope_info = compiler.get_scope(scope_id);

        assert_eq!(
            scope_info.sym_owner,
            Some(info.sym_id),
            "namespace scope of {:?} is not owned by its builtin's symbol",
            builtin_ty.kind()
        );
        assert_eq!(
            scope_info.mod_owner,
            core_mod_id,
            "namespace scope of {:?} is not owned by the core module",
            builtin_ty.kind()
        );
        assert_eq!(
            scope_info.scope.scope_type,
            ScopeType::Core,
            "namespace scope of {:?} is not a core scope",
            builtin_ty.kind()
        );
        assert_eq!(
            scope_info.scope.table.interned_to_sym.len(),
            ns.len(),
            "namespace scope of {:?} does not hold exactly its dataset entries",
            builtin_ty.kind()
        );
    }
}

/// Existing float namespace order is compatibility-sensitive. New metadata must be appended so
/// the positional identities of the original constants remain stable.
#[test]
fn core_float_namespace_ieee_metadata_is_append_only() {
    let legacy_prefix = [
        intern::INTERNED_MAX_UPPER,
        intern::INTERNED_MIN_UPPER,
        intern::INTERNED_BITS_UPPER,
        intern::INTERNED_BYTES_UPPER,
        intern::INTERNED_RADIX,
        intern::INTERNED_DIGITS,
        intern::INTERNED_MANTISSA_DIGITS,
        intern::INTERNED_EPSILON,
        intern::INTERNED_INF,
        intern::INTERNED_NEG_INF,
        intern::INTERNED_NAN,
        intern::INTERNED_MIN_POSITIVE,
        intern::INTERNED_PI_UPPER,
        intern::INTERNED_E_UPPER,
        intern::INTERNED_TAU,
        intern::INTERNED_FRAC_1_PI,
        intern::INTERNED_FRAC_1_SQRT_2,
        intern::INTERNED_FRAC_2_PI,
        intern::INTERNED_FRAC_2_SQRT_PI,
        intern::INTERNED_FRAC_PI_2,
        intern::INTERNED_FRAC_PI_3,
        intern::INTERNED_FRAC_PI_4,
        intern::INTERNED_FRAC_PI_6,
        intern::INTERNED_FRAC_PI_8,
        intern::INTERNED_LN_2,
        intern::INTERNED_LN_10,
        intern::INTERNED_LOG2_10,
        intern::INTERNED_LOG2_E,
        intern::INTERNED_LOG10_2,
        intern::INTERNED_LOG10_E,
        intern::INTERNED_SQRT_2,
        intern::INTERNED_SQRT_3,
        intern::INTERNED_GOLDEN_RATIO,
        intern::INTERNED_EULER_GAMMA,
    ];
    let ieee_suffix = [
        intern::INTERNED_SIGN_BITS,
        intern::INTERNED_EXPONENT_BITS,
        intern::INTERNED_SIGNIFICAND_BITS,
        intern::INTERNED_STORED_SIGNIFICAND_BITS,
        intern::INTERNED_EXPONENT_BIAS,
        intern::INTERNED_MIN_NORMAL_EXPONENT,
        intern::INTERNED_MAX_NORMAL_EXPONENT,
        intern::INTERNED_MIN_SUBNORMAL_EXPONENT,
    ];

    for kind in [
        BuiltinTypeKind::F16,
        BuiltinTypeKind::F32,
        BuiltinTypeKind::F64,
    ] {
        let (_, _, namespace) = CORE_BUILTIN_TYPES_DATASET
            .iter()
            .find(|(_, builtin_ty, _)| builtin_ty.kind() == kind)
            .expect("IEEE float type should be present in the core dataset");
        let names: Vec<u32> = namespace.iter().map(|base| base.name_id.id).collect();

        assert!(
            names.starts_with(&legacy_prefix),
            "{kind:?} legacy namespace prefix changed"
        );
        assert_eq!(
            names.get(legacy_prefix.len()..legacy_prefix.len() + ieee_suffix.len()),
            Some(ieee_suffix.as_slice()),
            "{kind:?} IEEE metadata order changed"
        );
    }

    let f128_prefix = [
        intern::INTERNED_BITS_UPPER,
        intern::INTERNED_BYTES_UPPER,
        intern::INTERNED_RADIX,
        intern::INTERNED_DIGITS,
        intern::INTERNED_MANTISSA_DIGITS,
    ];
    let (_, _, f128_namespace) = CORE_BUILTIN_TYPES_DATASET
        .iter()
        .find(|(_, builtin_ty, _)| builtin_ty.kind() == BuiltinTypeKind::F128)
        .expect("f128 should be present in the core dataset");
    let names: Vec<u32> = f128_namespace.iter().map(|base| base.name_id.id).collect();

    assert!(names.starts_with(&f128_prefix), "F128 prefix changed");
    assert_eq!(
        names.get(f128_prefix.len()..f128_prefix.len() + ieee_suffix.len()),
        Some(ieee_suffix.as_slice()),
        "F128 IEEE metadata order changed"
    );
}

#[test]
fn representative_core_namespace_values_are_registered() {
    let compiler = core_only_compiler();
    let cases = [
        (
            BuiltinTypeKind::I8,
            intern::INTERNED_MIN_UPPER,
            BuiltinTypeKind::I64,
            Value::ArbitraryInt(ArbitraryIntKind::I64(i8::MIN as i64)),
        ),
        (
            BuiltinTypeKind::U64,
            intern::INTERNED_MAX_UPPER,
            BuiltinTypeKind::U64,
            Value::ArbitraryInt(ArbitraryIntKind::U64(u64::MAX)),
        ),
        (
            BuiltinTypeKind::U128,
            intern::INTERNED_MAX_UPPER,
            BuiltinTypeKind::BigInt,
            Value::ArbitraryInt(ArbitraryIntKind::BigInt(IBig::from_parts_const(
                dashu_int::Sign::Positive,
                u128::MAX,
            ))),
        ),
        (
            BuiltinTypeKind::F16,
            intern::INTERNED_PI_UPPER,
            BuiltinTypeKind::F64,
            Value::ArbitraryFloat(ArbitraryFloatKind::F64(3.140625)),
        ),
        (
            BuiltinTypeKind::Char,
            intern::INTERNED_NUL,
            BuiltinTypeKind::Char,
            Value::Char('\0'),
        ),
        (
            BuiltinTypeKind::Str,
            intern::INTERNED_NEWLINE_UPPER,
            BuiltinTypeKind::Str,
            Value::InternedStr(InternedId::new(intern::INTERNED_NEWLINE_VALUE)),
        ),
    ];

    for (owner_kind, name, value_kind, expected) in cases {
        let (_, _, namespace) = CORE_BUILTIN_TYPES_DATASET
            .iter()
            .find(|(_, builtin_ty, _)| builtin_ty.kind() == owner_kind)
            .unwrap();
        let scope_id = builtin_ns_scope(
            &compiler,
            TypeId::new(builtin_ty_to_id(owner_kind)),
            namespace,
        )
        .unwrap();
        let found = registered_constant(&compiler, scope_id, InternedId::new(name));

        assert_eq!(found.sym.name_id, InternedId::new(name));
        assert!(matches!(found.sym.sym_origin, SymbolOrigin::Compiler));
        assert!(matches!(found.sym.kind, SymbolKind::Variable(_)));
        assert!(matches!(found.var.meta, VariableMetadata::Generated));
        assert_eq!(found.val.type_id, TypeId::new(builtin_ty_to_id(value_kind)));
        assert!(same_value(found.val.const_val.as_ref().unwrap(), &expected));
        assert_eq!(found.expr.type_id, found.val.type_id);
        assert!(matches!(found.expr.expr_hir, ExprHir::Val(val_id) if val_id == found.expr.val_id));
        assert!(matches!(found.expr.meta, ResolvedExprMetadata::Generated));
        assert_eq!(
            compiler.values[found.expr.val_id].expr_id,
            found.val.expr_id
        );
    }
}

#[test]
fn core_namespace_constants_stay_out_of_the_core_scope() {
    let compiler = core_only_compiler();
    let core_mod_id = compiler.intrinsic_registry.core_mod_id;
    let core_scope_id = compiler.extract_scope_id(ScopeType::Core, core_mod_id);
    let table = &compiler.get_scope(core_scope_id).scope.table;

    // Intrinsic namespace constants are only reachable through the built-in that owns them.
    // Leaking them into the core scope would make them resolve bare, and would collide across the ten namespaces
    for (_, _, ns) in &CORE_BUILTIN_TYPES_DATASET {
        for base in ns.iter() {
            assert!(
                !table.interned_to_sym.contains_key(&base.name_id),
                "core scope must not bind the intrinsic constant at interned id {}",
                base.name_id.id
            );
        }
    }

    let core_mod = &compiler.mods[core_mod_id];
    assert_eq!(
        core_mod.exports.len(),
        table.interned_to_sym.len(),
        "core exports must be exactly what the core scope binds"
    );
}

#[test]
fn core_namespace_arenas_are_reserved_exactly() {
    let compiler = core_only_compiler();
    let ns_counts = core_instantiation_reservations();

    // One `VarDef`, one `ResolvedExpr`, and one `ValueInfo` per constant, and nothing else has
    // been pushed yet, so the arenas are exactly as long as they were reserved
    for (label, len, capacity) in [
        ("variables", compiler.vars.len(), compiler.vars.capacity()),
        ("exprs", compiler.exprs.len(), compiler.exprs.capacity()),
        ("values", compiler.values.len(), compiler.values.capacity()),
    ] {
        assert_eq!(len, ns_counts.variables, "{label} length");
        assert_eq!(capacity, ns_counts.variables, "{label} capacity");
    }

    for (idx, var) in compiler.vars.iter().enumerate() {
        let var_id = VariableId::new(idx as u32);
        let sym = &compiler.syms[var.self_id.inner()];

        assert!(
            matches!(sym.kind, SymbolKind::Variable(found) if found == var_id),
            "variable {idx} and its symbol do not point at each other"
        );
    }
}

// -- `count_instantiation_bases` --

static COUNT_LEAF_DATASET: [InstantiationSymbolBase; 2] = [
    InstantiationSymbolBase::new(
        InternedId::new(intern::INTERNED_MAX_UPPER),
        SymbolOrigin::Compiler,
        ScopeType::Compiler,
        false,
        InstantiationSymbolKind::Variable(InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::I8),
            InstiationValue::I64(1),
        )),
    ),
    InstantiationSymbolBase::new(
        InternedId::new(intern::INTERNED_INT),
        SymbolOrigin::Compiler,
        ScopeType::Compiler,
        false,
        InstantiationSymbolKind::ExternType(ExternPlatformType::Java(JavaTypeKind::Int)),
    ),
];

static COUNT_ROOT_DATASET: [InstantiationSymbolBase; 2] = [
    InstantiationSymbolBase::new(
        InternedId::new(intern::INTERNED_TYPES_LOWER),
        SymbolOrigin::Compiler,
        ScopeType::Compiler,
        false,
        InstantiationSymbolKind::Namespace(&COUNT_LEAF_DATASET),
    ),
    InstantiationSymbolBase::new(
        InternedId::new(intern::INTERNED_MIN_UPPER),
        SymbolOrigin::Compiler,
        ScopeType::Compiler,
        false,
        InstantiationSymbolKind::Variable(InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::I8),
            InstiationValue::I64(0),
        )),
    ),
];

/// The core dataset is flat, so nesting is only covered here. `register_instantiation_bases`
/// descends into a namespace, which the reservation counts must follow.
#[test]
fn count_instantiation_bases_descends_into_namespaces() {
    let counts = count_instantiation_bases(&COUNT_ROOT_DATASET);

    // Namespace, its two members, and the root variable
    assert_eq!(counts.symbols, 4);
    // Only the nested namespace owns one; the root's own scope belongs to its caller
    assert_eq!(counts.scopes, 1);
    // The extern type contributes no value
    assert_eq!(counts.variables, 2);
}

#[test]
fn representative_float_math_constants_are_loaded() {
    let compiler = core_only_compiler();

    // `f16` has no stable Rust source, so these are the `f64` constants rounded to binary16,
    // spelled out as the exact `f64` holding each rounded value.
    let expected_f16 = [
        (intern::INTERNED_PI_UPPER, 3.140625),
        (intern::INTERNED_EULER_GAMMA, 0.5771484375),
    ];

    let expected_f32 = [
        (intern::INTERNED_PI_UPPER, std::f32::consts::PI as f64),
        (
            intern::INTERNED_EULER_GAMMA,
            std::f32::consts::EULER_GAMMA as f64,
        ),
    ];

    let expected_f64 = [
        (intern::INTERNED_PI_UPPER, std::f64::consts::PI),
        (intern::INTERNED_EULER_GAMMA, std::f64::consts::EULER_GAMMA),
    ];

    let f16_entry = CORE_BUILTIN_TYPES_DATASET
        .iter()
        .find(|(_, b, _)| b.kind() == BuiltinTypeKind::F16)
        .unwrap();
    let f16_scope = builtin_ns_scope(
        &compiler,
        TypeId::new(builtin_ty_to_id(BuiltinTypeKind::F16)),
        f16_entry.2,
    )
    .unwrap();

    for (name_id, expected_val) in expected_f16 {
        let rc = registered_constant(&compiler, f16_scope, InternedId::new(name_id));
        assert_eq!(
            rc.val.type_id,
            TypeId::new(builtin_ty_to_id(BuiltinTypeKind::F64))
        );
        assert!(same_value(
            rc.val.const_val.as_ref().unwrap(),
            &Value::ArbitraryFloat(ArbitraryFloatKind::F64(expected_val))
        ));
    }

    let f32_entry = CORE_BUILTIN_TYPES_DATASET
        .iter()
        .find(|(_, b, _)| b.kind() == BuiltinTypeKind::F32)
        .unwrap();
    let f32_scope = builtin_ns_scope(
        &compiler,
        TypeId::new(builtin_ty_to_id(BuiltinTypeKind::F32)),
        f32_entry.2,
    )
    .unwrap();

    for (name_id, expected_val) in expected_f32 {
        let rc = registered_constant(&compiler, f32_scope, InternedId::new(name_id));
        assert_eq!(
            rc.val.type_id,
            TypeId::new(builtin_ty_to_id(BuiltinTypeKind::F64))
        );
        assert!(same_value(
            rc.val.const_val.as_ref().unwrap(),
            &Value::ArbitraryFloat(ArbitraryFloatKind::F64(expected_val))
        ));
    }

    let f64_entry = CORE_BUILTIN_TYPES_DATASET
        .iter()
        .find(|(_, b, _)| b.kind() == BuiltinTypeKind::F64)
        .unwrap();
    let f64_scope = builtin_ns_scope(
        &compiler,
        TypeId::new(builtin_ty_to_id(BuiltinTypeKind::F64)),
        f64_entry.2,
    )
    .unwrap();

    for (name_id, expected_val) in expected_f64 {
        let rc = registered_constant(&compiler, f64_scope, InternedId::new(name_id));
        assert_eq!(
            rc.val.type_id,
            TypeId::new(builtin_ty_to_id(BuiltinTypeKind::F64))
        );
        assert!(same_value(
            rc.val.const_val.as_ref().unwrap(),
            &Value::ArbitraryFloat(ArbitraryFloatKind::F64(expected_val))
        ));
    }
}

#[test]
fn representative_core_namespaces_declare_bits_and_bytes() {
    let compiler = core_only_compiler();

    let expected_bits_bytes: &[(BuiltinTypeKind, i64, i64)] = &[
        (BuiltinTypeKind::I8, 8, 1),
        (BuiltinTypeKind::F32, 32, 4),
        (BuiltinTypeKind::U128, 128, 16),
        (BuiltinTypeKind::Bool, 1, 1),
        (BuiltinTypeKind::Char, 32, 4),
    ];

    let i64_type_id = TypeId::new(builtin_ty_to_id(BuiltinTypeKind::I64));

    for &(kind, expected_bits, expected_bytes) in expected_bits_bytes {
        let type_id = TypeId::new(builtin_ty_to_id(kind));
        let entry = CORE_BUILTIN_TYPES_DATASET
            .iter()
            .find(|(_, b, _)| b.kind() == kind)
            .unwrap();
        let scope_id = builtin_ns_scope(&compiler, type_id, entry.2).unwrap();

        let bits_rc = registered_constant(
            &compiler,
            scope_id,
            InternedId::new(intern::INTERNED_BITS_UPPER),
        );
        assert_eq!(bits_rc.val.type_id, i64_type_id);
        assert!(same_value(
            bits_rc.val.const_val.as_ref().unwrap(),
            &Value::ArbitraryInt(ArbitraryIntKind::I64(expected_bits))
        ));

        let bytes_rc = registered_constant(
            &compiler,
            scope_id,
            InternedId::new(intern::INTERNED_BYTES_UPPER),
        );
        assert_eq!(bytes_rc.val.type_id, i64_type_id);
        assert!(same_value(
            bytes_rc.val.const_val.as_ref().unwrap(),
            &Value::ArbitraryInt(ArbitraryIntKind::I64(expected_bytes))
        ));
    }
}

#[test]
fn float_namespace_declares_radix_but_integer_namespace_does_not() {
    let compiler = core_only_compiler();
    let i64_type_id = TypeId::new(builtin_ty_to_id(BuiltinTypeKind::I64));

    for (kind, expected) in [
        (BuiltinTypeKind::F32, Some(2)),
        (BuiltinTypeKind::I32, None),
    ] {
        let (_, _, namespace) = CORE_BUILTIN_TYPES_DATASET
            .iter()
            .find(|(_, builtin_ty, _)| builtin_ty.kind() == kind)
            .unwrap();
        let scope_id =
            builtin_ns_scope(&compiler, TypeId::new(builtin_ty_to_id(kind)), namespace).unwrap();
        let has_radix = compiler.scopes[scope_id]
            .scope
            .table
            .interned_to_sym
            .contains_key(&InternedId::new(intern::INTERNED_RADIX));

        if let Some(expected) = expected {
            assert!(has_radix);
            let radix_rc =
                registered_constant(&compiler, scope_id, InternedId::new(intern::INTERNED_RADIX));
            assert_eq!(radix_rc.val.type_id, i64_type_id);
            assert!(same_value(
                radix_rc.val.const_val.as_ref().unwrap(),
                &Value::ArbitraryInt(ArbitraryIntKind::I64(expected))
            ));
        } else {
            assert!(!has_radix);
        }
    }
}

#[test]
fn representative_char_and_str_constants_are_loaded() {
    let compiler = core_only_compiler();
    let interner = Intern::init();
    let i64_type_id = TypeId::new(builtin_ty_to_id(BuiltinTypeKind::I64));
    let char_type_id = TypeId::new(builtin_ty_to_id(BuiltinTypeKind::Char));
    let str_type_id = TypeId::new(builtin_ty_to_id(BuiltinTypeKind::Str));

    let entry = |kind: BuiltinTypeKind| {
        CORE_BUILTIN_TYPES_DATASET
            .iter()
            .find(|(_, b, _)| b.kind() == kind)
            .unwrap_or_else(|| panic!("{kind:?} entry in dataset"))
    };

    let check_i64 = |scope_id, name_id: u32, expected: i64| {
        let rc = registered_constant(&compiler, scope_id, InternedId::new(name_id));
        assert_eq!(rc.val.type_id, i64_type_id);
        assert!(same_value(
            rc.val.const_val.as_ref().unwrap(),
            &Value::ArbitraryInt(ArbitraryIntKind::I64(expected))
        ));
    };

    let char_ns = entry(BuiltinTypeKind::Char).2;
    let char_scope = builtin_ns_scope(
        &compiler,
        TypeId::new(builtin_ty_to_id(BuiltinTypeKind::Char)),
        char_ns,
    )
    .unwrap();
    for (name_id, expected) in [
        (intern::INTERNED_NUL, '\0'),
        (
            intern::INTERNED_REPLACEMENT_CHARACTER_UPPER,
            char::REPLACEMENT_CHARACTER,
        ),
    ] {
        let rc = registered_constant(&compiler, char_scope, InternedId::new(name_id));
        assert_eq!(rc.val.type_id, char_type_id);
        assert!(same_value(
            rc.val.const_val.as_ref().unwrap(),
            &Value::Char(expected)
        ));
    }
    check_i64(char_scope, intern::INTERNED_UTF16_MAX_CODE_UNITS, 2);

    let str_ns = entry(BuiltinTypeKind::Str).2;
    let str_scope = builtin_ns_scope(
        &compiler,
        TypeId::new(builtin_ty_to_id(BuiltinTypeKind::Str)),
        str_ns,
    )
    .unwrap();
    let newline_name = interner
        .try_search_str("NEWLINE")
        .expect("NEWLINE should be a compiler-known identifier");
    let newline = registered_constant(&compiler, str_scope, newline_name);
    assert_eq!(newline.val.type_id, str_type_id);
    let Value::InternedStr(newline_value) = newline.val.const_val.as_ref().unwrap() else {
        panic!("str::NEWLINE should store an interned string")
    };
    assert_eq!(interner.search_idx(newline_value.id as usize), "\n");
    check_i64(str_scope, intern::INTERNED_UTF8_MAX_BYTES, 4);
}

/// Protects the public float namespaces' numeric attributes and IEEE 754 binary interchange
/// format layout. In particular, `SIGNIFICAND_BITS` includes the implicit leading bit while
/// `STORED_SIGNIFICAND_BITS` counts only the fraction field physically present in the encoding.
#[test]
fn representative_float_attributes_are_loaded() {
    let compiler = core_only_compiler();
    let interner = Intern::init();
    let i64_type_id = TypeId::new(builtin_ty_to_id(BuiltinTypeKind::I64));
    let f64_type_id = TypeId::new(builtin_ty_to_id(BuiltinTypeKind::F64));

    let f16_type_id = TypeId::new(builtin_ty_to_id(BuiltinTypeKind::F16));
    let f16_entry = CORE_BUILTIN_TYPES_DATASET
        .iter()
        .find(|(_, b, _)| b.kind() == BuiltinTypeKind::F16)
        .unwrap();
    let f16_scope = builtin_ns_scope(&compiler, f16_type_id, f16_entry.2).unwrap();

    let check_f16_i64 = |name: &str, expected: i64| {
        let name_id = interner
            .try_search_str(name)
            .unwrap_or_else(|| panic!("{name} should be a compiler-known identifier"));
        let rc = registered_constant(&compiler, f16_scope, name_id);
        assert_eq!(rc.val.type_id, i64_type_id);
        assert!(same_value(
            rc.val.const_val.as_ref().unwrap(),
            &Value::ArbitraryInt(ArbitraryIntKind::I64(expected))
        ));
    };
    let check_f16_f64 = |name: &str, expected: f64| {
        let name_id = interner
            .try_search_str(name)
            .unwrap_or_else(|| panic!("{name} should be a compiler-known identifier"));
        let rc = registered_constant(&compiler, f16_scope, name_id);
        assert_eq!(rc.val.type_id, f64_type_id);
        assert!(same_value(
            rc.val.const_val.as_ref().unwrap(),
            &Value::ArbitraryFloat(ArbitraryFloatKind::F64(expected))
        ));
    };

    check_f16_i64("DIGITS", 3);
    check_f16_f64("EPSILON", 0.0009765625);

    let f32_type_id = TypeId::new(builtin_ty_to_id(BuiltinTypeKind::F32));
    let f32_entry = CORE_BUILTIN_TYPES_DATASET
        .iter()
        .find(|(_, b, _)| b.kind() == BuiltinTypeKind::F32)
        .unwrap();
    let f32_scope = builtin_ns_scope(&compiler, f32_type_id, f32_entry.2).unwrap();

    let check_f32_i64 = |name: &str, expected: i64| {
        let name_id = interner
            .try_search_str(name)
            .unwrap_or_else(|| panic!("{name} should be a compiler-known identifier"));
        let rc = registered_constant(&compiler, f32_scope, name_id);
        assert_eq!(rc.val.type_id, i64_type_id);
        assert!(same_value(
            rc.val.const_val.as_ref().unwrap(),
            &Value::ArbitraryInt(ArbitraryIntKind::I64(expected))
        ));
    };
    let check_f32_f64 = |name: &str, expected: f64| {
        let name_id = interner
            .try_search_str(name)
            .unwrap_or_else(|| panic!("{name} should be a compiler-known identifier"));
        let rc = registered_constant(&compiler, f32_scope, name_id);
        assert_eq!(rc.val.type_id, f64_type_id);
        assert!(same_value(
            rc.val.const_val.as_ref().unwrap(),
            &Value::ArbitraryFloat(ArbitraryFloatKind::F64(expected))
        ));
    };

    check_f32_i64("MANTISSA_DIGITS", std::f32::MANTISSA_DIGITS as i64);
    check_f32_f64("NAN", f32::NAN as f64);

    let f64_entry = CORE_BUILTIN_TYPES_DATASET
        .iter()
        .find(|(_, b, _)| b.kind() == BuiltinTypeKind::F64)
        .unwrap();
    let f64_scope = builtin_ns_scope(&compiler, f64_type_id, f64_entry.2).unwrap();

    let check_f64_f64 = |name: &str, expected: f64| {
        let name_id = interner
            .try_search_str(name)
            .unwrap_or_else(|| panic!("{name} should be a compiler-known identifier"));
        let rc = registered_constant(&compiler, f64_scope, name_id);
        assert_eq!(rc.val.type_id, f64_type_id);
        assert!(same_value(
            rc.val.const_val.as_ref().unwrap(),
            &Value::ArbitraryFloat(ArbitraryFloatKind::F64(expected))
        ));
    };

    check_f64_f64("INF", f64::INFINITY);
    check_f64_f64("NEG_INF", f64::NEG_INFINITY);
    check_f64_f64("MIN_POSITIVE", f64::MIN_POSITIVE);

    assert_eq!(interner.search_idx(intern::INTERNED_INF as usize), "INF");

    let ieee_layouts = [
        (BuiltinTypeKind::F16, 5, 11, 10, -24),
        (BuiltinTypeKind::F128, 15, 113, 112, -16494),
    ];

    for (kind, exponent_bits, significand_bits, stored_significand_bits, min_subnormal_exponent) in
        ieee_layouts
    {
        let entry = CORE_BUILTIN_TYPES_DATASET
            .iter()
            .find(|(_, builtin_ty, _)| builtin_ty.kind() == kind)
            .expect("IEEE float type should be present in the core dataset");
        let scope_id = builtin_ns_scope(&compiler, TypeId::new(builtin_ty_to_id(kind)), entry.2)
            .expect("IEEE float type should own a metadata namespace");

        for (name, expected) in [
            ("SIGN_BITS", 1),
            ("EXPONENT_BITS", exponent_bits),
            ("SIGNIFICAND_BITS", significand_bits),
            ("STORED_SIGNIFICAND_BITS", stored_significand_bits),
            ("MIN_SUBNORMAL_EXPONENT", min_subnormal_exponent),
        ] {
            let name_id = interner
                .try_search_str(name)
                .unwrap_or_else(|| panic!("{name} should be a compiler-known identifier"));
            let registered = registered_constant(&compiler, scope_id, name_id);

            assert_eq!(
                registered.val.type_id, i64_type_id,
                "{kind:?}::{name} should be typed as i64"
            );
            assert!(
                same_value(
                    registered.val.const_val.as_ref().unwrap(),
                    &Value::ArbitraryInt(ArbitraryIntKind::I64(expected)),
                ),
                "{kind:?}::{name} has the wrong IEEE 754 value"
            );
        }
    }
}
