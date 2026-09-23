use std::borrow::Cow;

use chrn_utils::id_types::TypeId;
use dashu_int::IBig;
use dashu_int::ops::BitTest;

use crate::lexer::notations::IntegerNotation;
use crate::script_compiler::compiler_consts;

/// Representable `chrn` integers.
#[derive(Debug, Clone)]
pub enum ArbitraryIntKind {
    I64(i64),
    U64(u64),
    BigInt(IBig),
}

impl ArbitraryIntKind {
    /// Returns compile-time `TypeId` for `Self`.
    pub const fn type_id(&self) -> TypeId {
        match self {
            Self::I64(_) => TypeId::new(compiler_consts::CORE_I64),
            Self::U64(_) => TypeId::new(compiler_consts::CORE_U64),
            Self::BigInt(_) => TypeId::new(compiler_consts::CORE_BIGINT),
        }
    }
    //WARN: Is this retrying ok?
    /// Parses an integer string under the specified notation.
    pub fn from_str(s: &str, notation: IntegerNotation) -> Option<Self> {
        Self::from_str_with_limit(s, notation, u64::MAX).ok()
    }

    /// Parses an integer literal respecting `max_bits`.
    pub fn from_str_with_limit(
        s: &str,
        notation: IntegerNotation,
        max_bits: u64,
    ) -> Result<Self, NumericIntParseError> {
        if let Ok(num) = i64::from_str_radix(s, notation.radix()) {
            let kind = Self::I64(num);
            return if kind.fits_numeric_bits(max_bits) {
                Ok(kind)
            } else {
                Err(NumericIntParseError::LimitExceeded)
            };
        } else if let Ok(num) = u64::from_str_radix(s, notation.radix()) {
            let kind = Self::U64(num);
            return if kind.fits_numeric_bits(max_bits) {
                Ok(kind)
            } else {
                Err(NumericIntParseError::LimitExceeded)
            };
        }

        if Self::literal_exceeds_numeric_bits(s, notation, max_bits) {
            return Err(NumericIntParseError::LimitExceeded);
        }

        match IBig::from_str_radix(s, notation.radix()) {
            Ok(v) => {
                let kind = Self::from_bigint(v);
                if kind.fits_numeric_bits(max_bits) {
                    Ok(kind)
                } else {
                    Err(NumericIntParseError::LimitExceeded)
                }
            }
            Err(_) => Err(NumericIntParseError::Invalid),
        }
    }

    /// Returns true when literal spelling proves its magnitude exceeds `max_bits`.
    pub fn literal_exceeds_numeric_bits(s: &str, notation: IntegerNotation, max_bits: u64) -> bool {
        let s = s.strip_prefix(['+', '-']).unwrap_or(s);
        let digits = s.trim_start_matches('0');
        if digits.is_empty() {
            return false;
        }

        let radix = notation.radix();
        let mut digit_count = 0u64;
        let mut first = 0u32;
        for (i, c) in digits.chars().enumerate() {
            // Defer invalid spellings to the exact `IBig` parse so they report
            // `Invalid` instead of `LimitExceeded`.
            let Some(digit) = c.to_digit(radix) else {
                return false;
            };
            if i == 0 {
                first = digit;
            }
            digit_count += 1;
        }
        let trailing_digits = digit_count - 1;
        let minimum_bits = match notation {
            IntegerNotation::Decimal => trailing_digits * 3321 / 1000 + 1,
            IntegerNotation::Bin | IntegerNotation::Octal | IntegerNotation::Hex => {
                trailing_digits * (notation.radix().ilog2() as u64) + (first.ilog2() as u64) + 1
            }
        };
        minimum_bits > max_bits
    }

    /// Magnitude bit length, excluding sign.
    pub fn numeric_bits(&self) -> u64 {
        match self {
            Self::I64(v) => (u64::BITS - v.unsigned_abs().leading_zeros()) as u64,
            Self::U64(v) => (u64::BITS - v.leading_zeros()) as u64,
            Self::BigInt(v) => v.bit_len() as u64,
        }
    }

    pub fn fits_numeric_bits(&self, max_bits: u64) -> bool {
        self.numeric_bits() <= max_bits
    }

    /// Converts to `IBig`.
    pub fn to_bigint(&self) -> IBig {
        match self {
            Self::I64(v) => IBig::from(*v),
            Self::U64(v) => IBig::from(*v),
            Self::BigInt(v) => v.clone(),
        }
    }

