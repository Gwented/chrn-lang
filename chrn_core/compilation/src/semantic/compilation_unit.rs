use chrn_utils::id_types::{ImplId, SymbolId, id_tags::TaggedId};

use crate::id_tag_decls::{AliasTag, ConfigRootTag, EnumTag, StructTag, TypeDefTag, VarTag};

/// Represents all possible user compilation units
#[derive(Debug, Copy, Clone)]
pub enum CompilationUnit {
    TypeDef(TaggedId<SymbolId, TypeDefTag>),
    Struct(TaggedId<SymbolId, StructTag>),
    Enum(TaggedId<SymbolId, EnumTag>),
    Alias(TaggedId<SymbolId, AliasTag>),
    Var(TaggedId<SymbolId, VarTag>),
    ConfigRoot(TaggedId<ImplId, ConfigRootTag>),
}
