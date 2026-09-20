use std::{borrow::Cow, str::FromStr};

use chrn_utils::id_types::TypeId;
use dashu_float::{
    DBig,
    round::mode::{HalfAway, HalfEven},
};
use dashu_int::{IBig, ops::BitTest};

use crate::script_compiler::compiler_constants;

/// Representable `chrn` floating points.
#[derive(Debug, Clone)]
pub enum ArbitraryFloatKind {
    // Is locking out 128-bit too much?
    F64(f64),
    BigFloat(DBig),
}

enum FloatOperand<'a> {
    Owned(ArbitraryFloatKind),
    Borrowed(&'a ArbitraryFloatKind),
}

impl<'a> FloatOperand<'a> {
    fn as_ref(&self) -> &ArbitraryFloatKind {
        match self {
            Self::Owned(value) => value,
            Self::Borrowed(value) => value,
        }
    }

    fn into_owned(self) -> ArbitraryFloatKind {
        match self {
            Self::Owned(value) => value,
            Self::Borrowed(value) => value.clone(),
        }
    }

    fn into_bigfloat(self) -> Option<Cow<'a, DBig>> {
        match self {
            Self::Owned(ArbitraryFloatKind::F64(value)) => {
                ArbitraryFloatKind::dbig_from_f64(value).map(Cow::Owned)
            }
            Self::Owned(ArbitraryFloatKind::BigFloat(value)) => Some(Cow::Owned(value)),
            Self::Borrowed(ArbitraryFloatKind::F64(value)) => {
                ArbitraryFloatKind::dbig_from_f64(*value).map(Cow::Owned)
            }
            Self::Borrowed(ArbitraryFloatKind::BigFloat(value)) => Some(Cow::Borrowed(value)),
        }
    }
}

impl ArbitraryFloatKind {
    /// Returns compile-time `TypeId` for `Self`.
    pub const fn type_id(&self) -> TypeId {
        match self {
            Self::F64(_) => TypeId::new(compiler_constants::CORE_F64),
            Self::BigFloat(_) => TypeId::new(compiler_constants::CORE_BIGFLOAT),
        }
    }
    /// Parses a float string.
    pub fn from_str(s: &str) -> Option<Self> {
        Self::from_str_with_limit(s, u64::MAX).ok()
    }

    /// Parses a float literal respecting `max_bits`.
    pub fn from_str_with_limit(s: &str, max_bits: u64) -> Result<Self, NumericFloatParseError> {
        if let Ok(num) = s.parse::<f64>() {
            let has_nonzero_digit = s.chars().any(|c| matches!(c, '1'..='9'));
            if num.is_nan()
                || (num.is_infinite() && !has_nonzero_digit)
                || (num.is_finite() && (num != 0.0 || !has_nonzero_digit))
            {
                let kind = Self::F64(num);
                return if kind.fits_numeric_bits(max_bits) {
                    Ok(kind)
                } else {
                    Err(NumericFloatParseError::LimitExceeded)
                };
            }
        }
        // Might insert scientific so keeps notation
        match DBig::from_str(s) {
            Ok(v) => {
                let kind = if !v.repr().is_infinite() && v.repr().significand().is_zero() {
                    let zero = if s.starts_with('-') { -0.0 } else { 0.0 };
                    Self::F64(zero)
                } else {
                    Self::BigFloat(v)
                };
                if kind.fits_numeric_bits(max_bits) {
                    Ok(kind)
                } else {
                    Err(NumericFloatParseError::LimitExceeded)
                }
            }
            Err(_) => Err(NumericFloatParseError::Invalid),
        }
    }

    /// Bit size work for `Self`, including scale and precision.
    pub fn numeric_bits(&self) -> u64 {
        match self {
            Self::F64(v) if !v.is_finite() => u64::MAX,
            Self::F64(_) => 64,
            Self::BigFloat(v) if v.repr().is_infinite() => u64::MAX,
            Self::BigFloat(v) => {
                let significand_bits = v.repr().significand().bit_len() as u64;
                let precision_bits = decimal_digits_to_bits(v.precision() as u64);
                let scale = v.repr().exponent().unsigned_abs() as u64;
                significand_bits
                    .max(precision_bits)
                    .saturating_add(decimal_digits_to_bits(scale))
            }
        }
    }

    pub fn fits_numeric_bits(&self, max_bits: u64) -> bool {
        self.numeric_bits() <= max_bits
    }

