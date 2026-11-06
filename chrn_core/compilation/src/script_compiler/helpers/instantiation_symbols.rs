use chrn_utils::id_types::{AstId, InternedIdUTF8, SymbolId};
use dashu_float::DBig;
use dashu_int::IBig;
use lang::types::{
    boundaries::TypeBoundaryFlags, builtins::BuiltinType, externs::ExternPlatformType,
};

use crate::{
    lookup::scopes::scopes_concepts::{AssociatedScopeKind, ScopeType},
    semantic::{
        arbitraries::{ArbitraryFloatKind, ArbitraryIntKind},
        hir::hir_symbols::{Symbol, SymbolKind, SymbolOrigin},
        values::Value,
    },
};

/// Allows for dynamic module ID determiniation based off environments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstantiationSymbolOrigin {
    /// Has no ID associated
    Compiler,
    /// Will be set to whatever `ModuleId` `core` lands on
    Core,
}

/// Abstraction to allow for `Symbol` to be made procedurally in a composed manner
#[derive(Debug)]
pub struct InstantiationSymbolBase {
    pub name_id: InternedIdUTF8,
    pub sym_origin: InstantiationSymbolOrigin,
    pub scope_origin: ScopeType,
    pub is_priv: bool,
    pub kind: InstantiationSymbolKind,
}

impl InstantiationSymbolBase {
    pub const fn new(
        name_id: InternedIdUTF8,
        sym_origin: InstantiationSymbolOrigin,
        scope_origin: ScopeType,
        is_priv: bool,
        kind: InstantiationSymbolKind,
    ) -> Self {
        Self {
            name_id,
            sym_origin,
            scope_origin,
            is_priv,
            kind,
        }
    }

    /// Helper to convert to symbol using the already present metadata
    pub fn to_sym(
        &self,
        sym_id: SymbolId,
        ast_id: Option<AstId>,
        associated_scope: Option<AssociatedScopeKind>,
        sym_origin: SymbolOrigin,
        kind: SymbolKind,
    ) -> Symbol {
        Symbol::new(
            self.name_id,
            sym_id,
            ast_id,
            sym_origin,
            self.is_priv,
            associated_scope,
            self.scope_origin,
            kind,
        )
    }
}

/// Abstraction to allow for `SymbolKind` to be made procedurally in a composed manner
#[derive(Debug)]
pub enum InstantiationSymbolKind {
    Namespace(InstantiationNamespace),
    Variable(InstantiationVariable),
    Type(InstantiationType),
    ExternType(ExternPlatformType),
}

/// Compile-time schema for namespaces
#[derive(Debug)]
pub struct InstantiationNamespace {
    pub ns: &'static [InstantiationSymbolBase],
}

impl InstantiationNamespace {
    pub const fn new(ns: &'static [InstantiationSymbolBase]) -> Self {
        Self { ns }
    }
}

/// Compile-time schema for variables
#[derive(Debug)]
pub struct InstantiationVariable {
    // It needs a symbol to exist so I'd presume this is ok since the symbol would have the
    // identifier already
    // pub name_id: InternedIdUTF8,
    pub ty: InstantiationTypeKind,
    pub val: InstantiationValueKind,
}

impl InstantiationVariable {
    pub const fn new(ty: InstantiationTypeKind, val: InstantiationValueKind) -> Self {
        Self { ty, val }
    }
}

/// Compile-time schema for types
#[derive(Debug)]
pub struct InstantiationType {
    pub kind: InstantiationTypeKind,
    /// `None` and an empty namespace both register a type without an associated scope.
    pub ns: Option<InstantiationNamespace>,
}

impl InstantiationType {
    pub const fn new(kind: InstantiationTypeKind, ns: Option<InstantiationNamespace>) -> Self {
        Self { kind, ns }
    }
}

/// Abstraction to allow for `Type` to be made procedurally in a composed manner
#[derive(Debug, Clone)]
pub enum InstantiationTypeKind {
    BuiltinType(BuiltinType),
    Boundary(TypeBoundaryFlags),
}

/// Compile-time schema for values
#[derive(Debug, Clone)]
pub enum InstantiationValueKind {
    BigInt(IBig),
    BigFloat(DBig),
    I64(i64),
    U64(u64),
    F64(f64),
    Bool(bool),
    Char(char),
    // Function!
    // Func(SymbolId),
    Tuple(Vec<InstantiationValueKind>),
    Array(Vec<InstantiationValueKind>),
    Str(InternedIdUTF8),
}

impl InstantiationValueKind {
    /// Converts itself to the `Value` type
    pub fn to_val(&self) -> Value {
        match self {
            InstantiationValueKind::BigInt(val) => {
                Value::ArbitraryInt(ArbitraryIntKind::BigInt(val.clone()))
            }
            //NOTE: Not. Not. BitNot.
            InstantiationValueKind::BigFloat(val) => {
                Value::ArbitraryFloat(ArbitraryFloatKind::BigFloat(val.clone()))
            }
            InstantiationValueKind::I64(val) => Value::ArbitraryInt(ArbitraryIntKind::I64(*val)),
            InstantiationValueKind::U64(val) => Value::ArbitraryInt(ArbitraryIntKind::U64(*val)),
            InstantiationValueKind::F64(val) => {
                Value::ArbitraryFloat(ArbitraryFloatKind::F64(*val))
            }
            InstantiationValueKind::Bool(b) => Value::Bool(*b),
            InstantiationValueKind::Char(c) => Value::Char(*c),
            InstantiationValueKind::Str(id) => Value::InternedStr(*id),
            InstantiationValueKind::Tuple(_) | InstantiationValueKind::Array(_) => unreachable!(),
        }
    }
}
