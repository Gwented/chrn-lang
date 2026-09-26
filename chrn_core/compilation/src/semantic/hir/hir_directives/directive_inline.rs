use chrn_utils::{id_types::InternedId, intern};
use lang::{
    chrn_classifier::{ChrnClassifiable, ChrnClassified},
    types::{boundaries::TypeBoundaryFlags, builtins::BuiltinType},
};

/// General directives not specific to anything
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveInline {
    Warn,
    Ignore,
    Type(TypeDirective),
}

impl From<TypeDirective> for DirectiveInline {
    fn from(val: TypeDirective) -> Self {
        DirectiveInline::Type(val)
    }
}

impl DirectiveInline {
    // Duplicating pathing
    pub const fn is_directive(interned_id: InternedId) -> bool {
        todo!()
    }

    /// If `self` has any form of restrictions, returns `true`
    /// Otherwise `false`
    pub fn has_restrictions(self) -> bool {
        match self {
            DirectiveInline::Warn | DirectiveInline::Ignore => false,
            DirectiveInline::Type(d) => d.has_restrictions(),
        }
    }
    pub fn supports_builtin_type(self, builtin_type: &BuiltinType) -> bool {
        match self {
            DirectiveInline::Warn | DirectiveInline::Ignore => true,
            DirectiveInline::Type(d) => d.supports_builtin_type(builtin_type),
        }
    }

    pub const fn boundaries(self) -> TypeBoundaryFlags {
        match self {
            DirectiveInline::Warn | DirectiveInline::Ignore => TypeBoundaryFlags::all(),
            DirectiveInline::Type(d) => d.boundaries(),
        }
    }

    pub const fn try_from_interned_str(interned_id: InternedId) -> Option<DirectiveInline> {
        match interned_id.id {
            intern::INTERNED_WARN => Some(DirectiveInline::Warn),
            intern::INTERNED_IGNORE => Some(DirectiveInline::Ignore),
            // Ok this looks confusing
            _ => {
                let Some(ty_direct) = TypeDirective::try_from_interned_str(interned_id) else {
                    return None;
                };
                Some(DirectiveInline::Type(ty_direct))
            }
        }
    }
}

impl ChrnClassifiable for DirectiveInline {
    fn to_classified(&self) -> lang::chrn_classifier::ChrnClassified {
        match self {
            DirectiveInline::Warn => ChrnClassified::DirectiveWarn,
            DirectiveInline::Ignore => ChrnClassified::DirectiveIgnore,
            DirectiveInline::Type(type_directive) => type_directive.to_classified(),
        }
    }
}

// May make it split to where there's compiler directive, type directive, but keepign it as this
// for now
/// Directives specific to types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeDirective {
    Scient,
    Hex,
    Bin,
    Octal,
    Unicode,
}

impl TypeDirective {
    // has_restrictions?
    /// Returns true if the given argument is applicable to every type, such as `#warn`, otherwise
    /// returns false
    pub fn has_restrictions(self) -> bool {
        match self {
            TypeDirective::Scient
            | TypeDirective::Hex
            | TypeDirective::Bin
            | TypeDirective::Unicode
            | TypeDirective::Octal => true,
        }
    }

    /// This MUST be used after ensuring the type is a primitive, not a data structure.
    // Maybe this is a good time to use kind
    pub fn supports_builtin_type(self, builtin_type: &BuiltinType) -> bool {
        match self {
            TypeDirective::Scient
            | TypeDirective::Hex
            | TypeDirective::Bin
            | TypeDirective::Octal => {
                match builtin_type {
                    BuiltinType::I8
                    | BuiltinType::U8
                    | BuiltinType::I16
                    | BuiltinType::U16
                    | BuiltinType::F16
                    | BuiltinType::I32
                    | BuiltinType::U32
                    | BuiltinType::F32
                    | BuiltinType::I64
                    | BuiltinType::U64
                    | BuiltinType::F64
                    | BuiltinType::I128
                    | BuiltinType::U128
                    | BuiltinType::F128
                    | BuiltinType::Sized
                    | BuiltinType::BigInt
                    | BuiltinType::BigFloat
                    | BuiltinType::Unsized
                    //NOTE: Checks this at runtime
                    |BuiltinType::Runtime => true,
                    //FIXME: This seems like an odd abstraction to accept!
                    //
                    // Maybe this means that it shouldn't be a method, it should be a function that
                    // has access to their inner, which can do the rolving. Rolving.
                    //
                    // This is unreachable because when arguments are resolved, it requires the
                    // data structures to be recursively resolved into a builtin type
                    BuiltinType::List(_)
                    |BuiltinType::Set(_)
                    |BuiltinType::Map(_, _) => unreachable!("ConstraintResolver broke"),
                    _ => false,
                }
            }
            TypeDirective::Unicode => match builtin_type {
                BuiltinType::Char | BuiltinType::Runtime => true,
                _ => false,
            },
        }
    }

    pub const fn boundaries(self) -> TypeBoundaryFlags {
        match self {
            TypeDirective::Scient
            | TypeDirective::Hex
            | TypeDirective::Bin
            | TypeDirective::Octal => TypeBoundaryFlags::NUMERIC,
            TypeDirective::Unicode => TypeBoundaryFlags::CHAR,
        }
    }

    pub const fn try_from_interned_str(interned_id: InternedId) -> Option<TypeDirective> {
        match interned_id.id {
            intern::INTERNED_SCIENT => Some(TypeDirective::Scient),
            intern::INTERNED_HEX => Some(TypeDirective::Hex),
            intern::INTERNED_BIN => Some(TypeDirective::Bin),
            intern::INTERNED_OCTAL => Some(TypeDirective::Octal),
            intern::INTERNED_UNICODE => Some(TypeDirective::Unicode),
            _ => None,
        }
    }
}

impl ChrnClassifiable for TypeDirective {
    fn to_classified(&self) -> ChrnClassified {
        match self {
            TypeDirective::Scient => ChrnClassified::DirectiveScient,
            TypeDirective::Hex => ChrnClassified::DirectiveHex,
            TypeDirective::Bin => ChrnClassified::DirectiveBin,
            TypeDirective::Octal => ChrnClassified::DirectiveOctal,
            TypeDirective::Unicode => ChrnClassified::DirectiveUnicode,
        }
    }
}