    /// Converts an `f64` to `DBig`. Returns `None` for NaN.
    fn dbig_from_f64(v: f64) -> Option<DBig> {
        if v.is_nan() {
            None
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                Some(DBig::INFINITY)
            } else {
                Some(DBig::NEG_INFINITY)
            }
        } else if v == 0.0 {
            let zero = if v.is_sign_negative() { "-0" } else { "0" };
            DBig::from_str(zero).ok()
        } else {
            let bits = v.to_bits();
            let negative = bits >> 63 != 0;
            let encoded_exponent = ((bits >> 52) & 0x7ff) as i32;
            let fraction = bits & ((1_u64 << 52) - 1);
            let (significand, exponent) = if encoded_exponent == 0 {
                (fraction, -1074)
            } else {
                (fraction | (1_u64 << 52), encoded_exponent - 1023 - 52)
            };

            let mut significand = IBig::from(significand);
            if negative {
                significand = -significand;
            }

            Some(if exponent >= 0 {
                DBig::from_parts(significand << exponent as usize, 0)
            } else {
                let decimal_scale = (-exponent) as usize;
                let decimal_significand = significand * IBig::from(5_u8).pow(decimal_scale);
                DBig::from_parts(decimal_significand, -(decimal_scale as isize))
            })
        }
    }

    /// Converts to `DBig`. Returns `None` for NaN.
    pub fn to_bigfloat(&self) -> Option<DBig> {
        match self {
            Self::F64(v) => Self::dbig_from_f64(*v),
            Self::BigFloat(v) => Some(v.clone()),
        }
    }

    /// Converts into `DBig`. Returns `None` for NaN.
    pub fn into_bigfloat(self) -> Option<DBig> {
        match self {
            Self::F64(v) => Self::dbig_from_f64(v),
            Self::BigFloat(v) => Some(v),
        }
    }

    /// Converts to `f64`, rounding if necessary.
    pub fn to_f64(&self) -> f64 {
        match self {
            Self::F64(v) => *v,
            Self::BigFloat(v) => v.clone().with_rounding::<HalfEven>().to_f64().value(),
        }
    }

    /// Raw bits as `u64`, rounding through `f64`.
    pub fn to_bits(&self) -> u64 {
        self.to_f64().to_bits()
    }

    /// `Self` == 0
    pub fn is_zero(&self) -> bool {
        match self {
            Self::F64(v) => *v == 0.0,
            // Infinities share a zero significand, so exclude them explicitly.
            Self::BigFloat(v) => !v.repr().is_infinite() && v.repr().significand().is_zero(),
        }
    }

    /// Returns false for NaN or infinities.
    pub fn is_finite(&self) -> bool {
        match self {
            Self::F64(v) => v.is_finite(),
            Self::BigFloat(v) => !v.repr().is_infinite(),
        }
    }

    fn eval_binop<'a, F>(
        lhs: FloatOperand<'a>,
        rhs: FloatOperand<'a>,
        is_div: bool,
        is_rem: bool,
        f64_op: fn(f64, f64) -> f64,
        dbig_op: F,
    ) -> Self
    where
        F: FnOnce(Cow<'a, DBig>, Cow<'a, DBig>) -> DBig,
    {
        match (lhs.as_ref(), rhs.as_ref()) {
            (Self::F64(a), Self::F64(b)) => Self::F64(f64_op(*a, *b)),
            (a, b) => {
                if is_div && b.is_zero() {
                    let lhs = a.to_f64();
                    let l_norm = if a.is_zero() {
                        0.0
                    } else if lhs.is_nan() {
                        f64::NAN
                    } else if lhs.is_sign_positive() {
                        1.0
                    } else {
                        -1.0
                    };
                    return Self::F64(l_norm / b.to_f64());
                }
                if is_rem && b.is_zero() {
                    return Self::F64(f64::NAN);
                }
                if !a.is_finite() || !b.is_finite() {
                    if is_rem && a.is_finite() && !b.is_finite() {
                        return lhs.into_owned();
                    }
                    return Self::op_with_non_finite(a, b, f64_op, is_rem);
                }
                let zero_sign = f64_op(a.ieee_proxy(), b.ieee_proxy()).is_sign_negative();
                let a_big = lhs.into_bigfloat().expect("finite float converts to DBig");
                let b_big = rhs.into_bigfloat().expect("finite float converts to DBig");
                let result = dbig_op(a_big, b_big);
                if !result.repr().is_infinite() && result.repr().significand().is_zero() {
                    Self::F64(if zero_sign { -0.0 } else { 0.0 })
                } else {
                    Self::BigFloat(result)
                }
            }
        }
    }

    /// Maps out-of-range finite values to a same-sign finite `f64` proxy.
    fn ieee_proxy(&self) -> f64 {
        let converted = self.to_f64();
        if self.is_finite() && (converted.is_infinite() || (converted == 0.0 && !self.is_zero())) {
            if converted.is_sign_positive() {
                1.0
            } else {
                -1.0
            }
        } else {
            converted
        }
    }

    fn op_with_non_finite(lhs: &Self, rhs: &Self, op: fn(f64, f64) -> f64, is_rem: bool) -> Self {
        let l_f = lhs.to_f64();
        let r_f = rhs.to_f64();
        if l_f.is_nan() || r_f.is_nan() {
            return Self::F64(f64::NAN);
        }
        if is_rem {
            return Self::F64(f64::NAN);
        }
        Self::F64(op(lhs.ieee_proxy(), rhs.ieee_proxy()))
    }

    /// Computes truncating remainder for `%`.
    fn dbig_trunc_rem(lhs: Cow<'_, DBig>, rhs: Cow<'_, DBig>) -> DBig {
        let lhs_is_negative = lhs.as_ref() < &DBig::ZERO;
        let rhs_is_negative = rhs.as_ref() < &DBig::ZERO;
        let mut remainder = lhs.into_owned().with_rounding::<HalfEven>()
            % rhs.as_ref().clone().with_rounding::<HalfEven>();

        if !remainder.repr().significand().is_zero() && (remainder < DBig::ZERO) != lhs_is_negative
        {
            let divisor_magnitude = if rhs_is_negative {
                -rhs.into_owned().with_rounding::<HalfEven>()
            } else {
                rhs.into_owned().with_rounding::<HalfEven>()
            };
            remainder = if lhs_is_negative {
                remainder - divisor_magnitude
            } else {
                remainder + divisor_magnitude
            };
        }

        remainder.with_rounding::<HalfAway>()
    }
}

