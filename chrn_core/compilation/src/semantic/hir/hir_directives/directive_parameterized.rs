//! Preprocessed directives are to be processed independently from what a stage like the lexer or
//! parser may do with the information. A failed preprocessed directive should only show it's
//! failure from compilation stages, never from the actual system applying directives.

use chrn_utils::{
    id_types::{ExprId, InternedId},
    utils::containers::SpannedContainer,
};
//// General directives not specific to anything
// #[derive(Debug, Clone)]
// pub struct DirectivePreprocess {
//     /// Spanned identifier of `self`
//     pub kind: DirectivePreprocessKind,
//     pub fields: SpannedContainer<SpannedContainer<DirectivePreprocess>>,
// }

// May just do `DirectiveForm` instead of the separation now that the design is clearer
//
// Although storing the schema upon success would make the directive being valid clearer, the
// padding is unneccessary so it should just be extracted.
/// Parameterized directives, which are only at compile-time
#[derive(Debug, Clone)]
pub struct DirectiveParameterizedHir {
    pub sp_name_id: SpannedContainer<InternedId>,
    // Is spanned so that it at least captures the "()" if there are no args for err reporting
    /// Which directive it's fields should correspond to
    pub args: Option<SpannedContainer<Vec<ExprId>>>,
    // pub is_preprocessed: bool,
}

impl DirectiveParameterizedHir {
    pub fn new(
        sp_name_id: SpannedContainer<InternedId>,
        args: Option<SpannedContainer<Vec<ExprId>>>,
    ) -> Self {
        Self { sp_name_id, args }
    }

    pub const fn try_from_interned_str(interned_id: InternedId) -> Option<Self> {
        // None right now
        match interned_id {
            _ => None,
        }
    }
}
