//TODO: Maybe move all of this to compilation since these are compilation implementations. Nothinga
//bout this is generararlajral
use std::{fmt::Debug, hash::Hash};

use crate::{
    id_types::id_tags::{ArenaIndexTag, TaggedId},
    intern,
};
pub mod id_tags;

//TODO: And this
/// A source ID proven to refer to a path.
pub type PathId = TaggedId<SourceId, crate::source_utils::source_manager::PathTag>;

//TODO: Move this
/// An interned ID whose bytes are UTF-8 text.
pub type InternedIdUTF8 = TaggedId<InternedId, intern::UTF8Tag>;

/// Represents a type that can be turned into `usize`
///
/// Specifically for `Arena<T, I>`
pub trait ArenaIndex: Copy {
    fn into_usize(self) -> usize;
    fn from_usize(val: usize) -> Self;
}

/// Convenience for `u32` containing implementers of `ArenaIndex`
macro_rules! arena_idx_impl_u32 {
    ($($t:ty),* $(,)?) => {
        $(
            impl ArenaIndex for $t {
                fn into_usize(self) -> usize {
                    self.id as usize
                }

                fn from_usize(val: usize) -> Self {
                    Self{id: val as u32}
                }
            }
        )*
    }
}

/// Convenience for `u16` containing implementers of `ArenaIndex`
macro_rules! arena_idx_impl_u16 {
    ($($t:ty),* $(,)?) => {
        $(
            impl ArenaIndex for $t {
                fn into_usize(self) -> usize {
                    self.id as usize
                }

                fn from_usize(val:usize) -> Self {
                    Self{id:val as u16}
                }
            }
        )*
    }
}

// This is most definitely too specific and should have some ids declared in compilation
arena_idx_impl_u32!(
    InternedId,
    SourceRegionId,
    TypeId,
    VariableId,
    ConfigRootId,
    DirectiveId,
    AstId,
    MemberId,
    SymbolId,
    ExprId,
    ModuleId,
    ValueId,
    ImplId,
    ImplMemberId,
    ExternTypeId,
    SourceId,
);

// All id type instantiations of u16
arena_idx_impl_u16!(ScopeId);

macro_rules! id_type_impl_u32 {
    ($($ident:ident),* $(,)?) => {
        $(
        /// Type-safe wrapper for using an index that contains a valid id
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
        pub struct $ident {
            pub id: u32
        }

        impl $ident {
            pub const fn new(id: u32) -> Self {
                Self { id }
            }
            pub const fn into_tagged<T: ArenaIndexTag>(self) -> TaggedId<Self, T> {
                TaggedId::new(self)
            }
        }
        )*
    }
}

macro_rules! id_type_impl_u16 {
    ($($ident:ident),* $(,)?) => {
        $(
        /// Type-safe wrapper for using an index that contains a valid id
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
        pub struct $ident {
            pub id: u16
        }

        impl $ident {
            pub const fn new(id: u16) -> Self {
                Self { id }
            }
        }
        )*
    }
}

// All id type instantiations of u32
id_type_impl_u32!(
    InternedId,
    SourceRegionId,
    TypeId,
    ExternTypeId,
    VariableId,
    ConfigRootId,
    SymbolId,
    DirectiveId,
    ModuleId,
    AstId,
    MemberId,
    ExprId,
    ValueId,
    ImplId,
    ImplMemberId,
    SourceId,
);
// All id type instantiations of u16
id_type_impl_u16!(ScopeId);