/// Bit estimate for decimal digits.
fn decimal_digits_to_bits(digits: u64) -> u64 {
    digits.saturating_mul(3322).div_ceil(1000)
}

/// Float parsing errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericFloatParseError {
    Invalid,
    LimitExceeded,
}

impl std::fmt::Display for ArbitraryFloatKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArbitraryFloatKind::F64(num) => write!(f, "{num}"),
            ArbitraryFloatKind::BigFloat(fbig) => write!(f, "{fbig}"),
        }
    }
}

impl PartialEq for ArbitraryFloatKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::F64(a), Self::F64(b)) => a == b,
            (Self::BigFloat(a), Self::BigFloat(b)) => a == b,
            // `DBig` cannot represent NaN, so check before converting.
            (Self::F64(a), Self::BigFloat(b)) => {
                if a.is_nan() {
                    return false;
                }
                if a.is_finite() && b.to_f64().value().is_infinite() {
                    return false;
                }
                Self::dbig_from_f64(*a).as_ref() == Some(b)
            }
            (Self::BigFloat(a), Self::F64(b)) => {
                if b.is_nan() {
                    return false;
                }
                if b.is_finite() && a.to_f64().value().is_infinite() {
                    return false;
                }
                Some(a) == Self::dbig_from_f64(*b).as_ref()
            }
        }
    }
}

impl PartialOrd for ArbitraryFloatKind {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::F64(a), Self::F64(b)) => a.partial_cmp(b),
            (Self::BigFloat(a), Self::BigFloat(b)) => a.partial_cmp(b),
            // `DBig` cannot represent NaN, so check before converting.
            (Self::F64(a), Self::BigFloat(b)) => {
                if a.is_nan() {
                    return None;
                }
                if a.is_finite() && b.to_f64().value().is_infinite() {
                    return if b.to_f64().value().is_sign_positive() {
                        Some(std::cmp::Ordering::Less)
                    } else {
                        Some(std::cmp::Ordering::Greater)
                    };
                }
                Self::dbig_from_f64(*a)?.partial_cmp(b)
            }
            (Self::BigFloat(a), Self::F64(b)) => {
                if b.is_nan() {
                    return None;
                }
                if b.is_finite() && a.to_f64().value().is_infinite() {
                    return if a.to_f64().value().is_sign_positive() {
                        Some(std::cmp::Ordering::Greater)
                    } else {
                        Some(std::cmp::Ordering::Less)
                    };
                }
                a.partial_cmp(&Self::dbig_from_f64(*b)?)
            }
        }
    }
}

/// Converts `f64` to `ArbitraryFloatKind`
pub fn float_from_f64(v: f64) -> ArbitraryFloatKind {
    ArbitraryFloatKind::F64(v)
}

/// Converts `f32` to `ArbitraryFloatKind`
pub fn float_from_f32(v: f32) -> ArbitraryFloatKind {
    ArbitraryFloatKind::F64(v as f64)
}

impl From<DBig> for ArbitraryFloatKind {
    fn from(v: DBig) -> Self {
        Self::BigFloat(v)
    }
}

impl std::ops::Neg for ArbitraryFloatKind {
    type Output = ArbitraryFloatKind;

    fn neg(self) -> Self::Output {
        match self {
            ArbitraryFloatKind::F64(v) => Self::F64(-v),
            ArbitraryFloatKind::BigFloat(bf) => Self::BigFloat(-bf),
        }
    }
}

