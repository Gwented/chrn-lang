use std::ops::Range;

use chrn_utils::{
    id_types::InternedId,
    intern::{self, UTF8Tag},
};
use dashu_int::IBig;
use lang::types::{boundaries::TypeBoundaryFlags, builtins::BuiltinType};

use crate::{
    constraints::FuncArgConstraint,
    lookup::scopes::scopes_concepts::ScopeType,
    script_compiler::{
        compiler_consts::{CORE_BOOL, CORE_UNKNOWN},
        helpers::instantiation_symbols::{
            InstantiationNamespace, InstantiationSymbolBase, InstantiationSymbolKind,
            InstantiationSymbolOrigin, InstantiationType, InstantiationTypeKind,
            InstantiationValueKind, InstantiationVariable,
        },
    },
    semantic::hir::hir_symbols::BuiltinFuncKind,
};

//TODO: Split these
//
pub(super) const CORE_SYM_ORIGIN: InstantiationSymbolOrigin = InstantiationSymbolOrigin::Core;
pub(super) const CORE_SCOPE_TYPE: ScopeType = ScopeType::Core;
pub(super) const CORE_IS_PRIV: bool = false;

/// Domain-specific constructor helper for `InstantiationSymbolBase`
const fn new_core_namespace(
    name_id: u32,
    syms: &'static [InstantiationSymbolBase],
) -> InstantiationSymbolBase {
    InstantiationSymbolBase::new(
        InternedId::new(name_id).into_tagged::<UTF8Tag>(),
        CORE_SYM_ORIGIN,
        CORE_SCOPE_TYPE,
        CORE_IS_PRIV,
        InstantiationSymbolKind::Namespace(InstantiationNamespace::new(syms)),
    )
}

/// Domain-specific constructor helper for `InstantiationSymbolBase`
const fn new_core_type(name_id: u32, ty: InstantiationType) -> InstantiationSymbolBase {
    InstantiationSymbolBase::new(
        InternedId::new(name_id).into_tagged::<UTF8Tag>(),
        CORE_SYM_ORIGIN,
        CORE_SCOPE_TYPE,
        CORE_IS_PRIV,
        InstantiationSymbolKind::Type(ty),
    )
}

// NOTE: May not use this
// core::*
pub static CORE_NAMESPACE_ENTRY: [InstantiationSymbolBase; 1] =
    [new_core_namespace(intern::INTERNED_CORE, &CORE_NAMESPACE)];

// core::types/funcs/..
pub static CORE_NAMESPACE: [InstantiationSymbolBase; 2] = [
    new_core_namespace(intern::INTERNED_TYPES_LOWER, &TYPES_NAMESPACE),
    new_core_namespace(intern::INTERNED_FUNCS, &FUNCS_NAMESPACE),
];

// Concrete/Boundary?
/// Built-in type declaration range
pub const BUILTIN_TYPES_RANGE: Range<usize> = 0..CORE_UNKNOWN as usize;
/// Boundary declaration range
pub const BOUNDARIES_RANGE: Range<usize> = CORE_UNKNOWN as usize..CORE_UNKNOWN as usize + 11;

