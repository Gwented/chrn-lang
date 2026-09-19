use super::helpers::*;
use crate::script_compiler::helpers::core_helpers::core_instantiation_reservations;
use crate::semantic::arbitraries::{
    ArbitraryFloatKind, float_from_f64, int_from_i32, int_from_u32,
};

/// Values the compiler generates before any user code, one per intrinsic constant such as
/// `i8::MAX`. User values start after them.
fn generated_value_count() -> usize {
    core_instantiation_reservations().variables
}

#[test]
fn variable_declaration_test() {
    let generated = generated_value_count();

    // let CONSTANT = 4
    let text = "
            let CONSTANT = 4
            ";

    let (compiler, _) = compile_and_resolve_single_module(text);

    assert_eq!(compiler.values.len(), generated + 1);
    let last_val = &compiler.values[ValueId::new(generated as u32)];
    match &last_val.const_val {
        Some(Value::ArbitraryInt(ArbitraryIntKind::I64(4))) => (),
        _ => panic!("Value mismatch, expected I64(4)"),
    };

    // let CONSTANT = "Hallo"
    let text = "
            let CONSTANT = \"Hallo\"
        ";

    let (compiler, interner) = compile_and_resolve_single_module(text);

    assert_eq!(compiler.values.len(), generated + 1);
    let last_val = &compiler.values[ValueId::new(generated as u32)];
    match &last_val.const_val {
        Some(Value::InternedStr(id)) => {
            assert_eq!("Hallo", interner.search(*id));
        }
        _ => panic!("Value mismatch, expected InternedStr(\"Hallo\")"),
    };

    // let CONSTANT = 0e-5
    let text = "
            let CONSTANT = 0e-5
        ";

    let (compiler, _) = compile_and_resolve_single_module(text);

    assert_eq!(compiler.values.len(), generated + 1);
    let last_val = &compiler.values[ValueId::new(generated as u32)];
    match &last_val.const_val {
        Some(Value::ArbitraryFloat(v)) if *v == float_from_f64(0e-5) => (),
        _ => panic!("Value mismatch, expected F64(0e-5)"),
    };

    // let CONSTANT = true
    let text = "
            let CONSTANT = true
        ";

    let (compiler, _) = compile_and_resolve_single_module(text);

    assert_eq!(compiler.values.len(), generated + 1);
    let last_val = &compiler.values[ValueId::new(generated as u32)];
    match &last_val.const_val {
        Some(Value::Bool(true)) => (),
        _ => panic!("Value mismatch, expected Bool(true)"),
    };

    // let CONSTANT = false
    let text = "
            let CONSTANT = false
        ";

    let (compiler, _) = compile_and_resolve_single_module(text);

    assert_eq!(compiler.values.len(), generated + 1);
    let last_val = &compiler.values[ValueId::new(generated as u32)];
    match &last_val.const_val {
        Some(Value::Bool(false)) => (),
        _ => panic!("Value mismatch, expected Bool(false)"),
    };

    // let character = 'c'
    let text = "
            let character = 'c'
        ";

    let (compiler, _) = compile_and_resolve_single_module(text);

    assert_eq!(compiler.values.len(), generated + 1);
    let last_val = &compiler.values[ValueId::new(generated as u32)];
    match &last_val.const_val {
        Some(Value::Char('c')) => (),
        _ => panic!("Value mismatch, expected Char('c')"),
    };
}

