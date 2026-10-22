use std::borrow::Cow;

use chrn_utils::{
    id_types::InternedId,
    intern::{self, Intern},
    utils::SharedU32,
};
use lang::chrn_classifier::{ChrnClassifiable, ChrnClassified};

use crate::{
    chrn_config::ChrnConfig,
    lexer::token::{Token, TokenKind},
    resolvers::resolver_state::CompilerStage,
    semantic::{
        arbitraries::ArbitraryIntKind,
        hir::{
            hir_directives::directive_preprocess::{
                consts::MAX_NUMERIC_BITS_SCHEMA,
                directive_preprocess_args::DirectivePreprocessArgLayout,
            },
            hir_exprs::ExprHir,
        },
        values::Value,
    },
};

mod consts;
pub mod directive_preprocess_args;

/// Target area to effect of a directive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectivePreprocessEffect {
    Compiler,
    // Module,
}

impl DirectivePreprocessEffect {
    pub fn from_preprocess_field(kind: DirectivePreprocessKind) -> Self {
        match kind {
            DirectivePreprocessKind::MaxNumericBits(_) => Self::Compiler,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DirectivePreprocessExpectInput<'a> {
    pub kind: DirectivePreprocessExpectInputKind<'a>,
    pub val_src: DirectivePreprocessValueSource,
}

impl<'a> DirectivePreprocessExpectInput<'a> {
    pub const fn new(
        kind: DirectivePreprocessExpectInputKind<'a>,
        val_src: DirectivePreprocessValueSource,
    ) -> Self {
        Self { val_src, kind }
    }

    /// Attempts to convert `Token` to `Self`
    pub fn from_tok(tok: &Token, interner: &Intern) -> Option<Self> {
        let val_src = DirectivePreprocessValueSource::from_tok_kind(tok.kind())?;
        let kind = match tok {
            Token::Integer(tok_int) => {
                let s = interner.extract_str_from_interned_id(tok_int.interned_id);
                let arbitrary = ArbitraryIntKind::from_str(s, tok_int.notation)?;
                DirectivePreprocessExpectInputKind::Integer(Cow::Owned(arbitrary))
            }
            _ => return None,
        };
        Some(Self::new(kind, val_src))
    }

    /// Attempts to convert `Value` to `Self`
    pub const fn from_val(val: &'a Value, expr_hir: &ExprHir) -> Option<Self> {
        let Some(val_src) = DirectivePreprocessValueSource::from_expr_hir(expr_hir) else {
            return None;
        };
        let kind = match val {
            Value::ArbitraryInt(int) => {
                DirectivePreprocessExpectInputKind::Integer(Cow::Borrowed(int))
            }
            _ => return None,
        };
        Some(Self::new(kind, val_src))
    }
}

// Depending on how constraints evolve this will be more specific to particular constraints rather
// than owned by the argument itself
/// Where the given value came from
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DirectivePreprocessValueSource {
    Literal,
    Var,
    BinaryOp,
    UnaryOp,
}

impl DirectivePreprocessValueSource {
    pub const fn from_tok_kind(kind: TokenKind) -> Option<Self> {
        let v = match kind {
            TokenKind::Char
            | TokenKind::BoolLiteral
            | TokenKind::Float
            | TokenKind::Integer
            | TokenKind::Str => Self::Literal,
            TokenKind::Id => Self::Var,
            // Not sure about keywords
            _ => return None,
        };
        Some(v)
    }

    pub const fn from_expr_hir(expr_hir: &ExprHir) -> Option<Self> {
        // I guess this is right?
        let s = match expr_hir {
            ExprHir::LiteralVal(_) => Self::Literal,
            ExprHir::Var(_) => Self::Var,
            ExprHir::Unary { .. } => Self::UnaryOp,
            ExprHir::BinaryExpr { .. } => Self::BinaryOp,
            // ExprHir::Call(expr_id, expr_ids) => todo!(),
            // ExprHir::Array(expr_ids) => todo!(),
            _ => return None,
        };
        Some(s)
    }
}

impl std::fmt::Display for DirectivePreprocessValueSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DirectivePreprocessValueSource::Literal => "Literal",
            DirectivePreprocessValueSource::Var => "Variable",
            // Not sure if these will actually be used because why would it want to restrict
            // something to only unary or binary?
            DirectivePreprocessValueSource::BinaryOp => "Binary op",
            DirectivePreprocessValueSource::UnaryOp => "Unary op",
        };
        write!(f, "{s}")
    }
}

// Input to be compared
/// Acts as the input to be put against constraints, as well as the value extracted
///
/// For example something like `max_numeric_bits` does checks against the integer it's given, then
/// uses it to put inside it's own parameter if valid.
#[derive(Debug, PartialEq, Clone)]
pub enum DirectivePreprocessExpectInputKind<'a> {
    //WARN: Maybe add arena where it stores the integer type because if a directive gets a fairly
    //big BigInt/Float it'll duplicate it here. Sad face.
    Integer(Cow<'a, ArbitraryIntKind>),
}