/// Core types and boundaries, preserving each group's compatibility-sensitive order.
pub static TYPES_NAMESPACE: [InstantiationSymbolBase; BOUNDARIES_RANGE.end] = [
    new_core_type(
        intern::INTERNED_I8,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::I8),
            Some(InstantiationNamespace::new(&NAMESPACE_I8)),
        ),
    ),
    new_core_type(
        intern::INTERNED_U8,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::U8),
            Some(InstantiationNamespace::new(&NAMESPACE_U8)),
        ),
    ),
    new_core_type(
        intern::INTERNED_I16,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::I16),
            Some(InstantiationNamespace::new(&NAMESPACE_I16)),
        ),
    ),
    new_core_type(
        intern::INTERNED_U16,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::U16),
            Some(InstantiationNamespace::new(&NAMESPACE_U16)),
        ),
    ),
    new_core_type(
        intern::INTERNED_F16,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F16),
            Some(InstantiationNamespace::new(&NAMESPACE_F16)),
        ),
    ),
    new_core_type(
        intern::INTERNED_I32,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::I32),
            Some(InstantiationNamespace::new(&NAMESPACE_I32)),
        ),
    ),
    new_core_type(
        intern::INTERNED_U32,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::U32),
            Some(InstantiationNamespace::new(&NAMESPACE_U32)),
        ),
    ),
    new_core_type(
        intern::INTERNED_F32,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F32),
            Some(InstantiationNamespace::new(&NAMESPACE_F32)),
        ),
    ),
    new_core_type(
        intern::INTERNED_I64,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::I64),
            Some(InstantiationNamespace::new(&NAMESPACE_I64)),
        ),
    ),
    // -- Eveneutally faojjkaj --
    new_core_type(
        intern::INTERNED_U64,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::U64),
            Some(InstantiationNamespace::new(&NAMESPACE_U64)),
        ),
    ),
    // -- Eveneutally faojjkaj --
    new_core_type(
        intern::INTERNED_F64,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            Some(InstantiationNamespace::new(&NAMESPACE_F64)),
        ),
    ),
    // -- Eveneutally faojjkaj --
    new_core_type(
        intern::INTERNED_I128,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::I128),
            Some(InstantiationNamespace::new(&NAMESPACE_I128)),
        ),
    ),
    new_core_type(
        intern::INTERNED_U128,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::U128),
            Some(InstantiationNamespace::new(&NAMESPACE_U128)),
        ),
    ),
    new_core_type(
        intern::INTERNED_F128,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F128),
            Some(InstantiationNamespace::new(&NAMESPACE_F128)),
        ),
    ),
    new_core_type(
        intern::INTERNED_SIZED,
        InstantiationType::new(InstantiationTypeKind::BuiltinType(BuiltinType::Sized), None),
    ),
    new_core_type(
        intern::INTERNED_UNSIZED,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::Unsized),
            None,
        ),
    ),
    // -- Eveneutally faojjkaj --
    // Unrelated but this should have .len() later
    new_core_type(
        intern::INTERNED_STR,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::Str),
            Some(InstantiationNamespace::new(&NAMESPACE_STR)),
        ),
    ),
    new_core_type(
        intern::INTERNED_CHAR,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::Char),
            Some(InstantiationNamespace::new(&NAMESPACE_CHAR)),
        ),
    ),
    new_core_type(
        intern::INTERNED_NIL,
        InstantiationType::new(InstantiationTypeKind::BuiltinType(BuiltinType::Nil), None),
    ),
    new_core_type(
        intern::INTERNED_BOOL,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::Bool),
            Some(InstantiationNamespace::new(&NAMESPACE_BOOL)),
        ),
    ),
    new_core_type(
        intern::INTERNED_BIGINT,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::BigInt),
            None,
        ),
    ),
    new_core_type(
        intern::INTERNED_BIGFLOAT,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::BigFloat),
            None,
        ),
    ),
    //TODO: How are we adding these
    // Will be enums for now
    // (intern::INTERNED_LIST, BuiltinType::List, &[]),
    // (intern::INTERNED_SET, BuiltinType::Set, &[]),
    // (intern::INTERNED_MAP, BuiltinType::Map, &[]),
    // (intern::INTERNED_TUPLE, BuiltinType::Tuple, &[]),
    new_core_type(
        intern::INTERNED_RUNTIME,
        InstantiationType::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::Runtime),
            None,
        ),
    ),
    new_core_type(
        intern::INTERNED_RANGED,
        InstantiationType::new(
            InstantiationTypeKind::Boundary(TypeBoundaryFlags::RANGED),
            None,
        ),
    ),
    new_core_type(
        intern::INTERNED_CHARACTER_MAPPABLE,
        InstantiationType::new(
            InstantiationTypeKind::Boundary(TypeBoundaryFlags::CHARACTER_MAPPABLE),
            None,
        ),
    ),
    new_core_type(
        intern::INTERNED_COLLECTION,
        InstantiationType::new(
            InstantiationTypeKind::Boundary(TypeBoundaryFlags::COLLECTION),
            None,
        ),
    ),
    new_core_type(
        intern::INTERNED_HAS_LEN,
        InstantiationType::new(
            InstantiationTypeKind::Boundary(TypeBoundaryFlags::HAS_LEN),
            None,
        ),
    ),
    new_core_type(
        intern::INTERNED_INTEGER,
        InstantiationType::new(
            InstantiationTypeKind::Boundary(TypeBoundaryFlags::INTEGER),
            None,
        ),
    ),
    new_core_type(
        intern::INTERNED_NUMERIC,
        InstantiationType::new(
            InstantiationTypeKind::Boundary(TypeBoundaryFlags::NUMERIC),
            None,
        ),
    ),
    new_core_type(
        intern::INTERNED_SIGNED_INTEGER,
        InstantiationType::new(
            InstantiationTypeKind::Boundary(TypeBoundaryFlags::SIGNED_INTEGER),
            None,
        ),
    ),
    new_core_type(
        intern::INTERNED_UNSIGNED_INTEGER,
        InstantiationType::new(
            InstantiationTypeKind::Boundary(TypeBoundaryFlags::UNSIGNED_INTEGER),
            None,
        ),
    ),
    new_core_type(
        intern::INTERNED_FLOAT,
        InstantiationType::new(
            InstantiationTypeKind::Boundary(TypeBoundaryFlags::FLOAT),
            None,
        ),
    ),
    new_core_type(
        intern::INTERNED_ORDERED,
        InstantiationType::new(
            InstantiationTypeKind::Boundary(TypeBoundaryFlags::ORDERED),
            None,
        ),
    ),
    new_core_type(
        intern::INTERNED_COMPARABLE,
        InstantiationType::new(
            InstantiationTypeKind::Boundary(TypeBoundaryFlags::COMPARABLE),
            None,
        ),
    ),
];

