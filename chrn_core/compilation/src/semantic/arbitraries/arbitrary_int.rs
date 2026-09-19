use std::borrow::Cow;

use chrn_utils::id_types::TypeId;
use dashu_int::IBig;

use crate::lexer::token::Notation;
use crate::script_compiler::compiler_constants;

// /// Arbitrarily sized integer
// #[derive(Debug, Clone)]
// pub struct BigIntBase {
//     pub inner: BigInt,
//     // pub notation: Notation,
// }
//
// impl BigIntBase {
//     pub const fn new(inner: BigInt) -> Self {
//         Self { inner }
//     }
// }

/// Representable `chrn` integers
///
/// This goes to `u64` as to avoid using string arithmetic just from a common unsigned bit size.
#[derive(Debug, Clone)]
pub enum ArbitraryIntKind {
    I64(i64),
    U64(u64),
    BigInt(IBig),
}

impl ArbitraryIntKind {
    /// Returns compile-time known `TypeId` for `Self`
    pub const fn type_id(&self) -> TypeId {
        match self {
            Self::I64(_) => TypeId::new(compiler_constants::CORE_I64),
            Self::U64(_) => TypeId::new(compiler_constants::CORE_U64),
            Self::BigInt(_) => TypeId::new(compiler_constants::CORE_BIGINT),
        }
    }
    //WARN: Is this retrying ok?
    /// Given a string and target notation, outputs `Self` with the appropriate variant.
    pub fn from_str(s: &str, notation: Notation) -> Option<Self> {
        if let Ok(num) = i64::from_str_radix(s, notation.radix()) {
            Self::I64(num).into()
        } else if let Ok(num) = u64::from_str_radix(s, notation.radix()) {
            Self::U64(num).into()
        } else {
            match IBig::from_str_radix(s, notation.radix()) {
                Ok(v) => Self::from_bigint(v).into(),
                Err(_) => None,
            }
        }
    }

    /// Converts to `IBig`, cloning on the `BigInt` variant.
    pub fn to_bigint(&self) -> IBig {
        match self {
            Self::I64(v) => IBig::from(*v),
            Self::U64(v) => IBig::from(*v),
            Self::BigInt(v) => v.clone(),
        }
    }

