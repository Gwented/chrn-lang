//! Compiler generated compiler-specific helpers

use chrn_utils::{id_types::InternedId, intern};
use lang::types::{boundaries::TypeBoundaryFlags, builtins::BuiltinType};

use crate::{
    constraints::ArgConstraint,
    lookup::scopes::scopes_concepts::ScopeType,
    script_compiler::{
        compiler_constants::{CORE_BOOL, CORE_UNKNOWN},
        helpers::instantiation_symbols::{
            InstantiationSymbolBase, InstantiationSymbolKind, InstantiationVariable, InstiationType,
        },
    },
    semantic::hir::hir_symbols::{BuiltinFuncKind, SymbolOrigin},
};

use super::instantiation_symbols::InstiationValue;
//TEST:

//BUG: This is all technically a large bug because we have access to u64::MAX but the compiler only
//allows i64. But keeping it like this because bugs are solved.
static NAMESPACE_I8: [InstantiationSymbolBase; 5] = [
    new_max(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(i8::MAX as i64),
    )),
    new_min(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(i8::MIN as i64),
    )),
    new_bits(8),
    new_bytes(1),
    new_radix(2),
];

static NAMESPACE_U8: [InstantiationSymbolBase; 5] = [
    new_max(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(u8::MAX as i64),
    )),
    new_min(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(u8::MIN as i64),
    )),
    new_bits(8),
    new_bytes(1),
    new_radix(2),
];

static NAMESPACE_I16: [InstantiationSymbolBase; 5] = [
    new_max(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(i16::MAX as i64),
    )),
    new_min(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(i16::MIN as i64),
    )),
    new_bits(16),
    new_bytes(2),
    new_radix(2),
];

static NAMESPACE_U16: [InstantiationSymbolBase; 5] = [
    new_max(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(u16::MAX as i64),
    )),
    new_min(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(u16::MIN as i64),
    )),
    new_bits(16),
    new_bytes(2),
    new_radix(2),
];

//NOTE: Rust has no stable `f16`, so the IEEE-754 binary16 bounds are spelled out. Both are exact
//in `f64`. The math constants below are the `f64` values rounded to binary16, spelled out as
//the exact `f64` that holds each rounded value.
static NAMESPACE_F16: [InstantiationSymbolBase; 34] = [
    new_max(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::F64),
        InstiationValue::F64(65504.0),
    )),
    new_min(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::F64),
        InstiationValue::F64(-65504.0),
    )),
    new_bits(16),
    new_bytes(2),
    new_radix(2),
    new_digits(3),
    new_mantissa_digits(11),
    new_const(
        intern::INTERNED_EPSILON,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(0.0009765625),
        ),
    ),
    new_const(
        intern::INTERNED_INFINITY,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(f64::INFINITY),
        ),
    ),
    new_const(
        intern::INTERNED_NEG_INFINITY,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(f64::NEG_INFINITY),
        ),
    ),
    new_const(
        intern::INTERNED_NAN,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(f64::NAN),
        ),
    ),
    new_const(
        intern::INTERNED_MIN_POSITIVE,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(0.00006103515625),
        ),
    ),
    new_const(
        intern::INTERNED_PI_UPPER,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(3.140625),
        ),
    ),
    new_const(
        intern::INTERNED_E_UPPER,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(2.71875),
        ),
    ),
    new_const(
        intern::INTERNED_TAU,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(6.28125),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_1_PI,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(0.318359375),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_1_SQRT_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(0.70703125),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_2_PI,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(0.63671875),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_2_SQRT_PI,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(1.1279296875),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(1.5703125),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_3,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(1.046875),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_4,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(0.78515625),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_6,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(0.5234375),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_8,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(0.392578125),
        ),
    ),
    new_const(
        intern::INTERNED_LN_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(0.693359375),
        ),
    ),
    new_const(
        intern::INTERNED_LN_10,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(2.302734375),
        ),
    ),
    new_const(
        intern::INTERNED_LOG2_10,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(3.322265625),
        ),
    ),
    new_const(
        intern::INTERNED_LOG2_E,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(1.4423828125),
        ),
    ),
    new_const(
        intern::INTERNED_LOG10_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(0.301025390625),
        ),
    ),
    new_const(
        intern::INTERNED_LOG10_E,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(0.434326171875),
        ),
    ),
    new_const(
        intern::INTERNED_SQRT_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(1.4140625),
        ),
    ),
    new_const(
        intern::INTERNED_SQRT_3,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(1.732421875),
        ),
    ),
    new_const(
        intern::INTERNED_GOLDEN_RATIO,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(1.6181640625),
        ),
    ),
    new_const(
        intern::INTERNED_EULER_GAMMA,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(0.5771484375),
        ),
    ),
];