pub static FUNCS_NAMESPACE: [InstantiationSymbolBase; 0] = [];

//NOTE: BuiltinTypes assigned must align with possible arbitrary values, rather than the actual
//value present. There is also no consistency in regards to something being i64 or u64 by default,
//maybe will deal with that when it poses an issue.
//BUGS ARE SOLVED

static NAMESPACE_I8: [InstantiationSymbolBase; 4] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::I64),
        InstantiationValueKind::I64(i8::MAX as i64),
    )),
    new_min(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::I64),
        InstantiationValueKind::I64(i8::MIN as i64),
    )),
    new_bits(8),
    new_bytes(1),
];

static NAMESPACE_U8: [InstantiationSymbolBase; 4] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::I64),
        InstantiationValueKind::I64(u8::MAX as i64),
    )),
    new_min_unsigned(),
    new_bits(8),
    new_bytes(1),
];

static NAMESPACE_I16: [InstantiationSymbolBase; 4] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::I64),
        InstantiationValueKind::I64(i16::MAX as i64),
    )),
    new_min(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::I64),
        InstantiationValueKind::I64(i16::MIN as i64),
    )),
    new_bits(16),
    new_bytes(2),
];

static NAMESPACE_U16: [InstantiationSymbolBase; 4] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::I64),
        InstantiationValueKind::I64(u16::MAX as i64),
    )),
    new_min_unsigned(),
    new_bits(16),
    new_bytes(2),
];

//NOTE: Rust has no stable `f16`, so the IEEE-754 binary16 bounds are spelled out. The math constants below are the `f64` values rounded to binary16.
static NAMESPACE_F16: [InstantiationSymbolBase; 42] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::F64),
        InstantiationValueKind::F64(65504.0),
    )),
    new_min(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::F64),
        InstantiationValueKind::F64(-65504.0),
    )),
    new_bits(16),
    new_bytes(2),
    new_radix(2),
    new_digits(3),
    new_mantissa_digits(11),
    new_const(
        intern::INTERNED_EPSILON,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(0.0009765625),
        ),
    ),
    new_const(
        intern::INTERNED_INF,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(f64::INFINITY),
        ),
    ),
    new_const(
        intern::INTERNED_NEG_INF,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(f64::NEG_INFINITY),
        ),
    ),
    new_const(
        intern::INTERNED_NAN,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(f64::NAN),
        ),
    ),
    new_const(
        intern::INTERNED_MIN_POSITIVE,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(0.00006103515625),
        ),
    ),
    new_const(
        intern::INTERNED_PI_UPPER,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(3.140625),
        ),
    ),
    new_const(
        intern::INTERNED_E_UPPER,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(2.71875),
        ),
    ),
    new_const(
        intern::INTERNED_TAU,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(6.28125),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_1_PI,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(0.318359375),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_1_SQRT_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(0.70703125),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_2_PI,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(0.63671875),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_2_SQRT_PI,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(1.1279296875),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(1.5703125),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_3,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(1.046875),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_4,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(0.78515625),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_6,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(0.5234375),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_8,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(0.392578125),
        ),
    ),
    new_const(
        intern::INTERNED_LN_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(0.693359375),
        ),
    ),
    new_const(
        intern::INTERNED_LN_10,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(2.302734375),
        ),
    ),
    new_const(
        intern::INTERNED_LOG2_10,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(3.322265625),
        ),
    ),
    new_const(
        intern::INTERNED_LOG2_E,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(1.4423828125),
        ),
    ),
    new_const(
        intern::INTERNED_LOG10_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(0.301025390625),
        ),
    ),
    new_const(
        intern::INTERNED_LOG10_E,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(0.434326171875),
        ),
    ),
    new_const(
        intern::INTERNED_SQRT_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(1.4140625),
        ),
    ),
    new_const(
        intern::INTERNED_SQRT_3,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(1.732421875),
        ),
    ),
    new_const(
        intern::INTERNED_GOLDEN_RATIO,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(1.6181640625),
        ),
    ),
    new_const(
        intern::INTERNED_EULER_GAMMA,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(0.5771484375),
        ),
    ),
    new_i64_metadata(intern::INTERNED_SIGN_BITS, 1),
    new_i64_metadata(intern::INTERNED_EXPONENT_BITS, 5),
    new_i64_metadata(intern::INTERNED_SIGNIFICAND_BITS, 11),
    new_i64_metadata(intern::INTERNED_STORED_SIGNIFICAND_BITS, 10),
    new_i64_metadata(intern::INTERNED_EXPONENT_BIAS, 15),
    new_i64_metadata(intern::INTERNED_MIN_NORMAL_EXPONENT, -14),
    new_i64_metadata(intern::INTERNED_MAX_NORMAL_EXPONENT, 15),
    new_i64_metadata(intern::INTERNED_MIN_SUBNORMAL_EXPONENT, -24),
];