#[test]
fn type_resolver_values_test() {
    let text = "
            let CONSTANT_INT = 4
            let CONSTANT_STR = \"Hallo\"
            let CONSTANT_FLOAT = 0e-5
            let CONSTANT_TRUE = true
            let CONSTANT_FALSE = false
            let CONSTANT_CHAR = 'c'
        ";

    let (compiler, interner) = compile_and_resolve_single_module(text);

    let find_val = |name: &str| -> &Value {
        let name_id = interner.try_search_str(name).unwrap();
        let var_def = find_user_var(&compiler, name_id);
        match &var_def.state {
            VariableState::Known(value_id) => compiler.values[*value_id]
                .const_val
                .as_ref()
                .expect("Variable '{name}' has no const_val"),
            VariableState::ReservedTypeSlot(_) => {
                panic!("Variable '{name}' is not yet resolved")
            }
        }
    };

    assert_eq!(compiler.values.len(), generated_value_count() + 6);
    assert!(matches!(
        find_val("CONSTANT_INT"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(4))
    ));
    assert!(
        matches!(find_val("CONSTANT_STR"), Value::InternedStr(id) if interner.search(*id) == "Hallo")
    );
    assert!(
        matches!(find_val("CONSTANT_FLOAT"), Value::ArbitraryFloat(v) if *v == float_from_f64(0e-5))
    );
    assert!(matches!(find_val("CONSTANT_TRUE"), Value::Bool(true)));
    assert!(matches!(find_val("CONSTANT_FALSE"), Value::Bool(false)));
    assert!(matches!(find_val("CONSTANT_CHAR"), Value::Char('c')));
}

