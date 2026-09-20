use super::helpers::*;
use crate::parser::ast::ast_concepts::{AbstractDecl, AbstractVar, Item, SectionKind};
use crate::parser::ast::ast_exprs::{AstExpr, SpannedExpr};
use crate::script_compiler::compiler_constants::{
    CORE_BIGFLOAT, CORE_BIGINT, CORE_F64, CORE_I64, CORE_STR, CORE_U64, CORE_UNKNOWN,
};
use crate::semantic::arbitraries::{ArbitraryIntKind, int_from_i64};
use crate::semantic::hir::hir_concepts::Type;
use crate::semantic::hir::hir_exprs::ResolvedExpr;
use crate::semantic::hir::hir_impls::ImplMemberKind;
use crate::semantic::hir::hir_symbols::SymbolKind;
use crate::semantic::values::ValueInfo;
use crate::walk_type_id_deferred;
use chrn_utils::id_types::TypeId;
use chrn_utils::source_map::source_diagnostic::annotations::AnnotationKind;
use chrn_utils::source_map::source_span::SourceSpan;

fn config_with_numeric_limit(max_numeric_bits: u32) -> ChrnConfig {
    ChrnConfig::builder()
        .add_max_numeric_bits(max_numeric_bits)
        .build()
}

fn assert_numeric_limit_error(
    source: &str,
    max_numeric_bits: u32,
    expected_annotations: &[(u32, u32, &str)],
) {
    let resolution = resolve_single_module_with_config(
        source,
        Stage::Type,
        config_with_numeric_limit(max_numeric_bits),
    );

    assert_eq!(
        resolution.err_count(),
        1,
        "unexpected diagnostics: {:?}",
        resolution.ty
    );
    assert_eq!(resolution.ty.err_count(), 1);
    let diagnostic = &resolution.ty.diags[0];
    assert_eq!(diagnostic.level, DiagnosticLevel::Error);
    assert_eq!(
        diagnostic.core_msg,
        format!("Numeric value exceeds the configured {max_numeric_bits}-bit limit")
    );
    assert!(
        diagnostic.help.is_empty(),
        "numeric-limit diagnostics must remain interface-neutral: {:?}",
        diagnostic.help
    );
    assert_eq!(diagnostic.annotations.len(), expected_annotations.len());
    for (annotation, &(start, end, expected_text)) in
        diagnostic.annotations.iter().zip(expected_annotations)
    {
        assert_eq!(annotation.kind, AnnotationKind::Primary);
        assert_eq!(
            annotation.span,
            SourceSpan::new(SourceRegionId::new(0), start, end)
        );
        assert_eq!(
            &source[annotation.span.range_exclusive_usize()],
            expected_text
        );
    }
}

#[test]
fn chrn_config_default_uses_max_numeric_bits_constant() {
    assert_eq!(
        ChrnConfig::default().max_numeric_bits(),
        crate::DEFAULT_MAX_NUMERIC_BITS
    );
}

/// Protects the magnitude-bit boundary: 255 needs exactly eight bits, while 256 needs nine.
#[test]
fn type_resolver_enforces_configured_integer_literal_limit() {
    let accepted =
        resolve_single_module_with_config("let X = 255", Stage::Type, config_with_numeric_limit(8))
            .expect_ok();
    assert!(matches!(
        accepted.value_of("X"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(255))
    ));

    assert_numeric_limit_error("let X = 256", 8, &[(8, 11, "256")]);
}

/// Protects result checking after arithmetic on individually valid operands.
#[test]
fn type_resolver_rejects_multiplication_result_above_numeric_limit() {
    let accepted = resolve_single_module_with_config(
        "let X = 15 * 17",
        Stage::Type,
        config_with_numeric_limit(8),
    )
    .expect_ok();
    assert!(matches!(
        accepted.value_of("X"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(255))
    ));

    assert_numeric_limit_error("let X = 16 * 16", 8, &[(8, 10, "16"), (13, 15, "16")]);
}