    /// Borrows the `BigInt` variant or creates an owned `IBig` from primitive variants.
    pub fn to_bigint_cow(&self) -> Cow<'_, IBig> {
        match self {
            Self::I64(v) => Cow::Owned(IBig::from(*v)),
            Self::U64(v) => Cow::Owned(IBig::from(*v)),
            Self::BigInt(v) => Cow::Borrowed(v),
        }
    }

    /// Converts to `IBig`, moving on the `BigInt` variant.
    pub fn into_bigint(self) -> IBig {
        match self {
            Self::I64(v) => IBig::from(v),
            Self::U64(v) => IBig::from(v),
            Self::BigInt(v) => v,
        }
    }

    /// Narrows an `IBig` back to the smallest fitting variant.
    ///
    /// This keeps small results small while letting arithmetic overflow into arbitrary precision
    /// instead of panicking like fixed-width ops would.
    pub fn from_bigint(value: IBig) -> Self {
        if let Ok(v) = i64::try_from(&value) {
            Self::I64(v)
        } else if let Ok(v) = u64::try_from(&value) {
            Self::U64(v)
        } else {
            Self::BigInt(value)
        }
    }

    /// Returns `true` when `Self` == 0, `false` otherwise
    pub fn is_zero(&self) -> bool {
        match self {
            Self::I64(v) => *v == 0,
            Self::U64(v) => *v == 0,
            Self::BigInt(v) => v.is_zero(),
        }
    }

    /// Maximum shift amount accepted for `<<` / `>>` const-evaluation.
    ///
    /// Bounds the resulting `IBig` allocation (`shift` bits need roughly
    /// `shift / 8` bytes) so a crafted literal such as `1 << 2000000000`
    /// is rejected with a diagnostic instead of hanging or OOMing the compiler.
    pub const MAX_SHIFT_BITS: u32 = 1_000_000;

    /// Validates a shift amount without panicking.
    ///
    /// Returns `None` for negative amounts, amounts exceeding `u32`, and
    /// amounts above `MAX_SHIFT_BITS`.
    pub fn checked_shift_amount(&self) -> Option<u32> {
        let amount = match self {
            Self::I64(v) => u32::try_from(*v).ok()?,
            Self::U64(v) => u32::try_from(*v).ok()?,
            Self::BigInt(v) => u32::try_from(v).ok()?,
        };
        if amount > Self::MAX_SHIFT_BITS {
            return None;
        }
        Some(amount)
    }

    /// Checked `<<`. Returns `None` when the shift amount
    /// is negative, exceeds `u32`, or exceeds `MAX_SHIFT_BITS`.
    pub fn checked_shl(&self, rhs: &Self) -> Option<Self> {
        let shift = rhs.checked_shift_amount()?;
        if shift == 0 {
            return Some(self.clone());
        }
        if self.is_zero() {
            return Some(Self::I64(0));
        }
        if shift < 64 {
            match self {
                Self::I64(v) => {
                    if *v > 0 {
                        let lz = (*v as u64).leading_zeros();
                        if lz > shift {
                            return Some(Self::I64(v << shift));
                        } else if lz == shift {
                            return Some(Self::U64((*v as u64) << shift));
                        }
                    } else {
                        let res = v.wrapping_shl(shift);
                        if res >> shift == *v {
                            return Some(Self::I64(res));
                        }
                    }
                }
                Self::U64(v) => {
                    let lz = v.leading_zeros();
                    if lz > shift {
                        let res = v << shift;
                        if res <= i64::MAX as u64 {
                            return Some(Self::I64(res as i64));
                        } else {
                            return Some(Self::U64(res));
                        }
                    } else if lz == shift {
                        return Some(Self::U64(v << shift));
                    }
                }
                Self::BigInt(_) => {}
            }
        }
        match self {
            Self::BigInt(v) => Some(Self::from_bigint(v << (shift as usize))),
            _ => Some(Self::from_bigint(self.to_bigint() << (shift as usize))),
        }
    }

    /// Checked `>>`. Returns `None` when the shift amount
    /// is negative, exceeds `u32`, or exceeds `MAX_SHIFT_BITS`.
    pub fn checked_shr(&self, rhs: &Self) -> Option<Self> {
        let shift = rhs.checked_shift_amount()?;
        if shift == 0 {
            return Some(self.clone());
        }
        if self.is_zero() {
            return Some(Self::I64(0));
        }
        match self {
            Self::I64(v) => {
                if shift >= 64 {
                    if *v < 0 {
                        return Some(Self::I64(-1));
                    } else {
                        return Some(Self::I64(0));
                    }
                }
                Some(Self::I64(v >> shift))
            }
            Self::U64(v) => {
                if shift >= 64 {
                    return Some(Self::I64(0));
                }
                let res = v >> shift;
                if res <= i64::MAX as u64 {
                    Some(Self::I64(res as i64))
                } else {
                    Some(Self::U64(res))
                }
            }
            Self::BigInt(v) => Some(Self::from_bigint(v >> (shift as usize))),
        }
    }

    fn binop_ref<F, P, U>(
        &self,
        rhs: &Self,
        primitive_i64: P,
        primitive_u64: U,
        bigint_op: F,
    ) -> Self
    where
        P: FnOnce(i64, i64) -> Option<Self>,
        U: FnOnce(u64, u64) -> Option<Self>,
        F: FnOnce(&IBig, &IBig) -> IBig,
    {
        if let (Self::I64(a), Self::I64(b)) = (self, rhs) {
            if let Some(res) = primitive_i64(*a, *b) {
                return res;
            }
        }
        if let (Self::U64(a), Self::U64(b)) = (self, rhs) {
            if let Some(res) = primitive_u64(*a, *b) {
                return res;
            }
        }
        let cow_a = self.to_bigint_cow();
        let cow_b = rhs.to_bigint_cow();
        Self::from_bigint(bigint_op(cow_a.as_ref(), cow_b.as_ref()))
    }
}