#[test]
fn all_operators_test() {
    let eval = |text: &str| -> Value {
        resolve_single_module(text, Stage::Constraint)
            .expect_ok()
            .value_of("X")
    };

    // -- Unary: ! (Not) --
    assert!(matches!(eval("let X = !true"), Value::Bool(false)));
    assert!(matches!(eval("let X = !false"), Value::Bool(true)));
    // -- Unary: - (Negate) --
    assert!(matches!(
        eval("let X = -5"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(-5))
    ));
    assert!(
        matches!(eval("let X = -3.14"), Value::ArbitraryFloat(v) if v == float_from_f64(-3.14))
    );
    // -- Unary: ~ (BitNot) --
    assert!(matches!(eval("let X = ~5"), Value::ArbitraryInt(x) if x == int_from_i32(!5)));

    // -- Binary: + --
    assert!(matches!(
        eval("let X = 10 + 20"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(30))
    ));
    assert!(
        matches!(eval("let X = 1.5 + 2.5"), Value::ArbitraryFloat(v) if v == float_from_f64(4.0))
    );
    // -- Binary: - --
    assert!(matches!(
        eval("let X = 10 - 3"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(7))
    ));
    assert!(
        matches!(eval("let X = 5.5 - 1.5"), Value::ArbitraryFloat(v) if v == float_from_f64(4.0))
    );
    // -- Binary: * --
    assert!(matches!(
        eval("let X = 3 * 7"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(21))
    ));
    assert!(
        matches!(eval("let X = 2.5 * 4.0"), Value::ArbitraryFloat(v) if v == float_from_f64(10.0))
    );
    // -- Binary: / --
    assert!(matches!(
        eval("let X = 10 / 3"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(3))
    ));
    assert!(
        matches!(eval("let X = 10.0 / 4.0"), Value::ArbitraryFloat(v) if v == float_from_f64(2.5))
    );
    // -- Binary: % --
    assert!(matches!(
        eval("let X = 10 % 3"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(1))
    ));

    // -- Binary: > --
    assert!(matches!(eval("let X = 5 > 3"), Value::Bool(true)));
    assert!(matches!(eval("let X = 3 > 5"), Value::Bool(false)));
    // -- Binary: < --
    assert!(matches!(eval("let X = 5 < 3"), Value::Bool(false)));
    assert!(matches!(eval("let X = 3 < 5"), Value::Bool(true)));
    // -- Binary: >= --
    assert!(matches!(eval("let X = 5 >= 3"), Value::Bool(true)));
    assert!(matches!(eval("let X = 5 >= 5"), Value::Bool(true)));
    assert!(matches!(eval("let X = 3 >= 5"), Value::Bool(false)));
    // -- Binary: <= --
    assert!(matches!(eval("let X = 3 <= 5"), Value::Bool(true)));
    assert!(matches!(eval("let X = 5 <= 5"), Value::Bool(true)));
    assert!(matches!(eval("let X = 5 <= 3"), Value::Bool(false)));
    // -- Binary: == --
    assert!(matches!(eval("let X = 5 == 5"), Value::Bool(true)));
    assert!(matches!(eval("let X = 5 == 3"), Value::Bool(false)));
    // -- Binary: != --
    assert!(matches!(eval("let X = 5 != 3"), Value::Bool(true)));
    assert!(matches!(eval("let X = 5 != 5"), Value::Bool(false)));

    // -- Binary: && --
    assert!(matches!(eval("let X = true && true"), Value::Bool(true)));
    assert!(matches!(eval("let X = true && false"), Value::Bool(false)));
    // -- Binary: || --
    assert!(matches!(eval("let X = true || false"), Value::Bool(true)));
    assert!(matches!(eval("let X = false || false"), Value::Bool(false)));

    // -- Binary: | (BitOr) --
    assert!(matches!(
        eval("let X = 5 | 3"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(7))
    ));
    // -- Binary: & (BitAnd) --
    assert!(matches!(
        eval("let X = 5 & 3"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(1))
    ));
    // -- Binary: ^ (BitXor) --
    assert!(matches!(
        eval("let X = 5 ^ 3"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(6))
    ));
    // -- Binary: << (BitLeftShift) --
    assert!(matches!(
        eval("let X = 1 << 2"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(4))
    ));
    // -- Binary: >> (BitRightShift) --
    assert!(matches!(
        eval("let X = 8 >> 1"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(4))
    ));

    // -- String comparison (!= only) --
    assert!(matches!(
        eval("let X = \"hello\" != \"world\""),
        Value::Bool(true)
    ));
    assert!(matches!(
        eval("let X = \"hello\" != \"hello\""),
        Value::Bool(false)
    ));

    // -- Char comparison --
    assert!(matches!(eval("let X = 'b' > 'a'"), Value::Bool(true)));
    assert!(matches!(eval("let X = 'a' == 'a'"), Value::Bool(true)));
    assert!(matches!(eval("let X = 'a' != 'b'"), Value::Bool(true)));
    assert!(matches!(eval("let X = 'a' < 'b'"), Value::Bool(true)));
    assert!(matches!(eval("let X = 'a' <= 'b'"), Value::Bool(true)));
    assert!(matches!(eval("let X = 'b' >= 'a'"), Value::Bool(true)));
    assert!(matches!(eval("let X = 'a' <= 'a'"), Value::Bool(true)));
    assert!(matches!(eval("let X = 'b' >= 'b'"), Value::Bool(true)));

    // -- Bool comparison (==, !=) --
    assert!(matches!(eval("let X = true == true"), Value::Bool(true)));
    assert!(matches!(eval("let X = true == false"), Value::Bool(false)));
    assert!(matches!(eval("let X = true != false"), Value::Bool(true)));

    // -- Float comparison --
    assert!(matches!(eval("let X = 3.14 > 2.0"), Value::Bool(true)));
    assert!(matches!(eval("let X = 3.14 == 3.14"), Value::Bool(true)));
    assert!(matches!(eval("let X = 3.14 != 2.0"), Value::Bool(true)));

    // -- Float mod --
    assert!(
        matches!(eval("let X = 5.5 % 2.0"), Value::ArbitraryFloat(v) if v == float_from_f64(1.5))
    );
}