static NAMESPACE_I32: [InstantiationSymbolBase; 5] = [
    new_max(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(i32::MAX as i64),
    )),
    new_min(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(i32::MIN as i64),
    )),
    new_bits(32),
    new_bytes(4),
    new_radix(2),
];

static NAMESPACE_U32: [InstantiationSymbolBase; 5] = [
    new_max(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(u32::MAX as i64),
    )),
    new_min(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(u32::MIN as i64),
    )),
    new_bits(32),
    new_bytes(4),
    new_radix(2),
];

static NAMESPACE_F32: [InstantiationSymbolBase; 34] = [
    new_max(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::F64),
        InstiationValue::F64(f32::MAX as f64),
    )),
    new_min(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::F64),
        InstiationValue::F64(f32::MIN as f64),
    )),
    new_bits(32),
    new_bytes(4),
    new_radix(2),
    new_digits(f32::DIGITS as i64),
    new_mantissa_digits(f32::MANTISSA_DIGITS as i64),
    new_const(
        intern::INTERNED_EPSILON,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(f32::EPSILON as f64),
        ),
    ),
    new_const(
        intern::INTERNED_INFINITY,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(f32::INFINITY as f64),
        ),
    ),
    new_const(
        intern::INTERNED_NEG_INFINITY,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(f32::NEG_INFINITY as f64),
        ),
    ),
    new_const(
        intern::INTERNED_NAN,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(f32::NAN as f64),
        ),
    ),
    new_const(
        intern::INTERNED_MIN_POSITIVE,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(f32::MIN_POSITIVE as f64),
        ),
    ),
    new_const(
        intern::INTERNED_PI_UPPER,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::PI as f64),
        ),
    ),
    new_const(
        intern::INTERNED_E_UPPER,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::E as f64),
        ),
    ),
    new_const(
        intern::INTERNED_TAU,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::TAU as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_1_PI,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::FRAC_1_PI as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_1_SQRT_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::FRAC_1_SQRT_2 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_2_PI,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::FRAC_2_PI as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_2_SQRT_PI,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::FRAC_2_SQRT_PI as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::FRAC_PI_2 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_3,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::FRAC_PI_3 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_4,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::FRAC_PI_4 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_6,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::FRAC_PI_6 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_8,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::FRAC_PI_8 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_LN_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::LN_2 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_LN_10,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::LN_10 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_LOG2_10,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::LOG2_10 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_LOG2_E,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::LOG2_E as f64),
        ),
    ),
    new_const(
        intern::INTERNED_LOG10_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::LOG10_2 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_LOG10_E,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::LOG10_E as f64),
        ),
    ),
    new_const(
        intern::INTERNED_SQRT_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::SQRT_2 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_SQRT_3,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(1.732050807568877293527446341505872366_f32 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_GOLDEN_RATIO,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::GOLDEN_RATIO as f64),
        ),
    ),
    new_const(
        intern::INTERNED_EULER_GAMMA,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f32::consts::EULER_GAMMA as f64),
        ),
    ),
];

static NAMESPACE_I64: [InstantiationSymbolBase; 5] = [
    new_max(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(i64::MAX),
    )),
    new_min(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::I64),
        InstiationValue::I64(i64::MIN),
    )),
    new_bits(64),
    new_bytes(8),
    new_radix(2),
];

static NAMESPACE_U64: [InstantiationSymbolBase; 3] = [new_bits(64), new_bytes(8), new_radix(2)];

