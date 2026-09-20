use std::borrow::Cow;
use std::str::FromStr;

use chrn_utils::id_types::TypeId;
use dashu_float::DBig;
use dashu_int::IBig;

use crate::lexer::token::Notation;
use crate::script_compiler::compiler_constants;
use crate::semantic::arbitraries::{
    ArbitraryFloatKind, ArbitraryIntKind, NumericFloatParseError, NumericIntError,
    NumericIntParseError, float_from_f32, float_from_f64, int_from_i32, int_from_i64, int_from_u32,
    int_from_u64,
};

#[test]
fn arbitrary_int_from_str_radixes() {
    // Binary radix
    assert_eq!(
        ArbitraryIntKind::from_str("1010", Notation::Bin),
        Some(ArbitraryIntKind::I64(10))
    );
    assert_eq!(
        ArbitraryIntKind::from_str("0", Notation::Bin),
        Some(ArbitraryIntKind::I64(0))
    );
    assert_eq!(
        ArbitraryIntKind::from_str("-101", Notation::Bin),
        Some(ArbitraryIntKind::I64(-5))
    );
    assert_eq!(
        ArbitraryIntKind::from_str("11111111", Notation::Bin),
        Some(ArbitraryIntKind::I64(255))
    );

    // Octal radix
    assert_eq!(
        ArbitraryIntKind::from_str("755", Notation::Octal),
        Some(ArbitraryIntKind::I64(493))
    );
    assert_eq!(
        ArbitraryIntKind::from_str("0", Notation::Octal),
        Some(ArbitraryIntKind::I64(0))
    );
    assert_eq!(
        ArbitraryIntKind::from_str("-77", Notation::Octal),
        Some(ArbitraryIntKind::I64(-63))
    );
    assert_eq!(
        ArbitraryIntKind::from_str("177777", Notation::Octal),
        Some(ArbitraryIntKind::I64(65535))
    );

    // Decimal radix
    assert_eq!(
        ArbitraryIntKind::from_str("12345", Notation::Decimal),
        Some(ArbitraryIntKind::I64(12345))
    );
    assert_eq!(
        ArbitraryIntKind::from_str("-987", Notation::Decimal),
        Some(ArbitraryIntKind::I64(-987))
    );
    assert_eq!(
        ArbitraryIntKind::from_str("0", Notation::Decimal),
        Some(ArbitraryIntKind::I64(0))
    );

    // Hex radix
    assert_eq!(
        ArbitraryIntKind::from_str("1a", Notation::Hex),
        Some(ArbitraryIntKind::I64(26))
    );
    assert_eq!(
        ArbitraryIntKind::from_str("1A", Notation::Hex),
        Some(ArbitraryIntKind::I64(26))
    );
    assert_eq!(
        ArbitraryIntKind::from_str("deadbeef", Notation::Hex),
        Some(ArbitraryIntKind::I64(3735928559))
    );
    assert_eq!(
        ArbitraryIntKind::from_str("DEADBEEF", Notation::Hex),
        Some(ArbitraryIntKind::I64(3735928559))
    );
    assert_eq!(
        ArbitraryIntKind::from_str("-ff", Notation::Hex),
        Some(ArbitraryIntKind::I64(-255))
    );
}

#[test]
fn arbitrary_int_from_str_tier_transitions() {
    // i64 boundaries
    assert!(matches!(
        ArbitraryIntKind::from_str("9223372036854775807", Notation::Decimal),
        Some(ArbitraryIntKind::I64(v)) if v == i64::MAX
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("-9223372036854775808", Notation::Decimal),
        Some(ArbitraryIntKind::I64(v)) if v == i64::MIN
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("7fffffffffffffff", Notation::Hex),
        Some(ArbitraryIntKind::I64(v)) if v == i64::MAX
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("777777777777777777777", Notation::Octal),
        Some(ArbitraryIntKind::I64(v)) if v == i64::MAX
    ));

    // Transition to U64 for [i64::MAX + 1, u64::MAX]
    assert!(matches!(
        ArbitraryIntKind::from_str("9223372036854775808", Notation::Decimal),
        Some(ArbitraryIntKind::U64(v)) if v == 1u64 << 63
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("8000000000000000", Notation::Hex),
        Some(ArbitraryIntKind::U64(v)) if v == 1u64 << 63
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str(
            "1000000000000000000000000000000000000000000000000000000000000000",
            Notation::Bin
        ),
        Some(ArbitraryIntKind::U64(v)) if v == 1u64 << 63
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("1000000000000000000000", Notation::Octal),
        Some(ArbitraryIntKind::U64(v)) if v == 1u64 << 63
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("18446744073709551615", Notation::Decimal),
        Some(ArbitraryIntKind::U64(v)) if v == u64::MAX
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("ffffffffffffffff", Notation::Hex),
        Some(ArbitraryIntKind::U64(v)) if v == u64::MAX
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("1777777777777777777777", Notation::Octal),
        Some(ArbitraryIntKind::U64(v)) if v == u64::MAX
    ));

    // Transition to BigInt for values > u64::MAX
    let over_u64 = IBig::from(u64::MAX) + IBig::from(1);
    assert!(matches!(
        ArbitraryIntKind::from_str("18446744073709551616", Notation::Decimal),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == over_u64
    ));
    let pow64 = IBig::from(1u64) << 64;
    assert!(matches!(
        ArbitraryIntKind::from_str("10000000000000000", Notation::Hex),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == pow64
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str(
            "10000000000000000000000000000000000000000000000000000000000000000",
            Notation::Bin
        ),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == pow64
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("2000000000000000000000", Notation::Octal),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == pow64
    ));

    // Transition to BigInt for values < i64::MIN
    let under_i64 = IBig::from(i64::MIN) - IBig::from(1);
    assert!(matches!(
        ArbitraryIntKind::from_str("-9223372036854775809", Notation::Decimal),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == under_i64
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("-8000000000000001", Notation::Hex),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == under_i64
    ));
    // Explicit leading `+` normalizes properly across tiers
    assert!(matches!(
        ArbitraryIntKind::from_str("+0", Notation::Decimal),
        Some(ArbitraryIntKind::I64(0))
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("+42", Notation::Decimal),
        Some(ArbitraryIntKind::I64(42))
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("+9223372036854775807", Notation::Decimal),
        Some(ArbitraryIntKind::I64(v)) if v == i64::MAX
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("+9223372036854775808", Notation::Decimal),
        Some(ArbitraryIntKind::U64(v)) if v == 1u64 << 63
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("+18446744073709551615", Notation::Decimal),
        Some(ArbitraryIntKind::U64(v)) if v == u64::MAX
    ));
    assert!(matches!(
        ArbitraryIntKind::from_str("+18446744073709551616", Notation::Decimal),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == over_u64
    ));
}

#[test]
fn arbitrary_int_from_str_invalid_and_empty() {
    // Empty string returns None for all notations
    assert_eq!(ArbitraryIntKind::from_str("", Notation::Bin), None);
    assert_eq!(ArbitraryIntKind::from_str("", Notation::Octal), None);
    assert_eq!(ArbitraryIntKind::from_str("", Notation::Decimal), None);
    assert_eq!(ArbitraryIntKind::from_str("", Notation::Hex), None);

    // Invalid digits for radix
    assert_eq!(ArbitraryIntKind::from_str("102", Notation::Bin), None);
    assert_eq!(ArbitraryIntKind::from_str("2", Notation::Bin), None);
    assert_eq!(ArbitraryIntKind::from_str("1a", Notation::Bin), None);

    assert_eq!(ArbitraryIntKind::from_str("89", Notation::Octal), None);
    assert_eq!(ArbitraryIntKind::from_str("78", Notation::Octal), None);
    assert_eq!(ArbitraryIntKind::from_str("8", Notation::Octal), None);

    assert_eq!(ArbitraryIntKind::from_str("12a3", Notation::Decimal), None);
    assert_eq!(ArbitraryIntKind::from_str("f", Notation::Decimal), None);
    assert_eq!(ArbitraryIntKind::from_str("1.0", Notation::Decimal), None);

    assert_eq!(ArbitraryIntKind::from_str("12g", Notation::Hex), None);
    assert_eq!(ArbitraryIntKind::from_str("xyz", Notation::Hex), None);
    assert_eq!(ArbitraryIntKind::from_str("0x10", Notation::Hex), None);

    // Isolated signs or symbols
    assert_eq!(ArbitraryIntKind::from_str("+", Notation::Decimal), None);
    assert_eq!(ArbitraryIntKind::from_str("-", Notation::Decimal), None);
    assert_eq!(ArbitraryIntKind::from_str("?", Notation::Decimal), None);
}

#[test]
fn arbitrary_int_from_bigint_normalization() {
    // [-2^63, 2^63 - 1] normalize to I64
    assert!(matches!(
        ArbitraryIntKind::from_bigint(IBig::from(i64::MIN)),
        ArbitraryIntKind::I64(v) if v == i64::MIN
    ));
    assert!(matches!(
        ArbitraryIntKind::from_bigint(IBig::from(i64::MAX)),
        ArbitraryIntKind::I64(v) if v == i64::MAX
    ));
    assert!(matches!(
        ArbitraryIntKind::from_bigint(IBig::from(0)),
        ArbitraryIntKind::I64(0)
    ));
    assert!(matches!(
        ArbitraryIntKind::from_bigint(IBig::from(-1)),
        ArbitraryIntKind::I64(-1)
    ));
    assert!(matches!(
        ArbitraryIntKind::from_bigint(IBig::from(1)),
        ArbitraryIntKind::I64(1)
    ));

    // [2^63, 2^64 - 1] normalize to U64
    assert!(matches!(
        ArbitraryIntKind::from_bigint(IBig::from(1u64 << 63)),
        ArbitraryIntKind::U64(v) if v == 1u64 << 63
    ));
    assert!(matches!(
        ArbitraryIntKind::from_bigint(IBig::from(u64::MAX)),
        ArbitraryIntKind::U64(v) if v == u64::MAX
    ));
    assert!(matches!(
        ArbitraryIntKind::from_bigint(IBig::from(1u64 << 63) + IBig::from(42)),
        ArbitraryIntKind::U64(v) if v == (1u64 << 63) + 42
    ));

    // < -2^63 or > 2^64 - 1 normalize to BigInt
    let under_min = IBig::from(i64::MIN) - IBig::from(1);
    assert!(matches!(
        ArbitraryIntKind::from_bigint(under_min.clone()),
        ArbitraryIntKind::BigInt(ref v) if *v == under_min
    ));

    let over_max = IBig::from(u64::MAX) + IBig::from(1);
    assert!(matches!(
        ArbitraryIntKind::from_bigint(over_max.clone()),
        ArbitraryIntKind::BigInt(ref v) if *v == over_max
    ));

    let huge_pos = IBig::from(1) << 100;
    assert!(matches!(
        ArbitraryIntKind::from_bigint(huge_pos.clone()),
        ArbitraryIntKind::BigInt(ref v) if *v == huge_pos
    ));

    let huge_neg = -(IBig::from(1) << 100);
    assert!(matches!(
        ArbitraryIntKind::from_bigint(huge_neg.clone()),
        ArbitraryIntKind::BigInt(ref v) if *v == huge_neg
    ));

    // From<IBig> trait calls from_bigint
    let from_trait: ArbitraryIntKind = IBig::from(42).into();
    assert!(matches!(from_trait, ArbitraryIntKind::I64(42)));
    let from_trait_u64: ArbitraryIntKind = IBig::from(1u64 << 63).into();
    assert!(matches!(from_trait_u64, ArbitraryIntKind::U64(v) if v == 1u64 << 63));
}