static NAMESPACE_I32: [InstantiationSymbolBase; 4] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::I64),
        InstantiationValueKind::I64(i32::MAX as i64),
    )),
    new_min(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::I64),
        InstantiationValueKind::I64(i32::MIN as i64),
    )),
    new_bits(32),
    new_bytes(4),
];

static NAMESPACE_U32: [InstantiationSymbolBase; 4] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::I64),
        InstantiationValueKind::I64(u32::MAX as i64),
    )),
    new_min_unsigned(),
    new_bits(32),
    new_bytes(4),
];

static NAMESPACE_F32: [InstantiationSymbolBase; 42] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::F64),
        InstantiationValueKind::F64(f32::MAX as f64),
    )),
    new_min(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::F64),
        InstantiationValueKind::F64(f32::MIN as f64),
    )),
    new_bits(32),
    new_bytes(4),
    new_radix(2),
    new_digits(f32::DIGITS as i64),
    new_mantissa_digits(f32::MANTISSA_DIGITS as i64),
    new_const(
        intern::INTERNED_EPSILON,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(f32::EPSILON as f64),
        ),
    ),
    new_const(
        intern::INTERNED_INF,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(f32::INFINITY as f64),
        ),
    ),
    new_const(
        intern::INTERNED_NEG_INF,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(f32::NEG_INFINITY as f64),
        ),
    ),
    new_const(
        intern::INTERNED_NAN,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(f32::NAN as f64),
        ),
    ),
    new_const(
        intern::INTERNED_MIN_POSITIVE,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(f32::MIN_POSITIVE as f64),
        ),
    ),
    new_const(
        intern::INTERNED_PI_UPPER,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::PI as f64),
        ),
    ),
    new_const(
        intern::INTERNED_E_UPPER,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::E as f64),
        ),
    ),
    new_const(
        intern::INTERNED_TAU,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::TAU as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_1_PI,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::FRAC_1_PI as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_1_SQRT_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::FRAC_1_SQRT_2 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_2_PI,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::FRAC_2_PI as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_2_SQRT_PI,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::FRAC_2_SQRT_PI as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::FRAC_PI_2 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_3,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::FRAC_PI_3 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_4,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::FRAC_PI_4 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_6,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::FRAC_PI_6 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_8,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::FRAC_PI_8 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_LN_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::LN_2 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_LN_10,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::LN_10 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_LOG2_10,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::LOG2_10 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_LOG2_E,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::LOG2_E as f64),
        ),
    ),
    new_const(
        intern::INTERNED_LOG10_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::LOG10_2 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_LOG10_E,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::LOG10_E as f64),
        ),
    ),
    new_const(
        intern::INTERNED_SQRT_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::SQRT_2 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_SQRT_3,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(1.732050807568877293527446341505872366_f32 as f64),
        ),
    ),
    new_const(
        intern::INTERNED_GOLDEN_RATIO,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::GOLDEN_RATIO as f64),
        ),
    ),
    new_const(
        intern::INTERNED_EULER_GAMMA,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f32::consts::EULER_GAMMA as f64),
        ),
    ),
    new_i64_metadata(intern::INTERNED_SIGN_BITS, 1),
    new_i64_metadata(intern::INTERNED_EXPONENT_BITS, 8),
    new_i64_metadata(intern::INTERNED_SIGNIFICAND_BITS, 24),
    new_i64_metadata(intern::INTERNED_STORED_SIGNIFICAND_BITS, 23),
    new_i64_metadata(intern::INTERNED_EXPONENT_BIAS, 127),
    new_i64_metadata(intern::INTERNED_MIN_NORMAL_EXPONENT, -126),
    new_i64_metadata(intern::INTERNED_MAX_NORMAL_EXPONENT, 127),
    new_i64_metadata(intern::INTERNED_MIN_SUBNORMAL_EXPONENT, -149),
];