impl std::fmt::Display for ArbitraryIntKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArbitraryIntKind::I64(num) => write!(f, "{num}"),
            ArbitraryIntKind::U64(num) => write!(f, "{num}"),
            ArbitraryIntKind::BigInt(ibig) => write!(f, "{ibig}"),
        }
    }
}

// May remove
const fn int_from_u64_narrow(v: u64) -> ArbitraryIntKind {
    if v <= i64::MAX as u64 {
        ArbitraryIntKind::I64(v as i64)
    } else {
        ArbitraryIntKind::U64(v)
    }
}

impl PartialEq for ArbitraryIntKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::I64(a), Self::I64(b)) => a == b,
            (Self::U64(a), Self::U64(b)) => a == b,
            (Self::BigInt(a), Self::BigInt(b)) => a == b,
            (Self::I64(a), Self::U64(b)) => *a >= 0 && (*a as u64) == *b,
            (Self::U64(a), Self::I64(b)) => *b >= 0 && *a == (*b as u64),
            (Self::BigInt(a), Self::I64(b)) | (Self::I64(b), Self::BigInt(a)) => {
                i64::try_from(a).map(|v| v == *b).unwrap_or(false)
            }
            (Self::BigInt(a), Self::U64(b)) | (Self::U64(b), Self::BigInt(a)) => {
                u64::try_from(a).map(|v| v == *b).unwrap_or(false)
            }
        }
    }
}

impl Eq for ArbitraryIntKind {}

impl PartialOrd for ArbitraryIntKind {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::I64(a), Self::I64(b)) => a.partial_cmp(b),
            (Self::U64(a), Self::U64(b)) => a.partial_cmp(b),
            (Self::BigInt(a), Self::BigInt(b)) => a.partial_cmp(b),
            (Self::I64(a), Self::U64(b)) => {
                if *a < 0 {
                    Some(std::cmp::Ordering::Less)
                } else {
                    (*a as u64).partial_cmp(b)
                }
            }
            (Self::U64(a), Self::I64(b)) => {
                if *b < 0 {
                    Some(std::cmp::Ordering::Greater)
                } else {
                    a.partial_cmp(&(*b as u64))
                }
            }
            (Self::BigInt(a), Self::I64(b)) => a.partial_cmp(&IBig::from(*b)),
            (Self::I64(a), Self::BigInt(b)) => IBig::from(*a).partial_cmp(b),
            (Self::BigInt(a), Self::U64(b)) => a.partial_cmp(&IBig::from(*b)),
            (Self::U64(a), Self::BigInt(b)) => IBig::from(*a).partial_cmp(b),
        }
    }
}

impl Ord for ArbitraryIntKind {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other)
            .expect("total ordering across all integer variants")
    }
}

/// Converts `i64` to `ArbitraryIntKind`
pub const fn int_from_i64(v: i64) -> ArbitraryIntKind {
    ArbitraryIntKind::I64(v)
}

/// Converts `u64` to `ArbitraryIntKind`
pub const fn int_from_u64(v: u64) -> ArbitraryIntKind {
    ArbitraryIntKind::U64(v)
}

/// Converts `i32` to `ArbitraryIntKind`
pub const fn int_from_i32(v: i32) -> ArbitraryIntKind {
    ArbitraryIntKind::I64(v as i64)
}

/// Converts `u32` to `ArbitraryIntKind`
pub const fn int_from_u32(v: u32) -> ArbitraryIntKind {
    ArbitraryIntKind::U64(v as u64)
}

impl From<IBig> for ArbitraryIntKind {
    fn from(v: IBig) -> Self {
        Self::from_bigint(v)
    }
}

impl std::ops::Neg for ArbitraryIntKind {
    type Output = Self;

    fn neg(self) -> Self::Output {
        (&self).neg()
    }
}

impl<'a> std::ops::Neg for &'a ArbitraryIntKind {
    type Output = ArbitraryIntKind;