    /// Borrows `BigInt` or converts primitive variants to an owned `IBig`.
    pub fn to_bigint_cow(&self) -> Cow<'_, IBig> {
        match self {
            Self::I64(v) => Cow::Owned(IBig::from(*v)),
            Self::U64(v) => Cow::Owned(IBig::from(*v)),
            Self::BigInt(v) => Cow::Borrowed(v),
        }
    }

    /// Converts into `IBig`.
    pub fn into_bigint(self) -> IBig {
        match self {
            Self::I64(v) => IBig::from(v),
            Self::U64(v) => IBig::from(v),
            Self::BigInt(v) => v,
        }
    }

    /// Narrows an `IBig` to the smallest fitting variant.
    pub fn from_bigint(value: IBig) -> Self {
        if let Ok(v) = i64::try_from(&value) {
            Self::I64(v)
        } else if let Ok(v) = u64::try_from(&value) {
            Self::U64(v)
        } else {
            Self::BigInt(value)
        }
    }

    /// Returns true if zero.
    pub fn is_zero(&self) -> bool {
        match self {
            Self::I64(v) => *v == 0,
            Self::U64(v) => *v == 0,
            Self::BigInt(v) => v.is_zero(),
        }
    }

    /// Maximum shift amount ceiling.
    pub const MAX_SHIFT_BITS: u32 = 1_000_000;

    /// Validates shift amount against `MAX_SHIFT_BITS`.
    pub fn checked_shift_amount(&self) -> Option<u32> {
        let amount = self.shift_amount()?;
        (amount <= Self::MAX_SHIFT_BITS).then_some(amount)
    }

    fn shift_amount(&self) -> Option<u32> {
        Some(match self {
            Self::I64(v) => u32::try_from(*v).ok()?,
            Self::U64(v) => u32::try_from(*v).ok()?,
            Self::BigInt(v) => u32::try_from(v).ok()?,
        })
    }

    /// Checked `<<` bounded by `max_bits`.
    pub fn checked_shl_with_limit(
        &self,
        rhs: &Self,
        max_bits: u64,
    ) -> Result<Self, NumericIntError> {
        let shift = rhs
            .checked_shift_amount()
            .ok_or(NumericIntError::InvalidShift)?;
        self.shl_amount_with_limit(shift, max_bits)
    }

    fn shl_amount_with_limit(&self, shift: u32, max_bits: u64) -> Result<Self, NumericIntError> {
        if shift == 0 {
            return self
                .fits_numeric_bits(max_bits)
                .then(|| self.clone())
                .ok_or(NumericIntError::LimitExceeded);
        }
        if self.is_zero() {
            return Ok(Self::I64(0));
        }
        let shift_bits = (shift) as u64;
        if shift_bits > max_bits || self.numeric_bits() > max_bits - shift_bits {
            return Err(NumericIntError::LimitExceeded);
        }
        if shift < 64 {
            match self {
                Self::I64(v) => {
                    if *v > 0 {
                        let lz = (*v as u64).leading_zeros();
                        if lz > shift {
                            return Ok(Self::I64(v << shift));
                        } else if lz == shift {
                            return Ok(Self::U64((*v as u64) << shift));
                        }
                    } else {
                        let res = v.wrapping_shl(shift);
                        if res >> shift == *v {
                            return Ok(Self::I64(res));
                        }
                    }
                }
                Self::U64(v) => {
                    let lz = v.leading_zeros();
                    if lz > shift {
                        let res = v << shift;
                        if res <= i64::MAX as u64 {
                            return Ok(Self::I64(res as i64));
                        } else {
                            return Ok(Self::U64(res));
                        }
                    } else if lz == shift {
                        return Ok(Self::U64(v << shift));
                    }
                }
                Self::BigInt(_) => {}
            }
        }
        match self {
            Self::BigInt(v) => Ok(Self::from_bigint(v << (shift as usize))),
            _ => Ok(Self::from_bigint(self.to_bigint() << (shift as usize))),
        }
    }

    /// Checked `<<`.
    pub fn checked_shl(&self, rhs: &Self) -> Option<Self> {
        let shift = rhs.checked_shift_amount()?;
        self.shl_amount_with_limit(shift, u64::MAX).ok()
    }

    /// Checked `>>`.
    pub fn checked_shr(&self, rhs: &Self) -> Option<Self> {
        let shift = rhs.checked_shift_amount()?;
        Some(self.shr_amount(shift))
    }

    /// Checked `>>` bounded by `max_bits`.
    pub fn checked_shr_with_limit(
        &self,
        rhs: &Self,
        max_bits: u64,
    ) -> Result<Self, NumericIntError> {
        let shift = rhs
            .checked_shift_amount()
            .ok_or(NumericIntError::InvalidShift)?;
        let res = self.shr_amount(shift);
        if res.fits_numeric_bits(max_bits) {
            Ok(res)
        } else {
            Err(NumericIntError::LimitExceeded)
        }
    }

    fn shr_amount(&self, shift: u32) -> Self {
        if shift == 0 {
            return self.clone();
        }
        if self.is_zero() {
            return Self::I64(0);
        }
        match self {
            Self::I64(v) => {
                if shift >= 64 {
                    if *v < 0 {
                        return Self::I64(-1);
                    } else {
                        return Self::I64(0);
                    }
                }
                Self::I64(v >> shift)
            }
            Self::U64(v) => {
                if shift >= 64 {
                    return Self::I64(0);
                }
                let res = v >> shift;
                if res <= i64::MAX as u64 {
                    Self::I64(res as i64)
                } else {
                    Self::U64(res)
                }
            }
            Self::BigInt(v) => {
                if shift as usize >= v.bit_len() {
                    Self::I64(if v < &IBig::ZERO { -1 } else { 0 })
                } else {
                    Self::from_bigint(v >> (shift as usize))
                }
            }
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

/// Numeric integer operation errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericIntError {
    InvalidShift,
    LimitExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericIntParseError {
    Invalid,
    LimitExceeded,
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
        Some(self.cmp(other))
    }
}

impl Ord for ArbitraryIntKind {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::I64(a), Self::I64(b)) => a.cmp(b),
            (Self::U64(a), Self::U64(b)) => a.cmp(b),
            (Self::BigInt(a), Self::BigInt(b)) => a.cmp(b),
            (Self::I64(a), Self::U64(b)) => {
                if *a < 0 {
                    std::cmp::Ordering::Less
                } else {
                    (*a as u64).cmp(b)
                }
            }
            (Self::U64(a), Self::I64(b)) => {
                if *b < 0 {
                    std::cmp::Ordering::Greater
                } else {
                    a.cmp(&(*b as u64))
                }
            }
            (Self::BigInt(a), Self::I64(b)) => a.cmp(&IBig::from(*b)),
            (Self::I64(a), Self::BigInt(b)) => IBig::from(*a).cmp(b),
            (Self::BigInt(a), Self::U64(b)) => a.cmp(&IBig::from(*b)),
            (Self::U64(a), Self::BigInt(b)) => IBig::from(*a).cmp(b),
        }
    }
}