impl<'a> std::ops::Neg for &'a ArbitraryFloatKind {
    type Output = ArbitraryFloatKind;

    fn neg(self) -> Self::Output {
        match self {
            ArbitraryFloatKind::F64(v) => ArbitraryFloatKind::F64(-*v),
            ArbitraryFloatKind::BigFloat(bf) => ArbitraryFloatKind::BigFloat(-bf),
        }
    }
}

// Float ops stay in `f64` when both sides are `F64`, preserving binary64
// rounding, overflow, NaN, infinity, and signed-zero behavior.
// BigFloat addition, subtraction, multiplication, and division temporarily
// switch from `DBig`'s half-away default to IEEE 754's default
// round-to-nearest, ties-to-even mode. BigFloat `%` separately preserves the
// truncating-quotient semantics used by `F64` and chrn's `%` operator.
// `DBig` has no NaN and rejects infinite inputs, so mixed operations with a
// non-finite side stay in `f64` where IEEE semantics apply.
// Bitwise ops and shifts are intentionally absent: they are not defined
// for floats, matching fixed-width float behavior.
macro_rules! apply_dbig_op {
    ($method:ident, $lhs:expr, $rhs:expr) => {
        match ($lhs, $rhs) {
            (Cow::Owned(lhs), Cow::Owned(rhs)) => lhs
                .with_rounding::<HalfEven>()
                .$method(rhs.with_rounding::<HalfEven>()),
            (Cow::Owned(lhs), Cow::Borrowed(rhs)) => lhs
                .with_rounding::<HalfEven>()
                .$method(rhs.clone().with_rounding::<HalfEven>()),
            (Cow::Borrowed(lhs), Cow::Owned(rhs)) => lhs
                .clone()
                .with_rounding::<HalfEven>()
                .$method(rhs.with_rounding::<HalfEven>()),
            (Cow::Borrowed(lhs), Cow::Borrowed(rhs)) => lhs
                .clone()
                .with_rounding::<HalfEven>()
                .$method(rhs.clone().with_rounding::<HalfEven>()),
        }
        .with_rounding::<HalfAway>()
    };
}

macro_rules! impl_float_binop {
    ($trait:ident, $method:ident, $is_div:expr, $is_rem:expr, $dbig_op:expr) => {
        impl std::ops::$trait for ArbitraryFloatKind {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                Self::eval_binop(
                    FloatOperand::Owned(self),
                    FloatOperand::Owned(rhs),
                    $is_div,
                    $is_rem,
                    |a, b| a.$method(b),
                    $dbig_op,
                )
            }
        }

        impl<'a, 'b> std::ops::$trait<&'b ArbitraryFloatKind> for &'a ArbitraryFloatKind {
            type Output = ArbitraryFloatKind;

            fn $method(self, rhs: &'b ArbitraryFloatKind) -> Self::Output {
                ArbitraryFloatKind::eval_binop(
                    FloatOperand::Borrowed(self),
                    FloatOperand::Borrowed(rhs),
                    $is_div,
                    $is_rem,
                    |a, b| a.$method(b),
                    $dbig_op,
                )
            }
        }

        impl<'a> std::ops::$trait<&'a ArbitraryFloatKind> for ArbitraryFloatKind {
            type Output = ArbitraryFloatKind;

            fn $method(self, rhs: &'a ArbitraryFloatKind) -> Self::Output {
                ArbitraryFloatKind::eval_binop(
                    FloatOperand::Owned(self),
                    FloatOperand::Borrowed(rhs),
                    $is_div,
                    $is_rem,
                    |a, b| a.$method(b),
                    $dbig_op,
                )
            }
        }

        impl<'a> std::ops::$trait<ArbitraryFloatKind> for &'a ArbitraryFloatKind {
            type Output = ArbitraryFloatKind;

            fn $method(self, rhs: ArbitraryFloatKind) -> Self::Output {
                ArbitraryFloatKind::eval_binop(
                    FloatOperand::Borrowed(self),
                    FloatOperand::Owned(rhs),
                    $is_div,
                    $is_rem,
                    |a, b| a.$method(b),
                    $dbig_op,
                )
            }
        }
    };
}

impl_float_binop!(Add, add, false, false, |a, b| apply_dbig_op!(add, a, b));
impl_float_binop!(Sub, sub, false, false, |a, b| apply_dbig_op!(sub, a, b));
impl_float_binop!(Mul, mul, false, false, |a, b| apply_dbig_op!(mul, a, b));
impl_float_binop!(Div, div, true, false, |a, b| apply_dbig_op!(div, a, b));
impl_float_binop!(Rem, rem, false, true, ArbitraryFloatKind::dbig_trunc_rem);
