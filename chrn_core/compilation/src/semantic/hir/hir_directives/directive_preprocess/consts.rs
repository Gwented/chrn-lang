use crate::{
    resolvers::resolver_state::CompilerStage,
    semantic::hir::hir_directives::{
        DirectivePreprocessEffect, DirectivePreprocessSchema, DirectivePreprocessSchemaKind,
        DirectivePreprocessValueSource,
        directive_preprocess::directive_preprocess_args::{
            DirectiveIntegerArgConstraint, DirectivePreprocessArg,
            DirectivePreprocessArgConstraint, DirectivePreprocessArgLayout,
        },
    },
};
pub(super) static MAX_NUMERIC_BITS_SCHEMA: DirectivePreprocessSchema =
    DirectivePreprocessSchema::new(
        DirectivePreprocessSchemaKind::MaxNumericBits,
        DirectivePreprocessArgLayout::new(&MAX_NUMERIC_BITS_ARGS),
        DirectivePreprocessEffect::Compiler,
        // Should probably make it clear that the compiler stage is the stage in which it is
        // resolved AFTER its done. So, CompilerStage::Lexer means after the lexer, this directive
        // is ready to be applied.
        CompilerStage::Lexer,
    );

pub const MAX_NUMERIC_BITS_MAX_BIT_COUNT: u32 = u32::MAX;

// Not really helping..
const fn new_preprocess_arg(
    val_src: DirectivePreprocessValueSource,
    constraints: &'static [DirectivePreprocessArgConstraint],
) -> DirectivePreprocessArg {
    DirectivePreprocessArg::new(val_src, constraints)
}

// Is there a preference for if these should be ref or not
//
// Need a better way of instantiation
//TODO: This needs to be given an abstract "Integer" idea.
//Meaning, Token::Integer and Type::I64/U64/BigInt understand what `Input::Integer` means in their
//own way.
pub(super) static MAX_NUMERIC_BITS_ARGS: [DirectivePreprocessArg; 1] = [new_preprocess_arg(
    DirectivePreprocessValueSource::Literal,
    &[DirectivePreprocessArgConstraint::Integer(
        // Uhhh uhhh ummm is this uh
        DirectiveIntegerArgConstraint::new(Some(0..MAX_NUMERIC_BITS_MAX_BIT_COUNT as u64 + 1)),
    )],
)];