#[test]
fn arithmetic_error_test() {
    use crate::parser::ast::ast_concepts::BinaryOp;
    use crate::semantic::evaluator::{BinaryOpResult, apply_binary_op};
    use chrn_utils::{source_map::source_span::SourceSpan, utils::containers::SpannedContainerRef};

    // Arithmetic errors originate in `TypeResolver::register_expr`, so `Stage::Type`
    // already surfaces them without running the constraint stage.
    let expect_err = |text: &str, expected_msg: &str| {
        let res = resolve_single_module(text, Stage::Type);
        assert!(
            res.err_count() >= 1,
            "expected >= 1 error for input {text:?}, got none"
        );
        let mut msgs = Vec::new();
        for summary in [&res.ns, &res.member, &res.ty, &res.cn] {
            msgs.extend(summary.diags.iter().map(|d| d.core_msg.clone()));
        }
        assert!(
            msgs.iter().any(|m| m.contains(expected_msg)),
            "expected diagnostic containing {expected_msg:?} for input {text:?}, got messages: {msgs:?}"
        );
    };

    // Int mod by zero: without the `Mod` zero guard the evaluator panics on `% 0`.
    expect_err("let X = 10 % 0", "Cannot divide by zero");
    // Int div by zero: `Div` has a separate guard returning the same variant.
    expect_err("let X = 10 / 0", "Cannot divide by zero");
    // Float div by zero: same guard for the float path.
    expect_err("let X = 10.0 / 0.0", "Cannot divide by zero");
    // Negative shift: `checked_shift_amount` rejects negatives; unchecked path panics/OOMs.
    expect_err("let X = 1 << -1", "Shift amount is out of range");
    // Shift exceeding u32: `checked_shift_amount` rejects amounts that do not fit in u32.
    expect_err("let X = 1 << 4294967296", "Shift amount is out of range");
    // Shift exceeding MAX_SHIFT_BITS (1_000_000): bounds BigInt allocation, prevents OOM/hang.
    expect_err("let X = 1 << 1000001", "Shift amount is out of range");
    // Oversized right shift takes the same checked path.
    expect_err("let X = 8 >> 4294967296", "Shift amount is out of range");

    // Bitwise NOT follows two's complement sign semantics (~x = -x - 1) across all variants:
    // `!int_from_u32(0)` evaluates to `I64(-1)` rather than variant-dependent `U64(u64::MAX)`.
    assert_eq!(!int_from_u32(0), ArbitraryIntKind::I64(-1));

    // Float remainder by zero follows IEEE semantics (`NaN`), matching
    // pre-arbitrary-precision behavior where `5.5 % 0.0` evaluated to `NaN`.
    // `DBig` cannot represent NaN and panics when the result would be NaN, so
    // the evaluator routes zero-divisor float remainders through `f64`.
    let nan = resolve_single_module("let X = 5.5 % 0.0", Stage::Constraint)
        .expect_ok()
        .value_of("X");
    match nan {
        Value::ArbitraryFloat(v) if v.to_f64().is_nan() => (),
        other => panic!("expected NaN for `5.5 % 0.0`, got {other:?}"),
    }

    // Direct guard contracts on the evaluator, independent of resolution plumbing.
    {
        let mut interner = mock_interner(0, 1);
        let span = SourceSpan::default();

        let lhs = Value::ArbitraryInt(ArbitraryIntKind::I64(10));
        let rhs = Value::ArbitraryInt(ArbitraryIntKind::I64(0));
        match apply_binary_op(
            SpannedContainerRef::new(&lhs, span),
            BinaryOp::Mod,
            SpannedContainerRef::new(&rhs, span),
            &mut interner,
        ) {
            BinaryOpResult::DivideByZero => (),
            BinaryOpResult::Output(_) | BinaryOpResult::Invalid | BinaryOpResult::InvalidShift => {
                panic!("expected DivideByZero for `10 % 0`")
            }
        }

        // Float remainder by zero yields `NaN`, not an error: without the
        // `f64` fallback the `DBig` path panics on the NaN result.
        let lhs = Value::ArbitraryFloat(float_from_f64(5.5));
        let rhs = Value::ArbitraryFloat(float_from_f64(0.0));
        match apply_binary_op(
            SpannedContainerRef::new(&lhs, span),
            BinaryOp::Mod,
            SpannedContainerRef::new(&rhs, span),
            &mut interner,
        ) {
            BinaryOpResult::Output(Value::ArbitraryFloat(v)) if v.to_f64().is_nan() => (),
            _ => panic!("expected NaN output for `5.5 % 0.0`"),
        }

        // Same contract with a `BigFloat` dividend, which must not reach
        // `DBig::rem` with a zero divisor.
        let lhs = Value::ArbitraryFloat(
            ArbitraryFloatKind::from_str("1e1000").expect("overflowing literal is BigFloat"),
        );
        let rhs = Value::ArbitraryFloat(float_from_f64(0.0));
        match apply_binary_op(
            SpannedContainerRef::new(&lhs, span),
            BinaryOp::Mod,
            SpannedContainerRef::new(&rhs, span),
            &mut interner,
        ) {
            BinaryOpResult::Output(Value::ArbitraryFloat(v)) if v.to_f64().is_nan() => (),
            _ => panic!("expected NaN output for `BigFloat % 0.0`"),
        }

        let lhs = Value::ArbitraryInt(ArbitraryIntKind::I64(1));
        let rhs = Value::ArbitraryInt(ArbitraryIntKind::I64(-1));
        match apply_binary_op(
            SpannedContainerRef::new(&lhs, span),
            BinaryOp::BitLeftShift,
            SpannedContainerRef::new(&rhs, span),
            &mut interner,
        ) {
            BinaryOpResult::InvalidShift => (),
            BinaryOpResult::Output(_) | BinaryOpResult::Invalid | BinaryOpResult::DivideByZero => {
                panic!("expected InvalidShift for `1 << -1`")
            }
        }
    }
}

