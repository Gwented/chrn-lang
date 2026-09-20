use chrn_utils::{intern::Intern, utils::containers::SpannedContainerRef};

use crate::{
    parser::ast::ast_concepts::{BinaryOp, UnaryOp},
    semantic::{
        arbitraries::{ArbitraryFloatKind, NumericIntError},
        values::Value,
    },
};

pub enum UnaryOpResult {
    Output(Value),
    Invalid,
    /// User configured numeric limit
    NumericLimitExceeded,
}

pub enum BinaryOpResult {
    Output(Value),
    Invalid,
    DivideByZero,
    InvalidShift,
    /// User configured numeric limit
    NumericLimitExceeded,
}

fn value_fits_numeric_limit(value: &Value, max_numeric_bits: u64) -> bool {
    match value {
        Value::ArbitraryInt(value) => value.fits_numeric_bits(max_numeric_bits),
        Value::ArbitraryFloat(value) => value.fits_numeric_bits(max_numeric_bits),
        _ => true,
    }
}

// Is this the type checker's?
/// Evaluates if the given unary operation is possible given language rules
pub fn is_compatible_unary(op: UnaryOp, operand: &Value) -> bool {
    match op {
        UnaryOp::Not => match operand {
            Value::Bool(_) => true,
            Value::ArbitraryInt(_)
            | Value::ArbitraryFloat(_)
            | Value::Char(_)
            | Value::Tuple(_)
            | Value::InternedStr(_)
            | Value::RuntimeStr(_)
            | Value::Func(_)
            | Value::Array(_)
            | Value::Unknown => false,
        },
        UnaryOp::Negate => match operand {
            Value::ArbitraryInt(_) | Value::ArbitraryFloat(_) => true,
            _ => false,
        },
        UnaryOp::BitNot => match operand {
            Value::ArbitraryInt(_) => true,
            _ => false,
        },
    }
}

/// Evaluates if the given binary operation is possible given language rules
///
/// Mirrors `apply_binary_op`: an operand pair accepted here has a matching arm there, and one
/// rejected here has none. Both sides must be updated together.
pub fn is_compatible_binary(lhs: &Value, op: BinaryOp, rhs: &Value) -> bool {
    if let Value::RuntimeStr(_) = lhs {
        unreachable!("Impossible to reach at compile time")
    }

    match op {
        BinaryOp::Add => matches!(
            (lhs, rhs),
            (Value::ArbitraryInt(_), Value::ArbitraryInt(_))
                | (Value::ArbitraryFloat(_), Value::ArbitraryFloat(_))
                | (Value::InternedStr(_), Value::InternedStr(_))
        ),
        BinaryOp::Sub | BinaryOp::Mult | BinaryOp::Div | BinaryOp::Mod => matches!(
            (lhs, rhs),
            (Value::ArbitraryInt(_), Value::ArbitraryInt(_))
                | (Value::ArbitraryFloat(_), Value::ArbitraryFloat(_))
        ),
        BinaryOp::Greater | BinaryOp::Less | BinaryOp::GreaterOrEq | BinaryOp::LessOrEq => {
            matches!(
                (lhs, rhs),
                (Value::ArbitraryInt(_), Value::ArbitraryInt(_))
                    | (Value::ArbitraryFloat(_), Value::ArbitraryFloat(_))
                    | (Value::Char(_), Value::Char(_))
                    | (Value::InternedStr(_), Value::InternedStr(_))
            )
        }
        BinaryOp::And | BinaryOp::Or => matches!((lhs, rhs), (Value::Bool(_), Value::Bool(_))),
        BinaryOp::EqTo | BinaryOp::NotEq => matches!(
            (lhs, rhs),
            (Value::ArbitraryInt(_), Value::ArbitraryInt(_))
                | (Value::ArbitraryFloat(_), Value::ArbitraryFloat(_))
                | (Value::Bool(_), Value::Bool(_))
                | (Value::Char(_), Value::Char(_))
                | (Value::InternedStr(_), Value::InternedStr(_))
        ),
        BinaryOp::BitOr
        | BinaryOp::BitAnd
        | BinaryOp::BitRightShift
        | BinaryOp::BitLeftShift
        | BinaryOp::BitXor => {
            matches!((lhs, rhs), (Value::ArbitraryInt(_), Value::ArbitraryInt(_)))
        }
    }
}

pub fn apply_unary_op(op: UnaryOp, sp_operand: SpannedContainerRef<Value>) -> UnaryOpResult {
    apply_unary_op_with_limit(op, sp_operand, crate::DEFAULT_MAX_NUMERIC_BITS as u64)
}