/// Protects the pre-allocation left-shift bound as well as its exact accepted edge.
#[test]
fn type_resolver_rejects_left_shift_result_above_numeric_limit() {
    let accepted = resolve_single_module_with_config(
        "let X = 1 << 7",
        Stage::Type,
        config_with_numeric_limit(8),
    )
    .expect_ok();
    assert!(matches!(
        accepted.value_of("X"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(128))
    ));

    assert_numeric_limit_error("let X = 1 << 8", 8, &[(8, 9, "1"), (13, 14, "8")]);
}

/// Protects the absolute shift ceiling when the configured numeric limit is higher.
#[test]
fn type_resolver_rejects_left_shift_above_absolute_limit() {
    let shift = ArbitraryIntKind::MAX_SHIFT_BITS + 1;
    let shift_text = shift.to_string();
    let source = format!("let X = 1 << {shift_text}");
    let resolution = resolve_single_module_with_config(
        &source,
        Stage::Type,
        config_with_numeric_limit(shift + 1),
    );

    assert_eq!(
        resolution.err_count(),
        1,
        "unexpected diagnostics: {:?}",
        resolution.ty
    );
    assert_eq!(resolution.ty.err_count(), 1);
    let diagnostic = &resolution.ty.diags[0];
    assert_eq!(diagnostic.level, DiagnosticLevel::Error);
    assert_eq!(diagnostic.core_msg, "Shift amount is out of range");
    assert!(diagnostic.help.is_empty());
    assert_eq!(diagnostic.annotations.len(), 2);
    assert!(
        diagnostic
            .annotations
            .iter()
            .all(|annotation| annotation.kind == AnnotationKind::Primary)
    );
    let annotated_text = diagnostic
        .annotations
        .iter()
        .map(|annotation| &source[annotation.span.range_exclusive_usize()])
        .collect::<Vec<_>>();
    assert_eq!(annotated_text, ["1", shift_text.as_str()]);
}

/// Protects both the fixed `f64` representation boundary and arbitrary exponent accounting.
#[test]
fn type_resolver_enforces_numeric_limit_for_float_literals() {
    let accepted = resolve_single_module_with_config(
        "let X = 1.5",
        Stage::Type,
        config_with_numeric_limit(64),
    )
    .expect_ok();
    assert!(matches!(
        accepted.value_of("X"),
        Value::ArbitraryFloat(ArbitraryFloatKind::F64(value)) if value == 1.5
    ));

    assert_numeric_limit_error("let X = 1.5", 63, &[(8, 11, "1.5")]);
    assert_numeric_limit_error("let X = 1e1000", 64, &[(8, 14, "1e1000")]);
    assert_numeric_limit_error("let X = 1e-1000", 64, &[(8, 15, "1e-1000")]);
}

/// Protects float binary operation overflow and subsequent numeric limit enforcement.
#[test]
fn type_resolver_enforces_numeric_limit_for_float_operations() {
    assert_numeric_limit_error(
        "let X = 1e308 * 2.0",
        128,
        &[(8, 13, "1e308"), (16, 19, "2.0")],
    );

    assert_numeric_limit_error(
        "let X = 1e308 * 2.0",
        u32::MAX,
        &[(8, 13, "1e308"), (16, 19, "2.0")],
    );
}

/// Protects numeric-limit validation of the IEEE NaN produced by float remainder by zero.
#[test]
fn type_resolver_enforces_numeric_limit_for_float_remainder_by_zero() {
    assert_numeric_limit_error("let X = 5.5 % 0.0", 64, &[(8, 11, "5.5"), (14, 17, "0.0")]);
}