#[test]
fn arbitrary_int_checked_shift_amount_bounds() {
    // Valid shift amounts across variants
    assert_eq!(ArbitraryIntKind::I64(0).checked_shift_amount(), Some(0));
    assert_eq!(ArbitraryIntKind::I64(42).checked_shift_amount(), Some(42));
    assert_eq!(
        ArbitraryIntKind::I64(ArbitraryIntKind::MAX_SHIFT_BITS as i64).checked_shift_amount(),
        Some(1_000_000)
    );
    assert_eq!(ArbitraryIntKind::U64(0).checked_shift_amount(), Some(0));
    assert_eq!(
        ArbitraryIntKind::U64(ArbitraryIntKind::MAX_SHIFT_BITS as u64).checked_shift_amount(),
        Some(1_000_000)
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(500)).checked_shift_amount(),
        Some(500)
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(ArbitraryIntKind::MAX_SHIFT_BITS))
            .checked_shift_amount(),
        Some(1_000_000)
    );

    // Negative amounts rejected
    assert_eq!(ArbitraryIntKind::I64(-1).checked_shift_amount(), None);
    assert_eq!(ArbitraryIntKind::I64(i64::MIN).checked_shift_amount(), None);
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(-1)).checked_shift_amount(),
        None
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(-1_000_000)).checked_shift_amount(),
        None
    );

    // Amounts exceeding MAX_SHIFT_BITS rejected
    assert_eq!(
        ArbitraryIntKind::I64((ArbitraryIntKind::MAX_SHIFT_BITS + 1) as i64).checked_shift_amount(),
        None
    );
    assert_eq!(
        ArbitraryIntKind::U64((ArbitraryIntKind::MAX_SHIFT_BITS + 1) as u64).checked_shift_amount(),
        None
    );
    assert_eq!(ArbitraryIntKind::U64(u64::MAX).checked_shift_amount(), None);
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(ArbitraryIntKind::MAX_SHIFT_BITS + 1))
            .checked_shift_amount(),
        None
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(1) << 64).checked_shift_amount(),
        None
    );
}

#[test]
fn arbitrary_int_checked_shl_semantics() {
    let sh0 = ArbitraryIntKind::I64(0);
    let sh1 = ArbitraryIntKind::I64(1);
    let sh62 = ArbitraryIntKind::I64(62);
    let sh63 = ArbitraryIntKind::I64(63);
    let sh64 = ArbitraryIntKind::I64(64);
    let sh_neg = ArbitraryIntKind::I64(-1);
    let sh_overflow = ArbitraryIntKind::I64(1_000_001);

    // Invalid rhs shift amount returns None
    assert_eq!(ArbitraryIntKind::I64(1).checked_shl(&sh_neg), None);
    assert_eq!(ArbitraryIntKind::I64(1).checked_shl(&sh_overflow), None);
    assert_eq!(
        ArbitraryIntKind::U64(1).checked_shl(&ArbitraryIntKind::U64(u64::MAX)),
        None
    );

    // Shift 0 returns clone
    assert_eq!(
        ArbitraryIntKind::I64(42).checked_shl(&sh0),
        Some(ArbitraryIntKind::I64(42))
    );
    assert_eq!(
        ArbitraryIntKind::I64(-99).checked_shl(&sh0),
        Some(ArbitraryIntKind::I64(-99))
    );
    assert_eq!(
        ArbitraryIntKind::U64(1u64 << 63).checked_shl(&sh0),
        Some(ArbitraryIntKind::U64(1u64 << 63))
    );
    let big = ArbitraryIntKind::BigInt(IBig::from(1) << 100);
    assert_eq!(big.checked_shl(&sh0), Some(big.clone()));

    // Zero operand returns I64(0)
    assert_eq!(
        ArbitraryIntKind::I64(0).checked_shl(&ArbitraryIntKind::I64(10)),
        Some(ArbitraryIntKind::I64(0))
    );
    assert_eq!(
        ArbitraryIntKind::U64(0).checked_shl(&ArbitraryIntKind::I64(50000)),
        Some(ArbitraryIntKind::I64(0))
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(0)).checked_shl(&ArbitraryIntKind::I64(100)),
        Some(ArbitraryIntKind::I64(0))
    );

    // Positive I64:
    // - within i64 stays I64
    assert_eq!(
        ArbitraryIntKind::I64(1).checked_shl(&sh62),
        Some(ArbitraryIntKind::I64(1 << 62))
    );
    assert_eq!(
        ArbitraryIntKind::I64(5).checked_shl(&ArbitraryIntKind::I64(3)),
        Some(ArbitraryIntKind::I64(40))
    );
    // - landing on bit 63 transitions to U64(1 << 63)
    assert!(matches!(
        ArbitraryIntKind::I64(1).checked_shl(&sh63),
        Some(ArbitraryIntKind::U64(v)) if v == 1u64 << 63
    ));
    assert!(matches!(
        ArbitraryIntKind::I64(3).checked_shl(&sh62),
        Some(ArbitraryIntKind::U64(v)) if v == 3u64 << 62
    ));
    // - exceeding 63 bits promotes to BigInt
    let pow64_i = IBig::from(1u64) << 64;
    assert!(matches!(
        ArbitraryIntKind::I64(1).checked_shl(&sh64),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == pow64_i
    ));
    let pow63_3 = IBig::from(3u64) << 63;
    assert!(matches!(
        ArbitraryIntKind::I64(3).checked_shl(&sh63),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == pow63_3
    ));

    // Negative I64:
    // - -1 << 1 == -2
    assert_eq!(
        ArbitraryIntKind::I64(-1).checked_shl(&sh1),
        Some(ArbitraryIntKind::I64(-2))
    );
    // - -1 << 63 == i64::MIN
    assert!(matches!(
        ArbitraryIntKind::I64(-1).checked_shl(&sh63),
        Some(ArbitraryIntKind::I64(v)) if v == i64::MIN
    ));
    // - overflow promotes to BigInt
    let neg_pow63_2 = IBig::from(-2) << 63;
    assert!(matches!(
        ArbitraryIntKind::I64(-2).checked_shl(&sh63),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == neg_pow63_2
    ));
    let neg_pow64_1 = IBig::from(-1) << 64;
    assert!(matches!(
        ArbitraryIntKind::I64(-1).checked_shl(&sh64),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == neg_pow64_1
    ));

    // U64:
    // - shifts within u64: narrows to I64 if result fits
    assert!(matches!(
        ArbitraryIntKind::U64(5).checked_shl(&ArbitraryIntKind::I64(2)),
        Some(ArbitraryIntKind::I64(20))
    ));
    assert!(matches!(
        ArbitraryIntKind::U64(1).checked_shl(&sh62),
        Some(ArbitraryIntKind::I64(v)) if v == 1 << 62
    ));
    assert!(matches!(
        ArbitraryIntKind::U64(3).checked_shl(&ArbitraryIntKind::I64(61)),
        Some(ArbitraryIntKind::I64(v)) if v == (3i64 << 61)
    ));
    // - shifts within u64: stays U64 when landing on or above bit 63 (lz == shift)
    assert!(matches!(
        ArbitraryIntKind::U64(1).checked_shl(&sh63),
        Some(ArbitraryIntKind::U64(v)) if v == 1u64 << 63
    ));
    assert!(matches!(
        ArbitraryIntKind::U64(3).checked_shl(&sh62),
        Some(ArbitraryIntKind::U64(v)) if v == 3u64 << 62
    ));
    // - promotes to BigInt
    let pow64_u = IBig::from(1u64) << 64;
    assert!(matches!(
        ArbitraryIntKind::U64(1u64 << 63).checked_shl(&sh1),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == pow64_u
    ));
    let over_max_sh = IBig::from(u64::MAX) << 1;
    assert!(matches!(
        ArbitraryIntKind::U64(u64::MAX).checked_shl(&sh1),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == over_max_sh
    ));

    // shift >= 64: non-zero promotes to BigInt
    assert_eq!(
        ArbitraryIntKind::I64(5).checked_shl(&sh64),
        Some(ArbitraryIntKind::BigInt(IBig::from(5) << 64))
    );
    assert_eq!(
        ArbitraryIntKind::U64(5).checked_shl(&sh64),
        Some(ArbitraryIntKind::BigInt(IBig::from(5) << 64))
    );

    // Operator << overloads work and forward correctly
    assert_eq!(
        ArbitraryIntKind::I64(1) << ArbitraryIntKind::I64(4),
        ArbitraryIntKind::I64(16)
    );
    assert_eq!(
        &ArbitraryIntKind::I64(1) << &ArbitraryIntKind::I64(4),
        ArbitraryIntKind::I64(16)
    );
}

