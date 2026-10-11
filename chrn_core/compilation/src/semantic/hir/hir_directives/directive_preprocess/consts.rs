use crate::{
    lexer::token::TokenKind,
    resolvers::resolver_state::CompilerStage,
    semantic::hir::hir_directives::{
        DirectivePreprocessEffect, DirectivePreprocessExpectInput, DirectivePreprocessSchema,
        DirectivePreprocessSchemaKind,
        directive_preprocess::directive_preprocess_args::{
            DirectivePreprocessArg, DirectivePreprocessArgConstraint, DirectivePreprocessArgLayout,
        },
    },
};
pub(super) static MAX_NUMERIC_BITS_SCHEMA: DirectivePreprocessSchema =
    DirectivePreprocessSchema::new(
        DirectivePreprocessSchemaKind::MaxNumericBits,
        DirectivePreprocessArgLayout::new(&MAX_NUMERIC_BITS_ARGS),
        DirectivePreprocessEffect::Compiler,
        CompilerStage::Lexer,
    );

// Is there a preference for if these should be ref or not
//
// Need a better way of instantiation
//TODO: This needs to be given an abstract "Integer" idea.
//Meaning, Token::Integer and Type::I64/U64/BigInt understand what `Input::Integer` means in their
//own way.
pub(super) static MAX_NUMERIC_BITS_ARGS: [DirectivePreprocessArg; 1] = [
    DirectivePreprocessArg::new(&[DirectivePreprocessArgConstraint::Input(
        DirectivePreprocessExpectInput::Token(TokenKind::Integer),
    )]),
];