static NAMESPACE_I64: [InstantiationSymbolBase; 4] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::I64),
        InstantiationValueKind::I64(i64::MAX),
    )),
    new_min(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::I64),
        InstantiationValueKind::I64(i64::MIN),
    )),
    new_bits(64),
    new_bytes(8),
];

static NAMESPACE_U64: [InstantiationSymbolBase; 4] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::U64),
        InstantiationValueKind::U64(u64::MAX),
    )),
    new_min_unsigned(),
    new_bits(64),
    new_bytes(8),
];

static NAMESPACE_F64: [InstantiationSymbolBase; 42] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::F64),
        InstantiationValueKind::F64(f64::MAX),
    )),
    new_min(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::F64),
        InstantiationValueKind::F64(f64::MIN),
    )),
    new_bits(64),
    new_bytes(8),
    new_radix(2),
    new_digits(f64::DIGITS as i64),
    new_mantissa_digits(f64::MANTISSA_DIGITS as i64),
    new_const(
        intern::INTERNED_EPSILON,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(f64::EPSILON),
        ),
    ),
    new_const(
        intern::INTERNED_INF,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(f64::INFINITY),
        ),
    ),
    new_const(
        intern::INTERNED_NEG_INF,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(f64::NEG_INFINITY),
        ),
    ),
    new_const(
        intern::INTERNED_NAN,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(f64::NAN),
        ),
    ),
    new_const(
        intern::INTERNED_MIN_POSITIVE,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(f64::MIN_POSITIVE),
        ),
    ),
    new_const(
        intern::INTERNED_PI_UPPER,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::PI),
        ),
    ),
    new_const(
        intern::INTERNED_E_UPPER,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::E),
        ),
    ),
    new_const(
        intern::INTERNED_TAU,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::TAU),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_1_PI,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::FRAC_1_PI),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_1_SQRT_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::FRAC_1_SQRT_2),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_2_PI,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::FRAC_2_PI),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_2_SQRT_PI,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::FRAC_2_SQRT_PI),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::FRAC_PI_2),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_3,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::FRAC_PI_3),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_4,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::FRAC_PI_4),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_6,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::FRAC_PI_6),
        ),
    ),
    new_const(
        intern::INTERNED_FRAC_PI_8,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::FRAC_PI_8),
        ),
    ),
    new_const(
        intern::INTERNED_LN_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::LN_2),
        ),
    ),
    new_const(
        intern::INTERNED_LN_10,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::LN_10),
        ),
    ),
    new_const(
        intern::INTERNED_LOG2_10,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::LOG2_10),
        ),
    ),
    new_const(
        intern::INTERNED_LOG2_E,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::LOG2_E),
        ),
    ),
    new_const(
        intern::INTERNED_LOG10_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::LOG10_2),
        ),
    ),
    new_const(
        intern::INTERNED_LOG10_E,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::LOG10_E),
        ),
    ),
    new_const(
        intern::INTERNED_SQRT_2,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::SQRT_2),
        ),
    ),
    new_const(
        intern::INTERNED_SQRT_3,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(1.732050807568877293527446341505872366_f64),
        ),
    ),
    new_const(
        intern::INTERNED_GOLDEN_RATIO,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::GOLDEN_RATIO),
        ),
    ),
    new_const(
        intern::INTERNED_EULER_GAMMA,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::F64),
            InstantiationValueKind::F64(std::f64::consts::EULER_GAMMA),
        ),
    ),
    new_i64_metadata(intern::INTERNED_SIGN_BITS, 1),
    new_i64_metadata(intern::INTERNED_EXPONENT_BITS, 11),
    new_i64_metadata(intern::INTERNED_SIGNIFICAND_BITS, 53),
    new_i64_metadata(intern::INTERNED_STORED_SIGNIFICAND_BITS, 52),
    new_i64_metadata(intern::INTERNED_EXPONENT_BIAS, 1023),
    new_i64_metadata(intern::INTERNED_MIN_NORMAL_EXPONENT, -1022),
    new_i64_metadata(intern::INTERNED_MAX_NORMAL_EXPONENT, 1023),
    new_i64_metadata(intern::INTERNED_MIN_SUBNORMAL_EXPONENT, -1074),
];