#[test]
fn arbitrary_int_checked_shr_semantics() {
    let sh0 = ArbitraryIntKind::I64(0);
    let sh1 = ArbitraryIntKind::I64(1);
    let sh2 = ArbitraryIntKind::I64(2);
    let sh63 = ArbitraryIntKind::I64(63);
    let sh64 = ArbitraryIntKind::I64(64);
    let sh1000 = ArbitraryIntKind::I64(1000);
    let sh_neg = ArbitraryIntKind::I64(-1);
    let sh_overflow = ArbitraryIntKind::I64(1_000_001);

    // Invalid rhs shift amount returns None
    assert_eq!(ArbitraryIntKind::I64(10).checked_shr(&sh_neg), None);
    assert_eq!(ArbitraryIntKind::I64(10).checked_shr(&sh_overflow), None);

    // Shift 0 returns clone
    assert_eq!(
        ArbitraryIntKind::I64(42).checked_shr(&sh0),
        Some(ArbitraryIntKind::I64(42))
    );
    assert_eq!(
        ArbitraryIntKind::U64(1u64 << 63).checked_shr(&sh0),
        Some(ArbitraryIntKind::U64(1u64 << 63))
    );
    let big = ArbitraryIntKind::BigInt(IBig::from(1) << 100);
    assert_eq!(big.checked_shr(&sh0), Some(big.clone()));

    // Zero operand returns I64(0)
    assert_eq!(
        ArbitraryIntKind::I64(0).checked_shr(&ArbitraryIntKind::I64(5)),
        Some(ArbitraryIntKind::I64(0))
    );
    assert_eq!(
        ArbitraryIntKind::U64(0).checked_shr(&ArbitraryIntKind::I64(100)),
        Some(ArbitraryIntKind::I64(0))
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(0)).checked_shr(&ArbitraryIntKind::I64(500)),
        Some(ArbitraryIntKind::I64(0))
    );

    // Positive I64:
    // - in range
    assert_eq!(
        ArbitraryIntKind::I64(16).checked_shr(&sh2),
        Some(ArbitraryIntKind::I64(4))
    );
    assert_eq!(
        ArbitraryIntKind::I64(17).checked_shr(&sh2),
        Some(ArbitraryIntKind::I64(4))
    );
    // - shift >= 64 returns I64(0)
    assert_eq!(
        ArbitraryIntKind::I64(100).checked_shr(&sh64),
        Some(ArbitraryIntKind::I64(0))
    );
    assert_eq!(
        ArbitraryIntKind::I64(100).checked_shr(&sh1000),
        Some(ArbitraryIntKind::I64(0))
    );
    assert_eq!(
        ArbitraryIntKind::I64(i64::MAX).checked_shr(&sh64),
        Some(ArbitraryIntKind::I64(0))
    );

    // Negative I64: arithmetic right shift
    // - -1 >> any == -1
    assert_eq!(
        ArbitraryIntKind::I64(-1).checked_shr(&sh1),
        Some(ArbitraryIntKind::I64(-1))
    );
    assert_eq!(
        ArbitraryIntKind::I64(-1).checked_shr(&sh63),
        Some(ArbitraryIntKind::I64(-1))
    );
    assert_eq!(
        ArbitraryIntKind::I64(-1).checked_shr(&sh64),
        Some(ArbitraryIntKind::I64(-1))
    );
    assert_eq!(
        ArbitraryIntKind::I64(-1).checked_shr(&sh1000),
        Some(ArbitraryIntKind::I64(-1))
    );
    // - -5 >> 1 == -3
    assert_eq!(
        ArbitraryIntKind::I64(-5).checked_shr(&sh1),
        Some(ArbitraryIntKind::I64(-3))
    );
    // - -5 >> 64 == -1
    assert_eq!(
        ArbitraryIntKind::I64(-5).checked_shr(&sh64),
        Some(ArbitraryIntKind::I64(-1))
    );
    // - i64::MIN >> 1
    assert_eq!(
        ArbitraryIntKind::I64(i64::MIN).checked_shr(&sh1),
        Some(ArbitraryIntKind::I64(i64::MIN >> 1))
    );
    assert_eq!(
        ArbitraryIntKind::I64(i64::MIN).checked_shr(&sh64),
        Some(ArbitraryIntKind::I64(-1))
    );

    // U64:
    // - shift >= 64 returns I64(0)
    assert_eq!(
        ArbitraryIntKind::U64(u64::MAX).checked_shr(&sh64),
        Some(ArbitraryIntKind::I64(0))
    );
    assert_eq!(
        ArbitraryIntKind::U64(1u64 << 63).checked_shr(&sh1000),
        Some(ArbitraryIntKind::I64(0))
    );
    // - narrows to I64 when shifted value <= i64::MAX
    assert!(matches!(
        ArbitraryIntKind::U64(1u64 << 63).checked_shr(&sh1),
        Some(ArbitraryIntKind::I64(v)) if v == 1 << 62
    ));
    assert!(matches!(
        ArbitraryIntKind::U64(u64::MAX).checked_shr(&sh1),
        Some(ArbitraryIntKind::I64(v)) if v == i64::MAX
    ));
    assert!(matches!(
        ArbitraryIntKind::U64(100).checked_shr(&sh2),
        Some(ArbitraryIntKind::I64(25))
    ));

    // BigInt: arithmetic flooring right shift, narrowing when magnitude fits
    let big_pow70 = ArbitraryIntKind::BigInt(IBig::from(1) << 70);
    // Shift 8 bits leaves 1 << 62, which fits in I64
    assert!(matches!(
        big_pow70.checked_shr(&ArbitraryIntKind::I64(8)),
        Some(ArbitraryIntKind::I64(v)) if v == 1 << 62
    ));
    // Shift 7 bits leaves 1 << 63, which fits in U64
    assert!(matches!(
        big_pow70.checked_shr(&ArbitraryIntKind::I64(7)),
        Some(ArbitraryIntKind::U64(v)) if v == 1u64 << 63
    ));
    // Shift 1 bit leaves 1 << 69, which stays BigInt
    let big_pow69 = IBig::from(1) << 69;
    assert!(matches!(
        big_pow70.checked_shr(&sh1),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == big_pow69
    ));

    // Negative BigInt arithmetic floor and narrowing
    let neg_big_pow70 = ArbitraryIntKind::BigInt(-(IBig::from(1) << 70));
    assert!(matches!(
        neg_big_pow70.checked_shr(&ArbitraryIntKind::I64(7)),
        Some(ArbitraryIntKind::I64(v)) if v == i64::MIN
    ));
    assert!(matches!(
        neg_big_pow70.checked_shr(&ArbitraryIntKind::I64(8)),
        Some(ArbitraryIntKind::I64(v)) if v == -(1 << 62)
    ));
    let neg_big_pow69 = -(IBig::from(1) << 69);
    assert!(matches!(
        neg_big_pow70.checked_shr(&sh1),
        Some(ArbitraryIntKind::BigInt(ref v)) if *v == neg_big_pow69
    ));

    // Negative BigInt flooring division (-5 >> 1 == -3)
    assert!(matches!(
        ArbitraryIntKind::BigInt(IBig::from(-5)).checked_shr(&sh1),
        Some(ArbitraryIntKind::I64(-3))
    ));

    // Operator >> overloads work and forward correctly
    assert_eq!(
        ArbitraryIntKind::I64(16) >> ArbitraryIntKind::I64(2),
        ArbitraryIntKind::I64(4)
    );
    assert_eq!(
        &ArbitraryIntKind::I64(16) >> &ArbitraryIntKind::I64(2),
        ArbitraryIntKind::I64(4)
    );
}

#[test]
fn arbitrary_int_checked_shr_with_limit_enforces_max_bits() {
    let big = ArbitraryIntKind::BigInt(IBig::from(1) << 100);
    // Shrinking by one still leaves 100 bits, over a 64-bit limit.
    assert_eq!(
        big.checked_shr_with_limit(&ArbitraryIntKind::I64(1), 64),
        Err(NumericIntError::LimitExceeded)
    );
    // A zero shift on an over-limit value reports `LimitExceeded`, mirroring `shl`.
    assert_eq!(
        big.checked_shr_with_limit(&ArbitraryIntKind::I64(0), 64),
        Err(NumericIntError::LimitExceeded)
    );
    // In-limit shifts still succeed.
    assert_eq!(
        ArbitraryIntKind::I64(16).checked_shr_with_limit(&ArbitraryIntKind::I64(2), 64),
        Ok(ArbitraryIntKind::I64(4))
    );
    // Collapsing a large valid shift stays `Ok` since the result fits.
    assert_eq!(
        big.checked_shr_with_limit(&ArbitraryIntKind::I64(1000), 64),
        Ok(ArbitraryIntKind::I64(0))
    );
    // Negative amounts still report `InvalidShift`, not `LimitExceeded`.
    assert_eq!(
        ArbitraryIntKind::I64(16).checked_shr_with_limit(&ArbitraryIntKind::I64(-1), 64),
        Err(NumericIntError::InvalidShift)
    );
}

#[test]
fn arbitrary_int_negation_and_inversion() {
    // -I64(i64::MIN) cleanly yields U64(1 << 63) without overflow panic
    assert!(matches!(
        -ArbitraryIntKind::I64(i64::MIN),
        ArbitraryIntKind::U64(v) if v == 1u64 << 63
    ));
    assert!(matches!(
        -&ArbitraryIntKind::I64(i64::MIN),
        ArbitraryIntKind::U64(v) if v == 1u64 << 63
    ));

    // -U64(1 << 63) yields I64(i64::MIN)
    assert!(matches!(
        -ArbitraryIntKind::U64(1u64 << 63),
        ArbitraryIntKind::I64(v) if v == i64::MIN
    ));
    assert!(matches!(
        -&ArbitraryIntKind::U64(1u64 << 63),
        ArbitraryIntKind::I64(v) if v == i64::MIN
    ));

    // -U64(v < 1 << 63) yields I64(-v)
    assert!(matches!(
        -ArbitraryIntKind::U64(42),
        ArbitraryIntKind::I64(-42)
    ));

    // -U64(v > 1 << 63) yields BigInt(-v) and negates back
    let v1 = (1u64 << 63) + 1;
    let neg1 = -ArbitraryIntKind::U64(v1);
    assert!(matches!(neg1, ArbitraryIntKind::BigInt(ref v) if *v == -IBig::from(v1)));
    assert!(matches!(-neg1, ArbitraryIntKind::U64(v) if v == v1));

    let max_u = u64::MAX;
    let neg_max = -ArbitraryIntKind::U64(max_u);
    assert!(matches!(neg_max, ArbitraryIntKind::BigInt(ref v) if *v == -IBig::from(max_u)));
    assert!(matches!(-neg_max, ArbitraryIntKind::U64(v) if v == max_u));

    // Standard I64 negation
    assert_eq!(-ArbitraryIntKind::I64(42), ArbitraryIntKind::I64(-42));
    assert_eq!(-ArbitraryIntKind::I64(-42), ArbitraryIntKind::I64(42));
    assert_eq!(-ArbitraryIntKind::I64(0), ArbitraryIntKind::I64(0));

    // Bitwise NOT !
    // !I64
    assert_eq!(!ArbitraryIntKind::I64(0), ArbitraryIntKind::I64(-1));
    assert_eq!(!ArbitraryIntKind::I64(-1), ArbitraryIntKind::I64(0));
    assert_eq!(!ArbitraryIntKind::I64(42), ArbitraryIntKind::I64(!42));
    assert_eq!(
        !ArbitraryIntKind::I64(i64::MIN),
        ArbitraryIntKind::I64(i64::MAX)
    );
    assert_eq!(!&ArbitraryIntKind::I64(0), ArbitraryIntKind::I64(-1));

    // !U64 (follows two's complement sign semantics ~x = -x - 1)
    assert_eq!(!ArbitraryIntKind::U64(0), ArbitraryIntKind::I64(-1));
    assert_eq!(!ArbitraryIntKind::U64(42), ArbitraryIntKind::I64(-43));
    assert_eq!(
        !ArbitraryIntKind::U64(u64::MAX),
        ArbitraryIntKind::BigInt(!IBig::from(u64::MAX))
    );
    assert_eq!(
        !ArbitraryIntKind::U64(1u64 << 63),
        ArbitraryIntKind::BigInt(!IBig::from(1u64 << 63))
    );
    assert_eq!(!&ArbitraryIntKind::U64(0), ArbitraryIntKind::I64(-1));

    // Involution: !(!x) == x
    assert_eq!(!(!ArbitraryIntKind::U64(0)), ArbitraryIntKind::I64(0));
    assert_eq!(
        !(!ArbitraryIntKind::U64(u64::MAX)),
        ArbitraryIntKind::U64(u64::MAX)
    );
    assert_eq!(
        !(!ArbitraryIntKind::U64(1u64 << 63)),
        ArbitraryIntKind::U64(1u64 << 63)
    );

    // Equality preservation: x == y => !x == !y
    assert_eq!(ArbitraryIntKind::U64(0), ArbitraryIntKind::I64(0));
    assert_eq!(!ArbitraryIntKind::U64(0), !ArbitraryIntKind::I64(0));
    assert_eq!(ArbitraryIntKind::U64(42), ArbitraryIntKind::I64(42));
    assert_eq!(!ArbitraryIntKind::U64(42), !ArbitraryIntKind::I64(42));

    // !BigInt
    assert_eq!(
        !ArbitraryIntKind::BigInt(IBig::from(0)),
        ArbitraryIntKind::I64(-1)
    );
    assert_eq!(
        !ArbitraryIntKind::BigInt(IBig::from(-1)),
        ArbitraryIntKind::I64(0)
    );
    let huge = IBig::from(1) << 100;
    assert_eq!(
        !ArbitraryIntKind::BigInt(huge.clone()),
        ArbitraryIntKind::BigInt(!huge)
    );
}

