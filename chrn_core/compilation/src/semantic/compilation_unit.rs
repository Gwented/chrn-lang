use chrn_utils::id_types::{
    ArenaIndex, ImplId, SymbolId,
    id_tags::{ArenaIndexTag, TaggedId},
};

//NOTE: Could attach kind directly
/// Represents all possible user compilation units
#[derive(Debug, Copy, Clone)]
pub enum CompilationUnit {
    Symbol(SymbolId),
    Impl(ImplId),
}