//TODO: `f128` bounds do not fit `InstiationValue`, which only carries `F64` and `DBig` (decimal).
//`sized`/`unsized` are pointer-sized, so their bounds belong to the target rather than the host.
//The 128-bit float stays empty until float values cover 128-bit IEEE floats.

static NAMESPACE_I128: [InstantiationSymbolBase; 4] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::BigInt),
        InstantiationValueKind::BigInt(IBig::from_parts_const(
            dashu_int::Sign::Positive,
            i128::MAX as u128,
        )),
    )),
    new_min(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::BigInt),
        InstantiationValueKind::BigInt(IBig::from_parts_const(
            dashu_int::Sign::Negative,
            i128::MIN.unsigned_abs(),
        )),
    )),
    new_bits(128),
    new_bytes(16),
];

static NAMESPACE_U128: [InstantiationSymbolBase; 4] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::BigInt),
        InstantiationValueKind::BigInt(IBig::from_parts_const(
            dashu_int::Sign::Positive,
            u128::MAX,
        )),
    )),
    new_min_unsigned(),
    new_bits(128),
    new_bytes(16),
];

//NOTE: `f128` bounds (MAX/MIN) do not fit `InstiationValue`, which only carries `F64` and `DBig` (decimal).
//`f128` acts as a metadata namespace providing representable bit-range and precision attributes.
static NAMESPACE_F128: [InstantiationSymbolBase; 13] = [
    new_bits(128),
    new_bytes(16),
    new_radix(2),
    new_digits(33),
    new_mantissa_digits(113),
    new_i64_metadata(intern::INTERNED_SIGN_BITS, 1),
    new_i64_metadata(intern::INTERNED_EXPONENT_BITS, 15),
    new_i64_metadata(intern::INTERNED_SIGNIFICAND_BITS, 113),
    new_i64_metadata(intern::INTERNED_STORED_SIGNIFICAND_BITS, 112),
    new_i64_metadata(intern::INTERNED_EXPONENT_BIAS, 16383),
    new_i64_metadata(intern::INTERNED_MIN_NORMAL_EXPONENT, -16382),
    new_i64_metadata(intern::INTERNED_MAX_NORMAL_EXPONENT, 16383),
    new_i64_metadata(intern::INTERNED_MIN_SUBNORMAL_EXPONENT, -16494),
];

static NAMESPACE_BOOL: [InstantiationSymbolBase; 2] = [new_bits(1), new_bytes(1)];

static NAMESPACE_CHAR: [InstantiationSymbolBase; 22] = [
    new_max(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::Char),
        InstantiationValueKind::Char(char::MAX),
    )),
    new_min(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::Char),
        InstantiationValueKind::Char(char::MIN),
    )),
    // Should chrn specific metadata be more so CHRN_BITS or is that odd?
    new_bits(32),
    new_bytes(4),
    new_char_const(intern::INTERNED_NUL, '\0'),
    new_char_const(intern::INTERNED_SPACE_UPPER, ' '),
    new_char_const(intern::INTERNED_TAB_UPPER, '\t'),
    new_char_const(intern::INTERNED_NEWLINE_UPPER, '\n'),
    new_char_const(intern::INTERNED_CARRIAGE_RETURN_UPPER, '\r'),
    new_char_const(
        intern::INTERNED_REPLACEMENT_CHARACTER_UPPER,
        char::REPLACEMENT_CHARACTER,
    ),
    new_i64_metadata(intern::INTERNED_UTF8_MAX_BYTES, 4),
    new_i64_metadata(intern::INTERNED_UTF8_MIN_BYTES, 1),
    new_i64_metadata(intern::INTERNED_UTF8_MAX_BITS, 32),
    new_i64_metadata(intern::INTERNED_UTF8_MIN_BITS, 8),
    new_i64_metadata(intern::INTERNED_UTF16_MAX_BYTES, 4),
    new_i64_metadata(intern::INTERNED_UTF16_MIN_BYTES, 2),
    new_i64_metadata(intern::INTERNED_UTF16_MAX_BITS, 32),
    new_i64_metadata(intern::INTERNED_UTF16_MIN_BITS, 16),
    new_i64_metadata(intern::INTERNED_UTF16_MAX_CODE_UNITS, 2),
    new_i64_metadata(intern::INTERNED_UTF16_MIN_CODE_UNITS, 1),
    new_i64_metadata(intern::INTERNED_UTF32_BYTES, 4),
    new_i64_metadata(intern::INTERNED_UTF32_BITS, 32),
];