static NAMESPACE_F64: [InstantiationSymbolBase; 34] = [
    new_max(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::F64),
        InstiationValue::F64(f64::MAX),
    )),
    new_min(InstantiationVariable::new(
        InstiationType::BuiltinType(BuiltinType::F64),
        InstiationValue::F64(f64::MIN),
    )),
    new_bits(64),
    new_bytes(8),
    new_radix(2),
    new_digits(f64::DIGITS as i64),
    new_mantissa_digits(f64::MANTISSA_DIGITS as i64),
    new_const(
        intern::INTERNED_EPSILON,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(f64::EPSILON),
        ),
    ),
    new_const(
        intern::INTERNED_INFINITY,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(f64::INFINITY),
        ),
    ),
    new_const(
        intern::INTERNED_NEG_INFINITY,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(f64::NEG_INFINITY),
        ),
    ),
    new_const(
        intern::INTERNED_NAN,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(f64::NAN),
        ),
    ),
    new_const(
        intern::INTERNED_MIN_POSITIVE,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(f64::MIN_POSITIVE),
        ),
    ),
    new_const(
        intern::INTERNED_PI_UPPER,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::PI),
        ),
    ),
    new_const(
        intern::INTERNED_E_UPPER,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::E),
        ),
    ),
    new_const(
        intern::INTERNED_TAU,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::TAU),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_1_PI,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::FRAC_1_PI),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_1_SQRT_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::FRAC_1_SQRT_2),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_2_PI,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::FRAC_2_PI),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_2_SQRT_PI,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::FRAC_2_SQRT_PI),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::FRAC_PI_2),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_3,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::FRAC_PI_3),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_4,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::FRAC_PI_4),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_6,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::FRAC_PI_6),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_8,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::FRAC_PI_8),
        ),
    ),
    new_const(
        intern::INTERNED_LN_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::LN_2),
        ),
    ),
    new_const(
        intern::INTERNED_LN_10,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::LN_10),
        ),
    ),
    new_const(
        intern::INTERNED_LOG2_10,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::LOG2_10),
        ),
    ),
    new_const(
        intern::INTERNED_LOG2_E,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::LOG2_E),
        ),
    ),
    new_const(
        intern::INTERNED_LOG10_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::LOG10_2),
        ),
    ),
    new_const(
        intern::INTERNED_LOG10_E,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::LOG10_E),
        ),
    ),
    new_const(
        intern::INTERNED_SQRT_2,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::SQRT_2),
        ),
    ),
    new_const(
        intern::INTERNED_SQRT_3,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(1.732050807568877293527446341505872366_f64),
        ),
    ),
    new_const(
        intern::INTERNED_GOLDEN_RATIO,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::GOLDEN_RATIO),
        ),
    ),
    new_const(
        intern::INTERNED_EULER_GAMMA,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::F64),
            InstiationValue::F64(std::f64::consts::EULER_GAMMA),
        ),
    ),
];

//TODO: `u64::MAX`, `i128`, `u128` and `f128` bounds do not fit `InstiationValue`, which only carries
//`I64` and `F64`. `sized`/`unsized` are pointer-sized, so their bounds belong to the target rather
//than the host. The 128-bit types stay empty until the value system covers them.