impl<'a> DirectivePreprocessExpectInputKind<'a> {
    // Testing `Self` gimmick
    /// Attempts to convert `Token` to `Self`
    pub fn from_tok(tok: &Token, interner: &Intern) -> Option<Self> {
        let input = match tok {
            Token::Integer(tok_int) => {
                let s = interner.extract_str_from_interned_id(tok_int.interned_id);
                let arbitrary = ArbitraryIntKind::from_str(s, tok_int.notation)?;
                DirectivePreprocessExpectInputKind::Integer(Cow::Owned(arbitrary))
            }
            _ => return None,
        };
        Some(input)
    }

    /// Attempts to convert `Value` to `Self`
    pub const fn from_val(val: &'a Value) -> Option<Self> {
        let input = match val {
            Value::ArbitraryInt(int) => {
                DirectivePreprocessExpectInputKind::Integer(Cow::Borrowed(int))
            }
            _ => return None,
        };
        Some(input)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DirectivePreprocessSchemaKind {
    MaxNumericBits,
}

impl DirectivePreprocessSchemaKind {
    pub const fn name_id(self) -> InternedId {
        match self {
            DirectivePreprocessSchemaKind::MaxNumericBits => {
                InternedId::new(intern::INTERNED_MAX_NUMERIC_BITS)
            }
        }
    }
    pub const fn schema(self) -> &'static DirectivePreprocessSchema {
        match self {
            DirectivePreprocessSchemaKind::MaxNumericBits => &MAX_NUMERIC_BITS_SCHEMA,
        }
    }
}

//TODO: Schemas have an `ArgLayout`, layouts have `[Arg]` which is position sensitive and
//contains what constraints the current argument should align with.
pub struct DirectivePreprocessSchema {
    /// Identifier of `self`
    pub kind: DirectivePreprocessSchemaKind,
    /// Layout of args expected by the schema
    pub arg_layout: DirectivePreprocessArgLayout,
    // More like effect level
    /// What compilation level this schema applies its effects to
    pub effect: DirectivePreprocessEffect,
    /// Stage when the compiler has enough information for this option to be processed
    pub ready_stage: CompilerStage,
}

impl DirectivePreprocessSchema {
    pub const fn new(
        kind: DirectivePreprocessSchemaKind,
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

    pub const fn try_from_interned_id(
        interned_id: InternedId,
    ) -> Option<&'static DirectivePreprocessSchema> {
        let schema = match interned_id.id {
            intern::INTERNED_MAX_NUMERIC_BITS => &MAX_NUMERIC_BITS_SCHEMA,
            _ => return None,
        };
        Some(schema)
    }

    //TODO: Naming unclear
    /// Attempts to convert inputs into `DirectivePreprocessKind` given the constraints
    /// of `Self`
    pub fn try_as_directive_preprocess(
        &self,
        inputs: &[DirectivePreprocessExpectInput],
    ) -> Option<DirectivePreprocessKind> {
        if inputs.len() != self.arg_layout.arg_len() {
            return None;
        }

        for (i, arg) in self.arg_layout.args().iter().enumerate() {
            let input_arg = &inputs[i];
            if arg.allows(input_arg).is_err() {
                return None;
            }
        }
        self.preprocess_kind_from_inputs(inputs)
    }

    //TEST:
    //Leaning towards this panicking and seeing wrong usage as developer error
    /// This is to be ran AFTER all constraints are checked.
    pub fn preprocess_kind_from_inputs(
        &self,
        inputs: &[DirectivePreprocessExpectInput],
    ) -> Option<DirectivePreprocessKind> {
        match self.kind {
            DirectivePreprocessSchemaKind::MaxNumericBits => {
                //WARN: Not sure if this should just return none if something fundamentally wrong
                //was given
                // debug_assert_eq!(inputs.len(), 1);
                if inputs.len() != self.arg_layout.arg_len() {
                    return None;
                }
                match &inputs[0].kind {
                    DirectivePreprocessExpectInputKind::Integer(cow_int) => {
                        // Ok but am I hallucinating
                        let num = match cow_int.as_ref() {
                            ArbitraryIntKind::I64(n)
                                if !n.is_negative() && *n <= (u32::MAX as i64) =>
                            {
                                *n as u32
                            }
                            ArbitraryIntKind::U64(n) if *n <= (u32::MAX as u64) => *n as u32,
                            _ => return None,
                        };
                        Some(DirectivePreprocessKind::MaxNumericBits(num))
                    }
                }
            }
        }
    }
}

///
#[derive(Debug, Clone, PartialEq)]
pub struct DirectivePreprocess {
    pub kind: DirectivePreprocessKind,
    pub effect: DirectivePreprocessEffect,
}

impl DirectivePreprocess {
    pub const fn name_id(&self) -> InternedId {
        let id = match self.kind {
            DirectivePreprocessKind::MaxNumericBits(_) => intern::INTERNED_MAX_NUMERIC_BITS,
        };
        InternedId::new(id)
    }
    pub const fn new(kind: DirectivePreprocessKind) -> Self {
        //TODO:
        let effect = match kind {
            DirectivePreprocessKind::MaxNumericBits(_) => DirectivePreprocessEffect::Compiler,
        };
        Self { kind, effect }
    }
}

/// General directives not specific to anything
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DirectivePreprocessKind {
    /// Assumed to be `TokenInteger`
    MaxNumericBits(u32),
}

impl DirectivePreprocessKind {
    /// Returns `true` if the given identifier is an existent directive.
    pub const fn exists(interned_id: InternedId) -> bool {
        match interned_id.id {
            intern::INTERNED_MAX_NUMERIC_BITS => true,
            _ => false,
        }
    }

