use chrn_utils::{
    id_types::InternedIdUTF8, source_map::source_span::SourceSpan,
    utils::containers::SpannedContainer,
};

use crate::{
    lexer::notations::{FloatNotation, IntegerNotation},
    parser::ast::ast_concepts::{AbstractMemberAccess, BinaryOp, Unary},
};

// This could look better...
// Does this need a literal specific variant?
#[derive(Debug)]
pub enum AstExpr {
    Var(InternedIdUTF8),
    /// `::`
    StaticAccess(Vec<SpannedContainer<PathSegment>>),
    Bool(bool),
    /// Variable name, along with optional default type
    Default(
        Box<SpannedContainer<AstExpr>>,
        Box<SpannedContainer<AstExpr>>,
    ),
    Integer(InternedIdUTF8, IntegerNotation),
    Float(InternedIdUTF8, FloatNotation),
    Str(InternedIdUTF8),
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
    pub(crate) name_id: InternedIdUTF8,
    pub(crate) spanned_expr: Vec<SpannedContainer<AstExpr>>,
}

impl CallExpr {
    pub(crate) fn new(
        name_id: InternedIdUTF8,
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
    Var(InternedIdUTF8),
    Path(Vec<SpannedContainer<PathSegment>>),
    Generic(AbstractGeneric),
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    Ident(InternedIdUTF8),
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
    pub base: InternedIdUTF8,
    // Change to tuple or something alike since max 2?
    pub inputs: Vec<SpannedContainer<TypeExpr>>,
}

impl AbstractGeneric {
    pub fn new(base: InternedIdUTF8, inputs: Vec<SpannedContainer<TypeExpr>>) -> AbstractGeneric {
        AbstractGeneric { base, inputs }
    }
}