pub fn apply_unary_op_with_limit(
    op: UnaryOp,
    sp_operand: SpannedContainerRef<Value>,
    max_numeric_bits: u64,
) -> UnaryOpResult {
    let operand = sp_operand.inner;
    if !value_fits_numeric_limit(operand, max_numeric_bits) {
        return UnaryOpResult::NumericLimitExceeded;
    }

    let res = match op {
        UnaryOp::Not => match operand {
            Value::Bool(v) => Some(Value::Bool(!v)),
            _ => None,
        },
        UnaryOp::Negate => match operand {
            Value::ArbitraryInt(kind) => Some(Value::ArbitraryInt(-kind)),
            // WARN: Suspicious
            Value::ArbitraryFloat(v) => Some(Value::ArbitraryFloat(-v)),
            _ => None,
        },
        UnaryOp::BitNot => match operand {
            Value::ArbitraryInt(v) => Some(Value::ArbitraryInt(!v)),
            _ => None,
        },
    };

    match res {
        Some(val) if !value_fits_numeric_limit(&val, max_numeric_bits) => {
            UnaryOpResult::NumericLimitExceeded
        }
        Some(val) => UnaryOpResult::Output(val),
        None => UnaryOpResult::Invalid,
    }
}

/// Applies operation assuming that lhs and rhs were checked for compatibility
//TODO: BIGFLOAT
pub fn apply_binary_op(
    sp_lhs: SpannedContainerRef<Value>,
    op: BinaryOp,
    sp_rhs: SpannedContainerRef<Value>,
    interner: &mut Intern,
) -> BinaryOpResult {
    apply_binary_op_with_limit(
        sp_lhs,
        op,
        sp_rhs,
        interner,
        (crate::DEFAULT_MAX_NUMERIC_BITS) as u64,
    )
}