    // Could be dynamic where sometimes it's preprocessed sometimes its n- why would I subject chrn
    // to that in the first place is the real question.
    /// Returns `true` if `Self` is a directive that is preprocessed
    pub const fn is_preprocessed(&self) -> bool {
        match self {
            DirectivePreprocessKind::MaxNumericBits(_) => true,
        }
    }

    /// Returns the schema that is needed to create `DirectivePreprocessKind`
    pub const fn schema(&self) -> &DirectivePreprocessSchema {
        match self {
            DirectivePreprocessKind::MaxNumericBits(_) => &MAX_NUMERIC_BITS_SCHEMA,
        }
    }
}

impl ChrnClassifiable for DirectivePreprocessKind {
    fn to_classified(&self) -> ChrnClassified {
        match self {
            //WARN: Should this be done? Seems too specific.
            DirectivePreprocessKind::MaxNumericBits(_) => ChrnClassified::DirectiveMaxNumericBits,
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
    directives: Vec<DirectivePreprocessKind>,
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
    pub fn push(&mut self, kind: DirectivePreprocessKind) {
        self.increment_from_kind(&kind);
        self.directives.push(kind);
    }

    /// Performs a swap remove, ensuring the stage count is decremented.
    pub fn swap_remove(&mut self, idx: usize) -> DirectivePreprocessKind {
        let kind = self.directives.swap_remove(idx);
        self.decrement_from_kind(&kind);
        kind
    }

    //TEST:
    fn increment_from_kind(&mut self, kind: &DirectivePreprocessKind) {
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

    fn decrement_from_kind(&mut self, kind: &DirectivePreprocessKind) {
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
    pub fn set_lexer_count(&mut self, val: u16) {
        self.mod_graph_lexer.set_right(val)
    }

    pub fn parser_count(&self) -> u16 {
        self.parser_name_resolver.left()
    }
    pub fn set_parser_count(&mut self, val: u16) {
        self.parser_name_resolver.set_left(val)
    }

    pub fn namespace_count(&self) -> u16 {
        self.parser_name_resolver.right()
    }
    pub fn set_namespace_count(&mut self, val: u16) {
        self.parser_name_resolver.set_right(val)
    }

    pub fn memb_count(&self) -> u16 {
        self.memb_ty_resolver.left()
    }
    pub fn set_memb_count(&mut self, val: u16) {
        self.memb_ty_resolver.set_left(val)
    }

    pub fn ty_count(&self) -> u16 {
        self.memb_ty_resolver.right()
    }
    pub fn set_ty_count(&mut self, val: u16) {
        self.memb_ty_resolver.set_right(val)
    }

    pub fn constraint_count(&self) -> u16 {
        self.constraint_resolver
    }
    pub fn set_constraint_count(&mut self, val: u16) {
        self.constraint_resolver = val;
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

// Needs to be elsewhere
/// Iterates through `directive_store` and applies it's effects where possible, given `stage`.
/// Whether a directive is processed successfully or not doesn't matter, both are discarded, the
/// only difference is that the success is applied.
pub fn apply_directives(
    stage: CompilerStage,
    directive_store: &mut DirectivePreprocessStore,
    cfg: &mut ChrnConfig,
) {
    let count = directive_store.stage_count(stage);
    if count == 0 {
        return;
    }

    //TEST: Make sure I work
    // Stores all indices marked for removal when the loop is done.
    let mut to_rm: Vec<usize> = Vec::with_capacity(count as usize);
    // Ok but you could just use to_rm.len()
    // Counts how many were found so the loop doesn't have to go through all elements by default.
    let mut found = 0;

    for (i, kind) in directive_store.directives.iter().enumerate() {
        if found == count {
            break;
        } else if kind.schema().ready_stage != stage {
            continue;
        }
        found += 1;
        to_rm.push(i);
        match kind {
            DirectivePreprocessKind::MaxNumericBits(new_bits) => {
                cfg.set_max_numeric_bits(*new_bits)
            }
        }
    }

    for idx in to_rm.iter().rev().copied() {
        directive_store.swap_remove(idx);
    }
}
