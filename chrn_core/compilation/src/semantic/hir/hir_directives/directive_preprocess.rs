//! Preprocessed directives are to be processed independently from what a stage like the lexer or
//! parser may do with the information. A failed preprocessed directive should only show it's
//! failure from compilation stages, never from the actual system applying directives.

use chrn_utils::{
    id_types::InternedId,
    intern::{self, Intern},
    utils::SharedU32,
};
mod consts;
mod directive_preprocess_args;
use lang::chrn_classifier::{ChrnClassifiable, ChrnClassified};

use crate::{
    chrn_config::ChrnConfig,
    lexer::token::{Token, TokenInt, TokenKind},
    resolvers::resolver_state::CompilerStage,
    semantic::hir::hir_directives::directive_preprocess::{
        consts::MAX_NUMERIC_BITS_SCHEMA,
        directive_preprocess_args::{
            DirectivePreprocessArg, DirectivePreprocessArgConstraint, DirectivePreprocessArgLayout,
        },
    },
};
/// General directives not specific to anything
#[derive(Debug, Clone, PartialEq)]
pub struct DirectivePreprocess {
    /// Which directive it's values should correspond to
    pub kind: DirectivePreprocessKind,
    /// Fields which correspond to `self.kind` metadata
    pub fields: Vec<DirectivePreprocessField>,
}

impl DirectivePreprocess {
    pub fn new(kind: DirectivePreprocessKind, fields: Vec<DirectivePreprocessField>) -> Self {
        Self { kind, fields }
    }

    pub const fn try_from_interned_str(interned_id: InternedId) -> Option<Self> {
        // None right now
        match interned_id {
            _ => None,
        }
    }
}

/// Compiler known preprocessed directives
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectivePreprocessKind {
    Chrn,
}

// impl DirectivePreprocessKind {
//     pub fn fields(&self) -> &[DirectivePreprocessFieldSchemaKind] {
//         match self {
//             DirectivePreprocessKind::Chrn => {
//
//             },
//         }
//     }
// }

/// Policy for what a directive applies it's effect to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectivePreprocessEffect {
    Compiler,
    // Module,
}

impl DirectivePreprocessEffect {
    pub fn from_preprocess_field(kind: DirectivePreprocessFieldKind) -> DirectivePreprocessEffect {
        match kind {
            DirectivePreprocessFieldKind::MaxNumericBits(_) => DirectivePreprocessEffect::Compiler,
        }
    }
}

pub enum DirectivePreprocessFieldConstraint {
    /// Type of value that can be used
    Input(DirectivePreprocessExpectInput),
}

// Input to be compared
/// Input constraints for `PreprocessDirectiveField`
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DirectivePreprocessExpectInput {
    Token(TokenKind),
}

// Input stored
/// Input constraints for `PreprocessDirectiveField`
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DirectivePreprocessInput<'a> {
    Token(&'a Token),
}

impl DirectivePreprocessExpectInput {
    /// Returns `true` if `given` is allowed by `self`
    pub fn allows(&self, given: &DirectivePreprocessExpectInput) -> bool {
        match (self, given) {
            (
                DirectivePreprocessExpectInput::Token(self_tok),
                DirectivePreprocessExpectInput::Token(given_tok),
            ) => self_tok == given_tok,
        }
    }

