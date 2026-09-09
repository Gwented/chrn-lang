use lang::chrn_classifier::ChrnClassifiable;

use crate::semantic::hir::{hir_concepts::TypeKind, hir_symbols::SymbolKindFlat};

// This is here because the typechecker will probably use this instead of just being for preset err
/// More dynamic way of expecting a certain kind without specifically picking a type like
/// `SymbolKindFlat`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExpectedKind {
    Symbol(SymbolKindFlat),
    Type(ExpectedKindType),
}

/// More dynamic way of expecting a certain type.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExpectedKindType {
    AnyBuiltin,
    Kind(TypeKind),
}

impl From<SymbolKindFlat> for ExpectedKind {
    fn from(kind: SymbolKindFlat) -> Self {
        ExpectedKind::Symbol(kind)
    }
}

impl From<TypeKind> for ExpectedKind {
    fn from(kind: TypeKind) -> Self {
        ExpectedKind::Type(kind.into())
    }
}

impl From<TypeKind> for ExpectedKindType {
    fn from(kind: TypeKind) -> Self {
        ExpectedKindType::Kind(kind)
    }
}

impl ExpectedKind {
    pub fn to_string(self) -> String {
        match self {
            ExpectedKind::Symbol(flat) => flat.to_classified().to_string(),
            ExpectedKind::Type(kind) => kind.to_string(),
        }
    }
}

impl ExpectedKindType {
    pub fn to_string(self) -> String {
        match self {
            // Should this be `chrn type`?
            ExpectedKindType::AnyBuiltin => "builtin type".to_string(),
            ExpectedKindType::Kind(kind) => kind.to_classified().to_string(),
        }
    }
}