#[test]
fn type_resolver_simple_test() {
    let wrong = "
            var->
                primitive: i32
                undeclared_type: Thing
            ";

    let resolution = resolve_single_module(wrong, Stage::Type);
    let res = &resolution.ty;
    let compiler = &resolution.compiler;
    let interner = &resolution.interner;

    let err = &res.diags;
    assert_eq!(
        err.len(),
        1,
        "Expected exactly one diagnostic for undeclared type"
    );
    assert_eq!(err[0].level, DiagnosticLevel::Error);
    assert!(
        err[0].core_msg.contains("Thing"),
        "Error should mention 'Thing': {}",
        err[0].core_msg
    );

    // The undeclared type's typedef inner type should remain unknown
    let undeclared_sym = compiler
        .syms
        .iter()
        .find(|s| interner.search(s.name_id) == "undeclared_type")
        .expect("undeclared_type symbol should exist");
    let undeclared_type_id = match &undeclared_sym.kind {
        SymbolKind::Type(type_id) => *type_id,
        other => panic!("undeclared_type symbol should be Type, got {:?}", other),
    };
    let undeclared_ty = &compiler.types[undeclared_type_id].ty;
    match undeclared_ty {
        Type::TypeDef(type_def) => {
            assert_eq!(
                type_def.type_id,
                TypeId::new(CORE_UNKNOWN),
                "undeclared_type should resolve to unknown"
            );
        }
        other => panic!("Expected undeclared_type to be a TypeDef, got {:?}", other),
    }

    // primitive should still resolve correctly despite the error
    let primitive_sym = compiler
        .syms
        .iter()
        .find(|s| interner.search(s.name_id) == "primitive")
        .expect("primitive symbol should exist");
    let primitive_type_id = match &primitive_sym.kind {
        SymbolKind::Type(type_id) => *type_id,
        other => panic!("primitive symbol should be Type, got {:?}", other),
    };
    let primitive_ty = &compiler.types[primitive_type_id].ty;
    match primitive_ty {
        Type::TypeDef(type_def) => {
            assert!(
                !compiler.check_unknown(type_def.type_id),
                "primitive should resolve to a known type, not unknown"
            );
        }
        other => panic!("Expected primitive to be a TypeDef, got {:?}", other),
    }

    let correct = "
            var->
                primitive: i32
                declared_type: Thing
            nest->
                struct Thing {}
            ";

    let resolution = resolve_single_module(correct, Stage::Type);
    let res = &resolution.ty;
    let compiler = &resolution.compiler;
    let interner = &resolution.interner;
    dbg!(&res);

    assert!(res.err_count() == 0, "Type resolution should succeed");

    // Verify Thing is a struct with no fields
    let thing_sym = compiler
        .syms
        .iter()
        .find(|s| interner.search(s.name_id) == "Thing")
        .expect("Thing symbol should exist");
    let thing_type_id = match &thing_sym.kind {
        SymbolKind::Type(type_id) => *type_id,
        other => panic!("Thing symbol should be Type, got {:?}", other),
    };
    let thing_type = &compiler.types[thing_type_id].ty;
    let thing_fields = match thing_type {
        Type::Struct(struct_def) => &struct_def.fields,
        other => panic!("Expected Thing to be a struct, got {:?}", other),
    };
    assert!(
        thing_fields.is_empty(),
        "Thing struct should have no fields"
    );

    // Verify primitive typedef resolves to a known type (i32)
    let primitive_sym = compiler
        .syms
        .iter()
        .find(|s| interner.search(s.name_id) == "primitive")
        .expect("primitive symbol should exist");
    let primitive_type_id = match &primitive_sym.kind {
        SymbolKind::Type(type_id) => *type_id,
        other => panic!("primitive symbol should be Type, got {:?}", other),
    };
    let primitive_ty = &compiler.types[primitive_type_id].ty;
    match primitive_ty {
        Type::TypeDef(type_def) => {
            assert!(
                !compiler.check_unknown(type_def.type_id),
                "primitive should resolve to a known type"
            );
        }
        other => panic!("Expected primitive to be a TypeDef, got {:?}", other),
    }

    // Verify declared_type typedef resolves to Thing's struct type
    let declared_type_sym = compiler
        .syms
        .iter()
        .find(|s| interner.search(s.name_id) == "declared_type")
        .expect("declared_type symbol should exist");
    let declared_type_id = match &declared_type_sym.kind {
        SymbolKind::Type(type_id) => *type_id,
        other => panic!("declared_type symbol should be Type, got {:?}", other),
    };
    let declared_ty = &compiler.types[declared_type_id].ty;
    match declared_ty {
        Type::TypeDef(type_def) => {
            assert_eq!(
                type_def.type_id, thing_type_id,
                "declared_type should resolve to Thing struct type"
            );
        }
        other => panic!("Expected declared_type to be a TypeDef, got {:?}", other),
    }
}

