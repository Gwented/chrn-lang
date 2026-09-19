use std::str::FromStr;

use chrn_utils::id_types::TypeId;
use dashu_float::DBig;

use crate::script_compiler::compiler_constants;

// /// Arbitrarily sized float
// #[derive(Debug, Clone)]
// pub struct BigFloatBase {
//     pub inner: BigFloat,
//     // pub notation: Notation,
// }
//
// impl BigFloatBase {
//     pub fn new(inner: BigFloat) -> Self {
//         Self { inner }
//     }
// }

/// Representable `chrn` floating points
#[derive(Debug, Clone)]
pub enum ArbitraryFloatKind {
    // Is locking out 128-bit too much?
    F64(f64),
    BigFloat(DBig),
}

impl ArbitraryFloatKind {
    /// Returns compile-time known `TypeId` for `Self`
    pub const fn type_id(&self) -> TypeId {
        match self {
            Self::F64(_) => TypeId::new(compiler_constants::CORE_F64),
            Self::BigFloat(_) => TypeId::new(compiler_constants::CORE_BIGFLOAT),
        }
    }
    /// Attempts to convert to a valid arbitrary float from a given `str`
    pub fn from_str(s: &str) -> Option<Self> {
        if let Ok(num) = s.parse::<f64>() {
            if num.is_finite() && (num != 0.0 || !s.chars().any(|c| matches!(c, '1'..='9'))) {
                return Self::F64(num).into();
            }
            // Overflow to `inf` or underflow to `0.0`: fall through to arbitrary-precision `DBig`
            // so huge or tiny literals don't collapse to infinity or zero.
        }
        // Might insert scientific so keeps notation
        match DBig::from_str(s) {
            Ok(v) => {
                if !v.repr().is_infinite() && v.repr().significand().is_zero() {
                    let zero = if s.starts_with('-') { -0.0 } else { 0.0 };
                    Self::F64(zero).into()
                } else {
                    Self::BigFloat(v).into()
                }
            }
            Err(_) => None,
        }
    }

    /// Converts a finite or infinite `f64` to `DBig` through its decimal `{:e}` rendering.
    ///
    /// Infinite inputs map to infinities. Returns `None` for `NaN` since `DBig` cannot
    /// represent NaN.
    fn dbig_from_f64(v: f64) -> Option<DBig> {
        if v.is_nan() {
            None
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                Some(DBig::INFINITY)
            } else {
                Some(DBig::NEG_INFINITY)
            }
        } else {
            DBig::from_str(&format!("{:e}", v)).ok()
        }
    }

    /// Converts to `DBig`, cloning on the `BigFloat` variant. Returns `None` if `self` is `NaN`.
    pub fn to_bigfloat(&self) -> Option<DBig> {
        match self {
            Self::F64(v) => Self::dbig_from_f64(*v),
            Self::BigFloat(v) => Some(v.clone()),
        }
    }

    /// Converts to `DBig`, moving on the `BigFloat` variant. Returns `None` if `self` is `NaN`.
    pub fn into_bigfloat(self) -> Option<DBig> {
        match self {
            Self::F64(v) => Self::dbig_from_f64(v),
            Self::BigFloat(v) => Some(v),
        }
    }

    /// Converts to `f64`, rounding on the `BigFloat` variant.
    pub fn to_f64(&self) -> f64 {
        match self {
            Self::F64(v) => *v,
            Self::BigFloat(v) => v.to_f64().value(),
        }
    }

    /// Mirrors `f64::to_bits`, rounding `BigFloat` through `f64`.
    pub fn to_bits(&self) -> u64 {
        self.to_f64().to_bits()
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Self::F64(v) => *v == 0.0,
            // Infinities share a zero significand, so exclude them explicitly.
            Self::BigFloat(v) => !v.repr().is_infinite() && v.repr().significand().is_zero(),
        }
    }

    /// Returns `false` for NaN, infinite `f64`, and infinite `DBig`.
    ///
    /// `DBig` has no NaN and rejects infinite inputs to operators, so mixed
    /// operator paths use this to stay in `f64` (which has IEEE semantics)
    /// instead of converting.
    pub fn is_finite(&self) -> bool {
        match self {
            Self::F64(v) => v.is_finite(),
            Self::BigFloat(v) => !v.repr().is_infinite(),
        }
    }

    fn eval_binop<F>(
        self,
        rhs: Self,
        is_div: bool,
        is_rem: bool,
        f64_op: fn(f64, f64) -> f64,
        dbig_op: F,
    ) -> Self
    where
        F: FnOnce(DBig, DBig) -> DBig,
    {
        match (self, rhs) {
            (Self::F64(a), Self::F64(b)) => Self::F64(f64_op(a, b)),
            (a, b) => {
                if is_div && b.is_zero() {
                    let l_norm = if a.is_zero() {
                        0.0
                    } else if a.to_f64().is_nan() {
                        f64::NAN
                    } else if a.to_f64().is_sign_positive() {
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
                    return Self::op_with_non_finite(&a, &b, f64_op, is_rem);
                }
                let a_big = a.into_bigfloat().expect("finite float converts to DBig");
                let b_big = b.into_bigfloat().expect("finite float converts to DBig");
                Self::BigFloat(dbig_op(a_big, b_big))
            }
        }
    }

    fn op_with_non_finite(lhs: &Self, rhs: &Self, op: fn(f64, f64) -> f64, is_rem: bool) -> Self {
        let l_f = lhs.to_f64();
        let r_f = rhs.to_f64();
        if l_f.is_nan() || r_f.is_nan() {
            return Self::F64(f64::NAN);
        }
        if is_rem {
            if lhs.is_finite() && !rhs.is_finite() {
                return lhs.clone();
            }
            return Self::F64(f64::NAN);
        }
        let l_norm = if lhs.is_finite() && (l_f.is_infinite() || (l_f == 0.0 && !lhs.is_zero())) {
            if l_f.is_sign_positive() { 1.0 } else { -1.0 }
        } else {
            l_f
        };
        let r_norm = if rhs.is_finite() && (r_f.is_infinite() || (r_f == 0.0 && !rhs.is_zero())) {
            if r_f.is_sign_positive() { 1.0 } else { -1.0 }
        } else {
            r_f
        };
        Self::F64(op(l_norm, r_norm))
    }
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
            ArbitraryFloatKind::BigFloat(bf) => ArbitraryFloatKind::BigFloat(-bf.clone()),
        }
    }
}