#[test]
fn const_dependency_resolution_test() {
    // Ok buddy
    let approx_eq = |a: f64, b: f64| (a - b).abs() < 1e-9;

    // 1) Reverse-ordered linear chain: each variable depends on the previous one, and the
    //    literal is declared last. This exercises the pending-expression propagation loop.
    let (compiler, interner) = compile_and_resolve_single_module(
        "
            let A = Q + 2
            let B = A * 3
            let C = B - 1
            let D = C / 2
            let Q = 4
        ",
    );
    assert!(matches!(
        value_of(&compiler, &interner, "A"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(6))
    ));
    assert!(matches!(
        value_of(&compiler, &interner, "B"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(18))
    ));
    assert!(matches!(
        value_of(&compiler, &interner, "C"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(17))
    ));
    assert!(matches!(
        value_of(&compiler, &interner, "D"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(8))
    ));
    assert!(matches!(
        value_of(&compiler, &interner, "Q"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(4))
    ));

    // 2) Diamond dependency: one base value feeds two branches that are later combined.
    let (compiler, interner) = compile_and_resolve_single_module(
        "
            let BASE = 2
            let LEFT = BASE * 3
            let RIGHT = BASE + 5
            let TOP = LEFT + RIGHT
        ",
    );
    assert!(matches!(
        value_of(&compiler, &interner, "TOP"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(13))
    ));

    // 3) Expression declared before its dependencies, referencing multiple pending variables.
    let (compiler, interner) = compile_and_resolve_single_module(
        "
            let Z = (X + Y) * (Y - W)
            let W = 2
            let X = W + 3
            let Y = X * W
        ",
    );
    assert!(matches!(
        value_of(&compiler, &interner, "Z"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(120))
    ));

    // 4) Long chain of pure references.
    let (compiler, interner) = compile_and_resolve_single_module(
        "
            let N1 = 7
            let N2 = N1
            let N3 = N2
            let N4 = N3
            let N5 = N4 + N3 * 2
        ",
    );
    assert!(matches!(
        value_of(&compiler, &interner, "N5"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(21))
    ));

    // 5) Boolean values derived from numeric comparisons.
    let (compiler, interner) = compile_and_resolve_single_module(
        "
            let THRESH = 5
            let VAL = 10
            let IS_BIG = VAL > THRESH
            let RESULT = IS_BIG || false
        ",
    );
    assert!(matches!(
        value_of(&compiler, &interner, "RESULT"),
        Value::Bool(true)
    ));

    // 6) Floating-point dependency chain.
    let (compiler, interner) = compile_and_resolve_single_module(
        "
            let PI_VAL = 3.14
            let R = 2.0
            let AREA = PI_VAL * R * R
        ",
    );
    match value_of(&compiler, &interner, "AREA") {
        Value::ArbitraryFloat(v) => assert!(approx_eq(v.to_f64(), 12.56), "AREA was {:?}", v),
        other => panic!("Expected F64 for AREA, got {:?}", other),
    }

    // 7) Unary operator propagation through a dependency.
    let (compiler, interner) = compile_and_resolve_single_module(
        "
            let NEG = -5
            let POS = -NEG + 1
        ",
    );
    assert!(matches!(
        value_of(&compiler, &interner, "POS"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(6))
    ));

    // 8) Mixed int/bool independent chains in the same module.
    let (compiler, interner) = compile_and_resolve_single_module(
        "
            let A = 3
            let B = 4
            let C = A > B
            let D = !C
            let Q = (A + B) * 2
            let F = Q > 10
        ",
    );
    assert!(matches!(
        value_of(&compiler, &interner, "D"),
        Value::Bool(true)
    ));
    assert!(matches!(
        value_of(&compiler, &interner, "F"),
        Value::Bool(true)
    ));
}