#[test]
fn type_resolver_complex_test() {
    let text = "
            let CONSTANT = 4
            ";

    let resolution = resolve_single_module(text, Stage::Type);
    let summary = &resolution.ty;
    let compiler = &resolution.compiler;
    let interner = &resolution.interner;
    assert!(summary.err_count() == 0, "Type resolution failed");

    let constant_sym = compiler
        .syms
        .iter()
        .find(|s| interner.search(s.name_id) == "CONSTANT")
        .expect("CONSTANT symbol should exist");
    let var_id = match &constant_sym.kind {
        SymbolKind::Variable(var_id) => *var_id,
        other => panic!("CONSTANT symbol should be Variable, got {:?}", other),
    };
    let var_def = &compiler.vars[var_id];
    let val_id = match &var_def.state {
        VariableState::Known(val_id) => *val_id,
        other => panic!("CONSTANT should be Known, got {:?}", other),
    };
    let val_info = &compiler.values[val_id];
    assert_eq!(
        val_info.type_id,
        TypeId::new(CORE_I64),
        "CONSTANT should have i64 type"
    );
    assert!(
        matches!(
            val_info.const_val,
            Some(Value::ArbitraryInt(ArbitraryIntKind::I64(4)))
        ),
        "CONSTANT should have const value Some(I64(4)), got {:?}",
        val_info.const_val
    );
}

#[test]
fn type_resolver_string_concat_basic_test() {
    let (compiler, interner) = compile_and_resolve_single_module("let X = \"Hello\" + \" World\"");

    let val = value_of(&compiler, &interner, "X");
    match &val {
        Value::InternedStr(id) => {
            assert_eq!(
                interner.search(*id),
                "Hello World",
                "String concat should produce exact value"
            );
        }
        other => panic!("Expected InternedStr, got {:?}", other),
    }
}

#[test]
fn type_resolver_string_concat_type_test() {
    let (compiler, interner) = compile_and_resolve_single_module("let X = \"Hello\" + \" World\"");

    let name_id = interner
        .try_search_str("X")
        .expect("'X' should be interned");
    let var_def = find_user_var(&compiler, name_id);
    let val_id = match &var_def.state {
        VariableState::Known(val_id) => *val_id,
        other => panic!("'X' should be Known, got {:?}", other),
    };
    let val_info = &compiler.values[val_id];

    assert_eq!(
        val_info.type_id,
        TypeId::new(CORE_STR),
        "String concat result should have str type"
    );
}

/// Proves that non-decimal integer literals (hexadecimal, binary, octal) resolve to
/// their corresponding integer values and types.
#[test]
fn type_resolver_non_decimal_integers_parse_to_values() {
    let (compiler, interner) = compile_and_resolve_single_module(
        "
        let HEX = 0xff
        let HEX_UPPER = 0xFF
        let HEX_SEP = 0xff_ff
        let BIN = 0b1010
        let OCT = 0o77
        ",
    );

    let hex = value_of(&compiler, &interner, "HEX");
    assert!(
        matches!(hex, Value::ArbitraryInt(ArbitraryIntKind::I64(255))),
        "HEX should be I64(255), got {hex:?}"
    );

    let hex_upper = value_of(&compiler, &interner, "HEX_UPPER");
    assert!(
        matches!(hex_upper, Value::ArbitraryInt(ArbitraryIntKind::I64(255))),
        "HEX_UPPER should be I64(255), got {hex_upper:?}"
    );

    let hex_sep = value_of(&compiler, &interner, "HEX_SEP");
    assert!(
        matches!(hex_sep, Value::ArbitraryInt(ArbitraryIntKind::I64(65_535))),
        "HEX_SEP should be I64(65535), got {hex_sep:?}"
    );

    let bin = value_of(&compiler, &interner, "BIN");
    assert!(
        matches!(bin, Value::ArbitraryInt(ArbitraryIntKind::I64(10))),
        "BIN should be I64(10), got {bin:?}"
    );

    let oct = value_of(&compiler, &interner, "OCT");
    assert!(
        matches!(oct, Value::ArbitraryInt(ArbitraryIntKind::I64(63))),
        "OCT should be I64(63), got {oct:?}"
    );
}

