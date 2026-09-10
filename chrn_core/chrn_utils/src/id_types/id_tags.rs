use std::marker::PhantomData;

use crate::id_types::ArenaIndex;
// The trait could just be defined in utils as well as the struct
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaggedId<I: ArenaIndex, T: ArenaIndexTag> {
    // Maybe this should be a method call so that it's more like `unsafe` in ease of search
    pub inner: I,
    _phantom_data: PhantomData<T>,
}

impl<I: ArenaIndex, T: ArenaIndexTag> TaggedId<I, T> {
    pub const fn new(inner: I) -> Self {
        Self {
            inner,
            _phantom_data: PhantomData,
        }
    }

    /// Converts self into `self.inner`
    pub const fn into_inner(self) -> I {
        self.inner
    }
}

pub trait ArenaIndexTag: Copy {}

/// Helper macro for tag declarations
#[macro_export]
macro_rules! tag_decl {
    ($($ident:ident),* $(,)?) => {
        $(
        /// Tag for restricting `ArenaIndex`'s that have broad applications.
    #[repr(transparent)]
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
        pub struct $ident;
        impl ArenaIndexTag for $ident {}
        )*
    }
}

// Is this ok?
/// Clones a given `Tagged<T,I>` into a `Vec` with only `I`
pub fn clone_vec_untagged<I: ArenaIndex, T: ArenaIndexTag>(tagged: &[TaggedId<I, T>]) -> Vec<I> {
    tagged.iter().map(|tag| tag.inner).collect()
}