#[test]
fn const_dependency_circular_test() {
    let assert_any_var_unknown = |(result, compiler, interner): (
        Result<(), Vec<SourceDiagnostic>>,
        ScriptCompiler,
        Intern,
    ),
                                  names: &[&str]| {
        assert!(result.is_err(), "Circular dependency should be rejected");
        let any_unknown = names.iter().any(|name| {
            let name_id = interner
                .try_search_str(name)
                .unwrap_or_else(|| panic!("Variable '{name}' was not interned"));
            let var_def = find_user_var(&compiler, name_id);
            matches!(var_def.state, VariableState::ReservedTypeSlot(_))
        });
        assert!(
            any_unknown,
            "At least one variable in the cycle should remain unresolved (ReservedTypeSlot), but all were Known: {:?}",
            names
                .iter()
                .map(|name| {
                    let name_id = interner.try_search_str(name).unwrap();
                    let var_def = find_user_var(&compiler, name_id);
                    (name, &var_def.state)
                })
                .collect::<Vec<_>>()
        );
    };

    // Linear dependency cycle should be rejected
    assert_any_var_unknown(
        type_resolve_single_module_keep_state("let x = y\nlet y = x"),
        &["x", "y"],
    );

    // Direct self reference.
    assert_any_var_unknown(type_resolve_single_module_keep_state("let X = X"), &["X"]);

    // Three-variable cycle.
    assert_any_var_unknown(
        type_resolve_single_module_keep_state("let A = B + 1\nlet B = C * 2\nlet C = A"),
        &["A", "B", "C"],
    );

    // Long indirect cycle.
    assert_any_var_unknown(
        type_resolve_single_module_keep_state(
            "
                let A = B
                let B = C
                let C = D
                let D = Q
                let Q = A
            ",
        ),
        &["A", "B", "C", "D", "Q"],
    );

    // Cycle hidden inside a larger expression.
    assert_any_var_unknown(
        type_resolve_single_module_keep_state("let X = (Y + 2) * 3\nlet Y = X - 1"),
        &["X", "Y"],
    );

    // Multiple independent cycles in the same module.
    assert_any_var_unknown(
        type_resolve_single_module_keep_state(
            "
                let A = B
                let B = A
                let C = D + 1
                let D = C
            ",
        ),
        &["A", "B", "C", "D"],
    );

    // A chain that leads into a cycle.
    assert_any_var_unknown(
        type_resolve_single_module_keep_state(
            "
                let A = B + 1
                let B = C
                let C = B
            ",
        ),
        &["A", "B", "C"],
    );
}