    fn neg(self) -> Self::Output {
        match self {
            ArbitraryIntKind::I64(v) => {
                if *v != i64::MIN {
                    ArbitraryIntKind::I64(-*v)
                } else {
                    ArbitraryIntKind::U64(1u64 << 63)
                }
            }
            ArbitraryIntKind::U64(v) => {
                if *v == 1u64 << 63 {
                    ArbitraryIntKind::I64(i64::MIN)
                } else {
                    ArbitraryIntKind::from_bigint(-IBig::from(*v))
                }
            }
            ArbitraryIntKind::BigInt(v) => ArbitraryIntKind::from_bigint(-v),
        }
    }
}

impl std::ops::Not for ArbitraryIntKind {
    type Output = Self;

    fn not(self) -> Self::Output {
        (&self).not()
    }
}

impl<'a> std::ops::Not for &'a ArbitraryIntKind {
    type Output = ArbitraryIntKind;

    fn not(self) -> Self::Output {
        // Bitwise NOT follows two's complement sign semantics (~x = -x - 1) across all variants,
        // ensuring involution (!(!x) == x) and value equality preservation (x == y => !x == !y).
        match self {
            ArbitraryIntKind::I64(v) => ArbitraryIntKind::I64(!*v),
            ArbitraryIntKind::U64(v) => {
                if *v <= i64::MAX as u64 {
                    ArbitraryIntKind::I64(!(*v as i64))
                } else {
                    ArbitraryIntKind::from_bigint(!IBig::from(*v))
                }
            }
            ArbitraryIntKind::BigInt(v) => ArbitraryIntKind::from_bigint(!v),
        }
    }
}

// Binary int ops promote through `BigInt` and narrow back, so mixed
// `I64`/`U64`/`BigInt` operands behave like one arbitrary-precision value.
macro_rules! impl_int_binop {
    ($trait:ident, $method:ident, $prim_i64:expr, $prim_u64:expr) => {
        impl std::ops::$trait for ArbitraryIntKind {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                self.binop_ref(&rhs, $prim_i64, $prim_u64, |a, b| a.$method(b))
            }
        }

        impl<'a, 'b> std::ops::$trait<&'b ArbitraryIntKind> for &'a ArbitraryIntKind {
            type Output = ArbitraryIntKind;

            fn $method(self, rhs: &'b ArbitraryIntKind) -> Self::Output {
                self.binop_ref(rhs, $prim_i64, $prim_u64, |a, b| a.$method(b))
            }
        }

        impl<'a> std::ops::$trait<&'a ArbitraryIntKind> for ArbitraryIntKind {
            type Output = ArbitraryIntKind;

            fn $method(self, rhs: &'a ArbitraryIntKind) -> Self::Output {
                self.binop_ref(rhs, $prim_i64, $prim_u64, |a, b| a.$method(b))
            }
        }

        impl<'a> std::ops::$trait<ArbitraryIntKind> for &'a ArbitraryIntKind {
            type Output = ArbitraryIntKind;

            fn $method(self, rhs: ArbitraryIntKind) -> Self::Output {
                self.binop_ref(&rhs, $prim_i64, $prim_u64, |a, b| a.$method(b))
            }
        }
    };
}