#[test]
fn arbitrary_int_equality_and_ordering() {
    // Cross-tier (I64, U64)
    assert_eq!(ArbitraryIntKind::I64(10), ArbitraryIntKind::U64(10));
    assert_eq!(ArbitraryIntKind::U64(10), ArbitraryIntKind::I64(10));
    assert_eq!(ArbitraryIntKind::I64(0), ArbitraryIntKind::U64(0));
    assert_eq!(
        ArbitraryIntKind::I64(i64::MAX),
        ArbitraryIntKind::U64(i64::MAX as u64)
    );
    assert_ne!(ArbitraryIntKind::I64(-1), ArbitraryIntKind::U64(u64::MAX));
    assert_ne!(ArbitraryIntKind::I64(-1), ArbitraryIntKind::U64(1));
    assert!(ArbitraryIntKind::I64(-1) < ArbitraryIntKind::U64(0));
    assert!(ArbitraryIntKind::U64(0) > ArbitraryIntKind::I64(-1));
    assert!(ArbitraryIntKind::I64(-100) < ArbitraryIntKind::U64(5));
    assert!(ArbitraryIntKind::I64(5) < ArbitraryIntKind::U64(10));
    assert!(ArbitraryIntKind::I64(20) > ArbitraryIntKind::U64(10));

    // Cross-tier (I64, BigInt)
    assert_eq!(
        ArbitraryIntKind::I64(10),
        ArbitraryIntKind::BigInt(IBig::from(10))
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(10)),
        ArbitraryIntKind::I64(10)
    );
    assert_eq!(
        ArbitraryIntKind::I64(-50),
        ArbitraryIntKind::BigInt(IBig::from(-50))
    );
    assert_ne!(
        ArbitraryIntKind::I64(10),
        ArbitraryIntKind::BigInt(IBig::from(11))
    );
    assert!(ArbitraryIntKind::I64(-5) < ArbitraryIntKind::BigInt(IBig::from(0)));
    assert!(ArbitraryIntKind::I64(100) < ArbitraryIntKind::BigInt(IBig::from(1) << 70));
    assert!(ArbitraryIntKind::I64(100) > ArbitraryIntKind::BigInt(-(IBig::from(1) << 70)));
    assert!(ArbitraryIntKind::I64(-100) > ArbitraryIntKind::BigInt(-(IBig::from(1) << 70)));

    // Cross-tier (U64, BigInt)
    assert_eq!(
        ArbitraryIntKind::U64(10),
        ArbitraryIntKind::BigInt(IBig::from(10))
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(10)),
        ArbitraryIntKind::U64(10)
    );
    assert_eq!(
        ArbitraryIntKind::U64(u64::MAX),
        ArbitraryIntKind::BigInt(IBig::from(u64::MAX))
    );
    assert!(ArbitraryIntKind::U64(0) > ArbitraryIntKind::BigInt(IBig::from(-1)));
    assert!(
        ArbitraryIntKind::U64(u64::MAX)
            < ArbitraryIntKind::BigInt(IBig::from(u64::MAX) + IBig::from(1))
    );
    assert!(ArbitraryIntKind::U64(10) < ArbitraryIntKind::BigInt(IBig::from(20)));

    // Negative I64 is always < U64 and positive BigInt
    assert!(ArbitraryIntKind::I64(-1000) < ArbitraryIntKind::U64(0));
    assert!(ArbitraryIntKind::I64(-1) < ArbitraryIntKind::BigInt(IBig::from(1)));

    // Transitivity check across all three tiers
    let ten_i64 = ArbitraryIntKind::I64(10);
    let ten_u64 = ArbitraryIntKind::U64(10);
    let ten_big = ArbitraryIntKind::BigInt(IBig::from(10));
    assert_eq!(ten_i64, ten_u64);
    assert_eq!(ten_u64, ten_big);
    assert_eq!(ten_i64, ten_big);
    assert_eq!(ten_i64.cmp(&ten_u64), std::cmp::Ordering::Equal);
    assert_eq!(ten_u64.cmp(&ten_i64), std::cmp::Ordering::Equal);
    assert_eq!(ten_u64.cmp(&ten_big), std::cmp::Ordering::Equal);
    assert_eq!(ten_big.cmp(&ten_u64), std::cmp::Ordering::Equal);
    assert_eq!(ten_big.cmp(&ten_i64), std::cmp::Ordering::Equal);
    assert_eq!(ten_i64.cmp(&ten_big), std::cmp::Ordering::Equal);
}

#[test]
fn arbitrary_int_binary_arithmetic_and_overflow() {
    // Add:
    // Fast path on (I64, I64)
    assert_eq!(
        ArbitraryIntKind::I64(100) + ArbitraryIntKind::I64(200),
        ArbitraryIntKind::I64(300)
    );
    // Overflow to U64
    assert!(matches!(
        ArbitraryIntKind::I64(i64::MAX) + ArbitraryIntKind::I64(1),
        ArbitraryIntKind::U64(v) if v == 1u64 << 63
    ));
    assert!(matches!(
        ArbitraryIntKind::I64(i64::MAX) + ArbitraryIntKind::I64(i64::MAX),
        ArbitraryIntKind::U64(v) if v == 2 * (i64::MAX as u64)
    ));
    // Overflow negative to BigInt
    let neg_overflow = IBig::from(i64::MIN) - IBig::from(1);
    assert!(matches!(
        ArbitraryIntKind::I64(i64::MIN) + ArbitraryIntKind::I64(-1),
        ArbitraryIntKind::BigInt(ref v) if *v == neg_overflow
    ));
    // Mixed tier addition
    assert!(matches!(
        ArbitraryIntKind::I64(10) + ArbitraryIntKind::U64(1u64 << 63),
        ArbitraryIntKind::U64(v) if v == (1u64 << 63) + 10
    ));
    let over_u64_add = IBig::from(u64::MAX) + IBig::from(1);
    assert!(matches!(
        ArbitraryIntKind::U64(u64::MAX) + ArbitraryIntKind::I64(1),
        ArbitraryIntKind::BigInt(ref v) if *v == over_u64_add
    ));

    // Sub:
    // Fast path on (I64, I64)
    assert!(matches!(
        ArbitraryIntKind::I64(300) - ArbitraryIntKind::I64(100),
        ArbitraryIntKind::I64(200)
    ));
    // Underflow to BigInt
    assert!(matches!(
        ArbitraryIntKind::I64(i64::MIN) - ArbitraryIntKind::I64(1),
        ArbitraryIntKind::BigInt(ref v) if *v == neg_overflow
    ));
    // Overflow positive to U64
    assert!(matches!(
        ArbitraryIntKind::I64(0) - ArbitraryIntKind::I64(i64::MIN),
        ArbitraryIntKind::U64(v) if v == 1u64 << 63
    ));
    assert!(matches!(
        ArbitraryIntKind::I64(1) - ArbitraryIntKind::I64(i64::MIN),
        ArbitraryIntKind::U64(v) if v == (1u64 << 63) + 1
    ));
    // Mixed tier subtraction
    assert!(matches!(
        ArbitraryIntKind::U64(1u64 << 63) - ArbitraryIntKind::I64(1),
        ArbitraryIntKind::I64(v) if v == i64::MAX
    ));
    assert!(matches!(
        ArbitraryIntKind::U64(1u64 << 63) - ArbitraryIntKind::U64(1u64 << 63),
        ArbitraryIntKind::I64(0)
    ));
    let neg_u64_max = -IBig::from(u64::MAX);
    assert!(matches!(
        ArbitraryIntKind::I64(0) - ArbitraryIntKind::U64(u64::MAX),
        ArbitraryIntKind::BigInt(ref v) if *v == neg_u64_max
    ));

    // Mul:
    // Fast path on (I64, I64)
    assert!(matches!(
        ArbitraryIntKind::I64(20) * ArbitraryIntKind::I64(15),
        ArbitraryIntKind::I64(300)
    ));
    // Overflow to U64
    assert!(matches!(
        ArbitraryIntKind::I64(i64::MAX) * ArbitraryIntKind::I64(2),
        ArbitraryIntKind::U64(v) if v == 2 * (i64::MAX as u64)
    ));
    // Overflow to BigInt
    let mul_big_pos = IBig::from(i64::MAX) * IBig::from(4);
    assert!(matches!(
        ArbitraryIntKind::I64(i64::MAX) * ArbitraryIntKind::I64(4),
        ArbitraryIntKind::BigInt(ref v) if *v == mul_big_pos
    ));
    let mul_big_neg = IBig::from(i64::MIN) * IBig::from(2);
    assert!(matches!(
        ArbitraryIntKind::I64(i64::MIN) * ArbitraryIntKind::I64(2),
        ArbitraryIntKind::BigInt(ref v) if *v == mul_big_neg
    ));
    // i64::MIN * -1 overflows to U64(1 << 63)
    assert!(matches!(
        ArbitraryIntKind::I64(i64::MIN) * ArbitraryIntKind::I64(-1),
        ArbitraryIntKind::U64(v) if v == 1u64 << 63
    ));

    // Reference operator combinations
    let a = ArbitraryIntKind::I64(10);
    let b = ArbitraryIntKind::I64(20);
    assert_eq!(&a + &b, ArbitraryIntKind::I64(30));
    assert_eq!(a.clone() + &b, ArbitraryIntKind::I64(30));
    assert_eq!(&a + b.clone(), ArbitraryIntKind::I64(30));
}

#[test]
fn arbitrary_int_division_and_remainder_edge_cases() {
    // Normal division and remainder
    assert!(matches!(
        ArbitraryIntKind::I64(300) / ArbitraryIntKind::I64(15),
        ArbitraryIntKind::I64(20)
    ));
    assert!(matches!(
        ArbitraryIntKind::I64(305) % ArbitraryIntKind::I64(15),
        ArbitraryIntKind::I64(5)
    ));

    // i64::MIN / -1 == U64(1 << 63) without panicking
    assert!(matches!(
        ArbitraryIntKind::I64(i64::MIN) / ArbitraryIntKind::I64(-1),
        ArbitraryIntKind::U64(v) if v == 1u64 << 63
    ));
    assert!(matches!(
        &ArbitraryIntKind::I64(i64::MIN) / &ArbitraryIntKind::I64(-1),
        ArbitraryIntKind::U64(v) if v == 1u64 << 63
    ));

    // i64::MIN % -1 == I64(0) without panicking
    assert!(matches!(
        ArbitraryIntKind::I64(i64::MIN) % ArbitraryIntKind::I64(-1),
        ArbitraryIntKind::I64(0)
    ));
    assert!(matches!(
        &ArbitraryIntKind::I64(i64::MIN) % &ArbitraryIntKind::I64(-1),
        ArbitraryIntKind::I64(0)
    ));

    // Division promoting / narrowing across tiers
    assert_eq!(
        ArbitraryIntKind::U64(1u64 << 63) / ArbitraryIntKind::I64(2),
        ArbitraryIntKind::I64(1 << 62)
    );
    assert_eq!(
        ArbitraryIntKind::U64(1u64 << 63) % ArbitraryIntKind::I64(2),
        ArbitraryIntKind::I64(0)
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(1) << 70) / ArbitraryIntKind::I64(2),
        ArbitraryIntKind::BigInt(IBig::from(1) << 69)
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(1) << 70)
            / ArbitraryIntKind::BigInt(IBig::from(1) << 69),
        ArbitraryIntKind::I64(2)
    );
}

#[test]
fn arbitrary_int_bitwise_operators() {
    let a = ArbitraryIntKind::I64(0b1100);
    let b = ArbitraryIntKind::I64(0b1010);

    // Primitive fast paths
    assert_eq!(&a & &b, ArbitraryIntKind::I64(0b1000));
    assert_eq!(&a | &b, ArbitraryIntKind::I64(0b1110));
    assert_eq!(&a ^ &b, ArbitraryIntKind::I64(0b0110));

    assert_eq!(a.clone() & b.clone(), ArbitraryIntKind::I64(0b1000));
    assert_eq!(a.clone() | b.clone(), ArbitraryIntKind::I64(0b1110));
    assert_eq!(a ^ b, ArbitraryIntKind::I64(0b0110));

    // Cross-tier bitwise ops
    assert_eq!(
        ArbitraryIntKind::I64(0b1100) & ArbitraryIntKind::U64((1u64 << 63) | 0b1010),
        ArbitraryIntKind::I64(0b1000)
    );
    assert_eq!(
        ArbitraryIntKind::I64(0) | ArbitraryIntKind::U64(1u64 << 63),
        ArbitraryIntKind::U64(1u64 << 63)
    );
    assert_eq!(
        ArbitraryIntKind::U64(1u64 << 63) ^ ArbitraryIntKind::U64(1u64 << 63),
        ArbitraryIntKind::I64(0)
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(1) << 70) & ArbitraryIntKind::I64(0),
        ArbitraryIntKind::I64(0)
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(1) << 70) | ArbitraryIntKind::I64(1),
        ArbitraryIntKind::BigInt((IBig::from(1) << 70) | 1)
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(1) << 70)
            ^ ArbitraryIntKind::BigInt(IBig::from(1) << 70),
        ArbitraryIntKind::I64(0)
    );
}