/// Invalid radix digits must never panic the type resolver.
///
/// Proves that scripts containing radix literals with invalid digits (such as `0b102`,
/// `0b2`, `0o89`, `0o8`) complete through type resolution cleanly without panicking,
/// regardless of diagnostic emission.
#[test]
fn type_resolver_invalid_radix_digits_do_not_panic() {
    for text in [
        "let X = 0b102",
        "let X = 0b2",
        "let X = 0o89",
        "let X = 0o8",
    ] {
        // Completing the pipeline without panicking proves resilience against malformed radix inputs.
        let _ = resolve_single_module(text, Stage::Type);
    }
}

/// Proves that unparseable integer literals reaching the type resolver report
/// `NumericOverflow` as a diagnostic rather than panicking.
#[test]
fn type_resolver_unparseable_integer_yields_numeric_overflow_diagnostic() {
    let (arena, mut interner, mut cfg, mut compiler) = mock_single_module_compiler("");

    let mut ast_info = AstInfo::new();
    let name_id = interner.intern("X");
    let val_id = interner.intern("102");
    let expr = SpannedExpr::new(
        AstExpr::Integer(val_id, Notation::Bin),
        SourceSpan::default(),
    );
    let var = AbstractVar::new(name_id, SourceSpan::default(), expr, false);
    ast_info.push_item(SectionKind::Neutral, Item::Decl(AbstractDecl::Var(var)));

    let asts = vec![Some(ast_info)];
    let reg_envs = build_registration_envs(&compiler, &arena, &asts);

    let (_ns, _member, ty, _cn) = run_stages(
        Stage::Type,
        &mut cfg,
        &mut interner,
        &mut compiler,
        &reg_envs,
        &arena,
        &asts,
    );

    assert_eq!(ty.err_count(), 1);
    assert!(ty.diags[0].core_msg.contains("had an overflow"));
    assert!(ty.diags[0].core_msg.contains("102"));
}

/// Float literals overflowing `f64` keep their magnitude as `BigFloat`
/// instead of collapsing to `inf`.
///
/// Proves that float literals exceeding `f64` representation range (such as `1e1000`)
/// resolve to finite `ArbitraryFloatKind::BigFloat` values preserving their full
/// magnitude and precision.
#[test]
fn type_resolver_overflowing_float_literal_becomes_bigfloat() {
    let res = resolve_single_module_with_config(
        "let X = 1e1000",
        Stage::Type,
        config_with_numeric_limit(4096),
    )
    .expect_ok();
    match res.value_of("X") {
        Value::ArbitraryFloat(ArbitraryFloatKind::BigFloat(v)) => {
            assert!(
                !v.repr().is_infinite(),
                "overflowing literal must stay finite, got {v}"
            );
            let rendered = format!("{v}");
            assert_eq!(
                rendered.len(),
                1001,
                "expected `1` followed by 1000 zeros, got {rendered}"
            );
            assert!(
                rendered.starts_with('1'),
                "expected magnitude 1e1000, got {rendered}"
            );
        }
        other => panic!("expected BigFloat for `1e1000`, got {other:?}"),
    }
}

#[test]
fn type_resolver_literal_types_match_arbitrary_type_id() {
    let res = resolve_single_module_with_config(
        "
        let A = 42
        let B = 9223372036854775808
        let C = 18446744073709551616
        let D = 3.14
        let E = 1e1000
        ",
        Stage::Type,
        config_with_numeric_limit(4096),
    )
    .expect_ok();

    let get_type = |name: &str| -> TypeId {
        let name_id = res.interner.try_search_str(name).unwrap();
        let var_def = find_user_var(&res.compiler, name_id);
        match &var_def.state {
            VariableState::Known(value_id) => res.compiler.values[*value_id].type_id,
            _ => panic!("unexpected var state"),
        }
    };

    assert_eq!(get_type("A"), TypeId::new(CORE_I64));
    assert_eq!(get_type("B"), TypeId::new(CORE_U64));
    assert_eq!(get_type("C"), TypeId::new(CORE_BIGINT));
    assert_eq!(get_type("D"), TypeId::new(CORE_F64));
    assert_eq!(get_type("E"), TypeId::new(CORE_BIGFLOAT));
}

