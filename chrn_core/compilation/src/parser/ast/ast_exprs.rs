use chrn_utils::{
    id_types::InternedId, source_map::source_span::SourceSpan, utils::containers::SpannedContainer,
};

use crate::{
    lexer::notations::{FloatNotation, IntegerNotation},
    parser::ast::ast_concepts::{AbstractMemberAccess, BinaryOp, Unary},
};

// This could look better...
// Does this need a literal specific variant?
#[derive(Debug)]
pub enum AstExpr {
    Var(InternedId),
    /// `::`
    StaticAccess(Vec<SpannedContainer<PathSegment>>),
    Bool(bool),
    /// Variable name, along with optional default type
    Default(
        Box<SpannedContainer<AstExpr>>,
        Box<SpannedContainer<AstExpr>>,
    ),
    Integer(InternedId, IntegerNotation),
    Float(InternedId, FloatNotation),
    Str(InternedId),
    Char(char),
    /// Caller, Args
    Call(
        Box<SpannedContainer<AstExpr>>,
        Vec<SpannedContainer<AstExpr>>,
    ),
    MemberAccess(AbstractMemberAccess),
    Unary(Unary),
    BinaryExpr {
        lhs: Box<SpannedContainer<AstExpr>>,
        op: BinaryOp,
        rhs: Box<SpannedContainer<AstExpr>>,
    },
    Array(ArrayExpr),
}

#[derive(Debug)]
pub(crate) struct CallExpr {
    pub(crate) name_id: InternedId,
    pub(crate) spanned_expr: Vec<SpannedContainer<AstExpr>>,
}

impl CallExpr {
    pub(crate) fn new(
        name_id: InternedId,
        spanned_expr: Vec<SpannedContainer<AstExpr>>,
    ) -> CallExpr {
        CallExpr {
            name_id,
            spanned_expr,
        }
    }
}

#[derive(Debug)]
pub struct ArrayExpr {
    pub elements: Vec<SpannedContainer<AstExpr>>,
}

impl ArrayExpr {
    pub fn new(elements: Vec<SpannedContainer<AstExpr>>) -> ArrayExpr {
        ArrayExpr { elements }
    }
}

#[derive(Debug, Clone)]
pub enum TypeExpr {
    Var(InternedId),
    Path(Vec<SpannedContainer<PathSegment>>),
    Generic(AbstractGeneric),
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    Ident(InternedId),
    Generic(AbstractGeneric),
}

// impl PathSegment {
//     pub fn as_type_expr_ref(&self) -> TypeExpr {
//         match self {
//             PathSegment::Ident(interned_id) => &TypeExpr::Var(*interned_id),
//             PathSegment::Generic(generic) => TypeExpr::Generic(&generic),
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct AbstractGeneric {
    pub base: InternedId,
    // Change to tuple or something alike since max 2?
    pub inputs: Vec<SpannedContainer<TypeExpr>>,
}

impl AbstractGeneric {
    pub fn new(base: InternedId, inputs: Vec<SpannedContainer<TypeExpr>>) -> AbstractGeneric {
        AbstractGeneric { base, inputs }
    }
}