#[test]
fn arbitrary_int_helper_constructors_and_conversions() {
    assert!(matches!(int_from_i64(42), ArbitraryIntKind::I64(42)));
    assert!(matches!(int_from_i64(-42), ArbitraryIntKind::I64(-42)));
    // Unsigned constructors narrow to `I64` when the value fits.
    assert!(matches!(int_from_u64(42), ArbitraryIntKind::I64(42)));
    assert!(matches!(
        int_from_u64(1u64 << 63),
        ArbitraryIntKind::U64(v) if v == 1u64 << 63
    ));
    assert!(matches!(int_from_i32(42), ArbitraryIntKind::I64(42)));
    assert!(matches!(int_from_u32(42), ArbitraryIntKind::I64(42)));

    // is_zero
    assert!(ArbitraryIntKind::I64(0).is_zero());
    assert!(!ArbitraryIntKind::I64(1).is_zero());
    assert!(ArbitraryIntKind::U64(0).is_zero());
    assert!(!ArbitraryIntKind::U64(1).is_zero());
    assert!(ArbitraryIntKind::BigInt(IBig::from(0)).is_zero());
    assert!(!ArbitraryIntKind::BigInt(IBig::from(1)).is_zero());

    // BigInt conversions
    assert_eq!(ArbitraryIntKind::I64(42).to_bigint(), IBig::from(42));
    assert_eq!(
        ArbitraryIntKind::U64(1u64 << 63).to_bigint(),
        IBig::from(1u64 << 63)
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(100)).to_bigint(),
        IBig::from(100)
    );

    assert_eq!(ArbitraryIntKind::I64(42).into_bigint(), IBig::from(42));
    assert_eq!(
        ArbitraryIntKind::U64(1u64 << 63).into_bigint(),
        IBig::from(1u64 << 63)
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(100)).into_bigint(),
        IBig::from(100)
    );

    // to_bigint_cow borrow vs owned
    let val_big = ArbitraryIntKind::BigInt(IBig::from(100));
    assert_eq!(val_big.to_bigint_cow(), Cow::Borrowed(&IBig::from(100)));
    let val_i64 = ArbitraryIntKind::I64(42);
    assert_eq!(val_i64.to_bigint_cow(), Cow::Owned(IBig::from(42)));
    let val_u64 = ArbitraryIntKind::U64(1u64 << 63);
    assert_eq!(val_u64.to_bigint_cow(), Cow::Owned(IBig::from(1u64 << 63)));
}

#[test]
fn arbitrary_float_from_str_precision_and_zeros() {
    // Normal finite floats parse as F64
    assert_eq!(
        ArbitraryFloatKind::from_str("3.14159"),
        Some(ArbitraryFloatKind::F64(3.14159))
    );
    assert_eq!(
        ArbitraryFloatKind::from_str("-42.5"),
        Some(ArbitraryFloatKind::F64(-42.5))
    );
    assert_eq!(
        ArbitraryFloatKind::from_str("1e10"),
        Some(ArbitraryFloatKind::F64(1e10))
    );
    assert_eq!(
        ArbitraryFloatKind::from_str("1e-10"),
        Some(ArbitraryFloatKind::F64(1e-10))
    );

    // Underflow preserves precision as BigFloat
    let uf = ArbitraryFloatKind::from_str("1e-400").expect("underflow literal parses");
    assert!(
        matches!(uf, ArbitraryFloatKind::BigFloat(_)),
        "expected BigFloat for underflow 1e-400, got {uf:?}"
    );
    assert!(
        !uf.is_zero(),
        "underflowing float must not collapse to zero"
    );
    assert!(uf.is_finite());

    let neg_uf = ArbitraryFloatKind::from_str("-1e-400").expect("neg underflow literal parses");
    assert!(
        matches!(neg_uf, ArbitraryFloatKind::BigFloat(_)),
        "expected BigFloat for neg underflow -1e-400, got {neg_uf:?}"
    );
    assert!(
        !neg_uf.is_zero(),
        "negative underflowing float must not collapse to zero"
    );
    assert!(neg_uf.is_finite());

    // Overflow preserves precision as BigFloat
    let of = ArbitraryFloatKind::from_str("1e1000").expect("overflow literal parses");
    assert!(
        matches!(of, ArbitraryFloatKind::BigFloat(_)),
        "expected BigFloat for overflow 1e1000, got {of:?}"
    );
    assert!(
        of.is_finite(),
        "overflowing float must remain finite in DBig"
    );
    assert!(!of.is_zero());

    let neg_of = ArbitraryFloatKind::from_str("-1e1000").expect("neg overflow literal parses");
    assert!(
        matches!(neg_of, ArbitraryFloatKind::BigFloat(_)),
        "expected BigFloat for neg overflow -1e1000, got {neg_of:?}"
    );
    assert!(
        neg_of.is_finite(),
        "neg overflowing float must remain finite in DBig"
    );
    assert!(!neg_of.is_zero());

    // Exact zeros parse as F64(0.0)
    let z1 = ArbitraryFloatKind::from_str("0.0").expect("0.0 parses");
    assert!(matches!(z1, ArbitraryFloatKind::F64(v) if v == 0.0));
    assert!(z1.is_zero());

    let z2 = ArbitraryFloatKind::from_str("-0.0").expect("-0.0 parses");
    assert!(matches!(z2, ArbitraryFloatKind::F64(v) if v == 0.0));
    assert!(z2.is_zero());

    let z3 = ArbitraryFloatKind::from_str("0e1").expect("0e1 parses");
    assert!(matches!(z3, ArbitraryFloatKind::F64(v) if v == 0.0));
    assert!(z3.is_zero());

    let z4 = ArbitraryFloatKind::from_str("0e10").expect("0e10 parses");
    assert!(matches!(z4, ArbitraryFloatKind::F64(v) if v == 0.0));
    assert!(z4.is_zero());

    let z5 = ArbitraryFloatKind::from_str("0.000").expect("0.000 parses");
    assert!(matches!(z5, ArbitraryFloatKind::F64(v) if v == 0.0));
    assert!(z5.is_zero());

    // Invalid string returns None
    assert_eq!(ArbitraryFloatKind::from_str(""), None);
    assert_eq!(ArbitraryFloatKind::from_str("abc"), None);
    assert_eq!(ArbitraryFloatKind::from_str("--1.0"), None);
    assert_eq!(ArbitraryFloatKind::from_str("1.0.0"), None);
}

