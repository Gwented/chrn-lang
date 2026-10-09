use crate::semantic::hir::hir_directives::DirectivePreprocessExpectInput;

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
    /// Returns `true` if given `Token` is valid for `Self`, `false` otherwise
    pub fn allows(&self, input: &DirectivePreprocessExpectInput) -> bool {
        for constraint in self.constraints {
            if !constraint.allows(&input) {
                return false;
            };
        }
        true
    }
}

#[derive(Debug)]
pub enum DirectivePreprocessArgConstraint {
    Input(DirectivePreprocessExpectInput),
}

// This is nested because the intent of this isn't known yet. Input itself is still supposed to be a
// TYPE of constraint
impl DirectivePreprocessArgConstraint {
    //TODO: Not sure if the `Input` abstraction is really what given will remain as
    /// Returns `true` if `given` is allowed by `self`
    pub fn allows(&self, given: &DirectivePreprocessExpectInput) -> bool {
        match self {
            DirectivePreprocessArgConstraint::Input(self_input) => self_input.allows(given),
        }
    }
}
