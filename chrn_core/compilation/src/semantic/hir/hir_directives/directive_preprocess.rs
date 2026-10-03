use chrn_utils::{id_types::InternedId, intern};
use lang::chrn_classifier::{ChrnClassifiable, ChrnClassified};

use crate::{
    lexer::token::{Token, TokenKind},
    resolvers::resolver_state::CompilerStage,
};
/// General directives not specific to anything
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// General directives not specific to anything
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectivePreprocessKind {
    Chrn,
}

static MAX_NUMERIC_BITS_SCHEMA: DirectivePreprocessSchema = DirectivePreprocessSchema::new(
    InternedId::new(intern::INTERNED_MAX_NUMERIC_BITS),
    DirectivePreprocessInput::Token(TokenKind::Integer),
    CompilerStage::Parser,
);

// More like input constraints but this is genuinely 900 characters lone
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

// Wow
pub struct DirectivePreprocessSchema {
    /// Identifier of `self`
    pub ident: InternedId,
    /// This
    pub input: DirectivePreprocessInput,
    /// Stage when the compiler has enough information for this option to be processed
    pub ready_stage: CompilerStage,
}

impl DirectivePreprocessSchema {
    pub const fn new(
        ident: InternedId,
        input: DirectivePreprocessInput,
        ready_stage: CompilerStage,
    ) -> Self {
        Self {
            ident,
            input,
            ready_stage,
        }
    }

    /// Attempts to convert `Token` into `DirectivePreprocessValue` given the constraints of `self`
    pub fn try_tok_as_val(&self, tok: &Token) -> Option<DirectivePreprocessField> {
        let tok_input = DirectivePreprocessInput::from_tok_kind(tok.kind());
        if !self.input.allows(&tok_input) {
            return None;
        }
        self.val_from_tok(tok)
        // Would need to do an (id, tok) conversion since identifier tells us what to target_
        // Only numeric bits exists so fine for now
        // match tok {
        //     Token::Integer(id, _) => Some(DirectivePreprocessValue::MaxNumericBits(id)),
        //     _ => None,
        // }
    }

    //TEST:
    fn val_from_tok(&self, tok: &Token) -> Option<DirectivePreprocessField> {
        match self.ident.id {
            // Feels like this should be another kind rather than identifier-based
            intern::INTERNED_MAX_NUMERIC_BITS => match tok {
                Token::Integer(id, _) => DirectivePreprocessField::MaxNumericBits(*id).into(),
                _ => None,
            },
            _ => None,
        }
    }
}

//TEST:
static DIRECTIVE_CHRN_FIELDS: [InternedId; 1] =
    [InternedId::new(intern::INTERNED_MAX_NUMERIC_BITS)];

impl DirectivePreprocessKind {
    /// Attempts to get the field of `ident` out of `self.kind`
    pub const fn get_field(&self, ident: InternedId) -> Option<&DirectivePreprocessSchema> {
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

/// General directives not specific to anything
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectivePreprocessField {
    /// Assumed to be `TokenInteger`
    MaxNumericBits(InternedId),
}