impl_int_binop!(
    Add,
    add,
    |a, b| a.checked_add(b).map(ArbitraryIntKind::I64),
    |a, b| a.checked_add(b).map(int_from_u64_narrow)
);
impl_int_binop!(
    Sub,
    sub,
    |a, b| a.checked_sub(b).map(ArbitraryIntKind::I64),
    |a, b| a.checked_sub(b).map(int_from_u64_narrow)
);
impl_int_binop!(
    Mul,
    mul,
    |a, b| a.checked_mul(b).map(ArbitraryIntKind::I64),
    |a, b| a.checked_mul(b).map(int_from_u64_narrow)
);
impl_int_binop!(
    Div,
    div,
    |a, b| if b != 0 && !(a == i64::MIN && b == -1) {
        Some(ArbitraryIntKind::I64(a / b))
    } else {
        None
    },
    |a, b| if b != 0 {
        Some(int_from_u64_narrow(a / b))
    } else {
        None
    }
);
impl_int_binop!(
    Rem,
    rem,
    |a, b| if b != 0 && !(a == i64::MIN && b == -1) {
        Some(ArbitraryIntKind::I64(a % b))
    } else {
        None
    },
    |a, b| if b != 0 {
        Some(int_from_u64_narrow(a % b))
    } else {
        None
    }
);
impl_int_binop!(
    BitAnd,
    bitand,
    |a, b| Some(ArbitraryIntKind::I64(a & b)),
    |a, b| Some(int_from_u64_narrow(a & b))
);
impl_int_binop!(
    BitOr,
    bitor,
    |a, b| Some(ArbitraryIntKind::I64(a | b)),
    |a, b| Some(int_from_u64_narrow(a | b))
);
impl_int_binop!(
    BitXor,
    bitxor,
    |a, b| Some(ArbitraryIntKind::I64(a ^ b)),
    |a, b| Some(int_from_u64_narrow(a ^ b))
);

impl std::ops::Shl<ArbitraryIntKind> for ArbitraryIntKind {
    type Output = Self;

    fn shl(self, rhs: ArbitraryIntKind) -> Self::Output {
        // Direct operator use panics on out-of-range shifts like `Div` panics
        // on zero. Compiler paths must use `checked_shl` and emit a diagnostic.
        self.checked_shl(&rhs)
            .expect("shift amount out of range or exceeds limit")
    }
}

impl<'a, 'b> std::ops::Shl<&'b ArbitraryIntKind> for &'a ArbitraryIntKind {
    type Output = ArbitraryIntKind;

    fn shl(self, rhs: &'b ArbitraryIntKind) -> Self::Output {
        self.checked_shl(rhs)
            .expect("shift amount out of range or exceeds limit")
    }
}

impl<'a> std::ops::Shl<&'a ArbitraryIntKind> for ArbitraryIntKind {
    type Output = ArbitraryIntKind;

    fn shl(self, rhs: &'a ArbitraryIntKind) -> Self::Output {
        self.checked_shl(rhs)
            .expect("shift amount out of range or exceeds limit")
    }
}

impl<'a> std::ops::Shl<ArbitraryIntKind> for &'a ArbitraryIntKind {
    type Output = ArbitraryIntKind;

    fn shl(self, rhs: ArbitraryIntKind) -> Self::Output {
        self.checked_shl(&rhs)
            .expect("shift amount out of range or exceeds limit")
    }
}

impl std::ops::Shr<ArbitraryIntKind> for ArbitraryIntKind {
    type Output = Self;

    fn shr(self, rhs: ArbitraryIntKind) -> Self::Output {
        // Direct operator use panics on out-of-range shifts like `Div` panics
        // on zero. Compiler paths must use `checked_shr` and emit a diagnostic.
        self.checked_shr(&rhs)
            .expect("shift amount out of range or exceeds limit")
    }
}

impl<'a, 'b> std::ops::Shr<&'b ArbitraryIntKind> for &'a ArbitraryIntKind {
    type Output = ArbitraryIntKind;

    fn shr(self, rhs: &'b ArbitraryIntKind) -> Self::Output {
        self.checked_shr(rhs)
            .expect("shift amount out of range or exceeds limit")
    }
}

impl<'a> std::ops::Shr<&'a ArbitraryIntKind> for ArbitraryIntKind {
    type Output = ArbitraryIntKind;

    fn shr(self, rhs: &'a ArbitraryIntKind) -> Self::Output {
        self.checked_shr(rhs)
            .expect("shift amount out of range or exceeds limit")
    }
}

impl<'a> std::ops::Shr<ArbitraryIntKind> for &'a ArbitraryIntKind {
    type Output = ArbitraryIntKind;

    fn shr(self, rhs: ArbitraryIntKind) -> Self::Output {
        self.checked_shr(&rhs)
            .expect("shift amount out of range or exceeds limit")
    }
}