#[test]
fn arbitrary_ops_edge_cases_test() {
    let eval = |text: &str| -> Value {
        resolve_single_module(text, Stage::Constraint)
            .expect_ok()
            .value_of("X")
    };

    // -- Left shift edge cases --
    // Shift by 0 returns original value without BigInt
    assert!(matches!(
        eval("let X = 42 << 0"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(42))
    ));
    // Zero shifted left returns zero
    assert!(matches!(
        eval("let X = 0 << 10"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(0))
    ));
    // Shift reaching bit 62 stays in I64
    assert!(matches!(
        eval("let X = 1 << 62"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(4611686018427387904))
    ));
    // Shift reaching bit 63 transitions to positive U64 (not negative!)
    assert!(matches!(
        eval("let X = 1 << 63"),
        Value::ArbitraryInt(ArbitraryIntKind::U64(v)) if v == 1u64 << 63
    ));
    // Shift overflowing 64 bits promotes to BigInt
    assert!(matches!(
        eval("let X = 1 << 64"),
        Value::ArbitraryInt(ArbitraryIntKind::BigInt(_))
    ));
    // Negative left shift in range stays I64
    assert!(matches!(
        eval("let X = -1 << 1"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(-2))
    ));
    assert!(matches!(
        eval("let X = -1 << 63"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(v)) if v == i64::MIN
    ));
    // Negative left shift overflowing i64 promotes to BigInt
    assert!(matches!(
        eval("let X = -2 << 63"),
        Value::ArbitraryInt(ArbitraryIntKind::BigInt(_))
    ));

    // -- Right shift edge cases --
    assert!(matches!(
        eval("let X = 42 >> 0"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(42))
    ));
    assert!(matches!(
        eval("let X = 0 >> 10"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(0))
    ));
    assert!(matches!(
        eval("let X = 16 >> 2"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(4))
    ));
    // Positive right shift >= 64 bits evaluates to 0
    assert!(matches!(
        eval("let X = 100 >> 64"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(0))
    ));
    assert!(matches!(
        eval("let X = 100 >> 1000"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(0))
    ));
    // Negative right shift in range preserves arithmetic sign
    assert!(matches!(
        eval("let X = -5 >> 1"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(-3))
    ));
    // Negative right shift >= 64 bits evaluates to -1
    assert!(matches!(
        eval("let X = -5 >> 64"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(-1))
    ));
    assert!(matches!(
        eval("let X = -1 >> 1000"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(-1))
    ));
    // Right shift on U64 narrows to I64
    assert!(matches!(
        eval("let X = (1 << 63) >> 1"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(4611686018427387904))
    ));

    // -- Negation edge cases --
    assert!(matches!(
        eval("let X = -42"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(-42))
    ));
    assert!(matches!(
        eval("let X = - -42"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(42))
    ));
    // Negating U64(1 << 63) yields I64(i64::MIN)
    assert!(matches!(
        eval("let X = -(1 << 63)"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(v)) if v == i64::MIN
    ));
    // Negating I64(i64::MIN) yields U64(1 << 63)
    assert!(matches!(
        eval("let X = -(-1 << 63)"),
        Value::ArbitraryInt(ArbitraryIntKind::U64(v)) if v == 1u64 << 63
    ));

    // -- Comparisons between variant tiers --
    assert!(matches!(eval("let X = -1 < (1 << 63)"), Value::Bool(true)));
    assert!(matches!(eval("let X = 10 < (1 << 63)"), Value::Bool(true)));
    assert!(matches!(eval("let X = (1 << 63) > 10"), Value::Bool(true)));
    assert!(matches!(
        eval("let X = (1 << 63) == (1 << 63)"),
        Value::Bool(true)
    ));
    assert!(matches!(
        eval("let X = 10 == (1 << 63)"),
        Value::Bool(false)
    ));
    assert!(matches!(
        eval("let X = (1 << 64) > (1 << 63)"),
        Value::Bool(true)
    ));
    assert!(matches!(eval("let X = 100 < (1 << 64)"), Value::Bool(true)));
    assert!(matches!(
        eval("let X = -100 < (1 << 64)"),
        Value::Bool(true)
    ));

    // -- Fast path arithmetic and bitwise ops --
    assert!(matches!(
        eval("let X = 100 + 200"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(300))
    ));
    assert!(matches!(
        eval("let X = 300 - 100"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(200))
    ));
    assert!(matches!(
        eval("let X = 20 * 15"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(300))
    ));
    assert!(matches!(
        eval("let X = 300 / 15"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(20))
    ));
    assert!(matches!(
        eval("let X = 305 % 15"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(5))
    ));
    assert!(matches!(
        eval("let X = 0b1100 & 0b1010"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(0b1000))
    ));
    assert!(matches!(
        eval("let X = 0b1100 | 0b1010"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(0b1110))
    ));
    assert!(matches!(
        eval("let X = 0b1100 ^ 0b1010"),
        Value::ArbitraryInt(ArbitraryIntKind::I64(0b0110))
    ));
    // Arithmetic overflow promotes cleanly to BigInt or U64
    assert!(matches!(
        eval("let X = 9223372036854775807 + 1"),
        Value::ArbitraryInt(ArbitraryIntKind::U64(v)) if v == 1u64 << 63
    ));
}

