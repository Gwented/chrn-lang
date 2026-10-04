use chrn_utils::{
    id_types::InternedId,
    intern::{self, Intern},
    utils::SharedU32,
};
use lang::chrn_classifier::{ChrnClassifiable, ChrnClassified};

use crate::{
    chrn_config::ChrnConfig,
    lexer::token::{Token, TokenInt, TokenKind},
    resolvers::resolver_state::CompilerStage,
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
    pub fn from_preprocess_kind(kind: DirectivePreprocessFieldKind) -> DirectivePreprocessEffect {
        match kind {
            DirectivePreprocessFieldKind::MaxNumericBits(_) => DirectivePreprocessEffect::Compiler,
        }
    }
}

static MAX_NUMERIC_BITS_SCHEMA: DirectivePreprocessFieldSchema =
    DirectivePreprocessFieldSchema::new(
        // Maybe shouldn't be identifier mainly at least
        DirectivePreprocessFieldSchemaKind::MaxNumericBits,
        DirectivePreprocessInput::Token(TokenKind::Integer),
        DirectivePreprocessEffect::Compiler,
        CompilerStage::Lexer,
    );

// Lorax!
/// Input constraints for `PreprocessDirectiveField`
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DirectivePreprocessInput {
    Token(TokenKind),
}

impl DirectivePreprocessInput {
    /// Returns `true` if `given` is allowed by `self`
    pub fn allows(&self, given: &DirectivePreprocessInput) -> bool {
        match (self, given) {
            (
                DirectivePreprocessInput::Token(self_tok),
                DirectivePreprocessInput::Token(given_tok),
            ) => self_tok == given_tok,
        }
    }

    /// Converts `TokenKind` to `DirectivePreprocessInput`
    pub fn from_tok_kind(kind: TokenKind) -> DirectivePreprocessInput {
        DirectivePreprocessInput::Token(kind)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DirectivePreprocessFieldSchemaKind {
    MaxNumericBits,
}

impl DirectivePreprocessFieldSchemaKind {
    pub fn name_id(self) -> InternedId {
        match self {
            DirectivePreprocessFieldSchemaKind::MaxNumericBits => {
                InternedId::new(intern::INTERNED_MAX_NUMERIC_BITS)
            }
        }
    }
}

// Wow
pub struct DirectivePreprocessFieldSchema {
    /// Identifier of `self`
    pub kind: DirectivePreprocessFieldSchemaKind,
    /// This
    pub input: DirectivePreprocessInput,
    /// What compilation level this schema applies its effects to
    pub effect: DirectivePreprocessEffect,
    /// Stage when the compiler has enough information for this option to be processed
    pub ready_stage: CompilerStage,
}

impl DirectivePreprocessFieldSchema {
    pub const fn new(
        kind: DirectivePreprocessFieldSchemaKind,
        input: DirectivePreprocessInput,
        effect: DirectivePreprocessEffect,
        ready_stage: CompilerStage,
    ) -> Self {
        Self {
            kind,
            input,
            effect,
            ready_stage,
        }
    }

    //TODO: Naming unclear
    /// Attempts to convert `Token` into `DirectivePreprocessValue` given the constraints of `self`
    pub fn try_tok_as_field_kind(&self, tok: &Token) -> Option<DirectivePreprocessFieldKind> {
        let tok_input = DirectivePreprocessInput::from_tok_kind(tok.kind());
        if !self.input.allows(&tok_input) {
            return None;
        }
        self.val_from_tok(tok)
    }

    //TEST:
    fn val_from_tok(&self, tok: &Token) -> Option<DirectivePreprocessFieldKind> {
        match self.kind {
            // Feels like this should be another kind rather than identifier-based
            DirectivePreprocessFieldSchemaKind::MaxNumericBits => match tok {
                Token::Integer(tok_int) => {
                    DirectivePreprocessFieldKind::MaxNumericBits(*tok_int).into()
                }
                _ => None,
            },
        }
    }
}

//TEST:
static DIRECTIVE_CHRN_FIELDS: [DirectivePreprocessFieldSchemaKind; 1] =
    [DirectivePreprocessFieldSchemaKind::MaxNumericBits];

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

//TODO: Just iterating for now but will use more optimized form
#[derive(Debug)]
pub struct DirectivePreprocessStore {
    mod_graph_lexer: SharedU32,
    /// Left: Lexer | Right: Parser
    parser_name_resolver: SharedU32,
    /// Left: NameResolver | Right: TypeResolver
    memb_ty_resolver: SharedU32,
    /// Left: TypeResolver | Right: ConstraintResolver
    constraint_resolver: u16,
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

    pub fn push(&mut self, kind: DirectivePreprocessFieldKind) {
        self.increment_from_kind(&kind);
        self.directives.push(kind);
    }

    pub fn swap_remove(&mut self, idx: usize) -> DirectivePreprocessFieldKind {
        let kind = self.directives.swap_remove(idx);
        self.decrement_from_kind(&kind);
        kind
    }

    //TEST:
    fn increment_from_kind(&mut self, kind: &DirectivePreprocessFieldKind) {
        // Maybe we just swap comp stage to enum
        // Would need bit iter otherwise
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
            CompilerStage::Lexer => todo!(),
            CompilerStage::Parser => self.parser_name_resolver.sub_left(1),
            CompilerStage::Namespace => todo!(),
            CompilerStage::Member => todo!(),
            CompilerStage::Type => todo!(),
            CompilerStage::Constraint => todo!(),
            CompilerStage::Complete => todo!(),
        }
    }

    pub fn has_mod_graph(&self) -> bool {
        self.mod_graph_lexer.left() > 0
    }

    pub fn has_lexer(&self) -> bool {
        self.mod_graph_lexer.right() > 0
    }

    pub fn has_parser(&self) -> bool {
        self.parser_name_resolver.left() > 0
    }

    pub fn has_namespace(&self) -> bool {
        self.parser_name_resolver.right() > 0
    }

    pub fn has_memb(&self) -> bool {
        self.memb_ty_resolver.left() > 0
    }

    pub fn has_ty(&self) -> bool {
        self.memb_ty_resolver.right() > 0
    }

    pub fn has_constraint(&self) -> bool {
        self.constraint_resolver > 0
    }

    pub fn has_stage(&self, stage: CompilerStage) -> bool {
        match stage {
            CompilerStage::ModuleGraph => self.has_mod_graph(),
            CompilerStage::Lexer => self.has_lexer(),
            CompilerStage::Parser => self.has_parser(),
            CompilerStage::Namespace => self.has_namespace(),
            CompilerStage::Member => self.has_memb(),
            CompilerStage::Type => self.has_ty(),
            CompilerStage::Constraint => self.has_constraint(),
            CompilerStage::Complete => true,
        }
    }
}

//TEST: :(
pub fn apply_directives(
    stage: CompilerStage,
    directive_store: &mut DirectivePreprocessStore,
    cfg: &mut ChrnConfig,
    interner: &Intern,
) {
    if !directive_store.has_stage(stage) {
        return;
    }

    // Small vecccc
    let mut to_rm: Vec<usize> = Vec::new();

    for (i, kind) in directive_store.directives.iter().enumerate() {
        if kind.schema().ready_stage != stage {
            continue;
        }
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