#[test]
fn arbitrary_float_equality_and_ordering() {
    // F64 vs BigFloat equality
    let bf_one = ArbitraryFloatKind::BigFloat(DBig::from_str("1.0").unwrap());
    let f_one = ArbitraryFloatKind::F64(1.0);
    assert_eq!(f_one, bf_one);
    assert_eq!(bf_one, f_one);

    let bf_frac = ArbitraryFloatKind::BigFloat(DBig::from_str("2.5").unwrap());
    let f_frac = ArbitraryFloatKind::F64(2.5);
    assert_eq!(f_frac, bf_frac);
    assert_eq!(bf_frac, f_frac);

    let bf_diff = ArbitraryFloatKind::BigFloat(DBig::from_str("1.0000000001").unwrap());
    assert_ne!(f_one, bf_diff);
    assert_ne!(bf_diff, f_one);

    // Out-of-range BigFloat compared with finite F64
    let huge = ArbitraryFloatKind::from_str("1e1000").unwrap();
    let neg_huge = ArbitraryFloatKind::from_str("-1e1000").unwrap();
    let finite = ArbitraryFloatKind::F64(5.0);

    assert_ne!(finite, huge);
    assert_ne!(huge, finite);
    assert!(finite < huge);
    assert!(huge > finite);

    assert_ne!(finite, neg_huge);
    assert_ne!(neg_huge, finite);
    assert!(finite > neg_huge);
    assert!(neg_huge < finite);

    assert!(neg_huge < huge);
    assert!(huge > neg_huge);

    // PartialOrd comparisons
    assert_eq!(finite.partial_cmp(&huge), Some(std::cmp::Ordering::Less));
    assert_eq!(huge.partial_cmp(&finite), Some(std::cmp::Ordering::Greater));
    assert_eq!(
        finite.partial_cmp(&neg_huge),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(
        neg_huge.partial_cmp(&finite),
        Some(std::cmp::Ordering::Less)
    );
}

#[test]
fn arbitrary_float_infinities_and_nan() {
    let nan = ArbitraryFloatKind::F64(f64::NAN);
    let huge = ArbitraryFloatKind::from_str("1e1000").unwrap();
    let neg_huge = ArbitraryFloatKind::from_str("-1e1000").unwrap();

    // NaN != NaN; NaN comparison returns None
    assert_ne!(nan, nan);
    assert_eq!(nan.partial_cmp(&nan), None);
    assert_eq!(nan.partial_cmp(&ArbitraryFloatKind::F64(1.0)), None);
    assert_eq!(ArbitraryFloatKind::F64(1.0).partial_cmp(&nan), None);
    assert_ne!(nan, huge);
    assert_ne!(huge, nan);
    assert_eq!(nan.partial_cmp(&huge), None);
    assert_eq!(huge.partial_cmp(&nan), None);

    // to_bigfloat / into_bigfloat on NaN returns None
    assert_eq!(nan.to_bigfloat(), None);
    assert_eq!(nan.into_bigfloat(), None);

    // Infinities
    let pos_inf = ArbitraryFloatKind::F64(f64::INFINITY);
    let pos_inf_bf = ArbitraryFloatKind::BigFloat(DBig::INFINITY);
    let neg_inf = ArbitraryFloatKind::F64(f64::NEG_INFINITY);
    let neg_inf_bf = ArbitraryFloatKind::BigFloat(DBig::NEG_INFINITY);

    // pos_inf == pos_inf_bf
    assert_eq!(pos_inf, pos_inf_bf);
    assert_eq!(pos_inf_bf, pos_inf);
    assert_eq!(neg_inf, neg_inf_bf);
    assert_eq!(neg_inf_bf, neg_inf);
    assert_ne!(pos_inf, neg_inf);

    // pos_inf > 1e1000
    assert!(pos_inf > huge);
    assert!(huge < pos_inf);
    assert_eq!(
        pos_inf.partial_cmp(&huge),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(huge.partial_cmp(&pos_inf), Some(std::cmp::Ordering::Less));

    // neg_inf < -1e1000
    assert!(neg_inf < neg_huge);
    assert!(neg_huge > neg_inf);
    assert_eq!(
        neg_inf.partial_cmp(&neg_huge),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        neg_huge.partial_cmp(&neg_inf),
        Some(std::cmp::Ordering::Greater)
    );

    // BigFloat infinity comparisons
    assert!(pos_inf_bf > huge);
    assert!(neg_inf_bf < neg_huge);
    assert!(pos_inf > neg_inf);
}

#[test]
fn arbitrary_float_binary_arithmetic() {
    // (F64, F64) stays in native f64
    assert_eq!(
        ArbitraryFloatKind::F64(2.5) + ArbitraryFloatKind::F64(3.5),
        ArbitraryFloatKind::F64(6.0)
    );
    assert_eq!(
        ArbitraryFloatKind::F64(10.0) - ArbitraryFloatKind::F64(3.0),
        ArbitraryFloatKind::F64(7.0)
    );
    assert_eq!(
        ArbitraryFloatKind::F64(4.0) * ArbitraryFloatKind::F64(2.5),
        ArbitraryFloatKind::F64(10.0)
    );
    assert_eq!(
        ArbitraryFloatKind::F64(15.0) / ArbitraryFloatKind::F64(3.0),
        ArbitraryFloatKind::F64(5.0)
    );
    assert_eq!(
        ArbitraryFloatKind::F64(17.0) % ArbitraryFloatKind::F64(5.0),
        ArbitraryFloatKind::F64(2.0)
    );

    // IEEE division by zero stays in F64
    assert_eq!(
        ArbitraryFloatKind::F64(1.0) / ArbitraryFloatKind::F64(0.0),
        ArbitraryFloatKind::F64(f64::INFINITY)
    );
    assert_eq!(
        ArbitraryFloatKind::F64(-1.0) / ArbitraryFloatKind::F64(0.0),
        ArbitraryFloatKind::F64(f64::NEG_INFINITY)
    );
    let nan_div = ArbitraryFloatKind::F64(0.0) / ArbitraryFloatKind::F64(0.0);
    assert!(nan_div.to_f64().is_nan());
    assert!(matches!(nan_div, ArbitraryFloatKind::F64(_)));

    // Mixed finite operations promote to BigFloat
    let bf_3 = ArbitraryFloatKind::BigFloat(DBig::from_str("3.0").unwrap());
    let add_res = ArbitraryFloatKind::F64(2.0) + &bf_3;
    assert!(
        matches!(add_res, ArbitraryFloatKind::BigFloat(_)),
        "expected BigFloat for mixed add, got {add_res:?}"
    );
    assert_eq!(
        add_res,
        ArbitraryFloatKind::BigFloat(DBig::from_str("5.0").unwrap())
    );

    let bf_10 = ArbitraryFloatKind::BigFloat(DBig::from_str("10.0").unwrap());
    let sub_res = &bf_10 - ArbitraryFloatKind::F64(3.0);
    assert!(
        matches!(sub_res, ArbitraryFloatKind::BigFloat(_)),
        "expected BigFloat for mixed sub, got {sub_res:?}"
    );
    assert_eq!(
        sub_res,
        ArbitraryFloatKind::BigFloat(DBig::from_str("7.0").unwrap())
    );

    let mul_res =
        ArbitraryFloatKind::F64(4.0) * ArbitraryFloatKind::BigFloat(DBig::from_str("2.5").unwrap());
    assert!(
        matches!(mul_res, ArbitraryFloatKind::BigFloat(_)),
        "expected BigFloat for mixed mul, got {mul_res:?}"
    );
    assert_eq!(
        mul_res,
        ArbitraryFloatKind::BigFloat(DBig::from_str("10.0").unwrap())
    );

    let div_res = ArbitraryFloatKind::BigFloat(DBig::from_str("15.0").unwrap())
        / ArbitraryFloatKind::F64(3.0);
    assert!(
        matches!(div_res, ArbitraryFloatKind::BigFloat(_)),
        "expected BigFloat for mixed div, got {div_res:?}"
    );
    assert_eq!(
        div_res,
        ArbitraryFloatKind::BigFloat(DBig::from_str("5.0").unwrap())
    );

    let rem_res = ArbitraryFloatKind::BigFloat(DBig::from_str("17.0").unwrap())
        % ArbitraryFloatKind::F64(5.0);
    assert!(
        matches!(rem_res, ArbitraryFloatKind::BigFloat(_)),
        "expected BigFloat for mixed rem, got {rem_res:?}"
    );
    assert_eq!(
        rem_res,
        ArbitraryFloatKind::BigFloat(DBig::from_str("2.0").unwrap())
    );

    // BigFloat + BigFloat preserves precision without collapsing
    let h1 = ArbitraryFloatKind::from_str("1e1000").unwrap();
    let h2 = ArbitraryFloatKind::from_str("2e1000").unwrap();
    let sum = &h1 + &h2;
    assert_eq!(sum, ArbitraryFloatKind::from_str("3e1000").unwrap());

    // Operations with non-finite operands stay in f64 where IEEE 754 applies
    let inf = ArbitraryFloatKind::F64(f64::INFINITY);
    let bf = ArbitraryFloatKind::from_str("1e500").unwrap();
    let res_inf1 = &inf + &bf;
    assert_eq!(res_inf1, ArbitraryFloatKind::F64(f64::INFINITY));
    let res_inf2 = &bf + &inf;
    assert_eq!(res_inf2, ArbitraryFloatKind::F64(f64::INFINITY));

    let nan = ArbitraryFloatKind::F64(f64::NAN);
    let res_nan1 = &nan + &bf;
    assert!(res_nan1.to_f64().is_nan());
    assert!(matches!(res_nan1, ArbitraryFloatKind::F64(_)));
    let res_nan2 = &bf * &nan;
    assert!(res_nan2.to_f64().is_nan());
    assert!(matches!(res_nan2, ArbitraryFloatKind::F64(_)));

    // Negation
    assert_eq!(-ArbitraryFloatKind::F64(3.5), ArbitraryFloatKind::F64(-3.5));
    assert_eq!(
        -&ArbitraryFloatKind::F64(3.5),
        ArbitraryFloatKind::F64(-3.5)
    );
    let neg_h1 = -h1.clone();
    assert_eq!(neg_h1, ArbitraryFloatKind::from_str("-1e1000").unwrap());
    assert_eq!(-neg_h1, h1);
}

#[test]
fn arbitrary_float_bigfloat_arithmetic_rounds_ties_to_even() {
    // Both operands have one decimal digit of precision. Their exact sum is 2.5e1000,
    // halfway between 2e1000 and 3e1000; IEEE's default mode selects the even 2.
    let even = ArbitraryFloatKind::from_str("2e1000").unwrap();
    let half = ArbitraryFloatKind::from_str("5e999").unwrap();

    assert_eq!(even.clone() + half.clone(), even);
    assert_eq!(even.clone() + &half, even);
    assert_eq!(&even + half.clone(), even);
    assert_eq!(&even + &half, even);
}

#[test]
fn arbitrary_float_bigfloat_remainder_truncates_quotient_toward_zero() {
    for (lhs_text, rhs_text, expected_text) in [
        ("7", "4", "3"),
        ("7", "-4", "3"),
        ("-7", "4", "-3"),
        ("-7", "-4", "-3"),
    ] {
        let lhs = ArbitraryFloatKind::BigFloat(DBig::from_str(lhs_text).unwrap());
        let rhs = ArbitraryFloatKind::BigFloat(DBig::from_str(rhs_text).unwrap());
        let expected = DBig::from_str(expected_text).unwrap();

        for result in [
            lhs.clone() % rhs.clone(),
            lhs.clone() % &rhs,
            &lhs % rhs.clone(),
            &lhs % &rhs,
        ] {
            assert!(
                matches!(result, ArbitraryFloatKind::BigFloat(ref actual) if actual == &expected),
                "expected {lhs_text} % {rhs_text} to be BigFloat({expected_text}), got {result:?}"
            );
        }
    }

    let mixed_rhs =
        ArbitraryFloatKind::F64(7.0) % ArbitraryFloatKind::BigFloat(DBig::from_str("4").unwrap());
    assert!(
        matches!(mixed_rhs, ArbitraryFloatKind::BigFloat(ref actual) if actual == &DBig::from_str("3").unwrap()),
        "mixed F64/BigFloat remainder must use truncating BigFloat semantics, got {mixed_rhs:?}"
    );
    let mixed_lhs =
        ArbitraryFloatKind::BigFloat(DBig::from_str("-7").unwrap()) % ArbitraryFloatKind::F64(-4.0);
    assert!(
        matches!(mixed_lhs, ArbitraryFloatKind::BigFloat(ref actual) if actual == &DBig::from_str("-3").unwrap()),
        "mixed BigFloat/F64 remainder must keep the dividend sign, got {mixed_lhs:?}"
    );
}

#[test]
fn arbitrary_float_helper_constructors_and_conversions() {
    assert_eq!(float_from_f64(3.14), ArbitraryFloatKind::F64(3.14));
    assert_eq!(
        float_from_f32(3.14f32),
        ArbitraryFloatKind::F64(3.14f32 as f64)
    );

    let dbig_val = DBig::from_str("1.5").unwrap();
    let from_trait: ArbitraryFloatKind = dbig_val.clone().into();
    assert_eq!(from_trait, ArbitraryFloatKind::BigFloat(dbig_val));

    // to_f64 and to_bits
    assert_eq!(ArbitraryFloatKind::F64(12.34).to_f64(), 12.34);
    assert_eq!(
        ArbitraryFloatKind::BigFloat(DBig::from_str("12.34").unwrap()).to_f64(),
        12.34
    );
    assert_eq!(ArbitraryFloatKind::F64(12.34).to_bits(), 12.34f64.to_bits());
    assert_eq!(
        ArbitraryFloatKind::BigFloat(DBig::from_str("12.34").unwrap()).to_bits(),
        12.34f64.to_bits()
    );

    // to_bigfloat / into_bigfloat
    assert_eq!(
        ArbitraryFloatKind::F64(12.5).to_bigfloat(),
        Some(DBig::from_str("12.5").unwrap())
    );
    assert_eq!(
        ArbitraryFloatKind::F64(f64::INFINITY).to_bigfloat(),
        Some(DBig::INFINITY)
    );
    assert_eq!(
        ArbitraryFloatKind::F64(f64::NEG_INFINITY).to_bigfloat(),
        Some(DBig::NEG_INFINITY)
    );
    assert_eq!(
        ArbitraryFloatKind::F64(12.5).into_bigfloat(),
        Some(DBig::from_str("12.5").unwrap())
    );
    let bf = DBig::from_str("12.5").unwrap();
    assert_eq!(
        ArbitraryFloatKind::BigFloat(bf.clone()).into_bigfloat(),
        Some(bf)
    );

    // is_zero
    assert!(ArbitraryFloatKind::F64(0.0).is_zero());
    assert!(ArbitraryFloatKind::F64(-0.0).is_zero());
    assert!(!ArbitraryFloatKind::F64(0.0001).is_zero());
    assert!(ArbitraryFloatKind::BigFloat(DBig::from_str("0.0").unwrap()).is_zero());
    assert!(!ArbitraryFloatKind::BigFloat(DBig::INFINITY).is_zero());
    assert!(!ArbitraryFloatKind::BigFloat(DBig::NEG_INFINITY).is_zero());
    assert!(!ArbitraryFloatKind::BigFloat(DBig::from_str("1e-400").unwrap()).is_zero());

    // is_finite
    assert!(ArbitraryFloatKind::F64(1.0).is_finite());
    assert!(!ArbitraryFloatKind::F64(f64::INFINITY).is_finite());
    assert!(!ArbitraryFloatKind::F64(f64::NEG_INFINITY).is_finite());
    assert!(!ArbitraryFloatKind::F64(f64::NAN).is_finite());
    assert!(ArbitraryFloatKind::BigFloat(DBig::from_str("1e1000").unwrap()).is_finite());
    assert!(!ArbitraryFloatKind::BigFloat(DBig::INFINITY).is_finite());
    assert!(!ArbitraryFloatKind::BigFloat(DBig::NEG_INFINITY).is_finite());
}

#[test]
fn arbitrary_float_conversions_use_ieee_rounding_and_exact_binary_values() {
    let midpoint = ArbitraryFloatKind::BigFloat(
        DBig::from_str("1.00000000000000011102230246251565404236316680908203125").unwrap(),
    );
    assert_eq!(midpoint.to_bits(), 1.0f64.to_bits());

    // This adjacent tie has an odd lower significand, so ties-to-even must round upward.
    let next_midpoint = ArbitraryFloatKind::BigFloat(
        DBig::from_str("1.00000000000000033306690738754696212708950042724609375").unwrap(),
    );
    assert_eq!(
        next_midpoint.to_bits(),
        (1.0f64.to_bits() + 2),
        "the even upper significand must win the adjacent tie"
    );

    // 2^-1075 is exactly halfway between zero and the smallest binary64 subnormal.
    // 3 * 2^-1075 is halfway between the first (odd) and second (even) subnormals.
    let half_min_subnormal = DBig::from_parts(IBig::from(5_u8).pow(1075), -1075);
    let three_halves_min_subnormal =
        DBig::from_parts(IBig::from(3_u8) * IBig::from(5_u8).pow(1075), -1075);
    assert_eq!(
        ArbitraryFloatKind::BigFloat(half_min_subnormal.clone()).to_bits(),
        0.0f64.to_bits()
    );
    assert_eq!(
        ArbitraryFloatKind::BigFloat(-half_min_subnormal).to_bits(),
        (-0.0f64).to_bits(),
        "underflow rounding must preserve the sign of zero"
    );
    assert_eq!(
        ArbitraryFloatKind::BigFloat(three_halves_min_subnormal).to_bits(),
        2,
        "the even second subnormal must win the tie"
    );

    let exact_binary_tenth =
        DBig::from_str("0.1000000000000000055511151231257827021181583404541015625").unwrap();
    let binary_tenth = ArbitraryFloatKind::F64(0.1);
    assert_eq!(binary_tenth.to_bigfloat(), Some(exact_binary_tenth.clone()));
    assert_eq!(
        binary_tenth,
        ArbitraryFloatKind::BigFloat(exact_binary_tenth)
    );
    assert_ne!(
        binary_tenth,
        ArbitraryFloatKind::BigFloat(DBig::from_str("0.1").unwrap())
    );

    let exact_min_subnormal = DBig::from_parts(IBig::from(5_u8).pow(1074), -1074);
    assert_eq!(
        ArbitraryFloatKind::F64(f64::from_bits(1)).to_bigfloat(),
        Some(exact_min_subnormal)
    );
}

#[test]
fn arbitrary_float_zero_divisor_ieee_safety() {
    // BigFloat / 0.0 must yield IEEE infinity rather than panicking in DBig
    let bf_ten = ArbitraryFloatKind::BigFloat(DBig::from_str("10.0").unwrap());
    let bf_zero = ArbitraryFloatKind::BigFloat(DBig::from_str("0.0").unwrap());
    let f_zero = ArbitraryFloatKind::F64(0.0);
    let neg_f_zero = ArbitraryFloatKind::F64(-0.0);

    let div1 = &bf_ten / &f_zero;
    assert_eq!(div1, ArbitraryFloatKind::F64(f64::INFINITY));

    let div2 = &bf_ten / &bf_zero;
    assert_eq!(div2, ArbitraryFloatKind::F64(f64::INFINITY));

    let div_neg = &bf_ten / &neg_f_zero;
    assert_eq!(div_neg, ArbitraryFloatKind::F64(f64::NEG_INFINITY));

    // 0.0 / 0.0 must yield NaN rather than panicking in DBig
    let nan_div1 = &bf_zero / &f_zero;
    assert!(nan_div1.to_f64().is_nan());

    let nan_div2 = &bf_zero / &bf_zero;
    assert!(nan_div2.to_f64().is_nan());

    // BigFloat % 0.0 must yield NaN rather than panicking in DBig
    let rem1 = &bf_ten % &f_zero;
    assert!(rem1.to_f64().is_nan());

    let rem2 = &bf_ten % &bf_zero;
    assert!(rem2.to_f64().is_nan());

    let huge = ArbitraryFloatKind::from_str("1e1000").unwrap();
    let rem_huge = &huge % &f_zero;
    assert!(rem_huge.to_f64().is_nan());

    // Underflowing BigFloat / 0.0 must yield IEEE infinity rather than collapsing to NaN
    let uf = ArbitraryFloatKind::from_str("1e-400").unwrap();
    let neg_uf = ArbitraryFloatKind::from_str("-1e-400").unwrap();
    assert_eq!(&uf / &f_zero, ArbitraryFloatKind::F64(f64::INFINITY));
    assert_eq!(
        &uf / &neg_f_zero,
        ArbitraryFloatKind::F64(f64::NEG_INFINITY)
    );
    assert_eq!(
        &neg_uf / &f_zero,
        ArbitraryFloatKind::F64(f64::NEG_INFINITY)
    );
    assert_eq!(
        &neg_uf / &neg_f_zero,
        ArbitraryFloatKind::F64(f64::INFINITY)
    );
    assert_eq!(&uf / &bf_zero, ArbitraryFloatKind::F64(f64::INFINITY));
    assert_eq!(
        &neg_uf / &bf_zero,
        ArbitraryFloatKind::F64(f64::NEG_INFINITY)
    );
}

#[test]
fn arbitrary_float_mixed_infinity_arithmetic_precision() {
    let huge = ArbitraryFloatKind::from_str("1e1000").unwrap();
    let neg_huge = ArbitraryFloatKind::from_str("-1e1000").unwrap();
    let pos_inf = ArbitraryFloatKind::F64(f64::INFINITY);
    let neg_inf = ArbitraryFloatKind::F64(f64::NEG_INFINITY);

    // finite - (+inf) == -inf (does not collapse to NaN)
    assert_eq!(&huge - &pos_inf, ArbitraryFloatKind::F64(f64::NEG_INFINITY));
    // (+inf) - finite == +inf
    assert_eq!(&pos_inf - &huge, ArbitraryFloatKind::F64(f64::INFINITY));

    // finite - (-inf) == +inf
    assert_eq!(&huge - &neg_inf, ArbitraryFloatKind::F64(f64::INFINITY));
    // (-inf) - finite == -inf
    assert_eq!(&neg_inf - &huge, ArbitraryFloatKind::F64(f64::NEG_INFINITY));

    // finite + (+inf) == +inf
    assert_eq!(&neg_huge + &pos_inf, ArbitraryFloatKind::F64(f64::INFINITY));
    assert_eq!(&pos_inf + &neg_huge, ArbitraryFloatKind::F64(f64::INFINITY));

    // finite / (+inf) == 0.0
    assert_eq!(&huge / &pos_inf, ArbitraryFloatKind::F64(0.0));
    assert_eq!(&neg_huge / &pos_inf, ArbitraryFloatKind::F64(-0.0));

    // (+inf) / finite == +inf
    assert_eq!(&pos_inf / &huge, ArbitraryFloatKind::F64(f64::INFINITY));
    assert_eq!(
        &pos_inf / &neg_huge,
        ArbitraryFloatKind::F64(f64::NEG_INFINITY)
    );

    // finite * (+inf) == +inf / -inf
    assert_eq!(&huge * &pos_inf, ArbitraryFloatKind::F64(f64::INFINITY));
    assert_eq!(
        &neg_huge * &pos_inf,
        ArbitraryFloatKind::F64(f64::NEG_INFINITY)
    );

    // underflow * inf == inf (does not collapse to NaN)
    let uf = ArbitraryFloatKind::from_str("1e-400").unwrap();
    let neg_uf = ArbitraryFloatKind::from_str("-1e-400").unwrap();
    assert_eq!(&uf * &pos_inf, ArbitraryFloatKind::F64(f64::INFINITY));
    assert_eq!(&pos_inf * &uf, ArbitraryFloatKind::F64(f64::INFINITY));
    assert_eq!(
        &neg_uf * &pos_inf,
        ArbitraryFloatKind::F64(f64::NEG_INFINITY)
    );
    assert_eq!(
        &pos_inf * &neg_uf,
        ArbitraryFloatKind::F64(f64::NEG_INFINITY)
    );
    assert_eq!(&uf * &neg_inf, ArbitraryFloatKind::F64(f64::NEG_INFINITY));
    assert_eq!(&neg_uf * &neg_inf, ArbitraryFloatKind::F64(f64::INFINITY));

    // underflow / inf == 0.0
    assert_eq!(&uf / &pos_inf, ArbitraryFloatKind::F64(0.0));
    assert_eq!(&neg_uf / &pos_inf, ArbitraryFloatKind::F64(-0.0));
    assert_eq!(&pos_inf / &uf, ArbitraryFloatKind::F64(f64::INFINITY));
    assert_eq!(
        &pos_inf / &neg_uf,
        ArbitraryFloatKind::F64(f64::NEG_INFINITY)
    );

    // IEEE remainder with infinity: x % ±inf == x; ±inf % y == NaN
    assert_eq!(&huge % &pos_inf, huge);
    assert_eq!(&huge % &neg_inf, huge);
    let rem_inf = &pos_inf % &huge;
    assert!(rem_inf.to_f64().is_nan());
}

#[test]
fn arbitrary_float_signed_zero_parsing() {
    let z_neg_exp = ArbitraryFloatKind::from_str("-0e10").expect("-0e10 parses");
    assert!(z_neg_exp.is_zero());
    assert!(z_neg_exp.to_f64().is_sign_negative());

    let z_neg_frac_exp = ArbitraryFloatKind::from_str("-0.0e-500").expect("-0.0e-500 parses");
    assert!(z_neg_frac_exp.is_zero());
    assert!(z_neg_frac_exp.to_f64().is_sign_negative());

    let z_pos_exp = ArbitraryFloatKind::from_str("0e10").expect("0e10 parses");
    assert!(z_pos_exp.is_zero());
    assert!(z_pos_exp.to_f64().is_sign_positive());
}

#[test]
fn arbitrary_float_explicit_ieee_special_values_remain_f64() {
    let nan = ArbitraryFloatKind::from_str("NaN").expect("NaN parses");
    assert!(matches!(nan, ArbitraryFloatKind::F64(value) if value.is_nan()));

    assert_eq!(
        ArbitraryFloatKind::from_str("inf"),
        Some(ArbitraryFloatKind::F64(f64::INFINITY))
    );
    assert_eq!(
        ArbitraryFloatKind::from_str("-inf"),
        Some(ArbitraryFloatKind::F64(f64::NEG_INFINITY))
    );
}

#[test]
fn arbitrary_float_signed_zero_arithmetic_follows_ieee_754() {
    let positive = ArbitraryFloatKind::BigFloat(DBig::from_str("3.0").unwrap());
    let negative = ArbitraryFloatKind::BigFloat(DBig::from_str("-3.0").unwrap());
    let positive_zero = ArbitraryFloatKind::F64(0.0);
    let negative_zero = ArbitraryFloatKind::F64(-0.0);

    assert_eq!(
        [
            (&positive * &negative_zero).to_bits(),
            (&negative * &negative_zero).to_bits(),
            (&negative_zero / &positive).to_bits(),
            (&negative_zero / &negative).to_bits(),
            (&negative_zero % &positive).to_bits(),
        ],
        [
            (-0.0f64).to_bits(),
            0.0f64.to_bits(),
            (-0.0f64).to_bits(),
            0.0f64.to_bits(),
            (-0.0f64).to_bits(),
        ]
    );

    assert_eq!(
        (&negative_zero + &negative_zero).to_bits(),
        (-0.0f64).to_bits()
    );
    assert_eq!(
        (&negative_zero + &positive_zero).to_bits(),
        0.0f64.to_bits()
    );
}

#[test]
fn arbitrary_float_bigfloat_zero_negation_produces_negative_zero() {
    let positive_zero = ArbitraryFloatKind::BigFloat(DBig::from_str("0").unwrap());

    assert_eq!(
        (-positive_zero.clone()).to_bits(),
        (-0.0f64).to_bits(),
        "owned negation must flip the sign of BigFloat zero"
    );
    assert_eq!(
        (-&positive_zero).to_bits(),
        (-0.0f64).to_bits(),
        "borrowed negation must flip the sign of BigFloat zero"
    );
}

#[test]
fn arbitrary_float_indeterminate_operations_produce_nan() {
    let positive_infinity = ArbitraryFloatKind::BigFloat(DBig::INFINITY);
    let negative_infinity = ArbitraryFloatKind::BigFloat(DBig::NEG_INFINITY);
    let zero = ArbitraryFloatKind::F64(0.0);
    let finite = ArbitraryFloatKind::BigFloat(DBig::from_str("3.0").unwrap());

    for result in [
        &positive_infinity + &negative_infinity,
        &positive_infinity - &positive_infinity,
        &zero * &positive_infinity,
        &positive_infinity * &zero,
        &positive_infinity / &positive_infinity,
        &positive_infinity % &finite,
    ] {
        assert!(
            matches!(result, ArbitraryFloatKind::F64(value) if value.is_nan()),
            "expected an f64 NaN, got {result:?}"
        );
    }
}

#[test]
fn arbitrary_int_ord_and_sorting() {
    let mut values = vec![
        ArbitraryIntKind::I64(10),
        ArbitraryIntKind::U64(1u64 << 63),
        ArbitraryIntKind::I64(-5),
        ArbitraryIntKind::BigInt(IBig::from(1) << 70),
        ArbitraryIntKind::BigInt(-(IBig::from(1) << 70)),
        ArbitraryIntKind::I64(0),
    ];
    values.sort();

    assert_eq!(
        values,
        vec![
            ArbitraryIntKind::BigInt(-(IBig::from(1) << 70)),
            ArbitraryIntKind::I64(-5),
            ArbitraryIntKind::I64(0),
            ArbitraryIntKind::I64(10),
            ArbitraryIntKind::U64(1u64 << 63),
            ArbitraryIntKind::BigInt(IBig::from(1) << 70),
        ]
    );
}

#[test]
fn arbitrary_int_u64_fast_paths() {
    let a = ArbitraryIntKind::U64(10);
    let b = ArbitraryIntKind::U64(20);
    assert_eq!(a + b, ArbitraryIntKind::I64(30));

    let u_mid = ArbitraryIntKind::U64(1u64 << 63);
    assert_eq!(
        &u_mid + &ArbitraryIntKind::U64(1),
        ArbitraryIntKind::U64((1u64 << 63) + 1)
    );

    // Overflow to BigInt
    assert_eq!(
        ArbitraryIntKind::U64(u64::MAX) + ArbitraryIntKind::U64(1),
        ArbitraryIntKind::BigInt(IBig::from(u64::MAX) + IBig::from(1))
    );

    // Bitwise ops fast path
    assert_eq!(
        ArbitraryIntKind::U64(0b1100) & ArbitraryIntKind::U64(0b1010),
        ArbitraryIntKind::I64(0b1000)
    );
    assert_eq!(
        ArbitraryIntKind::U64(1u64 << 63) | ArbitraryIntKind::U64(0b1010),
        ArbitraryIntKind::U64((1u64 << 63) | 0b1010)
    );
    assert_eq!(
        ArbitraryIntKind::U64(1u64 << 63) ^ ArbitraryIntKind::U64(1u64 << 63),
        ArbitraryIntKind::I64(0)
    );
}

#[test]
fn arbitrary_type_id_returns_expected_core_type_id() {
    assert_eq!(
        ArbitraryIntKind::I64(42).type_id(),
        TypeId::new(compiler_constants::CORE_I64)
    );
    assert_eq!(
        ArbitraryIntKind::U64(42).type_id(),
        TypeId::new(compiler_constants::CORE_U64)
    );
    assert_eq!(
        ArbitraryIntKind::BigInt(IBig::from(42)).type_id(),
        TypeId::new(compiler_constants::CORE_BIGINT)
    );

    assert_eq!(
        ArbitraryFloatKind::F64(42.0).type_id(),
        TypeId::new(compiler_constants::CORE_F64)
    );
    assert_eq!(
        ArbitraryFloatKind::BigFloat(DBig::from_str("1e1000").unwrap()).type_id(),
        TypeId::new(compiler_constants::CORE_BIGFLOAT)
    );
}

#[test]
fn arbitrary_type_id_is_const_evaluable() {
    const I64_TY: TypeId = ArbitraryIntKind::I64(0).type_id();
    const U64_TY: TypeId = ArbitraryIntKind::U64(0).type_id();
    const F64_TY: TypeId = ArbitraryFloatKind::F64(0.0).type_id();

    assert_eq!(I64_TY, TypeId::new(compiler_constants::CORE_I64));
    assert_eq!(U64_TY, TypeId::new(compiler_constants::CORE_U64));
    assert_eq!(F64_TY, TypeId::new(compiler_constants::CORE_F64));
}

#[test]
fn arbitrary_int_from_str_with_limit_boundaries() {
    const BIN_POW64: &str = "10000000000000000000000000000000000000000000000000000000000000000";
    const OCTAL_POW64: &str = "2000000000000000000000";
    const HEX_POW64: &str = "10000000000000000";

    let pow64 = IBig::from(1u8) << 64;

    assert_eq!(
        ArbitraryIntKind::from_str_with_limit("255", Notation::Decimal, 8),
        Ok(ArbitraryIntKind::I64(255))
    );
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit("256", Notation::Decimal, 8),
        Err(NumericIntParseError::LimitExceeded)
    );
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit(BIN_POW64, Notation::Bin, 64),
        Err(NumericIntParseError::LimitExceeded)
    );
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit(BIN_POW64, Notation::Bin, 65),
        Ok(ArbitraryIntKind::BigInt(pow64.clone()))
    );
    assert!(ArbitraryIntKind::literal_exceeds_numeric_bits(
        BIN_POW64,
        Notation::Bin,
        64
    ));
    assert!(!ArbitraryIntKind::literal_exceeds_numeric_bits(
        BIN_POW64,
        Notation::Bin,
        65
    ));
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit(OCTAL_POW64, Notation::Octal, 64),
        Err(NumericIntParseError::LimitExceeded)
    );
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit(OCTAL_POW64, Notation::Octal, 65),
        Ok(ArbitraryIntKind::BigInt(pow64.clone()))
    );
    assert!(ArbitraryIntKind::literal_exceeds_numeric_bits(
        OCTAL_POW64,
        Notation::Octal,
        64
    ));
    assert!(!ArbitraryIntKind::literal_exceeds_numeric_bits(
        OCTAL_POW64,
        Notation::Octal,
        65
    ));
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit(HEX_POW64, Notation::Hex, 64),
        Err(NumericIntParseError::LimitExceeded)
    );
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit(HEX_POW64, Notation::Hex, 65),
        Ok(ArbitraryIntKind::BigInt(pow64.clone()))
    );
    assert!(ArbitraryIntKind::literal_exceeds_numeric_bits(
        HEX_POW64,
        Notation::Hex,
        64
    ));
    assert!(!ArbitraryIntKind::literal_exceeds_numeric_bits(
        HEX_POW64,
        Notation::Hex,
        65
    ));
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit("-255", Notation::Decimal, 8),
        Ok(ArbitraryIntKind::I64(-255))
    );
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit("-256", Notation::Decimal, 8),
        Err(NumericIntParseError::LimitExceeded)
    );
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit("9223372036854775808", Notation::Decimal, 64),
        Ok(ArbitraryIntKind::U64(1u64 << 63))
    );
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit("9223372036854775808", Notation::Decimal, 63),
        Err(NumericIntParseError::LimitExceeded)
    );
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit("18446744073709551616", Notation::Decimal, 64),
        Err(NumericIntParseError::LimitExceeded)
    );
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit("18446744073709551616", Notation::Decimal, 65),
        Ok(ArbitraryIntKind::BigInt(pow64))
    );
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit("not_a_number", Notation::Decimal, 64),
        Err(NumericIntParseError::Invalid)
    );
    // Invalid spellings report `Invalid` even when the digit-length estimate
    // alone exceeds a small limit.
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit("12a3", Notation::Decimal, 8),
        Err(NumericIntParseError::Invalid)
    );
    assert!(!ArbitraryIntKind::literal_exceeds_numeric_bits(
        "12a3",
        Notation::Decimal,
        8
    ));
    assert_eq!(
        ArbitraryIntKind::from_str_with_limit("1x", Notation::Hex, 4),
        Err(NumericIntParseError::Invalid)
    );
    assert!(!ArbitraryIntKind::literal_exceeds_numeric_bits(
        "1x",
        Notation::Hex,
        4
    ));
}

