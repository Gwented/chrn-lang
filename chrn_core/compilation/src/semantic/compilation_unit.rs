use chrn_utils::id_types::{ImplId, SymbolId, id_tags::TaggedId};

use crate::id_tag_decls::{
    AliasTag, ConfigRootTag, DirectivePreprocessTag, DirectiveTag, EnumTag, StructTag, TypeDefTag,
    VarTag,
};

/// Represents all possible top-level user compilation units
#[derive(Debug, Copy, Clone)]
pub enum CompilationUnit {
    TypeDef(TaggedId<SymbolId, TypeDefTag>),
    Struct(TaggedId<SymbolId, StructTag>),
    Enum(TaggedId<SymbolId, EnumTag>),
    Alias(TaggedId<SymbolId, AliasTag>),
    Var(TaggedId<SymbolId, VarTag>),
    DirectivePreprocess(TaggedId<ImplId, DirectivePreprocessTag>),
    ConfigRoot(TaggedId<ImplId, ConfigRootTag>),
}