/// Converts `i64` to `ArbitraryIntKind`
pub const fn int_from_i64(v: i64) -> ArbitraryIntKind {
    ArbitraryIntKind::I64(v)
}

/// Converts `u64` to `ArbitraryIntKind`, narrowing to `I64` when the value fits.
pub const fn int_from_u64(v: u64) -> ArbitraryIntKind {
    if v <= i64::MAX as u64 {
        ArbitraryIntKind::I64(v as i64)
    } else {
        ArbitraryIntKind::U64(v)
    }
}

/// Converts `i32` to `ArbitraryIntKind`
pub const fn int_from_i32(v: i32) -> ArbitraryIntKind {
    ArbitraryIntKind::I64(v as i64)
}

/// Converts `u32` to `ArbitraryIntKind`.
pub const fn int_from_u32(v: u32) -> ArbitraryIntKind {
    ArbitraryIntKind::I64(v as i64)
}

impl From<IBig> for ArbitraryIntKind {
    fn from(v: IBig) -> Self {
        Self::from_bigint(v)
    }
}

impl std::ops::Neg for ArbitraryIntKind {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::I64(v) if v == i64::MIN => Self::U64(1u64 << 63),
            Self::I64(v) => Self::I64(-v),
            Self::U64(v) if v == 1u64 << 63 => Self::I64(i64::MIN),
            Self::U64(v) => Self::from_bigint(-IBig::from(v)),
            Self::BigInt(v) => Self::from_bigint(-v),
        }
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
        match self {
            Self::I64(v) => Self::I64(!v),
            Self::U64(v) if v <= i64::MAX as u64 => Self::I64(!(v as i64)),
            Self::U64(v) => Self::from_bigint(!IBig::from(v)),
            Self::BigInt(v) => Self::from_bigint(!v),
        }
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
    |a, b| a.checked_add(b).map(int_from_u64)
);
impl_int_binop!(
    Sub,
    sub,
    |a, b| a.checked_sub(b).map(ArbitraryIntKind::I64),
    |a, b| a.checked_sub(b).map(int_from_u64)
);
impl_int_binop!(
    Mul,
    mul,
    |a, b| a.checked_mul(b).map(ArbitraryIntKind::I64),
    |a, b| a.checked_mul(b).map(int_from_u64)
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
        Some(int_from_u64(a / b))
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
        Some(int_from_u64(a % b))
    } else {
        None
    }
);
impl_int_binop!(
    BitAnd,
    bitand,
    |a, b| Some(ArbitraryIntKind::I64(a & b)),
    |a, b| Some(int_from_u64(a & b))
);
impl_int_binop!(
    BitOr,
    bitor,
    |a, b| Some(ArbitraryIntKind::I64(a | b)),
    |a, b| Some(int_from_u64(a | b))
);
impl_int_binop!(
    BitXor,
    bitxor,
    |a, b| Some(ArbitraryIntKind::I64(a ^ b)),
    |a, b| Some(int_from_u64(a ^ b))
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