fn root_option_expr_and_value<'a>(
    resolution: &'a Resolution,
    option_name: &str,
) -> (&'a ResolvedExpr, &'a ValueInfo) {
    let option = resolution
        .compiler
        .impl_membs
        .iter()
        .find_map(|member| match member {
            ImplMemberKind::OptAssignmentRoot(option)
                if resolution.interner.search(option.name_id) == option_name =>
            {
                Some(option)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("root option `{option_name}` should exist"));
    let array_expr = &resolution.compiler.exprs[option.array_expr_id];
    (array_expr, &resolution.compiler.values[array_expr.val_id])
}

fn assert_i64_array(option_name: &str, value: &ValueInfo, expected: &[i64]) {
    let Some(Value::Array(values)) = &value.const_val else {
        panic!(
            "option `{option_name}` should hold a constant array, got {:?}",
            value.const_val
        );
    };
    assert_eq!(values.len(), expected.len());
    for (value, expected) in values.iter().zip(expected) {
        assert!(
            matches!(value, Value::ArbitraryInt(found) if *found == int_from_i64(*expected)),
            "expected i64 value {expected}, got {value:?}"
        );
    }
}

fn concrete_type(resolution: &Resolution, mut type_id: TypeId) -> TypeId {
    walk_type_id_deferred!(&resolution.compiler.types, type_id).inner
}

#[test]
fn standing_expr_resolves_pending_symbol_in_root_option() {
    let resolution = resolve_single_module(
        "
            let PENDING = SOURCE
            let SOURCE = 4
            nest->
                struct Settings {}
            complex->
                Settings { values = [PENDING] }
        ",
        Stage::Type,
    )
    .expect_ok();

    let (expr, value) = root_option_expr_and_value(&resolution, "values");
    assert_eq!(
        concrete_type(&resolution, expr.type_id),
        TypeId::new(CORE_I64)
    );
    assert_i64_array("values", value, &[4]);
}

#[test]
fn standing_expr_repairs_each_dependent_expression_tree() {
    let resolution = resolve_single_module(
        "
            let PENDING = SOURCE
            let SOURCE = 4
            nest->
                struct Settings {}
            complex->
                Settings {
                    direct = [PENDING]
                    computed = [1, PENDING + 2]
                }
        ",
        Stage::Type,
    )
    .expect_ok();

    let (direct_expr, direct_value) = root_option_expr_and_value(&resolution, "direct");
    assert_eq!(
        concrete_type(&resolution, direct_expr.type_id),
        TypeId::new(CORE_I64)
    );
    assert_i64_array("direct", direct_value, &[4]);

    let (computed_expr, computed_value) = root_option_expr_and_value(&resolution, "computed");
    assert_eq!(
        concrete_type(&resolution, computed_expr.type_id),
        TypeId::new(CORE_I64)
    );
    assert_i64_array("computed", computed_value, &[1, 6]);
}

#[test]
fn standing_expr_updates_resolved_value_type() {
    let resolution = resolve_single_module(
        "
            let PENDING = SOURCE
            let SOURCE = 4
            nest->
                struct Settings {}
            complex->
                Settings { values = [PENDING] }
        ",
        Stage::Type,
    )
    .expect_ok();

    let (expr, value) = root_option_expr_and_value(&resolution, "values");
    assert_eq!(
        concrete_type(&resolution, expr.type_id),
        TypeId::new(CORE_I64)
    );
    assert_eq!(
        concrete_type(&resolution, value.type_id),
        TypeId::new(CORE_I64),
        "resolving a standing expression must update its cached value type with its expression type"
    );
}