// Float ops stay in `f64` when both sides are `F64` to preserve native
// NaN/inf/signed-zero behavior, and promote to `DBig` otherwise.
// `DBig` has no NaN and rejects infinite inputs, so mixed operations with a
// non-finite side stay in `f64` where IEEE semantics apply.
// Bitwise ops and shifts are intentionally absent: they are not defined
// for floats, matching fixed-width float behavior.
macro_rules! impl_float_binop {
    ($trait:ident, $method:ident, $is_div:expr, $is_rem:expr) => {
        impl std::ops::$trait for ArbitraryFloatKind {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                self.eval_binop(
                    rhs,
                    $is_div,
                    $is_rem,
                    |a, b| a.$method(b),
                    |a, b| a.$method(b),
                )
            }
        }

        impl<'a, 'b> std::ops::$trait<&'b ArbitraryFloatKind> for &'a ArbitraryFloatKind {
            type Output = ArbitraryFloatKind;

            fn $method(self, rhs: &'b ArbitraryFloatKind) -> Self::Output {
                (*self).clone().eval_binop(
                    (*rhs).clone(),
                    $is_div,
                    $is_rem,
                    |a, b| a.$method(b),
                    |a, b| a.$method(b),
                )
            }
        }

        impl<'a> std::ops::$trait<&'a ArbitraryFloatKind> for ArbitraryFloatKind {
            type Output = ArbitraryFloatKind;

            fn $method(self, rhs: &'a ArbitraryFloatKind) -> Self::Output {
                self.eval_binop(
                    (*rhs).clone(),
                    $is_div,
                    $is_rem,
                    |a, b| a.$method(b),
                    |a, b| a.$method(b),
                )
            }
        }

        impl<'a> std::ops::$trait<ArbitraryFloatKind> for &'a ArbitraryFloatKind {
            type Output = ArbitraryFloatKind;

            fn $method(self, rhs: ArbitraryFloatKind) -> Self::Output {
                (*self).clone().eval_binop(
                    rhs,
                    $is_div,
                    $is_rem,
                    |a, b| a.$method(b),
                    |a, b| a.$method(b),
                )
            }
        }
    };
}

impl_float_binop!(Add, add, false, false);
impl_float_binop!(Sub, sub, false, false);
impl_float_binop!(Mul, mul, false, false);
impl_float_binop!(Div, div, true, false);
impl_float_binop!(Rem, rem, false, true);