    /// Converts `TokenKind` to `DirectivePreprocessInput`
    pub fn from_tok_kind(kind: TokenKind) -> DirectivePreprocessExpectInput {
        DirectivePreprocessExpectInput::Token(kind)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DirectivePreprocessFieldSchemaKind {
    MaxNumericBits,
}

impl DirectivePreprocessFieldSchemaKind {
    pub const fn name_id(self) -> InternedId {
        match self {
            DirectivePreprocessFieldSchemaKind::MaxNumericBits => {
                InternedId::new(intern::INTERNED_MAX_NUMERIC_BITS)
            }
        }
    }
}

//TODO: Schemas have an `ArgLayout`, layouts have `[Arg]` which is position sensitive and
//contains what constraints the current argument should align with.
pub struct DirectivePreprocessFieldSchema {
    /// Identifier of `self`
    pub kind: DirectivePreprocessFieldSchemaKind,
    /// Layout of args expected by the schema
    pub arg_layout: DirectivePreprocessArgLayout,
    /// What compilation level this schema applies its effects to
    pub effect: DirectivePreprocessEffect,
    /// Stage when the compiler has enough information for this option to be processed
    pub ready_stage: CompilerStage,
}

impl DirectivePreprocessFieldSchema {
    pub const fn new(
        kind: DirectivePreprocessFieldSchemaKind,
        arg_layout: DirectivePreprocessArgLayout,
        effect: DirectivePreprocessEffect,
        ready_stage: CompilerStage,
    ) -> Self {
        Self {
            kind,
            arg_layout,
            effect,
            ready_stage,
        }
    }

    //TODO: Naming unclear
    /// Attempts to convert `Token` into `DirectivePreprocessFieldKind` given the constraints
    /// of `self`
    pub fn try_as_field_kind(
        &self,
        inputs: &[(DirectivePreprocessExpectInput, DirectivePreprocessInput)],
        // arg: &DirectivePreprocessArg,
    ) -> Option<DirectivePreprocessFieldKind> {
        if inputs.len() != self.arg_layout.arg_len() {
            return None;
        }

        for (i, arg) in self.arg_layout.args().iter().enumerate() {
            let input_arg = &inputs[i].0;
            for constraint in arg.constraints {
                if !constraint.allows(input_arg) {
                    return None;
                };
            }
        }
        // TODO: Clear from here all green clean governed
        let actual_inputs: Vec<DirectivePreprocessInput> = inputs.iter().map(|i| i.1).collect();
        self.field_kind_from_inputs(&actual_inputs)
    }

    //TEST:
    fn field_kind_from_inputs(
        &self,
        inputs: &[DirectivePreprocessInput],
    ) -> Option<DirectivePreprocessFieldKind> {
        match self.kind {
            DirectivePreprocessFieldSchemaKind::MaxNumericBits => {
                //WARN: Not sure if this should just return none if something fundamentally wrong
                //was given
                // debug_assert_eq!(inputs.len(), 1);
                if inputs.len() != self.arg_layout.arg_len() {
                    return None;
                }
                match inputs[0] {
                    DirectivePreprocessInput::Token(tok) => {
                        let Token::Integer(tok_int) = tok else {
                            return None;
                        };
                        DirectivePreprocessFieldKind::MaxNumericBits(*tok_int).into()
                    }
                    _ => None,
                }
            }
        }
    }
}

impl DirectivePreprocessKind {
    /// Attempts to get the field of `ident` out of `self.kind`
    pub const fn get_field(&self, ident: InternedId) -> Option<&DirectivePreprocessFieldSchema> {
        match self {
            DirectivePreprocessKind::Chrn => match ident.id {
                intern::INTERNED_MAX_NUMERIC_BITS => Some(&MAX_NUMERIC_BITS_SCHEMA),
                _ => None,
            },
        }
    }

    /// Attempts to convert to `Self` using `InternedId`
    pub fn try_from_interned_str(interned_id: InternedId) -> Option<Self> {
        let kind = match interned_id.id {
            intern::INTERNED_CHRN => DirectivePreprocessKind::Chrn,
            _ => return None,
        };
        Some(kind)
    }
}

impl ChrnClassifiable for DirectivePreprocessKind {
    fn to_classified(&self) -> ChrnClassified {
        match self {
            DirectivePreprocessKind::Chrn => ChrnClassified::Chrn,
        }
    }
}

///
#[derive(Debug, Clone, PartialEq)]
pub struct DirectivePreprocessField {
    pub kind: DirectivePreprocessFieldKind,
    pub effect: DirectivePreprocessEffect,
}

impl DirectivePreprocessField {
    pub const fn new(kind: DirectivePreprocessFieldKind) -> Self {
        //TODO:
        let effect = match kind {
            DirectivePreprocessFieldKind::MaxNumericBits(_) => DirectivePreprocessEffect::Compiler,
        };
        Self { kind, effect }
    }
}

/// General directives not specific to anything
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DirectivePreprocessFieldKind {
    /// Assumed to be `TokenInteger`
    MaxNumericBits(TokenInt),
}

impl DirectivePreprocessFieldKind {
    pub const fn schema(&self) -> &DirectivePreprocessFieldSchema {
        match self {
            DirectivePreprocessFieldKind::MaxNumericBits(_) => &MAX_NUMERIC_BITS_SCHEMA,
        }
    }
}

// This could just be one bit-wise where it only asks if a stage exists rather than keeping count,
// but still trying out this form.
#[derive(Debug)]
pub struct DirectivePreprocessStore {
    // WARN: Maybe just rm mod_graph because it will probably remain unreachable. Forever.
    mod_graph_lexer: SharedU32,
    /// Left: Lexer | Right: Parser
    parser_name_resolver: SharedU32,
    /// Left: NameResolver | Right: TypeResolver
    memb_ty_resolver: SharedU32,
    /// Left: TypeResolver | Right: ConstraintResolver
    constraint_resolver: u16,
    /// Directives
    directives: Vec<DirectivePreprocessFieldKind>,
}

impl DirectivePreprocessStore {
    pub const fn new() -> Self {
        Self {
            mod_graph_lexer: SharedU32::zeroed(),
            parser_name_resolver: SharedU32::zeroed(),
            memb_ty_resolver: SharedU32::zeroed(),
            constraint_resolver: 0,
            directives: Vec::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            mod_graph_lexer: SharedU32::zeroed(),
            parser_name_resolver: SharedU32::zeroed(),
            memb_ty_resolver: SharedU32::zeroed(),
            constraint_resolver: 0,
            directives: Vec::with_capacity(cap),
        }
    }

    /// Pushes into `directives`, ensuring the stage count is incremented
    pub fn push(&mut self, kind: DirectivePreprocessFieldKind) {
        self.increment_from_kind(&kind);
        self.directives.push(kind);
    }

    /// Performs a swap remove, ensuring the stage count is decremented.
    pub fn swap_remove(&mut self, idx: usize) -> DirectivePreprocessFieldKind {
        let kind = self.directives.swap_remove(idx);
        self.decrement_from_kind(&kind);
        kind
    }

    //TEST:
    fn increment_from_kind(&mut self, kind: &DirectivePreprocessFieldKind) {
        match kind.schema().ready_stage {
            CompilerStage::ModuleGraph => unreachable!(),
            CompilerStage::Lexer => self.mod_graph_lexer.add_right(1),
            CompilerStage::Parser => self.parser_name_resolver.add_left(1),
            CompilerStage::Namespace => todo!(),
            CompilerStage::Member => todo!(),
            CompilerStage::Type => todo!(),
            CompilerStage::Constraint => todo!(),
            CompilerStage::Complete => todo!(),
        }
    }

    fn decrement_from_kind(&mut self, kind: &DirectivePreprocessFieldKind) {
        // Maybe we just swap comp stage to enum
        match kind.schema().ready_stage {
            CompilerStage::ModuleGraph => todo!(),
            CompilerStage::Lexer => self.mod_graph_lexer.sub_right(1),
            CompilerStage::Parser => self.parser_name_resolver.sub_left(1),
            CompilerStage::Namespace => todo!(),
            CompilerStage::Member => todo!(),
            CompilerStage::Type => todo!(),
            CompilerStage::Constraint => todo!(),
            CompilerStage::Complete => todo!(),
        }
    }

    pub fn lexer_count(&self) -> u16 {
        self.mod_graph_lexer.right()
    }

    pub fn parser_count(&self) -> u16 {
        self.parser_name_resolver.left()
    }

    pub fn namespace_count(&self) -> u16 {
        self.parser_name_resolver.right()
    }

    pub fn memb_count(&self) -> u16 {
        self.memb_ty_resolver.left()
    }

    pub fn ty_count(&self) -> u16 {
        self.memb_ty_resolver.right()
    }

    pub fn constraint_count(&self) -> u16 {
        self.constraint_resolver
    }

    /// Returns count of how many of the given `stage` exists
    pub fn stage_count(&self, stage: CompilerStage) -> u16 {
        match stage {
            CompilerStage::ModuleGraph => unreachable!(),
            CompilerStage::Lexer => self.lexer_count(),
            CompilerStage::Parser => self.parser_count(),
            CompilerStage::Namespace => self.namespace_count(),
            CompilerStage::Member => self.memb_count(),
            CompilerStage::Type => self.ty_count(),
            CompilerStage::Constraint => self.constraint_count(),
            CompilerStage::Complete => 0,
        }
    }

    /// Returns `Some` amount of the stage given
    pub fn has_stage(&self, stage: CompilerStage) -> bool {
        self.stage_count(stage) > 0
    }
}

//TEST: :(
/// Iterates through `directive_store` and applies it's effects where possible, given `stage`.
/// If a directive is processed successfully or fails, it is discarded.
pub fn apply_directives(
    stage: CompilerStage,
    directive_store: &mut DirectivePreprocessStore,
    cfg: &mut ChrnConfig,
    interner: &Intern,
) {
    let count = directive_store.stage_count(stage);
    if count == 0 {
        return;
    }

    // Small vecccc
    let mut to_rm: Vec<usize> = Vec::new();
    let mut found = 0;

    for (i, kind) in directive_store.directives.iter().enumerate() {
        if found == count {
            break;
        } else if kind.schema().ready_stage != stage {
            continue;
        }
        found += 1;
        match kind {
            DirectivePreprocessFieldKind::MaxNumericBits(tok_int) => {
                let s = interner.search(tok_int.interned_id);
                //This can only fail from overflow
                let Ok(new_bits) = u32::from_str_radix(s, tok_int.notation.radix()) else {
                    to_rm.push(i);
                    continue;
                };
                cfg.set_max_numeric_bits(new_bits);
            }
        }
    }

    for idx in to_rm {
        directive_store.swap_remove(idx);
    }
}