// Will just be [[]] like the ns entries
/// Every core builtin type, paired with its interned name and the `TypeId` it must have.
/// `load_core_types` loads them sequentially.
pub static CORE_BUILTIN_TYPES_DATASET: [(u32, BuiltinType, &'static [InstantiationSymbolBase]);
    CORE_UNKNOWN as usize] = [
    (intern::INTERNED_I8, BuiltinType::I8, &NAMESPACE_I8),
    (intern::INTERNED_U8, BuiltinType::U8, &NAMESPACE_U8),
    (intern::INTERNED_I16, BuiltinType::I16, &NAMESPACE_I16),
    (intern::INTERNED_U16, BuiltinType::U16, &NAMESPACE_U16),
    (intern::INTERNED_F16, BuiltinType::F16, &NAMESPACE_F16),
    (intern::INTERNED_I32, BuiltinType::I32, &NAMESPACE_I32),
    (intern::INTERNED_U32, BuiltinType::U32, &NAMESPACE_U32),
    (intern::INTERNED_F32, BuiltinType::F32, &NAMESPACE_F32),
    (intern::INTERNED_I64, BuiltinType::I64, &NAMESPACE_I64),
    // -- Eveneutally faojjkaj --
    (intern::INTERNED_U64, BuiltinType::U64, &NAMESPACE_U64),
    // -- Eveneutally faojjkaj --
    (intern::INTERNED_F64, BuiltinType::F64, &NAMESPACE_F64),
    // -- Eveneutally faojjkaj --
    (intern::INTERNED_I128, BuiltinType::I128, &[]),
    (intern::INTERNED_U128, BuiltinType::U128, &[]),
    (intern::INTERNED_F128, BuiltinType::F128, &[]),
    (intern::INTERNED_SIZED, BuiltinType::Sized, &[]),
    (intern::INTERNED_UNSIZED, BuiltinType::Unsized, &[]),
    // -- Eveneutally faojjkaj --
    // Unrelated but this should have .len() later
    (intern::INTERNED_STR, BuiltinType::Str, &[]),
    (intern::INTERNED_CHAR, BuiltinType::Char, &[]),
    (intern::INTERNED_NIL, BuiltinType::Nil, &[]),
    (intern::INTERNED_BOOL, BuiltinType::Bool, &[]),
    (intern::INTERNED_BIGINT, BuiltinType::BigInt, &[]),
    (intern::INTERNED_BIGFLOAT, BuiltinType::BigFloat, &[]),
    (intern::INTERNED_RUNTIME, BuiltinType::Runtime, &[]),
];

/// Every core boundary type, paired with its interned name. Loaded after `CORE_BUILTIN_TYPES` and
/// the unknown type, so these have no `CORE_*` constants.
pub static CORE_BOUNDARIES_DATASET: [(u32, TypeBoundaryFlags); 11] = [
    (intern::INTERNED_RANGED, TypeBoundaryFlags::RANGED),
    (
        intern::INTERNED_CHARACTER_MAPPABLE,
        TypeBoundaryFlags::CHARACTER_MAPPABLE,
    ),
    (intern::INTERNED_COLLECTION, TypeBoundaryFlags::COLLECTION),
    (intern::INTERNED_HAS_LEN, TypeBoundaryFlags::HAS_LEN),
    (intern::INTERNED_INTEGER, TypeBoundaryFlags::INTEGER),
    (intern::INTERNED_NUMERIC, TypeBoundaryFlags::NUMERIC),
    (
        intern::INTERNED_SIGNED_INTEGER,
        TypeBoundaryFlags::SIGNED_INTEGER,
    ),
    (
        intern::INTERNED_UNSIGNED_INTEGER,
        TypeBoundaryFlags::UNSIGNED_INTEGER,
    ),
    (intern::INTERNED_FLOAT, TypeBoundaryFlags::FLOAT),
    (intern::INTERNED_ORDERED, TypeBoundaryFlags::ORDERED),
    (intern::INTERNED_COMPARABLE, TypeBoundaryFlags::COMPARABLE),
];

/// What a set of instantiation bases contributes to the compiler's arenas when registered.
#[derive(Default)]
pub struct InstantiationReservations {
    pub symbols: usize,
    pub scopes: usize,
    /// One `VarDef`, one `ResolvedExpr`, and one `ValueInfo` each.
    pub variables: usize,
}

impl InstantiationReservations {
    fn merge(&mut self, other: InstantiationReservations) {
        self.symbols += other.symbols;
        self.scopes += other.scopes;
        self.variables += other.variables;
    }
}

/// Counts what `register_instantiation_bases` pushes for `bases`. Every base is one symbol, a
/// namespace additionally owns a scope and whatever it holds.
pub fn count_instantiation_bases(bases: &[InstantiationSymbolBase]) -> InstantiationReservations {
    let mut counts = InstantiationReservations::default();

    for base in bases {
        counts.symbols += 1;
        match &base.kind {
            InstantiationSymbolKind::Namespace(inner) => {
                counts.scopes += 1;
                counts.merge(count_instantiation_bases(inner));
            }
            InstantiationSymbolKind::Variable(_) => counts.variables += 1,
            InstantiationSymbolKind::ExternType(_) => (),
        }
    }

    counts
}

/// Counts what the intrinsic namespaces of `CORE_BUILTIN_TYPES_DATASET` push, such as the `i8::MAX`
/// symbol and the scope holding it. Every non-empty namespace owns a scope on its built-in.
pub fn core_instantiation_reservations() -> InstantiationReservations {
    let mut counts = InstantiationReservations::default();

    for (_, _, ns) in &CORE_BUILTIN_TYPES_DATASET {
        if ns.is_empty() {
            continue;
        }

        counts.scopes += 1;
        counts.merge(count_instantiation_bases(ns));
    }

    counts
}

/// A single core function or predicate. Mirrors `FuncDef`, minus the ids the compiler assigns
/// while loading.
pub struct CoreFunc {
    /// Interned index of the function's name
    pub name: u32,
    pub kind: BuiltinFuncKind,
    pub type_constraints: TypeBoundaryFlags,
    pub arg_constraints: &'static [ArgConstraint],
    pub affects_type_constraint: bool,
    /// `TypeId` of the return type, which must already be loaded as a core type
    pub ret_type: u32,
}

impl CoreFunc {
    const fn new(
        name: u32,
        kind: BuiltinFuncKind,
        type_constraints: TypeBoundaryFlags,
        arg_constraints: &'static [ArgConstraint],
        affects_type_constraint: bool,
        ret_type: u32,
    ) -> CoreFunc {
        CoreFunc {
            name,
            kind,
            type_constraints,
            arg_constraints,
            affects_type_constraint,
            ret_type,
        }
    }
}

/// Every core function and predicate. `load_core_funcs` loads them sequentially, after
/// `CORE_BUILTIN_TYPES_DATASET`, the unknown type, and `CORE_BOUNDARIES_DATASET`.
pub static CORE_FUNCS_DATASET: [CoreFunc; 7] = [
    CoreFunc::new(
        intern::INTERNED_IS_EMPTY,
        BuiltinFuncKind::IsEmpty,
        TypeBoundaryFlags::COLLECTION,
        &[ArgConstraint::ArgCount(0)],
        true,
        CORE_BOOL,
    ),
    CoreFunc::new(
        intern::INTERNED_IS_WHITESPACE,
        BuiltinFuncKind::IsWhitespace,
        TypeBoundaryFlags::CHARACTER_MAPPABLE,
        &[ArgConstraint::ArgCount(0), ArgConstraint::CharacterMappable],
        true,
        CORE_BOOL,
    ),
    CoreFunc::new(
        intern::INTERNED_CONTAINS,
        BuiltinFuncKind::Contains,
        TypeBoundaryFlags::CHARACTER_MAPPABLE,
        &[ArgConstraint::ArgCount(1), ArgConstraint::CharacterMappable],
        true,
        CORE_BOOL,
    ),
    CoreFunc::new(
        intern::INTERNED_STARTSW,
        BuiltinFuncKind::StartsW,
        TypeBoundaryFlags::CHARACTER_MAPPABLE,
        &[ArgConstraint::ArgCount(1), ArgConstraint::CharacterMappable],
        true,
        CORE_BOOL,
    ),
    CoreFunc::new(
        intern::INTERNED_ENDSW,
        BuiltinFuncKind::EndsW,
        TypeBoundaryFlags::CHARACTER_MAPPABLE,
        &[ArgConstraint::ArgCount(1), ArgConstraint::CharacterMappable],
        true,
        CORE_BOOL,
    ),
    CoreFunc::new(
        intern::INTERNED_RANGE,
        BuiltinFuncKind::Range,
        TypeBoundaryFlags::RANGED,
        &[
            ArgConstraint::ArgCount(2),
            ArgConstraint::Numeric,
            ArgConstraint::MatchingArgumentTypes,
            ArgConstraint::SameTypeAsSelf,
        ],
        true,
        CORE_BOOL,
    ),
    CoreFunc::new(
        intern::INTERNED_EQUALS,
        BuiltinFuncKind::Equals,
        TypeBoundaryFlags::COMPARABLE,
        &[
            ArgConstraint::ArgCount(1),
            ArgConstraint::Comparable,
            ArgConstraint::SameTypeAsSelf,
        ],
        true,
        CORE_BOOL,
    ),
];

const CORE_SYM_ORIGIN: SymbolOrigin = SymbolOrigin::Compiler;
const CORE_SCOPE_ORIGIN: ScopeType = ScopeType::Core;
const CORE_IS_PRIV: bool = false;

const fn new_const(id: u32, var: InstantiationVariable) -> InstantiationSymbolBase {
    InstantiationSymbolBase::new(
        InternedId::new(id),
        // More like core_sym_origin
        CORE_SYM_ORIGIN,
        CORE_SCOPE_ORIGIN,
        CORE_IS_PRIV,
        InstantiationSymbolKind::Variable(var),
    )
}
const fn new_max(var: InstantiationVariable) -> InstantiationSymbolBase {
    new_const(intern::INTERNED_MAX_UPPER, var)
}
const fn new_min(var: InstantiationVariable) -> InstantiationSymbolBase {
    new_const(intern::INTERNED_MIN_UPPER, var)
}
const fn new_bits(bits: i64) -> InstantiationSymbolBase {
    new_const(
        intern::INTERNED_BITS_UPPER,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::I64),
            InstiationValue::I64(bits),
        ),
    )
}
const fn new_bytes(bytes: i64) -> InstantiationSymbolBase {
    new_const(
        intern::INTERNED_BYTES_UPPER,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::I64),
            InstiationValue::I64(bytes),
        ),
    )
}
const fn new_radix(radix: i64) -> InstantiationSymbolBase {
    new_const(
        intern::INTERNED_RADIX,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::I64),
            InstiationValue::I64(radix),
        ),
    )
}
const fn new_digits(digits: i64) -> InstantiationSymbolBase {
    new_const(
        intern::INTERNED_DIGITS,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::I64),
            InstiationValue::I64(digits),
        ),
    )
}
const fn new_mantissa_digits(mantissa_digits: i64) -> InstantiationSymbolBase {
    new_const(
        intern::INTERNED_MANTISSA_DIGITS,
        InstantiationVariable::new(
            InstiationType::BuiltinType(BuiltinType::I64),
            InstiationValue::I64(mantissa_digits),
        ),
    )
}