#[test]
fn arbitrary_float_from_str_with_limit_evaluates_f64_first() {
    assert_eq!(
        ArbitraryFloatKind::from_str_with_limit("1.5", 64),
        Ok(ArbitraryFloatKind::F64(1.5))
    );
    assert_eq!(
        ArbitraryFloatKind::from_str_with_limit("1.5", 63),
        Err(NumericFloatParseError::LimitExceeded)
    );
    assert_eq!(
        ArbitraryFloatKind::from_str_with_limit("1e1000", 64),
        Err(NumericFloatParseError::LimitExceeded)
    );
    assert!(matches!(
        ArbitraryFloatKind::from_str_with_limit("1e1000", 4096),
        Ok(ArbitraryFloatKind::BigFloat(_))
    ));
    assert_eq!(
        ArbitraryFloatKind::from_str_with_limit("not_a_float", 64),
        Err(NumericFloatParseError::Invalid)
    );
}

#[test]
fn arbitrary_float_infinite_numeric_bits_is_max() {
    let inf = ArbitraryFloatKind::BigFloat(dashu_float::DBig::INFINITY);
    assert_eq!(inf.numeric_bits(), u64::MAX);
    assert!(!inf.fits_numeric_bits(4096));
    assert!(!inf.fits_numeric_bits(u64::MAX - 1));
    assert!(inf.fits_numeric_bits(u64::MAX));

    let neg_inf = ArbitraryFloatKind::BigFloat(dashu_float::DBig::NEG_INFINITY);
    assert_eq!(neg_inf.numeric_bits(), u64::MAX);
    assert!(!neg_inf.fits_numeric_bits(4096));

    let f64_inf = ArbitraryFloatKind::F64(f64::INFINITY);
    assert_eq!(f64_inf.numeric_bits(), u64::MAX);
    assert!(!f64_inf.fits_numeric_bits(4096));
    assert!(!f64_inf.fits_numeric_bits(u64::MAX - 1));
    assert!(f64_inf.fits_numeric_bits(u64::MAX));

    let f64_neg_inf = ArbitraryFloatKind::F64(f64::NEG_INFINITY);
    assert_eq!(f64_neg_inf.numeric_bits(), u64::MAX);
    assert!(!f64_neg_inf.fits_numeric_bits(4096));

    let f64_nan = ArbitraryFloatKind::F64(f64::NAN);
    assert_eq!(f64_nan.numeric_bits(), u64::MAX);
    assert!(!f64_nan.fits_numeric_bits(4096));
}

#[test]
fn arbitrary_float_f64_overflow_follows_ieee_754() {
    let mul = ArbitraryFloatKind::F64(1e308) * ArbitraryFloatKind::F64(2.0);
    assert_eq!(mul, ArbitraryFloatKind::F64(f64::INFINITY));

    let add = ArbitraryFloatKind::F64(1e308) + ArbitraryFloatKind::F64(1e308);
    assert_eq!(add, ArbitraryFloatKind::F64(f64::INFINITY));

    let sub = ArbitraryFloatKind::F64(-1e308) - ArbitraryFloatKind::F64(1e308);
    assert_eq!(sub, ArbitraryFloatKind::F64(f64::NEG_INFINITY));

    let div = ArbitraryFloatKind::F64(1e308) / ArbitraryFloatKind::F64(0.1);
    assert_eq!(div, ArbitraryFloatKind::F64(f64::INFINITY));

    let div_zero = ArbitraryFloatKind::F64(1.0) / ArbitraryFloatKind::F64(0.0);
    assert_eq!(div_zero, ArbitraryFloatKind::F64(f64::INFINITY));
    assert_eq!(div_zero.numeric_bits(), u64::MAX);
}
