use crate::{
    lexer::token::TokenKind,
    resolvers::resolver_state::CompilerStage,
    semantic::hir::hir_directives::{
        DirectivePreprocessEffect, DirectivePreprocessExpectInput, DirectivePreprocessFieldSchema,
        DirectivePreprocessFieldSchemaKind,
        directive_preprocess::directive_preprocess_args::{
            DirectivePreprocessArg, DirectivePreprocessArgConstraint, DirectivePreprocessArgLayout,
        },
    },
};

// Is there a preference for if these should be ref or not
//
// Need a better way of instantiation
pub(super) static MAX_NUMERIC_BITS_ARGS: [DirectivePreprocessArg; 1] = [
    DirectivePreprocessArg::new(&[DirectivePreprocessArgConstraint::Input(
        DirectivePreprocessExpectInput::Token(TokenKind::Integer),
    )]),
];

pub(super) static MAX_NUMERIC_BITS_SCHEMA: DirectivePreprocessFieldSchema =
    DirectivePreprocessFieldSchema::new(
        // Maybe shouldn't be identifier mainly at least
        DirectivePreprocessFieldSchemaKind::MaxNumericBits,
        //TODO: ArgConstraint concept but here.
        //What if there was a concept of match Fixed | Dynamic where instead of pushing specific
        //argument placements inside dynamic, we make it fixed. But then what if fixed wants dynamic
        //inside of itself? Maybe Fixed is inside dynamic where it's a nested enum? Could be taking
        //too far.
        DirectivePreprocessArgLayout::new(&MAX_NUMERIC_BITS_ARGS),
        DirectivePreprocessEffect::Compiler,
        CompilerStage::Lexer,
    );

//TEST:
static DIRECTIVE_CHRN_FIELDS: [DirectivePreprocessFieldSchemaKind; 1] =
    [DirectivePreprocessFieldSchemaKind::MaxNumericBits];