#[test]
fn arbitrary_float_edge_cases_test() {
    // Float underflow preserves non-zero precision as BigFloat, while exact zero stays F64.
    let underflow = ArbitraryFloatKind::from_str("1e-400").expect("underflow literal parses");
    assert!(
        matches!(underflow, ArbitraryFloatKind::BigFloat(_)),
        "expected BigFloat for underflowing literal `1e-400`, got {underflow:?}"
    );
    let zero_float = ArbitraryFloatKind::from_str("0.0").expect("zero parses");
    assert!(
        matches!(zero_float, ArbitraryFloatKind::F64(v) if v == 0.0),
        "expected F64(0.0) for `0.0`, got {zero_float:?}"
    );
    let zero_sci = ArbitraryFloatKind::from_str("0e1").expect("zero sci parses");
    assert!(
        matches!(zero_sci, ArbitraryFloatKind::F64(v) if v == 0.0),
        "expected F64(0.0) for `0e1`, got {zero_sci:?}"
    );

    // to_bigfloat on NaN returns None safely instead of panicking.
    let nan_kind = ArbitraryFloatKind::F64(f64::NAN);
    assert_eq!(nan_kind.to_bigfloat(), None);

    // Heterogeneous comparisons between F64 and out-of-range BigFloat.
    let huge = ArbitraryFloatKind::from_str("1e1000").expect("huge parses");
    let neg_huge = ArbitraryFloatKind::from_str("-1e1000").expect("neg huge parses");
    let finite_f64 = ArbitraryFloatKind::F64(5.0);

    assert_ne!(finite_f64, huge);
    assert_ne!(huge, finite_f64);
    assert!(finite_f64 < huge);
    assert!(huge > finite_f64);

    assert_ne!(finite_f64, neg_huge);
    assert_ne!(neg_huge, finite_f64);
    assert!(finite_f64 > neg_huge);
    assert!(neg_huge < finite_f64);

    assert_ne!(nan_kind, huge);
    assert_ne!(huge, nan_kind);
    assert_eq!(nan_kind.partial_cmp(&huge), None);
    assert_eq!(huge.partial_cmp(&nan_kind), None);
}