pub fn apply_binary_op_with_limit(
    sp_lhs: SpannedContainerRef<Value>,
    op: BinaryOp,
    sp_rhs: SpannedContainerRef<Value>,
    interner: &mut Intern,
    max_numeric_bits: u64,
) -> BinaryOpResult {
    let lhs = sp_lhs.inner;
    let rhs = sp_rhs.inner;
    if !value_fits_numeric_limit(lhs, max_numeric_bits)
        || !value_fits_numeric_limit(rhs, max_numeric_bits)
    {
        return BinaryOpResult::NumericLimitExceeded;
    }
    // Checks if bits(a * b) >= bits(a) + bits(b) - 1, so rejects before
    // creating a potentially large BigInt. Zeros excluded since always valid.
    if let (BinaryOp::Mult, Value::ArbitraryInt(lhs), Value::ArbitraryInt(rhs)) = (op, lhs, rhs)
        && !lhs.is_zero()
        && !rhs.is_zero()
        && lhs.numeric_bits() + rhs.numeric_bits() - 1 > max_numeric_bits
    {
        return BinaryOpResult::NumericLimitExceeded;
    }

    let res = match op {
        BinaryOp::Add => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => Some(Value::ArbitraryInt(lhs_inner + rhs_inner)),
                _ => None,
            },
            Value::ArbitraryFloat(lhs_inner) => match rhs {
                Value::ArbitraryFloat(rhs_inner) => {
                    Some(Value::ArbitraryFloat(lhs_inner + rhs_inner))
                }
                _ => None,
            },
            Value::InternedStr(lhs_inner) => match rhs {
                Value::InternedStr(rhs_inner) => {
                    let l_str = interner.search(*lhs_inner);
                    let r_str = interner.search(*rhs_inner);
                    let new_str = l_str.to_string() + r_str;
                    let new_interned_id = interner.intern(&new_str);
                    Some(Value::InternedStr(new_interned_id))
                }
                _ => None,
            },
            _ => None,
        },
        BinaryOp::Sub => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => Some(Value::ArbitraryInt(lhs_inner - rhs_inner)),
                _ => None,
            },
            Value::ArbitraryFloat(lhs_inner) => match rhs {
                Value::ArbitraryFloat(rhs_inner) => {
                    Some(Value::ArbitraryFloat(lhs_inner - rhs_inner))
                }
                _ => None,
            },
            _ => None,
        },
        BinaryOp::Mult => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => Some(Value::ArbitraryInt(lhs_inner * rhs_inner)),
                _ => None,
            },
            Value::ArbitraryFloat(lhs_inner) => match rhs {
                Value::ArbitraryFloat(rhs_inner) => {
                    Some(Value::ArbitraryFloat(lhs_inner * rhs_inner))
                }
                _ => None,
            },
            _ => None,
        },
        BinaryOp::Div => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => {
                    if rhs_inner.is_zero() {
                        return BinaryOpResult::DivideByZero;
                    }

                    Some(Value::ArbitraryInt(lhs_inner / rhs_inner))
                }
                _ => None,
            },
            Value::ArbitraryFloat(lhs_inner) => match rhs {
                Value::ArbitraryFloat(rhs_inner) => {
                    if rhs_inner.is_zero() {
                        return BinaryOpResult::DivideByZero;
                    }

                    Some(Value::ArbitraryFloat(lhs_inner / rhs_inner))
                }
                _ => None,
            },
            _ => None,
        },
        BinaryOp::Greater => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => Some(Value::Bool(lhs_inner > rhs_inner)),
                _ => None,
            },
            Value::ArbitraryFloat(lhs_inner) => match rhs {
                Value::ArbitraryFloat(rhs_inner) => Some(Value::Bool(lhs_inner > rhs_inner)),
                _ => None,
            },
            Value::Char(lhs_inner) => match rhs {
                Value::Char(rhs_inner) => Some(Value::Bool(lhs_inner > rhs_inner)),
                _ => None,
            },
            Value::InternedStr(lhs_inner) => match rhs {
                Value::InternedStr(rhs_inner) => {
                    let l_str = interner.search(*lhs_inner);
                    let r_str = interner.search(*rhs_inner);
                    Some(Value::Bool(l_str > r_str))
                }
                _ => None,
            },
            _ => None,
        },
        BinaryOp::Less => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => Some(Value::Bool(lhs_inner < rhs_inner)),
                _ => None,
            },
            Value::ArbitraryFloat(lhs_inner) => match rhs {
                Value::ArbitraryFloat(rhs_inner) => Some(Value::Bool(lhs_inner < rhs_inner)),
                _ => None,
            },
            Value::Char(lhs_inner) => match rhs {
                Value::Char(rhs_inner) => Some(Value::Bool(lhs_inner < rhs_inner)),
                _ => None,
            },
            Value::InternedStr(lhs_inner) => match rhs {
                Value::InternedStr(rhs_inner) => {
                    let l_str = interner.search(*lhs_inner);
                    let r_str = interner.search(*rhs_inner);
                    Some(Value::Bool(l_str < r_str))
                }
                _ => None,
            },
            _ => None,
        },
        BinaryOp::GreaterOrEq => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => Some(Value::Bool(lhs_inner >= rhs_inner)),
                _ => None,
            },
            Value::ArbitraryFloat(lhs_inner) => match rhs {
                Value::ArbitraryFloat(rhs_inner) => Some(Value::Bool(lhs_inner >= rhs_inner)),
                _ => None,
            },
            Value::Char(lhs_inner) => match rhs {
                Value::Char(rhs_inner) => Some(Value::Bool(lhs_inner >= rhs_inner)),
                _ => None,
            },
            Value::InternedStr(lhs_inner) => match rhs {
                Value::InternedStr(rhs_inner) => {
                    let l_str = interner.search(*lhs_inner);
                    let r_str = interner.search(*rhs_inner);
                    Some(Value::Bool(l_str >= r_str))
                }
                _ => None,
            },
            _ => None,
        },
        BinaryOp::LessOrEq => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => Some(Value::Bool(lhs_inner <= rhs_inner)),
                _ => None,
            },
            Value::ArbitraryFloat(lhs_inner) => match rhs {
                Value::ArbitraryFloat(rhs_inner) => Some(Value::Bool(lhs_inner <= rhs_inner)),
                _ => None,
            },
            Value::Char(lhs_inner) => match rhs {
                Value::Char(rhs_inner) => Some(Value::Bool(lhs_inner <= rhs_inner)),
                _ => None,
            },
            Value::InternedStr(lhs_inner) => match rhs {
                Value::InternedStr(rhs_inner) => {
                    let l_str = interner.search(*lhs_inner);
                    let r_str = interner.search(*rhs_inner);
                    Some(Value::Bool(l_str <= r_str))
                }
                _ => None,
            },
            _ => None,
        },
        BinaryOp::Mod => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => {
                    if rhs_inner.is_zero() {
                        return BinaryOpResult::DivideByZero;
                    }

                    Some(Value::ArbitraryInt(lhs_inner % rhs_inner))
                }
                _ => None,
            },
            Value::ArbitraryFloat(lhs_inner) => match rhs {
                Value::ArbitraryFloat(rhs_inner) => {
                    if rhs_inner.is_zero() {
                        // Preserve IEEE `f64` remainder semantics (`5.5 % 0.0`
                        // is `NaN`). `DBig` has no NaN representation and
                        // panics when the result would be NaN, so route
                        // zero-divisor cases through `f64`.
                        Some(Value::ArbitraryFloat(ArbitraryFloatKind::F64(
                            lhs_inner.to_f64() % rhs_inner.to_f64(),
                        )))
                    } else {
                        Some(Value::ArbitraryFloat(lhs_inner % rhs_inner))
                    }
                }
                _ => None,
            },
            _ => None,
        },
        BinaryOp::And => match lhs {
            Value::Bool(lhs_inner) => match rhs {
                Value::Bool(rhs_inner) => Some(Value::Bool(*lhs_inner && *rhs_inner)),
                _ => None,
            },
            _ => None,
        },
        BinaryOp::Or => match lhs {
            Value::Bool(lhs_inner) => match rhs {
                Value::Bool(rhs_inner) => Some(Value::Bool(*lhs_inner || *rhs_inner)),
                _ => None,
            },
            _ => None,
        },
        BinaryOp::EqTo => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => Some(Value::Bool(lhs_inner == rhs_inner)),
                _ => None,
            },
            Value::ArbitraryFloat(lhs_inner) => match rhs {
                Value::ArbitraryFloat(rhs_inner) => Some(Value::Bool(lhs_inner == rhs_inner)),
                _ => None,
            },
            Value::Bool(lhs_inner) => match rhs {
                Value::Bool(rhs_inner) => Some(Value::Bool(lhs_inner == rhs_inner)),
                _ => None,
            },
            Value::Char(lhs_inner) => match rhs {
                Value::Char(rhs_inner) => Some(Value::Bool(lhs_inner == rhs_inner)),
                _ => None,
            },
            Value::InternedStr(lhs_inner) => match rhs {
                Value::InternedStr(rhs_inner) => Some(Value::Bool(lhs_inner == rhs_inner)),
                _ => None,
            },
            _ => None,
        },
        BinaryOp::NotEq => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => Some(Value::Bool(lhs_inner != rhs_inner)),
                _ => None,
            },
            Value::ArbitraryFloat(lhs_inner) => match rhs {
                Value::ArbitraryFloat(rhs_inner) => Some(Value::Bool(lhs_inner != rhs_inner)),
                _ => None,
            },
            Value::Bool(lhs_inner) => match rhs {
                Value::Bool(rhs_inner) => Some(Value::Bool(lhs_inner != rhs_inner)),
                _ => None,
            },
            Value::Char(lhs_inner) => match rhs {
                Value::Char(rhs_inner) => Some(Value::Bool(lhs_inner != rhs_inner)),
                _ => None,
            },
            Value::InternedStr(lhs_inner) => match rhs {
                Value::InternedStr(rhs_inner) => Some(Value::Bool(lhs_inner != rhs_inner)),
                _ => None,
            },
            _ => None,
        },
        BinaryOp::BitOr => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => Some(Value::ArbitraryInt(lhs_inner | rhs_inner)),
                _ => None,
            },
            _ => None,
        },
        BinaryOp::BitAnd => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => Some(Value::ArbitraryInt(lhs_inner & rhs_inner)),
                _ => None,
            },
            _ => None,
        },
        BinaryOp::BitRightShift => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => {
                    match lhs_inner.checked_shr_with_limit(rhs_inner, max_numeric_bits) {
                        Ok(val) => Some(Value::ArbitraryInt(val)),
                        Err(NumericIntError::InvalidShift) => {
                            return BinaryOpResult::InvalidShift;
                        }
                        Err(NumericIntError::LimitExceeded) => {
                            return BinaryOpResult::NumericLimitExceeded;
                        }
                    }
                }
                _ => None,
            },
            _ => None,
        },
        BinaryOp::BitLeftShift => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => {
                    match lhs_inner.checked_shl_with_limit(rhs_inner, max_numeric_bits) {
                        Ok(val) => Some(Value::ArbitraryInt(val)),
                        Err(NumericIntError::InvalidShift) => {
                            return BinaryOpResult::InvalidShift;
                        }
                        Err(NumericIntError::LimitExceeded) => {
                            return BinaryOpResult::NumericLimitExceeded;
                        }
                    }
                }
                _ => None,
            },
            _ => None,
        },
        BinaryOp::BitXor => match lhs {
            Value::ArbitraryInt(lhs_inner) => match rhs {
                Value::ArbitraryInt(rhs_inner) => Some(Value::ArbitraryInt(lhs_inner ^ rhs_inner)),
                _ => None,
            },
            _ => None,
        },
    };

    match res {
        Some(val) if !value_fits_numeric_limit(&val, max_numeric_bits) => {
            BinaryOpResult::NumericLimitExceeded
        }
        Some(val) => BinaryOpResult::Output(val),
        None => BinaryOpResult::Invalid,
    }
}