static NAMESPACE_STR: [InstantiationSymbolBase; 7] = [
    new_str_const(intern::INTERNED_TAB_UPPER, intern::INTERNED_TAB_VALUE),
    new_str_const(
        intern::INTERNED_NEWLINE_UPPER,
        intern::INTERNED_NEWLINE_VALUE,
    ),
    new_str_const(
        intern::INTERNED_CARRIAGE_RETURN_UPPER,
        intern::INTERNED_CARRIAGE_RETURN_VALUE,
    ),
    new_i64_metadata(intern::INTERNED_UTF8_MAX_BYTES, 4),
    new_i64_metadata(intern::INTERNED_UTF8_MIN_BYTES, 1),
    new_i64_metadata(intern::INTERNED_UTF8_MAX_BITS, 32),
    new_i64_metadata(intern::INTERNED_UTF8_MIN_BITS, 8),
];

/// What a set of instantiation bases contributes to the compiler's arenas when registered.
#[derive(Default)]
pub struct InstantiationReservations {
    pub symbols: usize,
    pub scopes: usize,
    /// One `VarDef`, one `ResolvedExpr`, one `ValueInfo`, and one constant `Value` each.
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
                counts.merge(count_instantiation_bases(inner.ns));
            }
            InstantiationSymbolKind::Variable(_) => counts.variables += 1,
            InstantiationSymbolKind::Type(ty) => {
                if let Some(ns) = ty.ns.as_ref().filter(|ns| !ns.ns.is_empty()) {
                    counts.scopes += 1;
                    counts.merge(count_instantiation_bases(ns.ns));
                }
            }
            InstantiationSymbolKind::ExternType(_) => (),
        }
    }

    counts
}

