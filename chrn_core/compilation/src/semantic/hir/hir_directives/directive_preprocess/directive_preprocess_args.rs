use std::ops::Range;

use crate::semantic::{
    arbitraries::ArbitraryIntKind, hir::hir_directives::DirectivePreprocessExpectInput,
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

// Technically called parameters
#[derive(Debug)]
pub struct DirectivePreprocessArg {
    pub constraints: &'static [DirectivePreprocessArgConstraint],
}

impl DirectivePreprocessArg {
    pub const fn new(constraints: &'static [DirectivePreprocessArgConstraint]) -> Self {
        Self { constraints }
    }
    /// Returns `Ok` if the given input is valid for `Self`.
    /// Returns `Err` and the constraint index failed at on failure
    pub fn allows(&self, input: &DirectivePreprocessExpectInput) -> Result<(), usize> {
        for (i, constraint) in self.constraints.iter().enumerate() {
            if !constraint.allows(&input) {
                return Err(i);
            };
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct DirectiveIntegerArgConstraint {
    // This is load bearing, red, NOT governed
    /// Range the given integer must fall into
    pub range: Option<Range<u64>>,
}

impl DirectiveIntegerArgConstraint {
    pub const fn new(range: Option<Range<u64>>) -> Self {
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
    pub fn allows(&self, given: &DirectivePreprocessExpectInput) -> bool {
        match (self, given) {
            (
                DirectivePreprocessArgConstraint::Integer(constraint_int),
                DirectivePreprocessExpectInput::Integer(given_int),
            ) => {
                if let Some(range) = &constraint_int.range {
                    //method maybe
                    let num = match given_int.as_ref() {
                        ArbitraryIntKind::I64(n) => *n as u64,
                        ArbitraryIntKind::U64(n) => *n,
                        ArbitraryIntKind::BigInt(_) => return false,
                    };
                    if !range.contains(&num) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl std::fmt::Display for DirectivePreprocessArgConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // No constraints at all should probably
            // Also the formatting is not final
            DirectivePreprocessArgConstraint::Integer(int) => {
                let mut s = "Integer".to_string();

                // Maybe we can have a formatter where it turns a range into the nearest bit range
                if let Some(range) = &int.range {
                    s.push_str(&format!(" | range: {}..{}", range.start, range.end - 1));
                };
                write!(f, "{s}")
            }
        }
    }
}
