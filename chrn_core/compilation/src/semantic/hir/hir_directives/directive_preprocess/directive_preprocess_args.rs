use std::ops::Range;

use crate::semantic::{
    arbitraries::ArbitraryIntKind,
    hir::{
        hir_concepts::IntegerRangeKind,
        hir_directives::{
            DirectivePreprocessExpectInput, DirectivePreprocessExpectInputKind,
            DirectivePreprocessValueSource,
        },
    },
};

// In the case of metadata
/// `Arg` arena which contains metadata
#[derive(Debug, Default)]
pub struct DirectivePreprocessArgLayout {
    /// Argggp
    args: &'static [DirectivePreprocessArg],
}

impl DirectivePreprocessArgLayout {
    pub const fn new(args: &'static [DirectivePreprocessArg]) -> Self {
        Self { args }
    }

    pub const fn args(&self) -> &'static [DirectivePreprocessArg] {
        self.args
    }

    // May use different values
    pub const fn arg_len(&self) -> usize {
        self.args.len()
    }
}

#[derive(Debug)]
pub enum DirectivePreprocessConstraintErrorKind {
    ValueSourceMismatch {
        expected: DirectivePreprocessValueSource,
        found: DirectivePreprocessValueSource,
    },
    Integer(DirectiveIntegerArgConstraintErrorKind),
    // Better name please
    // Expect(ExpectedKind),
}

impl From<DirectiveIntegerArgConstraintErrorKind> for DirectivePreprocessConstraintErrorKind {
    fn from(v: DirectiveIntegerArgConstraintErrorKind) -> Self {
        DirectivePreprocessConstraintErrorKind::Integer(v)
    }
}

#[derive(Debug)]
pub enum DirectiveIntegerArgConstraintErrorKind {
    RangeMismatch { expected: IntegerRangeKind },
}

#[derive(Debug)]
pub struct DirectivePreprocessArg {
    pub constraints: &'static [DirectivePreprocessArgConstraint],
    pub val_src_constraint: DirectivePreprocessValueSource,
}

impl DirectivePreprocessArg {
    pub const fn new(
        val_src_constraint: DirectivePreprocessValueSource,
        constraints: &'static [DirectivePreprocessArgConstraint],
    ) -> Self {
        Self {
            val_src_constraint,
            constraints,
        }
    }

    // TODO: Maybe we need domain-specific enums
    /// Returns `Ok` if the given input is valid for `Self`.
    /// Returns `Err` and the constraint index failed at on failure
    pub fn allows(
        &self,
        input: &DirectivePreprocessExpectInput,
    ) -> Result<(), DirectivePreprocessConstraintErrorKind> {
        if self.val_src_constraint != input.val_src {
            return Err(
                DirectivePreprocessConstraintErrorKind::ValueSourceMismatch {
                    expected: self.val_src_constraint,
                    found: input.val_src,
                },
            );
        };

        for constraint in self.constraints {
            constraint.allows(&input.kind)?;
        }
        Ok(())
    }
}

//TEST:
#[derive(Debug)]
pub struct DirectiveValueSourceArgConstraint {
    // This is load bearing, red, NOT governed
    /// Range the given integer must fall into
    pub expected_src: Option<DirectivePreprocessValueSource>,
}

//It gets to ap oint it gets to a poitn it get sto  a pa oint it get s tpoi a point
//Ok.
// #[derive(Debug)]
// pub enum DirectiveValueSourceArgConstraintKind {
//     Any,
//     Enforced(ValueSource),
// }

// We could split this into signed and unsigned, if, wanted. Smiley face.
#[derive(Debug)]
pub struct DirectiveIntegerArgConstraint {
    // This is load bearing, red, NOT governed
    /// Range the given integer must fall into
    pub range: Option<IntegerRangeKind>,
}

impl DirectiveIntegerArgConstraint {
    pub const fn new(range: Option<IntegerRangeKind>) -> Self {
        Self { range }
    }
}

#[derive(Debug)]
pub enum DirectivePreprocessArgConstraint {
    Integer(DirectiveIntegerArgConstraint),
}

// This is nested because the intent of this isn't known yet. Input itself is still supposed to be a
// TYPE of constraint
impl DirectivePreprocessArgConstraint {
    //TODO: Not sure if the `Input` abstraction is really what given will remain as
    /// Returns `true` if `given` is allowed by `self`
    pub fn allows(
        &self,
        given: &DirectivePreprocessExpectInputKind,
    ) -> Result<(), DirectivePreprocessConstraintErrorKind> {
        match (self, given) {
            (
                DirectivePreprocessArgConstraint::Integer(constraint_int),
                DirectivePreprocessExpectInputKind::Integer(given_int),
            ) => {
                if let Some(range) = &constraint_int.range {
                    let num = match given_int.as_ref() {
                        ArbitraryIntKind::I64(n) => i128::from(*n),
                        ArbitraryIntKind::U64(n) => i128::from(*n),
                        ArbitraryIntKind::BigInt(_) => {
                            return Err(DirectiveIntegerArgConstraintErrorKind::RangeMismatch {
                                expected: range.clone(),
                            }
                            .into());
                        }
                    };
                    if !range.range().contains(&num) {
                        return Err(DirectiveIntegerArgConstraintErrorKind::RangeMismatch {
                            expected: range.clone(),
                        }
                        .into());
                    }
                }
            }
        }
        Ok(())
    }
}