/// Counts the types and funcs namespaces and intrinsic type members, excluding type and function
/// symbols reserved separately.
pub fn core_instantiation_reservations() -> InstantiationReservations {
    let mut counts = InstantiationReservations {
        symbols: CORE_NAMESPACE.len(),
        scopes: CORE_NAMESPACE.len(),
        variables: 0,
    };

    for base in &TYPES_NAMESPACE {
        let InstantiationSymbolKind::Type(ty) = &base.kind else {
            unreachable!();
        };
        let ns = ty.ns.as_ref().map(|ns| ns.ns).unwrap_or(&[]);
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
    pub arg_constraints: &'static [FuncArgConstraint],
    pub affects_type_constraint: bool,
    /// `TypeId` of the return type, which must already be loaded as a core type
    pub ret_type: u32,
}

impl CoreFunc {
    const fn new(
        name: u32,
        kind: BuiltinFuncKind,
        type_constraints: TypeBoundaryFlags,
        arg_constraints: &'static [FuncArgConstraint],
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
/// the built-in types, the unknown type, and the boundary types.
pub static CORE_FUNCS_DATASET: [CoreFunc; 7] = [
    CoreFunc::new(
        intern::INTERNED_IS_EMPTY,
        BuiltinFuncKind::IsEmpty,
        TypeBoundaryFlags::COLLECTION,
        &[FuncArgConstraint::ArgCount(0)],
        true,
        CORE_BOOL,
    ),
    CoreFunc::new(
        intern::INTERNED_IS_WHITESPACE,
        BuiltinFuncKind::IsWhitespace,
        TypeBoundaryFlags::CHARACTER_MAPPABLE,
        &[
            FuncArgConstraint::ArgCount(0),
            FuncArgConstraint::CharacterMappable,
        ],
        true,
        CORE_BOOL,
    ),
    CoreFunc::new(
        intern::INTERNED_CONTAINS,
        BuiltinFuncKind::Contains,
        TypeBoundaryFlags::CHARACTER_MAPPABLE,
        &[
            FuncArgConstraint::ArgCount(1),
            FuncArgConstraint::CharacterMappable,
        ],
        true,
        CORE_BOOL,
    ),
    CoreFunc::new(
        intern::INTERNED_STARTSW,
        BuiltinFuncKind::StartsW,
        TypeBoundaryFlags::CHARACTER_MAPPABLE,
        &[
            FuncArgConstraint::ArgCount(1),
            FuncArgConstraint::CharacterMappable,
        ],
        true,
        CORE_BOOL,
    ),
    CoreFunc::new(
        intern::INTERNED_ENDSW,
        BuiltinFuncKind::EndsW,
        TypeBoundaryFlags::CHARACTER_MAPPABLE,
        &[
            FuncArgConstraint::ArgCount(1),
            FuncArgConstraint::CharacterMappable,
        ],
        true,
        CORE_BOOL,
    ),
    CoreFunc::new(
        intern::INTERNED_RANGE,
        BuiltinFuncKind::Range,
        TypeBoundaryFlags::RANGED,
        &[
            FuncArgConstraint::ArgCount(2),
            FuncArgConstraint::Numeric,
            FuncArgConstraint::MatchingArgumentTypes,
            FuncArgConstraint::SameTypeAsSelf,
        ],
        true,
        CORE_BOOL,
    ),
    CoreFunc::new(
        intern::INTERNED_EQUALS,
        BuiltinFuncKind::Equals,
        TypeBoundaryFlags::COMPARABLE,
        &[
            FuncArgConstraint::ArgCount(1),
            FuncArgConstraint::Comparable,
            FuncArgConstraint::SameTypeAsSelf,
        ],
        true,
        CORE_BOOL,
    ),
];

const fn new_const(id: u32, var: InstantiationVariable) -> InstantiationSymbolBase {
    InstantiationSymbolBase::new(
        InternedId::new(id).into_tagged::<UTF8Tag>(),
        // More like core_sym_origin
        CORE_SYM_ORIGIN,
        CORE_SCOPE_TYPE,
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
const fn new_min_unsigned() -> InstantiationSymbolBase {
    new_min(InstantiationVariable::new(
        InstantiationTypeKind::BuiltinType(BuiltinType::U64),
        InstantiationValueKind::U64(0),
    ))
}
const fn new_bits(bits: i64) -> InstantiationSymbolBase {
    new_const(
        intern::INTERNED_BITS_UPPER,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::I64),
            InstantiationValueKind::I64(bits),
        ),
    )
}
const fn new_bytes(bytes: i64) -> InstantiationSymbolBase {
    new_const(
        intern::INTERNED_BYTES_UPPER,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::I64),
            InstantiationValueKind::I64(bytes),
        ),
    )
}
const fn new_radix(radix: i64) -> InstantiationSymbolBase {
    new_const(
        intern::INTERNED_RADIX,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::I64),
            InstantiationValueKind::I64(radix),
        ),
    )
}
const fn new_digits(digits: i64) -> InstantiationSymbolBase {
    new_const(
        intern::INTERNED_DIGITS,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::I64),
            InstantiationValueKind::I64(digits),
        ),
    )
}
const fn new_mantissa_digits(mantissa_digits: i64) -> InstantiationSymbolBase {
    new_const(
        intern::INTERNED_MANTISSA_DIGITS,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::I64),
            InstantiationValueKind::I64(mantissa_digits),
        ),
    )
}

const fn new_char_const(name: u32, value: char) -> InstantiationSymbolBase {
    new_const(
        name,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::Char),
            InstantiationValueKind::Char(value),
        ),
    )
}
const fn new_str_const(name: u32, value: u32) -> InstantiationSymbolBase {
    new_const(
        name,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::Str),
            InstantiationValueKind::Str(InternedId::new(value).into_tagged::<UTF8Tag>()),
        ),
    )
}
const fn new_i64_metadata(name: u32, value: i64) -> InstantiationSymbolBase {
    new_const(
        name,
        InstantiationVariable::new(
            InstantiationTypeKind::BuiltinType(BuiltinType::I64),
            InstantiationValueKind::I64(value),
        ),
    )
}
